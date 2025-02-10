use csv::{ReaderBuilder, Trim};
use std::{
    iter::Iterator,
    path::Path,
    slice::{Iter, IterMut},
};

#[allow(unused_imports)]
use crate::models::{
    bar::{Bar, BarChart},
    line::{Line, LineGraph},
    stacked_bar::{StackedBar, StackedBarChart},
    Point, Scale, ScaleKind,
};

mod utils;
pub use utils::*;

pub use error::*;

mod arraytext;
pub use arraytext::*;

mod arrayi32;
pub use arrayi32::*;

mod arrayu32;
pub use arrayu32::*;

mod arrayisize;
pub use arrayisize::*;

mod arrayusize;
pub use arrayusize::*;

mod arrayf32;
pub use arrayf32::*;

mod arrayf64;
pub use arrayf64::*;

mod arraybool;
pub use arraybool::*;

use super::builders::SheetBuilder;
use super::utils::{ColumnType as CT, HeaderLabelStrategy, HeaderTypesStrategy};

#[derive(Debug, Clone, Copy, PartialEq)]
enum ColumnType {
    None,
    Infer,
    Type(CT),
}

struct StrategyIter {
    strat: HeaderTypesStrategy,
    idx: usize,
}

impl StrategyIter {
    fn new(value: HeaderTypesStrategy) -> Self {
        Self {
            strat: value,
            idx: 0,
        }
    }
}

impl Iterator for StrategyIter {
    type Item = ColumnType;

    fn next(&mut self) -> Option<Self::Item> {
        let idx = self.idx;
        self.idx += 1;

        match &self.strat {
            HeaderTypesStrategy::Provided(headers) => {
                headers.get(idx).copied().map(|ct| ColumnType::Type(ct))
            }
            HeaderTypesStrategy::None => Some(ColumnType::None),
            HeaderTypesStrategy::Infer => Some(ColumnType::Infer),
        }
    }
}

pub struct ColumnSheet {
    /// The Columns in the sheet. All columns are guaranteed to have the same
    /// height
    columns: Vec<Box<dyn Column>>,
    /// The primary column of the sheet. Is None if the sheet is empty.
    primary: Option<usize>,
}

impl ColumnSheet {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        Self::from_builder(SheetBuilder::new(path))
    }

    pub fn from_builder<P: AsRef<Path>>(builder: SheetBuilder<P>) -> Result<Self> {
        let SheetBuilder {
            path,
            primary,
            trim,
            flexible,
            delimiter,
            label_strategy,
            type_strategy,
        } = builder;

        let trim = if trim { Trim::All } else { Trim::None };
        let has_headers = label_strategy == HeaderLabelStrategy::ReadLabels;

        let mut rdr = ReaderBuilder::new()
            .has_headers(has_headers)
            .trim(trim)
            .delimiter(delimiter)
            .flexible(flexible)
            .from_path(path)?;

        let (mut cols, max_rows) = {
            let mut cols: Vec<Vec<String>> = Vec::default();
            let mut row_len = 0;

            for (rows, record) in rdr.records().enumerate() {
                let record = record?;
                let len = record.len();

                for (idx, record) in record.into_iter().enumerate() {
                    let record = record.to_owned();

                    match cols.get_mut(idx) {
                        Some(col) => col.push(record),
                        None => {
                            let mut col = vec![String::default(); rows];
                            col.push(record);
                            cols.push(col);
                        }
                    };
                }

                if len > row_len {
                    row_len = len
                }
            }
            (cols, row_len)
        };

        cols.iter_mut()
            .for_each(|col| col.resize_with(max_rows, Default::default));

        let mut headers = match label_strategy {
            HeaderLabelStrategy::NoLabels => vec![None; cols.len()],
            HeaderLabelStrategy::Provided(headers) => headers.into_iter().map(Some).collect(),
            HeaderLabelStrategy::ReadLabels => rdr
                .headers()?
                .into_iter()
                .map(|header| {
                    if header.is_empty() {
                        None
                    } else {
                        Some(header.to_owned())
                    }
                })
                .collect(),
        };

        let longest = usize::max(cols.len(), headers.len());
        headers.resize_with(longest, Default::default);
        cols.resize_with(longest, Default::default);

        let columns: Vec<Box<dyn Column>> = Self::create_columns(cols, headers, type_strategy);
        let primary = if columns.is_empty() {
            None
        } else {
            Some(primary)
        };

        Ok(Self { columns, primary })
    }

    /// Constructs columns from inputs. Expects the length of `cols` and
    /// `headers` to be the same
    fn create_columns(
        cols: Vec<Vec<String>>,
        headers: Vec<Option<String>>,
        type_strategy: HeaderTypesStrategy,
    ) -> Vec<Box<dyn Column>> {
        // Dropping extra unused headers is most likely okay so the less than
        // comparison is okay.
        assert!(
            cols.len() >= headers.len(),
            "Column construction assertion failed"
        );

        let strategies = StrategyIter::new(type_strategy);

        cols.into_iter()
            .zip(headers)
            .zip(strategies)
            .map(|((col, header), kind)| parse_column(col, header, kind))
            .collect()
    }

    /// Returns an iterator over the columns of the sheet.
    pub fn iter(&self) -> Iter<'_, Box<dyn Column>> {
        self.columns.iter()
    }

    /// Returns an iterator over the headers of the [`ColumnSheet`].
    pub fn headers(&self) -> impl Iterator<Item = ColumnHeader<'_>> {
        self.columns.iter().map(|col| {
            let header = ColumnHeader {
                header: col.label(),
                kind: col.kind(),
            };
            header
        })
    }

    /// Sets the header of the column at `col` to `header`.
    pub fn set_col_header(&mut self, col: usize, header: impl Into<String>) {
        if let Some(column) = self.columns.get_mut(col) {
            column.set_header(header.into())
        }
    }

    /// Returns the width of the [`ColumnSheet`].
    ///
    /// This is essentially the same as the number of [`Column`]s in the sheet.
    pub fn width(&self) -> usize {
        self.columns.len()
    }

    /// Returns the height of the [`ColumnSheet`].
    ///
    /// All [`Column`]s within a [`ColumnSheet`] are guaranteed to have the same
    /// height. Shorter columns are padded with null to achieve this.
    pub fn height(&self) -> usize {
        self.columns.first().map_or(0, |col| col.len())
    }

    /// Sets the primary column of the [`ColumnSheet`].
    ///
    /// If `primary` is invalid, no change is made.
    pub fn set_primary(&mut self, primary: usize) -> Result<()> {
        if primary >= self.width() {
            return Err(Error::InvalidPrimary(primary));
        }

        self.primary = Some(primary);

        Ok(())
    }

    /// Returns a shared reference to the column at `idx`, if any.
    pub fn get_col(&self, idx: usize) -> Option<&dyn Column> {
        self.columns.get(idx).map(|boxed| boxed.as_ref())
    }

    /// Returns an exclusive reference to the column at `idx` if any.
    pub fn get_col_mut(&mut self, idx: usize) -> Option<&mut Box<dyn Column>> {
        self.columns.get_mut(idx)
    }

    /// Returns true if the [`ColumnSheet`] is empty.
    pub fn is_empty(&self) -> bool {
        self.columns.is_empty()
    }

    /// Appends a column to the back of the [`ColumnSheet`]
    ///
    /// No append occurs if `column` is not of the same height as `Self`.
    pub fn push_col(&mut self, column: Box<dyn Column>) -> Result<()> {
        self.insert_col(column, self.width())
    }

    /// Appends a row to the back of the [`ColumnSheet`]
    ///
    /// No append occurs if `row` is not of the same width as `Self`.
    pub fn push_row<I, R>(&mut self, row: R) -> Result<()>
    where
        I: AsRef<str>,
        R: ExactSizeIterator<Item = I>,
    {
        self.insert_row(row, self.height())
    }

    /// Removes the column at `idx` shifting all values to the left
    ///
    /// No remove occurs if `idx` is invalid
    pub fn remove_col(&mut self, idx: usize) -> Result<()> {
        let primary = self.primary.ok_or(Error::InvalidColumn(idx))?;

        self.columns.remove(idx);

        if self.is_empty() {
            self.primary = None;
        } else if idx < primary {
            self.primary = Some(primary - 1);
        } else if idx == primary && primary != 0 {
            self.primary = Some(primary - 1);
        }

        Ok(())
    }

    /// Removes the row at `idx` shifting all values to the up
    ///
    /// No remove occurs if `idx` is invalid
    pub fn remove_row(&mut self, idx: usize) -> Result<()> {
        if idx >= self.height() {
            return Err(Error::InvalidRow(idx));
        }

        self.columns
            .iter_mut()
            .for_each(|column| column.remove(idx));

        Ok(())
    }

    /// Inserts a column at `idx` shifting all values after right
    ///
    /// No insertion occurs if `column` has a different height than `Self`.
    pub fn insert_col(&mut self, column: Box<dyn Column>, idx: usize) -> Result<()> {
        let other = column.len();
        let own = self.height();

        if other != own && !self.is_empty() {
            return Err(Error::InvalidColumnHeight { own, other });
        }

        self.columns.insert(idx, column);

        if self.width() == 1 {
            self.primary = Some(0);
            return Ok(());
        }
        // self.primary is always a Some, unless self is empty. If self was
        // empty before insertion, the check right above would have caught that.
        // This is unwrap is safe.
        let primary = self.primary.unwrap();

        if idx <= primary {
            self.primary = Some(primary + 1);
        }

        Ok(())
    }

    /// Inserts a row at `idx` shifting all values after down
    ///
    /// No insertion occurs if `row` has a different width than `Self`.
    pub fn insert_row<I, R>(&mut self, row: R, idx: usize) -> Result<()>
    where
        I: AsRef<str>,
        R: ExactSizeIterator<Item = I>,
    {
        let own = self.width();
        let other = row.len();

        if other != own && !self.is_empty() {
            return Err(Error::InvalidRowWidth { own, other });
        }

        if self.is_empty() {
            let cols = row
                .map(|value| vec![value.as_ref().to_owned()])
                .collect::<Vec<Vec<String>>>();
            let len = cols.len();
            let columns = Self::create_columns(cols, vec![None; len], HeaderTypesStrategy::Infer);

            self.columns = columns;

            if len != 0 {
                self.primary = Some(0);
            }
        } else {
            self.columns
                .iter_mut()
                .zip(row)
                .for_each(|(column, value)| column.insert(value.as_ref(), idx));
        }

        Ok(())
    }

    /// Swaps the columns at `x` with those at `y`.
    ///
    /// Values are left unchanged if any one of the indices are invalid
    pub fn swap_cols(&mut self, x: usize, y: usize) -> Result<()> {
        if x >= self.width() {
            return Err(Error::InvalidColumn(x));
        }

        if y >= self.width() {
            return Err(Error::InvalidColumn(y));
        }

        self.columns.swap(x, y);

        if let Some(primary) = self.primary {
            if x == primary {
                self.primary = Some(y)
            } else if y == primary {
                self.primary = Some(x)
            }
        }

        Ok(())
    }

    /// Swaps the values at row `x` with those at row `y`.
    ///
    /// Values are left unchanged if any one of the indices are invalid
    pub fn swap_rows(&mut self, x: usize, y: usize) -> Result<()> {
        let height = self.height();

        if x >= height {
            return Err(Error::InvalidRow(x));
        }

        if y >= height {
            return Err(Error::InvalidRow(y));
        }

        self.columns.iter_mut().for_each(|col| col.swap(x, y));

        Ok(())
    }
}

fn parse_column(col: Vec<String>, header: Option<String>, strategy: ColumnType) -> Box<dyn Column> {
    let text = |col: Vec<String>, header: Option<String>| {
        let mut array = ArrayText::parse_str(&col);
        if let Some(header) = header {
            array.set_header(header);
        }
        Box::new(array)
    };

    match strategy {
        ColumnType::None => text(col, header),

        ColumnType::Infer => {
            if let Some(mut array) = ArrayI32::parse_str(&col) {
                if let Some(header) = header {
                    array.set_header(header);
                }
                return Box::new(array);
            };

            if let Some(mut array) = ArrayU32::parse_str(&col) {
                if let Some(header) = header {
                    array.set_header(header);
                }
                return Box::new(array);
            };

            if let Some(mut array) = ArrayISize::parse_str(&col) {
                if let Some(header) = header {
                    array.set_header(header);
                }
                return Box::new(array);
            };

            if let Some(mut array) = ArrayUSize::parse_str(&col) {
                if let Some(header) = header {
                    array.set_header(header);
                }
                return Box::new(array);
            };

            if let Some(mut array) = ArrayBool::parse_str(&col) {
                if let Some(header) = header {
                    array.set_header(header);
                }
                return Box::new(array);
            };

            if let Some(mut array) = ArrayF32::parse_str(&col) {
                if let Some(header) = header {
                    array.set_header(header);
                }
                return Box::new(array);
            };

            if let Some(mut array) = ArrayF64::parse_str(&col) {
                if let Some(header) = header {
                    array.set_header(header);
                }
                return Box::new(array);
            };

            text(col, header)
        }

        ColumnType::Type(CT::None) | ColumnType::Type(CT::Text) => text(col, header),

        ColumnType::Type(CT::Integer) => {
            if let Some(mut array) = ArrayI32::parse_str(&col) {
                if let Some(header) = header {
                    array.set_header(header);
                }
                return Box::new(array);
            };

            if let Some(mut array) = ArrayU32::parse_str(&col) {
                if let Some(header) = header {
                    array.set_header(header);
                }
                return Box::new(array);
            };

            text(col, header)
        }

        ColumnType::Type(CT::Number) => {
            if let Some(mut array) = ArrayISize::parse_str(&col) {
                if let Some(header) = header {
                    array.set_header(header);
                }
                return Box::new(array);
            };

            if let Some(mut array) = ArrayUSize::parse_str(&col) {
                if let Some(header) = header {
                    array.set_header(header);
                }
                return Box::new(array);
            };

            text(col, header)
        }

        ColumnType::Type(CT::Float) => {
            if let Some(mut array) = ArrayF32::parse_str(&col) {
                if let Some(header) = header {
                    array.set_header(header);
                }
                return Box::new(array);
            };

            if let Some(mut array) = ArrayF64::parse_str(&col) {
                if let Some(header) = header {
                    array.set_header(header);
                }
                return Box::new(array);
            };

            text(col, header)
        }

        ColumnType::Type(CT::Boolean) => {
            if let Some(mut array) = ArrayBool::parse_str(&col) {
                if let Some(header) = header {
                    array.set_header(header);
                }
                return Box::new(array);
            };

            text(col, header)
        }
    }
}

mod error {
    use csv::Error as CSVError;
    use std::{error, fmt};

    #[derive(Debug)]
    pub enum Error {
        CSV(CSVError),
        InvalidColumn(usize),
        InvalidRow(usize),
        InvalidPrimary(usize),
        InvalidColumnHeight { own: usize, other: usize },
        InvalidRowWidth { own: usize, other: usize },
    }

    impl From<CSVError> for Error {
        fn from(value: CSVError) -> Self {
            Self::CSV(value)
        }
    }

    impl fmt::Display for Error {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::CSV(error) => error.fmt(f),
                Self::InvalidRow(row) => write!(f, "Invalid Row at {row}"),
                Self::InvalidColumn(col) => write!(f, "Invalid Column at {col}"),
                Self::InvalidPrimary(primary) => write!(f, "Invalid Primary Column at {primary}"),
                Self::InvalidColumnHeight { own, other } => {
                    write!(f, "Invalid Column height of {other} instead of {own}")
                }
                Self::InvalidRowWidth { own, other } => {
                    write!(f, "Invalid Row width of {other} instead of {own}")
                }
            }
        }
    }

    impl error::Error for Error {
        fn source(&self) -> Option<&(dyn error::Error + 'static)> {
            if let Self::CSV(error) = self {
                error.source()
            } else {
                None
            }
        }
    }

    /// A short hand alias for `Sheet` error results
    pub type Result<T> = core::result::Result<T, Error>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn temp() {
        //let path = "./dummies/csv/flexible.csv";
        let path = "./dummies/csv/empty.csv";

        let builder = SheetBuilder::new(path)
            .types(HeaderTypesStrategy::Infer)
            .labels(HeaderLabelStrategy::ReadLabels)
            .flexible(false)
            .trim(true);

        let sht = ColumnSheet::from_builder(builder).unwrap();

        for column in sht.iter() {
            dbg!(column);
        }
    }
}

use csv::{ReaderBuilder, Trim};
use std::{
    fmt::Debug,
    path::Path,
    slice::{Iter, IterMut},
    str::FromStr,
};

#[allow(unused_imports)]
use crate::models::{
    bar::{Bar, BarChart},
    line::{Line, LineGraph},
    stacked_bar::{StackedBar, StackedBarChart},
    Point, Scale, ScaleKind,
};

mod arraytext;
pub use arraytext::*;

mod arrayi32;
pub use arrayi32::*;

mod arrayisize;
pub use arrayisize::*;

use super::builders::SheetBuilder;
use super::utils::{HeaderLabelStrategy, HeaderTypesStrategy};

use private::Sealed;

const NULL: &str = "<null>";

/// Data types supported by the current implementation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DataType {
    Int32,
    UInt32,
    ISize,
    USize,
    Boolean,
    F32,
    F64,
    Text,
}

pub trait Column: Sealed + Debug {
    fn label(&self) -> Option<&String>;

    fn kind(&self) -> DataType;

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn set_header(&mut self, header: String);

    /// Overwrites the value at `idx` with successfully parsed `value`. If
    /// parsing fails, `idx` is left as-is.
    fn set_position<'a>(&mut self, value: &'a str, idx: usize);

    /// Swaps the value at `x` with that at `y`.
    ///
    /// No swap occurs if either `x` or `y` are invalid indices.
    fn swap(&mut self, x: usize, y: usize);
}

#[derive(Debug)]
pub struct ColumnHeader<'a> {
    pub header: Option<&'a String>,
    pub kind: DataType,
}

pub struct ColumnSheet {
    /// The Columns in the sheet. All columns are guaranteed to have the same
    /// height
    columns: Vec<Box<dyn Column>>,
    /// The primary column of the sheet. Is None if the sheet is empty.
    primary: Option<usize>,
}

impl ColumnSheet {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Self {
        Self::from_builder(SheetBuilder::new(path))
    }

    pub fn from_builder<P: AsRef<Path>>(builder: SheetBuilder<P>) -> Self {
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
            .from_path(path)
            .unwrap();

        let mut cols = {
            let mut cols: Vec<Vec<String>> = Vec::default();
            let mut row_len = 0;

            for (rows, record) in rdr.records().enumerate() {
                let record = record.unwrap();
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

                for col in len..row_len {
                    let col = cols.get_mut(col).expect("Cannot see this failing. lol");
                    col.push(String::default())
                }

                if len > row_len {
                    row_len = len
                }
            }
            cols
        };

        let mut headers = match label_strategy {
            HeaderLabelStrategy::NoLabels => vec![None; cols.len()],
            HeaderLabelStrategy::Provided(headers) => headers.into_iter().map(Some).collect(),
            HeaderLabelStrategy::ReadLabels => rdr
                .headers()
                .unwrap()
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

        Self { columns, primary }
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

        cols.into_iter()
            .zip(headers)
            // Full implementation should not clone strategy
            .map(|(col, header)| parse_column(col, header, type_strategy.clone()))
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
    pub fn set_primary(&mut self, primary: usize) {
        if primary >= self.width() {
            return;
        }

        self.primary = Some(primary);
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
    pub fn push_col(&mut self, column: Box<dyn Column>) {
        self.insert_col(column, self.width())
    }

    /// Appends a row to the back of the [`ColumnSheet`]
    ///
    /// No append occurs if `row` is not of the same width as `Self`.
    pub fn push_row<I, R>(&mut self, row: R)
    where
        I: AsRef<str>,
        R: ExactSizeIterator<Item = I>,
    {
        self.insert_row(row, self.height());
    }

    /// Removes the column at `idx` shifting all values to the left
    ///
    /// No remove occurs if `idx` is invalid
    pub fn remove_col(&mut self, idx: usize) {
        if idx >= self.width() {
            return;
        }
        // Guaranteed by index check above
        let primary = self.primary.unwrap();

        self.columns.remove(idx);

        if self.is_empty() {
            self.primary = None;
        } else if idx < primary {
            self.primary = Some(primary - 1);
        } else if idx == primary && primary != 0 {
            self.primary = Some(primary - 1);
        }
    }

    /// Removes the row at `idx` shifting all values to the up
    ///
    /// No remove occurs if `idx` is invalid
    pub fn remove_row(&mut self, idx: usize) {
        if idx >= self.height() {
            return;
        }

        self.columns
            .iter_mut()
            .for_each(|column| column.remove(idx));
    }

    /// Inserts a column at `idx` shifting all values after right
    ///
    /// No insertion occurs if `column` has a different height than `Self`.
    pub fn insert_col(&mut self, column: Box<dyn Column>, idx: usize) {
        if column.len() != self.height() && !self.is_empty() {
            return;
        }

        self.columns.insert(idx, column);

        if self.width() == 1 {
            self.primary = Some(0);
            return;
        }
        // self.primary is always a Some, unless self is empty. If self was
        // empty before insertion, the check right above would have caught that.
        // This is unwrap is safe.
        let primary = self.primary.unwrap();

        if idx <= primary {
            self.primary = Some(primary + 1);
        }
    }

    /// Inserts a row at `idx` shifting all values after down
    ///
    /// No insertion occurs if `row` has a different width than `Self`.
    pub fn insert_row<I, R>(&mut self, row: R, idx: usize)
    where
        I: AsRef<str>,
        R: ExactSizeIterator<Item = I>,
    {
        if row.len() != self.width() && !self.is_empty() {
            return;
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
    }

    /// Swaps the columns at `x` with those at `y`.
    ///
    /// Values are left unchanged if any one of the indices are invalid
    pub fn swap_cols(&mut self, x: usize, y: usize) {
        if x >= self.width() || y >= self.width() {
            return;
        }

        self.columns.swap(x, y);

        if let Some(primary) = self.primary {
            if x == primary {
                self.primary = Some(y)
            } else if y == primary {
                self.primary = Some(x)
            }
        }
    }

    /// Swaps the values at row `x` with those at row `y`.
    ///
    /// Values are left unchanged if any one of the indices are invalid
    pub fn swap_rows(&mut self, x: usize, y: usize) {
        let height = self.height();
        if x >= height || y >= height {
            return;
        }

        self.columns.iter_mut().for_each(|col| col.swap(x, y));
    }
}

mod private {
    pub trait Sealed {
        /// Pushes `value` to the end of the column by parsing `value`.
        ///
        /// Should parsing fail, a null value is pushed instead
        fn push(&mut self, value: &str);

        /// Removes the value at `idx` if any, shifting the remaning values up.
        fn remove(&mut self, idx: usize);

        /// Inserts successfully parsed `value` at `idx` shifting all elements after
        /// to the right.
        ///
        /// Should parsing fail, no insertion is made.
        fn insert(&mut self, value: &str, idx: usize);
    }
}

fn parse_column(
    col: Vec<String>,
    header: Option<String>,
    infer: HeaderTypesStrategy,
) -> Box<dyn Column> {
    match infer {
        HeaderTypesStrategy::None => {
            let mut array = ArrayText::parse_str(col);
            if let Some(header) = header {
                array.set_header(header);
            }
            Box::new(array)
        }
        HeaderTypesStrategy::Infer => {
            if let Some(mut array) = ArrayI32::parse_str(&col) {
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

            let mut array = ArrayText::parse_str(col);
            if let Some(header) = header {
                array.set_header(header);
            }

            Box::new(array)
        }
        HeaderTypesStrategy::Provided(_kinds) => {
            // Remember to set header
            todo!("Header Type Strategy")
        }
    }
}

/// Parses `input` into given type, taking note of both empty and null strings.
///
/// On error, `()` is returned.
fn parse_helper<T: FromStr>(input: &str) -> Result<Option<T>, ()> {
    if input.is_empty() || input == NULL {
        return Ok(None);
    }

    input.parse::<T>().map_err(|_err| {}).map(Some)
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

        let sht = ColumnSheet::from_builder(builder);

        for column in sht.iter() {
            dbg!(column);
        }
    }
}

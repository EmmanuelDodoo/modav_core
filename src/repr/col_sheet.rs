use csv::{ReaderBuilder, Trim};
use std::{
    iter::{ExactSizeIterator, Iterator},
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

mod col_tests;

use super::config::*;
use super::utils::{ColumnType as CT, TypesStrategy};

/// Wrapper type for [`ColumnType`] and [`TypesStrategy`].
#[derive(Debug, Clone, Copy, PartialEq)]
enum ColumnType {
    None,
    Infer,
    Type(CT),
}

struct StrategyIter {
    strat: TypesStrategy,
    idx: usize,
}

impl StrategyIter {
    fn new(value: TypesStrategy) -> Self {
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
            TypesStrategy::Provided(headers) => headers.get(idx).copied().map(ColumnType::Type),
            TypesStrategy::None => Some(ColumnType::None),
            TypesStrategy::Infer => Some(ColumnType::Infer),
        }
    }
}

pub struct ColumnSheet {
    /// The Columns in the [`ColumnSheet`]. All columns are guaranteed to have the same
    /// height
    columns: Vec<Box<dyn Column>>,
    /// The primary column of the [`ColumnSheet`]. Is None if the [`ColumnSheet`] is empty.
    primary: Option<usize>,
    /// The number of cells in a column
    height: usize,
    /// The string which should be considered null.
    null_string: String,
}

impl ColumnSheet {
    /// Constructs a [`ColumnSheet`] from the provided path using the default
    /// [`Config`].
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        Self::with_config(Config::new(path))
    }

    /// Constructs a [`ColumnSheet`] using a configured [`Config`].
    pub fn with_config<P: AsRef<Path>>(config: Config<P>) -> Result<Self> {
        let Config {
            path,
            primary,
            trim,
            flexible,
            delimiter,
            label_strategy,
            type_strategy,
            null_string,
        } = config;

        let trim = if trim { Trim::All } else { Trim::None };
        let has_headers = label_strategy == HeaderStrategy::ReadLabels;

        let mut rdr = ReaderBuilder::new()
            .has_headers(has_headers)
            .trim(trim)
            .delimiter(delimiter)
            .flexible(flexible)
            .from_path(path)?;

        let (mut cols, height) = {
            let mut cols: Vec<Vec<String>> = Vec::default();
            let mut rows = 0;
            let mut columns = 0;

            for (row, record) in rdr.records().enumerate() {
                let record = record?;
                rows += 1;
                let curr_cols = record.len();

                for (col, record) in record.into_iter().enumerate() {
                    let record = record.to_owned();

                    match cols.get_mut(col) {
                        Some(col) => col.push(record),
                        // If this record(row) is longer than previous, construct
                        // the a new column, fill it with the default value and
                        // then also push this row's value for the column.
                        None => {
                            let mut col = vec![String::default(); row];
                            col.push(record);
                            cols.push(col);
                        }
                    };
                }

                if curr_cols > columns {
                    columns = curr_cols
                } else {
                    // If a previous record(row) was longer than this one, fill
                    // it with the default value
                    for missing in curr_cols..columns {
                        if let Some(missing) = cols.get_mut(missing) {
                            missing.push(String::default())
                        }
                    }
                }
            }
            (cols, rows)
        };

        //cols.iter_mut()
        //    .for_each(|col| col.resize_with(height, Default::default));

        let mut headers = match label_strategy {
            HeaderStrategy::NoLabels => vec![None; cols.len()],
            HeaderStrategy::Provided(headers) => headers.into_iter().map(Some).collect(),
            HeaderStrategy::ReadLabels => rdr
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

        let columns: Vec<Box<dyn Column>> =
            Self::create_columns(cols, headers, type_strategy, &null_string);
        let primary = if columns.is_empty() {
            None
        } else {
            Some(primary)
        };

        Ok(Self {
            columns,
            primary,
            height,
            null_string,
        })
    }

    /// Constructs columns from inputs. Expects the length of `cols` and
    /// `headers` to be the same
    fn create_columns(
        cols: Vec<Vec<String>>,
        headers: Vec<Option<String>>,
        type_strategy: TypesStrategy,
        null: &str,
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
            .map(|((col, header), kind)| parse_column(col, header, kind, null))
            .collect()
    }

    /// Returns an iterator over the columns of the [`ColumnSheet`].
    pub fn iter(&self) -> Iter<'_, Box<dyn Column>> {
        self.columns.iter()
    }

    /// Returns an iterator that allows modifying each column
    pub fn iter_mut(&mut self) -> IterMut<'_, Box<dyn Column>> {
        self.columns.iter_mut()
    }

    /// Returns a reference to the value within the cell at column `col`, row `row`
    pub fn get_cell(&self, col: usize, row: usize) -> Option<CellRef> {
        self.columns.get(col).and_then(|col| col.data_ref(row))
    }

    /// Overwrites the cell at `col`, `row` with `value` if parsing to the
    /// valid column type succeeds.
    pub fn set_cell(&mut self, value: impl AsRef<str>, col: usize, row: usize) -> Result<()> {
        if col >= self.width() {
            return Err(Error::InvalidColumn(col));
        }

        if row >= self.height() {
            return Err(Error::InvalidRow(row));
        }

        let success =
            self.columns
                .get_mut(col)
                .unwrap()
                .set_position(value.as_ref(), row, &self.null_string);

        if !success {
            return Err(Error::InvalidCellInput { col, row });
        }

        Ok(())
    }

    /// Returns the row at index `row` within the [`ColumnSheet`] if any.
    pub fn get_row(&self, row: usize) -> Option<Vec<CellRef<'_>>> {
        if row >= self.height {
            return None;
        }

        let mut output = Vec::with_capacity(self.width());

        for column in &self.columns {
            let data = column.data_ref(row)?;
            output.push(data);
        }

        Some(output)
    }

    /// Time Complexity: `O(width * log(k) + width)`
    fn sort_col_helper(&mut self, cell: usize, rev: bool) {
        if cell >= self.height {
            return;
        }

        let columns = &self.columns;
        let mut indices = (0..self.width()).collect::<Vec<usize>>();

        // O(width * log(k))
        indices.sort_by(|x, y| {
            if rev {
                columns[*y].data_ref(cell).cmp(&columns[*x].data_ref(cell))
            } else {
                columns[*x].data_ref(cell).cmp(&columns[*y].data_ref(cell))
            }
        });

        // O(width)
        index_sort_swap(&mut indices);

        let primary = self.primary;

        // 0(width)
        for (pos, elem) in indices.iter().enumerate() {
            if Some(elem) == primary.as_ref() {
                self.primary = Some(pos)
            }
            self.columns.swap(pos, *elem);
        }

        //self.columns.sort_by(|a, b| {
        //    if rev {
        //        b.data_ref(cell).cmp(&a.data_ref(cell))
        //    } else {
        //        a.data_ref(cell).cmp(&b.data_ref(cell))
        //    }
        //})
    }

    /// Sorts the columns of the [`ColumnSheet`] using `sort_col_by` with `cell` as 0.
    pub fn sort_col(&mut self) {
        if !self.true_is_empty() {
            self.sort_col_by(0)
        }
    }

    /// Sorts the columns of the [`ColumnSheet`] by comparing the values at `cell` for each
    /// column.
    ///
    /// This sort has a time complexity of `O(width * log(k) + width)`
    /// where `k` is the number of unique elements in the sorting column
    pub fn sort_col_by(&mut self, cell: usize) {
        self.sort_col_helper(cell, false)
    }

    /// Sorts the columns of the [`ColumnSheet`] like `sort_col_by` but in reverse order.
    pub fn sort_col_by_rev(&mut self, cell: usize) {
        self.sort_col_helper(cell, true)
    }

    /// Sorts the columns of the [`ColumnSheet`] like `sort_col` but in reverse order.
    pub fn sort_col_rev(&mut self) {
        if !self.true_is_empty() {
            self.sort_col_by_rev(0)
        }
    }

    /// Time Complexity: `O(height * (1 + log(k) +  width)`
    fn sort_row_helper(&mut self, cell: usize, rev: bool) {
        if cell >= self.width() {
            return;
        }

        let column = &self.columns[cell];
        let mut indices = (0..self.height).collect::<Vec<usize>>();

        // O(height * log(k))
        indices.sort_by(|x, y| {
            if rev {
                column.data_ref(*y).cmp(&column.data_ref(*x))
            } else {
                column.data_ref(*x).cmp(&column.data_ref(*y))
            }
        });

        // O(height)
        index_sort_swap(&mut indices);

        // O(height * width)
        self.columns
            .iter_mut()
            .for_each(|column| column.apply_index_swap(&indices));
    }

    /// Sorts the rows of the [`ColumnSheet`] using the primary column. If no
    /// primary column is selected, the 0th column is used instead.
    pub fn sort_row(&mut self) {
        if !self.is_empty() {
            let cell = self.primary.unwrap_or(0);
            self.sort_row_by(cell)
        }
    }

    /// Sorts the rows of the [`ColumnSheet`] like `sort_row` but in reverse order.
    pub fn sort_row_rev(&mut self) {
        if !self.is_empty() {
            self.sort_row_by_rev(0)
        }
    }

    /// Sorts the rows of the [`ColumnSheet`] by comparing the values at `cell` for
    /// each row.
    ///
    /// This sort has a time complexity of `O(height * log(k) + height + height * width)`
    /// where `k` is the number of unique elements in the sorting column
    pub fn sort_row_by(&mut self, cell: usize) {
        self.sort_row_helper(cell, false)
    }

    /// Sorts the rows of the [`ColumnSheet`] like `sort_row_by` but in reverse order.
    pub fn sort_row_by_rev(&mut self, cell: usize) {
        self.sort_row_helper(cell, true)
    }

    /// Returns an iterator over the headers of the [`ColumnSheet`].
    pub fn headers(&self) -> impl ExactSizeIterator<Item = ColumnHeader<'_>> {
        self.columns.iter().map(|col| {
            let header = ColumnHeader {
                header: col.label(),
                kind: col.kind(),
            };
            header
        })
    }

    /// Sets the header of the column at `col` to `header`.
    pub fn set_col_header(&mut self, col: usize, header: impl Into<String>) -> Result<()> {
        if col >= self.width() {
            return Err(Error::InvalidColumn(col));
        }

        if let Some(column) = self.columns.get_mut(col) {
            column.set_header(header.into())
        }

        Ok(())
    }

    /// Returns the width of the [`ColumnSheet`].
    ///
    /// This is essentially the same as the number of [`Column`]s in the [`ColumnSheet`].
    pub fn width(&self) -> usize {
        self.columns.len()
    }

    /// Returns the height of the [`ColumnSheet`].
    ///
    /// The height is defined as the number of cells within a [`Column`]. As such,
    /// a 0 height [`ColumnSheet`] may still contain [`Column`]s.
    ///
    /// All [`Column`]s within a [`ColumnSheet`] are guaranteed to have the same
    /// height. Shorter columns are padded with null to achieve this.
    pub fn height(&self) -> usize {
        self.height
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

    /// Returns the index of the primary column of the [`ColumnSheet`] if any.
    ///
    /// An empty [`ColumnSheet`] has [`None`] as its primary column.
    pub fn get_primary(&self) -> Option<usize> {
        self.primary
    }

    /// Returns the string considered as a null input for the [`ColumnSheet`] as
    /// a string slice.
    pub fn get_null_string(&self) -> &str {
        &self.null_string
    }

    /// Sets the primary column of the [`ColumnSheet`] to [`None`].
    pub fn clear_primary(&mut self) {
        self.primary = None;
    }

    /// Returns a shared reference to the column at `idx`, if any.
    pub fn get_col(&self, idx: usize) -> Option<&dyn Column> {
        self.columns.get(idx).map(|boxed| boxed.as_ref())
    }

    /// Returns an exclusive reference to the column at `idx` if any.
    pub fn get_col_mut(&mut self, idx: usize) -> Option<&mut Box<dyn Column>> {
        self.columns.get_mut(idx)
    }

    /// Returns true if the [`ColumnSheet`] has no occupyied cells.
    ///
    /// The [`ColumnSheet`] may still contain [`Column`]s, but they will be empty.
    pub fn is_empty(&self) -> bool {
        self.height == 0
    }

    /// Returns true if the [`ColumnSheet`] contains no [`Column`]s.
    pub fn true_is_empty(&self) -> bool {
        self.width() == 0
    }

    /// Appends a column to the back of the [`ColumnSheet`]
    ///
    /// Returns `Err` if `column` has a different width than `Self`.
    pub fn push_col(&mut self, column: Box<dyn Column>) -> Result<()> {
        self.insert_col(column, self.width())
    }

    /// Appends a row to the back of the [`ColumnSheet`]
    ///
    /// Returns `Err` if `row` has a different width than `Self`.
    pub fn push_row<I, R>(&mut self, row: R) -> Result<()>
    where
        I: AsRef<str>,
        R: ExactSizeIterator<Item = I>,
    {
        self.insert_row(row, self.height)
    }

    /// Duplicates  the [`Column`] at `col`. The duplicate column is inserted at
    /// `col`, shifting all [`Column`]s after to the right.
    pub fn duplicate_col(&mut self, idx: usize) -> Result<()> {
        if idx >= self.width() {
            return Err(Error::InvalidColumn(idx));
        }

        let column = &self.columns[idx];
        let copy = column.convert_col(column.kind());

        self.insert_col(copy, idx)
    }

    /// Removes and returns the last [`Column`] from the [`ColumnSheet`].
    pub fn pop_col(&mut self) -> Result<Box<dyn Column>> {
        if self.width() == 0 {
            return Err(Error::InvalidColumn(0));
        }

        self.remove_col(self.width() - 1)
    }

    /// Removes the last row from the [`ColumnSheet`].
    pub fn pop_row(&mut self) -> Result<()> {
        if self.is_empty() {
            return Err(Error::InvalidRow(0));
        }

        self.remove_row(self.height - 1)
    }

    /// Removes and returns the [`Column`] at `idx` shifting all values to the left
    ///
    /// Returns `Err` if `idx` >= `self.width`  
    pub fn remove_col(&mut self, idx: usize) -> Result<Box<dyn Column>> {
        if idx >= self.width() {
            return Err(Error::InvalidColumn(idx));
        }

        let removed = self.columns.remove(idx);

        let Some(primary) = self.primary else {
            return Ok(removed);
        };

        if self.true_is_empty() {
            self.primary = None;
        } else if idx <= primary && primary != 0 {
            self.primary = Some(primary - 1);
        }

        Ok(removed)
    }

    /// Removes all [`Column`]s within the [`ColumnSheet`].
    pub fn remove_all_cols(&mut self) {
        self.columns.clear();
        self.height = 0;
        self.primary = None;
    }

    /// Removes the row at `idx` shifting all values to the up
    ///
    /// Returns `Err` if `idx` >= `self.height`  
    pub fn remove_row(&mut self, idx: usize) -> Result<()> {
        if idx >= self.height {
            return Err(Error::InvalidRow(idx));
        }

        self.columns
            .iter_mut()
            .for_each(|column| column.remove(idx));

        self.height -= 1;

        Ok(())
    }

    /// Removes all cells in all the [`ColumnSheet`].
    ///
    /// All [`Column`]s in are left empty.
    pub fn remove_all_rows(&mut self) {
        self.columns.iter_mut().for_each(|col| col.remove_all());
        self.height = 0;
    }

    /// Inserts a column at `idx` shifting all values after right
    ///
    /// Returns `Err` if `idx` > `self.width`  
    /// Returns `Err` if `column` has a different width than `Self`.
    pub fn insert_col(&mut self, column: Box<dyn Column>, idx: usize) -> Result<()> {
        let other = column.len();
        let own = self.height;

        if other != own && !self.true_is_empty() {
            return Err(Error::InvalidColumnHeight { own, other });
        }

        if idx > self.width() {
            return Err(Error::InvalidInsertion(idx));
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
    /// Returns `Err` if `idx` > `self.height()`  
    /// Returns `Err` if `row` has a different width than `Self`.
    pub fn insert_row<I, R>(&mut self, row: R, idx: usize) -> Result<()>
    where
        I: AsRef<str>,
        R: ExactSizeIterator<Item = I>,
    {
        let own = self.width();
        let other = row.len();

        if other != own && !self.true_is_empty() {
            return Err(Error::InvalidRowWidth { own, other });
        }

        if idx > self.height() {
            return Err(Error::InvalidInsertion(idx));
        }

        if self.is_empty() {
            let cols = row
                .map(|value| vec![value.as_ref().to_owned()])
                .collect::<Vec<Vec<String>>>();
            let len = cols.len();
            let columns = Self::create_columns(
                cols,
                vec![None; len],
                TypesStrategy::Infer,
                &self.null_string,
            );

            self.columns = columns;

            if len != 0 {
                self.primary = Some(0);
            }
        } else {
            self.columns
                .iter_mut()
                .zip(row)
                .for_each(|(column, value)| column.insert(value.as_ref(), idx, &self.null_string));
        }

        self.height += 1;

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
        let height = self.height;

        if x >= height {
            return Err(Error::InvalidRow(x));
        }

        if y >= height {
            return Err(Error::InvalidRow(y));
        }

        self.columns.iter_mut().for_each(|col| col.swap(x, y));

        Ok(())
    }

    /// Replaces all values within the [`Column`] at `idx` with [`None`].
    pub fn clear_col(&mut self, idx: usize) -> Result<()> {
        if idx >= self.width() {
            return Err(Error::InvalidColumn(idx));
        }

        if let Some(col) = self.columns.get_mut(idx) {
            col.clear_all();
        }

        Ok(())
    }

    /// Replaces all values within the row at `idx` with [`None`].
    pub fn clear_row(&mut self, idx: usize) -> Result<()> {
        if idx >= self.height() {
            return Err(Error::InvalidRow(idx));
        }

        self.columns.iter_mut().for_each(|column| column.clear(idx));

        Ok(())
    }

    /// Replaces the value of the cell in `col` column at `row` row with [`None`].
    pub fn clear_cell(&mut self, col: usize, row: usize) -> Result<()> {
        if col >= self.width() {
            return Err(Error::InvalidColumn(col));
        }

        if row >= self.height() {
            return Err(Error::InvalidRow(row));
        }

        if let Some(col) = self.columns.get_mut(col) {
            col.clear(row);
        }

        Ok(())
    }

    /// Converts the [`Column`] at `idx`index to a `to` type column.
    ///
    /// Unlike [`ColumnSheet::convert_col`], this does not check for [`DataType`]
    /// compatibility which could lead to loss of information and inaccuracies.
    pub fn convert_col_unchecked(&mut self, idx: usize, to: DataType) -> Result<()> {
        if idx >= self.width() {
            return Err(Error::InvalidColumn(idx));
        }

        let from = &self.columns[idx];
        let new = from.convert_col(to);

        self.columns.push(new);
        self.columns.swap_remove(idx);

        Ok(())
    }

    /// Converts the [`Column`] at `idx`index to a `to` type column.
    ///
    /// Returns an error if [`Column::kind`] is incompatible with `to`.
    pub fn convert_col(&mut self, idx: usize, to: DataType) -> Result<()> {
        if idx >= self.width() {
            return Err(Error::InvalidColumn(idx));
        }

        let from = &self.columns[idx];
        let from = from.kind();

        if DataType::can_convert(from, to) {
            self.convert_col_unchecked(idx, to)
        } else {
            Err(Error::InvalidColConversion { col: idx, from, to })
        }
    }
}

impl<P: AsRef<Path>> TryFrom<Config<P>> for ColumnSheet {
    type Error = Error;

    fn try_from(value: Config<P>) -> Result<Self> {
        Self::with_config(value)
    }
}

fn parse_column(
    col: Vec<String>,
    header: Option<String>,
    strategy: ColumnType,
    null: &str,
) -> Box<dyn Column> {
    let text = |col: Vec<String>, header: Option<String>| {
        let mut array = ArrayText::parse_str(&col, null);
        if let Some(header) = header {
            array.set_header(header);
        }
        Box::new(array)
    };

    match strategy {
        ColumnType::None => text(col, header),

        ColumnType::Infer => {
            if let Some(mut array) = ArrayI32::parse_str(&col, null) {
                if let Some(header) = header {
                    array.set_header(header);
                }
                return Box::new(array);
            };

            if let Some(mut array) = ArrayU32::parse_str(&col, null) {
                if let Some(header) = header {
                    array.set_header(header);
                }
                return Box::new(array);
            };

            if let Some(mut array) = ArrayISize::parse_str(&col, null) {
                if let Some(header) = header {
                    array.set_header(header);
                }
                return Box::new(array);
            };

            if let Some(mut array) = ArrayUSize::parse_str(&col, null) {
                if let Some(header) = header {
                    array.set_header(header);
                }
                return Box::new(array);
            };

            if let Some(mut array) = ArrayBool::parse_str(&col, null) {
                if let Some(header) = header {
                    array.set_header(header);
                }
                return Box::new(array);
            };

            if let Some(mut array) = ArrayF32::parse_str(&col, null) {
                if let Some(header) = header {
                    array.set_header(header);
                }
                return Box::new(array);
            };

            if let Some(mut array) = ArrayF64::parse_str(&col, null) {
                if let Some(header) = header {
                    array.set_header(header);
                }
                return Box::new(array);
            };

            text(col, header)
        }

        ColumnType::Type(CT::None) | ColumnType::Type(CT::Text) => text(col, header),

        ColumnType::Type(CT::Integer) => {
            if let Some(mut array) = ArrayI32::parse_str(&col, null) {
                if let Some(header) = header {
                    array.set_header(header);
                }
                return Box::new(array);
            };

            if let Some(mut array) = ArrayU32::parse_str(&col, null) {
                if let Some(header) = header {
                    array.set_header(header);
                }
                return Box::new(array);
            };

            text(col, header)
        }

        ColumnType::Type(CT::Number) => {
            if let Some(mut array) = ArrayISize::parse_str(&col, null) {
                if let Some(header) = header {
                    array.set_header(header);
                }
                return Box::new(array);
            };

            if let Some(mut array) = ArrayUSize::parse_str(&col, null) {
                if let Some(header) = header {
                    array.set_header(header);
                }
                return Box::new(array);
            };

            text(col, header)
        }

        ColumnType::Type(CT::Float) => {
            if let Some(mut array) = ArrayF32::parse_str(&col, null) {
                if let Some(header) = header {
                    array.set_header(header);
                }
                return Box::new(array);
            };

            if let Some(mut array) = ArrayF64::parse_str(&col, null) {
                if let Some(header) = header {
                    array.set_header(header);
                }
                return Box::new(array);
            };

            text(col, header)
        }

        ColumnType::Type(CT::Boolean) => {
            if let Some(mut array) = ArrayBool::parse_str(&col, null) {
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
    #[allow(unused_imports)]
    use super::*;
    use csv::Error as CSVError;
    use std::{error, fmt};

    #[derive(Debug)]
    pub enum Error {
        CSV(CSVError),
        InvalidColumn(usize),
        InvalidRow(usize),
        InvalidPrimary(usize),
        InvalidColumnHeight {
            own: usize,
            other: usize,
        },
        InvalidRowWidth {
            own: usize,
            other: usize,
        },
        InvalidInsertion(usize),
        InvalidCellInput {
            col: usize,
            row: usize,
        },
        InvalidColConversion {
            col: usize,
            from: DataType,
            to: DataType,
        },
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
                Self::InvalidInsertion(idx) => {
                    write!(f, "Invalid insertion at index {idx}")
                }
                Self::InvalidCellInput { col, row } => {
                    write!(f, "Invalid input for cell at column: {col}, row: {row}")
                }
                Self::InvalidColConversion { col, from, to } => {
                    write!(
                        f,
                        "Invalid column conversion from {from} to {to} at column {col}"
                    )
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

    /// A short hand alias for [`ColumnSheet`] error results
    pub type Result<T> = core::result::Result<T, Error>;
}

fn index_sort_swap(indices: &mut [usize]) {
    let mut pos = 0;
    let end = indices.len();

    while pos < end {
        let elem = indices[pos];

        if pos > elem {
            indices[pos] = indices[elem]
        } else {
            pos += 1
        }
    }
}

/// Dont want to have to type these out every time.
mod arrays {
    pub use super::{
        ArrayBool, ArrayF32, ArrayF64, ArrayI32, ArrayISize, ArrayText, ArrayU32, ArrayUSize,
    };
}

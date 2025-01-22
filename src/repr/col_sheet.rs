#![allow(unused_imports, dead_code)]
use csv::{ReaderBuilder, Trim};
use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    path::{Path, PathBuf},
    slice::{Iter, IterMut},
    str::FromStr,
};

use crate::models::{
    bar::{Bar, BarChart},
    line::{Line, LineGraph},
    stacked_bar::{StackedBar, StackedBarChart},
    Point, Scale, ScaleKind,
};

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

#[derive(Debug, Clone, PartialEq)]
pub enum Union {
    I32(i32),
    U32(u32),
    ISize(isize),
    USize(usize),
    Boolean(bool),
    F32(f32),
    F64(f64),
    Text(String),
    Null,
}

impl Union {
    pub fn parse(input: impl Into<String>) -> Self {
        let input: String = input.into();

        if input.is_empty() || input == *NULL {
            return Self::Null;
        }

        if let Ok(parsed_u32) = input.parse::<u32>() {
            return Self::U32(parsed_u32);
        }

        if let Ok(parsed_i32) = input.parse::<i32>() {
            return Self::I32(parsed_i32);
        }

        if let Ok(parsed_usize) = input.parse::<usize>() {
            return Self::USize(parsed_usize);
        }

        if let Ok(parsed_isize) = input.parse::<isize>() {
            return Self::ISize(parsed_isize);
        }

        if let Ok(parsed_f32) = input.parse::<f32>() {
            return Self::F32(parsed_f32);
        }

        if let Ok(parsed_f64) = input.parse::<f64>() {
            return Self::F64(parsed_f64);
        }

        if let Ok(parsed_bool) = input.parse::<bool>() {
            return Self::Boolean(parsed_bool);
        };

        Self::Text(input)
    }
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
    fn set_position(&mut self, value: &str, idx: usize);

    /// Swaps the value at `x` with that at `y`.
    ///
    /// No swap occurs if either `x` or `y` are invalid indices.
    fn swap(&mut self, x: usize, y: usize);
}

#[derive(Debug)]
pub struct ColumnHeader<'a> {
    header: Option<&'a String>,
    kind: DataType,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ArrayI32 {
    header: Option<String>,
    cells: Vec<Option<i32>>,
}

impl ArrayI32 {
    fn new() -> Self {
        Self::default()
    }

    fn from_iterator(values: impl Iterator<Item = i32>) -> Self {
        Self {
            cells: values.map(Some).collect(),
            ..Default::default()
        }
    }

    fn from_iterator_option(values: impl Iterator<Item = Option<i32>>) -> Self {
        Self {
            cells: values.collect(),
            ..Default::default()
        }
    }

    fn set_header(&mut self, header: String) -> &mut Self {
        self.header = Some(header);
        self
    }

    fn get(&self, idx: usize) -> Option<i32> {
        self.cells.get(idx)?.as_ref().copied()
    }

    fn iter(&self) -> Iter<'_, Option<i32>> {
        self.cells.iter()
    }

    fn parse_str(values: &Vec<String>) -> Option<Self> {
        let mut cells = Vec::default();

        for value in values {
            let value = parse_helper::<i32>(value).ok()?;
            cells.push(value)
        }

        Some(Self {
            header: None,
            cells,
        })
    }
}

impl Sealed for ArrayI32 {
    fn push(&mut self, value: &str) {
        let value = match parse_helper::<i32>(value) {
            Ok(val) => val,
            Err(_) => None,
        };
        self.cells.push(value)
    }

    fn remove(&mut self, idx: usize) {
        if idx >= self.len() {
            return;
        }
        self.cells.remove(idx);
    }

    fn insert(&mut self, value: &str, idx: usize) {
        if idx >= self.len() {
            return;
        }

        let Ok(parsed) = parse_helper::<i32>(value) else {
            return;
        };

        self.cells.insert(idx, parsed);
    }
}

impl Column for ArrayI32 {
    fn label(&self) -> Option<&String> {
        self.header.as_ref()
    }

    fn kind(&self) -> DataType {
        DataType::Int32
    }

    fn len(&self) -> usize {
        self.cells.len()
    }

    fn set_header(&mut self, header: String) {
        self.header = Some(header);
    }

    fn set_position(&mut self, value: &str, idx: usize) {
        let Ok(parsed) = parse_helper::<i32>(value) else {
            return;
        };

        let Some(prev) = self.cells.get_mut(idx) else {
            return;
        };

        *prev = parsed;
    }

    fn swap(&mut self, x: usize, y: usize) {
        if x >= self.len() || y >= self.len() {
            return;
        }

        self.cells.swap(x, y);
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ArrayISize {
    header: Option<String>,
    cells: Vec<Option<isize>>,
}

impl ArrayISize {
    fn new() -> Self {
        Self::default()
    }

    fn from_iterator(values: impl Iterator<Item = isize>) -> Self {
        Self {
            cells: values.map(Some).collect(),
            ..Default::default()
        }
    }

    fn from_iterator_option(values: impl Iterator<Item = Option<isize>>) -> Self {
        Self {
            cells: values.collect(),
            ..Default::default()
        }
    }

    fn set_header(&mut self, header: String) -> &mut Self {
        self.header = Some(header);
        self
    }

    fn get(&self, idx: usize) -> Option<isize> {
        self.cells.get(idx)?.as_ref().copied()
    }

    fn iter(&self) -> Iter<'_, Option<isize>> {
        self.cells.iter()
    }

    fn parse_str(values: &Vec<String>) -> Option<Self> {
        let mut cells = Vec::default();

        for value in values {
            let value = parse_helper::<isize>(value).ok()?;
            cells.push(value)
        }

        Some(Self {
            header: None,
            cells,
        })
    }
}

impl Sealed for ArrayISize {
    fn push(&mut self, value: &str) {
        let value = match parse_helper::<isize>(value) {
            Ok(val) => val,
            Err(_) => None,
        };
        self.cells.push(value)
    }

    fn remove(&mut self, idx: usize) {
        if idx >= self.len() {
            return;
        }
        self.cells.remove(idx);
    }

    fn insert(&mut self, value: &str, idx: usize) {
        if idx >= self.len() {
            return;
        }

        let Ok(parsed) = parse_helper::<isize>(value) else {
            return;
        };

        self.cells.insert(idx, parsed);
    }
}

impl Column for ArrayISize {
    fn len(&self) -> usize {
        self.cells.len()
    }

    fn kind(&self) -> DataType {
        DataType::ISize
    }

    fn label(&self) -> Option<&String> {
        self.header.as_ref()
    }

    fn set_header(&mut self, header: String) {
        self.header = Some(header)
    }

    fn set_position(&mut self, value: &str, idx: usize) {
        let Ok(parsed) = parse_helper::<isize>(value) else {
            return;
        };

        let Some(prev) = self.cells.get_mut(idx) else {
            return;
        };

        *prev = parsed;
    }

    fn swap(&mut self, x: usize, y: usize) {
        if x >= self.len() || y >= self.len() {
            return;
        }

        self.cells.swap(x, y)
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ArrayText {
    header: Option<String>,
    cells: Vec<Option<String>>,
}

impl ArrayText {
    fn new() -> Self {
        Self::default()
    }

    fn from_iterator(values: impl Iterator<Item = String>) -> Self {
        Self {
            cells: values.map(Some).collect(),
            ..Default::default()
        }
    }

    fn from_iterator_option(values: impl Iterator<Item = Option<String>>) -> Self {
        Self {
            cells: values.collect(),
            ..Default::default()
        }
    }

    fn set_header(&mut self, header: impl Into<String>) -> &mut Self {
        self.header = Some(header.into());
        self
    }

    fn get(&self, idx: usize) -> Option<String> {
        self.cells.get(idx)?.as_ref().cloned()
    }

    fn get_ref(&self, idx: usize) -> Option<&String> {
        self.cells.get(idx)?.as_ref()
    }

    fn iter(&self) -> Iter<'_, Option<String>> {
        self.cells.iter()
    }

    fn parse_str(values: Vec<String>) -> Self {
        let mut cells = Vec::default();

        for value in values {
            // Always successful
            let value = match parse_helper::<String>(&value) {
                Ok(val) => val,
                Err(_) => None,
            };
            cells.push(value);
        }

        Self {
            header: None,
            cells,
        }
    }
}

impl Sealed for ArrayText {
    fn push(&mut self, value: &str) {
        let value = match parse_helper::<String>(value) {
            Ok(val) => val,
            Err(_) => None,
        };
        self.cells.push(value)
    }

    fn remove(&mut self, idx: usize) {
        if idx >= self.len() {
            return;
        }
        self.cells.remove(idx);
    }

    fn insert(&mut self, value: &str, idx: usize) {
        if idx >= self.len() {
            return;
        }
        let Ok(parsed) = parse_helper::<String>(value) else {
            return;
        };

        self.cells.insert(idx, parsed);
    }
}

impl Column for ArrayText {
    fn len(&self) -> usize {
        self.cells.len()
    }

    fn kind(&self) -> DataType {
        DataType::Text
    }

    fn label(&self) -> Option<&String> {
        self.header.as_ref()
    }

    fn set_header(&mut self, header: String) {
        self.set_header(header);
    }

    fn set_position(&mut self, value: &str, idx: usize) {
        let Ok(parsed) = parse_helper::<String>(value) else {
            return;
        };

        let Some(prev) = self.cells.get_mut(idx) else {
            return;
        };

        *prev = parsed;
    }

    fn swap(&mut self, x: usize, y: usize) {
        if x >= self.len() || y >= self.len() {
            return;
        }

        self.cells.swap(x, y)
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

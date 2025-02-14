use std::{
    any::Any,
    cmp::{Eq, Ord, Ordering, PartialOrd},
    fmt::{Debug, Display},
    str::FromStr,
};

pub(super) use private::Sealed;

/// Data types supported by the current implementation.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum DataType {
    I32,
    U32,
    ISize,
    USize,
    Bool,
    F32,
    F64,
    #[default]
    Text,
}

impl DataType {
    /// Returns true if a lossless conversion can be made.
    pub fn can_convert(from: Self, to: Self) -> bool {
        match (from, to) {
            (Self::Text, Self::Text) => true,
            (_, Self::Text) => true,
            (Self::Text, _) => false,

            (Self::U32, Self::U32) => true,
            (Self::U32, Self::USize) => true,
            (Self::U32, Self::ISize) => true,
            (Self::U32, Self::F64) => true,
            (Self::U32, _) => false,

            (Self::I32, Self::I32) => true,
            (Self::I32, Self::ISize) => true,
            (Self::I32, Self::F64) => true,
            (Self::I32, _) => false,

            (Self::ISize, Self::ISize) => true,
            (Self::ISize, _) => false,

            (Self::USize, Self::USize) => true,
            (Self::USize, _) => false,

            (Self::F32, Self::F32) => true,
            (Self::F32, Self::F64) => true,
            (Self::F32, _) => false,

            (Self::F64, Self::F64) => true,
            (Self::F64, _) => false,

            (Self::Bool, Self::Bool) => true,
            (Self::Bool, _) => false,
        }
    }
}

impl Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

pub trait Column: Sealed + Debug + Any {
    fn as_any(&self) -> &dyn Any;

    /// Returns the a reference to the header label of the [`Column`].
    fn label(&self) -> Option<&str>;

    /// Returns the type of data within the [`Column`].
    fn kind(&self) -> DataType;

    /// Returns a reference to the data at index `idx` within the [`Column`].
    ///
    /// A [`None`] value is returned if `idx` is out of range.
    fn data_ref(&self, idx: usize) -> Option<CellRef<'_>>;

    /// Returns the length of the [`Column`].
    fn len(&self) -> usize;

    /// Returns true if the [`Column`] has no element.
    ///
    /// Null values are considered to be elements.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Discards the value at `idx` leaving a [`None`] in its place.
    fn clear(&mut self, idx: usize);

    /// Replaces all values within the [`Column`] with [`None`].
    fn clear_all(&mut self);

    /// Sets the header label for the [`Column`].
    fn set_header(&mut self, header: String);

    /// Overwrites the value at `idx` with successfully parsed `value`.
    ///
    /// If `value` matches `null`, a [`None`] is written at `idx`.
    /// If parsing fails, `idx` is left as-is,  returning false.
    fn set_position(&mut self, value: &str, idx: usize, null: &str) -> bool;

    /// Swaps the value at `x` with that at `y`.
    ///
    /// No swap occurs if either `x` or `y` are invalid indices.
    fn swap(&mut self, x: usize, y: usize);

    /// Returns a new [`Column`] from the converted values of self.
    ///
    /// Incompatible conversions will lead to information loss and inaccuracies.
    fn convert_col(&self, to: DataType) -> Box<dyn Column>;
}

#[derive(Debug, PartialEq)]
pub struct ColumnHeader<'a> {
    pub header: Option<&'a str>,
    pub kind: DataType,
}

/// Reference to the data within a [`Column`]'s cell.
#[derive(Debug, PartialEq)]
pub enum CellRef<'a> {
    I32(i32),
    U32(u32),
    ISize(isize),
    USize(usize),
    Bool(bool),
    F32(f32),
    F64(f64),
    Text(&'a str),
    None,
}

impl<'a> CellRef<'a> {
    pub(super) fn cmp(&self, b: &Self) -> Ordering {
        match (self, b) {
            (CellRef::None, CellRef::None) => Ordering::Equal,
            (CellRef::None, _) => Ordering::Less,
            (_, CellRef::None) => Ordering::Greater,

            (CellRef::I32(x), CellRef::I32(y)) => x.cmp(y),
            (CellRef::I32(x), CellRef::U32(y)) => {
                if *x < 0 {
                    Ordering::Less
                } else {
                    (*x as u32).cmp(y)
                }
            }
            (CellRef::U32(x), CellRef::I32(y)) => {
                if *y < 0 {
                    Ordering::Less
                } else {
                    (*y as u32).cmp(x)
                }
            }
            (CellRef::I32(x), CellRef::ISize(y)) => (*x as isize).cmp(y),
            (CellRef::ISize(x), CellRef::I32(y)) => (*y as isize).cmp(x),
            (CellRef::I32(x), CellRef::USize(y)) => {
                if *x < 0 {
                    Ordering::Less
                } else {
                    (*x as usize).cmp(y)
                }
            }
            (CellRef::USize(x), CellRef::I32(y)) => {
                if *y < 0 {
                    Ordering::Less
                } else {
                    (*y as usize).cmp(x)
                }
            }
            (CellRef::I32(x), CellRef::F32(y)) => (*x as f64).total_cmp(&(*y as f64)),
            (CellRef::F32(x), CellRef::I32(y)) => (*y as f64).total_cmp(&(*x as f64)),
            (CellRef::I32(x), CellRef::F64(y)) => (*x as f64).total_cmp(y),
            (CellRef::F64(x), CellRef::I32(y)) => (*y as f64).total_cmp(x),

            (CellRef::U32(x), CellRef::U32(y)) => x.cmp(y),
            (CellRef::U32(x), CellRef::USize(y)) => (*x as usize).cmp(y),
            (CellRef::USize(x), CellRef::U32(y)) => (*y as usize).cmp(x),
            (CellRef::U32(x), CellRef::ISize(y)) => (*x as isize).cmp(y),
            (CellRef::ISize(x), CellRef::U32(y)) => (*y as isize).cmp(x),
            (CellRef::U32(x), CellRef::F32(y)) => (*x as f64).total_cmp(&(*y as f64)),
            (CellRef::F32(x), CellRef::U32(y)) => (*y as f64).total_cmp(&(*x as f64)),
            (CellRef::U32(x), CellRef::F64(y)) => (*x as f64).total_cmp(y),
            (CellRef::F64(x), CellRef::U32(y)) => (*y as f64).total_cmp(x),

            (CellRef::ISize(x), CellRef::ISize(y)) => x.cmp(y),
            (CellRef::ISize(x), CellRef::USize(y)) => {
                if *x < 0 {
                    Ordering::Less
                } else {
                    (*x as usize).cmp(y)
                }
            }
            (CellRef::USize(x), CellRef::ISize(y)) => {
                if *y < 0 {
                    Ordering::Less
                } else {
                    (*y as usize).cmp(x)
                }
            }
            (CellRef::ISize(x), CellRef::F32(y)) => (*x as f64).total_cmp(&(*y as f64)),
            (CellRef::F32(x), CellRef::ISize(y)) => (*y as f64).total_cmp(&(*x as f64)),
            (CellRef::ISize(x), CellRef::F64(y)) => (*x as f64).total_cmp(y),
            (CellRef::F64(x), CellRef::ISize(y)) => (*y as f64).total_cmp(x),

            (CellRef::USize(x), CellRef::USize(y)) => x.cmp(y),
            (CellRef::USize(x), CellRef::F32(y)) => (*x as f64).total_cmp(&(*y as f64)),
            (CellRef::F32(x), CellRef::USize(y)) => (*y as f64).total_cmp(&(*x as f64)),
            (CellRef::USize(x), CellRef::F64(y)) => (*x as f64).total_cmp(y),
            (CellRef::F64(x), CellRef::USize(y)) => (*y as f64).total_cmp(x),

            (CellRef::Bool(x), CellRef::Bool(y)) => x.cmp(y),
            (CellRef::Bool(_), _) => Ordering::Less,
            (_, CellRef::Bool(_)) => Ordering::Greater,

            (CellRef::F32(x), CellRef::F32(y)) => x.total_cmp(y),
            (CellRef::F32(x), CellRef::F64(y)) => (*x as f64).total_cmp(y),
            (CellRef::F64(x), CellRef::F32(y)) => (*y as f64).total_cmp(x),

            (CellRef::F64(x), CellRef::F64(y)) => x.total_cmp(y),

            (CellRef::Text(x), CellRef::Text(y)) => x.cmp(y),

            (CellRef::Text(_), _) => Ordering::Greater,
            (_, CellRef::Text(_)) => Ordering::Less,
        }
    }
}

impl<'a> From<CellRef<'a>> for Option<String> {
    fn from(value: CellRef<'a>) -> Self {
        match value {
            CellRef::I32(value) => Some(value.to_string()),
            CellRef::U32(value) => Some(value.to_string()),
            CellRef::ISize(value) => Some(value.to_string()),
            CellRef::USize(value) => Some(value.to_string()),
            CellRef::F32(value) => Some(value.to_string()),
            CellRef::F64(value) => Some(value.to_string()),
            CellRef::Bool(value) => Some(value.to_string()),
            CellRef::Text(value) => Some(value.to_owned()),
            CellRef::None => None,
        }
    }
}

impl<'a> PartialOrd for CellRef<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(Ord::cmp(self, other))
    }
}

impl<'a> Ord for CellRef<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cmp(other)
    }
}

impl<'a> Eq for CellRef<'a> {}

impl<'a> From<&'a str> for CellRef<'a> {
    fn from(value: &'a str) -> Self {
        Self::Text(value)
    }
}

impl<'a> From<Option<&'a str>> for CellRef<'a> {
    fn from(value: Option<&'a str>) -> Self {
        match value {
            Some(value) => Self::Text(value),
            None => Self::None,
        }
    }
}

impl<'a> From<f64> for CellRef<'a> {
    fn from(value: f64) -> Self {
        Self::F64(value)
    }
}

impl<'a> From<Option<f64>> for CellRef<'a> {
    fn from(value: Option<f64>) -> Self {
        match value {
            Some(value) => Self::F64(value),
            None => Self::None,
        }
    }
}

impl<'a> From<f32> for CellRef<'a> {
    fn from(value: f32) -> Self {
        Self::F32(value)
    }
}

impl<'a> From<Option<f32>> for CellRef<'a> {
    fn from(value: Option<f32>) -> Self {
        match value {
            Some(value) => Self::F32(value),
            None => Self::None,
        }
    }
}

impl<'a> From<bool> for CellRef<'a> {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl<'a> From<Option<bool>> for CellRef<'a> {
    fn from(value: Option<bool>) -> Self {
        match value {
            Some(value) => Self::Bool(value),
            None => Self::None,
        }
    }
}

impl<'a> From<usize> for CellRef<'a> {
    fn from(value: usize) -> Self {
        Self::USize(value)
    }
}

impl<'a> From<Option<usize>> for CellRef<'a> {
    fn from(value: Option<usize>) -> Self {
        match value {
            Some(value) => Self::USize(value),
            None => Self::None,
        }
    }
}

impl<'a> From<isize> for CellRef<'a> {
    fn from(value: isize) -> Self {
        Self::ISize(value)
    }
}

impl<'a> From<Option<isize>> for CellRef<'a> {
    fn from(value: Option<isize>) -> Self {
        match value {
            Some(value) => Self::ISize(value),
            None => Self::None,
        }
    }
}

impl<'a> From<u32> for CellRef<'a> {
    fn from(value: u32) -> Self {
        Self::U32(value)
    }
}

impl<'a> From<Option<u32>> for CellRef<'a> {
    fn from(value: Option<u32>) -> Self {
        match value {
            Some(value) => Self::U32(value),
            None => Self::None,
        }
    }
}

impl<'a> From<i32> for CellRef<'a> {
    fn from(value: i32) -> Self {
        Self::I32(value)
    }
}

impl<'a> From<Option<i32>> for CellRef<'a> {
    fn from(value: Option<i32>) -> Self {
        match value {
            Some(value) => Self::I32(value),
            None => Self::None,
        }
    }
}

/// Parses `input` into given type, taking note of both empty and null strings.
///
/// On error, `()` is returned.
pub(super) fn parse_helper<T: FromStr>(input: &str, null: &str) -> Result<Option<T>, ()> {
    if input.is_empty() || input == null {
        return Ok(None);
    }

    input.parse::<T>().map_err(|_err| {}).map(Some)
}

/// Discards the error from `parse_helper`.
///
/// Logs any parsing failures
pub(super) fn parse_unchecked<T: FromStr>(input: &str, null: &str) -> Option<T> {
    parse_helper(input, null).ok()?
}

mod private {
    #![allow(unused_imports)]
    use super::super::ColumnSheet;
    use super::Column;
    /// Methods within this trait are kept private to ensure all invariants on
    /// [`ColumnSheet`] are maintained.
    pub trait Sealed {
        /// Pushes `value` to the end of the column by parsing `value`. If
        /// `value` matches `null`, a [`None`] is appended instead.
        ///
        /// Should parsing fail, a [`None`] value is pushed instead
        fn push(&mut self, value: &str, null: &str);

        /// Removes the value at `idx` if any, shifting the remaning values up.
        fn remove(&mut self, idx: usize);

        /// Removes all values within the [`Column`]
        fn remove_all(&mut self);

        /// Inserts successfully parsed `value` at `idx` shifting all elements after
        /// to the right. If `value` matches `null`, a [`None`] is inserted
        /// instead.
        ///
        /// Should parsing fail, a [`None`] is inserted.
        fn insert(&mut self, value: &str, idx: usize, null: &str);

        /// Applies the provided swap indices to self, sorting the contents of
        /// self as a result.
        fn apply_index_swap(&mut self, indices: &[usize]);
    }
}

use std::{
    any::Any,
    cmp::{Eq, Ord, Ordering, PartialOrd},
    fmt::Debug,
    str::FromStr,
};

pub(super) use private::Sealed;

const NULL: &str = "<null>";

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

pub trait Column: Sealed + Debug + Any {
    fn as_any(&self) -> &dyn Any;

    /// Returns the a reference to the header label of the [`Column`].
    fn label(&self) -> Option<&str>;

    /// Returns the type of data within the [`Column`].
    fn kind(&self) -> DataType;

    /// Returns a reference to the data at index `idx` within the [`Column`].
    ///
    /// A [`None`] value is returned if `idx` is out of range.
    fn data_ref(&self, idx: usize) -> Option<DataRef<'_>>;

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

    /// Overwrites the value at `idx` with successfully parsed `value`. If
    /// parsing fails, `idx` is left as-is,  returning false.
    fn set_position(&mut self, value: &str, idx: usize) -> bool;

    /// Swaps the value at `x` with that at `y`.
    ///
    /// No swap occurs if either `x` or `y` are invalid indices.
    fn swap(&mut self, x: usize, y: usize);
}

#[derive(Debug, PartialEq)]
pub struct ColumnHeader<'a> {
    pub header: Option<&'a str>,
    pub kind: DataType,
}

/// Reference to the data within a [`Column`]'s cell.
#[derive(Debug, PartialEq)]
pub enum DataRef<'a> {
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

impl<'a> DataRef<'a> {
    pub(super) fn cmp(&self, b: &Self) -> Ordering {
        match (self, b) {
            (DataRef::None, DataRef::None) => Ordering::Equal,
            (DataRef::None, _) => Ordering::Less,
            (_, DataRef::None) => Ordering::Greater,

            (DataRef::I32(x), DataRef::I32(y)) => x.cmp(y),
            (DataRef::I32(x), DataRef::U32(y)) => {
                if *x < 0 {
                    Ordering::Less
                } else {
                    (*x as u32).cmp(y)
                }
            }
            (DataRef::U32(x), DataRef::I32(y)) => {
                if *y < 0 {
                    Ordering::Less
                } else {
                    (*y as u32).cmp(x)
                }
            }
            (DataRef::I32(x), DataRef::ISize(y)) => (*x as isize).cmp(y),
            (DataRef::ISize(x), DataRef::I32(y)) => (*y as isize).cmp(x),
            (DataRef::I32(x), DataRef::USize(y)) => {
                if *x < 0 {
                    Ordering::Less
                } else {
                    (*x as usize).cmp(y)
                }
            }
            (DataRef::USize(x), DataRef::I32(y)) => {
                if *y < 0 {
                    Ordering::Less
                } else {
                    (*y as usize).cmp(x)
                }
            }
            (DataRef::I32(x), DataRef::F32(y)) => (*x as f64).total_cmp(&(*y as f64)),
            (DataRef::F32(x), DataRef::I32(y)) => (*y as f64).total_cmp(&(*x as f64)),
            (DataRef::I32(x), DataRef::F64(y)) => (*x as f64).total_cmp(y),
            (DataRef::F64(x), DataRef::I32(y)) => (*y as f64).total_cmp(x),

            (DataRef::U32(x), DataRef::U32(y)) => x.cmp(y),
            (DataRef::U32(x), DataRef::USize(y)) => (*x as usize).cmp(y),
            (DataRef::USize(x), DataRef::U32(y)) => (*y as usize).cmp(x),
            (DataRef::U32(x), DataRef::ISize(y)) => (*x as isize).cmp(y),
            (DataRef::ISize(x), DataRef::U32(y)) => (*y as isize).cmp(x),
            (DataRef::U32(x), DataRef::F32(y)) => (*x as f64).total_cmp(&(*y as f64)),
            (DataRef::F32(x), DataRef::U32(y)) => (*y as f64).total_cmp(&(*x as f64)),
            (DataRef::U32(x), DataRef::F64(y)) => (*x as f64).total_cmp(y),
            (DataRef::F64(x), DataRef::U32(y)) => (*y as f64).total_cmp(x),

            (DataRef::ISize(x), DataRef::ISize(y)) => x.cmp(y),
            (DataRef::ISize(x), DataRef::USize(y)) => {
                if *x < 0 {
                    Ordering::Less
                } else {
                    (*x as usize).cmp(y)
                }
            }
            (DataRef::USize(x), DataRef::ISize(y)) => {
                if *y < 0 {
                    Ordering::Less
                } else {
                    (*y as usize).cmp(x)
                }
            }
            (DataRef::ISize(x), DataRef::F32(y)) => (*x as f64).total_cmp(&(*y as f64)),
            (DataRef::F32(x), DataRef::ISize(y)) => (*y as f64).total_cmp(&(*x as f64)),
            (DataRef::ISize(x), DataRef::F64(y)) => (*x as f64).total_cmp(y),
            (DataRef::F64(x), DataRef::ISize(y)) => (*y as f64).total_cmp(x),

            (DataRef::USize(x), DataRef::USize(y)) => x.cmp(y),
            (DataRef::USize(x), DataRef::F32(y)) => (*x as f64).total_cmp(&(*y as f64)),
            (DataRef::F32(x), DataRef::USize(y)) => (*y as f64).total_cmp(&(*x as f64)),
            (DataRef::USize(x), DataRef::F64(y)) => (*x as f64).total_cmp(y),
            (DataRef::F64(x), DataRef::USize(y)) => (*y as f64).total_cmp(x),

            (DataRef::Bool(x), DataRef::Bool(y)) => x.cmp(y),
            (DataRef::Bool(_), _) => Ordering::Less,
            (_, DataRef::Bool(_)) => Ordering::Greater,

            (DataRef::F32(x), DataRef::F32(y)) => x.total_cmp(y),
            (DataRef::F32(x), DataRef::F64(y)) => (*x as f64).total_cmp(y),
            (DataRef::F64(x), DataRef::F32(y)) => (*y as f64).total_cmp(x),

            (DataRef::F64(x), DataRef::F64(y)) => x.total_cmp(y),

            (DataRef::Text(x), DataRef::Text(y)) => x.cmp(y),

            (DataRef::Text(_), _) => Ordering::Greater,
            (_, DataRef::Text(_)) => Ordering::Less,
        }
    }
}

impl<'a> PartialOrd for DataRef<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(Ord::cmp(self, other))
    }
}

impl<'a> Ord for DataRef<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cmp(other)
    }
}

impl<'a> Eq for DataRef<'a> {}

/// Parses `input` into given type, taking note of both empty and null strings.
///
/// On error, `()` is returned.
pub(super) fn parse_helper<T: FromStr>(input: &str) -> Result<Option<T>, ()> {
    if input.is_empty() || input == NULL {
        return Ok(None);
    }

    input.parse::<T>().map_err(|_err| {}).map(Some)
}

/// Discards the error from `parse_helper`.
///
/// Logs any parsing failures
pub(super) fn parse_unchecked<T: FromStr>(input: &str) -> Option<T> {
    parse_helper(input).ok()?
}

mod private {
    #![allow(unused_imports)]
    use super::super::ColumnSheet;
    use super::Column;
    /// Methods within this trait are kept private to ensure all invariants on
    /// [`ColumnSheet`] are maintained.
    pub trait Sealed {
        /// Pushes `value` to the end of the column by parsing `value`.
        ///
        /// Should parsing fail, a null value is pushed instead
        fn push(&mut self, value: &str);

        /// Removes the value at `idx` if any, shifting the remaning values up.
        fn remove(&mut self, idx: usize);

        /// Removes all values within the [`Column`]
        fn remove_all(&mut self);

        /// Inserts successfully parsed `value` at `idx` shifting all elements after
        /// to the right.
        ///
        /// Should parsing fail, a [`None`] is inserted.
        fn insert(&mut self, value: &str, idx: usize);

        /// Applies the provided swap indices to self, sorting the contents of
        /// self as a result.
        fn apply_index_swap(&mut self, indices: &[usize]);
    }
}

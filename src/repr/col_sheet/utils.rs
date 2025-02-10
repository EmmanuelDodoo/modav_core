use std::{fmt::Debug, str::FromStr};

pub(super) use private::Sealed;

const NULL: &str = "<null>";

/// Data types supported by the current implementation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DataType {
    I32,
    U32,
    ISize,
    USize,
    Bool,
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

/// Parses `input` into given type, taking note of both empty and null strings.
///
/// On error, `()` is returned.
pub(super) fn parse_helper<T: FromStr>(input: &str) -> Result<Option<T>, ()> {
    if input.is_empty() || input == NULL {
        return Ok(None);
    }

    input.parse::<T>().map_err(|_err| {}).map(Some)
}

mod private {
    #[allow(unused_imports)]
    use super::super::ColumnSheet;
    /// Methods within this trait are kept private to ensure all invariants on
    /// [`ColumnSheet`] are maintained.
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

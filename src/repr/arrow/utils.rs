use std::fmt::Debug;

/// Data types supported by the current implementation of Apache Arrow.
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
    Union,
}

pub trait Array:
    Clone + PartialEq + Debug + IntoIterator<Item = Option<Self::Data>, IntoIter = IntoIter<Self>>
{
    /// The data type used by the array.
    type Data;
    /// The reference type returned by the array.
    type Ref<'a>
    where
        Self: 'a;

    fn new<I>(values: I) -> Self
    where
        I: IntoIterator<Item = Option<Self::Data>>,
        I::IntoIter: ExactSizeIterator;

    /// Returns an owned value at `idx` if any.
    ///
    /// Returns None if `idx` is out of range
    fn get(&self, idx: usize) -> Option<Self::Data>;

    /// Returns a shared reference to the value at `idx` if any.
    ///
    /// Returns None if `idx` is out of range
    fn get_ref(&self, idx: usize) -> Option<Self::Ref<'_>>;

    /// Returns true if the value contained at `idx` is null
    ///
    /// May panic if `idx` is out of bounds
    fn check_null(&self, idx: usize) -> bool;

    /// Returns true if the array contains only `null` elements
    fn all_null(&self) -> bool;

    /// Returns true if the array is completely empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn len(&self) -> usize;

    /// Returns the [`DataType`] of this array.
    fn data_type(&self) -> DataType;

    /// Returns an iterator over the values in the array
    fn iter(&self) -> Iter<'_, Self> {
        Iter::new(self)
    }

    /// Returns an iterator over copied array values.
    ///
    /// The array is not consumed in the process.
    fn copied_iter(&self) -> CopiedIter<'_, Self>
    where
        Self::Data: Copy,
    {
        CopiedIter::new(self)
    }
}

pub struct Iter<'a, T: Array> {
    array: &'a T,
    idx: usize,
}

impl<'a, T> Iter<'a, T>
where
    T: Array,
{
    fn new(array: &'a T) -> Self {
        Self { array, idx: 0 }
    }
}

impl<'a, T> Iterator for Iter<'a, T>
where
    T: Array,
{
    type Item = Option<T::Ref<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        let idx = self.idx;
        self.idx += 1;
        if idx >= self.array.len() {
            None
        } else {
            Some(self.array.get_ref(idx))
        }
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.array.len() - self.idx
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.array.len() - self.idx;
        (len, Some(len))
    }
}

impl<'a, T> ExactSizeIterator for Iter<'a, T>
where
    T: Array,
{
    fn len(&self) -> usize {
        self.array.len() - self.idx
    }
}

pub struct CopiedIter<'a, T: Array> {
    array: &'a T,
    idx: usize,
}

impl<'a, T> CopiedIter<'a, T>
where
    T: Array,
{
    fn new(array: &'a T) -> Self {
        Self { array, idx: 0 }
    }
}

impl<'a, T> Iterator for CopiedIter<'a, T>
where
    T: Array,
{
    type Item = Option<T::Data>;

    fn next(&mut self) -> Option<Self::Item> {
        let idx = self.idx;
        self.idx += 1;

        if idx >= self.array.len() {
            None
        } else {
            Some(self.array.get(idx))
        }
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.array.len() - self.idx
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.array.len() - self.idx;
        (len, Some(len))
    }
}

impl<'a, T> ExactSizeIterator for CopiedIter<'a, T>
where
    T: Array,
{
    fn len(&self) -> usize {
        self.array.len() - self.idx
    }
}

pub struct IntoIter<T: Array> {
    array: T,
    idx: usize,
}

impl<T> IntoIter<T>
where
    T: Array,
{
    pub fn new(array: T) -> Self {
        Self { array, idx: 0 }
    }
}

impl<T> Iterator for IntoIter<T>
where
    T: Array,
{
    type Item = Option<T::Data>;

    fn next(&mut self) -> Option<Self::Item> {
        let idx = self.idx;
        self.idx += 1;
        if idx >= self.array.len() {
            None
        } else {
            Some(self.array.get(idx))
        }
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.array.len() - self.idx
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.array.len() - self.idx;
        (len, Some(len))
    }
}

impl<T> ExactSizeIterator for IntoIter<T>
where
    T: Array,
{
    fn len(&self) -> usize {
        self.array.len() - self.idx
    }
}

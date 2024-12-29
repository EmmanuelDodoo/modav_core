use std::alloc::{self, Layout};
use std::fmt::Debug;
use std::ptr::{self, NonNull};

use super::utils::{Array, DataType, IntoIter};

pub type I32 = Option<i32>;

/// Column of `i32` conforming to Apache Arrow's fix sized primitive
/// layout
pub struct ArrayI32 {
    /// Pointer to the values buffer
    ptr: Option<NonNull<i32>>,
    /// Pointer to the validity buffer
    val_ptr: Option<NonNull<u8>>,
    /// The number of elements in the array
    len: usize,
    /// The number of nulls in the array
    nulls: usize,
}

impl ArrayI32 {
    /// Creates an [`ArrayI32`] from a vec.
    pub fn from_vec(values: Vec<I32>) -> Self {
        Self::from_sized_iter(values.into_iter())
    }

    fn from_sized_iter<S>(sized: S) -> Self
    where
        S: Iterator<Item = I32> + ExactSizeIterator,
    {
        let len = sized.len();

        if len == 0 {
            return Self {
                ptr: None,
                val_ptr: None,
                len: 0,
                nulls: 0,
            };
        }

        let (values_ptr, validity_ptr) = Self::allocate(len);

        let mut val_byte = 0_u8;
        let mut val_offset = 0;
        let mut nulls = 0;

        for (idx, value) in sized.into_iter().enumerate() {
            match value {
                Some(value) => {
                    unsafe { ptr::write(values_ptr.as_ptr().add(idx), value) };
                    let pos = 1 << (idx % 8);
                    val_byte |= pos;
                }
                None => {
                    nulls += 1;
                    let pos = !(1 << (idx % 8));
                    val_byte &= pos;
                }
            }

            if (idx + 1) % 8 == 0 {
                unsafe {
                    ptr::write(validity_ptr.as_ptr().add(val_offset), val_byte);
                }

                val_byte = 0_u8;
                val_offset += 1;
            }
        }

        // Condition in for loop wouldn't have been triggered for the write
        if len % 8 != 0 {
            unsafe { ptr::write(validity_ptr.as_ptr().add(val_offset), val_byte) };
        }

        if nulls == 0 {
            Self::dealloc_validity(Some(validity_ptr), len);
        }

        if nulls == len {
            Self::dealloc_values(Some(values_ptr), len);
            Self::dealloc_validity(Some(validity_ptr), len);

            return Self {
                ptr: None,
                val_ptr: None,
                len,
                nulls,
            };
        }

        Self {
            ptr: if nulls == len { None } else { Some(values_ptr) },
            val_ptr: if nulls == 0 { None } else { Some(validity_ptr) },
            len,
            nulls,
        }
    }

    /// Returns true if the validity buffers of `Self` and `Other` are equal.
    ///
    /// Assumes both buffers are equal in length.
    fn compare_validity(&self, other: &Self) -> bool {
        let buffer_len = (self.len + 7) / 8;

        match (self.val_ptr, other.val_ptr) {
            (Some(own), Some(other)) => {
                for offset in 0..buffer_len {
                    let own = unsafe { ptr::read(own.as_ptr().add(offset)) };
                    let other = unsafe { ptr::read(other.as_ptr().add(offset)) };

                    if own != other {
                        return false;
                    }
                }
            }
            (None, Some(_)) => return false,
            (Some(_), None) => return false,
            (None, None) => return true,
        }

        true
    }

    /// Returns true if the values of `Self` and `Other` are equal.
    ///
    /// Assumes both buffers are equal in length.
    fn compare_values(&self, other: &Self) -> bool {
        let len = self.len;

        for idx in 0..len {
            let own = self.get(idx);
            let other = other.get(idx);

            if own != other {
                return false;
            }
        }

        true
    }

    /// Allocates both values and validity buffers
    ///
    /// Must ensure len != 0
    fn allocate(len: usize) -> (NonNull<i32>, NonNull<u8>) {
        // Values
        let values_size = len * std::mem::size_of::<i32>();
        let values_layout = Layout::from_size_align(values_size, 8)
            .expect("ArrayI32: values size overflowed isize::max");

        let values_ptr = unsafe { alloc::alloc(values_layout) };

        let values_ptr = match NonNull::new(values_ptr as *mut i32) {
            Some(ptr) => ptr,
            None => alloc::handle_alloc_error(values_layout),
        };

        // Validity
        let validity_size = (len + 7) / 8;
        let validity_layout = Layout::from_size_align(validity_size, 8)
            .expect("ArrayI32: validity size overflowed isize::max");

        let validity_ptr = unsafe { alloc::alloc(validity_layout) };

        let validity_ptr = match NonNull::new(validity_ptr) {
            Some(ptr) => ptr,
            None => alloc::handle_alloc_error(validity_layout),
        };

        (values_ptr, validity_ptr)
    }

    fn dealloc_validity(ptr: Option<NonNull<u8>>, len: usize) {
        let Some(val_ptr) = ptr else { return };
        let validity_size = (len + 7) / 8;
        let validity_layout = Layout::from_size_align(validity_size, 8)
            .expect("ArrayI32 drop: validity size overflowed isize::max");
        let ptr = val_ptr.as_ptr();
        unsafe { alloc::dealloc(ptr, validity_layout) };
    }

    fn dealloc_values(ptr: Option<NonNull<i32>>, len: usize) {
        let Some(ptr) = ptr else { return };
        let values_size = len * std::mem::size_of::<i32>();
        let values_layout = Layout::from_size_align(values_size, 8)
            .expect("ArrayI32 drop: values size overflowed isize::max");
        let ptr = ptr.as_ptr() as *mut u8;

        unsafe { alloc::dealloc(ptr, values_layout) };
    }
}

impl Drop for ArrayI32 {
    fn drop(&mut self) {
        Self::dealloc_values(self.ptr, self.len());
        Self::dealloc_validity(self.val_ptr, self.len())
    }
}

impl Clone for ArrayI32 {
    fn clone(&self) -> Self {
        if self.len() == 0 {
            return Self {
                ptr: None,
                val_ptr: None,
                len: 0,
                nulls: 0,
            };
        }

        let (values_ptr, validity_ptr) = Self::allocate(self.len());

        let validity_ptr = match self.val_ptr {
            Some(ptr) => {
                let count = (self.len() + 7) / 8;
                unsafe { ptr::copy(ptr.as_ptr(), validity_ptr.as_ptr(), count) };
                Some(validity_ptr)
            }
            None => {
                Self::dealloc_validity(Some(validity_ptr), self.len());
                None
            }
        };

        let values_ptr = match self.ptr {
            Some(ptr) => {
                unsafe { ptr::copy(ptr.as_ptr(), values_ptr.as_ptr(), self.len()) };
                Some(values_ptr)
            }
            None => {
                Self::dealloc_values(Some(values_ptr), self.len());
                None
            }
        };

        Self {
            ptr: values_ptr,
            val_ptr: validity_ptr,
            len: self.len(),
            nulls: self.nulls,
        }
    }
}

impl Debug for ArrayI32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut vals = self
            .iter()
            .map(|val| match val {
                Some(val) => val.to_string(),
                None => "null".into(),
            })
            .peekable();

        let vals = {
            let mut acc = String::new();
            while let Some(val) = vals.next() {
                let join = match vals.peek() {
                    Some(_) => ", ",
                    None => "",
                };
                acc = format!("{acc}{val}{join}");
            }
            acc
        };

        write!(f, "ArrayI32 [{vals}]")
    }
}

impl PartialEq for ArrayI32 {
    fn eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }

        if self.nulls != other.nulls {
            return false;
        }

        if !self.compare_validity(other) {
            return false;
        }

        if !self.compare_values(other) {
            return false;
        }

        true
    }
}

impl Array for ArrayI32 {
    type Data = i32;
    type Ref<'a> = i32;

    fn new<I>(values: I) -> Self
    where
        I: IntoIterator<Item = Option<Self::Data>>,
        I::IntoIter: ExactSizeIterator,
    {
        Self::from_sized_iter(values.into_iter())
    }

    fn get(&self, idx: usize) -> Option<Self::Data> {
        if idx >= self.len {
            return None;
        }

        if self.check_null(idx) {
            return None;
        }

        let ptr = self.ptr?;
        let val = unsafe { ptr::read(ptr.as_ptr().add(idx)) };

        Some(val)
    }

    fn get_ref(&self, idx: usize) -> Option<Self::Ref<'_>> {
        self.get(idx)
    }

    fn len(&self) -> usize {
        self.len
    }

    fn data_type(&self) -> DataType {
        DataType::Int32
    }

    fn check_null(&self, idx: usize) -> bool {
        assert!(
            idx < self.len,
            "Tried to index {} when array length is {}",
            idx,
            self.len
        );

        if self.all_null() {
            return true;
        }

        let Some(val_ptr) = self.val_ptr else {
            return false;
        };

        let byte_index = idx / 8;

        let val_byte = unsafe { ptr::read(val_ptr.as_ptr().add(byte_index)) };

        val_byte & (1 << (idx % 8)) == 0
    }

    fn all_null(&self) -> bool {
        self.nulls == self.len
    }
}

impl Eq for ArrayI32 {}

impl From<Vec<i32>> for ArrayI32 {
    fn from(value: Vec<i32>) -> Self {
        Self::from_sized_iter(value.into_iter().map(Some))
    }
}

impl From<Vec<I32>> for ArrayI32 {
    fn from(value: Vec<I32>) -> Self {
        Self::from_vec(value)
    }
}

impl<const N: usize> From<&[i32; N]> for ArrayI32 {
    fn from(value: &[i32; N]) -> Self {
        Self::from_sized_iter(value.iter().map(|num| Some(*num)))
    }
}

impl<const N: usize> From<[i32; N]> for ArrayI32 {
    fn from(value: [i32; N]) -> Self {
        Self::from_sized_iter(value.into_iter().map(Some))
    }
}

impl<const N: usize> From<&[I32; N]> for ArrayI32 {
    fn from(value: &[I32; N]) -> Self {
        Self::from_sized_iter(value.iter().copied())
    }
}

impl<const N: usize> From<[I32; N]> for ArrayI32 {
    fn from(value: [I32; N]) -> Self {
        Self::from_sized_iter(value.into_iter())
    }
}

impl IntoIterator for ArrayI32 {
    type Item = Option<i32>;
    type IntoIter = IntoIter<Self>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter::new(self)
    }
}

impl From<ArrayI32> for Vec<Option<i32>> {
    fn from(value: ArrayI32) -> Self {
        value.into_iter().collect()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_partial_eq() {
        let one = (0..5)
            .into_iter()
            .map(|num| if num % 2 == 0 { Some(num) } else { None });
        // Some(0), None, Some(2), None, Some(4)
        let one = ArrayI32::new(one);
        assert!(!one.all_null());

        // Zero: Self equality
        assert_eq!(one, one);
        assert_eq!(one, one.clone());

        // One: Perfect case
        let two = vec![Some(0), None, Some(2), None, Some(4)];
        let two = ArrayI32::new(two);

        assert_eq!(one, two);
        // One: Symmetry
        assert_eq!(two, one);

        // Two: Varying order
        let two = vec![None, None, Some(0), Some(2), Some(4)];
        let two = ArrayI32::new(two);

        assert_ne!(one, two);

        // Two: Varying order
        let two = vec![None, Some(0), None, Some(2), Some(4)];
        let two = ArrayI32::new(two);

        assert_ne!(one, two);

        // Two: Varying order
        let two = vec![Some(0), Some(2), Some(4)];
        let two = ArrayI32::new(two);
        let three = vec![Some(4), Some(0), Some(2)];
        let three = ArrayI32::new(three);

        assert_ne!(three, two);

        // Four: Varying length
        let two = vec![Some(0), Some(2), Some(4)];
        let two = ArrayI32::new(two);

        assert_ne!(one, two);

        // Five: Varying null count
        let two = vec![None, None, None, None, Some(0)];
        let two = ArrayI32::new(two);

        assert_ne!(one, two);

        // Six: Varying element values
        let two = vec![Some(0), Some(2), Some(3)];
        let two = ArrayI32::new(two);
        let three = vec![Some(1), Some(2), Some(3)];
        let three = ArrayI32::new(three);

        assert_ne!(two, three);
    }

    #[test]
    fn test_into_iter() {
        let one = (0..5)
            .into_iter()
            .map(|num| if num % 2 == 0 { Some(num) } else { None });
        // Some(0), None, Some(2), None, Some(4)
        let one = ArrayI32::new(one);

        let mut iter = one.into_iter();

        assert_eq!(Some(0), iter.next().unwrap());
        assert_eq!(None, iter.next().unwrap());
        iter.next();
        assert_eq!(None, iter.next().unwrap());
        assert_eq!(Some(4), iter.next().unwrap());
    }

    #[test]
    fn test_all_nulls() {
        let one = vec![None, None, None, None, None];

        let one = ArrayI32::new(one);
        assert!(one.all_null());

        assert_eq!(5, one.len());

        assert!(one.check_null(0));

        assert!(one.check_null(2));

        assert!(one.check_null(4));

        let mut iter = one.into_iter();

        assert_eq!(None, iter.next().unwrap());
        iter.next();
        assert_eq!(None, iter.next().unwrap())
    }

    #[test]
    fn test_empty() {
        let one = vec![];
        let one = ArrayI32::new(one);

        assert_eq!(0, one.len());
    }
}

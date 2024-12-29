use std::alloc::{self, Layout};
use std::fmt::Debug;
use std::ptr::{self, NonNull};

use super::utils::{Array, DataType, IntoIter};

pub type Text = Option<String>;

/// Column of `booleans` conforming to Apache Arrow's variable sized primitive
/// layout
pub struct ArrayText {
    /// Pointer to the values buffer.
    ptr: Option<NonNull<u8>>,
    /// Pointer to the validity buffer.
    val_ptr: Option<NonNull<u8>>,
    /// Pointer to the offsets buffer.
    offsets_ptr: Option<NonNull<u64>>,
    /// The number of elements in the array.
    len: usize,
    /// The combined length of all strings in the array.
    str_len: usize,
    /// The number of nulls in the array.
    nulls: usize,
}

impl ArrayText {
    fn empty() -> Self {
        Self {
            ptr: None,
            val_ptr: None,
            offsets_ptr: None,
            len: 0,
            str_len: 0,
            nulls: 0,
        }
    }

    pub fn from_str_iter<'b, S>(sized: S) -> Self
    where
        S: Iterator<Item = &'b str> + ExactSizeIterator,
    {
        let temp = sized.map(|text| Some(text.into()));

        Self::from_sized_iter(temp)
    }

    fn from_sized_iter<S>(sized: S) -> Self
    where
        S: Iterator<Item = Text> + ExactSizeIterator,
    {
        let len = sized.len();

        if len == 0 {
            return Self::empty();
        }

        let mut str_len = 0;
        let mut collected = Vec::with_capacity(len);

        for text in sized {
            if let Some(text) = text.as_ref() {
                str_len += text.len();
            }

            collected.push(text)
        }

        if str_len == 0 {
            // Filled with nulls
            return Self {
                ptr: None,
                offsets_ptr: None,
                val_ptr: None,
                len,
                str_len: 0,
                nulls: len,
            };
        }

        let (values_ptr, offsets_ptr, validity_ptr) = Self::allocate(len, str_len);

        let mut val_byte = 0_u8;
        let mut val_offset = 0;
        let mut nulls = 0;
        let mut offset = 0;

        for (idx, text) in collected.into_iter().enumerate() {
            unsafe { ptr::write(offsets_ptr.as_ptr().add(idx), offset) };

            match text {
                Some(text) => {
                    unsafe {
                        ptr::copy(
                            text.as_ptr(),
                            values_ptr.as_ptr().add(offset as usize),
                            text.len(),
                        )
                    };

                    offset += text.len() as u64;
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

        unsafe { ptr::write(offsets_ptr.as_ptr().add(len), offset) };

        // Condition in for loop wouldn't have been triggered for the write
        if len % 8 != 0 {
            unsafe { ptr::write(validity_ptr.as_ptr().add(val_offset), val_byte) };
        }

        if nulls == 0 {
            Self::dealloc_validity(Some(validity_ptr), len);
        }

        if nulls == len {
            Self::dealloc_values(Some(values_ptr), str_len);
            Self::dealloc_offsets(Some(offsets_ptr), len);
            Self::dealloc_validity(Some(validity_ptr), len);

            return Self {
                ptr: None,
                offsets_ptr: None,
                val_ptr: None,
                len,
                str_len: 0,
                nulls,
            };
        }

        Self {
            ptr: if nulls == len { None } else { Some(values_ptr) },
            val_ptr: if nulls == 0 { None } else { Some(validity_ptr) },
            offsets_ptr: if nulls == len {
                None
            } else {
                Some(offsets_ptr)
            },
            len,
            str_len,
            nulls,
        }
    }

    /// Creates an [`ArrayText`] from a vec.
    pub fn from_vec(values: Vec<Text>) -> Self {
        Self::from_sized_iter(values.into_iter())
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

    /// Returns true if the offsets buffer of `Self` and `other` are equal.
    ///
    /// Assumes both buffers are equal in length.
    fn compare_offsets(&self, other: &Self) -> bool {
        let len = self.len + 1;

        match (self.offsets_ptr, other.offsets_ptr) {
            (Some(own), Some(other)) => {
                for offset in 0..len {
                    let own = unsafe { ptr::read(own.as_ptr().add(offset)) };
                    let other = unsafe { ptr::read(other.as_ptr().add(offset)) };

                    if own != other {
                        return false;
                    }
                }
            }
            (None, None) => return true,
            _ => return false,
        }

        true
    }

    /// Returns true if the values of `Self` and `Other` are equal.
    ///
    /// Assumes both buffers are equal in length.
    fn compare_values(&self, other: &Self) -> bool {
        match (self.ptr, other.ptr) {
            (Some(own), Some(other)) => {
                for idx in 0..self.str_len {
                    let own = unsafe { ptr::read(own.as_ptr().add(idx)) };
                    let other = unsafe { ptr::read(other.as_ptr().add(idx)) };

                    if own != other {
                        return false;
                    }
                }

                true
            }
            (None, None) => true,
            _ => false,
        }
    }

    fn get_str(&self, idx: usize) -> Option<&str> {
        if idx >= self.len {
            return None;
        }

        if self.check_null(idx) {
            return None;
        }

        let offsets = self.offsets_ptr?;
        let values = self.ptr?;

        let start = unsafe { ptr::read(offsets.as_ptr().add(idx)) } as usize;
        let end = unsafe { ptr::read(offsets.as_ptr().add(idx + 1)) } as usize;

        let slice = unsafe { std::slice::from_raw_parts(values.as_ptr().add(start), end - start) };

        std::str::from_utf8(slice).ok()
    }

    fn check_null(&self, idx: usize) -> bool {
        assert!(
            idx < self.len,
            "Tried to index {} when array length is {}",
            idx,
            self.len
        );

        if self.len == self.nulls {
            return true;
        }

        let Some(ptr) = self.val_ptr else {
            return false;
        };
        let byte_index = idx / 8;

        let byte = unsafe { ptr::read(ptr.as_ptr().add(byte_index)) };

        byte & (1 << (idx % 8)) == 0
    }

    /// Allocates the required buffers
    ///
    /// Must ensure len != 0 and str_len != 0
    fn allocate(len: usize, str_len: usize) -> (NonNull<u8>, NonNull<u64>, NonNull<u8>) {
        assert!(len != 0, "ArrayText: Tried to allocate 0 sized memory");
        assert!(str_len != 0, "ArrayText: Tried to allocate 0 sized memory");

        // Validity
        let validity_size = (len + 7) / 8;
        let validity_layout = Layout::from_size_align(validity_size, 8)
            .expect("ArrayText: validity size overflowed isize::max");
        let validity_ptr = unsafe { alloc::alloc(validity_layout) };
        let validity_ptr = match NonNull::new(validity_ptr) {
            Some(ptr) => ptr,
            None => alloc::handle_alloc_error(validity_layout),
        };

        // Offsets
        let offset_size = (len + 1) * std::mem::size_of::<u64>();
        let offset_layout = Layout::from_size_align(offset_size, 8)
            .expect("ArrayText: Offsets size overflowed isize::max");
        let offsets_ptr = unsafe { alloc::alloc(offset_layout) };
        let offsets_ptr = match NonNull::new(offsets_ptr as *mut u64) {
            Some(ptr) => ptr,
            None => alloc::handle_alloc_error(offset_layout),
        };

        // Data
        let values_size = str_len * std::mem::size_of::<u8>();
        let values_layout = Layout::from_size_align(values_size, 8)
            .expect("ArrayText: Values size overflowed isize::max");
        let values_ptr = unsafe { alloc::alloc(values_layout) };
        let values_ptr = match NonNull::new(values_ptr) {
            Some(ptr) => ptr,
            None => alloc::handle_alloc_error(values_layout),
        };

        (values_ptr, offsets_ptr, validity_ptr)
    }

    fn dealloc_validity(ptr: Option<NonNull<u8>>, len: usize) {
        let Some(val_ptr) = ptr else { return };
        let validity_size = (len + 7) / 8;
        let validity_layout = Layout::from_size_align(validity_size, 8)
            .expect("ArrayText drop: Validity size overflowed isize::max");
        let ptr = val_ptr.as_ptr();

        unsafe { alloc::dealloc(ptr, validity_layout) }
    }

    fn dealloc_values(ptr: Option<NonNull<u8>>, str_len: usize) {
        let Some(ptr) = ptr else { return };
        let size = str_len * std::mem::size_of::<u8>();
        let layout = Layout::from_size_align(size, 8)
            .expect("ArrayText drop: Values size overflowed isize::max");
        let ptr = ptr.as_ptr();

        unsafe { alloc::dealloc(ptr, layout) }
    }

    /// Does not add the 1
    fn dealloc_offsets(ptr: Option<NonNull<u64>>, len: usize) {
        let Some(ptr) = ptr else { return };
        let size = (len + 1) * std::mem::size_of::<u64>();
        let layout = Layout::from_size_align(size, 8)
            .expect("ArrayText drop: Offsets size overflowed isize::max");
        let ptr = ptr.as_ptr() as *mut u8;

        unsafe { alloc::dealloc(ptr, layout) }
    }
}

impl Array for ArrayText {
    type Data = String;
    type Ref<'a> = &'a str
        where Self: 'a;

    fn new<I>(values: I) -> Self
    where
        I: IntoIterator<Item = Option<Self::Data>>,
        I::IntoIter: ExactSizeIterator,
    {
        Self::from_sized_iter(values.into_iter())
    }

    fn get(&self, idx: usize) -> Option<Self::Data> {
        let text = self.get_str(idx)?;

        Some(text.into())
    }

    fn get_ref(&self, idx: usize) -> Option<Self::Ref<'_>> {
        self.get_str(idx)
    }

    fn len(&self) -> usize {
        self.len
    }

    fn data_type(&self) -> DataType {
        DataType::Text
    }

    fn check_null(&self, idx: usize) -> bool {
        self.check_null(idx)
    }

    fn all_null(&self) -> bool {
        self.nulls == self.len
    }
}

impl Drop for ArrayText {
    fn drop(&mut self) {
        Self::dealloc_offsets(self.offsets_ptr, self.len);
        Self::dealloc_values(self.ptr, self.str_len);
        Self::dealloc_validity(self.val_ptr, self.len);
    }
}

impl Clone for ArrayText {
    fn clone(&self) -> Self {
        if self.len == 0 || self.str_len == 0 {
            return Self::empty();
        }

        let (values_ptr, offset_ptr, validity_ptr) = Self::allocate(self.len, self.str_len);

        let values_ptr = match self.ptr {
            Some(ptr) => {
                unsafe { ptr::copy(ptr.as_ptr(), values_ptr.as_ptr(), self.str_len) };
                Some(values_ptr)
            }
            None => {
                Self::dealloc_values(Some(values_ptr), self.str_len);
                None
            }
        };

        let offsets_ptr = match self.offsets_ptr {
            Some(ptr) => {
                unsafe { ptr::copy(ptr.as_ptr(), offset_ptr.as_ptr(), self.len + 1) };
                Some(offset_ptr)
            }
            None => {
                Self::dealloc_offsets(Some(offset_ptr), self.len);
                None
            }
        };

        let validity_ptr = match self.val_ptr {
            Some(ptr) => {
                let count = (self.len + 7) / 8;
                unsafe { ptr::copy(ptr.as_ptr(), validity_ptr.as_ptr(), count) };
                Some(validity_ptr)
            }
            None => {
                Self::dealloc_validity(Some(validity_ptr), self.len);
                None
            }
        };

        Self {
            ptr: values_ptr,
            val_ptr: validity_ptr,
            offsets_ptr,
            len: self.len,
            str_len: self.str_len,
            nulls: self.nulls,
        }
    }
}

impl PartialEq for ArrayText {
    fn eq(&self, other: &Self) -> bool {
        if self.len != other.len {
            return false;
        }

        if self.str_len != other.str_len {
            return false;
        }

        if self.nulls != other.nulls {
            return false;
        }

        if !self.compare_validity(other) {
            return false;
        }

        if !self.compare_offsets(other) {
            return false;
        }

        if !self.compare_values(other) {
            return false;
        }

        true
    }
}

impl Eq for ArrayText {}

impl IntoIterator for ArrayText {
    type Item = Option<String>;
    type IntoIter = IntoIter<Self>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter::new(self)
    }
}

impl Debug for ArrayText {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut vals = self.iter().map(|val| val.unwrap_or("null")).peekable();

        let vals = {
            let mut acc = String::new();
            while let Some(val) = vals.next() {
                let join = match vals.peek() {
                    Some(_) => ", ",
                    None => "",
                };
                acc = format!("{acc}\"{val}\"{join}");
            }
            acc
        };

        write!(f, "ArrayText [{vals}]")
    }
}

impl From<ArrayText> for Vec<Option<String>> {
    fn from(value: ArrayText) -> Self {
        value.into_iter().collect()
    }
}

impl From<Vec<String>> for ArrayText {
    fn from(value: Vec<String>) -> Self {
        Self::from_sized_iter(value.into_iter().map(Some))
    }
}

impl From<Vec<&str>> for ArrayText {
    fn from(value: Vec<&str>) -> Self {
        Self::from_sized_iter(value.into_iter().map(|text| Some(text.into())))
    }
}

impl From<Vec<Text>> for ArrayText {
    fn from(value: Vec<Text>) -> Self {
        Self::from_vec(value)
    }
}

impl<const N: usize> From<&[String; N]> for ArrayText {
    fn from(value: &[String; N]) -> Self {
        Self::from_sized_iter(value.iter().cloned().map(Some))
    }
}

impl<const N: usize> From<&[&str; N]> for ArrayText {
    fn from(value: &[&str; N]) -> Self {
        Self::from_sized_iter(value.iter().map(|text| Some(Into::<String>::into(*text))))
    }
}

impl<const N: usize> From<&[Text; N]> for ArrayText {
    fn from(value: &[Text; N]) -> Self {
        Self::from_sized_iter(value.iter().cloned())
    }
}

impl<const N: usize> From<[String; N]> for ArrayText {
    fn from(value: [String; N]) -> Self {
        Self::from_sized_iter(value.into_iter().map(Some))
    }
}

impl<const N: usize> From<[&str; N]> for ArrayText {
    fn from(value: [&str; N]) -> Self {
        Self::from_sized_iter(value.into_iter().map(|text| Some(text.into())))
    }
}

impl<const N: usize> From<[Text; N]> for ArrayText {
    fn from(value: [Text; N]) -> Self {
        Self::from_sized_iter(value.into_iter())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_partial_eq() {
        let one = [
            Some("For the faith,".into()),
            None,
            Some("For the way of the sword".into()),
            None,
            Some("Come and tell their story again".into()),
        ];
        let one = ArrayText::new(one);
        assert!(!one.all_null());

        // Zero: Self equality
        assert_eq!(one, one);
        assert_eq!(one, one.clone());

        // One: Perfect case
        let two = [
            Some("For the faith,".into()),
            None,
            Some("For the way of the sword".into()),
            None,
            Some("Come and tell their story again".into()),
        ];
        let two = ArrayText::new(two);

        assert_eq!(one, two);
        // One: Symmetry
        assert_eq!(two, one);

        // Two: Varying order
        let two = [
            None,
            None,
            Some("For the faith,".into()),
            Some("For the way of the sword".into()),
            Some("Come and tell their story again".into()),
        ];
        let two = ArrayText::new(two);

        assert_ne!(one, two);

        // Two: Varying order
        let two = [
            Some("For the faith,".into()),
            Some("For the way of the sword".into()),
            Some("Come and tell their story again".into()),
            None,
            None,
        ];
        let two = ArrayText::new(two);

        assert_ne!(one, two);

        // Two: Varying order
        let two = vec!["Welcome", "To Cosco", "I love you 💚"];
        let two = Into::<ArrayText>::into(two);
        let three = vec!["I love you 💚", "Welcome", "To Cosco"];
        let three = ArrayText::from_str_iter(three.into_iter());

        assert_ne!(three, two);

        // Four: Varying length
        let two = [
            Some("For the faith,".into()),
            Some("For the way of the sword".into()),
            Some("Come and tell their story again".into()),
        ];
        let two = ArrayText::new(two);

        assert_ne!(one, two);

        // Five: Varying null count
        let two = vec![None, None, None, None, Some("For the faith".into())];
        let two = ArrayText::new(two);

        assert_ne!(one, two);

        // Six: Varying element values
        let two = [
            Some("Perversions of ideals of science".into()),
            None,
            Some("Lost words of alienated wife".into()),
            None,
            Some("... Maths or morality alone?".into()),
        ];
        let two = ArrayText::new(two);
        let three = [
            Some("Where will this lead?".into()),
            None,
            Some("What's coming next".into()),
            None,
            Some("From your inventions?".into()),
        ];
        let three = ArrayText::new(three);

        assert_ne!(two, three);
    }

    #[test]
    fn test_into_iter() {
        let one = [
            Some("It was".into()),
            None,
            None,
            Some("The best".into()),
            Some("Of times.".into()),
            None,
            Some("It was the worst".into()),
            Some("of times".into()),
        ];

        let one = ArrayText::new(one);

        let mut iter = one.into_iter();

        assert_eq!(Some("It was".into()), iter.next().unwrap());
        assert_eq!(None, iter.next().unwrap());
        iter.next();
        assert_eq!(Some("The best".into()), iter.next().unwrap());
        assert_eq!(Some("Of times.".into()), iter.next().unwrap());
    }

    #[test]
    fn test_all_nulls() {
        let one: Vec<Option<String>> = vec![None, None, None, None, None];

        let one = ArrayText::new(one);

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
        let one: Vec<Option<String>> = vec![];
        let one = ArrayText::new(one);

        assert!(one.is_empty());
        assert_eq!(0, one.len());
    }
}

use std::alloc::{self, Layout};
use std::convert::Infallible;
use std::fmt::Debug;
use std::ptr::{self, NonNull};
use std::str::FromStr;

use super::utils::{Array, DataType, IntoIter};
use super::{
    ArrayBoolean, ArrayF32, ArrayF64, ArrayI32, ArrayISize, ArrayText, ArrayU32, ArrayUSize,
};

#[derive(Debug, Clone, PartialEq)]
/// Owned value contained in a [`Union`].
pub enum UnionType {
    U32(u32),
    I32(i32),
    USize(usize),
    ISize(isize),
    F32(f32),
    F64(f64),
    Boolean(bool),
    Text(String),
    Null,
}

impl UnionType {
    /// Attempts to parse `input` into a [`UnionType`].
    ///
    /// Both an empty string and the string `"null"` are parsed as [`UnionType::Null`].
    pub fn parse(input: impl Into<String>) -> Self {
        let input: String = input.into();

        if input.is_empty() || input == *"null" {
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

    pub fn borrow(&self) -> UnionRef<'_> {
        match self {
            Self::U32(val) => UnionRef::U32(*val),
            Self::USize(val) => UnionRef::USize(*val),
            Self::I32(val) => UnionRef::I32(*val),
            Self::ISize(val) => UnionRef::ISize(*val),
            Self::F32(val) => UnionRef::F32(*val),
            Self::F64(val) => UnionRef::F64(*val),
            Self::Boolean(val) => UnionRef::Boolean(*val),
            Self::Text(val) => UnionRef::Text(val),
            Self::Null => UnionRef::Null,
        }
    }

    fn convert_option(input: Option<Self>) -> Self {
        input.unwrap_or(Self::Null)
    }
}

#[derive(Debug, Clone, PartialEq)]
/// Shared reference to values in a [`Union`].
pub enum UnionRef<'a> {
    U32(u32),
    I32(i32),
    USize(usize),
    ISize(isize),
    F32(f32),
    F64(f64),
    Boolean(bool),
    Text(&'a str),
    Null,
}

impl<'a> UnionRef<'a> {
    #[allow(clippy::wrong_self_convention)]
    pub fn to_owned(self) -> UnionType {
        match self {
            Self::U32(val) => UnionType::U32(val),
            Self::USize(val) => UnionType::USize(val),
            Self::I32(val) => UnionType::I32(val),
            Self::ISize(val) => UnionType::ISize(val),
            Self::F32(val) => UnionType::F32(val),
            Self::F64(val) => UnionType::F64(val),
            Self::Boolean(val) => UnionType::Boolean(val),
            Self::Text(val) => UnionType::Text(val.to_owned()),
            Self::Null => UnionType::Null,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct UnionBuilder {
    tracker: Vec<(u8, usize)>,
    /// 0
    uint32: Vec<u32>,
    /// 1
    int32: Vec<i32>,
    /// 2
    uintsize: Vec<usize>,
    /// 3
    intsize: Vec<isize>,
    /// 4
    float32: Vec<f32>,
    /// 5
    float64: Vec<f64>,
    /// 6
    boolean: Vec<bool>,
    /// 7
    text: Vec<String>,
}

impl UnionBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, kind: UnionType) {
        match kind {
            UnionType::Null => self.push_none(),
            UnionType::U32(value) => self.push_u32(value),
            UnionType::I32(value) => self.push_i32(value),
            UnionType::USize(value) => self.push_usize(value),
            UnionType::ISize(value) => self.push_isize(value),
            UnionType::F32(value) => self.push_f32(value),
            UnionType::F64(value) => self.push_f64(value),
            UnionType::Boolean(value) => self.push_bool(value),
            UnionType::Text(value) => self.push_string(value),
        }
    }

    pub fn from_sized_iter<S>(sized: S) -> Self
    where
        S: Iterator<Item = UnionType> + ExactSizeIterator,
    {
        let mut own = Self::new();

        for elem in sized {
            own.push(elem);
        }

        own
    }

    pub fn from_sized_iter_str<S, V>(sized: S) -> Self
    where
        S: Iterator<Item = V> + ExactSizeIterator,
        V: Into<String>,
    {
        let mut own = Self::new();

        for elem in sized {
            own.parse_push(elem);
        }

        own
    }

    /// Attempts to parse `input` into a supported type, pushing the result onto
    /// self.
    ///
    /// Both an empty string and the string `"null"` are parsed as None.
    pub fn parse_push(&mut self, input: impl Into<String>) {
        let input: String = input.into();

        if input.is_empty() || input == *"null" {
            self.push_none();
            return;
        }

        if let Ok(parsed_u32) = input.parse::<u32>() {
            self.push_u32(parsed_u32);
            return;
        }

        if let Ok(parsed_i32) = input.parse::<i32>() {
            self.push_i32(parsed_i32);
            return;
        }

        if let Ok(parsed_usize) = input.parse::<usize>() {
            self.push_usize(parsed_usize);
            return;
        }

        if let Ok(parsed_isize) = input.parse::<isize>() {
            self.push_isize(parsed_isize);
            return;
        }

        if let Ok(parsed_f32) = input.parse::<f32>() {
            self.push_f32(parsed_f32);
            return;
        }

        if let Ok(parsed_f64) = input.parse::<f64>() {
            self.push_f64(parsed_f64);
            return;
        }

        if let Ok(parsed_bool) = input.parse::<bool>() {
            self.push_bool(parsed_bool);
            return;
        };

        self.push_string(input);
    }

    pub fn get(&self, idx: usize) -> Option<UnionType> {
        assert!(
            idx < self.tracker.len(),
            "Tried to index {} when array length is {}",
            idx,
            self.tracker.len()
        );

        let (kind, offset) = self.tracker[idx];

        match kind {
            0 => self.uint32.get(offset).copied().map(UnionType::U32),
            1 => self.int32.get(offset).copied().map(UnionType::I32),
            2 => self.uintsize.get(offset).copied().map(UnionType::USize),
            3 => self.intsize.get(offset).copied().map(UnionType::ISize),
            4 => self.float32.get(offset).copied().map(UnionType::F32),
            5 => self.float64.get(offset).copied().map(UnionType::F64),
            6 => self.boolean.get(offset).copied().map(UnionType::Boolean),
            7 => self.text.get(offset).cloned().map(UnionType::Text),
            8 => Some(UnionType::Null),
            _ => panic!("Tried to access beyond type support"),
        }
    }

    pub fn push_u32(&mut self, value: u32) {
        self.tracker.push((0, self.uint32.len()));
        self.uint32.push(value)
    }

    pub fn push_i32(&mut self, value: i32) {
        self.tracker.push((1, self.int32.len()));
        self.int32.push(value)
    }

    pub fn push_usize(&mut self, value: usize) {
        self.tracker.push((2, self.uintsize.len()));
        self.uintsize.push(value)
    }

    pub fn push_isize(&mut self, value: isize) {
        self.tracker.push((3, self.intsize.len()));
        self.intsize.push(value)
    }

    pub fn push_f32(&mut self, value: f32) {
        self.tracker.push((4, self.float32.len()));
        self.float32.push(value)
    }

    pub fn push_f64(&mut self, value: f64) {
        self.tracker.push((5, self.float64.len()));
        self.float64.push(value)
    }

    pub fn push_bool(&mut self, value: bool) {
        self.tracker.push((6, self.boolean.len()));
        self.boolean.push(value)
    }

    pub fn push_string(&mut self, value: String) {
        self.tracker.push((7, self.text.len()));
        self.text.push(value)
    }

    pub fn push_none(&mut self) {
        self.tracker.push((8, 0));
    }

    pub fn len(&self) -> usize {
        self.tracker.len()
    }
}

/// An array of mixed types corresponding to Apache Arrow's Dense Union type
pub struct Union {
    /// Pointer to the types buffer.
    types_ptr: Option<NonNull<u8>>,
    /// Pointer to the offsets buffer.
    offsets_ptr: Option<NonNull<u32>>,
    /// The number of elements in the array.
    len: usize,
    /// The number of null elements in the array.
    nulls: usize,

    /// type: 0
    uint32: Option<ArrayU32>,
    /// type: 1
    int32: Option<ArrayI32>,
    /// type: 2
    uintsize: Option<ArrayUSize>,
    /// type: 3
    intsize: Option<ArrayISize>,
    /// type: 4
    float32: Option<ArrayF32>,
    /// type: 5
    float64: Option<ArrayF64>,
    /// type: 6
    boolean: Option<ArrayBoolean>,
    /// type: 7
    text: Option<ArrayText>,
}

impl Union {
    fn empty() -> Self {
        Self {
            types_ptr: None,
            offsets_ptr: None,
            len: 0,
            nulls: 0,
            uint32: None,
            int32: None,
            uintsize: None,
            intsize: None,
            float32: None,
            float64: None,
            boolean: None,
            text: None,
        }
    }

    fn from_sized_iter<S>(sized: S) -> Self
    where
        S: Iterator<Item = UnionType> + ExactSizeIterator,
    {
        let builder = UnionBuilder::from_sized_iter(sized);
        Self::from_builder(builder)
    }

    pub fn from_builder(builder: UnionBuilder) -> Self {
        let len = builder.len();

        if len == 0 {
            return Self::empty();
        }

        let (types_ptr, offsets_ptr) = Self::allocate(builder.len());
        let mut nulls = 0;
        let UnionBuilder {
            tracker,
            uint32,
            int32,
            uintsize,
            intsize,
            float32,
            float64,
            boolean,
            text,
        } = builder;

        for (idx, (types, offset)) in tracker.into_iter().enumerate() {
            let offset = offset as u32;

            if types == 8 {
                nulls += 1;
            }

            unsafe { ptr::write(types_ptr.as_ptr().add(idx), types) };

            unsafe { ptr::write(offsets_ptr.as_ptr().add(idx), offset) };
        }

        if nulls == len {
            Self::dealloc_types(Some(types_ptr), len);
            Self::dealloc_offsets(Some(offsets_ptr), len);

            return Self {
                types_ptr: None,
                offsets_ptr: None,
                len,
                nulls,
                uintsize: None,
                uint32: None,
                intsize: None,
                int32: None,
                float64: None,
                float32: None,
                boolean: None,
                text: None,
            };
        }

        let uint32 = if uint32.is_empty() {
            None
        } else {
            Some(Into::<ArrayU32>::into(uint32))
        };

        let int32 = if int32.is_empty() {
            None
        } else {
            Some(Into::<ArrayI32>::into(int32))
        };

        let uintsize = if uintsize.is_empty() {
            None
        } else {
            Some(Into::<ArrayUSize>::into(uintsize))
        };

        let intsize = if intsize.is_empty() {
            None
        } else {
            Some(Into::<ArrayISize>::into(intsize))
        };

        let float32 = if float32.is_empty() {
            None
        } else {
            Some(Into::<ArrayF32>::into(float32))
        };

        let float64 = if float64.is_empty() {
            None
        } else {
            Some(Into::<ArrayF64>::into(float64))
        };

        let boolean = if boolean.is_empty() {
            None
        } else {
            Some(Into::<ArrayBoolean>::into(boolean))
        };

        let text = if text.is_empty() {
            None
        } else {
            Some(Into::<ArrayText>::into(text))
        };

        Self {
            types_ptr: Some(types_ptr),
            offsets_ptr: Some(offsets_ptr),
            len,
            nulls,

            uint32,
            int32,
            intsize,
            uintsize,
            float32,
            float64,
            boolean,
            text,
        }
    }

    /// Creates an [`Union`] from a vec.
    pub fn from_vec(values: Vec<UnionType>) -> Self {
        Self::from_sized_iter(values.into_iter())
    }

    /// Returns true if the types buffers of `Self` and `Other` are equal.
    ///
    /// Assumes both buffers are equal in length.
    fn compare_types(&self, other: &Self) -> bool {
        match (self.types_ptr, other.types_ptr) {
            (Some(own), Some(other)) => {
                for offset in 0..self.len {
                    let own = unsafe { ptr::read(own.as_ptr().add(offset)) };
                    let other = unsafe { ptr::read(other.as_ptr().add(offset)) };

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

    /// Returns true if the offsets buffers of `Self` and `Other` are equal.
    ///
    /// Assumes both buffers are equal in length.
    fn compare_offsets(&self, other: &Self) -> bool {
        match (self.offsets_ptr, other.offsets_ptr) {
            (Some(own), Some(other)) => {
                for offset in 0..self.len {
                    let own = unsafe { ptr::read(own.as_ptr().add(offset)) };
                    let other = unsafe { ptr::read(other.as_ptr().add(offset)) };

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

    fn get_helper(&self, kind: u8, offset: usize) -> Option<UnionType> {
        match kind {
            0 => {
                let value = self.uint32.as_ref()?.get(offset)?;
                Some(UnionType::U32(value))
            }
            1 => {
                let value = self.int32.as_ref()?.get(offset)?;
                Some(UnionType::I32(value))
            }
            2 => {
                let value = self.uintsize.as_ref()?.get(offset)?;
                Some(UnionType::USize(value))
            }
            3 => {
                let value = self.intsize.as_ref()?.get(offset)?;
                Some(UnionType::ISize(value))
            }
            4 => {
                let value = self.float32.as_ref()?.get(offset)?;
                Some(UnionType::F32(value))
            }
            5 => {
                let value = self.float64.as_ref()?.get(offset)?;
                Some(UnionType::F64(value))
            }
            6 => {
                let value = self.boolean.as_ref()?.get(offset)?;
                Some(UnionType::Boolean(value))
            }
            7 => {
                let value = self.text.as_ref()?.get(offset)?;
                Some(UnionType::Text(value))
            }
            8 => Some(UnionType::Null),
            _ => panic!("Union: Code should really not reach here!"),
        }
    }

    fn get_ref_helper(&self, kind: u8, offset: usize) -> Option<UnionRef<'_>> {
        match kind {
            0 => {
                let value = self.uint32.as_ref()?.get(offset)?;
                Some(UnionRef::U32(value))
            }
            1 => {
                let value = self.int32.as_ref()?.get(offset)?;
                Some(UnionRef::I32(value))
            }
            2 => {
                let value = self.uintsize.as_ref()?.get(offset)?;
                Some(UnionRef::USize(value))
            }
            3 => {
                let value = self.intsize.as_ref()?.get(offset)?;
                Some(UnionRef::ISize(value))
            }
            4 => {
                let value = self.float32.as_ref()?.get(offset)?;
                Some(UnionRef::F32(value))
            }
            5 => {
                let value = self.float64.as_ref()?.get(offset)?;
                Some(UnionRef::F64(value))
            }
            6 => {
                let value = self.boolean.as_ref()?.get(offset)?;
                Some(UnionRef::Boolean(value))
            }
            7 => {
                let value = self.text.as_ref()?.get_ref(offset)?;
                Some(UnionRef::Text(value))
            }
            8 => Some(UnionRef::Null),
            _ => panic!("Union: Code should really not reach here!"),
        }
    }

    /// Allocates both types and offset buffers
    ///
    /// Must ensure len != 0
    fn allocate(len: usize) -> (NonNull<u8>, NonNull<u32>) {
        // Offsets
        let offsets_size = len * std::mem::size_of::<u32>();
        let offsets_layout = Layout::from_size_align(offsets_size, 8)
            .expect("Union: offsets size overflowed isize::max");

        let offsets_ptr = unsafe { alloc::alloc(offsets_layout) };

        let offsets_ptr = match NonNull::new(offsets_ptr as *mut u32) {
            Some(ptr) => ptr,
            None => alloc::handle_alloc_error(offsets_layout),
        };

        // Types
        let types_size = (len + 7) / 8;
        let types_layout = Layout::from_size_align(types_size, 8)
            .expect("Union: types size overflowed isize::max");

        let types_ptr = unsafe { alloc::alloc(types_layout) };

        let types_ptr = match NonNull::new(types_ptr) {
            Some(ptr) => ptr,
            None => alloc::handle_alloc_error(types_layout),
        };

        (types_ptr, offsets_ptr)
    }

    fn dealloc_types(ptr: Option<NonNull<u8>>, len: usize) {
        let Some(ptr) = ptr else { return };
        let size = len * std::mem::size_of::<u8>();
        let layout =
            Layout::from_size_align(size, 8).expect("Union: types size overflowed isize::max");
        let ptr = ptr.as_ptr();

        unsafe { alloc::dealloc(ptr, layout) }
    }

    fn dealloc_offsets(ptr: Option<NonNull<u32>>, len: usize) {
        let Some(ptr) = ptr else { return };
        let offsets_size = len * std::mem::size_of::<u32>();
        let offsets_layout = Layout::from_size_align(offsets_size, 8)
            .expect("Union: offsets size overflowed isize::max");

        let ptr = ptr.as_ptr() as *mut u8;

        unsafe { alloc::dealloc(ptr, offsets_layout) }
    }
}

impl Array for Union {
    type Data = UnionType;
    type Ref<'a> = UnionRef<'a>
        where Self: 'a;

    fn new<I>(values: I) -> Self
    where
        I: IntoIterator<Item = Option<Self::Data>>,
        I::IntoIter: ExactSizeIterator,
    {
        Self::from_sized_iter(values.into_iter().map(UnionType::convert_option))
    }

    fn get(&self, idx: usize) -> Option<Self::Data> {
        if idx >= self.len {
            return None;
        }

        if self.check_null(idx) {
            return Some(UnionType::Null);
        }

        let offsets_ptr = self.offsets_ptr?;
        let offset = unsafe { ptr::read(offsets_ptr.as_ptr().add(idx)) };

        let types_ptr = self.types_ptr?;
        let kind = unsafe { ptr::read(types_ptr.as_ptr().add(idx)) };

        self.get_helper(kind, offset as usize)
    }

    fn get_ref(&self, idx: usize) -> Option<Self::Ref<'_>> {
        if idx >= self.len {
            return None;
        }

        if self.check_null(idx) {
            return Some(UnionRef::Null);
        }

        let offsets_ptr = self.offsets_ptr?;
        let offset = unsafe { ptr::read(offsets_ptr.as_ptr().add(idx)) };

        let types_ptr = self.types_ptr?;
        let kind = unsafe { ptr::read(types_ptr.as_ptr().add(idx)) };

        self.get_ref_helper(kind, offset as usize)
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

        let Some(types_ptr) = self.types_ptr else {
            return false;
        };

        let kind = unsafe { ptr::read(types_ptr.as_ptr().add(idx)) };

        kind == 8
    }

    fn len(&self) -> usize {
        self.len
    }

    fn data_type(&self) -> DataType {
        DataType::Union
    }

    fn all_null(&self) -> bool {
        self.nulls == self.len
    }
}

impl IntoIterator for Union {
    type Item = Option<UnionType>;
    type IntoIter = IntoIter<Self>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter::new(self)
    }
}

impl Drop for Union {
    fn drop(&mut self) {
        Self::dealloc_offsets(self.offsets_ptr, self.len);
        Self::dealloc_types(self.types_ptr, self.len);
    }
}

impl Clone for Union {
    fn clone(&self) -> Self {
        if self.len == 0 {
            return Self::empty();
        }

        let (types_ptr, offsets_ptr) = Self::allocate(self.len);

        let types_ptr = match self.types_ptr {
            Some(ptr) => {
                unsafe { ptr::copy(ptr.as_ptr(), types_ptr.as_ptr(), self.len) };
                Some(types_ptr)
            }
            None => {
                Self::dealloc_types(Some(types_ptr), self.len);
                None
            }
        };

        let offsets_ptr = match self.offsets_ptr {
            Some(ptr) => {
                unsafe { ptr::copy(ptr.as_ptr(), offsets_ptr.as_ptr(), self.len) };
                Some(offsets_ptr)
            }
            None => {
                Self::dealloc_offsets(Some(offsets_ptr), self.len);
                None
            }
        };

        Self {
            types_ptr,
            offsets_ptr,
            len: self.len,
            nulls: self.nulls,

            uint32: self.uint32.clone(),
            int32: self.int32.clone(),
            uintsize: self.uintsize.clone(),
            intsize: self.intsize.clone(),
            float32: self.float32.clone(),
            float64: self.float64.clone(),
            boolean: self.boolean.clone(),
            text: self.text.clone(),
        }
    }
}

impl PartialEq for Union {
    fn eq(&self, other: &Self) -> bool {
        if self.len != other.len {
            return false;
        }

        if self.nulls != other.nulls {
            return false;
        }

        if !self.compare_types(other) {
            return false;
        }

        if !self.compare_offsets(other) {
            return false;
        }

        self.uint32 == other.uint32
            && self.int32 == other.int32
            && self.uintsize == other.uintsize
            && self.intsize == other.intsize
            && self.float32 == other.float32
            && self.float64 == other.float64
            && self.boolean == other.boolean
            && self.text == other.text
    }
}

impl Debug for Union {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut vals = self
            .iter()
            .map(|val| match val {
                Some(val) => {
                    format!("{val:?}")
                }
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

        write!(f, "Union [{vals}]")
    }
}

impl From<Option<UnionType>> for UnionType {
    fn from(value: Option<UnionType>) -> Self {
        Self::convert_option(value)
    }
}

impl From<String> for UnionType {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

impl From<Option<String>> for UnionType {
    fn from(value: Option<String>) -> Self {
        match value {
            Some(value) => Self::Text(value),
            None => Self::Null,
        }
    }
}

impl From<&str> for UnionType {
    fn from(value: &str) -> Self {
        Self::Text(value.to_string())
    }
}

impl From<Option<&str>> for UnionType {
    fn from(value: Option<&str>) -> Self {
        match value {
            Some(value) => Self::Text(value.into()),
            None => Self::Null,
        }
    }
}

impl From<bool> for UnionType {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

impl From<Option<bool>> for UnionType {
    fn from(value: Option<bool>) -> Self {
        match value {
            Some(value) => Self::Boolean(value),
            None => Self::Null,
        }
    }
}

impl From<f64> for UnionType {
    fn from(value: f64) -> Self {
        Self::F64(value)
    }
}

impl From<Option<f64>> for UnionType {
    fn from(value: Option<f64>) -> Self {
        match value {
            Some(value) => Self::F64(value),
            None => Self::Null,
        }
    }
}

impl From<f32> for UnionType {
    fn from(value: f32) -> Self {
        Self::F32(value)
    }
}

impl From<Option<f32>> for UnionType {
    fn from(value: Option<f32>) -> Self {
        match value {
            Some(value) => Self::F32(value),
            None => Self::Null,
        }
    }
}

impl From<isize> for UnionType {
    fn from(value: isize) -> Self {
        Self::ISize(value)
    }
}

impl From<Option<isize>> for UnionType {
    fn from(value: Option<isize>) -> Self {
        match value {
            Some(value) => Self::ISize(value),
            None => Self::Null,
        }
    }
}

impl From<usize> for UnionType {
    fn from(value: usize) -> Self {
        Self::USize(value)
    }
}

impl From<Option<usize>> for UnionType {
    fn from(value: Option<usize>) -> Self {
        match value {
            Some(value) => Self::USize(value),
            None => Self::Null,
        }
    }
}

impl From<i32> for UnionType {
    fn from(value: i32) -> Self {
        Self::I32(value)
    }
}

impl From<Option<i32>> for UnionType {
    fn from(value: Option<i32>) -> Self {
        match value {
            Some(value) => Self::I32(value),
            None => Self::Null,
        }
    }
}

impl From<u32> for UnionType {
    fn from(value: u32) -> Self {
        Self::U32(value)
    }
}

impl From<Option<u32>> for UnionType {
    fn from(value: Option<u32>) -> Self {
        match value {
            Some(value) => Self::U32(value),
            None => Self::Null,
        }
    }
}

impl FromStr for UnionType {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::parse(s))
    }
}

impl From<Union> for Vec<Option<UnionType>> {
    fn from(value: Union) -> Self {
        value.into_iter().collect()
    }
}

impl From<Vec<UnionType>> for Union {
    fn from(value: Vec<UnionType>) -> Self {
        Self::from_sized_iter(value.into_iter())
    }
}

impl From<Vec<Option<UnionType>>> for Union {
    fn from(value: Vec<Option<UnionType>>) -> Self {
        Self::from_sized_iter(value.into_iter().map(UnionType::convert_option))
    }
}

impl<const N: usize> From<&[UnionType; N]> for Union {
    fn from(value: &[UnionType; N]) -> Self {
        Self::from_sized_iter(value.iter().cloned())
    }
}

impl<const N: usize> From<[UnionType; N]> for Union {
    fn from(value: [UnionType; N]) -> Self {
        Self::from_sized_iter(value.into_iter())
    }
}

impl<const N: usize> From<&[Option<UnionType>; N]> for Union {
    fn from(value: &[Option<UnionType>; N]) -> Self {
        Self::from_sized_iter(value.iter().cloned().map(UnionType::convert_option))
    }
}

impl<const N: usize> From<[Option<UnionType>; N]> for Union {
    fn from(value: [Option<UnionType>; N]) -> Self {
        Self::from_sized_iter(value.into_iter().map(UnionType::convert_option))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_partial_eq() {
        let one = ["one", "1", "1.00", "", "-14", "false", "null", "-25.0"];
        let builder = UnionBuilder::from_sized_iter_str(one.into_iter());

        let one = Union::from_builder(builder);
        assert!(!one.all_null());

        let alt = one.iter().map(|val| match val {
            Some(val) => Some(val.to_owned()),
            None => None,
        });
        let alt = Union::new(alt);

        // Zero: Self equality
        assert_eq!(one, one);
        assert_eq!(one, one.clone());
        assert_eq!(one, alt);

        // One: Perfect case
        let two = [
            UnionType::Text("one".into()),
            UnionType::U32(1),
            UnionType::F32(1.0),
            UnionType::Null,
            UnionType::I32(-14),
            UnionType::Boolean(false),
            UnionType::Null,
            UnionType::F32(-25.0),
        ];
        let two = Into::<Union>::into(two);

        assert_eq!(one, two);
        // One: Symmetry
        assert_eq!(two, one);

        // Two: Varying order
        let two = vec![
            UnionType::U32(1),
            UnionType::F32(1.0),
            UnionType::Text("one".into()),
            UnionType::Null,
            UnionType::I32(-14),
            UnionType::F32(-25.0),
            UnionType::Boolean(false),
            UnionType::Null,
        ];
        let two = Into::<Union>::into(two);

        assert_ne!(one, two);

        // Two: Varying order
        let two = vec![
            UnionType::F32(1.0),
            UnionType::U32(1),
            UnionType::Text("one".into()),
            UnionType::I32(-14),
            UnionType::F32(-25.0),
            UnionType::Null,
            UnionType::Boolean(false),
            UnionType::Null,
        ];
        let two = Into::<Union>::into(two);

        assert_ne!(one, two);

        // Two: Varying order
        let two = [
            UnionType::I32(4),
            UnionType::F64(2.0),
            UnionType::Text("0".into()),
        ];
        let two = Into::<Union>::into(two);
        let three = [
            UnionType::Text("0".into()),
            UnionType::I32(4),
            UnionType::F64(2.0),
        ];
        let three = Into::<Union>::into(three);

        assert_ne!(three, two);

        // Four: Varying length
        let two = [
            UnionType::I32(4),
            UnionType::Null,
            UnionType::F64(2.0),
            UnionType::Text("0".into()),
        ];
        let two = Into::<Union>::into(&two);

        assert_ne!(one, two);

        // Five: Nulls
        let two = [
            UnionType::Null,
            UnionType::Null,
            UnionType::Null,
            UnionType::Null,
            UnionType::Null,
        ];
        let two = Into::<Union>::into(&two);

        assert_ne!(one, two);
    }

    #[test]
    fn test_into_iter() {
        let one = [
            Some(UnionType::Text("one".into())),
            None,
            Some(UnionType::U32(1)),
            None,
            Some(UnionType::F32(1.00)),
        ];

        let one = Union::new(one);

        let mut iter = one.into_iter();

        assert_eq!(UnionType::Text("one".into()), iter.next().unwrap().unwrap());
        assert_eq!(Some(UnionType::Null), iter.next().unwrap());
        iter.next();
        assert_eq!(UnionType::Null, iter.next().unwrap().unwrap());
        assert_eq!(UnionType::F32(1.00), iter.next().unwrap().unwrap());
    }

    #[test]
    fn test_all_nulls() {
        let one = vec![None; 5];

        let one = Union::new(one);

        assert!(one.all_null());

        assert_eq!(5, one.len());

        assert!(one.check_null(0));

        assert!(one.check_null(2));

        assert!(one.check_null(4));

        let mut iter = one.into_iter();

        assert_eq!(Some(UnionType::Null), iter.next().unwrap());
        iter.next();
        assert_eq!(Some(UnionType::Null), iter.next().unwrap())
    }

    #[test]
    fn test_empty() {
        let one = vec![];
        let one = Union::new(one);

        assert_eq!(0, one.len())
    }

    #[test]
    fn test_mixed_builder() {
        let mut builder = UnionBuilder::new();

        let elems = ["one", "1", "1.00", "", "-14", "false", "null", "Buble"];
        elems.into_iter().for_each(|val| builder.parse_push(val));

        assert_eq!(8, builder.len());

        assert_eq!(UnionType::Text("one".into()), builder.get(0).unwrap());
        assert_eq!(UnionType::Null, builder.get(3).unwrap());
        assert_eq!(UnionType::I32(-14), builder.get(4).unwrap());
        assert_eq!(UnionType::Boolean(false), builder.get(5).unwrap());
        assert_eq!(UnionType::Null, builder.get(6).unwrap());

        let max = (i32::MAX as isize) + 1;
        builder.parse_push(max.to_string());
        assert_eq!(UnionType::U32(max as u32), builder.get(8).unwrap());

        let max = (f32::MAX as f64) * 1.5;
        builder.parse_push(max.to_string());
        assert_eq!(UnionType::F32(f32::INFINITY), builder.get(9).unwrap());
    }
}

use super::{parse_helper, parse_unchecked, utils::*, Iter, IterMut};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ArrayI32 {
    header: Option<String>,
    cells: Vec<Option<i32>>,
}

impl ArrayI32 {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_iterator(values: impl Iterator<Item = i32>) -> Self {
        Self {
            cells: values.map(Some).collect(),
            ..Default::default()
        }
    }

    pub fn from_iterator_option(values: impl Iterator<Item = Option<i32>>) -> Self {
        Self {
            cells: values.collect(),
            ..Default::default()
        }
    }

    pub fn set_header(&mut self, header: String) -> &mut Self {
        self.header = Some(header);
        self
    }

    pub fn get(&self, idx: usize) -> Option<i32> {
        self.cells.get(idx)?.as_ref().copied()
    }

    pub fn get_mut(&mut self, idx: usize) -> Option<&mut i32> {
        self.cells.get_mut(idx)?.as_mut()
    }

    pub fn iter(&self) -> Iter<'_, Option<i32>> {
        self.cells.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, Option<i32>> {
        self.cells.iter_mut()
    }

    pub fn parse_str(values: &Vec<String>) -> Option<Self> {
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
        let parsed = parse_unchecked(value);
        self.cells.push(parsed)
    }

    fn remove(&mut self, idx: usize) {
        if idx >= self.len() {
            return;
        }
        self.cells.remove(idx);
    }

    fn insert(&mut self, value: &str, idx: usize) {
        if idx > self.len() {
            return;
        }

        let parsed = parse_unchecked::<i32>(value);

        self.cells.insert(idx, parsed);
    }
}

impl Column for ArrayI32 {
    fn label(&self) -> Option<&str> {
        self.header.as_deref()
    }

    fn kind(&self) -> DataType {
        DataType::I32
    }

    fn len(&self) -> usize {
        self.cells.len()
    }

    fn set_header(&mut self, header: String) {
        self.header = Some(header);
    }

    fn set_position(&mut self, value: &str, idx: usize) {
        let parsed = parse_unchecked::<i32>(value);

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

    fn data_ref(&self, idx: usize) -> DataRef<'_> {
        match self.cells.get(idx).copied() {
            Some(Some(value)) => DataRef::I32(value),
            _ => DataRef::None,
        }
    }
}

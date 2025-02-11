use super::{parse_helper, parse_unchecked, Column, DataType, Iter, IterMut, Sealed};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ArrayUSize {
    header: Option<String>,
    cells: Vec<Option<usize>>,
}

impl ArrayUSize {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_iterator(values: impl Iterator<Item = usize>) -> Self {
        Self {
            cells: values.map(Some).collect(),
            ..Default::default()
        }
    }

    pub fn from_iterator_option(values: impl Iterator<Item = Option<usize>>) -> Self {
        Self {
            cells: values.collect(),
            ..Default::default()
        }
    }

    pub fn set_header(&mut self, header: String) -> &mut Self {
        self.header = Some(header);
        self
    }

    pub fn get(&self, idx: usize) -> Option<usize> {
        self.cells.get(idx)?.as_ref().copied()
    }

    pub fn get_mut(&mut self, idx: usize) -> Option<&mut usize> {
        self.cells.get_mut(idx)?.as_mut()
    }

    pub fn iter(&self) -> Iter<'_, Option<usize>> {
        self.cells.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, Option<usize>> {
        self.cells.iter_mut()
    }

    pub fn parse_str(values: &Vec<String>) -> Option<Self> {
        let mut cells = Vec::default();

        for value in values {
            let parsed = parse_helper::<usize>(value).ok()?;
            cells.push(parsed)
        }

        Some(Self {
            header: None,
            cells,
        })
    }
}

impl Sealed for ArrayUSize {
    fn push(&mut self, value: &str) {
        let value = parse_unchecked::<usize>(value);
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

        let parsed = parse_unchecked::<usize>(value);

        self.cells.insert(idx, parsed);
    }
}

impl Column for ArrayUSize {
    fn len(&self) -> usize {
        self.cells.len()
    }

    fn kind(&self) -> DataType {
        DataType::USize
    }

    fn label(&self) -> Option<&str> {
        self.header.as_deref()
    }

    fn set_header(&mut self, header: String) {
        self.header = Some(header)
    }

    fn set_position(&mut self, value: &str, idx: usize) {
        let parsed = parse_unchecked::<usize>(value);

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

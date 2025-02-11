use super::{parse_helper, Column, DataType, Iter, IterMut, Sealed};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ArrayText {
    header: Option<String>,
    cells: Vec<Option<String>>,
}

impl ArrayText {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_iterator(values: impl Iterator<Item = String>) -> Self {
        Self {
            cells: values.map(Some).collect(),
            ..Default::default()
        }
    }

    pub fn from_iterator_option(values: impl Iterator<Item = Option<String>>) -> Self {
        Self {
            cells: values.collect(),
            ..Default::default()
        }
    }

    pub fn set_header(&mut self, header: impl Into<String>) -> &mut Self {
        self.header = Some(header.into());
        self
    }

    pub fn get(&self, idx: usize) -> Option<String> {
        self.cells.get(idx)?.as_ref().cloned()
    }

    pub fn get_mut(&mut self, idx: usize) -> Option<&mut String> {
        self.cells.get_mut(idx)?.as_mut()
    }

    pub fn get_ref(&self, idx: usize) -> Option<&String> {
        self.cells.get(idx)?.as_ref()
    }

    pub fn iter(&self) -> Iter<'_, Option<String>> {
        self.cells.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, Option<String>> {
        self.cells.iter_mut()
    }

    pub fn parse_str(values: &Vec<String>) -> Self {
        let mut cells = Vec::default();

        for value in values {
            // Always successful
            let parsed = parse_helper(value).unwrap_or_default();
            cells.push(parsed);
        }

        Self {
            header: None,
            cells,
        }
    }
}

impl Sealed for ArrayText {
    fn push(&mut self, value: &str) {
        let parsed = parse_helper(value).unwrap_or_default();
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
        let parsed = parse_helper(value).unwrap_or_default();

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

    fn label(&self) -> Option<&str> {
        self.header.as_deref()
    }

    fn set_header(&mut self, header: String) {
        self.set_header(header);
    }

    fn set_position(&mut self, value: &str, idx: usize) {
        let parsed = parse_helper(value).unwrap_or_default();

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

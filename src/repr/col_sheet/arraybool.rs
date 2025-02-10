use super::{parse_helper, parse_unchecked, Column, DataType, Iter, IterMut, Sealed};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ArrayBool {
    header: Option<String>,
    cells: Vec<Option<bool>>,
}

impl ArrayBool {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_iterator(values: impl Iterator<Item = bool>) -> Self {
        Self {
            cells: values.map(Some).collect(),
            ..Default::default()
        }
    }

    pub fn from_iterator_option(values: impl Iterator<Item = Option<bool>>) -> Self {
        Self {
            cells: values.collect(),
            ..Default::default()
        }
    }

    pub fn set_header(&mut self, header: String) -> &mut Self {
        self.header = Some(header);
        self
    }

    pub fn get(&self, idx: usize) -> Option<bool> {
        self.cells.get(idx)?.as_ref().copied()
    }

    pub fn get_mut(&mut self, idx: usize) -> Option<&mut bool> {
        self.cells.get_mut(idx)?.as_mut()
    }

    pub fn iter(&self) -> Iter<'_, Option<bool>> {
        self.cells.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, Option<bool>> {
        self.cells.iter_mut()
    }

    pub fn parse_str(values: &Vec<String>) -> Option<Self> {
        let mut cells = Vec::default();

        for value in values {
            let parsed = parse_helper::<bool>(value).ok()?;
            cells.push(parsed)
        }

        Some(Self {
            header: None,
            cells,
        })
    }
}

impl Sealed for ArrayBool {
    fn push(&mut self, value: &str) {
        let parsed = parse_unchecked::<bool>(value);
        self.cells.push(parsed)
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

        let parsed = parse_unchecked::<bool>(value);

        self.cells.insert(idx, parsed);
    }
}

impl Column for ArrayBool {
    fn label(&self) -> Option<&String> {
        self.header.as_ref()
    }

    fn kind(&self) -> DataType {
        DataType::Bool
    }

    fn len(&self) -> usize {
        self.cells.len()
    }

    fn set_header(&mut self, header: String) {
        self.header = Some(header);
    }

    fn set_position<'a>(&mut self, value: &'a str, idx: usize) {
        let parsed = parse_unchecked::<bool>(value);

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
}

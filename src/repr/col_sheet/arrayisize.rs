use super::{parse_helper, Column, DataType, Iter, Sealed};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ArrayISize {
    header: Option<String>,
    cells: Vec<Option<isize>>,
}

impl ArrayISize {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_iterator(values: impl Iterator<Item = isize>) -> Self {
        Self {
            cells: values.map(Some).collect(),
            ..Default::default()
        }
    }

    pub fn from_iterator_option(values: impl Iterator<Item = Option<isize>>) -> Self {
        Self {
            cells: values.collect(),
            ..Default::default()
        }
    }

    pub fn set_header(&mut self, header: String) -> &mut Self {
        self.header = Some(header);
        self
    }

    pub fn get(&self, idx: usize) -> Option<isize> {
        self.cells.get(idx)?.as_ref().copied()
    }

    pub fn iter(&self) -> Iter<'_, Option<isize>> {
        self.cells.iter()
    }

    pub fn parse_str(values: &Vec<String>) -> Option<Self> {
        let mut cells = Vec::default();

        for value in values {
            let value = parse_helper::<isize>(value).ok()?;
            cells.push(value)
        }

        Some(Self {
            header: None,
            cells,
        })
    }
}

impl Sealed for ArrayISize {
    fn push(&mut self, value: &str) {
        let value = match parse_helper::<isize>(value) {
            Ok(val) => val,
            Err(_) => None,
        };
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

        let Ok(parsed) = parse_helper::<isize>(value) else {
            return;
        };

        self.cells.insert(idx, parsed);
    }
}

impl Column for ArrayISize {
    fn len(&self) -> usize {
        self.cells.len()
    }

    fn kind(&self) -> DataType {
        DataType::ISize
    }

    fn label(&self) -> Option<&String> {
        self.header.as_ref()
    }

    fn set_header(&mut self, header: String) {
        self.header = Some(header)
    }

    fn set_position(&mut self, value: &str, idx: usize) {
        let Ok(parsed) = parse_helper::<isize>(value) else {
            return;
        };

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

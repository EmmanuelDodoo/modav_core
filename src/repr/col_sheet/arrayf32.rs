use super::{parse_helper, Column, DataType, Iter, IterMut, Sealed};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ArrayF32 {
    header: Option<String>,
    cells: Vec<Option<f32>>,
}

impl ArrayF32 {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_iterator(values: impl Iterator<Item = f32>) -> Self {
        Self {
            cells: values.map(Some).collect(),
            ..Default::default()
        }
    }

    pub fn from_iterator_option(values: impl Iterator<Item = Option<f32>>) -> Self {
        Self {
            cells: values.collect(),
            ..Default::default()
        }
    }

    pub fn set_header(&mut self, header: String) -> &mut Self {
        self.header = Some(header);
        self
    }

    pub fn get(&self, idx: usize) -> Option<f32> {
        self.cells.get(idx)?.as_ref().copied()
    }

    pub fn get_mut(&mut self, idx: usize) -> Option<&mut f32> {
        self.cells.get_mut(idx)?.as_mut()
    }

    pub fn iter(&self) -> Iter<'_, Option<f32>> {
        self.cells.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, Option<f32>> {
        self.cells.iter_mut()
    }

    pub fn parse_str(values: &Vec<String>) -> Option<Self> {
        let mut cells = Vec::default();

        for value in values {
            let value = parse_helper::<f32>(value).ok()?;
            cells.push(value)
        }

        Some(Self {
            header: None,
            cells,
        })
    }
}

impl Sealed for ArrayF32 {
    fn push(&mut self, value: &str) {
        let value = match parse_helper::<f32>(value) {
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

        let Ok(parsed) = parse_helper::<f32>(value) else {
            return;
        };

        self.cells.insert(idx, parsed);
    }
}

impl Column for ArrayF32 {
    fn label(&self) -> Option<&String> {
        self.header.as_ref()
    }

    fn kind(&self) -> DataType {
        DataType::F32
    }

    fn len(&self) -> usize {
        self.cells.len()
    }

    fn set_header(&mut self, header: String) {
        self.header = Some(header);
    }

    fn set_position<'a>(&mut self, value: &'a str, idx: usize) {
        let Ok(parsed) = parse_helper::<f32>(value) else {
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

        self.cells.swap(x, y);
    }
}

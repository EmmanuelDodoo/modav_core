use super::{parse_helper, utils::*, Iter, IterMut};

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

    fn apply_index_swap(&mut self, indices: &[usize]) {
        for (pos, elem) in indices.iter().enumerate() {
            self.cells.swap(pos, *elem);
        }
    }

    fn remove_all(&mut self) {
        self.cells.clear()
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

    fn set_position(&mut self, value: &str, idx: usize) -> bool {
        let Ok(parsed) = parse_helper::<String>(value) else {
            return false;
        };

        let Some(prev) = self.cells.get_mut(idx) else {
            // This is ok because the Column sheet would have caught the out-of-bounds
            // earlier
            return true;
        };

        *prev = parsed;

        true
    }

    fn swap(&mut self, x: usize, y: usize) {
        if x >= self.len() || y >= self.len() {
            return;
        }

        self.cells.swap(x, y)
    }

    fn data_ref(&self, idx: usize) -> Option<DataRef<'_>> {
        match self.cells.get(idx)? {
            Some(value) => Some(DataRef::Text(value)),
            None => Some(DataRef::None),
        }
    }

    fn clear(&mut self, idx: usize) {
        if let Some(cell) = self.cells.get_mut(idx) {
            cell.take();
        }
    }

    fn clear_all(&mut self) {
        let len = self.cells.len();

        self.cells = vec![None; len];
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

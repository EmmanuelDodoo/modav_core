use super::{arrays::*, parse_helper, parse_unchecked, utils::*, Iter, IterMut};

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

    pub fn parse_str(values: &Vec<String>, null: &str) -> Option<Self> {
        let mut cells = Vec::default();

        for value in values {
            let parsed = parse_helper::<usize>(value, null).ok()?;
            cells.push(parsed)
        }

        Some(Self {
            header: None,
            cells,
        })
    }
}

impl Sealed for ArrayUSize {
    fn push(&mut self, value: &str, null: &str) {
        let value = parse_unchecked::<usize>(value, null);
        self.cells.push(value)
    }

    fn remove(&mut self, idx: usize) {
        if idx >= self.len() {
            return;
        }
        self.cells.remove(idx);
    }

    fn insert(&mut self, value: &str, idx: usize, null: &str) {
        if idx > self.len() {
            return;
        }

        let parsed = parse_unchecked::<usize>(value, null);

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

    fn set_position(&mut self, value: &str, idx: usize, null: &str) -> bool {
        let Ok(parsed) = parse_helper::<usize>(value, null) else {
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

    fn data_ref(&self, idx: usize) -> Option<CellRef<'_>> {
        match self.cells.get(idx)? {
            Some(value) => Some(CellRef::USize(*value)),
            None => Some(CellRef::None),
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

    fn convert_col(&self, to: DataType) -> Box<dyn Column> {
        let iter = self.iter().copied();

        match to {
            DataType::USize => Box::new(self.clone()),
            DataType::U32 => {
                let mut array = ArrayU32::from_iterator_option(
                    iter.map(|value| value.map(|value| value as u32)),
                );

                if let Some(header) = self.header.as_ref() {
                    array.set_header(header.clone());
                }

                Box::new(array)
            }
            DataType::I32 => {
                let mut array = ArrayI32::from_iterator_option(
                    iter.map(|value| value.map(|value| value as i32)),
                );

                if let Some(header) = self.header.as_ref() {
                    array.set_header(header.clone());
                }

                Box::new(array)
            }
            DataType::ISize => {
                let mut array = ArrayISize::from_iterator_option(
                    iter.map(|value| value.map(|value| value as isize)),
                );

                if let Some(header) = self.header.as_ref() {
                    array.set_header(header.clone());
                }

                Box::new(array)
            }
            DataType::F32 => {
                let mut array = ArrayF32::from_iterator_option(
                    iter.map(|value| value.map(|value| value as f32)),
                );

                if let Some(header) = self.header.as_ref() {
                    array.set_header(header.clone());
                }

                Box::new(array)
            }
            DataType::F64 => {
                let mut array = ArrayF64::from_iterator_option(
                    iter.map(|value| value.map(|value| value as f64)),
                );

                if let Some(header) = self.header.as_ref() {
                    array.set_header(header.clone());
                }

                Box::new(array)
            }
            DataType::Bool => {
                let mut array = ArrayBool::from_iterator_option(
                    iter.map(|value| value.map(|value| value != 0)),
                );

                if let Some(header) = self.header.as_ref() {
                    array.set_header(header.clone());
                }

                Box::new(array)
            }
            DataType::Text => {
                let mut array = ArrayText::from_iterator_option(
                    iter.map(|value| value.map(|value| value.to_string())),
                );

                if let Some(header) = self.header.as_ref() {
                    array.set_header(header.clone());
                }

                Box::new(array)
            }
        }
    }
}

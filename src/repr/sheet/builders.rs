use std::path::Path;

use super::{
    error::*,
    utils::{HeaderLabelStrategy, HeaderTypesStrategy},
    Sheet,
};

#[derive(Debug, Clone, PartialEq)]
pub struct SheetBuilder<P: AsRef<Path>> {
    pub(crate) path: P,
    pub(crate) primary: usize,
    pub(crate) trim: bool,
    pub(crate) label_strategy: HeaderLabelStrategy,
    pub(crate) flexible: bool,
    pub(crate) type_strategy: HeaderTypesStrategy,
    pub(crate) delimiter: u8,
}

impl<P: AsRef<Path>> SheetBuilder<P> {
    pub fn new(path: P) -> Self {
        Self {
            path,
            primary: 0,
            trim: false,
            label_strategy: HeaderLabelStrategy::NoLabels,
            flexible: false,
            type_strategy: HeaderTypesStrategy::None,
            delimiter: b',',
        }
    }

    pub fn primary(self, primary: usize) -> Self {
        Self { primary, ..self }
    }

    pub fn trim(self, trim: bool) -> Self {
        Self { trim, ..self }
    }

    pub fn flexible(self, flexible: bool) -> Self {
        Self { flexible, ..self }
    }

    pub fn types(self, strategy: HeaderTypesStrategy) -> Self {
        Self {
            type_strategy: strategy,
            ..self
        }
    }

    pub fn labels(self, strategy: HeaderLabelStrategy) -> Self {
        Self {
            label_strategy: strategy,
            ..self
        }
    }

    pub fn delimiter(mut self, delimiter: u8) -> Self {
        self.delimiter = delimiter;
        self
    }

    pub fn build(self) -> Result<Sheet> {
        Sheet::from_builder(self)
    }
}

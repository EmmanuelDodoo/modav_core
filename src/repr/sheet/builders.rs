use std::path::PathBuf;

use super::{
    error::*,
    utils::{HeaderLabelStrategy, HeaderTypesStrategy},
    Sheet,
};

#[derive(Debug, Clone, PartialEq)]
pub struct SheetBuilder {
    path: PathBuf,
    primary: usize,
    trim: bool,
    label_strategy: HeaderLabelStrategy,
    flexible: bool,
    type_strategy: HeaderTypesStrategy,
    delimiter: u8,
}

impl SheetBuilder {
    pub fn new(path: PathBuf) -> Self {
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
        Sheet::new(
            self.path,
            self.primary,
            self.label_strategy,
            self.type_strategy,
            self.trim,
            self.flexible,
            self.delimiter,
        )
    }
}

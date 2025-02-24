use std::{fmt, path::Path};

use super::utils::TypesStrategy;

const NULL: &str = "<null>";

/// Determines how headers read
#[derive(Debug, Clone, PartialEq, Default)]
pub enum HeaderStrategy {
    #[default]
    /// No labels for all columns
    NoLabels,
    /// First csv row taken as labels
    ReadLabels,
    /// Labels are provided
    Provided(Vec<String>),
}

impl fmt::Display for HeaderStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Provided(_) => "Header Labels Provided",
                Self::ReadLabels => "Read Header Labels",
                Self::NoLabels => "No Header Labels",
            }
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Config<P: AsRef<Path>> {
    pub(super) path: P,
    pub(super) primary: usize,
    pub(super) trim: bool,
    pub(super) label_strategy: HeaderStrategy,
    pub(super) flexible: bool,
    pub(super) type_strategy: TypesStrategy,
    pub(super) delimiter: u8,
    pub(super) null_string: String,
}

impl<P: AsRef<Path>> Config<P> {
    /// Returns a new default [`Config`] with the provided path.
    pub fn new(path: P) -> Self {
        Self {
            path,
            primary: 0,
            trim: false,
            label_strategy: HeaderStrategy::NoLabels,
            flexible: false,
            type_strategy: TypesStrategy::None,
            delimiter: b',',
            null_string: NULL.to_string(),
        }
    }

    /// Sets the primary column.
    pub fn primary(self, primary: usize) -> Self {
        Self { primary, ..self }
    }

    /// Whether fields are trimmed of leading and trailing whitespaces or not.
    pub fn trim(self, trim: bool) -> Self {
        Self { trim, ..self }
    }

    /// Whether the number of fields in records are allowed to change or not.
    pub fn flexible(self, flexible: bool) -> Self {
        Self { flexible, ..self }
    }

    /// How the type of each column is determined.
    pub fn types(self, strategy: TypesStrategy) -> Self {
        Self {
            type_strategy: strategy,
            ..self
        }
    }

    /// How headers are determined.
    pub fn labels(self, strategy: HeaderStrategy) -> Self {
        Self {
            label_strategy: strategy,
            ..self
        }
    }

    /// The field delimiter to use when parsing CSV.
    pub fn delimiter(mut self, delimiter: u8) -> Self {
        self.delimiter = delimiter;
        self
    }

    /// The string to be considered as a null field.
    pub fn null_string(mut self, null_string: impl Into<String>) -> Self {
        self.null_string = null_string.into();
        self
    }
}

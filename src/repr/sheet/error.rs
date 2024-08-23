use crate::models::line::utils::LineGraphError;
use std::{error, fmt};

#[derive(Debug)]
pub enum Error {
    /// Invalid primary key
    InvalidPrimaryKey(String),
    /// Error from csv reader
    CSVReaderError(csv::Error),
    /// Column type and value mismatch
    InvalidColumnType(String),
    /// Out of bounds column or uneven column number
    InvalidColumnLength(String),
    /// Non-uniform column sorting
    InvalidColumnSort(String),
    LineGraphConversionError(String),
    LineGraphError(LineGraphError),
    TransposeError(String),
}

impl From<csv::Error> for Error {
    fn from(value: csv::Error) -> Self {
        Error::CSVReaderError(value)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::CSVReaderError(e) => e.fmt(f),
            Error::InvalidColumnLength(s) => {
                write!(f, "Invalid Column Length: {}", s)
            }
            Error::InvalidPrimaryKey(s) => {
                write!(f, "Primary Key is invalid. {}", s)
            }
            Error::InvalidColumnType(s) => write!(f, "Invalid Column type: {}", s),
            Error::InvalidColumnSort(s) => write!(f, "Invalid Column Sort: {}", s),
            Error::LineGraphConversionError(s) => {
                write!(f, "Line Graph Conversion Error: {}", s)
            }
            Error::LineGraphError(lg) => lg.fmt(f),
            Error::TransposeError(s) => write!(f, "Transposing Error: {}", s),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Error::CSVReaderError(e) => Some(e),
            Error::InvalidColumnLength(_) => None,
            Error::InvalidPrimaryKey(_) => None,
            Error::InvalidColumnType(_) => None,
            Error::InvalidColumnSort(_) => None,
            Error::LineGraphConversionError(_) => None,
            Error::LineGraphError(_) => None,
            Error::TransposeError(_) => None,
        }
    }
}

/// A short hand alias for [`Sheet`] error results
pub type Result<T> = core::result::Result<T, Error>;

use crate::models::{bar::BarChartError, line::LineGraphError, stacked_bar::StackedBarChartError};
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
    /// Error when converting the sheet to another type
    ConversionError(String),
    /// Error from creating a new linegraph from sheet
    LineGraphError(LineGraphError),
    /// Error during a transpose
    TransposeError(String),
    /// Error from creating a new barchart from sheet
    BarChartError(BarChartError),
    /// Error from creating a new stacked barchart from sheet
    StackedBarChart(StackedBarChartError),
}

impl From<csv::Error> for Error {
    fn from(value: csv::Error) -> Self {
        Error::CSVReaderError(value)
    }
}

impl From<LineGraphError> for Error {
    fn from(value: LineGraphError) -> Self {
        Self::LineGraphError(value)
    }
}

impl From<BarChartError> for Error {
    fn from(value: BarChartError) -> Self {
        Self::BarChartError(value)
    }
}

impl From<StackedBarChartError> for Error {
    fn from(value: StackedBarChartError) -> Self {
        Self::StackedBarChart(value)
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
            Error::ConversionError(s) => {
                write!(f, "Conversion Error: {}", s)
            }
            Error::LineGraphError(lg) => lg.fmt(f),
            Error::TransposeError(s) => write!(f, "Transposing Error: {}", s),
            Error::BarChartError(bar) => bar.fmt(f),
            Error::StackedBarChart(bar) => bar.fmt(f),
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
            Error::ConversionError(_) => None,
            Error::LineGraphError(lg) => Some(lg),
            Error::TransposeError(_) => None,
            Error::BarChartError(bar) => Some(bar),
            Error::StackedBarChart(bar) => Some(bar),
        }
    }
}

/// A short hand alias for `Sheet` error results
pub type Result<T> = core::result::Result<T, Error>;

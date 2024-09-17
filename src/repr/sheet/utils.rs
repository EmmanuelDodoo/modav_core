use std::{
    cmp::{self, Ordering},
    default, fmt, hash,
};

#[derive(Debug, Clone, Default, PartialEq)]
pub enum Data {
    /// A text
    Text(String),
    /// A 32 bit signed integer
    Integer(i32),
    /// A 32 bit float
    Float(f32),
    /// A signed integer
    Number(isize),
    /// A boolean value
    Boolean(bool),
    /// An empty cell
    #[default]
    None,
}

#[allow(dead_code)]
impl Data {
    pub(crate) fn is_negative(&self) -> bool {
        match self {
            Data::Number(num) => *num < 0,
            Data::Float(float) => *float < 0.0,
            Data::Integer(int) => *int < 0,
            _ => false,
        }
    }
}

#[allow(clippy::non_canonical_partial_ord_impl)]
impl cmp::PartialOrd for Data {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        match (self, other) {
            (Data::Text(x), Data::Text(y)) => x.partial_cmp(y),
            (Data::Text(_), _) => Some(Ordering::Greater),
            (Data::Number(x), Data::Number(y)) => x.partial_cmp(y),
            (Data::Number(_), Data::Text(_)) => Some(Ordering::Less),
            (Data::Number(_), _) => Some(Ordering::Greater),
            (Data::Float(x), Data::Float(y)) => x.partial_cmp(y),
            (Data::Float(_), Data::Text(_)) => Some(Ordering::Less),
            (Data::Float(_), Data::Number(_)) => Some(Ordering::Less),
            (Data::Float(_), _) => Some(Ordering::Greater),
            (Data::Integer(x), Data::Integer(y)) => x.partial_cmp(y),
            (Data::Integer(_), Data::Text(_)) => Some(Ordering::Less),
            (Data::Integer(_), Data::Number(_)) => Some(Ordering::Less),
            (Data::Integer(_), Data::Float(_)) => Some(Ordering::Less),
            (Data::Integer(_), _) => Some(Ordering::Greater),
            (Data::Boolean(x), Data::Boolean(y)) => x.partial_cmp(y),
            (Data::Boolean(_), Data::Text(_)) => Some(Ordering::Less),
            (Data::Boolean(_), Data::Number(_)) => Some(Ordering::Less),
            (Data::Boolean(_), Data::Float(_)) => Some(Ordering::Less),
            (Data::Boolean(_), Data::Integer(_)) => Some(Ordering::Less),
            (Data::Boolean(_), _) => Some(Ordering::Greater),
            (Data::None, Data::None) => Some(Ordering::Equal),
            (Data::None, _) => Some(Ordering::Less),
        }
    }
}

impl Eq for Data {}

impl cmp::Ord for Data {
    fn cmp(&self, other: &Self) -> Ordering {
        if let Some(ord) = self.partial_cmp(other) {
            ord
        } else {
            // Special case for NaN. Should only happend when both are f32
            match self {
                Data::Float(f) => {
                    if f.is_nan() {
                        Ordering::Less
                    } else {
                        Ordering::Greater
                    }
                }

                _ => panic!("Partial_cmp for Data returned None. Only floats should do so"),
            }
        }
    }
}

impl hash::Hash for Data {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        match self {
            Data::Text(t) => t.hash(state),
            Data::Integer(i) => i.hash(state),
            Data::Number(n) => n.hash(state),
            Data::Boolean(b) => b.hash(state),
            Data::Float(f) => format!("{}", f).hash(state),
            Data::None => "<None>".hash(state),
        }
    }
}

impl fmt::Display for Data {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Text(t) => write!(f, "{}", t),
            Self::Integer(i) => write!(f, "{}", i),
            Self::Float(fl) => write!(f, "{}", fl),
            Self::Boolean(b) => write!(f, "{}", b),
            Self::Number(n) => write!(f, "{}", n),
            Self::None => write!(f, "<None>"),
        }
    }
}

impl From<bool> for Data {
    fn from(value: bool) -> Self {
        Data::Boolean(value)
    }
}

impl From<String> for Data {
    fn from(value: String) -> Self {
        if value.is_empty() {
            return Data::None;
        }

        if let Ok(parsed_i32) = value.parse::<i32>() {
            return Data::Integer(parsed_i32);
        };

        if let Ok(parsed_bool) = value.parse::<bool>() {
            return Data::Boolean(parsed_bool);
        };

        if let Ok(parsed_float) = value.parse::<f32>() {
            return Data::Float(parsed_float);
        }

        if let Ok(parsed_num) = value.parse::<isize>() {
            return Data::Number(parsed_num);
        };

        if value == Data::None.to_string() {
            return Data::None;
        }
        Data::Text(value)
    }
}

impl From<&str> for Data {
    fn from(value: &str) -> Self {
        value.to_string().into()
    }
}

impl From<i32> for Data {
    fn from(value: i32) -> Self {
        Data::Integer(value)
    }
}

impl From<f32> for Data {
    fn from(value: f32) -> Self {
        Data::Float(value)
    }
}

impl From<isize> for Data {
    fn from(value: isize) -> Self {
        Data::Number(value)
    }
}

impl From<Data> for String {
    fn from(value: Data) -> Self {
        value.to_string()
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum ColumnType {
    /// A text column
    Text,
    /// A 32 bit signed integer column
    Integer,
    /// A signed integer column
    Number,
    /// A 32 bit floating point number column
    Float,
    /// A boolean column
    Boolean,
    #[default]
    /// A non-uniform type column
    None,
}

impl Eq for ColumnType {}

impl ColumnType {
    /// Returns true if data is equivalent to this column type.
    /// For flexibility reasons, ColumnType::None always returns true
    pub fn crosscheck_type(&self, data: &Data) -> bool {
        if let Data::None = data {
            return true;
        };
        let conv: ColumnType = data.clone().into();
        match self {
            ColumnType::None => true,
            _ => &conv == self,
        }
    }
}

impl From<Data> for ColumnType {
    fn from(value: Data) -> Self {
        match value {
            Data::Text(_) => Self::Text,
            Data::Float(_) => Self::Float,
            Data::Number(_) => Self::Number,
            Data::Integer(_) => Self::Integer,
            Data::Boolean(_) => Self::Boolean,
            Data::None => Self::None,
        }
    }
}

impl fmt::Display for ColumnType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::None => "No Column Type",
                Self::Boolean => "Boolean Column Type",
                Self::Text => "Text Column Type",
                Self::Float => "Float Column Type",
                Self::Integer => "Integer Column Type",
                Self::Number => "Number Column Type",
            }
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ColumnHeader {
    /// The label for the column
    pub label: String,
    /// The type of column
    pub kind: ColumnType,
}

impl ColumnHeader {
    pub fn new(label: String, kind: ColumnType) -> Self {
        Self { label, kind }
    }

    pub fn set_label(&mut self, label: String) {
        self.label = label;
    }

    /// Returns true if data is equivalent to this column type.
    /// For flexibility reasons, ColumnType::None always returns true
    pub fn crosscheck_type(&self, data: &Data) -> bool {
        self.kind.crosscheck_type(data)
    }
}

impl default::Default for ColumnHeader {
    fn default() -> Self {
        Self {
            label: "".into(),
            kind: ColumnType::None,
        }
    }
}

/// Determines how headers for a `Sheet` are created
#[derive(Debug, Clone, PartialEq, Default)]
pub enum HeaderLabelStrategy {
    #[default]
    /// No labels for all columns
    NoLabels,
    /// First csv row taken as labels
    ReadLabels,
    /// Labels are provided
    Provided(Vec<String>),
}

impl fmt::Display for HeaderLabelStrategy {
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

#[derive(Debug, Clone, PartialEq, Default)]
pub enum HeaderTypesStrategy {
    /// The types are infered from the csv
    Infer,
    /// The types are provided as a vector
    Provided(Vec<ColumnType>),
    /// All columns have a None type
    #[default]
    None,
}

impl fmt::Display for HeaderTypesStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Infer => "Infer types",
                Self::Provided(_) => "Provide types",
                Self::None => "No types",
            },
        )
    }
}

/// Determines how the labels of the line graph created from a sheet are handled
#[derive(Debug, Clone, PartialEq, Default)]
pub enum LineLabelStrategy {
    /// Label is derived from a the cells of a column. The values are not used
    /// within the line graph
    FromCell(usize),
    /// Labels for each line are provided. Excess labels are ignored. Lines with
    /// no labels receive a [`LineLabelStrategy::None`]
    Provided(Vec<String>),
    /// No labels
    #[default]
    None,
}

impl fmt::Display for LineLabelStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::None => "No line labels",
                Self::FromCell(_) => "Label using a cell",
                Self::Provided(_) => "Label provided",
            }
        )
    }
}

/// Determines how the axis labels are generated for a bar chart
#[derive(Debug, Default, Clone, PartialEq)]
pub enum BarChartAxisLabelStrategy {
    /// Any corresponding hearders present will serve as the labels for the axis
    Headers,
    /// The axis labels are provided.
    Provided { x: String, y: String },
    /// No labels are generated
    #[default]
    None,
}

impl fmt::Display for BarChartAxisLabelStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::None => "No labels",
                Self::Provided { .. } => "Labels provided",
                Self::Headers => "Use corresponding headers",
            }
        )
    }
}

/// Determines how the labels for individual bars are generated
#[derive(Debug, Default, Clone, PartialEq)]
pub enum BarChartBarLabels {
    /// No labels generated
    #[default]
    None,
    /// Values from corresponding column used as the labels
    FromColumn(usize),
    /// Labels are provided Excess labels are ignored. Lines with
    /// no labels receive a [`BarChartBarLabels::None`]
    Provided(Vec<String>),
}

impl fmt::Display for BarChartBarLabels {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::None => "No labels",
                Self::Provided(_) => "Labels provided",
                Self::FromColumn(_) => "Labels from a column",
            }
        )
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub enum StackedBarChartAxisLabelStrategy {
    /// The y axis label is provided, while the header for the x column is used
    /// as the x axis label
    Header(String),
    /// Both axis labels are provided
    Provided { x: String, y: String },
    /// No labels for both axis
    #[default]
    None,
}

impl fmt::Display for StackedBarChartAxisLabelStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::None => "No labels",
                Self::Provided { .. } => "Both Axis labels provided",
                Self::Header(_) => "Y axis provided",
            }
        )
    }
}

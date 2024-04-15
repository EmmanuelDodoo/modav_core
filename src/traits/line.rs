use std::fmt::Debug;

use crate::models::line::LineGraph;

pub trait ToLineGraph {
    type X: Clone + Debug;
    type Y: Clone + Debug;
    type ErrorType;

    fn to_line_graph(
        self: &Self,
        x_label: Option<String>,
        y_label: Option<String>,
    ) -> Result<LineGraph<Self::X, Self::Y>, Self::ErrorType>;
}

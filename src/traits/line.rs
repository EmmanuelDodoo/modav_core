use crate::models::line::LineGraph;

pub trait ToLineGraph {
    type X;
    type Y;
    type ErrorType;

    fn to_line_graph(
        self: &Self,
        x_label: Option<String>,
        y_label: Option<String>,
    ) -> Result<LineGraph<Self::X, Self::Y>, Self::ErrorType>;
}

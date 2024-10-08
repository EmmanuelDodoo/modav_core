use crate::repr::Data;
use std::fmt::Debug;
pub use utils::*;

use super::{Point, Scale};

#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    pub points: Vec<Point<Data, Data>>,
    pub label: Option<String>,
}

impl Line {
    pub fn new(points: impl IntoIterator<Item = (impl Into<Data>, impl Into<Data>)>) -> Self {
        let points = points
            .into_iter()
            .map(|(x, y)| Point::new(x.into(), y.into()));
        Self {
            points: points.collect(),
            label: None,
        }
    }

    pub fn from_points(points: impl IntoIterator<Item = Point<Data, Data>>) -> Self {
        Self {
            points: points.into_iter().collect(),
            label: None,
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LineGraph {
    pub lines: Vec<Line>,
    pub x_label: String,
    pub y_label: String,
    pub x_scale: Scale,
    pub y_scale: Scale,
}

#[allow(dead_code)]
impl LineGraph {
    pub fn new(
        lines: Vec<Line>,
        x_label: Option<String>,
        y_label: Option<String>,
        x_scale: Scale,
        y_scale: Scale,
    ) -> Result<Self, LineGraphError> {
        let x_label = x_label.unwrap_or_default();

        let y_label = y_label.unwrap_or_default();

        LineGraph::assert_x_scale(&x_scale, &lines)?;

        LineGraph::assert_y_scale(&y_scale, &lines)?;

        Ok(Self {
            lines,
            x_label,
            y_label,
            x_scale,
            y_scale,
        })
    }

    fn assert_x_scale(scale: &Scale, lines: &[Line]) -> Result<(), LineGraphError> {
        for x in lines
            .iter()
            .flat_map(|line| line.points.iter().map(|point| &point.x))
        {
            if !scale.contains(x) {
                return Err(LineGraphError::OutOfRange("X".into(), x.to_string()));
            }
        }

        Ok(())
    }

    fn assert_y_scale(scale: &Scale, lines: &[Line]) -> Result<(), LineGraphError> {
        for y in lines
            .iter()
            .flat_map(|line| line.points.iter().map(|point| &point.y))
        {
            if !scale.contains(y) {
                return Err(LineGraphError::OutOfRange("Y".into(), y.to_string()));
            }
        }

        Ok(())
    }
}

pub mod utils {
    use std::fmt;

    #[derive(Debug, Clone, PartialEq)]
    pub enum LineGraphError {
        OutOfRange(String, String),
        ScaleLengthError(String),
    }

    impl fmt::Display for LineGraphError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                LineGraphError::ScaleLengthError(sc) => {
                    write!(f, "The {} axis has smaller scale than expected", sc)
                }
                LineGraphError::OutOfRange(sc, val) => {
                    write!(
                        f,
                        "The point with value {} on the {} axis is out of range",
                        val, sc
                    )
                }
            }
        }
    }

    impl std::error::Error for LineGraphError {}
}

#[cfg(test)]
mod line_tests {
    use super::super::common::ScaleKind;
    use super::*;

    fn create_point<X, Y>(x: X, y: Y) -> Point<X, Y> {
        Point::new(x, y)
    }

    fn create_line_from_points(xs: Vec<&str>, label: impl Into<String>) -> Line {
        let points: Vec<Point> = xs
            .into_iter()
            .enumerate()
            .map(|(i, x)| create_point(Data::Number(i as isize), Data::Text(x.to_owned())))
            .collect();

        Line::from_points(points).label(label)
    }

    fn create_line_from_new(xs: Vec<(usize, &str)>, label: impl Into<String>) -> Line {
        let points = xs
            .iter()
            .map(|(idx, text)| (Data::Number(*idx as isize), Data::Text(text.to_string())));
        Line::new(points).label(label)
    }

    fn create_graph() -> LineGraph {
        let p1 = vec!["one", "two", "three", "four", "five"];
        let p2: Vec<(usize, &str)> = vec![
            (10, "one"),
            (20, "two"),
            (30, "three"),
            (40, "four"),
            (50, "five"),
        ];

        let pnt1 = create_line_from_new(p2, "Deutsch");
        let pnt2 = create_line_from_points(p1, "English");

        let x_scale = {
            let rng = 0..60;
            let rng = rng.into_iter().map(|num| Data::Number(num as isize));

            Scale::new(rng, ScaleKind::Number)
        };

        let y_scale = {
            let values = vec!["one", "two", "three", "four", "five"];

            Scale::new(values, ScaleKind::Categorical)
        };

        match LineGraph::new(
            vec![pnt1, pnt2],
            Some("Number".into()),
            Some("Language".into()),
            x_scale,
            y_scale,
        ) {
            Ok(lg) => return lg,
            Err(e) => panic!("{}", e),
        }
    }

    fn faulty_graph1() -> Result<LineGraph, LineGraphError> {
        let p1: Vec<(i32, i32)> = vec![(0, 0), (1, 1), (20, 2), (3, 35)];
        let p2: Vec<(i32, i32)> = vec![(10, 10), (4, 8), (-3, 3)];

        let x_scale: Scale = {
            let rng = -5..11;

            Scale::new(rng, ScaleKind::Integer)
        };
        let y_scale: Scale = {
            let rng = 2..10;

            Scale::new(rng, ScaleKind::Integer)
        };

        let l1 = Line::new(p1);
        let l2 = Line::new(p2);

        LineGraph::new(vec![l1, l2], None, None, x_scale, y_scale)
    }

    #[test]
    fn test_line_line() {
        let pts = vec!["one", "two", "three"];
        let line = create_line_from_points(pts, "Line 1");

        assert_eq!(line.label, Some(String::from("Line 1")));

        let temp: Vec<String> = line.points.iter().fold(vec![], |acc, curr| {
            let mut acc = acc.clone();
            acc.push(curr.y.to_string());
            acc
        });
        assert_eq!(vec!["one", "two", "three"], temp)
    }

    #[test]
    fn test_line_graph() {
        let graph = create_graph();

        assert_eq!(graph.y_label, String::from("Language"));
        assert_eq!(graph.x_label, String::from("Number"));

        graph
            .lines
            .iter()
            .for_each(|ln| assert_eq!(ln.points.len(), 5))
    }

    #[test]
    fn test_faulty_graph() {
        let expected = LineGraphError::OutOfRange(String::from("X"), String::from("20"));
        match faulty_graph1() {
            Ok(_) => panic!("Should not reach this test case"),
            Err(e) => assert_eq!(e, expected),
        }
    }
}

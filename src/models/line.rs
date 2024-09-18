use crate::repr::Data;
use std::{collections::HashSet, fmt::Debug, hash::Hash, ops::Range};
pub use utils::*;

use super::{Point, Scale};

#[derive(Debug, Clone, PartialEq)]
pub struct Line<X = Data, Y = Data> {
    pub points: Vec<Point<X, Y>>,
    pub label: Option<String>,
}

impl<X, Y> Line<X, Y> {
    pub fn new(points: Vec<(X, Y)>) -> Self {
        let points = points.into_iter().map(|(x, y)| Point::new(x, y));
        Self {
            points: points.collect(),
            label: None,
        }
    }

    pub fn from_points(points: impl IntoIterator<Item = Point<X, Y>>) -> Self {
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
pub struct LineGraph<X = Data, Y = Data>
where
    X: Clone + Debug,
    Y: Clone + Debug,
{
    pub lines: Vec<Line<X, Y>>,
    pub x_label: String,
    pub y_label: String,
    pub x_scale: Scale<X>,
    pub y_scale: Scale<Y>,
}

#[allow(dead_code)]
impl<X, Y> LineGraph<X, Y>
where
    X: Eq + Clone + Hash + PartialOrd + ToString + Debug,
    Y: Eq + Clone + Hash + PartialOrd + ToString + Debug,
{
    pub fn new(
        lines: Vec<Line<X, Y>>,
        x_label: Option<String>,
        y_label: Option<String>,
        x_scale: Scale<X>,
        y_scale: Scale<Y>,
    ) -> Result<Self, LineGraphError> {
        let x_label = x_label.unwrap_or_default();

        let y_label = y_label.unwrap_or_default();

        match &x_scale {
            // Scale::Range(rng) => Scale::Range(LineGraph::assert_range_scales_x(rng, &lines)?),
            Scale::List(lst) => LineGraph::assert_list_scales_x(lst, &lines)?,
        };

        match &y_scale {
            // Scale::Range(rng) => Scale::Range(LineGraph::assert_range_scales_y(rng, &lines)?),
            Scale::List(lst) => LineGraph::assert_list_scales_y(lst, &lines)?,
        };

        Ok(Self {
            lines,
            x_label,
            y_label,
            x_scale,
            y_scale,
        })
    }

    fn assert_range_scales_x(
        rng: Range<X>,
        lines: &[Line<X, Y>],
    ) -> Result<Range<X>, LineGraphError> {
        let rng = rng.start..rng.end;

        let mut invalid: Option<X> = None;

        let valid = lines.iter().fold(true, |acc, curr| {
            return acc
                && curr.points.iter().fold(true, |acc, curr| {
                    if !rng.contains(&curr.x) {
                        invalid = Some(curr.x.clone());
                    }
                    acc && rng.contains(&curr.x)
                });
        });

        if valid {
            Ok(rng)
        } else {
            Err(LineGraphError::OutOfRange(
                "X".into(),
                invalid.unwrap().to_string(),
            ))
        }
    }

    fn assert_range_scales_y(
        rng: Range<Y>,
        lines: &[Line<X, Y>],
    ) -> Result<Range<Y>, LineGraphError> {
        let rng = rng.start..rng.end;
        let mut invalid: Option<Y> = None;
        let valid = lines.iter().fold(true, |acc, curr| {
            return acc
                && curr.points.iter().fold(true, |acc, curr| {
                    if !rng.contains(&curr.y) {
                        invalid = Some(curr.y.clone());
                    }
                    acc && rng.contains(&curr.y)
                });
        });

        if valid {
            Ok(rng)
        } else {
            Err(LineGraphError::OutOfRange(
                "Y".into(),
                invalid.unwrap().to_string(),
            ))
        }
    }

    fn assert_list_scales_x(lst: &[X], lines: &[Line<X, Y>]) -> Result<(), LineGraphError> {
        // Duplicate check and removal
        let mut lst: Vec<X> = lst.to_vec();
        let set: HashSet<X> = lst.drain(..).collect();

        let mut invalid: Option<X> = None;

        // Check if all points are on scale.
        let valid = lines.iter().fold(true, |acc, cur| {
            return acc
                && cur.points.iter().fold(true, |acc, curr| {
                    if !set.contains(&curr.x) {
                        invalid = Some(curr.x.clone());
                    }
                    acc && set.contains(&curr.x)
                });
        });

        if valid {
            Ok(())
        } else {
            Err(LineGraphError::OutOfRange(
                "X".into(),
                invalid.unwrap().to_string(),
            ))
        }
    }

    fn assert_list_scales_y(lst: &[Y], lines: &[Line<X, Y>]) -> Result<(), LineGraphError> {
        // Duplicate check and removal
        let mut lst: Vec<Y> = lst.to_vec();
        let set: HashSet<Y> = lst.drain(..).collect();

        // Check if all points are on scale.
        let mut invalid: Option<Y> = None;
        let valid = lines.iter().fold(true, |acc, cur| {
            return acc
                && cur.points.iter().fold(true, |acc, curr| {
                    if !set.contains(&curr.y) {
                        invalid = Some(curr.y.clone())
                    }
                    acc && set.contains(&curr.y)
                });
        });

        if valid {
            Ok(())
        } else {
            Err(LineGraphError::OutOfRange(
                "Y".into(),
                invalid.unwrap().to_string(),
            ))
        }
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
    use super::*;

    fn create_point<X, Y>(x: X, y: Y) -> Point<X, Y> {
        Point::new(x, y)
    }

    fn create_line_from_points(xs: Vec<&str>, label: impl Into<String>) -> Line<usize, &str> {
        let points: Vec<Point<usize, &str>> = xs
            .into_iter()
            .enumerate()
            .map(|(i, x)| create_point(i, x))
            .collect();

        Line::from_points(points).label(label)
    }

    fn create_line_from_new(xs: Vec<(usize, &str)>, label: impl Into<String>) -> Line<usize, &str> {
        Line::new(xs).label(label)
    }

    fn create_graph<'a>() -> LineGraph<usize, &'a str> {
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

        // let x_scale: Scale<usize> = Scale::Range(0..60);
        let x_scale: Scale<usize> = {
            let rng = 0..60;

            Scale::List(rng.collect())
        };
        let y_scale: Scale<&str> = Scale::List(vec!["one", "two", "three", "four", "five"]);

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

    fn faulty_graph1() -> Result<LineGraph<isize, isize>, LineGraphError> {
        let p1: Vec<(isize, isize)> = vec![(0, 0), (1, 1), (20, 2), (3, 35)];
        let p2: Vec<(isize, isize)> = vec![(10, 10), (4, 8), (-3, 3)];

        let x_scale: Scale<isize> = {
            let rng = -5..11;

            Scale::List(rng.collect())
        };
        let y_scale: Scale<isize> = {
            let rng = 2..10;
            Scale::List(rng.collect())
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
        let temp: Vec<&str> = line.points.iter().fold(vec![], |acc, curr| {
            let mut acc = acc.clone();
            acc.push(curr.y);
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

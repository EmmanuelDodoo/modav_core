use std::{
    collections::HashSet,
    fmt::{self, Debug},
    hash::Hash,
};

use super::{Point, Scale};
use crate::repr::Data;

#[derive(Clone, Debug, PartialEq)]
pub struct Bar<X = Data, Y = Data> {
    pub label: Option<String>,
    pub point: Point<X, Y>,
}

impl<X, Y> Bar<X, Y> {
    pub fn new(label: impl Into<String>, point: impl Into<Point<X, Y>>) -> Self {
        Self {
            point: point.into(),
            label: Some(label.into()),
        }
    }

    pub fn from_point(point: impl Into<Point<X, Y>>) -> Self {
        Self {
            point: point.into(),
            label: None,
        }
    }

    pub fn label(mut self, label: String) -> Self {
        self.label = Some(label);
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BarChart<X = Data, Y = Data>
where
    X: Clone + Debug,
    Y: Clone + Debug,
{
    pub bars: Vec<Bar<X, Y>>,
    pub x_label: Option<String>,
    pub y_label: Option<String>,
    pub x_scale: Scale<X>,
    pub y_scale: Scale<Y>,
}

#[allow(dead_code)]
impl<X, Y> BarChart<X, Y>
where
    X: Eq + Clone + Hash + PartialOrd + ToString + Debug,
    Y: Eq + Clone + Hash + PartialOrd + ToString + Debug,
{
    pub fn new(
        bars: Vec<Bar<X, Y>>,
        x_scale: Scale<X>,
        y_scale: Scale<Y>,
    ) -> Result<Self, BarChartError> {
        match &x_scale {
            Scale::List(scale) => Self::assert_list_scales_x(scale, &bars)?,
        };

        match &y_scale {
            Scale::List(scale) => Self::assert_list_scales_y(scale, &bars)?,
        };

        Ok(Self {
            x_scale,
            y_scale,
            bars,
            x_label: None,
            y_label: None,
        })
    }

    fn assert_list_scales_y(lst: &[Y], bars: &[Bar<X, Y>]) -> Result<(), BarChartError> {
        // Duplicate check and removal
        let mut lst: Vec<Y> = lst.to_vec();
        let set: HashSet<Y> = lst.drain(..).collect();

        // Check if all points are on scale.
        let mut invalid: Option<Y> = None;
        let valid = bars.iter().fold(true, |acc, curr| {
            if !acc {
                return acc;
            }

            if !set.contains(&curr.point.y) {
                invalid = Some(curr.point.y.clone());
                false
            } else {
                true
            }
        });

        if valid {
            Ok(())
        } else {
            Err(BarChartError::OutOfRange(
                "Y".into(),
                invalid.unwrap().to_string(),
            ))
        }
    }

    fn assert_list_scales_x(lst: &[X], bars: &[Bar<X, Y>]) -> Result<(), BarChartError> {
        // Duplicate check and removal
        let mut lst: Vec<X> = lst.to_vec();
        let set: HashSet<X> = lst.drain(..).collect();

        let mut invalid: Option<X> = None;

        let valid = bars.iter().fold(true, |acc, curr| {
            if !acc {
                return acc;
            }

            if !set.contains(&curr.point.x) {
                invalid = Some(curr.point.x.clone());
                false
            } else {
                true
            }
        });

        if valid {
            Ok(())
        } else {
            Err(BarChartError::OutOfRange(
                "X".into(),
                invalid.unwrap().to_string(),
            ))
        }
    }

    pub fn x_label(mut self, label: impl Into<String>) -> Self {
        self.x_label = Some(label.into());
        self
    }

    pub fn y_label(mut self, label: impl Into<String>) -> Self {
        self.y_label = Some(label.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BarChartError {
    OutOfRange(String, String),
}

impl fmt::Display for BarChartError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BarChartError::OutOfRange(sc, val) => {
                write!(
                    f,
                    "The point with value {} on the {} axis is out of range",
                    val, sc
                )
            }
        }
    }
}

impl std::error::Error for BarChartError {}

#[cfg(test)]
mod barchart_tests {
    use super::*;

    fn create_barchart<'a>() -> BarChart<usize, &'a str> {
        let p1 = vec!["one", "two", "three", "four", "five"];
        let p2 = [1, 2, 3, 4, 5];

        let bars = p2
            .into_iter()
            .zip(p1.into_iter())
            .map(|point| Bar::from_point(point))
            .collect();

        let x_scale: Scale<usize> = {
            let rng = 0..60;

            Scale::List(rng.collect())
        };
        let y_scale: Scale<&str> = Scale::List(vec!["one", "two", "three", "four", "five"]);

        match BarChart::new(bars, x_scale, y_scale) {
            Ok(bar) => bar.x_label("Number").y_label("Language"),
            Err(e) => panic!("{}", e),
        }
    }

    fn out_of_range() -> Result<BarChart<isize, isize>, BarChartError> {
        let xs = [1, 5, 6, 11, 15];
        let ys = [4, 5, 6, 7, 8];

        let bars = xs
            .into_iter()
            .zip(ys.into_iter())
            .map(|point| Bar::from_point(point))
            .collect();

        let x_scale: Scale<isize> = {
            let rng = -5..11;

            Scale::List(rng.collect())
        };
        let y_scale: Scale<isize> = {
            let rng = 2..10;
            Scale::List(rng.collect())
        };

        BarChart::new(bars, x_scale, y_scale)
    }

    #[test]
    fn test_barchart() {
        let barchart = create_barchart();

        assert_eq!(barchart.y_label.unwrap(), String::from("Language"));
        assert_eq!(barchart.x_label.unwrap(), String::from("Number"));

        assert_eq!(barchart.bars.len(), 5)
    }

    #[test]
    fn test_faulty_barchart() {
        let expected = BarChartError::OutOfRange(String::from("X"), String::from("11"));
        match out_of_range() {
            Ok(_) => panic!("Should not reach this test case"),
            Err(e) => assert_eq!(e, expected),
        }
    }
}

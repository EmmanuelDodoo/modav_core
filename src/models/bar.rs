use std::fmt::{self, Debug};

use super::{Point, Scale};

#[derive(Clone, Debug, PartialEq)]
pub struct Bar {
    pub label: Option<String>,
    pub point: Point,
}

impl Bar {
    pub fn new(label: impl Into<String>, point: impl Into<Point>) -> Self {
        Self {
            point: point.into(),
            label: Some(label.into()),
        }
    }

    pub fn from_point(point: impl Into<Point>) -> Self {
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
pub struct BarChart {
    pub bars: Vec<Bar>,
    pub x_label: Option<String>,
    pub y_label: Option<String>,
    pub x_scale: Scale,
    pub y_scale: Scale,
}

#[allow(dead_code)]
impl BarChart {
    pub fn new(bars: Vec<Bar>, x_scale: Scale, y_scale: Scale) -> Result<Self, BarChartError> {
        Self::assert_x_scale(&x_scale, &bars)?;
        Self::assert_y_scale(&y_scale, &bars)?;

        Ok(Self {
            x_scale,
            y_scale,
            bars,
            x_label: None,
            y_label: None,
        })
    }

    fn assert_x_scale(scale: &Scale, bars: &[Bar]) -> Result<(), BarChartError> {
        for x in bars.iter().map(|bar| &bar.point.x) {
            if !scale.contains(x) {
                return Err(BarChartError::OutOfRange("X".into(), x.to_string()));
            }
        }

        Ok(())
    }

    fn assert_y_scale(scale: &Scale, bars: &[Bar]) -> Result<(), BarChartError> {
        for y in bars.iter().map(|bar| &bar.point.y) {
            if !scale.contains(y) {
                return Err(BarChartError::OutOfRange("Y".into(), y.to_string()));
            }
        }

        Ok(())
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
    use super::super::ScaleKind;
    use super::*;
    use crate::repr::Data;

    fn create_barchart() -> BarChart {
        let p1 = vec!["one", "two", "three", "four", "five"];
        let p2 = [1, 2, 3, 4, 5];

        let bars = p2
            .into_iter()
            .zip(p1.into_iter())
            .map(|point| Bar::from_point((Data::Integer(point.0), Data::Text(point.1.to_string()))))
            .collect();

        let x_scale = {
            let rng = 0..60;

            Scale::new(rng, ScaleKind::Integer)
        };
        let y_scale = {
            let values = vec!["one", "two", "three", "four", "five"];

            Scale::new(values, ScaleKind::Text)
        };

        match BarChart::new(bars, x_scale, y_scale) {
            Ok(bar) => bar.x_label("Number").y_label("Language"),
            Err(e) => panic!("{}", e),
        }
    }

    fn out_of_range() -> Result<BarChart, BarChartError> {
        let xs = [1, 5, 6, 11, 15];
        let ys = [4, 5, 6, 7, 8];

        let bars = xs
            .into_iter()
            .zip(ys.into_iter())
            .map(|point| Bar::from_point((Data::Integer(point.0), Data::Integer(point.1))))
            .collect();

        let x_scale = {
            let rng = -5..11;

            Scale::new(rng, ScaleKind::Integer)
        };
        let y_scale = {
            let rng = 2..10;

            Scale::new(rng, ScaleKind::Integer)
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

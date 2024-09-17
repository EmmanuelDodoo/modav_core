use std::{
    collections::{HashMap, HashSet},
    fmt::{self, Debug},
    hash::Hash,
};

use super::{Point, Scale};
use crate::repr::Data;

#[derive(Clone, Debug, PartialEq)]
pub struct StackedBar<X = Data, Y = Data> {
    /// The (x, y) points for the bar
    pub point: Point<X, Y>,
    /// The percentage makeup  of the bar. For all
    /// k, v in `fractions` v1 + v2 + v3 + .. = 1.0
    pub fractions: HashMap<String, f64>,
    /// Is true of all points within the bar are negative
    pub is_negative: bool,
}

impl<X, Y> StackedBar<X, Y> {
    pub fn new(point: Point<X, Y>, fractions: HashMap<String, f64>, is_negative: bool) -> Self {
        Self {
            point,
            fractions,
            is_negative,
        }
    }

    pub fn from_point(point: impl Into<Point<X, Y>>, is_negative: bool) -> Self {
        Self {
            point: point.into(),
            fractions: HashMap::default(),
            is_negative,
        }
    }

    pub fn set_fractions(mut self, fractions: HashMap<String, f64>) -> Self {
        self.fractions = fractions;
        self
    }

    pub fn get_fractions(&self) -> &HashMap<String, f64> {
        &self.fractions
    }

    pub fn get_point(&self) -> &Point<X, Y> {
        &self.point
    }
}

impl StackedBar<Data, Data> {
    /// Returns true if the point is empty. For a Stacked bar chart, an empty point
    /// is defined as one which has a y data value of 0 or 0.0
    pub(crate) fn is_empty(&self) -> bool {
        match &self.point.y {
            Data::Integer(i) => *i == 0,
            Data::Number(n) => *n == 0,
            Data::Float(f) => *f == 0.0,
            _ => false,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct StackedBarChart<X = Data, Y = Data>
where
    X: Clone + Debug,
    Y: Clone + Debug,
{
    pub bars: Vec<StackedBar<X, Y>>,
    pub x_axis: Option<String>,
    pub y_axis: Option<String>,
    pub labels: HashSet<String>,
    pub x_scale: Scale<X>,
    pub y_scale: Scale<Y>,
    pub has_negatives: bool,
    pub has_positives: bool,
}

#[allow(dead_code)]
impl<X, Y> StackedBarChart<X, Y>
where
    X: Eq + Clone + Hash + PartialOrd + ToString + Debug,
    Y: Eq + Clone + Hash + PartialOrd + ToString + Debug,
{
    pub fn new(
        bars: Vec<StackedBar<X, Y>>,
        x_scale: Scale<X>,
        y_scale: Scale<Y>,
        labels: HashSet<String>,
    ) -> Result<Self, StackedBarChartError> {
        match &x_scale {
            Scale::List(scale) => Self::assert_list_scales_x(scale, &bars)?,
        };

        match &y_scale {
            Scale::List(scale) => Self::assert_list_scales_y(scale, &bars)?,
        };

        let has_negatives = bars.iter().any(|bar| bar.is_negative);

        let has_positives = bars.iter().any(|bar| !bar.is_negative);
        Ok(Self {
            x_scale,
            y_scale,
            bars,
            x_axis: None,
            y_axis: None,
            labels,
            has_negatives,
            has_positives,
        })
    }

    fn assert_list_scales_y(
        lst: &[Y],
        bars: &[StackedBar<X, Y>],
    ) -> Result<(), StackedBarChartError> {
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
            Err(StackedBarChartError::OutOfRange(
                "Y".into(),
                invalid.unwrap().to_string(),
            ))
        }
    }

    fn assert_list_scales_x(
        lst: &[X],
        bars: &[StackedBar<X, Y>],
    ) -> Result<(), StackedBarChartError> {
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
            Err(StackedBarChartError::OutOfRange(
                "X".into(),
                invalid.unwrap().to_string(),
            ))
        }
    }

    pub fn x_axis(mut self, label: impl Into<String>) -> Self {
        self.x_axis = Some(label.into());
        self
    }

    pub fn y_axis(mut self, label: impl Into<String>) -> Self {
        self.y_axis = Some(label.into());
        self
    }

    pub fn filter_negatives(&mut self) {
        self.bars.retain(|bar| !bar.is_negative);
        self.has_negatives = false;
    }

    pub fn filter_positives(&mut self) {
        self.bars.retain(|bar| bar.is_negative);
        self.has_positives = false;
    }
}

impl StackedBarChart<Data, Data> {
    /// Returns true any negative bar is not completely empty. For a Stacked bar chart, an empty point
    /// is defined as one which has a y data value of 0 or 0.0
    pub fn has_true_negatives(&self) -> bool {
        self.bars
            .iter()
            .any(|bar| bar.is_negative && !bar.is_empty())
    }

    /// Returns true any positive bar is not completely empty. For a Stacked bar chart, an empty point
    /// is defined as one which has a y data value of 0 or 0.0
    pub fn has_true_positives(&self) -> bool {
        self.bars
            .iter()
            .any(|bar| !bar.is_negative && !bar.is_empty())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StackedBarChartError {
    OutOfRange(String, String),
}

impl fmt::Display for StackedBarChartError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StackedBarChartError::OutOfRange(sc, val) => {
                write!(
                    f,
                    "The point with value {} on the {} axis is out of range",
                    val, sc
                )
            }
        }
    }
}

impl std::error::Error for StackedBarChartError {}

#[cfg(test)]
mod stacked_barchart_tests {
    use super::*;

    fn create_barchart<'a>() -> StackedBarChart<&'a str, i32> {
        let mut bars = Vec::with_capacity(5);

        let pnt = Point::new("One", 19);

        let fractions = HashMap::from([
            (String::from("Soda"), 3.0 / 19.0),
            (String::from("Cream"), 3.0 / 19.0),
            (String::from("Coffee"), 5.0 / 19.0),
            (String::from("Choco"), 8.0 / 19.0),
        ]);

        let bar = StackedBar::new(pnt, fractions, false);

        bars.push(bar);

        let pnt = Point::new("Two", 19);

        let fractions = HashMap::from([
            (String::from("Soda"), 3.0 / 19.0),
            (String::from("Cream"), 6.0 / 19.0),
            (String::from("Coffee"), 10.0 / 19.0),
            (String::from("Choco"), 0.0 / 19.0),
        ]);

        let bar = StackedBar::new(pnt, fractions, false);
        bars.push(bar);

        let pnt = Point::new("Three", 14);

        let fractions = HashMap::from([
            (String::from("Soda"), 6.0 / 14.0),
            (String::from("Cream"), 0.0 / 14.0),
            (String::from("Coffee"), 8.0 / 14.0),
            (String::from("Choco"), 0.0 / 14.0),
        ]);

        let bar = StackedBar::new(pnt, fractions, false);
        bars.push(bar);

        let pnt = Point::new("Four", 16);

        let fractions = HashMap::from([
            (String::from("Soda"), 3.0 / 16.0),
            (String::from("Cream"), 0.0 / 16.0),
            (String::from("Coffee"), 7.0 / 16.0),
            (String::from("Choco"), 6.0 / 16.0),
        ]);

        let bar = StackedBar::new(pnt, fractions, false);
        bars.push(bar);

        let pnt = Point::new("Five", 19);

        let fractions = HashMap::from([
            (String::from("Soda"), 9.0 / 19.0),
            (String::from("Cream"), 0.0 / 19.0),
            (String::from("Coffee"), 10.0 / 19.0),
            (String::from("Choco"), 0.0 / 19.0),
        ]);

        let bar = StackedBar::new(pnt, fractions, false);
        bars.push(bar);

        let x_scale: Scale<&str> = {
            let lst = vec!["One", "Two", "Three", "Four", "Five"];

            Scale::List(lst)
        };

        let y_scale: Scale<i32> = Scale::List(vec![14, 16, 19]);

        let labels = HashSet::from([
            (String::from("Soda")),
            (String::from("Cream")),
            (String::from("Coffee")),
            (String::from("Choco")),
        ]);

        match StackedBarChart::new(bars, x_scale, y_scale, labels) {
            Ok(bar) => bar.x_axis("Number").y_axis("Total"),
            Err(e) => panic!("{}", e),
        }
    }

    fn out_of_range() -> Result<StackedBarChart<isize, isize>, StackedBarChartError> {
        let xs = [1, 5, 6, 11, 15];
        let ys = [4, 5, 6, 7, 8];

        let bars = xs
            .into_iter()
            .zip(ys.into_iter())
            .map(|point| StackedBar::from_point(point, false))
            .collect();

        let x_scale: Scale<isize> = {
            let rng = -5..11;

            Scale::List(rng.collect())
        };
        let y_scale: Scale<isize> = {
            let rng = 2..10;
            Scale::List(rng.collect())
        };

        StackedBarChart::new(bars, x_scale, y_scale, HashSet::default())
    }

    #[test]
    fn test_barchart() {
        let barchart = create_barchart();

        assert_eq!(barchart.x_axis.unwrap(), String::from("Number"));
        assert_eq!(barchart.y_axis.unwrap(), String::from("Total"));

        assert_eq!(
            barchart.bars[0].fractions.get(&String::from("Soda")),
            Some(&(3.0 / 19.0))
        );

        assert_eq!(
            barchart.labels,
            HashSet::from([
                String::from("Soda"),
                String::from("Cream"),
                String::from("Coffee"),
                String::from("Choco"),
            ])
        );

        assert_eq!(barchart.bars.len(), 5)
    }

    #[test]
    fn test_faulty_barchart() {
        let expected = StackedBarChartError::OutOfRange(String::from("X"), String::from("11"));
        match out_of_range() {
            Ok(_) => panic!("Should not reach this test case"),
            Err(e) => assert_eq!(e, expected),
        }
    }
}

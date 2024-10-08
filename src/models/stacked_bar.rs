use std::{
    collections::{HashMap, HashSet},
    fmt::{self, Debug},
};

use super::{Point, Scale};
use crate::repr::Data;

#[derive(Clone, Debug, PartialEq)]
pub struct StackedBar {
    /// The (x, y) points for the bar
    pub point: Point,
    /// The percentage makeup  of the bar. For all
    /// k, v in `fractions` v1 + v2 + v3 + .. = 1.0
    pub fractions: HashMap<String, f64>,
    /// Is true of all points within the bar are negative
    pub is_negative: bool,
    /// The full value of the stacked bar
    true_y: Data,
    /// Keeps track of sections removed from the bar
    removed_sections: HashSet<String>,
}

impl StackedBar {
    pub(crate) fn new(point: Point, fractions: HashMap<String, f64>, is_negative: bool) -> Self {
        let true_y = point.y.clone();
        Self {
            point,
            fractions,
            is_negative,
            true_y,
            removed_sections: HashSet::new(),
        }
    }

    pub fn from_point(point: impl Into<Point>, is_negative: bool) -> Self {
        let point = point.into();
        let true_y = point.y.clone();
        Self {
            point,
            fractions: HashMap::default(),
            is_negative,
            true_y,
            removed_sections: HashSet::new(),
        }
    }

    pub fn restore(&mut self) {
        self.point.y = self.true_y.clone();
    }

    pub fn get_fractions(&self) -> &HashMap<String, f64> {
        &self.fractions
    }

    pub fn get_point(&self) -> &Point {
        &self.point
    }

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

    /// Effectively removes the contribution of specified section from the
    /// stacked bar if it exists
    pub fn remove_section(&mut self, section: impl Into<String>) {
        let section = section.into();

        if self.removed_sections.contains(&section) {
            return;
        }

        let fraction = self.fractions.get(&section);

        let Some(fraction) = fraction else { return };

        let contribution = match self.true_y {
            Data::Number(n) => (n as f64) * fraction,
            Data::Integer(i) => (i as f64) * fraction,
            Data::Float(f) => (f as f64) * fraction,
            _ => 0.0,
        };

        match self.point.y {
            Data::Number(n) => self.point.y = Data::Number(((n as f64) - contribution) as isize),
            Data::Integer(i) => self.point.y = Data::Integer(((i as f64) - contribution) as i32),
            Data::Float(f) => self.point.y = Data::Float(((f as f64) - contribution) as f32),
            _ => {}
        };

        self.removed_sections.insert(section);
    }

    /// Effectively re-adds the contribution of specified section to the
    /// stacked bar if it exists
    pub fn add_section(&mut self, section: impl Into<String>) {
        let section = section.into();

        if !self.removed_sections.contains(&section) {
            return;
        }

        let fraction = self.fractions.get(&section);

        let Some(fraction) = fraction else { return };

        let contribution = match self.true_y {
            Data::Number(n) => (n as f64) * fraction,
            Data::Integer(i) => (i as f64) * fraction,
            Data::Float(f) => (f as f64) * fraction,
            _ => 0.0,
        };

        match self.point.y {
            Data::Number(n) => self.point.y = Data::Number(((n as f64) + contribution) as isize),
            Data::Integer(i) => self.point.y = Data::Integer(((i as f64) + contribution) as i32),
            Data::Float(f) => self.point.y = Data::Float(((f as f64) + contribution) as f32),
            _ => {}
        }

        self.removed_sections.remove(&section);
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct StackedBarChart {
    pub bars: Vec<StackedBar>,
    pub x_axis: Option<String>,
    pub y_axis: Option<String>,
    pub labels: HashSet<String>,
    pub x_scale: Scale,
    pub y_scale: Scale,
    pub has_negatives: bool,
    pub has_positives: bool,
}

#[allow(dead_code)]
impl StackedBarChart {
    pub(crate) fn new(
        bars: Vec<StackedBar>,
        x_scale: Scale,
        y_scale: Scale,
        labels: HashSet<String>,
    ) -> Result<Self, StackedBarChartError> {
        Self::assert_x_scale(&x_scale, &bars)?;
        Self::assert_y_scale(&y_scale, &bars)?;

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

    fn assert_x_scale(scale: &Scale, bars: &[StackedBar]) -> Result<(), StackedBarChartError> {
        for x in bars.iter().map(|bar| &bar.point.x) {
            if !scale.contains(x) {
                return Err(StackedBarChartError::OutOfRange(
                    "X".to_string(),
                    x.to_string(),
                ));
            }
        }

        Ok(())
    }

    fn assert_y_scale(scale: &Scale, bars: &[StackedBar]) -> Result<(), StackedBarChartError> {
        for y in bars.iter().map(|bar| &bar.point.y) {
            if !scale.contains(y) {
                return Err(StackedBarChartError::OutOfRange(
                    "Y".to_string(),
                    y.to_string(),
                ));
            }
        }

        Ok(())
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

    pub fn remove_section(&mut self, bar: usize, section: impl Into<String>) {
        if let Some(bar) = self.bars.get_mut(bar) {
            bar.remove_section(section);
        };
    }

    pub fn remove_section_all(&mut self, section: impl Into<String>) {
        let section: String = section.into();
        self.bars.iter_mut().for_each(|bar| {
            bar.remove_section(section.clone());
        });
    }

    pub fn add_section(&mut self, bar: usize, section: impl Into<String>) {
        if let Some(bar) = self.bars.get_mut(bar) {
            bar.add_section(section);
        };
    }

    pub fn add_section_all(&mut self, section: impl Into<String>) {
        let section: String = section.into();
        self.bars.iter_mut().for_each(|bar| {
            bar.add_section(section.clone());
        });
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
    use crate::models::ScaleKind;

    use super::*;

    fn create_barchart<'a>() -> StackedBarChart {
        let mut bars = Vec::with_capacity(5);

        let pnt = Point::new(Data::Text("One".into()), Data::Integer(19));

        let fractions = HashMap::from([
            (String::from("Soda"), 3.0 / 19.0),
            (String::from("Cream"), 3.0 / 19.0),
            (String::from("Coffee"), 5.0 / 19.0),
            (String::from("Choco"), 8.0 / 19.0),
        ]);

        let bar = StackedBar::new(pnt, fractions, false);

        bars.push(bar);

        let pnt = Point::new(Data::Text("Two".into()), Data::Integer(19));

        let fractions = HashMap::from([
            (String::from("Soda"), 3.0 / 19.0),
            (String::from("Cream"), 6.0 / 19.0),
            (String::from("Coffee"), 10.0 / 19.0),
            (String::from("Choco"), 0.0 / 19.0),
        ]);

        let bar = StackedBar::new(pnt, fractions, false);
        bars.push(bar);

        let pnt = Point::new(Data::Text("Three".into()), Data::Integer(14));

        let fractions = HashMap::from([
            (String::from("Soda"), 6.0 / 14.0),
            (String::from("Cream"), 0.0 / 14.0),
            (String::from("Coffee"), 8.0 / 14.0),
            (String::from("Choco"), 0.0 / 14.0),
        ]);

        let bar = StackedBar::new(pnt, fractions, false);
        bars.push(bar);

        let pnt = Point::new(Data::Text("Four".into()), Data::Integer(16));

        let fractions = HashMap::from([
            (String::from("Soda"), 3.0 / 16.0),
            (String::from("Cream"), 0.0 / 16.0),
            (String::from("Coffee"), 7.0 / 16.0),
            (String::from("Choco"), 6.0 / 16.0),
        ]);

        let bar = StackedBar::new(pnt, fractions, false);
        bars.push(bar);

        let pnt = Point::new(Data::Text("Five".into()), Data::Integer(19));

        let fractions = HashMap::from([
            (String::from("Soda"), 9.0 / 19.0),
            (String::from("Cream"), 0.0 / 19.0),
            (String::from("Coffee"), 10.0 / 19.0),
            (String::from("Choco"), 0.0 / 19.0),
        ]);

        let bar = StackedBar::new(pnt, fractions, false);
        bars.push(bar);

        let x_scale = {
            let values = vec!["One", "Two", "Three", "Four", "Five"];

            Scale::new(values, ScaleKind::Categorical)
        };

        let y_scale = vec![14, 16, 19].into();

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

    fn out_of_range() -> Result<StackedBarChart, StackedBarChartError> {
        let xs = [1, 5, 6, 11, 15];
        let ys = [4, 5, 6, 7, 8];

        let bars = xs
            .into_iter()
            .zip(ys.into_iter())
            .map(|point| {
                StackedBar::from_point((Data::Integer(point.0), Data::Integer(point.1)), false)
            })
            .collect();

        let x_scale = {
            let rng = -5..11;

            Scale::new(rng, ScaleKind::Integer)
        };
        let y_scale = {
            let rng = 2..10;

            Scale::new(rng, ScaleKind::Integer)
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

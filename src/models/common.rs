use crate::repr::{ColumnType, Data};
use std::{collections::HashSet, fmt::Debug};

#[derive(Debug, Clone, PartialEq)]
pub struct Point<X = Data, Y = Data> {
    pub x: X,
    pub y: Y,
}

impl<X, Y> Point<X, Y> {
    pub fn new(x: X, y: Y) -> Self {
        Self { x, y }
    }
}

impl<X, Y> From<(X, Y)> for Point<X, Y> {
    fn from(value: (X, Y)) -> Self {
        Point::new(value.0, value.1)
    }
}

/// Determines how points on the scale are handled
///
///
/// Points on a [`ScaleKind::Text`] are treated categorically with all duplicates removed and in an arbitary order. Points on other [`ScaleKind`] are treated numerically as a range
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScaleKind {
    Number,
    Integer,
    Float,
    Text,
}

impl From<ColumnType> for ScaleKind {
    fn from(value: ColumnType) -> Self {
        match value {
            ColumnType::Number => ScaleKind::Number,
            ColumnType::Integer => ScaleKind::Integer,
            ColumnType::Float => ScaleKind::Float,
            _ => ScaleKind::Text,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum ScaleValues {
    Number {
        start: isize,
        end: isize,
        step: isize,
    },
    Integer {
        start: i32,
        end: i32,
        step: i32,
    },
    Float {
        start: f32,
        end: f32,
        step: f32,
    },
    Text(Vec<String>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Scale {
    pub kind: ScaleKind,
    values: ScaleValues,
}

impl Scale {
    /// Returns a new scale of the specified type from the given points.
    /// If the scale type specified cannot be created from the points, a [`ScaleKind::Text`] is
    /// created instead.
    pub(crate) fn new(points: impl Iterator<Item = Data>, kind: ScaleKind) -> Self {
        match kind {
            ScaleKind::Text => {
                let values = points
                    .map(|point| point.to_string())
                    .collect::<HashSet<String>>();
                let values = ScaleValues::Text(values.into_iter().collect());

                Self { kind, values }
            }
            ScaleKind::Integer => {
                let mut valid = HashSet::new();
                let mut invalid = HashSet::new();

                for point in points {
                    match point {
                        Data::Integer(num) => {
                            valid.insert(num);
                        }
                        other => {
                            invalid.insert(other.to_string());
                        }
                    };
                }

                if !invalid.is_empty() {
                    for point in valid.into_iter() {
                        invalid.insert(point.to_string());
                    }

                    Self {
                        kind: ScaleKind::Text,
                        values: ScaleValues::Text(invalid.into_iter().collect()),
                    }
                } else {
                    Self::from_i32(valid.into_iter().collect::<Vec<i32>>())
                }
            }
            ScaleKind::Number => {
                let mut valid = HashSet::new();
                let mut invalid = HashSet::new();

                for point in points {
                    match point {
                        Data::Number(num) => {
                            valid.insert(num);
                        }
                        other => {
                            invalid.insert(other.to_string());
                        }
                    }
                }

                if !invalid.is_empty() {
                    for point in valid.into_iter() {
                        invalid.insert(point.to_string());
                    }

                    Self {
                        kind: ScaleKind::Text,
                        values: ScaleValues::Text(invalid.into_iter().collect()),
                    }
                } else {
                    Self::from_isize(valid.into_iter().collect::<Vec<isize>>())
                }
            }
            ScaleKind::Float => {
                // f32 doesn't implement Hash or Eq
                let mut valid: Vec<f32> = Vec::new();
                let mut invalid = HashSet::new();

                for point in points {
                    match point {
                        Data::Float(float) => {
                            if !valid.iter().any(|x| *x == float) {
                                valid.push(float);
                            }
                        }
                        other => {
                            invalid.insert(other.to_string());
                        }
                    }
                }

                if !invalid.is_empty() {
                    for point in valid.into_iter() {
                        invalid.insert(point.to_string());
                    }

                    Self {
                        kind: ScaleKind::Text,
                        values: ScaleValues::Text(invalid.into_iter().collect()),
                    }
                } else {
                    Self::from_f32(valid)
                }
            }
        }
    }

    pub fn points(&self) -> Vec<Data> {
        match &self.values {
            ScaleValues::Text(values) => values.iter().cloned().map(From::from).collect(),
            ScaleValues::Number { start, end, step } => {
                let mut output = Vec::default();
                let n = ((end - start) / step) + 1;

                for i in 0..n {
                    let curr = *start + (i * step);
                    output.push(Data::Number(curr));
                }

                output
            }
            ScaleValues::Integer { start, end, step } => {
                let mut output = Vec::default();
                let n = ((end - start) / step) + 1;

                for i in 0..n {
                    let curr = *start + (i * step);
                    output.push(Data::Integer(curr));
                }

                output
            }
            ScaleValues::Float { start, end, step } => {
                let mut output = Vec::default();
                let n = (((end - start) / step) + 1.0) as isize;

                for i in 0..n {
                    let curr = *start + ((i as f32) * step);
                    output.push(Data::Float(curr));
                }

                output
            }
        }
    }

    pub fn contains(&self, value: &Data) -> bool {
        match (&self.values, value) {
            (ScaleValues::Text(values), Data::Text(val)) => values.contains(val),
            (ScaleValues::Number { start, end, .. }, Data::Number(num)) => {
                start <= num && num <= end
            }
            (ScaleValues::Integer { start, end, .. }, Data::Integer(num)) => {
                start <= num && num <= end
            }
            (ScaleValues::Float { start, end, .. }, Data::Float(num)) => start <= num && num <= end,
            _ => false,
        }
    }

    /// Assumes points is not empty
    pub fn from_i32(points: impl Into<Vec<i32>>) -> Self {
        let mut points: Vec<i32> = points.into();
        points.sort();

        // Scale contains both negative and positive values
        let temp = points.iter().fold((false, false), |acc, curr| {
            if acc.0 && acc.1 {
                acc
            } else if *curr < 0 {
                (true, acc.1)
            } else {
                (acc.0, true)
            }
        });

        let len = points.len();
        let min = points.first().unwrap(); // Guaranteed by iteration in Self::new
        let max = points.get(len - 1).unwrap(); // Guaranteed by iteration in Self::new
        let len = if temp.0 && temp.1 {
            points.len() + 1
        } else {
            points.len()
        };

        let mut step = (max - min) / len as i32;

        if step * (len as i32) != max - min {
            step += 1
        }

        Self {
            kind: ScaleKind::Integer,
            values: ScaleValues::Integer {
                start: *min,
                end: *max,
                step,
            },
        }
    }

    /// Assumes points is not empty
    pub fn from_isize(points: impl Into<Vec<isize>>) -> Self {
        let mut points: Vec<isize> = points.into();
        points.sort();

        let temp = points.iter().fold((false, false), |acc, curr| {
            if acc.0 && acc.1 {
                acc
            } else if *curr < 0 {
                (true, acc.1)
            } else {
                (acc.0, true)
            }
        });

        let len = points.len();
        let min = points.first().unwrap(); // Guaranteed by iteration in Self::new
        let max = points.get(len - 1).unwrap(); // Guaranteed by iteration in Self::new
        let len = if temp.0 && temp.1 {
            points.len() + 1
        } else {
            points.len()
        };

        let mut step = (max - min) / len as isize;

        if step * (len as isize) != max - min {
            step += 1
        }

        Self {
            kind: ScaleKind::Number,
            values: ScaleValues::Number {
                start: *min,
                end: *max,
                step,
            },
        }
    }

    pub fn from_f32(points: impl Into<Vec<f32>>) -> Self {
        let points: Vec<f32> = points.into();

        let mut min = *points.first().unwrap();
        let mut max = *points.first().unwrap();

        for point in points.iter() {
            let point = *point;

            if point < min {
                min = point;
                continue;
            }

            if point > max {
                max = point;
                continue;
            }
        }

        let temp = points.iter().fold((false, false), |acc, curr| {
            if acc.0 && acc.1 {
                acc
            } else if *curr < 0.0 {
                (true, acc.1)
            } else {
                (acc.0, true)
            }
        });

        let len = if temp.0 && temp.1 {
            points.len() + 1
        } else {
            points.len()
        };

        let mut step = (max - min) / len as f32;

        if step * (len as f32) != max - min {
            step += 1.0
        }

        Self {
            kind: ScaleKind::Float,
            values: ScaleValues::Float {
                start: min,
                end: max,
                step,
            },
        }
    }

    pub fn sort(&mut self) {
        if let ScaleValues::Text(values) = &mut self.values {
            values.sort();
        }
    }
}

impl From<Vec<i32>> for Scale {
    fn from(value: Vec<i32>) -> Self {
        let values = value.into_iter().map(From::from);
        Self::new(values, ScaleKind::Integer)
    }
}

impl From<Vec<isize>> for Scale {
    fn from(value: Vec<isize>) -> Self {
        let values = value.into_iter().map(From::from);
        Self::new(values, ScaleKind::Number)
    }
}

impl From<Vec<f32>> for Scale {
    fn from(value: Vec<f32>) -> Self {
        let values = value.into_iter().map(From::from);
        Self::new(values, ScaleKind::Float)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_point<X, Y>(x: X, y: Y) -> Point<X, Y> {
        Point::new(x, y)
    }

    #[test]
    fn test_point() {
        let p1 = create_point(2, 3);
        assert_eq!(p1.x, 2);
        assert_eq!(p1.y, 3);

        let p2 = create_point(-4, 0);
        assert_eq!(p2.x, -4);
        assert_eq!(p2.y, 0);

        let p3 = create_point("Something", "else");
        assert_eq!(p3.x, "Something");
        assert_eq!(p3.y, "else");

        let p4: Point<&str, f32> = ("tired", 0.50).into();
        assert_eq!(p4.x, "tired");
        assert_eq!(p4.y, 0.50);
    }
}

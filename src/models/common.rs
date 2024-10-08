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
    pub length: usize,
}

impl Scale {
    /// Returns a new scale of the specified type from the given points.
    /// If the scale type specified cannot be created from the points, a [`ScaleKind::Text`] is
    /// created instead.
    pub(crate) fn new(points: impl IntoIterator<Item = impl Into<Data>>, kind: ScaleKind) -> Self {
        let points = points.into_iter().map(Into::into);
        match kind {
            ScaleKind::Text => {
                let values = points
                    .map(|point| point.to_string())
                    .collect::<HashSet<String>>();
                let values = values.into_iter().collect::<Vec<String>>();
                let length = values.len();
                let values = ScaleValues::Text(values);

                Self {
                    kind,
                    values,
                    length,
                }
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

                if valid.is_empty() && invalid.is_empty() {
                    Self {
                        kind,
                        values: ScaleValues::Integer {
                            start: 0,
                            end: 0,
                            step: 0,
                        },
                        length: 1,
                    }
                } else if !invalid.is_empty() {
                    for point in valid.into_iter() {
                        invalid.insert(point.to_string());
                    }

                    let invalid = invalid.into_iter().collect::<Vec<String>>();
                    let length = invalid.len();

                    Self {
                        kind: ScaleKind::Text,
                        values: ScaleValues::Text(invalid),
                        length,
                    }
                } else {
                    Self::from_i32(valid.into_iter())
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

                if valid.is_empty() && invalid.is_empty() {
                    Self {
                        kind,
                        values: ScaleValues::Number {
                            start: 0,
                            end: 0,
                            step: 0,
                        },
                        length: 1,
                    }
                } else if !invalid.is_empty() {
                    for point in valid.into_iter() {
                        invalid.insert(point.to_string());
                    }

                    let invalid = invalid.into_iter().collect::<Vec<String>>();
                    let length = invalid.len();

                    Self {
                        kind: ScaleKind::Text,
                        values: ScaleValues::Text(invalid),
                        length,
                    }
                } else {
                    Self::from_isize(valid.into_iter())
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

                if valid.is_empty() && invalid.is_empty() {
                    Self {
                        kind,
                        values: ScaleValues::Float {
                            start: 0.0,
                            end: 0.0,
                            step: 0.0,
                        },
                        length: 1,
                    }
                } else if !invalid.is_empty() {
                    for point in valid.into_iter() {
                        invalid.insert(point.to_string());
                    }

                    let invalid = invalid.into_iter().collect::<Vec<String>>();
                    let length = invalid.len();

                    Self {
                        kind: ScaleKind::Text,
                        values: ScaleValues::Text(invalid),
                        length,
                    }
                } else {
                    Self::from_f32(valid.into_iter())
                }
            }
        }
    }

    pub fn points(&self) -> Vec<Data> {
        match &self.values {
            ScaleValues::Text(values) => values.iter().cloned().map(Data::Text).collect(),
            ScaleValues::Number { start, step, .. } => {
                let mut output = Vec::default();
                let n = self.length as isize;

                for i in 0..n {
                    let curr = *start + (i * step);
                    output.push(Data::Number(curr));
                }

                output
            }
            ScaleValues::Integer { start, step, .. } => {
                let mut output = Vec::default();
                let n = self.length as i32;

                for i in 0..n {
                    let curr = *start + (i * step);
                    output.push(Data::Integer(curr));
                }

                output
            }
            ScaleValues::Float { start, step, .. } => {
                let mut output = Vec::default();
                let n = self.length as isize;

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
            (ScaleValues::Number { start, step, .. }, Data::Number(num)) => {
                let end = start + (*step * (self.length - 1) as isize);
                start <= num && num <= &end
            }
            (ScaleValues::Integer { start, step, .. }, Data::Integer(num)) => {
                let end = start + (*step * (self.length - 1) as i32);
                start <= num && num <= &end
            }
            (ScaleValues::Float { start, step, .. }, Data::Float(num)) => {
                let end = start + (*step * (self.length - 1) as f32);
                start <= num && num <= &end
            }
            _ => false,
        }
    }

    /// Assumes points is not empty
    fn from_i32(points: impl Iterator<Item = i32>) -> Self {
        let deduped = points.collect::<HashSet<i32>>();

        let mut min = None;
        let mut max = None;
        let mut has_neg = false;
        let mut has_pos = false;

        for num in deduped.iter() {
            let num = *num;
            has_neg = num < 0;
            has_pos = num >= 0;

            if let Some(prev) = min {
                if num < prev {
                    min = Some(num);
                }
            } else {
                min = Some(num);
            }

            if let Some(prev) = max {
                if num > prev {
                    max = Some(num);
                }
            } else {
                max = Some(num);
            }
        }

        let len = if has_pos && has_neg {
            deduped.len() + 1
        } else {
            deduped.len()
        };

        let min = min.unwrap();
        let max = max.unwrap();
        let mut step = (max - min) / len as i32;

        if step * (len as i32) != max - min {
            step += 1
        }

        let length = if ((len - 1) as i32) * step + min < max {
            len + 1
        } else {
            len
        };

        Self {
            kind: ScaleKind::Integer,
            length,
            values: ScaleValues::Integer {
                start: min,
                end: max,
                step,
            },
        }
    }

    /// Assumes points is not empty
    fn from_isize(points: impl Iterator<Item = isize>) -> Self {
        let deduped = points.collect::<HashSet<isize>>();

        let mut min = None;
        let mut max = None;
        let mut has_neg = false;
        let mut has_pos = false;

        for num in deduped.iter() {
            let num = *num;
            has_neg = num < 0;
            has_pos = num >= 0;

            if let Some(prev) = min {
                if num < prev {
                    min = Some(num);
                }
            } else {
                min = Some(num);
            }

            if let Some(prev) = max {
                if num > prev {
                    max = Some(num);
                }
            } else {
                max = Some(num);
            }
        }

        let len = if has_pos && has_neg {
            deduped.len() + 1
        } else {
            deduped.len()
        };

        let min = min.unwrap();
        let max = max.unwrap();
        let mut step = (max - min) / len as isize;

        if step * (len as isize) != max - min {
            step += 1
        }

        let length = if ((len - 1) as isize) * step + min < max {
            len + 1
        } else {
            len
        };

        Self {
            kind: ScaleKind::Number,
            length,
            values: ScaleValues::Number {
                start: min,
                end: max,
                step,
            },
        }
    }

    fn from_f32(points: impl Iterator<Item = f32>) -> Self {
        let mut min = None;
        let mut max = None;
        let mut has_neg = false;
        let mut has_pos = false;
        let mut seen = Vec::default();

        for point in points {
            if !seen.iter().any(|pnt| *pnt == point) {
                seen.push(point);
            }

            has_neg = point < 0.0;
            has_pos = point >= 0.0;

            // I'm not quite certain how the < and > would work around NaN,
            if let Some(prev) = min {
                if point < prev {
                    min = Some(point);
                }
            } else {
                min = Some(point);
            }

            if let Some(prev) = min {
                if point > prev {
                    max = Some(point);
                }
            } else {
                max = Some(point);
            }
        }

        let min = min.unwrap();
        let max = max.unwrap();

        let len = if has_pos && has_neg {
            seen.len() + 1
        } else {
            seen.len()
        };

        let mut step = (max - min) / len as f32;

        if step * (len as f32) != max - min {
            step += 1.0
        }

        let length = if ((len - 1) as f32) * step + min < max {
            len + 1
        } else {
            len
        };

        Self {
            kind: ScaleKind::Float,
            length,
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
        Self::new(value, ScaleKind::Integer)
    }
}

impl From<Vec<isize>> for Scale {
    fn from(value: Vec<isize>) -> Self {
        Self::new(value, ScaleKind::Number)
    }
}

impl From<Vec<f32>> for Scale {
    fn from(value: Vec<f32>) -> Self {
        Self::new(value, ScaleKind::Float)
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

    #[test]
    fn test_scale_dedup() {
        let pnts = vec![1, 2, 3, 4, 5];
        let scale = Scale::new(pnts, ScaleKind::Integer);

        assert_eq!(scale.length, 5);
        assert_eq!(
            scale.points(),
            vec![
                Data::Integer(1),
                Data::Integer(2),
                Data::Integer(3),
                Data::Integer(4),
                Data::Integer(5)
            ]
        );

        let pnts = vec![1, 3, 2, 2, 3, 4, 1, 5];
        let scale = Scale::new(pnts, ScaleKind::Integer);

        assert_eq!(scale.length, 5);
        assert_eq!(
            scale.points(),
            vec![
                Data::Integer(1),
                Data::Integer(2),
                Data::Integer(3),
                Data::Integer(4),
                Data::Integer(5)
            ]
        );

        let pnts: Vec<isize> = vec![1, 12, 12, 6, 4, 1, 25];
        let scale = Scale::new(pnts, ScaleKind::Number);

        assert_eq!(scale.length, 6);
        assert!(scale.contains(&Data::Number(25)));
        assert!(scale.contains(&Data::Number(26)));
        assert!(!scale.contains(&Data::Number(30)));
        assert!(!scale.contains(&Data::Integer(25)));

        let pnts: Vec<f32> = vec![1.0, 3.0, 2.0, 2.0, 3.0, 4.0, 1.0, 5.0];
        let scale = Scale::new(pnts, ScaleKind::Float);

        assert_eq!(scale.length, 6);
        assert!(!scale.contains(&Data::Float(0.99)));

        let pnts: Vec<isize> = vec![1, 12, 12, 6, 4, 1, 25];
        let mut scale = Scale::new(pnts, ScaleKind::Text);
        scale.sort();

        assert_eq!(scale.length, 5);
        assert_eq!(
            scale.points(),
            vec![
                Data::Text("1".into()),
                Data::Text("12".into()),
                Data::Text("25".into()),
                Data::Text("4".into()),
                Data::Text("6".into()),
            ]
        );

        let pnts = vec![
            Data::Integer(44),
            Data::Text("Test".into()),
            Data::None,
            Data::Integer(4),
        ];
        let mut scale = Scale::new(pnts, ScaleKind::Integer);
        scale.sort();

        assert_eq!(scale.length, 4);
        assert_eq!(
            scale.points(),
            vec![
                Data::Text("4".into()),
                Data::Text("44".into()),
                Data::Text("<None>".into()),
                Data::Text("Test".into()),
            ]
        );
        assert!(scale.contains(&Data::Text("44".into())));
        assert!(!scale.contains(&Data::Integer(44)));
        assert!(!scale.contains(&Data::None));
        assert!(scale.contains(&Data::Text("Test".into())));
        assert!(scale.contains(&Data::Text("<None>".into())));
    }

    #[test]
    fn test_scale_pos_neg() {
        let pnts = vec![-1, -8, -3];
        let scale = Scale::new(pnts, ScaleKind::Integer);

        assert_eq!(scale.length, 4);
        assert_eq!(
            scale.points(),
            vec![
                Data::Integer(-8),
                Data::Integer(-5),
                Data::Integer(-2),
                Data::Integer(1),
            ]
        );
        assert!(scale.contains(&Data::Integer(0)));

        let pnts = vec![-2, 0, 1, 2, 5];
        let scale = Scale::new(pnts, ScaleKind::Integer);

        assert_eq!(scale.length, 5);
        assert_eq!(
            scale.points(),
            vec![
                Data::Integer(-2),
                Data::Integer(0),
                Data::Integer(2),
                Data::Integer(4),
                Data::Integer(6),
            ]
        );
        assert!(scale.contains(&Data::Integer(6)));
        assert!(!scale.contains(&Data::Integer(-3)));

        let pnts = vec![-3, -10, -1, 2, -5];
        let scale = Scale::new(pnts, ScaleKind::Integer);

        assert_eq!(scale.length, 5);
        assert_eq!(
            scale.points(),
            vec![
                Data::Integer(-10),
                Data::Integer(-7),
                Data::Integer(-4),
                Data::Integer(-1),
                Data::Integer(2),
            ]
        );
    }

    #[test]
    fn test_scale_single() {
        let pnt = vec![1];
        let scale = Scale::new(pnt, ScaleKind::Integer);

        assert_eq!(scale.length, 1);
        assert_eq!(scale.points(), vec![Data::Integer(1)]);
        assert!(scale.contains(&Data::Integer(1)));
        assert!(!scale.contains(&Data::Integer(2)));
    }

    #[test]
    fn test_scale_none() {
        let scale = Scale::new(Vec::<Data>::new(), ScaleKind::Integer);

        assert_eq!(scale.length, 1);
        assert_eq!(scale.points(), vec![Data::Integer(0)]);
        assert!(scale.contains(&Data::Integer(0)));
    }
}

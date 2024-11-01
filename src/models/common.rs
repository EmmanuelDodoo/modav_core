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
/// Points on a [`ScaleKind::Categorical`] are treated categorically with all duplicates removed and in an arbitary order. Points on other [`ScaleKind`] are treated numerically as a range
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum ScaleKind {
    Number,
    Integer,
    Float,
    Categorical,
}

impl From<ColumnType> for ScaleKind {
    fn from(value: ColumnType) -> Self {
        match value {
            ColumnType::Number => ScaleKind::Number,
            ColumnType::Integer => ScaleKind::Integer,
            ColumnType::Float => ScaleKind::Float,
            _ => ScaleKind::Categorical,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum ScaleValues {
    /// Both ends are inclusive
    Number {
        start: isize,
        end: isize,
        step: isize,
    },
    /// Both ends are inclusive
    Integer {
        start: i32,
        end: i32,
        step: i32,
    },
    /// Both ends are inclusive
    Float {
        start: f32,
        end: f32,
        step: f32,
    },
    Categorical(Vec<Data>),
}

#[derive(Debug, Clone, PartialEq)]
/// Representation of [`Scale`] points on an Axis.
pub enum AxisPoints {
    /// Categorical points with no concept of negatives and positives.
    Categorical(Vec<Data>),
    /// Numeric points with positive points and negative points split. Zero is
    /// considered a positive.
    Numeric {
        positives: Vec<Data>,
        negatives: Vec<Data>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Scale {
    /// The type of scale
    pub(crate) kind: ScaleKind,
    /// The values within the scale
    values: ScaleValues,
    /// The number of points on the scale.
    ///
    /// For non-categorical data this is at most one more than the number of
    /// points used to generate the scale
    pub length: usize,
}

impl Scale {
    /// Returns a new scale of the specified type from the given points.
    /// If the scale type specified cannot be created from the points, a [`ScaleKind::Categorical`] is
    /// created instead.
    pub(crate) fn new(points: impl IntoIterator<Item = impl Into<Data>>, kind: ScaleKind) -> Self {
        let points = points.into_iter().map(Into::into);
        match kind {
            ScaleKind::Categorical => {
                let mut values = Vec::default();

                for point in points {
                    if !values.iter().any(|pnt| pnt == &point) {
                        values.push(point);
                    }
                }

                let length = values.len();
                let values = ScaleValues::Categorical(values);

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
                            invalid.insert(other);
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
                        invalid.insert(point.into());
                    }

                    let invalid = invalid.into_iter().collect::<Vec<Data>>();
                    let length = invalid.len();

                    Self {
                        kind: ScaleKind::Categorical,
                        values: ScaleValues::Categorical(invalid),
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
                            invalid.insert(other);
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
                        invalid.insert(point.into());
                    }

                    let invalid = invalid.into_iter().collect::<Vec<Data>>();
                    let length = invalid.len();

                    Self {
                        kind: ScaleKind::Categorical,
                        values: ScaleValues::Categorical(invalid),
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
                            invalid.insert(other);
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
                        invalid.insert(point.into());
                    }

                    let invalid = invalid.into_iter().collect::<Vec<Data>>();
                    let length = invalid.len();

                    Self {
                        kind: ScaleKind::Categorical,
                        values: ScaleValues::Categorical(invalid),
                        length,
                    }
                } else {
                    Self::from_f32(valid.into_iter())
                }
            }
        }
    }

    /// Returns the points on the scale.
    ///
    /// Categorical scales return all points used to generate the scale.
    ///
    /// Non-Categorical scales return a ordered generated range, guaranteed to contain all initial points.
    pub fn points(&self) -> Vec<Data> {
        match &self.values {
            ScaleValues::Categorical(values) => values.clone(),
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

    /// Returns the successive points on the scale. For categorical and floating
    /// point scales, this is the same as [`Scale::points`]
    ///
    /// # Example
    ///
    /// ```
    /// use modav_core::{repr::Data, models::Scale};
    ///
    /// let scale = Scale::from(vec![1,2,9,10]);
    /// assert_eq!(scale.ranged(), (1..=10).map(From::from).collect::<Vec<Data>>())
    ///
    /// ```
    pub fn ranged(&self) -> Vec<Data> {
        match &self.values {
            ScaleValues::Integer { start, end, .. } => {
                let range = *start..=*end;
                range.map(From::from).collect()
            }
            ScaleValues::Number { start, end, .. } => {
                let range = *start..=*end;
                range.map(From::from).collect()
            }
            _ => self.points(),
        }
    }

    /// Returns true if the scale contains the given [`Data`].
    ///
    /// For non-categorical scales, true is returned if a valid data value falls
    /// the range min(Scale::points), max(Scale::points).
    ///
    /// # Example
    ///
    /// ```
    /// use modav_core::{repr::Data, models::Scale};
    ///
    /// let scale = Scale::from(vec![1,3,4,5]);
    /// assert!(scale.contains(&Data::Integer(3)));
    ///
    /// /// Returns true, even though 2 was not in original scale points
    /// assert!(scale.contains(&Data::Integer(2)));
    ///
    /// /// scale doesn't contain [`Data::Number`]
    /// assert!(!scale.contains(&Data::Number(3)));
    ///
    /// ```
    pub fn contains(&self, value: &Data) -> bool {
        match (&self.values, value) {
            (ScaleValues::Categorical(values), data) => values.contains(data),
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

    /// Returns the points on the scale as a [`AxisPoints`].
    ///
    /// For non-categorical, non-floating point scales, points are generated
    /// sequentially if `sequential` is true.
    ///
    /// Points for non-categorical scales are guaranteed to be in order.
    pub fn axis_points(&self, sequential: bool) -> AxisPoints {
        match &self.values {
            ScaleValues::Categorical(vals) => AxisPoints::Categorical(vals.clone()),
            ScaleValues::Number { start, end, step } => {
                let mut pos = vec![];
                let mut neg = vec![];

                if sequential {
                    for i in *start..=*end {
                        if i < 0 {
                            neg.push(i.into());
                        } else {
                            pos.push(i.into());
                        }
                    }
                } else {
                    let n = self.length as isize;

                    for i in 0..n {
                        let curr = *start + (i * step);
                        if curr < 0 {
                            neg.push(curr.into());
                        } else {
                            pos.push(curr.into());
                        }
                    }
                }

                AxisPoints::Numeric {
                    positives: pos,
                    negatives: neg,
                }
            }
            ScaleValues::Integer { start, end, step } => {
                let mut pos = vec![];
                let mut neg = vec![];

                if sequential {
                    for i in *start..=*end {
                        if i < 0 {
                            neg.push(i.into());
                        } else {
                            pos.push(i.into());
                        }
                    }
                } else {
                    let n = self.length as i32;

                    for i in 0..n {
                        let curr = *start + (i * step);
                        if curr < 0 {
                            neg.push(curr.into());
                        } else {
                            pos.push(curr.into());
                        }
                    }
                }

                AxisPoints::Numeric {
                    positives: pos,
                    negatives: neg,
                }
            }
            ScaleValues::Float { start, step, .. } => {
                let mut pos = vec![];
                let mut neg = vec![];

                let n = self.length;

                for i in 0..n {
                    let curr = *start + ((i as f32) * step);
                    if curr < 0.0 {
                        neg.push(curr.into());
                    } else {
                        pos.push(curr.into());
                    }
                }

                AxisPoints::Numeric {
                    positives: pos,
                    negatives: neg,
                }
            }
        }
    }

    /// Returns true if the scale is categorical
    pub fn is_categorical(&self) -> bool {
        self.kind == ScaleKind::Categorical
    }

    /// Assumes points is not empty
    fn from_i32(points: impl Iterator<Item = i32>) -> Self {
        let deduped = points.collect::<HashSet<i32>>();

        let mut min = None;
        let mut max = None;

        for num in deduped.iter() {
            let num = *num;

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

        let mut length = deduped.len();

        let min = min.unwrap();
        let max = max.unwrap();
        let mut step = (max - min) / length as i32;

        if step * (length as i32) != max - min {
            step += 1;
        }

        if ((length - 1) as i32) * step + min < max {
            length += 1;
        }

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

        for num in deduped.iter() {
            let num = *num;

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

        let mut length = deduped.len();

        let min = min.unwrap();
        let max = max.unwrap();
        let mut step = (max - min) / length as isize;

        if step * (length as isize) != max - min {
            step += 1;
        }

        if ((length - 1) as isize) * step + min < max {
            length += 1;
        }

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
        let mut seen = Vec::default();

        for point in points {
            if !seen.iter().any(|pnt| *pnt == point) {
                seen.push(point);
            }

            // I'm not quite certain how the < and > would work around NaN,
            if let Some(prev) = min {
                if point < prev {
                    min = Some(point);
                }
            } else {
                min = Some(point);
            }

            if let Some(prev) = max {
                if point > prev {
                    max = Some(point);
                }
            } else {
                max = Some(point);
            }
        }

        let min = min.unwrap();
        let max = max.unwrap();
        println!("{min}: {max}");

        let mut length = seen.len();

        let mut step = (max - min) / length as f32;

        if step * (length as f32) != max - min {
            step += 1.0
        }

        if ((length - 1) as f32) * step + min < max {
            length += 1;
        }

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
        if let ScaleValues::Categorical(values) = &mut self.values {
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
        dbg!(&scale);

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
        let mut scale = Scale::new(pnts, ScaleKind::Categorical);
        scale.sort();

        assert_eq!(scale.length, 5);
        assert_eq!(
            scale.points(),
            vec![
                Data::Number(1),
                Data::Number(4),
                Data::Number(6),
                Data::Number(12),
                Data::Number(25),
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
                Data::None,
                Data::Integer(4),
                Data::Integer(44),
                Data::Text("Test".into()),
            ]
        );
        assert!(!scale.contains(&Data::Text("44".into())));
        assert!(scale.contains(&Data::Integer(44)));
        assert!(scale.contains(&Data::None));
        assert!(scale.contains(&Data::Text("Test".into())));
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

    #[test]
    fn test_scale_ranged() {
        let pnts = vec![1, 2, 9, 10];
        let scale = Scale::new(pnts, ScaleKind::Integer);
        let rng = scale.ranged();

        assert_eq!(
            rng,
            vec![
                Data::Integer(1),
                Data::Integer(2),
                Data::Integer(3),
                Data::Integer(4),
                Data::Integer(5),
                Data::Integer(6),
                Data::Integer(7),
                Data::Integer(8),
                Data::Integer(9),
                Data::Integer(10),
            ]
        );

        let pnts: Vec<isize> = vec![1, 2, 9, 10];
        let scale = Scale::new(pnts, ScaleKind::Number);
        let rng = scale.ranged();

        assert_eq!(
            rng,
            vec![
                Data::Number(1),
                Data::Number(2),
                Data::Number(3),
                Data::Number(4),
                Data::Number(5),
                Data::Number(6),
                Data::Number(7),
                Data::Number(8),
                Data::Number(9),
                Data::Number(10),
            ]
        )
    }

    #[test]
    fn test_axis_points() {
        let pnts = vec![2.0, 5.0, -15.0, 0.0];
        let scale = Scale::from(pnts);

        assert_eq!(scale.length, 5);
        assert_eq!(
            scale.axis_points(true),
            AxisPoints::Numeric {
                positives: vec![0.0, 5.0].into_iter().map(From::from).collect(),
                negatives: vec![-15.0, -10.0, -5.0]
                    .into_iter()
                    .map(From::from)
                    .collect()
            }
        );

        let pnts = vec![5.0, 15.0, 0.0];
        let scale = Scale::from(pnts);

        assert_eq!(
            scale.axis_points(false),
            AxisPoints::Numeric {
                positives: vec![0.0, 5.0, 10.0, 15.0]
                    .into_iter()
                    .map(From::from)
                    .collect(),
                negatives: vec![]
            }
        );

        let pnts = vec![1, 2, 9, 10];
        let scale = Scale::from(pnts);

        assert_eq!(
            scale.axis_points(true),
            AxisPoints::Numeric {
                positives: (1..=10).map(From::from).collect(),
                negatives: vec![]
            }
        );

        let pnts = vec![1, 2, -9, 10];
        let scale = Scale::from(pnts);

        assert_eq!(
            scale.axis_points(true),
            AxisPoints::Numeric {
                positives: (0..=10).map(From::from).collect(),
                negatives: (-9..=-1).map(From::from).collect(),
            }
        );

        let pnts = vec![-1, -2, -9, -10];
        let scale = Scale::from(pnts);

        assert_eq!(
            scale.axis_points(true),
            AxisPoints::Numeric {
                positives: vec![],
                negatives: (-10..=-1).map(From::from).collect(),
            }
        );

        let pnts: Vec<isize> = vec![1, 2, 9, 10];
        let scale = Scale::from(pnts);

        assert_eq!(
            scale.axis_points(false),
            AxisPoints::Numeric {
                positives: vec![
                    Data::Number(1),
                    Data::Number(4),
                    Data::Number(7),
                    Data::Number(10),
                ],
                negatives: vec![]
            }
        );

        let pnts: Vec<isize> = vec![1, 2, -9, 10];
        let scale = Scale::from(pnts);

        assert_eq!(scale.length, 5);
        assert_eq!(
            scale.axis_points(false),
            AxisPoints::Numeric {
                positives: vec![Data::Number(1), Data::Number(6), Data::Number(11),],
                negatives: vec![Data::Number(-9), Data::Number(-4),],
            }
        );

        let pnts: Vec<isize> = vec![-1, -2, -9, -10];
        let scale = Scale::from(pnts);

        assert_eq!(
            scale.axis_points(false),
            AxisPoints::Numeric {
                positives: vec![],
                negatives: vec![
                    Data::Number(-10),
                    Data::Number(-7),
                    Data::Number(-4),
                    Data::Number(-1),
                ],
            }
        );
    }
}

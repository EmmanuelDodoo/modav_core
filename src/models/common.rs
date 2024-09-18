use crate::repr::Data;
use std::fmt::Debug;

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

#[derive(Debug, Clone, PartialEq)]
pub enum Scale<T>
where
    T: Clone + Debug,
{
    // Range(Range<T>),
    List(Vec<T>),
}

impl<T> Scale<T>
where
    T: Clone + Debug,
{
    pub fn points(&self) -> Vec<T> {
        match self {
            Self::List(lst) => lst.clone(),
        }
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

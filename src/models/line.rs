// Line graph

#[derive(Debug, Clone, PartialEq)]
pub struct Point<X, Y> {
    pub x: X,
    pub y: Y,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Line<X, Y> {
    pub points: Vec<Point<X, Y>>,
    pub label: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LineGraph<X, Y> {
    pub lines: Vec<Line<X, Y>>,
    pub x_label: String,
    pub y_label: String,
}

impl<X, Y> Point<X, Y> {
    pub fn new(x: X, y: Y) -> Self {
        Self { x, y }
    }
}

impl<X, Y> Line<X, Y> {
    pub fn new(points: Vec<(X, Y)>, label: Option<String>) -> Self {
        let points = points.into_iter().map(|(x, y)| Point::new(x, y));
        Self {
            points: points.collect(),
            label,
        }
    }

    pub fn from_points(points: Vec<Point<X, Y>>, label: Option<String>) -> Self {
        Self { points, label }
    }
}

impl<X, Y> LineGraph<X, Y> {
    pub fn new(lines: Vec<Line<X, Y>>, x_label: Option<String>, y_label: Option<String>) -> Self {
        let x = match x_label {
            Some(label) => label,
            None => String::new(),
        };

        let y = match y_label {
            Some(label) => label,
            None => String::new(),
        };

        Self {
            lines,
            x_label: x,
            y_label: y,
        }
    }
}

#[cfg(test)]
mod line_tests {
    use super::*;

    fn create_point<X, Y>(x: X, y: Y) -> Point<X, Y> {
        Point::new(x, y)
    }

    fn create_line_from_points(xs: Vec<&str>, label: Option<String>) -> Line<&str, usize> {
        let points: Vec<Point<&str, usize>> = xs
            .into_iter()
            .enumerate()
            .map(|(i, x)| create_point(x, i))
            .collect();

        Line::from_points(points, label)
    }

    fn create_line_from_new(xs: Vec<(&str, usize)>, label: Option<String>) -> Line<&str, usize> {
        Line::new(xs, label)
    }

    fn create_graph<'a>() -> LineGraph<&'a str, usize> {
        let p1 = vec!["one", "two", "three", "four", "five"];
        let p2: Vec<(&str, usize)> = vec![
            ("eins", 10),
            ("zwei", 20),
            ("drei", 30),
            ("vier", 4),
            ("fünf", 5),
        ];

        let pnt1 = create_line_from_new(p2, Some("Deutsch".into()));
        let pnt2 = create_line_from_points(p1, Some("English".into()));

        LineGraph {
            lines: vec![pnt1, pnt2],
            x_label: "Language".into(),
            y_label: "Number".into(),
        }
    }

    #[test]
    fn test_line_point() {
        let p1 = create_point(2, 3);
        assert_eq!(p1.x, 2);
        assert_eq!(p1.y, 3);

        let p2 = create_point(-4, 0);
        assert_eq!(p2.x, -4);
        assert_eq!(p2.y, 0);

        let p3 = create_point("Something", "else");
        assert_eq!(p3.x, "Something");
        assert_eq!(p3.y, "else");

        let p4 = create_point(String::from("tired"), 0.50);
        assert_eq!(p4.x, "tired");
        assert_eq!(p4.y, 0.50);
    }

    #[test]
    fn test_line_line() {
        let pts = vec!["one", "two", "three"];
        let line = create_line_from_points(pts, Some("Line 1".into()));

        assert_eq!(line.label, Some(String::from("Line 1")));
        let temp: Vec<&str> = line.points.iter().fold(vec![], |acc, curr| {
            let mut acc = acc.clone();
            acc.push(curr.x);
            acc
        });
        assert_eq!(vec!["one", "two", "three"], temp)
    }

    #[test]
    fn test_line_graph() {
        let graph = create_graph();

        assert_eq!(graph.x_label, String::from("Language"));
        assert_eq!(graph.y_label, String::from("Number"));

        graph
            .lines
            .iter()
            .for_each(|ln| assert_eq!(ln.points.len(), 5))
    }
}

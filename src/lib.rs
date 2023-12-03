mod custom_csv {
    use std::{error::Error, ffi::OsString, vec};

    #[derive(Debug, Clone, Default, PartialEq)]
    pub enum Data {
        Text(String),
        Integer(i32),
        Float(f32),
        Number(isize),
        Boolean(bool),
        #[default]
        None,
    }

    impl From<bool> for Data {
        fn from(value: bool) -> Self {
            Data::Boolean(value)
        }
    }

    impl From<String> for Data {
        fn from(value: String) -> Self {
            if value.is_empty() {
                return Data::None;
            }
            Data::Text(value)
        }
    }

    impl From<i32> for Data {
        fn from(value: i32) -> Self {
            Data::Integer(value)
        }
    }

    impl From<f32> for Data {
        fn from(value: f32) -> Self {
            Data::Float(value)
        }
    }

    impl From<isize> for Data {
        fn from(value: isize) -> Self {
            Data::Number(value)
        }
    }

    #[derive(Debug, Clone)]
    pub struct Cell {
        id: i32,
        data: Data,
    }

    #[derive(Debug, Clone)]
    pub struct Row {
        id: i32,
        cells: Vec<Cell>,
        primary: i32,
        id_counter: i32,
    }

    #[derive(Debug, Clone)]
    pub struct Sheet {
        rows: Vec<Row>,
        header: Vec<String>,
        id_counter: i32,
    }

    impl Cell {
        pub fn new(id: i32, data: Data) -> Self {
            Cell { id, data }
        }

        fn get_data(&self) -> Data {
            self.data.clone()
        }
    }

    impl Row {
        pub fn new(record: csv::StringRecord, id: i32, primary_index: i32) -> Self {
            let mut counter: i32 = 0;
            let cells: Vec<Cell> = {
                let mut cells = vec![];

                record.iter().for_each(|val| {
                    let data: Data = val.to_string().into();
                    let cell = Cell::new(counter, data);
                    cells.push(cell);
                    counter += 1;
                });
                cells
            };

            Row {
                id,
                cells,
                primary: primary_index,
                id_counter: counter,
            }
        }
    }

    impl Sheet {
        /// Create a new sheet given the path to a csv file
        pub fn new(
            path: OsString,
            with_header: bool,
            primary: i32,
        ) -> Result<Self, Box<dyn Error>> {
            let mut counter: i32 = 0;
            let mut rdr = csv::ReaderBuilder::new()
                .has_headers(with_header)
                .from_path(path)?;

            let rows: Vec<Row> = {
                let mut rows = vec![];

                for record in rdr.records() {
                    let record = record?;
                    let row = Row::new(record, counter, primary);
                    rows.push(row);
                    counter += 1;
                }
                rows
            };

            let header: Vec<String> = if with_header {
                let hr = rdr.headers()?.clone();
                hr.iter().map(|x| x.to_string()).collect()
            } else {
                Vec::new()
            };

            Ok(Sheet {
                rows,
                header,
                id_counter: counter,
            })
        }

        pub fn get_row_by_index(&self, index: usize) -> Option<&Row> {
            self.rows.get(index)
        }

        pub fn get_row_by_id(&self, id: i32) -> Option<&Row> {
            self.rows.iter().find(|row| row.id == id)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::custom_csv::*;
    use super::*;
    use std::error::Error;
    use std::ffi::OsString;

    #[test]
    fn test_cell() {
        let tdata = String::from("Something");
        let tcell = Cell::new(0, tdata.into());
        assert_eq!(
            "Cell { id: 0, data: Text(\"Something\") }",
            format!("{:?}", tcell)
        );

        let ndata: isize = 333;
        let ncell = Cell::new(0, ndata.into());
        assert_eq!("Cell { id: 0, data: Number(333) }", format!("{:?}", ncell));

        let bdata = true;
        let bcell = Cell::new(0, bdata.into());
        assert_eq!(
            "Cell { id: 0, data: Boolean(true) }",
            format!("{:?}", bcell)
        );

        let idata = 32;
        let icell = Cell::new(0, idata.into());
        assert_eq!("Cell { id: 0, data: Integer(32) }", format!("{:?}", icell));

        let fdata = 33.2;
        let fcell = Cell::new(0, fdata.into());
        assert_eq!("Cell { id: 0, data: Float(33.2) }", format!("{:?}", fcell));

        let nodata = String::from("");
        let nocell = Cell::new(0, nodata.into());
        assert_eq!("Cell { id: 0, data: None }", format!("{:?}", nocell));
    }

    #[test]
    fn test_row() {
        let row = {
            let sr = csv::StringRecord::from(vec!["3", "2"]);
            Row::new(sr, 4, 1)
        };

        assert_eq!(
            "Row { id: 4, cells: [Cell { id: 0, data: Text(\"3\") }, Cell { id: 1, data: Text(\"2\") }], primary: 1, id_counter: 2 }",
            format!("{:?}", row)
        )
    }
}

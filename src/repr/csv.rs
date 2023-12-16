pub mod csv_repr {
    use super::utils::*;
    use std::{
        ffi::OsString,
        slice::{Iter, IterMut},
    };

    use csv::Trim;

    #[derive(Debug, Clone)]
    pub struct Cell {
        id: usize,
        data: Data,
    }

    #[derive(Debug, Clone)]
    pub struct Row {
        id: usize,
        cells: Vec<Cell>,
        primary: usize,
        id_counter: usize,
    }

    #[derive(Debug, Clone)]
    pub struct Sheet {
        rows: Vec<Row>,
        headers: Vec<ColumnHeader>,
        id_counter: usize,
        primary_key: usize,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct SheetBuilder {
        path: OsString,
        primary: usize,
        with_header: bool,
        trim: bool,
    }

    impl SheetBuilder {
        pub fn new(path: OsString) -> Self {
            Self {
                path,
                primary: 0,
                with_header: false,
                trim: false,
            }
        }

        pub fn primary(self, primary: usize) -> Self {
            let path = self.path.clone();
            let with_header = self.with_header.clone();
            let trim = self.trim.clone();

            Self {
                path,
                with_header,
                primary,
                trim,
            }
        }

        pub fn header(self, header: bool) -> Self {
            Self {
                path: self.path.clone(),
                with_header: header,
                trim: self.trim,
                primary: self.primary,
            }
        }

        pub fn trim(self, trim: bool) -> Self {
            Self {
                path: self.path.clone(),
                with_header: self.with_header,
                trim,
                primary: self.primary,
            }
        }

        pub fn build(self) -> Result<Sheet, CSVError> {
            Sheet::new(
                self.path,
                self.primary,
                self.trim,
                HeaderStrategy::NoHeaders,
            )
        }
    }

    impl Cell {
        pub fn new(id: usize, data: Data) -> Self {
            Cell { id, data }
        }

        pub fn get_data(&self) -> Data {
            self.data.clone()
        }

        pub fn get_data_mut(&mut self) -> &mut Data {
            &mut self.data
        }

        /// Modifies the data in this cell
        pub fn set_data(&mut self, new_data: Data) {
            self.data = new_data;
        }
    }

    impl Row {
        pub fn new(record: csv::StringRecord, id: usize, primary_index: usize) -> Self {
            let mut counter: usize = 0;
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

        fn is_key_valid(&self, key: usize) -> bool {
            self.cells.len() > key
        }

        fn is_primary_key_valid(&self) -> bool {
            self.is_key_valid(self.primary)
        }

        pub fn set_primary_key(&mut self, new_primary: usize) -> Result<(), CSVError> {
            if new_primary < self.cells.len() {
                self.primary = new_primary;
                Ok(())
            } else {
                Err(CSVError::InvalidPrimaryKey)
            }
        }

        pub fn iter_cells(&self) -> Iter<'_, Cell> {
            self.cells.iter()
        }

        pub fn iter_cells_mut(&mut self) -> IterMut<'_, Cell> {
            self.cells.iter_mut()
        }

        pub fn get_primary_key(&self) -> usize {
            self.primary
        }

        pub fn get_primary_cell(&self) -> Option<&Cell> {
            self.cells.get(self.primary)
        }
    }

    impl Sheet {
        /// Returns a new T vector with length equal to given length
        /// A default T is used as padding. Any extras are trimmed
        fn balance_vector<T: Clone + Default>(lst: Vec<T>, size: usize) -> Vec<T> {
            let len = lst.len();
            if len == size {
                return lst;
            } else if len < size {
                let mut cln = lst.clone();
                let mut pad = vec![T::default(); size - len];

                cln.append(&mut pad);

                return cln;
            } else {
                let mut cln = lst.clone();
                cln.truncate(size);
                return cln;
            }
        }

        /// Create a new sheet given the path to a csv file
        fn new(
            path: OsString,
            primary: usize,
            trim: bool,
            strategy: HeaderStrategy,
        ) -> Result<Self, CSVError> {
            let mut counter: usize = 0;
            let mut longest_row = 0;

            let has_headers = match strategy {
                HeaderStrategy::ReadLabels(_) => true,
                _ => false,
            };

            let trim = {
                if trim {
                    Trim::All
                } else {
                    Trim::None
                }
            };
            let mut rdr = csv::ReaderBuilder::new()
                .has_headers(has_headers)
                .trim(trim)
                .from_path(path)?;

            let rows: Vec<Row> = {
                let mut rows = vec![];

                for record in rdr.records() {
                    let record = record?;
                    let row = Row::new(record, counter, primary);
                    if row.id_counter > longest_row {
                        longest_row = row.id_counter;
                    }
                    rows.push(row);
                    counter += 1;
                }
                rows
            };

            let headers = match strategy {
                HeaderStrategy::Provided(ch) => Sheet::balance_vector(ch, longest_row),
                HeaderStrategy::NoHeaders => {
                    Sheet::balance_vector(Vec::<ColumnHeader>::new(), longest_row)
                }
                HeaderStrategy::ReadLabels(ct) => {
                    let labels: Vec<String> = rdr
                        .headers()?
                        .clone()
                        .into_iter()
                        .map(|curr| curr.to_string())
                        .collect();
                    let labels = Sheet::balance_vector(labels, longest_row);
                    let ct = Sheet::balance_vector(ct, longest_row);

                    labels
                        .into_iter()
                        .zip(ct.into_iter())
                        .map(|(l, ct)| ColumnHeader::new(l, ct))
                        .collect()
                }
            };

            let sh = Sheet {
                rows,
                headers,
                id_counter: counter,
                primary_key: primary,
            };

            if Sheet::is_primary_valid(&sh) {
                Ok(sh)
            } else {
                Err(CSVError::InvalidPrimaryKey)
            }
        }

        pub fn get_row_by_index(&self, index: usize) -> Option<&Row> {
            self.rows.get(index)
        }

        pub fn get_row_by_id(&self, id: usize) -> Option<&Row> {
            self.rows.iter().find(|row| row.id == id)
        }

        fn is_primary_valid(sh: &Sheet) -> bool {
            sh.rows
                .iter()
                .fold(true, |acc, curr| acc && curr.is_primary_key_valid())
                && sh.headers.len() > sh.primary_key
        }

        fn set_primary_key(&mut self, new_key: usize) -> Result<(), CSVError> {
            if self
                .rows
                .iter()
                .fold(true, |acc, curr| acc && curr.is_key_valid(new_key))
            {
                self.primary_key = new_key;
                self.rows
                    .iter_mut()
                    .for_each(|row| row.set_primary_key(new_key).unwrap());
                return Ok(());
            }
            Err(CSVError::InvalidPrimaryKey)
        }

        pub fn get_primary_key(&self) -> usize {
            self.primary_key
        }

        pub fn iter_rows(&self) -> Iter<'_, Row> {
            self.rows.iter()
        }

        pub fn iter_rows_mut(&mut self) -> IterMut<'_, Row> {
            self.rows.iter_mut()
        }

        pub fn get_headers(&self) -> &Vec<ColumnHeader> {
            &self.headers
        }
    }
}

pub mod utils {
    use std::{
        cmp::{self, Ordering},
        default,
        error::Error,
        fmt,
    };

    #[derive(Debug)]
    pub enum CSVError {
        InvalidPrimaryKey,
        CSVModuleError(csv::Error),
    }

    impl From<csv::Error> for CSVError {
        fn from(value: csv::Error) -> Self {
            CSVError::CSVModuleError(value)
        }
    }

    impl fmt::Display for CSVError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                CSVError::CSVModuleError(e) => e.fmt(f),

                CSVError::InvalidPrimaryKey => {
                    write!(f, "Primary Key is invalid. It might be out of range")
                }
            }
        }
    }

    impl Error for CSVError {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            match self {
                CSVError::CSVModuleError(e) => e.source(),
                CSVError::InvalidPrimaryKey => None,
            }
        }
    }

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

    impl cmp::PartialOrd for Data {
        fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
            match (self, other) {
                (Data::Text(x), Data::Text(y)) => x.partial_cmp(y),
                (Data::Text(_), _) => Some(Ordering::Greater),
                (Data::Number(x), Data::Number(y)) => x.partial_cmp(y),
                (Data::Number(_), Data::Text(_)) => Some(Ordering::Less),
                (Data::Number(_), _) => Some(Ordering::Greater),
                (Data::Float(x), Data::Float(y)) => x.partial_cmp(y),
                (Data::Float(_), Data::Text(_)) => Some(Ordering::Less),
                (Data::Float(_), Data::Number(_)) => Some(Ordering::Less),
                (Data::Float(_), _) => Some(Ordering::Greater),
                (Data::Integer(x), Data::Integer(y)) => x.partial_cmp(y),
                (Data::Integer(_), Data::Text(_)) => Some(Ordering::Less),
                (Data::Integer(_), Data::Number(_)) => Some(Ordering::Less),
                (Data::Integer(_), Data::Float(_)) => Some(Ordering::Less),
                (Data::Integer(_), _) => Some(Ordering::Greater),
                (Data::Boolean(x), Data::Boolean(y)) => x.partial_cmp(y),
                (Data::Boolean(_), Data::Text(_)) => Some(Ordering::Less),
                (Data::Boolean(_), Data::Number(_)) => Some(Ordering::Less),
                (Data::Boolean(_), Data::Float(_)) => Some(Ordering::Less),
                (Data::Boolean(_), Data::Integer(_)) => Some(Ordering::Less),
                (Data::Boolean(_), _) => Some(Ordering::Greater),
                (Data::None, Data::None) => Some(Ordering::Equal),
                (Data::None, _) => Some(Ordering::Less),
            }
        }
    }

    impl fmt::Display for Data {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::Text(t) => write!(f, "{}", t),
                Self::Integer(i) => write!(f, "{}", i),
                Self::Float(fl) => write!(f, "{}", fl),
                Self::Boolean(b) => write!(f, "{}", b),
                Self::Number(n) => write!(f, "{}", n),
                Self::None => write!(f, "<None>"),
            }
        }
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

            if let Ok(parsed_i32) = value.parse::<i32>() {
                return Data::Integer(parsed_i32);
            };

            if let Ok(parsed_bool) = value.parse::<bool>() {
                return Data::Boolean(parsed_bool);
            };

            if let Ok(parsed_float) = value.parse::<f32>() {
                return Data::Float(parsed_float);
            }

            if let Ok(parsed_num) = value.parse::<isize>() {
                return Data::Number(parsed_num);
            };

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

    #[derive(Debug, Clone, Default, PartialEq)]
    pub enum ColumnType {
        Text,
        Integer,
        Number,
        Float,
        Bool,
        #[default]
        None,
    }

    impl From<Data> for ColumnType {
        fn from(value: Data) -> Self {
            match value {
                Data::Text(_) => Self::Text,
                Data::Float(_) => Self::Float,
                Data::Number(_) => Self::Number,
                Data::Integer(_) => Self::Integer,
                Data::Boolean(_) => Self::Bool,
                Data::None => Self::None,
            }
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct ColumnHeader {
        label: String,
        kind: ColumnType,
    }

    impl ColumnHeader {
        pub fn new(label: String, kind: ColumnType) -> Self {
            Self { label, kind }
        }

        pub fn set_label(&mut self, label: String) {
            self.label = label;
        }

        /// Returns true if data is equivalent to this column type.
        /// For flexibility reasons, ColumnType::None always returns true
        pub fn crosscheck_data(&self, data: Data) -> bool {
            let conv: ColumnType = data.into();
            match self.kind {
                ColumnType::None => true,
                _ => conv == self.kind,
            }
        }
    }

    impl default::Default for ColumnHeader {
        fn default() -> Self {
            Self {
                label: "".into(),
                kind: ColumnType::None,
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Default)]
    pub enum HeaderStrategy {
        #[default]
        NoHeaders,
        ReadLabels(Vec<ColumnType>),
        Provided(Vec<ColumnHeader>),
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use super::csv_repr::*;
    use super::utils::*;

    fn create_row() -> Row {
        let sr = csv::StringRecord::from(vec!["3", "2", "1"]);
        Row::new(sr, 4, 0)
    }

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
        let row = create_row();
        assert_eq!(
            "Row { id: 4, cells: [Cell { id: 0, data: Integer(3) }, Cell { id: 1, data: Integer(2) }, Cell { id: 2, data: Integer(1) }], primary: 0, id_counter: 3 }",
            format!("{:?}", row)
        )
    }

    #[test]
    fn test_iter_cells() {
        let row = create_row();

        assert_eq!(
            "Row { id: 4, cells: [Cell { id: 0, data: Integer(3) }, Cell { id: 1, data: Integer(2) }, Cell { id: 2, data: Integer(1) }], primary: 0, id_counter: 3 }",
            format!("{:?}", row)
        );

        let new_cells: Vec<Cell> = row
            .iter_cells()
            .map(|cell| {
                let prev = cell.get_data();
                let new = match prev {
                    Data::Integer(i) => Data::Integer(i + 100),
                    _ => Data::None,
                };
                Cell::new(0, new)
            })
            .collect();

        assert_eq!("[Cell { id: 0, data: Integer(103) }, Cell { id: 0, data: Integer(102) }, Cell { id: 0, data: Integer(101) }]", format!("{:?}", new_cells))
    }

    #[test]
    fn test_iter_cells_mut() {
        let mut row = create_row();

        assert_eq!(
            "Row { id: 4, cells: [Cell { id: 0, data: Integer(3) }, Cell { id: 1, data: Integer(2) }, Cell { id: 2, data: Integer(1) }], primary: 0, id_counter: 3 }",
            format!("{:?}", row)
        );

        row.iter_cells_mut().for_each(|cell| {
            if let Data::Integer(i) = cell.get_data_mut() {
                *i += 100;
            };
        });

        assert_eq!("Row { id: 4, cells: [Cell { id: 0, data: Integer(103) }, Cell { id: 1, data: Integer(102) }, Cell { id: 2, data: Integer(101) }], primary: 0, id_counter: 3 }", 
            format!("{:?}", row));

        row.iter_cells_mut()
            .for_each(|cell| cell.set_data(Data::None));

        assert_eq!(
            "Row { id: 4, cells: [Cell { id: 0, data: None }, Cell { id: 1, data: None }, Cell { id: 2, data: None }], primary: 0, id_counter: 3 }",
            format!("{:?}", row)
        )
    }

    #[test]
    fn test_row_set_primary_key() {
        let mut row = create_row();

        assert_eq!(0, row.get_primary_key());

        if let Err(_) = row.set_primary_key(1) {
            panic!("Something went wrong which shouldn't")
        };
        assert_eq!(1, row.get_primary_key());

        if let Ok(_) = row.set_primary_key(3) {
            panic!("Something went wrong whcih shouldn't have")
        }

        assert_eq!(1, row.get_primary_key())
    }

    #[test]
    fn test_get_primary_cell() {
        let mut row = create_row();

        let cell = row.get_primary_cell();

        assert_eq!(
            "Some(Cell { id: 0, data: Integer(3) })",
            format!("{:?}", cell)
        );

        if let Err(_) = row.set_primary_key(2) {
            panic!("Something which shouldn't happen, happened")
        };

        let cell = row.get_primary_cell();

        assert_eq!(
            "Some(Cell { id: 2, data: Integer(1) })",
            format!("{:?}", cell)
        )
    }

    #[test]
    fn test_sheet_builder() {
        let path: OsString = "./dummies/csv/air.csv".into();
        let res = SheetBuilder::new(path)
            .header(true)
            .trim(true)
            .primary(0)
            .build();

        match res {
            Ok(sht) => {}
            Err(e) => panic!("{}", e),
        }
    }
}

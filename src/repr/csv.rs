pub mod csv_repr {
    use super::utils::*;
    use crate::models::line::{Line, LineGraph, Point, Scale};
    use std::{
        collections::HashSet,
        ffi::OsString,
        slice::{Iter, IterMut},
    };

    use csv::Trim;

    #[derive(Debug, Clone, PartialEq)]
    pub struct Cell {
        id: usize,
        data: Data,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct Row {
        id: usize,
        cells: Vec<Cell>,
        primary: usize,
        id_counter: usize,
    }

    #[derive(Debug, Clone, PartialEq)]
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
        trim: bool,
        header_strategy: HeaderStrategy,
        flexible: bool,
    }

    impl SheetBuilder {
        pub fn new(path: OsString) -> Self {
            Self {
                path,
                primary: 0,
                trim: false,
                header_strategy: HeaderStrategy::NoHeaders,
                flexible: false,
            }
        }

        pub fn primary(self, primary: usize) -> Self {
            Self { primary, ..self }
        }

        pub fn trim(self, trim: bool) -> Self {
            Self { trim, ..self }
        }

        pub fn flexible(self, flexible: bool) -> Self {
            Self { flexible, ..self }
        }

        pub fn header_strategy(self, strategy: HeaderStrategy) -> Self {
            Self {
                header_strategy: strategy,
                ..self
            }
        }

        pub fn build(self) -> Result<Sheet, CSVError> {
            Sheet::new(
                self.path,
                self.primary,
                self.header_strategy,
                self.trim,
                self.flexible,
            )
        }
    }

    impl Cell {
        pub fn new(id: usize, data: Data) -> Self {
            Cell { id, data }
        }

        pub fn get_data(&self) -> &Data {
            &self.data
        }

        pub fn get_data_mut(&mut self) -> &mut Data {
            &mut self.data
        }

        /// Modifies the data in this cell
        pub fn set_data(&mut self, new_data: Data) {
            self.data = new_data;
        }

        fn validate_type(&self, kind: &ColumnType) -> Result<(), CSVError> {
            if kind.crosscheck_data(self.data.clone()) {
                return Ok(());
            } else {
                Err(CSVError::InvalidColumnType(format!(
                    "Expected {:?} type but had {:?} type for cell with id: {}",
                    kind, self.data, self.id
                )))
            }
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

        pub fn is_primary_key_valid(&self) -> Result<(), CSVError> {
            if !self.is_key_valid(self.primary) {
                return Err(CSVError::InvalidPrimaryKey(format!(
                    "Primary key is invalid for row with id: {}",
                    self.id
                )));
            };
            Ok(())
        }

        fn validate_all_cols(&self, headers: &Vec<ColumnHeader>) -> Result<(), CSVError> {
            if self.cells.len() != headers.len() {
                return Err(CSVError::InvalidColumnLength(format!(
                    "Row with id, {}, has unbalanced cells.",
                    self.id
                )));
            }

            self.iter_cells()
                .enumerate()
                .fold(Ok(()), |acc, curr| match acc {
                    Err(e) => Err(e),
                    Ok(()) => {
                        let header = headers.get(curr.0).unwrap();
                        if header.crosscheck_data(curr.1.clone().data) {
                            return Ok(());
                        } else {
                            return Err(CSVError::InvalidColumnType(
                                    format!("Expected {:?} type but had {:?} type for cell id: {}, in row id: {}. ",
                                        header.kind, curr.1.data, curr.1.id, self.id )))
                        }
                    }
                })
        }

        fn validate_col(&self, header: &ColumnHeader, col: usize) -> Result<(), CSVError> {
            let cell = self.cells.get(col);
            match cell {
                None => Err(CSVError::InvalidColumnLength(
                    "Tried to validate out of bounds column".into(),
                )),
                Some(cl) => {
                    if header.crosscheck_data(cl.data.clone()) {
                        Ok(())
                    } else {
                        Err(CSVError::InvalidColumnType(format!(
                            "Expected cell of {:?} type, but had {:?} type in cell id {} in row id {}",
                            header.kind, cl.data, cl.id, self.id
                        )))
                    }
                }
            }
        }

        pub fn set_primary_key(&mut self, new_primary: usize) -> Result<(), CSVError> {
            if new_primary < self.cells.len() {
                self.primary = new_primary;
                Ok(())
            } else {
                Err(CSVError::InvalidPrimaryKey(
                    "Tried to set primary key to invalid value".into(),
                ))
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

        pub fn get_cell_by_id(&self, id: usize) -> Option<&Cell> {
            self.cells.iter().find(|cl| cl.id == id)
        }

        pub fn get_cell_by_index(&self, index: usize) -> Option<&Cell> {
            self.cells.get(index)
        }

        /// Fill the row with empty cells up to a given length
        fn balance_cells(&mut self, len: usize) {
            let ln = self.cells.len();

            if ln >= len {
                return;
            }

            for _ in 0..(len - ln) {
                let empty = Cell::new(self.id_counter, Data::None);
                self.cells.push(empty);
                self.id_counter += 1;
            }
        }

        ///  Returns a Line whose points have x values from the vector provided
        ///  and y values as the data in each corresponding cell in this row.
        ///
        ///  Intended for use in creating LineGraphs.
        ///
        ///  Any unpaired x or y values are ignored
        fn create_line(
            &self,
            label: &LineLabelStrategy,
            x_values: &Vec<String>,
            exclude: &HashSet<usize>,
        ) -> Line<String, Data> {
            let points: Vec<Point<String, Data>> = match label {
                LineLabelStrategy::FromCell(idx) => {
                    let points = x_values
                        .iter()
                        .zip(self.cells.iter())
                        .enumerate()
                        .filter(|(id, _)| id != idx && !exclude.contains(id))
                        .map(|(_, (x, cell))| Point::new(x.clone(), cell.data.clone()))
                        .collect();

                    points
                }

                _ => x_values
                    .iter()
                    .zip(self.cells.iter())
                    .enumerate()
                    .filter(|(id, _)| !exclude.contains(id))
                    .map(|(_, (x, cell))| Point::new(x.clone(), cell.data.clone()))
                    .collect(),
            };

            let lbl: Option<String> = match label {
                LineLabelStrategy::None => None,
                LineLabelStrategy::Provided(s) => Some(s.clone()),
                LineLabelStrategy::FromCell(idx) => {
                    if let Some(cell) = self.cells.get(idx.clone()) {
                        Some(cell.data.to_string())
                    } else {
                        None
                    }
                }
            };
            Line::from_points(points, lbl)
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
            strategy: HeaderStrategy,
            trim: bool,
            flexible: bool,
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
                .flexible(flexible)
                .from_path(path)?;

            let mut rows: Vec<Row> = {
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

            if flexible {
                rows.iter_mut()
                    .for_each(|row| row.balance_cells(longest_row));
            }

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

            sh.validate()?;

            Ok(sh)
        }

        pub fn get_row_by_index(&self, index: usize) -> Option<&Row> {
            self.rows.get(index)
        }

        pub fn get_row_by_id(&self, id: usize) -> Option<&Row> {
            self.rows.iter().find(|row| row.id == id)
        }

        /// Could be expensive
        pub fn validate(&self) -> Result<(), CSVError> {
            // Validating could be expensive
            Self::is_primary_valid(self)?;
            Self::validate_all_cols(self)?;

            Ok(())
        }

        /// Checks if the type for each column cell is as expected
        fn validate_all_cols(sh: &Sheet) -> Result<(), CSVError> {
            let hrs = &sh.headers;
            sh.iter_rows().fold(Ok(()), |acc, curr| match acc {
                Err(e) => Err(e),
                Ok(()) => curr.validate_all_cols(hrs),
            })
        }

        pub fn validate_col(&self, col: usize) -> Result<(), CSVError> {
            let hdr = self
                .headers
                .get(col)
                .ok_or(CSVError::InvalidColumnLength(format!(
                    "Tried to access out of range column"
                )))?;

            self.iter_rows().fold(Ok(()), |acc, curr| match acc {
                Err(e) => Err(e),
                Ok(()) => curr.validate_col(hdr, col),
            })
        }

        fn is_primary_valid(sh: &Sheet) -> Result<(), CSVError> {
            let len = sh.headers.len();
            let pk = sh.primary_key;

            if (len == pk && pk != 0) || (len < pk) {
                return Err(CSVError::InvalidPrimaryKey(
                    "Primary key out of column range".into(),
                ));
            }

            sh.rows.iter().fold(Ok(()), |acc, curr| match acc {
                Err(e) => Err(e),
                Ok(()) => curr.is_primary_key_valid(),
            })
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
            Err(CSVError::InvalidPrimaryKey(
                "Tried setting primary key to invalid value".into(),
            ))
        }

        pub fn get_primary_key(&self) -> usize {
            self.primary_key
        }

        pub fn iter_rows(&self) -> Iter<'_, Row> {
            self.rows.iter()
        }

        /// Should probably call Sheet::validate after using this function
        pub fn iter_rows_mut(&mut self) -> IterMut<'_, Row> {
            self.rows.iter_mut()
        }

        pub fn get_headers(&self) -> &Vec<ColumnHeader> {
            &self.headers
        }

        pub fn sort_rows(&mut self, col: usize) -> Result<(), CSVError> {
            let ch = self
                .headers
                .get(col)
                .ok_or(CSVError::InvalidColumnLength("Column out of range".into()))?;

            match ch {
                ColumnHeader {
                    label: _,
                    kind: ColumnType::None,
                } => {
                    return Err(CSVError::InvalidColumnSort(
                        "Tried to sort by an unstructured column ".into(),
                    ))
                }
                _ => {}
            };

            self.validate_col(col)?;

            let asc = |x: &Row, y: &Row| {
                let d1 = &x.cells.get(col).unwrap().data;
                let d2 = &y.cells.get(col).unwrap().data;

                match (d1, d2) {
                    (Data::None, Data::None) => std::cmp::Ordering::Equal,
                    (Data::Text(s1), Data::Text(s2)) => s1.cmp(s2),
                    (Data::Float(f1), Data::Float(f2)) => f1.total_cmp(f2),
                    (Data::Number(n1), Data::Number(n2)) => n1.cmp(n2),
                    (Data::Integer(i1), Data::Integer(i2)) => i1.cmp(i2),
                    (Data::Boolean(b1), Data::Boolean(b2)) => b1.cmp(b2),
                    // Should never reach this case. Previous checks ensure that
                    _ => std::cmp::Ordering::Equal,
                }
            };

            self.rows.sort_by(asc);

            Ok(())
        }

        pub fn sort_rows_rev(&mut self, col: usize) -> Result<(), CSVError> {
            let ch = self
                .headers
                .get(col)
                .ok_or(CSVError::InvalidColumnLength("Column out of range".into()))?;

            match ch {
                ColumnHeader {
                    label: _,
                    kind: ColumnType::None,
                } => {
                    return Err(CSVError::InvalidColumnSort(
                        "Tried to sort by an unstructured column ".into(),
                    ))
                }
                _ => {}
            };

            self.validate_col(col)?;

            let desc = |x: &Row, y: &Row| {
                let d1 = &x.cells.get(col).unwrap().data;
                let d2 = &y.cells.get(col).unwrap().data;

                match (d1, d2) {
                    (Data::None, Data::None) => std::cmp::Ordering::Equal,
                    (Data::Text(s1), Data::Text(s2)) => s2.cmp(s1),
                    (Data::Float(f1), Data::Float(f2)) => f2.total_cmp(f1),
                    (Data::Number(n1), Data::Number(n2)) => n2.cmp(n1),
                    (Data::Integer(i1), Data::Integer(i2)) => i2.cmp(i1),
                    (Data::Boolean(b1), Data::Boolean(b2)) => b2.cmp(b1),
                    // Should never reach this case. Previous checks ensure that
                    _ => std::cmp::Ordering::Equal,
                }
            };

            self.rows.sort_by(desc);

            Ok(())
        }

        fn infer_col_kinds(sh: &mut Self, header_len: usize) {
            let mut is_first_iteration = true;
            let col_kinds: Vec<ColumnType> = sh
                .iter_rows()
                .map(|rw| {
                    rw.iter_cells()
                        .map(|cl| cl.get_data().clone().into())
                        .collect::<Vec<ColumnType>>()
                })
                .fold(vec![None; header_len], |acc, curr| {
                    acc.into_iter()
                        .zip(curr)
                        .map(|(ac, cr)| match ac {
                            None => {
                                is_first_iteration = false;
                                Some(cr)
                            }
                            Some(ac) => match (ac, cr) {
                                (ColumnType::None, x) => {
                                    if is_first_iteration {
                                        Some(x)
                                    } else {
                                        Some(ColumnType::None)
                                    }
                                }
                                (y, ColumnType::None) => Some(y),
                                (ac, cr) if ac == cr => Some(ac),
                                _ => Some(ColumnType::None),
                            },
                        })
                        .collect()
                })
                .into_iter()
                .map(|op| op.unwrap_or(ColumnType::default()))
                .collect();

            sh.headers.iter_mut().zip(col_kinds).for_each(|(hdr, knd)| {
                hdr.kind = knd;
            });
        }

        /// initial_header: The new label for the initial header, if any
        ///
        /// uniform_type: Whether every non-zeroth column has the same type.
        /// types are lost if false
        pub fn transpose(self: &Self, initial_header: Option<String>) -> Result<Self, CSVError> {
            Sheet::validate(&self)?;

            let width = self.headers.len();
            let depth = self.rows.len() + 1;

            let mut headers: Vec<ColumnHeader> = Vec::new();
            let mut rows: Vec<Vec<Cell>> = Vec::new();

            for idx in 0..width {
                let hr = match self.headers.get(idx) {
                    Some(hdr) => {
                        let mut h = hdr.clone();
                        h.kind = ColumnType::Text;
                        h
                    }
                    None => {
                        return Err(CSVError::TransposeError("Sheet has missing headers".into()))
                    }
                };

                if idx == 0 {
                    let hr = match &initial_header {
                        None => hr,
                        Some(lbl) => ColumnHeader::new(lbl.clone(), hr.kind),
                    };
                    let mut hrs = self
                        .iter_rows()
                        .fold(Vec::<ColumnHeader>::new(), |acc, curr| {
                            let cln = match curr.get_cell_by_index(0).unwrap() {
                                Cell {
                                    id: _,
                                    data: Data::None,
                                } => String::new(),
                                Cell { id: _, data: d } => d.to_string(),
                            };
                            let hdr = ColumnHeader::new(cln, ColumnType::None);
                            let mut acc = acc;
                            acc.push(hdr);
                            acc
                        });
                    headers.push(hr);
                    headers.append(&mut hrs);
                } else {
                    let first = Cell::new(0, hr.label.into());
                    let mut rw = vec![first];
                    let mut cls: Vec<Cell> = self
                        .iter_rows()
                        .enumerate()
                        .map(|(id, rw)| {
                            let id = id + 1;
                            match rw.get_cell_by_index(idx) {
                                Some(cl) => {
                                    let mut cl = cl.clone();
                                    cl.id = id;
                                    cl
                                }
                                None => Cell::new(id, Data::default()),
                            }
                        })
                        .collect();
                    rw.append(&mut cls);
                    rows.push(rw);
                }
            }

            let rows: Vec<Row> = rows
                .into_iter()
                .enumerate()
                .map(|(id, cells)| Row {
                    cells,
                    primary: 0,
                    id,
                    id_counter: depth,
                })
                .collect();

            let mut sh = Sheet {
                rows,
                headers,
                id_counter: width - 1,
                primary_key: 0,
            };

            Self::infer_col_kinds(&mut sh, depth);

            Self::validate(&sh)?;

            Ok(sh)
        }

        fn copy_col_data(&self, col: usize) -> Result<Vec<Data>, CSVError> {
            self.validate_col(col)?;

            let data: Vec<Data> = self
                .iter_rows()
                .map(|row| {
                    let cl = row.get_cell_by_index(col).unwrap();
                    cl.data.clone()
                })
                .collect();

            Ok(data)
        }

        fn grab_header(&self, col: usize) -> Result<&ColumnHeader, CSVError> {
            let hr = self.headers.get(col).ok_or(CSVError::InvalidColumnLength(
                "Tried accessing an out of bounds Header".into(),
            ))?;

            match hr.kind {
                ColumnType::None => {
                    return Err(CSVError::ConversionError(
                        "Cannot convert non uniform type column".into(),
                    ))
                }
                _ => Ok(hr),
            }
        }

        fn validate_to_line_graph(&self, label_strat: &LineLabelStrategy) -> Result<(), CSVError> {
            // None type Columns
            self.headers
                .iter()
                .fold(Ok(()), |acc, curr| match (acc, &curr.kind) {
                    (Err(e), _) => return Err(e),
                    (Ok(_), ColumnType::None) => {
                        return Err(CSVError::ConversionError(
                            "Cannot convert non uniform type column".into(),
                        ));
                    }
                    (Ok(_), _) => Ok(()),
                })?;

            let check_uniform_type = |acc: Result<ColumnType, CSVError>, ct: ColumnType| {
                if let Ok(acc) = acc {
                    match (&acc, &ct) {
                        (ColumnType::None, _) => Ok(ct),
                        (x, y) => {
                            if x == y {
                                return Ok(ct);
                            } else {
                                return Err(CSVError::ConversionError(
                                    "Cannot convert different column types".into(),
                                ));
                            }
                        }
                    }
                } else {
                    return acc;
                }
            };

            // Uniform type columns
            match label_strat {
                LineLabelStrategy::FromCell(idx) => {
                    if idx >= &self.headers.len() {
                        return Err(CSVError::ConversionError(
                            "Tried to assign invalid column as label".into(),
                        ));
                    }

                    self.headers
                        .iter()
                        .map(|hrd| &hrd.kind)
                        .enumerate()
                        .filter(|(ind, _)| ind != idx)
                        .fold(Ok(ColumnType::None), |acc, (_, ct)| {
                            check_uniform_type(acc, ct.clone())
                        })?;
                }

                _ => {
                    self.headers
                        .iter()
                        .map(|hdr| &hdr.kind)
                        .fold(Ok(ColumnType::None), |acc, ct| {
                            check_uniform_type(acc, ct.clone())
                        })?;
                }
            }

            Ok(())
        }

        /// Returns a new line graph created from this csv struct
        ///
        /// exclude_row: The positions of the rows to exclude in this transformation
        /// exclude_column: The positions of columns to exclude in the
        /// transformation
        pub fn to_line_graph(
            self: &Self,
            x_label: Option<String>,
            y_label: Option<String>,
            label_strat: LineLabelStrategy,
            exclude_row: HashSet<usize>,
            exclude_column: HashSet<usize>,
        ) -> Result<LineGraph<String, Data>, CSVError> {
            self.validate()?;
            self.validate_to_line_graph(&label_strat)?;

            let x_values: Vec<String> = {
                let values: Vec<String> = self
                    .headers
                    .iter()
                    .enumerate()
                    .map(|(_, hdr)| hdr.label.clone())
                    .collect();
                values
            };

            let lines: Vec<Line<String, Data>> = self
                .iter_rows()
                .enumerate()
                .filter(|(idx, _)| !exclude_row.contains(idx))
                .map(|(_, rw)| rw.create_line(&label_strat, &x_values, &exclude_column))
                .collect();

            let y_scale: Scale<Data> = {
                let mut values: Vec<Data> = lines
                    .iter()
                    .flat_map(|ln| ln.points.iter().map(|pnt| pnt.y.clone()))
                    .collect();
                values.sort();
                values.dedup();

                Scale::List(values)
            };

            let x_scale = {
                let mut values: Vec<String> = x_values
                    .into_iter()
                    .enumerate()
                    .filter(|(idx, _)| !exclude_column.contains(idx))
                    .map(|(_, lbl)| lbl)
                    .collect();
                values.sort();
                values.dedup();
                Scale::List(values)
            };

            let lg = LineGraph::new(lines, x_label, y_label, x_scale, y_scale)
                .map_err(CSVError::LineGraphError)?;

            Ok(lg)
        }
    }
}

pub mod utils {
    use crate::models::line::utils::LineGraphError;

    use std::{
        cmp::{self, Ordering},
        default,
        error::Error,
        fmt, hash,
    };

    #[derive(Debug)]
    pub enum CSVError {
        InvalidPrimaryKey(String),
        CSVModuleError(csv::Error),
        InvalidColumnType(String),
        InvalidColumnLength(String),
        InvalidColumnSort(String),
        ConversionError(String),
        LineGraphError(LineGraphError),
        TransposeError(String),
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
                CSVError::InvalidColumnLength(s) => {
                    write!(f, "Invalid Column Length: {}", s)
                }
                CSVError::InvalidPrimaryKey(s) => {
                    write!(f, "Primary Key is invalid. {}", s)
                }
                CSVError::InvalidColumnType(s) => write!(f, "Invalid Column type: {}", s),
                CSVError::InvalidColumnSort(s) => write!(f, "Invalid Column Sort: {}", s),
                CSVError::ConversionError(s) => {
                    write!(f, "Line Graph Conversion Error: {}", s)
                }
                CSVError::LineGraphError(lg) => lg.fmt(f),
                CSVError::TransposeError(s) => write!(f, "Transposing Error: {}", s),
            }
        }
    }

    impl Error for CSVError {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            match self {
                CSVError::CSVModuleError(e) => e.source(),
                CSVError::InvalidColumnLength(_) => None,
                CSVError::InvalidPrimaryKey(_) => None,
                CSVError::InvalidColumnType(_) => None,
                CSVError::InvalidColumnSort(_) => None,
                CSVError::ConversionError(_) => None,
                CSVError::LineGraphError(_) => None,
                CSVError::TransposeError(_) => None,
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

    impl Eq for Data {}

    impl cmp::Ord for Data {
        fn cmp(&self, other: &Self) -> Ordering {
            if let Some(ord) = self.partial_cmp(other) {
                return ord;
            } else {
                // Special case for NaN. Should only happend when both are f32
                match self {
                    Data::Float(f) => {
                        if f.is_nan() {
                            return Ordering::Less;
                        } else {
                            return Ordering::Greater;
                        }
                    }

                    _ => panic!("Partial_cmp for Data returned None. Only floats should do so"),
                }
            }
        }
    }

    impl hash::Hash for Data {
        fn hash<H: hash::Hasher>(&self, state: &mut H) {
            match self {
                Data::Text(t) => t.hash(state),
                Data::Integer(i) => i.hash(state),
                Data::Number(n) => n.hash(state),
                Data::Boolean(b) => b.hash(state),
                Data::Float(f) => format!("{}", f).hash(state),
                Data::None => "<None>".hash(state),
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

            if value == Data::None.to_string() {
                return Data::None;
            }
            Data::Text(value)
        }
    }

    impl From<&str> for Data {
        fn from(value: &str) -> Self {
            value.to_string().into()
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

    impl From<Data> for String {
        fn from(value: Data) -> Self {
            value.to_string()
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

    impl ColumnType {
        /// Returns true if data is equivalent to this column type.
        /// For flexibility reasons, ColumnType::None always returns true
        pub fn crosscheck_data(&self, data: Data) -> bool {
            if let Data::None = data {
                return true;
            };
            let conv: ColumnType = data.into();
            match self {
                ColumnType::None => true,
                _ => &conv == self,
            }
        }
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
        pub label: String,
        pub kind: ColumnType,
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
            if let Data::None = data {
                return true;
            };
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

    #[derive(Debug, Clone, PartialEq, Default)]
    pub enum LineLabelStrategy {
        FromCell(usize),
        Provided(String),
        #[default]
        None,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::ffi::OsString;
    use std::usize;

    use super::csv_repr::*;
    use super::utils::*;

    fn create_row() -> Row {
        let sr = csv::StringRecord::from(vec!["3", "2", "1"]);
        Row::new(sr, 4, 0)
    }

    fn create_air_csv() -> Result<Sheet, CSVError> {
        let path: OsString = "./dummies/csv/air.csv".into();

        let ct = vec![
            ColumnType::Text,
            ColumnType::Integer,
            ColumnType::Integer,
            ColumnType::Integer,
        ];

        SheetBuilder::new(path.clone())
            .trim(true)
            .primary(0)
            .header_strategy(HeaderStrategy::ReadLabels(ct))
            .build()
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
        let path2: OsString = "./dummies/csv/air2.csv".into();

        let ct = vec![
            ColumnType::Text,
            ColumnType::Integer,
            ColumnType::Integer,
            ColumnType::Integer,
        ];

        let res = SheetBuilder::new(path.clone())
            .trim(true)
            .primary(0)
            .header_strategy(HeaderStrategy::ReadLabels(ct))
            .build();

        match res {
            Ok(sht) => {
                let hrs = sht.get_headers();
                match hrs.get(0) {
                    None => panic!("No headers when there should have been some"),
                    Some(hr) => {
                        assert_eq!(
                            "ColumnHeader { label: \"Month\", kind: Text }",
                            format!("{:?}", hr)
                        )
                    }
                }

                match hrs.get(2) {
                    None => panic!("Missing third header"),
                    Some(hr) => assert_eq!(
                        "ColumnHeader { label: \"1959\", kind: Integer }",
                        format!("{:?}", hr)
                    ),
                }
            }
            Err(e) => panic!("{}", e),
        };

        let res = SheetBuilder::new(path.clone())
            .trim(true)
            .primary(0)
            .build();

        match res {
            Err(e) => panic!("{}", e),
            Ok(sht) => match sht.get_headers().get(1) {
                None => panic!("No second header found"),
                Some(hr) => assert_eq!(
                    "ColumnHeader { label: \"\", kind: None }",
                    format!("{:?}", hr)
                ),
            },
        }

        let lbl = vec!["Month", "1958", "1959"];
        let ct = vec![ColumnType::Text, ColumnType::Integer, ColumnType::Integer];
        let chs: Vec<ColumnHeader> = lbl
            .into_iter()
            .zip(ct.into_iter())
            .map(|(lb, ty)| ColumnHeader::new(lb.into(), ty))
            .collect();

        let res = SheetBuilder::new(path2)
            .trim(true)
            .header_strategy(HeaderStrategy::Provided(chs))
            .build();

        match res {
            Err(e) => panic!("{}", e),
            Ok(sht) => {
                match sht.get_headers().get(0) {
                    None => panic!("No Header when there should be one"),
                    Some(hr) => {
                        assert_eq!(
                            "ColumnHeader { label: \"Month\", kind: Text }",
                            format!("{:?}", hr)
                        )
                    }
                };

                match sht.get_headers().get(3) {
                    None => panic!("Missing padded header"),
                    Some(hr) => {
                        assert_eq!(
                            "ColumnHeader { label: \"\", kind: None }",
                            format!("{:?}", hr)
                        )
                    }
                };
            }
        }
    }

    #[test]
    #[should_panic]
    fn test_col_validation() {
        let path1: OsString = "./dummies/csv/invalid1.csv".into();

        let ct = vec![
            ColumnType::Text,
            ColumnType::Integer,
            ColumnType::Integer,
            ColumnType::Integer,
        ];

        let res = SheetBuilder::new(path1)
            .trim(true)
            .header_strategy(HeaderStrategy::ReadLabels(ct))
            .build();

        match res {
            Err(e) => panic!("{}", e),
            Ok(_) => (),
        }
    }

    #[test]
    fn test_col_validation2() {
        let path: OsString = "./dummies/csv/invalid2.csv".into();

        let ct = vec![
            ColumnType::Text,
            ColumnType::Integer,
            ColumnType::None,
            ColumnType::Integer,
        ];

        if let Err(e) = SheetBuilder::new(path)
            .trim(true)
            .header_strategy(HeaderStrategy::ReadLabels(ct))
            .build()
        {
            panic!("{}", e)
        };
    }

    #[test]
    fn test_empty_csv() {
        let path: OsString = "./dummies/csv/empty.csv".into();

        if let Err(e) = SheetBuilder::new(path)
            .header_strategy(HeaderStrategy::NoHeaders)
            .trim(true)
            .build()
        {
            panic!("{}", e)
        }
    }

    #[test]
    fn testing_empty_field() {
        let path: OsString = "./dummies/csv/address.csv".into();

        let res = SheetBuilder::new(path)
            .header_strategy(HeaderStrategy::NoHeaders)
            .trim(true)
            .build();

        match res {
            Err(e) => panic!("{}", e),
            Ok(sht) => sht.iter_rows().for_each(|row| {
                // println!("{:?}", row);
                // println!("")
            }),
        }
    }

    #[test]
    fn testing_flexible() {
        let path: OsString = "./dummies/csv/flexible.csv".into();

        let res = SheetBuilder::new(path)
            .trim(true)
            .header_strategy(HeaderStrategy::NoHeaders)
            .flexible(true)
            .build();

        match res {
            Err(e) => panic!("{}", e),
            Ok(sh) => sh.iter_rows().for_each(|row| {
                // println!("{:?}", row);
                // println!("")
            }),
        }
    }

    #[test]
    fn test_sort() {
        let path: OsString = "./dummies/csv/air.csv".into();

        let ct = vec![
            ColumnType::Text,
            ColumnType::Integer,
            ColumnType::Integer,
            ColumnType::Integer,
        ];

        let res = SheetBuilder::new(path)
            .trim(true)
            .header_strategy(HeaderStrategy::ReadLabels(ct))
            .build();

        match res {
            Err(e) => panic!("{}", e),
            Ok(mut sh) => {
                if let Some(rw) = sh.get_row_by_index(0) {
                    if let Some(cell) = rw.get_cell_by_index(0) {
                        assert_eq!("Cell { id: 0, data: Text(\"JAN\") }", format!("{:?}", cell))
                    } else {
                        panic!("There should be an index 0 cell")
                    }
                } else {
                    panic!("There should be an index 0 row")
                };

                match sh.sort_rows(1) {
                    Err(e) => panic!("{}", e),
                    Ok(_) => {
                        if let Some(rw) = sh.get_row_by_index(0) {
                            if let Some(cell) = rw.get_cell_by_index(0) {
                                assert_eq!(
                                    "Cell { id: 0, data: Text(\"NOV\") }",
                                    format!("{:?}", cell)
                                )
                            } else {
                                panic!("There should be an index 0 cell")
                            }
                        } else {
                            panic!("There should be an index 0 row")
                        };
                    }
                };
            }
        }
    }

    #[test]
    fn test_sort_reversed() {
        let path: OsString = "./dummies/csv/air.csv".into();

        let ct = vec![
            ColumnType::Text,
            ColumnType::Integer,
            ColumnType::Integer,
            ColumnType::Integer,
        ];

        let res = SheetBuilder::new(path)
            .trim(true)
            .header_strategy(HeaderStrategy::ReadLabels(ct))
            .build();

        match res {
            Err(e) => panic!("{}", e),
            Ok(mut sh) => {
                if let Some(rw) = sh.get_row_by_index(0) {
                    if let Some(cell) = rw.get_cell_by_index(0) {
                        assert_eq!("Cell { id: 0, data: Text(\"JAN\") }", format!("{:?}", cell))
                    } else {
                        panic!("There should be an index 0 cell")
                    }
                } else {
                    panic!("There should be an index 0 row")
                };

                match sh.sort_rows_rev(1) {
                    Err(e) => panic!("{}", e),
                    Ok(_) => {
                        if let Some(rw) = sh.get_row_by_index(0) {
                            if let Some(cell) = rw.get_cell_by_index(0) {
                                assert_eq!(
                                    "Cell { id: 0, data: Text(\"AUG\") }",
                                    format!("{:?}", cell)
                                )
                            } else {
                                panic!("There should be an index 0 cell")
                            }
                        } else {
                            panic!("There should be an index 0 row")
                        };
                    }
                };
            }
        }
    }

    #[test]
    fn test_sort_panic() {
        let path: OsString = "./dummies/csv/air.csv".into();

        let res = SheetBuilder::new(path).build();

        match res {
            Err(e) => panic!("{}", e),
            Ok(mut sh) => match sh.sort_rows(1) {
                Ok(_) => panic!("Test should have panicked"),
                Err(e) => {
                    assert_eq!(
                        format!("{}", e),
                        "Invalid Column Sort: Tried to sort by an unstructured column "
                    )
                }
            },
        }
    }

    #[test]
    fn test_create_line_graph() {
        let res = create_air_csv().unwrap();

        let x_label = Some(String::from("X Label"));
        let y_label = Some(String::from("Y Label"));
        let label_strat = LineLabelStrategy::FromCell(0);
        let exclude_row = {
            let mut exl: HashSet<usize> = HashSet::new();
            exl.insert(2);
            exl.insert(5);
            exl
        };
        let exclude_column = {
            let mut exl: HashSet<usize> = HashSet::new();
            exl.insert(2);
            exl.insert(1);
            exl
        };

        if let Ok(lg) =
            res.to_line_graph(x_label, y_label, label_strat, exclude_row, exclude_column)
        {
            println!("{:?}", lg);
        };
    }

    #[test]
    fn test_transpose() {
        match create_air_csv() {
            Err(e) => panic!("Should'nt have errored here. {}", e),
            Ok(sht) => {
                match Sheet::transpose(&sht, Some(String::from("YEAR"))) {
                    Err(e) => panic!("{}", e),
                    Ok(res) => {
                        let rw1 = res.get_row_by_index(1).unwrap();

                        assert_eq!(
                            "360",
                            rw1.get_cell_by_index(1).unwrap().get_data().to_string()
                        );

                        let rw2 = res.get_row_by_index(2).unwrap();
                        assert_eq!(
                            "535",
                            rw2.get_cell_by_index(6).unwrap().get_data().to_string()
                        );

                        let rw0 = res.get_row_by_index(0).unwrap();
                        assert_eq!(
                            &Data::Integer(1958),
                            rw0.get_cell_by_index(0).unwrap().get_data()
                        );

                        let hr6 = res.get_headers().get(6).unwrap();
                        assert_eq!(&ColumnHeader::new("JUN".into(), ColumnType::Integer), hr6);
                        assert_eq!(ColumnType::Integer, hr6.kind);

                        let hr0 = res.get_headers().get(0).unwrap();
                        assert_eq!(&ColumnHeader::new("YEAR".into(), ColumnType::Integer), hr0);
                    }
                };
            }
        }
    }

    #[test]
    fn test_transpose_flexible() {
        let path: OsString = "./dummies/csv/transpose1.csv".into();

        let ct = vec![ColumnType::Text, ColumnType::Integer, ColumnType::Integer];

        let res = SheetBuilder::new(path)
            .trim(true)
            .header_strategy(HeaderStrategy::ReadLabels(ct))
            .flexible(true)
            .primary(0)
            .build();

        match res {
            Err(e) => panic!("Transpose flexible: {}", e),
            Ok(sht) => match Sheet::transpose(&sht, Some("Year".into())) {
                Err(e) => panic!("{}", e),
                Ok(res) => {
                    let rw0 = res.get_row_by_index(0).unwrap();
                    assert_eq!(
                        &Data::Integer(1958),
                        rw0.get_cell_by_index(0).unwrap().get_data()
                    );
                    assert_eq!(
                        &Data::Integer(3),
                        rw0.get_cell_by_index(2).unwrap().get_data()
                    );

                    let rw1 = res.get_row_by_index(1).unwrap();
                    assert_eq!(
                        &Data::Integer(2),
                        rw1.get_cell_by_index(1).unwrap().get_data()
                    );
                    assert_eq!(&Data::None, rw1.get_cell_by_index(2).unwrap().get_data());

                    if let Some(_) = res.get_row_by_index(2) {
                        panic!("Nothing should have been returned");
                    }

                    let hr2 = res.get_headers().get(2).unwrap();
                    assert_eq!(ColumnType::Integer, hr2.kind);
                }
            },
        };
    }

    #[test]
    fn test_transpose_headless() {
        let path: OsString = "./dummies/csv/headless.csv".into();
        match SheetBuilder::new(path)
            .header_strategy(HeaderStrategy::NoHeaders)
            .build()
        {
            Err(e) => panic!("{}", e),
            Ok(sht) => match Sheet::transpose(&sht, None) {
                Err(e) => panic!("{}", e),
                Ok(res) => {
                    let rw2 = res.get_row_by_index(2).unwrap();
                    assert_eq!(&Data::None, rw2.get_cell_by_index(0).unwrap().get_data());

                    let hr0 = res.get_headers().get(0).unwrap();
                    assert_eq!(&ColumnHeader::new(String::new(), ColumnType::None), hr0);

                    let hr2 = res.get_headers().get(2).unwrap();
                    assert_eq!(&ColumnHeader::new("Feb".into(), ColumnType::Text), hr2);

                    if let Some(_) = res.get_headers().get(3) {
                        panic!("Shouldn't have returned anything");
                    };
                }
            },
        }
    }

    #[test]
    fn test_transpose_symmetry() {
        let headless: OsString = "./dummies/csv/headless.csv".into();

        let ct = vec![
            ColumnHeader::new("".into(), ColumnType::Text),
            ColumnHeader::new("".into(), ColumnType::Integer),
            ColumnHeader::new("".into(), ColumnType::Integer),
            ColumnHeader::new("".into(), ColumnType::Integer),
        ];

        match SheetBuilder::new(headless)
            .header_strategy(HeaderStrategy::Provided(ct))
            .trim(true)
            .build()
        {
            Err(e) => panic!("{}", e),
            Ok(sh) => match Sheet::transpose(&sh, None) {
                Err(e) => panic!("{}", e),
                Ok(res) => match Sheet::transpose(&res, None) {
                    Err(e) => panic!("{}", e),
                    Ok(sh2) => {
                        assert_eq!(sh, sh2);
                    }
                },
            },
        }

        let flexible: OsString = "./dummies/csv/transpose1.csv".into();
        let ct = vec![ColumnType::Text, ColumnType::Integer, ColumnType::Integer];

        match SheetBuilder::new(flexible)
            .header_strategy(HeaderStrategy::ReadLabels(ct))
            .flexible(true)
            .trim(true)
            .build()
        {
            Err(e) => panic!("{}", e),
            Ok(sh) => match Sheet::transpose(&sh, None) {
                Err(e) => panic!("{}", e),
                Ok(res) => match Sheet::transpose(&res, None) {
                    Err(e) => panic!("{}", e),
                    Ok(sh2) => assert_eq!(sh, sh2),
                },
            },
        };

        match create_air_csv() {
            Err(e) => panic!("{}", e),
            Ok(sh) => match Sheet::transpose(&sh, None) {
                Err(e) => panic!("{}", e),
                Ok(res) => match Sheet::transpose(&res, None) {
                    Err(e) => panic!("{}", e),
                    Ok(sh2) => assert_eq!(sh, sh2),
                },
            },
        };
    }
}

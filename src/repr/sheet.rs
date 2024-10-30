use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    slice::{Iter, IterMut},
};

use csv::Trim;

use crate::models::{
    bar::{Bar, BarChart},
    line::{Line, LineGraph},
    stacked_bar::{StackedBar, StackedBarChart},
    ScaleKind,
};
use crate::models::{Point, Scale};

pub mod error;
pub use error::*;
pub mod utils;
pub use utils::*;
pub mod builders;
mod tests;

#[derive(Debug, Clone, PartialEq)]
pub struct Cell {
    id: usize,
    data: Data,
}

#[allow(dead_code)]
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

    pub fn is_empty(&self) -> bool {
        self.data == Data::None
    }

    /// Modifies the data in this cell
    pub fn set_data(&mut self, new_data: Data) {
        self.data = new_data;
    }

    fn validate_type(&self, kind: &ColumnType) -> Result<()> {
        if kind.crosscheck_type(&self.data) {
            Ok(())
        } else {
            Err(Error::InvalidColumnType(format!(
                "Expected {:?} type but had {:?} type for cell with id: {}",
                kind, self.data, self.id
            )))
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    id: usize,
    cells: Vec<Cell>,
    primary: usize,
    id_counter: usize,
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

    fn width(&self) -> usize {
        self.cells.len()
    }

    fn is_key_valid(&self, key: usize) -> bool {
        self.cells.len() > key
    }

    pub fn is_primary_key_valid(&self) -> Result<()> {
        if !self.is_key_valid(self.primary) {
            return Err(Error::InvalidPrimaryKey(format!(
                "Primary key is invalid for row with id: {}",
                self.id
            )));
        };
        Ok(())
    }

    fn validate_all_cols(&self, headers: &[ColumnHeader]) -> Result<()> {
        if self.cells.len() != headers.len() {
            return Err(Error::InvalidColumnLength(format!(
                "Row with id, {}, has unbalanced cells.",
                self.id
            )));
        }

        self.iter_cells().enumerate().try_fold((), |_, curr| {
            let header = headers.get(curr.0).unwrap();
            if header.crosscheck_type(&curr.1.data) {
                Ok(())
            } else {
                Err(Error::InvalidColumnType(format!(
                    "Expected {:?} type but had {:?} type for cell id: {}, in row id: {}. ",
                    header.kind, curr.1.data, curr.1.id, self.id
                )))
            }
        })
    }

    fn validate_col(&self, header: &ColumnHeader, col: usize) -> Result<()> {
        let cell = self.cells.get(col);
        match cell {
            None => Err(Error::InvalidColumnLength(
                "Tried to validate out of bounds column".into(),
            )),
            Some(cl) => {
                if header.crosscheck_type(&cl.data) {
                    Ok(())
                } else {
                    Err(Error::InvalidColumnType(format!(
                        "Expected cell of {:?} type, but had {:?} type in cell id {} in row id {}",
                        header.kind, cl.data, cl.id, self.id
                    )))
                }
            }
        }
    }

    pub fn set_primary_key(&mut self, new_primary: usize) -> Result<()> {
        if new_primary < self.cells.len() {
            self.primary = new_primary;
            Ok(())
        } else {
            Err(Error::InvalidPrimaryKey(
                "Tried to set primary key to invalid value".into(),
            ))
        }
    }

    pub fn is_empty(&self) -> bool {
        self.cells.iter().all(|cell| cell.is_empty())
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
        x_values: &[Data],
        exclude: &HashSet<usize>,
        idx: usize,
    ) -> Line {
        let points: Vec<Point> = match label {
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

        match label {
            LineLabelStrategy::None => Line::from_points(points),
            LineLabelStrategy::Provided(labels) => match labels.get(idx).cloned() {
                Some(label) => Line::from_points(points).label(label),
                None => Line::from_points(points),
            },
            LineLabelStrategy::FromCell(idx) => {
                let label = self.cells.get(*idx).map(|cell| cell.data.to_string());
                match label {
                    Some(label) => Line::from_points(points).label(label),
                    None => Line::from_points(points),
                }
            }
        }
    }

    fn create_stacked_bar_chart(
        &self,
        x_col: usize,
        cols: &[usize],
        labels: &[String],
    ) -> Result<[(StackedBar, Data); 2]> {
        let x = self
            .cells
            .get(x_col)
            .cloned()
            .expect("Row create stacked bar: Validations failed.")
            .data;

        let mut pos = Vec::with_capacity(cols.len());
        let mut neg = Vec::with_capacity(cols.len());

        for (col, label) in cols.iter().zip(labels) {
            let data = self
                .cells
                .get(*col)
                .cloned()
                .expect("Row create stacked bar: Validations failed")
                .data;

            if data.is_negative() {
                neg.push((label.clone(), data));
            } else {
                pos.push((label.clone(), data));
            }
        }

        let pos_total = pos
            .iter()
            .map(|(_, data)| data)
            .fold(Data::None, |acc, curr| match (acc, curr) {
                (Data::None, Data::Integer(i)) => Data::Integer(*i),
                (Data::None, Data::Float(f)) => Data::Float(*f),
                (Data::None, Data::Number(n)) => Data::Number(*n),
                (Data::Integer(x), Data::Integer(y)) => Data::Integer(x + y),
                (Data::Number(x), Data::Number(y)) => Data::Number(x + y),
                (Data::Float(x), Data::Float(y)) => Data::Float(x + y),
                _ => Data::None,
            });

        let neg_total = neg
            .iter()
            .map(|(_, data)| data)
            .fold(Data::None, |acc, curr| match (acc, curr) {
                (Data::None, Data::Integer(i)) => Data::Integer(*i),
                (Data::None, Data::Float(f)) => Data::Float(*f),
                (Data::None, Data::Number(n)) => Data::Number(*n),
                (Data::Integer(x), Data::Integer(y)) => Data::Integer(x + y),
                (Data::Number(x), Data::Number(y)) => Data::Number(x + y),
                (Data::Float(x), Data::Float(y)) => Data::Float(x + y),
                _ => Data::None,
            });

        let pos_fractions = pos
            .into_iter()
            .map(|(label, data)| {
                let fraction = match (&pos_total, data) {
                    (Data::Integer(t), Data::Integer(i)) => {
                        if *t == 0 {
                            0.0
                        } else {
                            (i as f64) / (*t as f64)
                        }
                    }
                    (Data::Number(t), Data::Number(n)) => {
                        if *t == 0 {
                            0.0
                        } else {
                            (n as f64) / (*t as f64)
                        }
                    }
                    (Data::Float(t), Data::Float(f)) => {
                        if *t == 0.0 {
                            0.0
                        } else {
                            (f as f64) / (*t as f64)
                        }
                    }
                    _ => panic!("Row create stacked bar: So many validations failed"),
                };
                (label, fraction)
            })
            .collect::<HashMap<String, f64>>();

        let neg_fractions = neg
            .into_iter()
            .map(|(label, data)| {
                let fraction = match (&neg_total, data) {
                    (Data::Integer(t), Data::Integer(i)) => {
                        if *t == 0 {
                            0.0
                        } else {
                            (i as f64) / (*t as f64)
                        }
                    }
                    (Data::Number(t), Data::Number(n)) => {
                        if *t == 0 {
                            0.0
                        } else {
                            (n as f64) / (*t as f64)
                        }
                    }
                    (Data::Float(t), Data::Float(f)) => {
                        if *t == 0.0 {
                            0.0
                        } else {
                            (f as f64) / (*t as f64)
                        }
                    }
                    _ => {
                        panic!("Row create stacked bar: So many validations failed")
                    }
                };
                (label, fraction)
            })
            .collect::<HashMap<String, f64>>();

        let pos_pnt = Point::new(x.clone(), pos_total.clone());
        let pos_bar = StackedBar::new(pos_pnt, pos_fractions, false);

        let neg_pnt = Point::new(x, neg_total.clone());
        let neg_bar = StackedBar::new(neg_pnt, neg_fractions, true);

        Ok([(pos_bar, pos_total), (neg_bar, neg_total)])
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Sheet {
    rows: Vec<Row>,
    headers: Vec<ColumnHeader>,
    id_counter: usize,
    primary_key: usize,
}

#[allow(dead_code)]
impl Sheet {
    /// Returns a new T vector with length equal to given length
    /// A default T is used as padding. Any extras are trimmed
    fn balance_vector<T: Clone + Default>(lst: Vec<T>, size: usize) -> Vec<T> {
        let len = lst.len();
        match len {
            len if len < size => {
                let mut cln = lst.clone();
                let mut pad = vec![T::default(); size - len];

                cln.append(&mut pad);

                cln
            }
            len if len < size => {
                let mut cln = lst.clone();
                cln.truncate(size);
                cln
            }
            _ => lst,
        }
    }

    /// Create a new sheet given the path to a csv file
    pub fn new(
        path: PathBuf,
        primary: usize,
        label_strategy: HeaderLabelStrategy,
        type_strategy: HeaderTypesStrategy,
        trim: bool,
        flexible: bool,
        delimiter: u8,
    ) -> Result<Self> {
        let mut counter: usize = 0;
        let mut longest_row = 0;

        let has_headers = match label_strategy {
            HeaderLabelStrategy::ReadLabels => true,
            HeaderLabelStrategy::NoLabels => false,
            HeaderLabelStrategy::Provided(_) => false,
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
            .delimiter(delimiter)
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

        let types = match &type_strategy {
            HeaderTypesStrategy::Provided(ct) => Sheet::balance_vector(ct.to_owned(), longest_row),
            HeaderTypesStrategy::Infer => {
                Sheet::balance_vector(Vec::<ColumnType>::new(), longest_row)
            }
            HeaderTypesStrategy::None => {
                Sheet::balance_vector(Vec::<ColumnType>::new(), longest_row)
            }
        };

        let labels = match &label_strategy {
            HeaderLabelStrategy::Provided(ch) => Sheet::balance_vector(ch.to_owned(), longest_row),
            HeaderLabelStrategy::NoLabels => {
                Sheet::balance_vector(Vec::<String>::new(), longest_row)
            }
            HeaderLabelStrategy::ReadLabels => {
                let labels: Vec<String> = rdr
                    .headers()?
                    .clone()
                    .into_iter()
                    .map(|curr| curr.to_string())
                    .collect();
                Sheet::balance_vector(labels, longest_row)
            }
        };

        let headers: Vec<ColumnHeader> = labels
            .into_iter()
            .zip(types)
            .map(|(lbl, typ)| ColumnHeader::new(lbl, typ))
            .collect();

        let mut sh = Sheet {
            rows,
            headers,
            id_counter: counter,
            primary_key: primary,
        };

        if type_strategy == HeaderTypesStrategy::Infer {
            Sheet::infer_col_kinds(&mut sh, longest_row);
        }

        sh.validate()?;

        Ok(sh)
    }

    /// Returns the width of the [`Sheet`].
    pub fn width(&self) -> usize {
        self.rows.first().map(|row| row.width()).unwrap_or(0)
    }

    /// Returns the height of the [`Sheet`].
    ///
    /// A [`Sheet`]'s height is equivalent to the number of rows present within
    /// it
    pub fn height(&self) -> usize {
        self.rows.len()
    }

    pub fn get_row_by_index(&self, index: usize) -> Option<&Row> {
        self.rows.get(index)
    }

    pub fn get_row_by_id(&self, id: usize) -> Option<&Row> {
        self.rows.iter().find(|row| row.id == id)
    }

    pub fn is_empty(&self) -> bool {
        self.rows.iter().all(|row| row.is_empty())
    }

    /// Could be expensive
    pub fn validate(&self) -> Result<()> {
        // Validating could be expensive
        Self::is_primary_valid(self)?;
        Self::validate_all_cols(self)?;

        Ok(())
    }

    /// Checks if the type for each column cell is as expected
    fn validate_all_cols(sh: &Sheet) -> Result<()> {
        let hrs = &sh.headers;

        sh.iter_rows()
            .try_fold((), |_, curr| curr.validate_all_cols(hrs))
    }

    fn validate_col(&self, col: usize) -> Result<()> {
        let hdr = self.headers.get(col).ok_or(Error::InvalidColumnLength(
            "Tried to access out of range column".to_string(),
        ))?;

        self.iter_rows()
            .try_fold((), |_, curr| curr.validate_col(hdr, col))
    }

    fn is_primary_valid(sh: &Sheet) -> Result<()> {
        let len = sh.headers.len();
        let pk = sh.primary_key;

        if (len == pk && pk != 0) || (len < pk) {
            return Err(Error::InvalidPrimaryKey(
                "Primary key out of column range".into(),
            ));
        }

        sh.rows
            .iter()
            .try_fold((), |_acc, curr| curr.is_primary_key_valid())
    }

    fn set_primary_key(&mut self, new_key: usize) -> Result<()> {
        if self.rows.iter().all(|curr| curr.is_key_valid(new_key)) {
            self.primary_key = new_key;
            self.rows
                .iter_mut()
                .for_each(|row| row.set_primary_key(new_key).unwrap());
            return Ok(());
        }
        Err(Error::InvalidPrimaryKey(
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

    pub fn sort_rows(&mut self, col: usize) -> Result<()> {
        let ch = self
            .headers
            .get(col)
            .ok_or(Error::InvalidColumnLength("Column out of range".into()))?;

        if let ColumnHeader {
            kind: ColumnType::None,
            ..
        } = ch
        {
            return Err(Error::InvalidColumnSort(
                "Tried to sort by an unstructured column ".into(),
            ));
        }

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

    pub fn sort_rows_rev(&mut self, col: usize) -> Result<()> {
        let ch = self
            .headers
            .get(col)
            .ok_or(Error::InvalidColumnLength("Column out of range".into()))?;

        if let ColumnHeader {
            kind: ColumnType::None,
            ..
        } = ch
        {
            return Err(Error::InvalidColumnSort(
                "Tried to sort by an unstructured column ".into(),
            ));
        }

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
                        None => Some(cr),
                        Some(ac) => match (ac, cr) {
                            (ColumnType::None, x) => {
                                if is_first_iteration {
                                    is_first_iteration = false;
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
            .map(|op| op.unwrap_or_default())
            .collect();

        sh.headers.iter_mut().zip(col_kinds).for_each(|(hdr, knd)| {
            hdr.kind = knd;
        });
    }

    /// initial_header: The new label for the initial header, if any
    ///
    /// uniform_type: Whether every non-zeroth column has the same type.
    /// types are lost if false
    pub fn transpose(sheet: &Sheet, initial_header: Option<String>) -> Result<Self> {
        Sheet::validate(sheet)?;

        let width = sheet.headers.len();
        let depth = sheet.rows.len() + 1;

        let mut headers: Vec<ColumnHeader> = Vec::new();
        let mut rows: Vec<Vec<Cell>> = Vec::new();

        for idx in 0..width {
            let hr = match sheet.headers.get(idx) {
                Some(hdr) => {
                    let mut h = hdr.clone();
                    h.kind = ColumnType::Text;
                    h
                }
                None => return Err(Error::TransposeError("Sheet has missing headers".into())),
            };

            if idx == 0 {
                let hr = match &initial_header {
                    None => hr,
                    Some(lbl) => ColumnHeader::new(lbl.clone(), hr.kind),
                };
                let mut hrs = sheet
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
                let mut cls: Vec<Cell> = sheet
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

    fn copy_col_data(&self, col: usize) -> Result<Vec<Data>> {
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

    fn grab_header(&self, col: usize) -> Result<&ColumnHeader> {
        let hr = self.headers.get(col).ok_or(Error::InvalidColumnLength(
            "Tried accessing an out of bounds Header".into(),
        ))?;

        match hr.kind {
            ColumnType::None => Err(Error::ConversionError(
                "Cannot convert non uniform type column".into(),
            )),
            _ => Ok(hr),
        }
    }

    fn validate_to_line_graph(&self, label_strat: &LineLabelStrategy) -> Result<ScaleKind> {
        // None type Columns
        self.headers.iter().try_fold((), |_acc, curr| {
            if let ColumnHeader {
                kind: ColumnType::None,
                ..
            } = curr
            {
                Err(Error::ConversionError(
                    "Cannot convert non uniform type column".into(),
                ))
            } else {
                Ok(())
            }
        })?;

        let check_uniform_type = |acc: ColumnType, ct: ColumnType| match (&acc, &ct) {
            (ColumnType::None, _) => Ok(ct),
            (x, y) => {
                if x == y {
                    Ok(ct)
                } else {
                    Err(Error::ConversionError(
                        "Cannot convert different column types".into(),
                    ))
                }
            }
        };

        // Uniform type columns
        let kind = match label_strat {
            LineLabelStrategy::FromCell(idx) => {
                if idx >= &self.headers.len() {
                    return Err(Error::ConversionError(
                        "Tried to assign invalid column as label".into(),
                    ));
                }

                self.headers
                    .iter()
                    .map(|hrd| &hrd.kind)
                    .enumerate()
                    .filter(|(ind, _)| ind != idx)
                    .try_fold(ColumnType::None, |acc, (_, ct)| {
                        check_uniform_type(acc, *ct)
                    })?
            }

            _ => self
                .headers
                .iter()
                .map(|hdr| &hdr.kind)
                .try_fold(ColumnType::None, |acc, ct| check_uniform_type(acc, *ct))?,
        };

        Ok(kind.into())
    }

    fn validate_to_barchart(
        &self,
        x_col: usize,
        y_col: usize,
        bar_label: &BarChartBarLabels,
    ) -> Result<(ScaleKind, ScaleKind)> {
        if let BarChartBarLabels::FromColumn(idx) = bar_label {
            if idx >= &self.headers.len() {
                return Err(Error::ConversionError(
                    "Bar chart label column out of range".into(),
                ));
            }
        }

        let x_type = self
            .headers
            .get(x_col)
            .ok_or(Error::ConversionError(
                "Bar chart column out of range".into(),
            ))?
            .kind;

        if x_type == ColumnType::None {
            return Err(Error::ConversionError(
                "Cannot convert from non-uniform column".into(),
            ));
        }

        let y_type = self
            .headers
            .get(y_col)
            .ok_or(Error::ConversionError(
                "Bar chart column out of range".into(),
            ))?
            .kind;

        if y_type == ColumnType::None {
            return Err(Error::ConversionError(
                "Cannot convert from non-uniform column".into(),
            ));
        }

        Ok((x_type.into(), y_type.into()))
    }

    fn validate_to_stacked_bar_chart(
        &self,
        x_col: usize,
        cols: &[usize],
    ) -> Result<(Vec<String>, ScaleKind)> {
        self.headers.get(x_col).ok_or(Error::ConversionError(
            "Stacked Bar chart: x column out of range".into(),
        ))?;

        let mut kind = None;
        let mut labels = Vec::with_capacity(cols.len());

        for col in cols.iter() {
            let header = self
                .headers
                .get(*col)
                .cloned()
                .ok_or(Error::ConversionError(
                    "Stacked Bar chart: Accumulating column, out of range".into(),
                ))?;

            match kind {
                None => kind = Some(header.kind),
                Some(prev) => {
                    if prev != header.kind {
                        return Err(Error::ConversionError(
                            "Stacked Bar chart: Cannot Accumulate different column types".into(),
                        ));
                    }
                }
            };

            labels.push(header.label);
        }

        match kind {
            Some(ColumnType::Number) | Some(ColumnType::Float) | Some(ColumnType::Integer) => {
                Ok((labels, kind.unwrap().into()))
            }
            Some(_) => Err(Error::ConversionError(
                "Stacked Bar Chart Cannot accumulate column type".into(),
            )),
            None => Err(Error::ConversionError(
                "Stacked Bar chart: Empty Accumulation columns".into(),
            )),
        }
    }

    /// Returns a new line graph created from this csv struct
    ///
    /// exclude_row: The positions of the rows to exclude in this transformation
    /// exclude_column: The positions of columns to exclude in the
    /// transformation
    pub fn create_line_graph(
        &self,
        x_label: Option<String>,
        y_label: Option<String>,
        label_strat: LineLabelStrategy,
        exclude_row: HashSet<usize>,
        exclude_column: HashSet<usize>,
    ) -> Result<LineGraph> {
        self.validate()?;
        let scale_kind = self.validate_to_line_graph(&label_strat)?;

        if self.is_empty() {
            return Err(Error::ConversionError(
                "Cannot convert an empty sheet".into(),
            ));
        }

        let x_values: Vec<Data> = self
            .headers
            .iter()
            .map(|hdr| Data::Text(hdr.label.clone()))
            .collect();

        let lines: Vec<Line> = self
            .iter_rows()
            .enumerate()
            .filter(|(idx, _)| !exclude_row.contains(idx))
            .map(|(_, row)| row)
            .enumerate()
            .map(|(idx, rw)| rw.create_line(&label_strat, &x_values, &exclude_column, idx))
            .collect();

        let y_scale = {
            let values = lines
                .iter()
                .flat_map(|ln| ln.points.iter().map(|pnt| pnt.y.clone()));

            Scale::new(values, scale_kind)
        };

        let x_scale = match label_strat {
            LineLabelStrategy::FromCell(id) => {
                let values = x_values.into_iter().enumerate().filter_map(|(idx, lbl)| {
                    if idx != id && !exclude_column.contains(&idx) {
                        Some(lbl)
                    } else {
                        None
                    }
                });
                Scale::new(values, ScaleKind::Categorical)
            }
            _ => {
                let values = x_values.into_iter().enumerate().filter_map(|(idx, lbl)| {
                    if !exclude_column.contains(&idx) {
                        Some(lbl)
                    } else {
                        None
                    }
                });
                Scale::new(values, ScaleKind::Categorical)
            }
        };

        let lg = LineGraph::new(lines, x_label, y_label, x_scale, y_scale)
            .map_err(Error::LineGraphError)?;

        Ok(lg)
    }

    pub fn create_bar_chart(
        self,
        x_col: usize,
        y_col: usize,
        bar_label: BarChartBarLabels,
        axis_labels: BarChartAxisLabelStrategy,
        exclude_row: HashSet<usize>,
    ) -> Result<BarChart> {
        let (x_kind, y_kind) = self.validate_to_barchart(x_col, y_col, &bar_label)?;

        if self.is_empty() {
            return Err(Error::ConversionError(
                "Cannot convert an empty sheet".into(),
            ));
        }

        let x_values = self
            .rows
            .iter()
            .enumerate()
            .filter(|(idx, _)| !exclude_row.contains(idx))
            .map(|(_, row)| {
                row.cells
                    .get(x_col)
                    .expect("Bar conversion: All Rows should have the same length")
                    .data
                    .clone()
            });

        let y_values = self
            .rows
            .iter()
            .enumerate()
            .filter(|(idx, _)| !exclude_row.contains(idx))
            .map(|(_, row)| {
                row.cells
                    .get(y_col)
                    .expect("Bar conversion: All Rows should have the same length")
                    .data
                    .clone()
            });

        let points = x_values
            .into_iter()
            .zip(y_values)
            .map(|(x, y)| Point::new(x, y));

        let labels: Vec<Option<String>> = match bar_label {
            BarChartBarLabels::Provided(labels) => labels
                .into_iter()
                .map(|label| if label.is_empty() { None } else { Some(label) })
                .collect(),
            BarChartBarLabels::FromColumn(ind) => self
                .rows
                .iter()
                .enumerate()
                .filter(|(idx, _)| !exclude_row.contains(idx))
                .map(|(_, row)| {
                    let label = row
                        .cells
                        .get(ind)
                        .expect("Bar conversion: All Rows should have the same length")
                        .data
                        .to_string();
                    Some(label)
                })
                .collect(),
            BarChartBarLabels::None => vec![None; self.rows.len()],
        };

        let labels = Self::balance_vector(labels, self.rows.len());

        let bars = labels
            .into_iter()
            .zip(points)
            .map(|(label, point)| match label {
                Some(label) => Bar::new(label, point),
                None => Bar::from_point(point),
            })
            .collect::<Vec<Bar>>();

        let x_scale = {
            let values = bars.iter().map(|bar| bar.point.x.clone());

            Scale::new(values, x_kind)
        };

        let y_scale = {
            let values = bars.iter().map(|bar| bar.point.y.clone());

            Scale::new(values, y_kind)
        };

        let barchart = BarChart::new(bars, x_scale, y_scale)?;

        match axis_labels {
            BarChartAxisLabelStrategy::Headers => {
                let x = self
                    .headers
                    .get(x_col)
                    .expect("Bar conversion: Invalid header access")
                    .label
                    .clone();
                let y = self
                    .headers
                    .get(y_col)
                    .expect("Bar conversion: Invalid header access")
                    .label
                    .clone();

                Ok(barchart.x_label(x).y_label(y))
            }
            BarChartAxisLabelStrategy::Provided { x, y } => Ok(barchart.x_label(x).y_label(y)),
            BarChartAxisLabelStrategy::None => Ok(barchart),
        }
    }

    pub fn create_stacked_bar_chart(
        self,
        x_col: usize,
        cols: impl IntoIterator<Item = usize>,
        axis_labels: StackedBarChartAxisLabelStrategy,
    ) -> Result<StackedBarChart> {
        let cols = cols
            .into_iter()
            .collect::<HashSet<usize>>()
            .into_iter()
            .collect::<Vec<usize>>();

        let (acc_labels, y_kind) = self.validate_to_stacked_bar_chart(x_col, &cols)?;

        if self.is_empty() {
            return Err(Error::ConversionError(
                "Cannot convert an empty sheet".into(),
            ));
        }

        let x_values = self.rows.iter().map(|row| {
            row.cells
                .get(x_col)
                .cloned()
                .expect("Stacked Bar Chart conversion: Validations failed")
                .data
        });
        let mut y_values = Vec::default();
        let mut bars = Vec::default();

        for row in self.rows.iter() {
            let [pos, neg] = row.create_stacked_bar_chart(x_col, &cols, &acc_labels)?;

            if pos.1 != Data::None {
                bars.push(pos.0);
                y_values.push(pos.1);
            }

            if neg.1 != Data::None {
                bars.push(neg.0);
                y_values.push(neg.1);
            }
        }

        let x_scale = {
            let kind = self
                .headers
                .get(x_col)
                .expect("Stacked Bar Chart conversion: Validations failed")
                .kind;

            Scale::new(x_values, kind.into())
        };

        let y_scale = Scale::new(y_values, y_kind);

        let acc_labels = acc_labels.into_iter().collect();

        let stacked = StackedBarChart::new(bars, x_scale, y_scale, acc_labels)?;

        match axis_labels {
            StackedBarChartAxisLabelStrategy::None => Ok(stacked),
            StackedBarChartAxisLabelStrategy::Header(y_label) => {
                let x_label = self
                    .headers
                    .get(x_col)
                    .cloned()
                    .map(|header| header.label)
                    .unwrap_or(String::default());

                Ok(stacked.x_axis(x_label).y_axis(y_label))
            }
            StackedBarChartAxisLabelStrategy::Provided { x, y } => Ok(stacked.x_axis(x).y_axis(y)),
        }
    }
}

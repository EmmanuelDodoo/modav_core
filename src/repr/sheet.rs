use std::{
    collections::HashSet,
    path::PathBuf,
    slice::{Iter, IterMut},
};

use csv::Trim;

use crate::models::line::{Line, LineGraph, Point, Scale};
use crate::traits::line::ToLineGraph;

pub mod error;
use error::*;
pub mod utils;
use utils::*;
pub mod builders;

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

    /// Modifies the data in this cell
    pub fn set_data(&mut self, new_data: Data) {
        self.data = new_data;
    }

    fn validate_type(&self, kind: &ColumnType) -> Result<()> {
        if kind.crosscheck_type(&self.data) {
            return Ok(());
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

    fn validate_all_cols(&self, headers: &Vec<ColumnHeader>) -> Result<()> {
        if self.cells.len() != headers.len() {
            return Err(Error::InvalidColumnLength(format!(
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
                    if header.crosscheck_type(&curr.1.data) {
                        return Ok(());
                    } else {
                        return Err(Error::InvalidColumnType(format!(
                            "Expected {:?} type but had {:?} type for cell id: {}, in row id: {}. ",
                            header.kind, curr.1.data, curr.1.id, self.id
                        )));
                    }
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
        idx: usize,
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
            LineLabelStrategy::Provided(labels) => labels.get(idx).cloned(),
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
            .zip(types.into_iter())
            .map(|(lbl, typ)| ColumnHeader::new(lbl, typ))
            .collect();

        let mut sh = Sheet {
            rows,
            headers,
            id_counter: counter,
            primary_key: primary,
        };

        match type_strategy {
            HeaderTypesStrategy::Infer => {
                Sheet::infer_col_kinds(&mut sh, longest_row);
            }
            _ => {}
        }

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
    pub fn validate(&self) -> Result<()> {
        // Validating could be expensive
        Self::is_primary_valid(self)?;
        Self::validate_all_cols(self)?;

        Ok(())
    }

    /// Checks if the type for each column cell is as expected
    fn validate_all_cols(sh: &Sheet) -> Result<()> {
        let hrs = &sh.headers;
        sh.iter_rows().fold(Ok(()), |acc, curr| match acc {
            Err(e) => Err(e),
            Ok(()) => curr.validate_all_cols(hrs),
        })
    }

    fn validate_col(&self, col: usize) -> Result<()> {
        let hdr = self
            .headers
            .get(col)
            .ok_or(Error::InvalidColumnLength(format!(
                "Tried to access out of range column"
            )))?;

        self.iter_rows().fold(Ok(()), |acc, curr| match acc {
            Err(e) => Err(e),
            Ok(()) => curr.validate_col(hdr, col),
        })
    }

    fn is_primary_valid(sh: &Sheet) -> Result<()> {
        let len = sh.headers.len();
        let pk = sh.primary_key;

        if (len == pk && pk != 0) || (len < pk) {
            return Err(Error::InvalidPrimaryKey(
                "Primary key out of column range".into(),
            ));
        }

        sh.rows.iter().fold(Ok(()), |acc, curr| match acc {
            Err(e) => Err(e),
            Ok(()) => curr.is_primary_key_valid(),
        })
    }

    fn set_primary_key(&mut self, new_key: usize) -> Result<()> {
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

        match ch {
            ColumnHeader {
                label: _,
                kind: ColumnType::None,
            } => {
                return Err(Error::InvalidColumnSort(
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

    pub fn sort_rows_rev(&mut self, col: usize) -> Result<()> {
        let ch = self
            .headers
            .get(col)
            .ok_or(Error::InvalidColumnLength("Column out of range".into()))?;

        match ch {
            ColumnHeader {
                label: _,
                kind: ColumnType::None,
            } => {
                return Err(Error::InvalidColumnSort(
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
    pub fn transpose(self: &Self, initial_header: Option<String>) -> Result<Self> {
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
                None => return Err(Error::TransposeError("Sheet has missing headers".into())),
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
            ColumnType::None => {
                return Err(Error::LineGraphConversionError(
                    "Cannot convert non uniform type column".into(),
                ))
            }
            _ => Ok(hr),
        }
    }

    fn validate_to_line_graph(&self, label_strat: &LineLabelStrategy) -> Result<()> {
        // None type Columns
        self.headers
            .iter()
            .fold(Ok(()), |acc, curr| match (acc, &curr.kind) {
                (Err(e), _) => return Err(e),
                (Ok(_), ColumnType::None) => {
                    return Err(Error::LineGraphConversionError(
                        "Cannot convert non uniform type column".into(),
                    ));
                }
                (Ok(_), _) => Ok(()),
            })?;

        let check_uniform_type = |acc: Result<ColumnType>, ct: ColumnType| {
            if let Ok(acc) = acc {
                match (&acc, &ct) {
                    (ColumnType::None, _) => Ok(ct),
                    (x, y) => {
                        if x == y {
                            return Ok(ct);
                        } else {
                            return Err(Error::LineGraphConversionError(
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
                    return Err(Error::LineGraphConversionError(
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
    pub fn create_line_graph(
        self: &Self,
        x_label: Option<String>,
        y_label: Option<String>,
        label_strat: LineLabelStrategy,
        exclude_row: HashSet<usize>,
        exclude_column: HashSet<usize>,
    ) -> Result<LineGraph<String, Data>> {
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
            .map(|(_, row)| row)
            .enumerate()
            .map(|(idx, rw)| rw.create_line(&label_strat, &x_values, &exclude_column, idx))
            .collect();

        let y_scale: Scale<Data> = {
            let values: HashSet<Data> = lines
                .iter()
                .flat_map(|ln| ln.points.iter().map(|pnt| pnt.y.clone()))
                .collect();

            let mut values = values.into_iter().collect::<Vec<Data>>();
            values.sort();

            Scale::List(values)
        };

        let x_scale = {
            let values: HashSet<String> = match label_strat {
                LineLabelStrategy::FromCell(ref id) => x_values
                    .into_iter()
                    .enumerate()
                    .filter(|(idx, _)| idx != id && !exclude_column.contains(idx))
                    .map(|(_, lbl)| lbl)
                    .collect(),
                _ => x_values
                    .into_iter()
                    .enumerate()
                    .filter(|(idx, _)| !exclude_column.contains(idx))
                    .map(|(_, lbl)| lbl)
                    .collect(),
            };

            let mut values = values.into_iter().collect::<Vec<String>>();
            values.sort();

            Scale::List(values)
        };

        let lg = LineGraph::new(lines, x_label, y_label, x_scale, y_scale)
            .map_err(Error::LineGraphError)?;

        Ok(lg)
    }
}

impl ToLineGraph for Sheet {
    type X = String;
    type Y = Data;
    type ErrorType = Error;

    fn to_line_graph(
        self: &Self,
        x_label: Option<String>,
        y_label: Option<String>,
    ) -> Result<LineGraph<Self::X, Self::Y>> {
        Sheet::create_line_graph(
            &self,
            x_label,
            y_label,
            LineLabelStrategy::None,
            HashSet::new(),
            HashSet::new(),
        )
    }
}
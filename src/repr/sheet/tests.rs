#![allow(unused_variables)]
#![cfg(test)]
use core::panic;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::usize;

use crate::models::Scale;

use super::{
    builders::SheetBuilder,
    error::*,
    utils::{
        BarChartAxisLabelStrategy, BarChartBarLabels, ColumnHeader, ColumnType, Data,
        HeaderLabelStrategy, HeaderTypesStrategy, LineLabelStrategy,
        StackedBarChartAxisLabelStrategy,
    },
    Cell, Row, Sheet,
};

fn create_row() -> Row {
    let sr = csv::StringRecord::from(vec!["3", "2", "1"]);
    Row::new(sr, 4, 0)
}

fn create_air_csv() -> Result<Sheet> {
    let path: PathBuf = "./dummies/csv/air.csv".into();

    let ct = vec![
        ColumnType::Text,
        ColumnType::Integer,
        ColumnType::Integer,
        ColumnType::Integer,
    ];

    SheetBuilder::new(path.clone())
        .trim(true)
        .primary(0)
        .types(HeaderTypesStrategy::Provided(ct))
        .labels(HeaderLabelStrategy::ReadLabels)
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
    let path: PathBuf = "./dummies/csv/air.csv".into();
    let path2: PathBuf = "./dummies/csv/air2.csv".into();

    let ct = vec![
        ColumnType::Text,
        ColumnType::Integer,
        ColumnType::Integer,
        ColumnType::Integer,
    ];

    let res = SheetBuilder::new(path.clone())
        .trim(true)
        .primary(0)
        .labels(HeaderLabelStrategy::ReadLabels)
        .types(HeaderTypesStrategy::Provided(ct))
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

    let lbl: Vec<String> = vec!["Month".into(), "1958".into(), "1959".into()];

    let res = SheetBuilder::new(path2)
        .trim(true)
        .types(HeaderTypesStrategy::Infer)
        .labels(HeaderLabelStrategy::Provided(lbl))
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
                        "ColumnHeader { label: \"\", kind: Integer }",
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
    let path1: PathBuf = "./dummies/csv/invalid1.csv".into();

    let ct = vec![
        ColumnType::Text,
        ColumnType::Integer,
        ColumnType::Integer,
        ColumnType::Integer,
    ];

    let res = SheetBuilder::new(path1)
        .trim(true)
        .labels(HeaderLabelStrategy::ReadLabels)
        .types(HeaderTypesStrategy::Provided(ct))
        .build();

    match res {
        Err(e) => panic!("{}", e),
        Ok(_) => (),
    }
}

#[test]
fn test_col_validation2() {
    let path: PathBuf = "./dummies/csv/invalid2.csv".into();

    let ct = vec![
        ColumnType::Text,
        ColumnType::Integer,
        ColumnType::None,
        ColumnType::Integer,
    ];

    if let Err(e) = SheetBuilder::new(path)
        .trim(true)
        .labels(HeaderLabelStrategy::ReadLabels)
        .types(HeaderTypesStrategy::Provided(ct))
        .build()
    {
        panic!("{}", e)
    };
}

#[test]
fn test_empty_csv() {
    let path: PathBuf = "./dummies/csv/empty.csv".into();

    if let Err(e) = SheetBuilder::new(path)
        .labels(HeaderLabelStrategy::NoLabels)
        .trim(true)
        .build()
    {
        panic!("{}", e)
    }
}

#[test]
fn testing_empty_field() {
    let path: PathBuf = "./dummies/csv/address.csv".into();

    let res = SheetBuilder::new(path)
        .labels(HeaderLabelStrategy::NoLabels)
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
    let path: PathBuf = "./dummies/csv/flexible.csv".into();

    let res = SheetBuilder::new(path)
        .trim(true)
        .labels(HeaderLabelStrategy::NoLabels)
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
    let path: PathBuf = "./dummies/csv/air.csv".into();

    let ct = vec![
        ColumnType::Text,
        ColumnType::Integer,
        ColumnType::Integer,
        ColumnType::Integer,
    ];

    let res = SheetBuilder::new(path)
        .trim(true)
        .labels(HeaderLabelStrategy::ReadLabels)
        .types(HeaderTypesStrategy::Provided(ct))
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
                            assert_eq!("Cell { id: 0, data: Text(\"NOV\") }", format!("{:?}", cell))
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
    let path: PathBuf = "./dummies/csv/air.csv".into();

    let ct = vec![
        ColumnType::Text,
        ColumnType::Integer,
        ColumnType::Integer,
        ColumnType::Integer,
    ];

    let res = SheetBuilder::new(path)
        .trim(true)
        .labels(HeaderLabelStrategy::ReadLabels)
        .types(HeaderTypesStrategy::Provided(ct))
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
                            assert_eq!("Cell { id: 0, data: Text(\"AUG\") }", format!("{:?}", cell))
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
    let path: PathBuf = "./dummies/csv/air.csv".into();

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
        res.create_line_graph(x_label, y_label, label_strat, exclude_row, exclude_column)
    {
        println!("{:?}", lg);
    };
}

#[test]
fn test_line_scales() {
    let path: PathBuf = "./dummies/csv/alter.csv".into();

    let sht = SheetBuilder::new(path.clone())
        .trim(true)
        .flexible(false)
        .primary(0)
        .types(HeaderTypesStrategy::Infer)
        .labels(HeaderLabelStrategy::ReadLabels)
        .build()
        .expect("Building alter csv failure");

    let mut line = sht
        .create_line_graph(
            None,
            None,
            LineLabelStrategy::FromCell(0),
            HashSet::default(),
            HashSet::default(),
        )
        .expect("Building alter csv line graph failure");

    let mut expected_x_scale = {
        let values = vec![1958, 1959, 1960];
        let values = values.into_iter().map(|year| Data::Text(year.to_string()));

        Scale::new(values, crate::models::ScaleKind::Integer)
    };

    line.x_scale.sort();
    expected_x_scale.sort();
    assert_eq!(line.x_scale, expected_x_scale);

    let expected_y_scale = {
        let values = vec![
            318, 340, 342, 348, 360, 362, 363, 391, 396, 406, 417, 419, 420, 461, 472,
        ];

        let values = values.into_iter().map(|year| Data::Integer(year));

        Scale::new(values, crate::models::ScaleKind::Integer)
    };

    assert_eq!(line.y_scale, expected_y_scale);
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
    let path: PathBuf = "./dummies/csv/transpose1.csv".into();

    let ct = vec![ColumnType::Text, ColumnType::Integer, ColumnType::Integer];

    let res = SheetBuilder::new(path)
        .trim(true)
        .labels(HeaderLabelStrategy::ReadLabels)
        .types(HeaderTypesStrategy::Provided(ct))
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
    let path: PathBuf = "./dummies/csv/headless.csv".into();
    match SheetBuilder::new(path)
        .labels(HeaderLabelStrategy::NoLabels)
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
    let headless: PathBuf = "./dummies/csv/headless.csv".into();

    let labels: Vec<String> = vec![];

    match SheetBuilder::new(headless)
        .labels(HeaderLabelStrategy::Provided(labels))
        .types(HeaderTypesStrategy::Infer)
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

    let flexible: PathBuf = "./dummies/csv/transpose1.csv".into();
    let ct = vec![ColumnType::Text, ColumnType::Integer, ColumnType::Integer];

    match SheetBuilder::new(flexible)
        .labels(HeaderLabelStrategy::ReadLabels)
        .flexible(true)
        .types(HeaderTypesStrategy::Provided(ct))
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

#[test]
fn test_infer_types() {
    let path: PathBuf = "./dummies/csv/infer.csv".into();

    let res = SheetBuilder::new(path)
        .labels(HeaderLabelStrategy::ReadLabels)
        .trim(true)
        .types(HeaderTypesStrategy::Infer)
        .build();

    match res {
        Err(e) => panic!("{}", e),
        Ok(sh) => {
            let hr0 = sh.get_headers().get(0).unwrap();
            assert_eq!(ColumnType::Text, hr0.kind);

            let hr2 = sh.get_headers().get(2).unwrap();
            assert_eq!(ColumnType::Float, hr2.kind);

            let hr3 = sh.get_headers().get(3).unwrap();
            assert_eq!(ColumnType::None, hr3.kind);

            let hr5 = sh.get_headers().get(5).unwrap();
            assert_eq!(ColumnType::Text, hr5.kind);
        }
    }
}

#[test]
fn test_create_bar_chart() {
    let path: PathBuf = "./dummies/csv/infer.csv".into();

    let res = SheetBuilder::new(path)
        .labels(HeaderLabelStrategy::ReadLabels)
        .trim(true)
        .types(HeaderTypesStrategy::Infer)
        .build()
        .unwrap();

    let barchart = res
        .clone()
        .create_bar_chart(
            1,
            2,
            BarChartBarLabels::None,
            BarChartAxisLabelStrategy::None,
            HashSet::default(),
        )
        .unwrap();

    assert_eq!(barchart.x_label, None);
    assert_eq!(barchart.y_label, None);
    assert_eq!(barchart.bars.len(), 3);
    assert_eq!(barchart.bars.get(1).unwrap().label, None);

    let barchart = res
        .clone()
        .create_bar_chart(
            1,
            2,
            BarChartBarLabels::FromColumn(0),
            BarChartAxisLabelStrategy::Headers,
            HashSet::from([2]),
        )
        .unwrap();

    assert_eq!(barchart.x_label.unwrap(), "Year");
    assert_eq!(barchart.y_label.unwrap(), "Percentage");
    assert_eq!(barchart.bars.len(), 2);
    assert_eq!(barchart.bars.get(1).unwrap().label.clone().unwrap(), "FEB");

    let barchart = res
        .clone()
        .create_bar_chart(
            1,
            2,
            //BarChartBarLabels::Provided(vec![String::from("One")]),
            BarChartBarLabels::Provided(vec![String::from("One")]),
            BarChartAxisLabelStrategy::Provided {
                x: "Xer".into(),
                y: "Yer".into(),
            },
            HashSet::default(),
        )
        .unwrap();

    assert_eq!(barchart.x_label.unwrap(), "Xer");
    assert_eq!(barchart.y_label.unwrap(), "Yer");
    assert_eq!(barchart.bars.len(), 3);
    assert_eq!(barchart.bars.get(0).unwrap().label.clone().unwrap(), "One");
    assert_eq!(barchart.bars.get(1).unwrap().label, None);

    let barchart = res
        .clone()
        .create_bar_chart(
            1,
            2,
            //BarChartBarLabels::Provided(vec![String::from("One")]),
            BarChartBarLabels::Provided(vec![
                String::from("One"),
                String::from("Two"),
                String::from("Three"),
                String::from("Four"),
                String::from("Five"),
            ]),
            BarChartAxisLabelStrategy::Provided {
                x: "Xer".into(),
                y: "Yer".into(),
            },
            HashSet::default(),
        )
        .unwrap();

    assert_eq!(barchart.bars.len(), 3);
    assert_eq!(
        barchart.bars.get(2).unwrap().label.clone().unwrap(),
        "Three"
    );

    // Non uniform column test
    let barchart = res.clone().create_bar_chart(
        4,
        3,
        BarChartBarLabels::None,
        BarChartAxisLabelStrategy::None,
        HashSet::default(),
    );

    match barchart {
        Ok(_) => panic!("Bar chart false success"),
        Err(e) => {
            assert_eq!(
                e.to_string(),
                "Conversion Error: Cannot convert from non-uniform column"
            );
        }
    }
    //
    // out of range column test
    let barchart = res.clone().create_bar_chart(
        40,
        3,
        BarChartBarLabels::None,
        BarChartAxisLabelStrategy::None,
        HashSet::default(),
    );

    match barchart {
        Ok(_) => panic!("Bar chart false success"),
        Err(e) => {
            assert_eq!(
                e.to_string(),
                "Conversion Error: Bar chart column out of range"
            );
        }
    }

    let barchart = res.clone().create_bar_chart(
        4,
        3,
        BarChartBarLabels::FromColumn(40),
        BarChartAxisLabelStrategy::None,
        HashSet::default(),
    );

    match barchart {
        Ok(_) => panic!("Bar chart false success"),
        Err(e) => {
            assert_eq!(
                e.to_string(),
                "Conversion Error: Bar chart label column out of range"
            );
        }
    }
}

#[test]
fn test_stacked_bar_char() {
    let path: PathBuf = "./dummies/csv/stacked.csv".into();

    let res = SheetBuilder::new(path)
        .labels(HeaderLabelStrategy::ReadLabels)
        .trim(true)
        .types(HeaderTypesStrategy::Infer)
        .build()
        .unwrap();

    let labels = HashSet::from([
        String::from("Soda"),
        String::from("Chocolate"),
        String::from("Coffee"),
        String::from("Ice cream"),
    ]);

    let stacked = res
        .clone()
        .create_stacked_bar_chart(0, [1, 2, 3, 4], StackedBarChartAxisLabelStrategy::None)
        .unwrap();

    assert_eq!(stacked.x_axis, None);
    assert_eq!(stacked.y_axis, None);
    assert!(&stacked
        .bars
        .iter()
        .all(|bar| { bar.fractions.keys().all(|key| labels.contains(key)) }));
    assert_eq!(stacked.bars.get(1).unwrap().point.y, 19.into());
    assert_eq!(stacked.bars.len(), 7);
    assert!(!stacked.has_true_negatives());
    assert!(!stacked.has_true_negatives());
    assert_eq!(&labels, &stacked.labels);

    let stacked = res
        .clone()
        .create_stacked_bar_chart(
            0,
            [1, 4],
            StackedBarChartAxisLabelStrategy::Header("Total".into()),
        )
        .unwrap();

    assert_eq!(stacked.x_axis.unwrap(), "Day of Week");
    assert_eq!(stacked.y_axis.unwrap(), "Total");
    assert_eq!(stacked.bars.get(1).unwrap().point.y, Data::Integer(16));
    assert!(!stacked.bars.get(1).unwrap().is_negative);

    let fraction = HashMap::from([
        (String::from("Coffee"), (7 as f64) / (16 as f64)),
        (String::from("Chocolate"), (6 as f64) / (16 as f64)),
        (String::from("Soda"), (3 as f64) / (16 as f64)),
        (String::from("Ice cream"), (0 as f64) / (16 as f64)),
    ]);
    let stacked = res
        .clone()
        .create_stacked_bar_chart(
            0,
            [1, 2, 3, 4],
            StackedBarChartAxisLabelStrategy::Provided {
                x: "Some X".into(),
                y: "Some Y".into(),
            },
        )
        .unwrap();

    assert_eq!(stacked.x_axis.unwrap(), "Some X");
    assert_eq!(stacked.y_axis.unwrap(), "Some Y");
    assert_eq!(stacked.bars.get(3).unwrap().fractions, fraction);

    let stacked = res
        .clone()
        .create_stacked_bar_chart(
            0,
            [1, 2, 3, 4],
            StackedBarChartAxisLabelStrategy::Provided {
                x: "Some X".into(),
                y: "Some Y".into(),
            },
        )
        .unwrap();

    assert_eq!(stacked.bars.get(1).unwrap().point.x, "Tuesday".into());
    let mut temp = stacked_helper(
        &stacked.bars.get(3).unwrap().point.y,
        &stacked.bars.get(3).unwrap().fractions,
    );
    temp.sort();
    assert_eq!(
        temp,
        vec![
            Data::Integer(0),
            Data::Integer(3),
            Data::Integer(6),
            Data::Integer(7)
        ]
    );

    let mut stacked = res
        .create_stacked_bar_chart(0, [1, 2, 3, 4], StackedBarChartAxisLabelStrategy::None)
        .unwrap();
    // test multiple remove/add of the same section
    assert_eq!(stacked.bars.get(2).unwrap().point.y, 14.into());
    stacked.remove_section(1, "Coffee");
    assert_eq!(stacked.bars.get(1).unwrap().point.y, 9.into());
    stacked.remove_section(1, "Coffee");
    stacked.remove_section(1, "Coffee");
    assert_eq!(stacked.bars.get(1).unwrap().point.y, 9.into());
    stacked.add_section(1, "Coffee");
    assert_eq!(stacked.bars.get(1).unwrap().point.y, 19.into());
    stacked.add_section(1, "Coffee");
    stacked.add_section(1, "Coffee");
    assert_eq!(stacked.bars.get(1).unwrap().point.y, 19.into());
    stacked.remove_section_all("Soda");
    assert_eq!(stacked.bars.get(0).unwrap().point.y, 16.into());
    assert_eq!(stacked.bars.get(5).unwrap().point.y, 11.into());
    stacked.remove_section_all("Soda");
    stacked.remove_section_all("Soda");
    stacked.remove_section_all("Soda");
    assert_eq!(stacked.bars.get(0).unwrap().point.y, 16.into());
    assert_eq!(stacked.bars.get(5).unwrap().point.y, 11.into());
    stacked.add_section_all("Soda");
    assert_eq!(stacked.bars.get(0).unwrap().point.y, 19.into());
    assert_eq!(stacked.bars.get(5).unwrap().point.y, 11.into());
    stacked.add_section_all("Soda");
    stacked.add_section_all("Soda");
    assert_eq!(stacked.bars.get(0).unwrap().point.y, 19.into());
    assert_eq!(stacked.bars.get(5).unwrap().point.y, 11.into());

    let path: PathBuf = "./dummies/csv/stacked_neg.csv".into();

    let res = SheetBuilder::new(path)
        .labels(HeaderLabelStrategy::ReadLabels)
        .trim(true)
        .types(HeaderTypesStrategy::Infer)
        .build()
        .unwrap();

    let stacked = res
        .clone()
        .create_stacked_bar_chart(0, [1, 2, 3, 4], StackedBarChartAxisLabelStrategy::None)
        .unwrap();

    assert!(stacked.has_true_negatives());
    assert!(stacked.has_true_positives());

    assert!(stacked.bars.get(0).unwrap().is_empty());
    assert_eq!(stacked.bars.get(0).unwrap().point.y, Data::Integer(0));
    assert_eq!(
        stacked.bars.get(2).unwrap().point.x,
        Data::Text("Tuesday".into())
    );
    assert_eq!(stacked.bars.get(2).unwrap().point.y, Data::Integer(-10));
    assert_eq!(stacked.bars.get(4).unwrap().point.y, Data::Integer(-18));
    assert_eq!(stacked.bars.len(), 9);
}

fn stacked_helper(total: &Data, fractions: &HashMap<String, f64>) -> Vec<Data> {
    fractions
        .values()
        .into_iter()
        .map(|val| match total {
            Data::Integer(i) => (*val * (*i as f64)) as i32,
            _ => panic!("Stacked Bar Chart test helper panic"),
        })
        .map(|val| Data::Integer(val))
        .collect()
}

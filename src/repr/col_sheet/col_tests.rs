#![cfg(test)]
use super::{
    index_sort_swap, ArrayI32, ArrayText, CellRef, Column, ColumnHeader, ColumnSheet, DataType,
    HeaderLabelStrategy, SheetBuilder, TypesStrategy,
};
use crate::repr::ColumnType;
use proptest::{arbitrary::any, collection, proptest, strategy::Strategy};

const OVERKILL_PROPTEST: bool = false;

fn create_empty() -> ColumnSheet {
    let builder = SheetBuilder::new("./dummies/csv/empty.csv")
        .types(TypesStrategy::Infer)
        .labels(HeaderLabelStrategy::ReadLabels)
        .flexible(false)
        .trim(true);

    ColumnSheet::from_builder(builder).unwrap()
}

fn create_air_csv() -> ColumnSheet {
    let path = "./dummies/csv/air.csv";

    let ct = vec![
        ColumnType::Text,
        ColumnType::Integer,
        ColumnType::Integer,
        ColumnType::Integer,
    ];

    let builder = SheetBuilder::new(path)
        .trim(true)
        .primary(0)
        .types(TypesStrategy::Provided(ct))
        .labels(HeaderLabelStrategy::ReadLabels);

    ColumnSheet::from_builder(builder).unwrap()
}

#[test]
fn flexible() {
    let path = "./dummies/csv/flexible.csv";

    let builder = SheetBuilder::new(path)
        .trim(true)
        .flexible(true)
        .labels(HeaderLabelStrategy::NoLabels)
        .types(TypesStrategy::Infer);

    let sht = ColumnSheet::from_builder(builder).unwrap();
    let actuals = sht.headers().map(|header| header.kind);
    let expected = [DataType::Text, DataType::I32, DataType::I32, DataType::I32];

    for (expected, actual) in expected.into_iter().zip(actuals) {
        assert_eq!(expected, actual);
    }

    assert_eq!(4, sht.width());
    assert_eq!(12, sht.height());

    assert_eq!(Some(CellRef::None), sht.get_cell(2, 7));
    assert_eq!(Some(CellRef::None), sht.get_cell(3, 7));
    assert_eq!(Some(CellRef::None), sht.get_cell(3, 6));
}

#[test]
fn infer() {
    let path = "./dummies/csv/infer.csv";

    let builder = SheetBuilder::new(path)
        .trim(true)
        .types(TypesStrategy::Infer)
        .labels(HeaderLabelStrategy::ReadLabels);

    let sht = ColumnSheet::from_builder(builder).unwrap();
    let actuals = sht.headers().map(|header| header.kind);
    let expected = [
        DataType::Text,
        DataType::I32,
        DataType::F32,
        DataType::Text,
        DataType::Bool,
        DataType::Text,
    ];

    for (expected, actual) in expected.into_iter().zip(actuals) {
        assert_eq!(expected, actual);
    }
}

#[test]
fn headerless() {
    let path = "./dummies/csv/headless.csv";
    let ct = vec![
        ColumnType::Text,
        ColumnType::Integer,
        ColumnType::Integer,
        ColumnType::Integer,
    ];

    let builder = SheetBuilder::new(path)
        .trim(true)
        .primary(0)
        .types(TypesStrategy::Provided(ct))
        .labels(HeaderLabelStrategy::NoLabels);

    let sht = ColumnSheet::from_builder(builder).unwrap();
    let headers = sht.headers();
    let expected = [
        ColumnHeader {
            kind: DataType::Text,
            header: None,
        },
        ColumnHeader {
            kind: DataType::I32,
            header: None,
        },
        ColumnHeader {
            kind: DataType::I32,
            header: None,
        },
    ];

    for (expected, actual) in expected.into_iter().zip(headers) {
        assert_eq!(expected, actual);
    }
}

#[test]
fn test_cells() {
    let empty = create_empty();
    assert!(empty.get_cell(0, 0).is_none());

    let mut sht = create_air_csv();
    assert!(sht.get_cell(100, 100).is_none());
    assert_eq!(Some(CellRef::I32(420)), sht.get_cell(2, 4));

    assert!(sht.set_cell("aa", 2, 4).is_err());
    assert!(sht.set_cell("69", 2, 4).is_ok());
    assert_eq!(Some(CellRef::I32(69)), sht.get_cell(2, 4));
}

#[test]
fn test_empty() {
    let mut empty = create_empty();

    // Get
    assert_eq!(None, empty.get_primary());
    assert!(empty.set_primary(0).is_err());
    assert!(empty.get_row(0).is_none());
    assert!(empty.get_col(0).is_none());
    assert!(empty.is_empty());
    assert!(empty.true_is_empty());
    assert!(empty.headers().next().is_none());

    // Insert row invalid
    let row = vec!["1", "2", "3", "4"].into_iter();
    assert!(empty.insert_row(row.clone(), 1).is_err());
    assert_eq!(None, empty.get_primary());
    assert!(empty.get_row(0).is_none());
    assert!(empty.is_empty());
    assert!(empty.true_is_empty());

    // Insert row valid
    assert!(empty.insert_row(row, 0).is_ok());
    assert_eq!(Some(0), empty.get_primary());
    assert!(empty.set_primary(100).is_err());
    assert!(empty.set_primary(1).is_ok());
    assert_eq!(Some(1), empty.get_primary());
    assert!(empty.get_row(0).is_some());
    assert!(!empty.is_empty());
    assert!(!empty.true_is_empty());
    assert_eq!(empty.width(), 4);
    assert_eq!(empty.height(), 1);
    assert_eq!(
        empty.headers().next().unwrap(),
        ColumnHeader {
            kind: DataType::I32,
            header: None
        }
    );

    let column = ArrayI32::from_iterator([9].into_iter());
    let column = Box::new(column);
    // Insert Col invalid
    assert!(empty.insert_col(column.clone(), 9).is_err());
    assert_eq!(empty.width(), 4);

    // Insert Col valid
    assert!(empty.insert_col(column.clone(), 4).is_ok());
    assert_eq!(empty.width(), 5);
    assert_eq!(Some(1), empty.get_primary());
    assert!(empty.insert_col(column, 1).is_ok());
    assert_eq!(empty.width(), 6);
    assert_eq!(Some(2), empty.get_primary());

    // Get col
    assert!(empty.get_col(100).is_none());
    assert!(empty.get_col(1).is_some());

    // Del col invalid
    assert!(empty.remove_col(empty.width() + 1).is_err());

    // Del col valid
    assert!(empty.remove_col(5).is_ok());
    assert_eq!(Some(2), empty.get_primary());
    assert!(empty.remove_col(1).is_ok());
    assert_eq!(Some(1), empty.get_primary());

    // Del row invalid
    assert!(empty.remove_row(1).is_err());
    assert_eq!(Some(1), empty.get_primary());
    assert!(empty.get_row(0).is_some());
    assert!(!empty.is_empty());

    // Del row valid
    assert!(empty.remove_row(0).is_ok());
    assert_eq!(Some(1), empty.get_primary());
    assert!(empty.is_empty());
    assert!(!empty.true_is_empty());
    assert_eq!(empty.width(), 4);
    assert!(empty.get_row(0).is_none());
    assert!(empty.headers().next().is_some());
}

#[test]
fn test_primary_key() {
    let mut sht = create_air_csv();

    // Initial Primary
    assert_eq!(Some(0), sht.get_primary());

    // Invalid primary
    assert!(sht.set_primary(133333).is_err());
    assert_eq!(Some(0), sht.get_primary());

    // Valid primary
    assert!(sht.set_primary(2).is_ok());
    assert_eq!(Some(2), sht.get_primary());

    // After insertion
    let valid = ArrayI32::from_iterator((24..36).into_iter());
    let valid = Box::new(valid);
    assert!(sht.insert_col(valid, sht.get_primary().unwrap()).is_ok());
    assert_eq!(Some(3), sht.get_primary());

    // Sorting
    let mut sht = create_air_csv();
    let _ = sht.set_primary(2);
    sht.sort_col_by(3);
    assert_eq!(Some(1), sht.get_primary());

    // Swap Self
    let mut sht = create_air_csv();
    assert!(sht.swap_cols(1, 1).is_ok());
    assert_eq!(Some(0), sht.get_primary());
    assert!(sht.swap_cols(0, 0).is_ok());
    assert_eq!(Some(0), sht.get_primary());
    // Swap Other
    assert!(sht.swap_cols(1, 3).is_ok());
    assert_eq!(Some(0), sht.get_primary());
    assert!(sht.swap_cols(0, 3).is_ok());
    assert_eq!(Some(3), sht.get_primary());

    // Del
    let _ = sht.set_primary(2);
    assert!(sht.remove_col(3).is_ok());
    assert_eq!(Some(2), sht.get_primary());
    assert!(sht.remove_col(1).is_ok());
    assert_eq!(Some(1), sht.get_primary());
    assert!(sht.remove_col(1).is_ok());
    assert_eq!(Some(0), sht.get_primary());

    // Nuke
    sht.remove_all_cols();
    assert_eq!(None, sht.get_primary());

    // Clear primary
    let mut sht = create_air_csv();
    sht.clear_primary();
    assert_eq!(None, sht.get_primary());
}

#[test]
fn test_cols() {
    let mut sht = create_air_csv();

    assert!(sht.get_col(16).is_none());

    let months = sht.get_col(0).unwrap();
    let two = sht.get_col(2).unwrap();

    let months = months.as_any().downcast_ref::<ArrayText>().unwrap();

    assert_eq!(12, months.len());
    assert_eq!(DataType::Text, months.kind());
    assert_eq!(DataType::I32, two.kind());
    assert_eq!("APR", months.get(3).unwrap());

    // Insert Invalid
    let invalid = ArrayI32::from_iterator((24..66).into_iter());
    let invalid = Box::new(invalid);
    assert!(sht.insert_col(invalid, 10).is_err());

    // Insert valid
    let valid = ArrayI32::from_iterator((24..36).into_iter());
    let valid = Box::new(valid);
    assert!(sht.insert_col(valid, 3).is_ok());

    // Check insertion
    let valid = sht
        .get_col(3)
        .and_then(|col| col.as_any().downcast_ref::<ArrayI32>())
        .unwrap();
    let prev = sht
        .get_col(4)
        .and_then(|col| col.as_any().downcast_ref::<ArrayI32>())
        .unwrap();
    assert_eq!(Some(25), valid.get(1));
    assert_eq!(Some(391), prev.get(1));

    // Sorting
    let mut sht = create_air_csv();
    sht.sort_col_by(3);
    let valid = sht
        .get_col(1)
        .and_then(|col| col.as_any().downcast_ref::<ArrayI32>())
        .unwrap();
    let prev = sht
        .get_col(3)
        .and_then(|col| col.as_any().downcast_ref::<ArrayText>())
        .unwrap();
    assert_eq!(Some(342), valid.get(1));
    assert_eq!(Some("FEB".into()), prev.get(1));

    // Swap Invalid
    assert!(sht.swap_cols(100, 0).is_err());
    assert!(sht.swap_cols(1, 100).is_err());
    // Swap Self
    assert!(sht.swap_cols(1, 1).is_ok());
    let valid = sht
        .get_col(1)
        .and_then(|col| col.as_any().downcast_ref::<ArrayI32>())
        .unwrap();
    assert_eq!(Some(342), valid.get(1));
    // Swap Other
    assert!(sht.swap_cols(1, 3).is_ok());
    let valid = sht
        .get_col(1)
        .and_then(|col| col.as_any().downcast_ref::<ArrayText>())
        .unwrap();
    let prev = sht
        .get_col(3)
        .and_then(|col| col.as_any().downcast_ref::<ArrayI32>())
        .unwrap();
    assert_eq!(Some("FEB".into()), valid.get(1));
    assert_eq!(Some(342), prev.get(1));

    // Clear Cell
    assert!(sht.clear_cell(100, 0).is_err());
    assert!(sht.clear_cell(0, 100).is_err());
    assert!(sht.clear_cell(1, 1).is_ok());
    let valid = sht
        .get_col(1)
        .and_then(|col| col.as_any().downcast_ref::<ArrayText>())
        .unwrap();
    assert_eq!(None, valid.get(1));

    // Clear Col
    assert!(sht.clear_col(100).is_err());
    assert!(sht.clear_col(1).is_ok());
    let valid = sht
        .get_col(1)
        .and_then(|col| col.as_any().downcast_ref::<ArrayText>())
        .unwrap();
    assert!(valid.iter().all(|cell| cell.is_none()));

    // Del
    assert!(sht.remove_col(100).is_err());
    assert!(sht.remove_col(1).is_ok());
    assert_eq!(3, sht.width());
    let valid = sht
        .get_col(1)
        .and_then(|col| col.as_any().downcast_ref::<ArrayI32>())
        .unwrap();
    assert_eq!(Some(391), valid.get(1));

    // Nuke
    sht.remove_all_cols();
    assert!(sht.is_empty());
    assert!(sht.true_is_empty());
    assert_eq!(0, sht.width());
    assert_eq!(0, sht.height());
}

#[test]
fn test_rows() {
    let mut sht = create_air_csv();

    // Insert Invalid
    let invalid = vec!["SOME", "0", "1", "2"].into_iter();
    assert!(sht.insert_row(invalid, 100).is_err());

    // Insert Valid
    let valid = vec!["SOME", "0", "1", "2"].into_iter();
    assert!(sht.insert_row(valid, 12).is_ok());
    let valid = vec!["SOME", "0", "1", "2"].into_iter();
    assert!(sht.insert_row(valid, 5).is_ok());

    // Check insertion
    assert!(sht.get_row(15).is_none());
    let row = sht.get_row(5).unwrap();
    assert_eq!(
        vec![
            CellRef::Text("SOME"),
            CellRef::I32(0),
            CellRef::I32(1),
            CellRef::I32(2),
        ],
        row
    );

    let row = sht.get_row(0).unwrap();
    assert_eq!(
        vec![
            CellRef::Text("JAN"),
            CellRef::I32(340),
            CellRef::I32(360),
            CellRef::I32(417),
        ],
        row
    );

    // Sorting
    let mut sht = create_air_csv();
    sht.sort_row_by(1);

    assert_eq!(
        vec![
            CellRef::Text("NOV"),
            CellRef::I32(310),
            CellRef::I32(362),
            CellRef::I32(390),
        ],
        sht.get_row(0).unwrap()
    );

    assert_eq!(
        vec![
            CellRef::Text("OCT"),
            CellRef::I32(359),
            CellRef::I32(407),
            CellRef::I32(461),
        ],
        sht.get_row(5).unwrap()
    );

    // Swap Invalid
    let mut sht = create_air_csv();
    assert!(sht.swap_rows(100, 0).is_err());
    assert!(sht.swap_rows(0, 100).is_err());
    // Swap self
    assert!(sht.swap_rows(1, 1).is_ok());
    assert_eq!(
        vec![
            CellRef::Text("FEB"),
            CellRef::I32(318),
            CellRef::I32(342),
            CellRef::I32(391),
        ],
        sht.get_row(1).unwrap()
    );
    // Swap Other
    assert!(sht.swap_rows(1, 3).is_ok());
    assert_eq!(
        vec![
            CellRef::Text("APR"),
            CellRef::I32(348),
            CellRef::I32(396),
            CellRef::I32(461),
        ],
        sht.get_row(1).unwrap()
    );

    // Clear Row
    assert!(sht.clear_row(100).is_err());
    assert!(sht.clear_row(1).is_ok());
    assert_eq!(
        vec![CellRef::None, CellRef::None, CellRef::None, CellRef::None,],
        sht.get_row(1).unwrap()
    );

    // Del
    let mut sht = create_air_csv();
    assert!(sht.remove_row(100).is_err());
    assert!(sht.remove_row(5).is_ok());
    assert_eq!(11, sht.height());
    assert_eq!(
        vec![
            CellRef::Text("JUL"),
            CellRef::I32(491),
            CellRef::I32(548),
            CellRef::I32(622),
        ],
        sht.get_row(5).unwrap()
    );

    // Nuke
    sht.remove_all_rows();
    assert!(sht.is_empty());
    assert!(!sht.true_is_empty());
    assert_eq!(4, sht.width());
}

#[test]
fn test_headers() {
    let empty = create_empty();
    assert_eq!(0, empty.headers().len());

    let mut sht = create_air_csv();
    let actuals = sht.headers();
    let headers = [
        ColumnHeader {
            kind: DataType::Text,
            header: Some("Month"),
        },
        ColumnHeader {
            kind: DataType::I32,
            header: Some("1958"),
        },
        ColumnHeader {
            kind: DataType::I32,
            header: Some("1959"),
        },
        ColumnHeader {
            kind: DataType::I32,
            header: Some("1960"),
        },
    ];

    assert_eq!(4, actuals.len());

    for (expected, actual) in headers.into_iter().zip(actuals) {
        assert_eq!(expected, actual);
    }

    assert!(sht.set_col_header(100, "Failure").is_err());
    assert!(sht.set_col_header(1, "Success").is_ok());

    let headers = [
        ColumnHeader {
            kind: DataType::Text,
            header: Some("Month"),
        },
        ColumnHeader {
            kind: DataType::I32,
            header: Some("Success"),
        },
        ColumnHeader {
            kind: DataType::I32,
            header: Some("1959"),
        },
        ColumnHeader {
            kind: DataType::I32,
            header: Some("1960"),
        },
    ];

    for (expected, actual) in headers.into_iter().zip(sht.headers()) {
        assert_eq!(expected, actual);
    }
}

fn test_vec() -> impl Strategy<Value = Vec<isize>> {
    let max = if OVERKILL_PROPTEST { 1_000_000 } else { 1000 };
    collection::vec(any::<isize>(), 0..max)
}

proptest! {
    #[test]
    fn test_index_sort_swap(vec in test_vec()) {
        let mut sorted = vec.clone();
        sorted.sort();

        let mut vec = vec;

        let mut indices = (0..vec.len()).collect::<Vec<usize>>();

        indices.sort_by(|x, y| vec[*x].cmp(&vec[*y]));

        index_sort_swap(&mut indices);

        for (pos, elem) in indices.into_iter().enumerate(){
            vec.swap(pos, elem);
        }


        assert_eq!(sorted, vec);
    }

    #[test]
    fn test_index_sort_swap_rev(vec in test_vec()) {
        let mut sorted = vec.clone();
        sorted.sort();
        sorted.reverse();


        let mut vec = vec;

        let mut indices = (0..vec.len()).collect::<Vec<usize>>();

        indices.sort_by(|y, x| vec[*x].cmp(&vec[*y]));

        index_sort_swap(&mut indices);

        for (pos, elem) in indices.into_iter().enumerate(){
            vec.swap(pos, elem);
        }


        assert_eq!(sorted, vec);
    }
}

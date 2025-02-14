# Sheet

# 1. Introduction

The `Sheet` is the primary representation model in our project. It serves as the central structure from which other models (charts, tables) can be derived. For tables, the sheet will be the model returned when conversion is requested. As such sheets can also be thought of as tables

## 1.1 Invariants

1. **Unique Row IDs**: No two rows within a single sheet share the same integer ID
2. **Consistent Row Length**: All rows within a sheet have the same length
3. **Header-Row Alignment**: The number of headers is equal to the length of the longest row in a sheet
4. **Column Types**:
    - Each column has a unique `ColumnType` enum variant
    - `ColumnType` variants ideally mirror variants of `Data`
    - `ColumnType::None` columns can contain a mixture of other value types

# 2. Components of a Sheet

A sheet has the following components;

1. **Headers**: A vector of `ColumnHeader` objects
2. **Rows**: A vector of `Row` objects
3. **ID Counter**: An integer for assigning unique IDs to each row within the sheet
4. **Primary Column Tracker**: Integer for keeping track of the current primary column of the sheet

## 2.1 Cell

The `Cell` type represents a single cell within a row or column in a sheet.

### Fields

| Field | Type | Description |
| --- | --- | --- |
| id | Integer | Unique identifier for the cell |
| data | Data | The content of the cell |

## 2.2 Row

A `Row` represents a sequence of `Cell`s within a `Sheet`.

### Fields

| Field | Type | Description |
| --- | --- | --- |
| id_counter | Integer | For assigning each cell a unique integer ID |
| primary_key | Integer | Keeps track of the current primary cell |

## 2.3 ColumnHeader

The `ColumnHeader` type represents a header of a sheet.

### Components

| Component | Type | Description |
| --- | --- | --- |
| label | String | The header text |
| kind | ColumnType | Represents the type of values stored in the column |

## 2.4 Data

The `Data` type is an enum representing the data stored within cells of a `Sheet`.

### General Characteristics

- All variants except `None` contain their associated values
- Implements comparisons (equality and complete ordering) and hashing interfaces
- Supports conversions from all supported value types
- The `Display` trait is implemented for all variants:
    - Non-`None` variants return a string view of the contained value
    - `None` variant returns the string value `<None>`

### Variants

| Variant | Description |
| --- | --- |
| Boolean(b) | Represents the boolean b |
| Float(f) | Represents the 32-bit floating point number f |
| Integer(i) | Represents the 32-bit signed integer i |
| Number(n) | Represents the maximum bit supported signed integer n |
| Text(t) | Represents the string t |
| None | Represents the empty data value (default variant) |

### Future Extensions

Potential future extensions of this type may include:

- An error variant
- A date variant

## 2.5 ColumnType

The `ColumnType` enum represents the value type of `Data` stored within a column.

### Characteristics

- Variants mirror those of `Data`, but exclude the actual associated value
- Implements conversion from `Data`
- `Data::None` is valid for any `ColumnType`, as it represents an empty cell

### Variants

| Variant | Description |
| --- | --- |
| Text | A text column |
| Integer | A 32-bit signed integer column |
| Number | A maximum bit supported signed integer column |
| Float | A 32-bit floating point number column |
| Boolean | A boolean column |
| None | A column with no unique value types (default variant) |

### Note on `None` Variant

- Supports all variants of `Data`
- Serves as the default variant for `ColumnType`

# 3. Sheet Construction

## 3.1 From CSV

Sheet construction from CSV is handled by the `Config` within the builders module.shsh

### Capabilities

- Handles CSV-like data formats where the `,` can be replaced with another character (e.g., whitespace)``
- Only utf-8 encoded files are currently supported
- Future builders for other data formats should implement a builder interface for uniformity
- Note: A default interface might not be possible for builders that read data from a file, as they require a file path

### Config Fields

| Field | Default | Description |
| --- | --- | --- |
| path | No Default | File path to read from |
| primary | 0 | Integer ID for the primary column of the sheet |
| trim | false | If true, whitespaces are trimmed from headers and fields during parsing |
| flexible | false | If true, handles uneven rows; if false, returns an error for uneven rows |
| label_strategy | HeaderLabelStrategy::No labels | Enum representing how headers should be handled |
| type_strategy | TypesStrategy::None | Enum representing how column types should be handled |
| delimiter | , | Character delimiting each field of the data |

### Additional Notes

- When `flexible` is true, all rows in the resulting sheet will have the same length as the longest row. Shorter rows are padded with cells having the `Data::default` value.
- `label_strategy`: By default, assumes no headers should be parsed. Header lengths are either trimmed (when too long) or padded with 'empty' headers to match the longest row length.
- `type_strategy`: By default, columns have no unique type.
- Changing `delimiter` to `\\t` enables support for TSV files.

# 4. Utility Types

## 4.1 HeaderLabelStrategy

The `HeaderLabelStrategy` enum determines how headers for a `Sheet` should be handled during building.

### Variants

| Variant | Description |
| --- | --- |
| ReadLabels | Headers are assumed to be the first sequence of the data and are read as such |
| Provided(h) | Headers are provided as a string vector h |
| NoLabels | The Sheet is created without header labels, i.e an empty label string. This is the default variant |

## 4.2 TypesStrategy

The `TypesStrategy` enum determines how the `ColumnType` for each column is determined.

### Variants

| Variant | Description |
| --- | --- |
| Infer | Types are inferred from the data. If a column contains only one Data type throughout, the appropriate ColumnType is assigned to it. |
| Provided(t) | Types are provided as a vector of ColumnType, t |
| None | All columns are assigned the ColumnType::None type |

## 4.3 Error

The `Error` enum represents the various errors which may occur when the module is used. All variants contain fields describing where the error occurred

### Variants

| Variant | Description |
| --- | --- |
| CSVReaderError(error) | Represents errors sourcing from the underlying CSV reader. |
| InvalidPrimaryKey(string) | An out of range primary key was encountered. Primary keys are expected to be in the range [0, len(row))  |
| InvalidColumnType(string) | There’s a mismatch between the stated type of a column and the value contained in one of its cells.  |
| InvalidColumnLength(string) | A row has too many or too few columns. |
| InvalidColumnSort(string) | There was an attempt to sort a non-uniform type column |
| LineGraphConversionError(string) | An error occurred while converting a Sheet into a LineGraph |
| LineGraphError(error) | Wrapper around errors thrown by LineGraph |
| TransposeError(string) | An error occurring during a transpose operation |

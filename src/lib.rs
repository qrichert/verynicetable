//! Number one table.
//!
//! Very basic and lightweight table builder to print tabular data.
//!
//! The struct of interest is [`Table`], which is a builder that
//! implements `Display`.
//!
//! # Examples
//!
//! ```
//! use std::fmt::Alignment::{Left, Right};
//! use verynicetable::Table;
//!
//! let ports = vec![
//!     vec!["rapportd", "449", "Quentin", "*:61165"],
//!     vec!["Python", "22396", "Quentin", "*:8000"],
//!     vec!["foo", "108", "root", "*:1337"],
//!     vec!["rustrover", "30928", "Quentin", "127.0.0.1:63342"],
//!     vec!["Transmiss", "94671", "Quentin", "*:51413"],
//!     vec!["Transmiss", "94671", "Quentin", "*:51413"],
//! ];
//!
//! let table = Table::new()
//!     .headers(&["COMMAND", "PID", "USER", "HOST:PORTS"])
//!     .alignments(&[Left, Right, Left, Right])
//!     .data(&ports)
//!     .max_rows(5)
//!     .to_string();
//!
//! assert_eq!(
//!     table,
//!     "\
//! COMMAND      PID  USER          HOST:PORTS
//! rapportd     449  Quentin          *:61165
//! Python     22396  Quentin           *:8000
//! ...          ...  ...                  ...
//! rustrover  30928  Quentin  127.0.0.1:63342
//! Transmiss  94671  Quentin          *:51413
//! Transmiss  94671  Quentin          *:51413
//! "
//! );
//! ```

use std::{fmt, fmt::Write, iter};

const DEFAULT_COLUMN_SEPARATOR: &str = "  ";

/// Ready-to-render `Table` blueprint with checks and conversions made.
///
/// `Table` can hold "invalid" state during the build process; you can't
/// possibly set everything at once. And also `alignments`, while being
/// required during rendering, can be omitted in the builder as they
/// have defaults we can use.
///
/// `TableBlueprint` on the other hand, is ready-to-render. All required
/// fields are ensured to be set, and it holds additional context for
/// drawing (e.g., `columns_width`).
struct TableBlueprint<'a> {
    headers: Vec<&'a str>,
    alignments: Vec<fmt::Alignment>,
    data: Vec<Vec<&'a str>>,
    columns_width: Vec<usize>,
    column_separator: &'a str,
}

/// `Table` builder.
///
/// The methods of interest are [`new()`](Self::new),
/// [`headers()`](Self::headers), [`alignments()`](Self::alignments),
/// [`data()`](Self::data) and [`max_rows()`](Self::max_rows).
///
/// To render the table, use the `Display` trait's method `to_string()`.
///
/// # Implementation Details
///
/// `Table` can possibly hold intermediary "invalid" state during
/// building. Which is perfectly normal for a builder.
///
/// During rendering, a `TableBuilder` (private) is first created
/// through `make_table_blueprint()`. `TableBuilder` then drives the
/// printing of the table to the terminal.
///
/// Contrary to `Table`, `TableBuilder` can only hold valid
/// ready-to-render state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Table<'a> {
    headers: Option<Vec<&'a str>>,
    alignments: Option<&'a [fmt::Alignment]>,
    data: Option<Vec<Vec<&'a str>>>,
    max_rows: Option<usize>,
    column_separator: Option<&'a str>,
}

impl<'a> Default for Table<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Table<'a> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            headers: None,
            alignments: None,
            data: None,
            max_rows: None,
            column_separator: None,
        }
    }

    pub fn headers(&mut self, headers: &'a [impl AsRef<str>]) -> &mut Self {
        let headers: Vec<&str> = headers.iter().map(AsRef::as_ref).collect();
        self.headers = Some(headers);
        self
    }

    pub fn alignments(&mut self, alignments: &'a [fmt::Alignment]) -> &mut Self {
        self.alignments = Some(alignments);
        self
    }

    pub fn data(&mut self, data: &'a [Vec<impl AsRef<str>>]) -> &mut Self {
        let data: Vec<Vec<&str>> = data
            .iter()
            .map(|row| row.iter().map(AsRef::as_ref).collect())
            .collect();
        self.data = Some(data);
        self
    }

    pub fn max_rows(&mut self, max_rows: usize) -> &mut Self {
        self.max_rows = Some(max_rows);
        self
    }

    pub fn column_separator(&mut self, separator: &'a impl AsRef<str>) -> &mut Self {
        self.column_separator = Some(separator.as_ref());
        self
    }

    fn render(&self) -> String {
        let table = self.make_table_blueprint();

        if table.data.is_empty() {
            return format!("{}\n", table.headers.join("  "));
        }

        let mut output = String::new();

        let mut render_row = |row: &Vec<&str>| {
            for (i, cell) in row.iter().enumerate() {
                let width = table.columns_width[i];
                let alignment = table.alignments[i];

                let is_last_column = i == table.headers.len() - 1;

                let _ = match alignment {
                    fmt::Alignment::Left if is_last_column => write!(output, "{cell}"),
                    fmt::Alignment::Left => write!(output, "{cell:<width$}"),
                    fmt::Alignment::Right => write!(output, "{cell:>width$}"),
                    fmt::Alignment::Center => write!(output, "{cell:^width$}"),
                };

                if is_last_column {
                    output.push('\n');
                } else {
                    output.push_str(table.column_separator);
                }
            }
        };

        if !table.headers.iter().all(|header| header.is_empty()) {
            render_row(&table.headers);
        }

        for row in table.data {
            render_row(&row);
        }

        output
    }

    fn make_table_blueprint(&self) -> TableBlueprint {
        let nb_cols = self.determine_nb_columns();

        let headers = self.get_headers_or_default(nb_cols);
        let alignments = self.get_alignments_or_default(nb_cols);
        let mut data = self.data.as_ref().expect("data is required").to_owned();

        Self::ensure_data_consistency(&headers, &alignments, &data);

        if let Some(max_rows) = self.max_rows {
            #[cfg(not(tarpaulin_include))] // Wrongly marked uncovered.
            {
                data = Self::apply_max_rows(data, max_rows, nb_cols);
            }
        }

        let columns_width = Self::determine_columns_width(&headers, &data);
        let column_separator = self.column_separator.unwrap_or(DEFAULT_COLUMN_SEPARATOR);

        TableBlueprint {
            headers,
            alignments,
            data,
            columns_width,
            column_separator,
        }
    }

    #[cfg(not(tarpaulin_include))] // Wrongly marked uncovered.
    fn determine_nb_columns(&self) -> usize {
        if let Some(headers) = self.headers.as_ref() {
            return headers.len();
        }
        if let Some(data) = self.data.as_ref() {
            if !data.is_empty() {
                return data[0].len();
            }
        }
        panic!("headers and data cannot both be empty");
    }

    fn get_headers_or_default(&self, nb_cols: usize) -> Vec<&str> {
        match self.headers.as_ref() {
            Some(headers) => headers.to_owned(),
            // This may look a bit hacky (it is), but it plays nicely
            // with the overall logic (`Option` would make the code too
            // convoluted). Moreover, it has the added benefit of
            // handling the special case where the user does it himself.
            None => [""].repeat(nb_cols),
        }
    }

    fn get_alignments_or_default(&self, nb_cols: usize) -> Vec<fmt::Alignment> {
        match self.alignments {
            Some(alignments) => alignments.to_vec(),
            None => [fmt::Alignment::Left].repeat(nb_cols),
        }
    }

    /// Ensure data is consistent.
    ///
    /// "Consistent" essentially means the number of headers matches
    /// the number of alignment properties, and the number of columns
    /// in the data.
    fn ensure_data_consistency(
        headers: &[&str],
        alignments: &[fmt::Alignment],
        data: &[Vec<&str>],
    ) {
        assert_eq!(
            headers.len(),
            alignments.len(),
            "number of headers must match alignments"
        );
        assert!(
            data.iter().all(|row| row.len() == headers.len()),
            "number of headers must match columns in data"
        );
    }

    fn apply_max_rows(mut data: Vec<Vec<&str>>, max_rows: usize, nb_cols: usize) -> Vec<Vec<&str>> {
        if data.len() <= max_rows {
            return data; // no-op.
        }

        if max_rows == 0 {
            return vec![["..."].repeat(nb_cols)];
        }

        if max_rows == 1 {
            data.truncate(1);
            return data
                .into_iter()
                .chain(iter::once(["..."].repeat(nb_cols)))
                .collect();
        }

        // Bias towards more tail elements.
        let nb_head = max_rows / 2;
        let nb_tail = max_rows - nb_head;

        let tail = data.split_off(data.len() - nb_tail);
        data.truncate(nb_head);
        let head = data;

        head.into_iter()
            .chain(iter::once(["..."].repeat(nb_cols)))
            .chain(tail)
            .collect()
    }

    /// Determine the width of each column.
    ///
    /// The width of a column is the number of characters in the longest
    /// value held in the column (including header).
    fn determine_columns_width(headers: &[&str], data: &[Vec<&str>]) -> Vec<usize> {
        let mut cols_width = vec![0; headers.len()];
        for i in 0..headers.len() {
            let column_values: Vec<&str> = data.iter().map(|x| x[i]).collect();
            let max_width = Self::width_of_longest_value_in_column(headers[i], &column_values);
            cols_width[i] = max_width;
        }
        cols_width
    }

    fn width_of_longest_value_in_column(header: &str, column_values: &[&str]) -> usize {
        let header = iter::once(&header);
        let column_values = column_values.iter();

        header
            .chain(column_values)
            .map(|x| x.chars().count())
            .max()
            .expect("iterator cannot be empty because header is required")
    }
}

impl fmt::Display for Table<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let output = self.render();
        write!(f, "{output}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_default_builder() {
        assert_eq!(Table::new(), Table::default());
    }

    #[test]
    fn table_regular() {
        let table = Table::new()
            .headers(&["SHORT", "WITH SPACE", "LAST COLUMN"])
            .alignments(&[
                fmt::Alignment::Left,
                fmt::Alignment::Left,
                fmt::Alignment::Left,
            ])
            .data(&[
                vec![
                    "Value larger than header",
                    "Column name has space",
                    "No trailing whitespace",
                ],
                vec!["---", "---", "---"],
            ])
            .to_string();

        println!("{table}");
        assert_eq!(
            table,
            "\
SHORT                     WITH SPACE             LAST COLUMN
Value larger than header  Column name has space  No trailing whitespace
---                       ---                    ---
"
        );
    }

    #[test]
    fn table_all_empty_headers_not_rendered() {
        let table = Table::new()
            .headers(&["", ""])
            .data(&[vec!["---", "----------------"]])
            .to_string();

        println!("{table}");
        assert_eq!(
            table,
            "\
---  ----------------
"
        );
    }

    #[test]
    fn table_some_empty_headers_all_rendered() {
        let table = Table::new()
            .headers(&["", "-"])
            .data(&[vec!["---", "----------------"]])
            .to_string();

        println!("{table}");
        assert_eq!(
            table,
            r"     -
---  ----------------
"
        );
    }

    #[test]
    fn table_default_headers() {
        let table = Table::new()
            .data(&[vec!["---", "----------------"]])
            .to_string();

        println!("{table}");
        assert_eq!(
            table,
            "\
---  ----------------
"
        );
    }

    #[test]
    fn table_headers_alignment() {
        let table = Table::new()
            .headers(&["ALIGN-LEFT", "ALIGN-CENTER", "ALIGN-RIGHT"])
            .alignments(&[
                fmt::Alignment::Left,
                fmt::Alignment::Center,
                fmt::Alignment::Right,
            ])
            .data(&[
                vec![
                    "Header is aligned Left",
                    "Header is aligned Center",
                    "Header is aligned Right",
                ],
                vec!["---", "---", "---"],
            ])
            .to_string();

        println!("{table}");
        assert_eq!(
            table,
            "\
ALIGN-LEFT                    ALIGN-CENTER                    ALIGN-RIGHT
Header is aligned Left  Header is aligned Center  Header is aligned Right
---                               ---                                 ---
"
        );
    }

    #[test]
    fn table_values_alignment() {
        let table = Table::new()
            .headers(&["ALIGN-LEFT", "ALIGN-CENTER", "ALIGN-RIGHT"])
            .alignments(&[
                fmt::Alignment::Left,
                fmt::Alignment::Center,
                fmt::Alignment::Right,
            ])
            .data(&[vec!["Left", "Center", "Right"], vec!["---", "---", "---"]])
            .to_string();

        println!("{table}");
        assert_eq!(
            table,
            "\
ALIGN-LEFT  ALIGN-CENTER  ALIGN-RIGHT
Left           Center           Right
---             ---               ---
"
        );
    }

    #[test]
    fn table_default_alignments() {
        let table = Table::new()
            .headers(&["VALUE LEFT", "COLUMN LEFT"])
            .data(&[vec!["---", "----------------"]])
            .to_string();

        println!("{table}");
        assert_eq!(
            table,
            "\
VALUE LEFT  COLUMN LEFT
---         ----------------
"
        );
    }

    #[test]
    fn table_default_headers_and_alignments() {
        let table = Table::new()
            .data(&[
                vec!["---", "----------------"],
                vec!["----------------", "---"],
            ])
            .to_string();

        println!("{table}");
        assert_eq!(
            table,
            "\
---               ----------------
----------------  ---
"
        );
    }

    #[test]
    fn table_with_empty_data() {
        let table = Table::new()
            .headers(&["SHORT", "WITH SPACE", "LAST COLUMN"])
            .alignments(&[
                fmt::Alignment::Left,
                fmt::Alignment::Left,
                fmt::Alignment::Left,
            ])
            .data(&[] as &[Vec<&str>; 0])
            .to_string();

        println!("{table}");
        assert_eq!(
            table,
            "\
SHORT  WITH SPACE  LAST COLUMN
"
        );
    }

    #[test]
    fn table_completely_empty() {
        let table = Table::new()
            .headers(&[] as &[&str; 0])
            .alignments(&[])
            .data(&[] as &[Vec<&str>; 0])
            .to_string();

        println!("{table}");
        assert_eq!(table, "\n");
    }

    #[test]
    #[should_panic(expected = "headers and data cannot both be empty")]
    fn table_error_completely_empty_with_default_headers() {
        let table = Table::new()
            .alignments(&[])
            .data(&[] as &[Vec<&str>; 0])
            .to_string();

        println!("{table}");
        assert_eq!(table, "\n");
    }

    #[test]
    fn table_completely_empty_with_default_alignments() {
        let table = Table::new()
            .headers(&[] as &[&str; 0])
            .data(&[] as &[Vec<&str>; 0])
            .to_string();

        println!("{table}");
        assert_eq!(table, "\n");
    }

    #[test]
    #[should_panic(expected = "headers and data cannot both be empty")]
    fn table_error_completely_empty_with_default_headers_and_alignments() {
        let table = Table::new().data(&[] as &[Vec<&str>; 0]).to_string();

        println!("{table}");
        assert_eq!(table, "\n");
    }

    #[test]
    #[should_panic(expected = "number of headers must match alignments")]
    fn table_error_nb_headers_neq_nb_alignments() {
        Table::new()
            .headers(&["COLUMN 1", "COLUMN 2"])
            .alignments(&[
                fmt::Alignment::Left,
                fmt::Alignment::Left,
                fmt::Alignment::Left,
            ])
            .data(&[vec!["---", "---"]])
            .to_string();
    }

    #[test]
    #[should_panic(expected = "number of headers must match columns in data")]
    fn table_error_nb_headers_neq_nb_columns_in_data() {
        Table::new()
            .headers(&["COLUMN 1", "COLUMN 2"])
            .alignments(&[fmt::Alignment::Left, fmt::Alignment::Left])
            .data(&[
                vec!["---", "---"],
                vec!["---", "---", "---"],
                vec!["---", "---"],
            ])
            .to_string();
    }

    #[test]
    fn table_max_rows_regular() {
        let table = Table::new()
            .max_rows(5)
            .headers(&["#", "COLUMN 1", "COLUMN 2"])
            .alignments(&[
                fmt::Alignment::Left,
                fmt::Alignment::Left,
                fmt::Alignment::Right,
            ])
            .data(&[
                vec!["1.", "---", "---"],
                vec!["2.", "---", "---"],
                vec!["3.", "------------", "------------"],
                vec!["4.", "------------", "------------"],
                vec!["5.", "---", "---"],
                vec!["6.", "---", "---"],
                vec!["7.", "---", "---"],
            ])
            .to_string();

        println!("{table}");
        assert_eq!(
            table,
            "\
#    COLUMN 1  COLUMN 2
1.   ---            ---
2.   ---            ---
...  ...            ...
5.   ---            ---
6.   ---            ---
7.   ---            ---
"
        );
    }

    #[test]
    fn table_max_rows_smallest_regular_case() {
        let table = Table::new()
            .max_rows(2)
            .headers(&["#", "COLUMN 1", "COLUMN 2"])
            .alignments(&[
                fmt::Alignment::Left,
                fmt::Alignment::Left,
                fmt::Alignment::Right,
            ])
            .data(&[
                vec!["1.", "---", "---"],
                vec!["2.", "---", "---"],
                vec!["3.", "---", "---"],
            ])
            .to_string();

        println!("{table}");
        assert_eq!(
            table,
            "\
#    COLUMN 1  COLUMN 2
1.   ---            ---
...  ...            ...
3.   ---            ---
"
        );
    }

    #[test]
    fn table_max_rows_elided_rows_do_not_impact_column_width() {
        let table = Table::new()
            .max_rows(1)
            .headers(&["#", "COLUMN 1", "COLUMN 2"])
            .alignments(&[
                fmt::Alignment::Left,
                fmt::Alignment::Left,
                fmt::Alignment::Right,
            ])
            .data(&[
                vec!["1.", "---", "---"],
                vec!["2.", "------------", "------------"],
            ])
            .to_string();

        println!("{table}");
        assert_eq!(
            table,
            "\
#    COLUMN 1  COLUMN 2
1.   ---            ---
...  ...            ...
"
        );
    }

    #[test]
    fn table_max_rows_gt_nb_rows() {
        let table = Table::new()
            .max_rows(8)
            .headers(&["#", "COLUMN 1", "COLUMN 2"])
            .alignments(&[
                fmt::Alignment::Left,
                fmt::Alignment::Left,
                fmt::Alignment::Right,
            ])
            .data(&[
                vec!["1.", "---", "---"],
                vec!["2.", "---", "---"],
                vec!["3.", "------------", "------------"],
                vec!["4.", "------------", "------------"],
                vec!["5.", "---", "---"],
                vec!["6.", "---", "---"],
                vec!["7.", "---", "---"],
            ])
            .to_string();

        println!("{table}");
        assert_eq!(
            table,
            "\
#   COLUMN 1          COLUMN 2
1.  ---                    ---
2.  ---                    ---
3.  ------------  ------------
4.  ------------  ------------
5.  ---                    ---
6.  ---                    ---
7.  ---                    ---
"
        );
    }

    #[test]
    fn table_max_rows_eq_nb_rows() {
        let table = Table::new()
            .max_rows(7)
            .headers(&["#", "COLUMN 1", "COLUMN 2"])
            .alignments(&[
                fmt::Alignment::Left,
                fmt::Alignment::Left,
                fmt::Alignment::Right,
            ])
            .data(&[
                vec!["1.", "---", "---"],
                vec!["2.", "---", "---"],
                vec!["3.", "------------", "------------"],
                vec!["4.", "------------", "------------"],
                vec!["5.", "---", "---"],
                vec!["6.", "---", "---"],
                vec!["7.", "---", "---"],
            ])
            .to_string();

        println!("{table}");
        assert_eq!(
            table,
            "\
#   COLUMN 1          COLUMN 2
1.  ---                    ---
2.  ---                    ---
3.  ------------  ------------
4.  ------------  ------------
5.  ---                    ---
6.  ---                    ---
7.  ---                    ---
"
        );
    }

    #[test]
    fn table_max_rows_max_zero() {
        let table = Table::new()
            .max_rows(0)
            .headers(&["#", "COLUMN 1", "COLUMN 2"])
            .alignments(&[
                fmt::Alignment::Left,
                fmt::Alignment::Left,
                fmt::Alignment::Right,
            ])
            .data(&[
                vec!["1.", "---", "---"],
                vec!["2.", "------------", "------------"],
            ])
            .to_string();

        println!("{table}");
        assert_eq!(
            table,
            "\
#    COLUMN 1  COLUMN 2
...  ...            ...
"
        );
    }

    #[test]
    fn table_max_rows_max_zero_with_empty_data() {
        let table = Table::new()
            .max_rows(0)
            .headers(&["#", "COLUMN 1", "COLUMN 2"])
            .alignments(&[
                fmt::Alignment::Left,
                fmt::Alignment::Left,
                fmt::Alignment::Right,
            ])
            .data(&[] as &[Vec<&str>; 0])
            .to_string();

        println!("{table}");
        assert_eq!(
            table,
            "\
#  COLUMN 1  COLUMN 2
"
        );
    }

    #[test]
    fn table_max_rows_max_zero_without_header() {
        // It is forbidden to have both empty headers and empty data.
        // Here we render with a 100% valid table, but clear the data
        // through `max_rows(0)`.
        let table = Table::new()
            .max_rows(0)
            .data(&[vec!["---", "----------------"]])
            .to_string();

        println!("{table}");
        assert_eq!(table, "...  ...\n");
    }

    #[test]
    fn table_max_rows_max_one() {
        let table = Table::new()
            .max_rows(1)
            .headers(&["#", "COLUMN 1", "COLUMN 2"])
            .alignments(&[
                fmt::Alignment::Left,
                fmt::Alignment::Left,
                fmt::Alignment::Right,
            ])
            .data(&[
                vec!["1.", "---", "---"],
                vec!["2.", "------------", "------------"],
                vec!["3.", "---", "---"],
            ])
            .to_string();

        println!("{table}");
        assert_eq!(
            table,
            "\
#    COLUMN 1  COLUMN 2
1.   ---            ---
...  ...            ...
"
        );
    }

    #[test]
    fn table_column_separator() {
        let table = Table::new()
            .headers(&["1", "2", "3"])
            .alignments(&[
                fmt::Alignment::Right,
                fmt::Alignment::Center,
                fmt::Alignment::Left,
            ])
            .data(&[
                vec!["---", "---", "---"],
                vec!["------", "------", "------"],
                vec!["---", "---", "---"],
            ])
            .column_separator(&"|")
            .to_string();

        println!("{table}");
        assert_eq!(
            table,
            "     1|  2   |3
   ---| ---  |---
------|------|------
   ---| ---  |---
"
        );
    }

    #[test]
    fn table_render_multiple_times() {
        let alignments = [fmt::Alignment::Left];
        let data = [vec!["---"]];
        let table = Table::new()
            .headers(&["HEADER"])
            .alignments(&alignments)
            .data(&data)
            .to_owned();

        let render_1 = table.to_string();
        let render_2 = table.to_string();

        println!("{table}");

        assert_eq!(render_1, "HEADER\n---\n");
        assert_eq!(render_1, render_2);
    }
}

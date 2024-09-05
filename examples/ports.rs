use std::fmt::Alignment::{Left, Right};
use verynicetable::Table;

fn main() {
    let ports = vec![
        vec!["rapportd", "449", "Quentin", "*:61165"],
        vec!["Python", "22396", "Quentin", "*:8000"],
        vec!["foo", "108", "root", "*:1337"],
        vec!["rustrover", "30928", "Quentin", "127.0.0.1:63342"],
        vec!["Transmiss", "94671", "Quentin", "*:51413"],
        vec!["Transmiss", "94671", "Quentin", "*:51413"],
    ];

    let table = Table::new()
        .headers(&["COMMAND", "PID", "USER", "HOST:PORTS"])
        .alignments(&[Left, Right, Left, Right])
        .data(&ports)
        .max_rows(5)
        .to_string();

    print!("{table}");
}

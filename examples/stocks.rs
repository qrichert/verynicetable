use std::fmt::Alignment::{Left, Right};

use verynicetable::Table;

fn up(value: &str) -> String {
    format!("\x1b[92m{value}\x1b[0m")
}

fn down(value: &str) -> String {
    format!("\x1b[91m{value}\x1b[0m")
}

fn main() {
    let markets = vec![
        vec![
            String::from("DOW"),
            String::from("United States"),
            up("42,313.00"),
            up("+ 137.89"),
            up("0.33%"),
        ],
        vec![
            String::from("S&P 500"),
            String::from("United States"),
            down("5,738.17"),
            down("- 7.20"),
            down("0.13%"),
        ],
        vec![
            String::from("NASDAQ"),
            String::from("United States"),
            down("18,119.59"),
            down("- 70.70"),
            down("0.39%"),
        ],
        vec![
            String::from("CAC 40"),
            String::from("France"),
            up("7,791.79"),
            up("+ 49.70"),
            up("0.64%"),
        ],
        vec![
            String::from("FTSE 100"),
            String::from("United Kingdom"),
            up("8,320.76"),
            up("+ 35.85"),
            up("0.43%"),
        ],
        vec![
            String::from("DAX"),
            String::from("Germany"),
            up("19,473.63"),
            up("+ 235.27"),
            up("1.22%"),
        ],
    ];

    let table = Table::new()
        .headers(&["MARKET", "", "PRICE", "CHANGE", "%CHANGE"])
        .alignments(&[Left, Left, Right, Right, Right])
        .data(&markets)
        .column_separator(" | ")
        .to_string();

    print!("{table}");
}

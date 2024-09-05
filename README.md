# very nice table

[![license: MIT](https://img.shields.io/badge/license-MIT-blue)](https://opensource.org/license/mit)
![GitHub Tag](https://img.shields.io/github/v/tag/qrichert/verynicetable?sort=semver&filter=*.*.*&label=release)
[![crates.io](https://img.shields.io/crates/d/verynicetable?logo=rust&logoColor=white&color=orange)](https://crates.io/crates/verynicetable)
[![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/qrichert/verynicetable/run-tests.yml?label=tests)](https://github.com/qrichert/verynicetable/actions)

_Number one table._

Very basic and lightweight table builder to print tabular data.

## Example

```rust
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
```

```
COMMAND      PID  USER          HOST:PORTS
rapportd     449  Quentin          *:61165
Python     22396  Quentin           *:8000
...          ...  ...                  ...
rustrover  30928  Quentin  127.0.0.1:63342
Transmiss  94671  Quentin          *:51413
Transmiss  94671  Quentin          *:51413
```

That's about it.

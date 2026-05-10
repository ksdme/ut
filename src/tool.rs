use anyhow::Context;
use serde_json::Value;
use std::io::{self, Write};
use tabled::{
    Table, Tabled,
    builder::Builder,
    settings::{Padding, Remove, Style, object::Rows},
};

// Represents a tool under ut.
pub trait Tool {
    // The contribution of this tool to the ut CLI. The clap::Command
    // returned here will be set up as a subcommand on the ut binary.
    fn cli() -> clap::Command;

    // Run the tool. All the context that the tool requires should be
    // using the cli above.
    fn execute(&self) -> anyhow::Result<Option<Output>>;
}

#[derive(Debug)]
pub enum Output {
    Bytes(Vec<u8>),
    JsonValue(serde_json::Value),
    /// An array of homogeneous objects rendered as a columnar table in human
    /// mode and as a JSON array when `--json` is passed.
    Table(serde_json::Value),
    Text(String),
}

impl Output {
    // Write out the output.
    pub fn flush(&self, structured: bool) -> anyhow::Result<()> {
        match self {
            Output::Bytes(bytes) => {
                io::stdout()
                    .write_all(&bytes)
                    .context("Could not write bytes to stdout")?;
            }
            Output::JsonValue(value) => {
                if structured {
                    println!("{}", value.to_string());
                } else {
                    println!("{}", value_to_string(value));
                }
            }
            Output::Table(value) => {
                if structured {
                    println!("{}", value);
                } else if let Value::Array(rows) = value {
                    println!("{}", render_object_table(rows));
                } else {
                    println!("{}", value_to_string(value));
                }
            }
            Output::Text(text) => {
                println!("{}", text);
            }
        }

        Ok(())
    }
}

/// Renders a slice of JSON objects as a columnar table.
/// Column order follows the key insertion order of the first row.
fn render_object_table(rows: &[Value]) -> String {
    let Some(Value::Object(first)) = rows.first() else {
        return String::new();
    };

    let headers: Vec<String> = first.keys().cloned().collect();
    let mut builder = Builder::default();
    builder.push_record(headers.iter().map(String::as_str));

    for row in rows {
        if let Value::Object(obj) = row {
            let vals: Vec<String> = headers
                .iter()
                .map(|h| value_to_string(obj.get(h).unwrap_or(&Value::Null)))
                .collect();
            builder.push_record(vals.iter().map(String::as_str));
        }
    }

    let mut table = builder.build();
    table
        .with(Style::empty())
        .with(Padding::new(0, 2, 0, 0));
    table.to_string()
}

fn value_to_string(value: &Value) -> String {
    match value {
        Value::Object(o) => {
            if o.is_empty() {
                return "{}".to_owned();
            }

            #[derive(Tabled)]
            struct KeyValue {
                key: String,
                value: String,
            }

            let rows: Vec<KeyValue> = o
                .iter()
                .map(|(k, v)| KeyValue {
                    key: k.clone(),
                    value: value_to_string(v),
                })
                .collect();

            let mut table = Table::new(rows);
            table
                .with(Style::empty())
                .with(Remove::row(Rows::first()))
                .with(Padding::new(0, 1, 0, 0));

            table.to_string()
        }
        Value::Array(a) => {
            let items = a
                .iter()
                .map(|val| value_to_string(val))
                .collect::<Vec<String>>();

            let mut table = Table::new(items);
            table
                .with(Style::empty())
                .with(Remove::row(Rows::first()))
                .with(Padding::new(0, 1, 0, 0));

            table.to_string()
        }
        Value::String(s) => s.clone(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::Null => "null".to_string(),
    }
}

use anyhow::Context;
use serde_json::Value;
use std::io::{self, Write};
use tabled::{
    Table, Tabled,
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
    Text(String),
}

impl Output {
    // Write out the output.
    pub fn flush(&self, human: bool) -> anyhow::Result<()> {
        match self {
            Output::Bytes(bytes) => {
                io::stdout()
                    .write_all(&bytes)
                    .context("Could not write bytes to stdout")?;
            }
            Output::JsonValue(value) => {
                if human {
                    println!("{}", value_to_string(value));
                } else {
                    println!("{}", value.to_string());
                }
            }
            Output::Text(text) => {
                println!("{}", text);
            }
        }

        Ok(())
    }
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

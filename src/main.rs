mod args;
mod data;
mod tool;
mod tools;

use std::io::{self, Write};

use clap::FromArgMatches;
use serde_json::Value;
use tabled::{
    Table, Tabled,
    settings::{Padding, Remove, Style, object::Rows},
};

use crate::tool::{Output, Tool};
use anyhow::{Context, anyhow};

// This way of building main is not ideal.
macro_rules! toolbox {
    ($cmd:ident, $(($tool:path, $name:literal, $($alias:literal),*)),+) => {
        {
            // Register the tools.
            $(
                $cmd = $cmd.subcommand(
                    <$tool>::cli()
                    .name($name)
                    $(.alias($alias))*
                );
            )*

            // Parse args.
            let matches = $cmd.get_matches();
            let (subcommand_name, subcommand_matches) = matches
                .subcommand()
                .context("Could not determine subcommand")?;

            // Run the specific tool.
            match subcommand_name {
                $(
                    $name => {
                        let output = <$tool>::from_arg_matches(subcommand_matches)
                            .context("Could not initialize the tool")?
                            .execute()
                            .context("Could not execute tool")?;

                        Ok(output)
                    }
                )*
                _ => {
                    Err(anyhow!("Unknown subcommand"))
                }
            }
        }
    };
}

fn main() -> anyhow::Result<()> {
    let mut cli = clap::builder::Command::new("ut").about("a utility toolkit");

    let output = toolbox!(
        cli,
        (tools::base64::Base64Tool, "base64",),
        (tools::calc::CalcTool, "calc",),
        (tools::case::CaseTool, "case",),
        (tools::color::ColorTool, "color",),
        (tools::datetime::DateTimeTool, "datetime",),
        (tools::diff::DiffTool, "diff",),
        (tools::hash::HashTool, "hash",),
        (tools::http::HttpStatusTool, "http-status",),
        (tools::json::JsonTool, "json",),
        (tools::lorem::LoremTool, "lorem",),
        (tools::pp::PrettyPrintTool, "pretty-print", "pp"),
        (tools::qr::QRTool, "qr",),
        (tools::random::RandomTool, "random",),
        (tools::regex::RegexTool, "regex",),
        (tools::serve::ServeTool, "serve",),
        (tools::token::TokenTool, "token",),
        (tools::url::UrlTool, "url",),
        (tools::uuid::UUIDTool, "uuid",),
        (tools::unicode::UnicodeTool, "unicode",)
    )
    .context("Could not run tool")?;

    match output {
        Some(Output::Bytes(bytes)) => {
            io::stdout()
                .write_all(&bytes)
                .context("Could not write bytes to stdout")?;
        }
        Some(Output::JsonValue(value)) => {
            print_json_value(&value)?;
        }
        Some(Output::Text(text)) => {
            println!("{}", text);
        }
        None => {}
    }

    Ok(())
}

fn print_json_value(value: &Value) -> anyhow::Result<()> {
    match value {
        // Object - print as table
        Value::Object(obj) => {
            if obj.is_empty() {
                println!("{{}}");
                return Ok(());
            }

            #[derive(Tabled)]
            struct KeyValue {
                key: String,
                value: String,
            }

            let rows: Vec<KeyValue> = obj
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

            println!("{}", table);
        }

        // Arrays.
        Value::Array(arr) => {
            for elem in arr {
                print_json_value(elem)?;
            }
        }

        // Scalar values - reuse value_to_string
        _ => println!("{}", value_to_string(value)),
    }

    Ok(())
}

fn value_to_string(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.clone(),
        _ => serde_json::to_string(value).unwrap_or_default(),
    }
}

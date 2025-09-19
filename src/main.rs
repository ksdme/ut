mod args;
mod tool;
mod tools;

use std::io::{self, Write};

use clap::FromArgMatches;

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
        (tools::case::CaseTool, "case",),
        (tools::hash::HashTool, "hash",),
        (tools::http_status::HttpTool, "http-status",),
        (tools::lorem::LoremTool, "lorem",),
        (tools::random::RandomTool, "random",),
        (tools::token::TokenTool, "token",),
        (tools::url::UrlTool, "url",),
        (tools::uuid::UUIDTool, "uuid",)
    )
    .context("Could not run tool")?;

    match output {
        Some(Output::Bytes(bytes)) => {
            io::stdout()
                .write_all(&bytes)
                .context("Could not write bytes to stdout")?;
        }
        Some(Output::JsonValue(value)) => {
            print!(
                "{}",
                serde_json::to_string_pretty(&value).context("Could not serialize result")?
            );
        }
        None => {}
    }

    Ok(())
}

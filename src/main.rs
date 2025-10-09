mod args;
mod data;
mod tool;
mod tools;

use clap::CommandFactory;
use clap::FromArgMatches;
use clap::Parser;
use clap_complete::{Shell, generate};
use std::io;

use crate::tool::Tool;
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
            let matches = $cmd.clone().get_matches();
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
                "completions" => {
                    Completions::from_arg_matches(subcommand_matches)
                        .context("Could not initialize the tool")?
                        .execute(&mut $cmd);

                    Ok(None)
                }
                _ => {
                    Err(anyhow!("Unknown subcommand"))
                }
            }
        }
    };
}

fn main() -> anyhow::Result<()> {
    let mut cli = clap::builder::Command::new("ut")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(Completions::command().name("completions"));

    let output = toolbox!(
        cli,
        (tools::base64::Base64Tool, "base64",),
        (tools::bcrypt::BcryptTool, "bcrypt",),
        (tools::calc::CalcTool, "calc", "cal"),
        (tools::case::CaseTool, "case",),
        (tools::color::ColorTool, "color",),
        (tools::crontab::CrontabTool, "crontab", "cron"),
        (tools::datetime::DateTimeTool, "datetime", "dt"),
        (tools::diff::DiffTool, "diff",),
        (tools::hash::HashTool, "hash",),
        (tools::http::HttpTool, "http",),
        (tools::json::JsonTool, "json",),
        (tools::lorem::LoremTool, "lorem",),
        (tools::pp::PrettyPrintTool, "pretty-print", "pp"),
        (tools::qr::QRTool, "qr",),
        (tools::random::RandomTool, "random",),
        (tools::regex::RegexTool, "regex",),
        (tools::serve::ServeTool, "serve",),
        (tools::token::TokenTool, "token", "secret"),
        (tools::url::UrlTool, "url",),
        (tools::uuid::UUIDTool, "uuid",),
        (tools::unicode::UnicodeTool, "unicode",)
    )
    .context("Could not run tool")?;

    match output {
        Some(output) => output.flush(),
        None => Ok(()),
    }
}

#[derive(Parser, Debug)]
#[command(name = "completions", about = "Generate shell completions for ut")]
struct Completions {
    shell: Shell,
}

impl Completions {
    fn execute(&self, cli: &mut clap::Command) {
        generate(self.shell, cli, "ut", &mut io::stdout());
    }
}

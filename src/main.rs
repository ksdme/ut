mod args;
mod data;
mod tool;
mod tools;

use clap::CommandFactory;
use clap::FromArgMatches;
use clap::Parser;
use clap_complete::generate;
use clap_complete_nushell::Nushell;
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
                    $(.visible_alias($alias))*
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
        (tools::jwt::JwtTool, "jwt",),
        (tools::lorem::LoremTool, "lorem",),
        (tools::pp::PrettyPrintTool, "pretty-print", "pp"),
        (tools::qr::QRTool, "qr",),
        (tools::random::RandomTool, "random",),
        (tools::regex::RegexTool, "regex",),
        (tools::serve::ServeTool, "serve",),
        (tools::token::TokenTool, "token", "secret", "password"),
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
#[command(
    name = "completions",
    about = "Generate shell completions for ut",
    long_about = "Generate shell completion scripts for ut.\n\n\
                  Examples:\n  \
                  ut completions zsh > ~/.zsh/completions/_ut\n  \
                  ut completions bash > ~/.local/share/bash-completion/completions/ut\n  \
                  ut completions nushell > ~/.config/nushell/completions/ut.nu"
)]
struct Completions {
    shell: Shell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
#[value(rename_all = "lowercase")]
enum Shell {
    Bash,
    Elvish,
    Fish,
    PowerShell,
    Zsh,
    #[value(alias = "nu")]
    Nushell,
}

impl Completions {
    fn execute(&self, cli: &mut clap::Command) {
        let out = &mut io::stdout();
        match self.shell {
            Shell::Bash => generate(clap_complete::Shell::Bash, cli, "ut", out),
            Shell::Elvish => generate(clap_complete::Shell::Elvish, cli, "ut", out),
            Shell::Fish => generate(clap_complete::Shell::Fish, cli, "ut", out),
            Shell::PowerShell => generate(clap_complete::Shell::PowerShell, cli, "ut", out),
            Shell::Zsh => generate(clap_complete::Shell::Zsh, cli, "ut", out),
            Shell::Nushell => generate(Nushell, cli, "ut", out),
        }
    }
}

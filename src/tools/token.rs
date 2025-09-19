use crate::tool::{Output, Tool};
use anyhow::bail;
use clap::{Command, CommandFactory, Parser};
use rand::{Rng, rngs::OsRng};

#[derive(Parser, Debug)]
#[command(
    name = "token",
    about = "Generate a cryptographically secure random token."
)]
pub struct TokenTool {
    /// Length of the token to generate
    #[arg(long, short, default_value = "64")]
    length: usize,

    /// Do not include uppercase letters
    #[arg(long, default_value = "false")]
    no_uppercase: bool,

    /// Do not include lowercase letters
    #[arg(long, default_value = "false")]
    no_lowercase: bool,

    /// Do not include numbers
    #[arg(long, default_value = "false")]
    no_numbers: bool,

    /// Do not include symbols
    #[arg(long, default_value = "false")]
    no_symbols: bool,
}

impl Tool for TokenTool {
    fn cli() -> Command {
        TokenTool::command()
    }

    fn execute(&self) -> anyhow::Result<Option<Output>> {
        let mut charset = String::new();

        if !self.no_uppercase {
            charset.push_str("ABCDEFGHIJKLMNOPQRSTUVWXYZ");
        }

        if !self.no_lowercase {
            charset.push_str("abcdefghijklmnopqrstuvwxyz");
        }

        if !self.no_numbers {
            charset.push_str("0123456789");
        }

        if !self.no_symbols {
            charset.push_str("!@#$%^&*_+-");
        }

        if charset.is_empty() {
            bail!("At least one character set must be enabled")
        }

        // https://crates.io/crates/getrandom
        // This is a cryptographically secure randomness.
        let mut rng = OsRng;

        let charset_chars: Vec<char> = charset.chars().collect();
        let token: String = (0..self.length)
            .map(|_| charset_chars[rng.gen_range(0..charset_chars.len())])
            .collect();

        Ok(Some(Output::JsonValue(serde_json::json!(token))))
    }
}

/// Based on https://it-tools.tech/token-generator
use crate::tool::{Output, Tool};

use anyhow::Context;
use rand::distr::{Distribution, slice::Choose};
use serde_json::json;

#[derive(clap::Parser, Default, Debug)]
#[command(about = "Generate random token/string.")]
pub struct TokenGenerator {
    /// Include uppercase letters.
    #[arg(long = "uppercase", default_value_t = true, action = clap::ArgAction::Set)]
    pub uppercase: bool,

    /// Include lowercase letters.
    #[arg(long = "lowercase", default_value_t = true, action = clap::ArgAction::Set)]
    pub lowercase: bool,

    /// Include lowercase letters.
    #[arg(long = "numbers", default_value_t = true, action = clap::ArgAction::Set)]
    pub numbers: bool,

    /// Include symbol letters.
    #[arg(long = "symbols", default_value_t = false, action = clap::ArgAction::Set)]
    pub symbols: bool,

    /// The length of the generated string.
    #[arg(short = 'l', long = "length", default_value_t = 16)]
    pub length: usize,
}

impl Tool for TokenGenerator {
    fn execute(&self) -> anyhow::Result<Option<Output>> {
        let mut space = String::new();

        if self.lowercase {
            space += "abcdefghijklmnopqrstuvwxyz";
        }
        if self.uppercase {
            space += "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
        }
        if self.symbols {
            space += "!@#$%^&*()-_+=[{}]|;:'\",<>./?";
        }
        if self.numbers {
            space += "0123456789";
        }

        let token: Vec<u8> = Choose::new(space.as_bytes())
            .context("Could not build sampler")?
            .sample_iter(&mut rand::rng())
            .take(self.length)
            .map(|b| b.clone())
            .collect();

        Ok(Some(Output::JsonValue(json!(
            String::from_utf8(token).context("Could not decode token")?
        ))))
    }
}

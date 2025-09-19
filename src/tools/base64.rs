use crate::tool::{Output, Tool};
use anyhow::Context;
use base64::{Engine as _, engine::general_purpose};
use clap::{Command, CommandFactory, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "base64")]
pub struct Base64Tool {
    #[command(subcommand)]
    command: Base64Command,
}

#[derive(Subcommand, Debug)]
enum Base64Command {
    /// base64 encode text
    Encode {
        text: String,
        /// use urlsafe encoding
        #[arg(long)]
        urlsafe: bool,
    },
    /// base64 decode text
    Decode {
        text: String,
        /// use urlsafe decoding
        #[arg(long)]
        urlsafe: bool,
    },
}

impl Tool for Base64Tool {
    fn cli() -> Command {
        Base64Tool::command()
    }

    fn execute(&self) -> anyhow::Result<Option<Output>> {
        match &self.command {
            Base64Command::Encode { text, urlsafe } => {
                let encoded = if *urlsafe {
                    general_purpose::URL_SAFE.encode(text)
                } else {
                    general_purpose::STANDARD.encode(text)
                };

                Ok(Some(Output::JsonValue(serde_json::json!(encoded))))
            }
            Base64Command::Decode { text, urlsafe } => {
                let engine = if *urlsafe {
                    &general_purpose::URL_SAFE
                } else {
                    &general_purpose::STANDARD
                };

                Ok(Some(Output::Bytes(
                    engine.decode(text).context("Could not decode base64")?,
                )))
            }
        }
    }
}

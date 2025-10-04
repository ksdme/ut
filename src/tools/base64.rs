use crate::{
    args::StringInput,
    tool::{Output, Tool},
};
use anyhow::Context;
use base64::{Engine as _, engine::general_purpose};
use clap::{Command, CommandFactory, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "base64", about = "Base64 encode and decode utilities")]
pub struct Base64Tool {
    #[command(subcommand)]
    command: Base64Command,
}

#[derive(Subcommand, Debug)]
enum Base64Command {
    /// Base64 encode contents
    Encode {
        /// Input to encode (use "-" for stdin)
        text: StringInput,
        /// Encode with urlsafe character set
        #[arg(long)]
        urlsafe: bool,
    },
    /// Base64 decode contents
    Decode {
        /// Input to decode (use "-" for stdin)
        text: StringInput,
        /// Decode with urlsafe character set
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
                    general_purpose::URL_SAFE.encode(&text.0)
                } else {
                    general_purpose::STANDARD.encode(&text.0)
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
                    engine.decode(&text.0).context("Could not decode base64")?,
                )))
            }
        }
    }
}

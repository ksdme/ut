use crate::tool::{Output, Tool};
use anyhow::Context;
use clap::{Command, CommandFactory, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "url", about = "URL encode and decode utilities")]
pub struct UrlTool {
    #[command(subcommand)]
    command: UrlCommand,
}

#[derive(Subcommand, Debug)]
enum UrlCommand {
    /// URL encode text
    Encode {
        /// Text to URL encode
        text: String,
    },
    /// URL decode text
    Decode {
        /// Text to URL decode
        text: String,
    },
}

impl Tool for UrlTool {
    fn cli() -> Command {
        UrlTool::command()
    }

    fn execute(&self) -> anyhow::Result<Option<Output>> {
        let result = match &self.command {
            UrlCommand::Encode { text } => urlencoding::encode(text).into_owned(),
            UrlCommand::Decode { text } => urlencoding::decode(text)
                .context("Could not decode")?
                .into_owned(),
        };

        Ok(Some(Output::JsonValue(serde_json::json!(result))))
    }
}

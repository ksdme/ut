use crate::tool::{Output, Tool};
use anyhow::Context;
use clap::{Command, CommandFactory, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "url")]
pub struct Url {
    #[command(subcommand)]
    command: UrlCommand,
}

#[derive(Subcommand, Debug)]
enum UrlCommand {
    /// url encode text
    Encode { text: String },
    /// url decode text
    Decode { text: String },
}

impl Tool for Url {
    fn cli() -> Command {
        Url::command()
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

use crate::args::StringInput;
use crate::tool::{Output, Tool};
use anyhow::Context;
use clap::{Command, CommandFactory, Parser};

// TODO: Add a table parser
#[derive(Parser, Debug)]
#[command(
    name = "pretty-print",
    about = "Resolve escaped newlines and tab characters"
)]
pub struct PrettyPrintTool {
    /// Text to unescape. Use "-" to read from stdin, or omit to open editor.
    text: Option<StringInput>,
}

impl Tool for PrettyPrintTool {
    fn cli() -> Command {
        PrettyPrintTool::command()
    }

    fn execute(&self) -> anyhow::Result<Option<Output>> {
        let input = match &self.text {
            Some(text) => text.as_ref().to_string(),
            None => edit::edit("").context("Could not read value")?,
        };

        let result = input
            .replace("\\\\", "\\")
            .replace("\\n", "\n")
            .replace("\\n", "\n")
            .replace("\\t", "\t")
            .replace("\\r", "\r");

        Ok(Some(Output::Text(result)))
    }
}

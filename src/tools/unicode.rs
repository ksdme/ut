use crate::{
    data,
    tool::{Output, Tool},
};
use clap::{Command, CommandFactory, Parser};

#[derive(Parser, Debug)]
#[command(name = "unicode", about = "Unicode symbol reference")]
pub struct UnicodeTool {}

impl Tool for UnicodeTool {
    fn cli() -> Command {
        UnicodeTool::command()
    }

    fn execute(&self) -> anyhow::Result<Option<Output>> {
        let mut output = String::new();
        for (_, items) in data::unicode::UNICODE_CHARS {
            for (name, letter) in items.iter() {
                output.push_str(&format!("{} {}\n", letter, name));
            }
        }

        Ok(Some(Output::Text(output)))
    }
}

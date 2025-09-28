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
        for (_, items) in data::unicode::UNICODE_CHARS {
            for (name, letter) in items.iter() {
                println!("{} {}", letter, name);
            }
        }

        // To prevent copying all the data, we return nothing from this tool
        // and instead print it out here itself.
        Ok(None)
    }
}

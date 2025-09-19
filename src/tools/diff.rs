use crate::tool::{Output, Tool};
use anyhow::{Context, Result};
use clap::{Command, CommandFactory, Parser};
use colored::Colorize;
use similar::{ChangeTag, TextDiff};
use std::fs;
use std::path::Path;

#[derive(Parser, Debug)]
#[command(name = "diff")]
pub struct DiffTool {
    /// First version of the file (if omitted, opens editor for contents)
    a: Option<String>,

    /// Second version of the file (if omitted, opens editor for contents)
    b: Option<String>,
}

impl Tool for DiffTool {
    fn cli() -> Command {
        DiffTool::command()
    }

    fn execute(&self) -> Result<Option<Output>> {
        let first_content = match &self.a {
            Some(arg) => get_content(arg)?,
            None => get_content_from_editor("# a")?,
        };

        let second_content = match &self.b {
            Some(arg) => get_content(arg)?,
            None => get_content_from_editor("# b")?,
        };

        let diff = TextDiff::from_lines(&first_content, &second_content)
            .iter_all_changes()
            .map(|change| match change.tag() {
                ChangeTag::Equal => format!(" {}", change.to_string()),
                ChangeTag::Delete => format!("{}{}", "-".red(), change.to_string().red()),
                ChangeTag::Insert => format!("{}{}", "+".green(), change.to_string().green()),
            })
            .collect::<Vec<String>>()
            .join("");

        println!("{}", diff);

        Ok(None)
    }
}

fn get_content(arg: &str) -> Result<String> {
    fs::read_to_string(Path::new(arg)).context("Could not read file")
}

fn get_content_from_editor(prompt: &str) -> Result<String> {
    let content = edit::edit(prompt)?;
    Ok(content)
}

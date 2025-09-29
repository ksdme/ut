use crate::tool::{Output, Tool};
use anyhow::{Context, Result};
use clap::{Command, CommandFactory, Parser};
use colored::Colorize;
use similar::{ChangeTag, TextDiff};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(name = "diff", about = "Compare text contents")]
pub struct DiffTool {
    /// First version of the file, omit to use editor
    a: Option<PathBuf>,

    /// Second version of the file, omit to use editor
    b: Option<PathBuf>,
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

        let line_no_width = (first_content
            .lines()
            .count()
            .max(second_content.lines().count()) as f64)
            .log10()
            .ceil() as usize;

        let mut buffer = String::new();
        let mut lines: Vec<(Option<u64>, Option<u64>, String)> = vec![];

        let mut o_line_no: u64 = 0;
        let mut o_o_line_no = o_line_no;

        let mut n_line_no: u64 = 0;
        let mut o_n_line_no = n_line_no;

        for change in TextDiff::from_chars(&first_content, &second_content).iter_all_changes() {
            let Some(ch) = change.as_str() else {
                continue;
            };

            // Handle line breaks so we can keep track of line numbers.
            if ch == "\n" {
                o_o_line_no = o_line_no;
                o_n_line_no = n_line_no;

                let push: bool;
                match change.tag() {
                    ChangeTag::Equal => {
                        push = true;

                        buffer.push_str(&ch);

                        o_line_no += 1;
                        n_line_no += 1;
                    }
                    ChangeTag::Delete => {
                        push = buffer.is_empty();

                        buffer.push_str(&"↙".black().on_red().to_string());
                        if push {
                            buffer.push_str(&ch);
                        }

                        o_line_no += 1;
                    }
                    ChangeTag::Insert => {
                        push = true;

                        buffer.push_str(&format!("{}{}", "↙".black().on_green(), ch));

                        n_line_no += 1;
                    }
                };

                if push {
                    lines.push((
                        if o_o_line_no == o_line_no {
                            None
                        } else {
                            Some(o_line_no)
                        },
                        if o_n_line_no == n_line_no {
                            None
                        } else {
                            Some(n_line_no)
                        },
                        buffer.clone(),
                    ));
                    buffer.clear();
                }
            } else {
                // Represent meta characters.
                let ch = match change.tag() {
                    ChangeTag::Equal if ch == "\r" => "␍",
                    _ => ch,
                };

                match change.tag() {
                    ChangeTag::Equal => buffer.push_str(&ch),
                    ChangeTag::Delete => buffer.push_str(&ch.black().on_red().to_string()),
                    ChangeTag::Insert => buffer.push_str(&ch.black().on_green().to_string()),
                }
            }
        }

        // Flush the last line if the files weren't terminated with a newline.
        if !buffer.is_empty() {
            lines.push((
                if o_o_line_no == o_line_no {
                    None
                } else {
                    Some(o_line_no)
                },
                if o_n_line_no == n_line_no {
                    None
                } else {
                    Some(n_line_no)
                },
                buffer.clone(),
            ));
        }

        for (o_line_no, n_line_no, line) in lines.iter() {
            print!(
                "{} {}",
                format!(
                    "{:>width$} {:>width$} ┊",
                    o_line_no.map(|l| l.to_string()).unwrap_or_default(),
                    n_line_no.map(|l| l.to_string()).unwrap_or_default(),
                    width = line_no_width,
                )
                .dimmed(),
                line,
            );
        }

        Ok(None)
    }
}

fn get_content(arg: &PathBuf) -> Result<String> {
    fs::read_to_string(Path::new(arg)).context("Could not read file")
}

fn get_content_from_editor(prompt: &str) -> Result<String> {
    let content = edit::edit(prompt)?;
    Ok(content)
}

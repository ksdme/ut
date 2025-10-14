use crate::args::StringInput;
use crate::tool::{Output, Tool};
use anyhow::{Context, Result};
use clap::{Command, CommandFactory, Parser, Subcommand};
use csscolorparser::Color;
use serde_json::json;

#[derive(Parser, Debug)]
#[command(name = "color", about = "Color utilities")]
pub struct ColorTool {
    #[command(subcommand)]
    command: ColorCommand,
}

#[derive(Subcommand, Debug)]
enum ColorCommand {
    /// Convert colors between different formats
    Convert {
        /// Color value in any supported format (e.g., hex, rgb, rgba, hsl, hwb, cmyk, lch, oklch)
        color: StringInput,
    },
}

impl Tool for ColorTool {
    fn cli() -> Command {
        ColorTool::command()
    }

    fn execute(&self) -> Result<Option<Output>> {
        match &self.command {
            ColorCommand::Convert { color } => {
                let color = color.as_ref().parse::<Color>().context("Failed to parse color")?;

                let [r, g, b, _] = color.to_rgba8();
                let [_, _, _, a_f] = color.to_array();

                Ok(Some(Output::JsonValue(json!({
                    "rgb": color.to_css_rgb(),
                    "rgba": format!("rgba({}, {}, {}, {:.3})", r, g, b, a_f),
                    "hex": color.to_css_hex(),
                    "hsl": color.to_css_hsl(),
                    "hwb": color.to_css_hwb(),
                    "lab": color.to_css_lab(),
                    "lch": color.to_css_lch(),
                    "oklab": color.to_css_oklab(),
                    "oklch": color.to_css_oklch(),
                }))))
            }
        }
    }
}

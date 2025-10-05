use crate::tool::{Output, Tool};
use anyhow::{Context, Result};
use clap::{Command, CommandFactory, Parser};
use csscolorparser::Color;
use serde_json::json;

#[derive(Parser, Debug)]
#[command(name = "color", about = "Convert colors between different formats")]
pub struct ColorConvertTool {
    /// Color value in any supported format (hex, rgb, rgba, hsl, hwb, cmyk, lch, oklch)
    color: String,
}

impl Tool for ColorConvertTool {
    fn cli() -> Command {
        ColorConvertTool::command()
    }

    fn execute(&self) -> Result<Option<Output>> {
        let color = self
            .color
            .parse::<Color>()
            .context("Failed to parse color")?;

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

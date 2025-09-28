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

        Ok(Some(Output::JsonValue(json!([
            {
                "format": "rgb",
                "value": color.to_css_rgb()
            },
            {
                "format": "rgba",
                "value": format!("rgba({}, {}, {}, {:.3})", r, g, b, a_f)
            },
            {
                "format": "hex",
                "value": color.to_css_hex(),
            },
            {
                "format": "hsl",
                "value": color.to_css_hsl(),
            },
            {
                "format": "hwb",
                "value": color.to_css_hwb(),
            },
            {
                "format": "lab",
                "value": color.to_css_lab(),
            },
            {
                "format": "lch",
                "value": color.to_css_lch(),
            },
            {
                "format": "oklab",
                "value": color.to_css_oklab(),
            },
            {
                "format": "oklch",
                "value": color.to_css_oklch(),
            },
        ]))))
    }
}

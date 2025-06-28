use std::path::{self, PathBuf};

use anyhow::{Context, anyhow};
use image::Rgb;

use crate::tool::Tool;

#[derive(Debug, clap::Parser)]
#[command(about = "Generate a QR Code")]
pub struct QRGenerator {
    contents: String,

    /// Foreground color on the QR Code in a CSS color format (e.g. #000)
    #[arg(long = "fg-color", default_value = "#000")]
    foreground_color: String,

    /// Background color of the QR Code in a CSS color format (e.g. #fff)
    #[arg(long = "bg-color", default_value = "#fff")]
    background_color: String,

    /// The output file path for the generated QR Code image.
    #[arg(short = 'o', long = "out-file")]
    out_file: Option<String>,
}

impl Tool for QRGenerator {
    fn execute(&self) -> anyhow::Result<Option<crate::tool::Output>> {
        let out_file = self
            .out_file
            .clone()
            .or_else(|| Some(format!("{}.png", chrono::Local::now().timestamp())))
            .context("Could not resolve output file path")?;

        let out_path = path::absolute(PathBuf::from(out_file))
            .context("Could not resolve output file path")?;

        if out_path.exists() {
            return Err(anyhow!("File already exists at output path"));
        }

        let code =
            qrcode::QrCode::new(self.contents.as_bytes()).context("Could not construct QR Code")?;

        let [fr, fg, fb, _] = csscolorparser::parse(&self.foreground_color)
            .context("Could not parse foreground color")?
            .to_rgba8();

        let [br, bg, bb, _] = csscolorparser::parse(&self.background_color)
            .context("Could not parse background color")?
            .to_rgba8();

        let image = code
            .render::<Rgb<u8>>()
            .dark_color(Rgb([fr, fg, fb]))
            .light_color(Rgb([br, bg, bb]))
            .quiet_zone(true)
            .build();

        image
            .save(out_path)
            .context("Could not write the image to file")?;

        Ok(None)
    }
}

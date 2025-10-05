use crate::tool::{Output, Tool};
use anyhow::{Context, Result};
use clap::{Command, CommandFactory, Parser};
use qrcode::QrCode;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "qr", about = "Generate QR codes")]
pub struct QRTool {
    /// The text or URL to encode as QR code
    text: String,

    /// Save QR code to file (PNG format)
    #[arg(short, long)]
    output: Option<PathBuf>,
}

impl Tool for QRTool {
    fn cli() -> Command {
        QRTool::command()
    }

    fn execute(&self) -> Result<Option<Output>> {
        let code = QrCode::new(&self.text).context("Failed to generate QR code")?;

        if let Some(output_path) = &self.output {
            // Save to file
            let image = code
                .render::<image::Luma<u8>>()
                .max_dimensions(512, 512)
                .build();

            image
                .save(output_path)
                .context("Failed to save QR code image")?;

            Ok(None)
        } else {
            // Display in terminal
            let string = code
                .render::<char>()
                .quiet_zone(false)
                .module_dimensions(2, 1)
                .build();

            Ok(Some(Output::Text(string)))
        }
    }
}

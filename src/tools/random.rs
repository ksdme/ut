use crate::tool::{Output, Tool};
use anyhow::{Context, bail};
use clap::{Command, CommandFactory, Parser};
use num_traits::ToPrimitive;
use rand::{Rng, rngs::OsRng};
use rust_decimal::{Decimal, dec};

#[derive(Parser, Debug)]
#[command(name = "random")]
pub struct RandomTool {
    /// Number of random numbers to generate
    #[arg(short = 'c', long = "count", default_value = "1")]
    quantity: usize,

    /// Minimum value (inclusive)
    #[arg(long, default_value = "0")]
    min: rust_decimal::Decimal,

    /// Maximum value (inclusive)
    #[arg(long, default_value = "100")]
    max: rust_decimal::Decimal,

    /// Step value for precision (e.g., 0.01 for 2 decimal places)
    #[arg(short, long, default_value = "1")]
    step: rust_decimal::Decimal,
}

impl Tool for RandomTool {
    fn cli() -> Command {
        RandomTool::command()
    }

    fn execute(&self) -> anyhow::Result<Option<Output>> {
        if self.quantity == 0 {
            bail!("Count must be greater than 0");
        }

        if self.min > self.max {
            bail!("Minimum value cannot be greater than maximum value");
        }

        if self.step <= dec!(0) {
            bail!("Step value must be greater than 0");
        }

        let range = self.max - self.min;
        let steps = (range / self.step)
            .floor()
            .to_u64()
            .context("Could not resolve step count")?;

        let mut rng = OsRng;
        let values: Vec<_> = (0..self.quantity)
            .map(|_| self.min + self.step * Decimal::from(rng.gen_range(0..=steps)))
            .collect();

        Ok(Some(Output::JsonValue(serde_json::json!(values))))
    }
}

/// Based on https://it-tools.tech/uuid-generator
use serde_json::json;
use uuid::Uuid;

use crate::tool::{Output, Tool};

#[derive(clap::Parser, Debug)]
#[command(about = "Generate v4 UUIDs")]
pub struct UUID4Generator {
    /// Number of UUIDs to generate
    #[arg(short = 'n', long = "count", default_value_t = 1)]
    pub count: usize,
}

impl Tool for UUID4Generator {
    fn execute(&self) -> anyhow::Result<Option<Output>> {
        let uuids: Vec<String> = (0..self.count)
            .map(|_| Uuid::new_v4().to_string())
            .collect();

        Ok(Some(Output::JsonValue(json!(uuids))))
    }
}

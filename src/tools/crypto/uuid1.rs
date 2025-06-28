/// Based on https://it-tools.tech/uuid-generator
use anyhow::Context;
use serde_json::json;
use uuid::Uuid;

use crate::tool::{Output, Tool};

#[derive(clap::Parser, Debug)]
#[command(about = "Generate v1 UUIDs")]
pub struct UUID1Generator {
    /// Number of UUIDs to generate
    #[arg(short = 'n', long = "count", default_value_t = 1)]
    pub count: usize,

    /// Node ID (6 bytes)
    #[arg(long = "node-id")]
    pub node_id: Option<u64>,
}

impl Tool for UUID1Generator {
    fn execute(&self) -> anyhow::Result<Option<Output>> {
        let node_id = self
            .node_id
            .or_else(|| Some(rand::random::<u64>()))
            .map(|n| {
                [
                    (n >> 40) as u8,
                    (n >> 32) as u8,
                    (n >> 24) as u8,
                    (n >> 16) as u8,
                    (n >> 8) as u8,
                    n as u8,
                ]
            })
            .context("Could not generate node-id")?;

        let uuids: Vec<String> = (0..self.count)
            .map(|_| Uuid::now_v1(&node_id).to_string())
            .collect();

        Ok(Some(Output::JsonValue(json!(uuids))))
    }
}

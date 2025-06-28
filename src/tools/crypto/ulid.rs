/// Based on https://it-tools.tech/ulid-generator
use serde_json::json;
use ulid::Ulid;

use crate::tool::{Output, Tool};

#[derive(clap::Parser, Debug)]
#[command(about = "Generate ULIDs (Universally Unique Lexicographically Sortable Identifiers)")]
pub struct ULIDGenerator {
    /// Number of ULIDs to generate
    #[arg(short = 'n', long = "count", default_value_t = 1)]
    pub count: usize,
}

impl Tool for ULIDGenerator {
    fn execute(&self) -> anyhow::Result<Option<Output>> {
        let uuids: Vec<String> = (0..self.count).map(|_| Ulid::new().to_string()).collect();

        Ok(Some(Output::JsonValue(json!(uuids))))
    }
}

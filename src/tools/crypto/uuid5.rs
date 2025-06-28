/// Based on https://it-tools.tech/uuid-generator
use crate::{
    tool::{Output, Tool},
    tools::crypto::uuid3::NamespaceType,
};

use anyhow::Context;
use serde_json::json;
use uuid::Uuid;

#[derive(clap::Parser, Debug)]
#[command(about = "Generate v5 UUIDs")]
pub struct UUID5Generator {
    /// Namespace type.
    #[arg(long = "ns-type", value_enum, default_value = "url")]
    pub ns_type: NamespaceType,

    /// Custom namespace value.
    #[arg(long = "ns-value")]
    pub ns_value: Option<String>,

    /// Name of the UUID subject.
    #[arg(long = "name")]
    pub name: Option<String>,
}

impl Tool for UUID5Generator {
    fn execute(&self) -> anyhow::Result<Option<Output>> {
        let name = self
            .name
            .as_ref()
            .context("name is required for v5 UUIDs")?;

        let namespace = self
            .ns_type
            .get_namespace(&self.ns_value)
            .context("Could not generate namespace")?;

        Ok(Some(Output::JsonValue(json!(
            Uuid::new_v5(&namespace, name.as_bytes()).to_string()
        ))))
    }
}

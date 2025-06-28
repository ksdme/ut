/// Based on https://it-tools.tech/uuid-generator
use crate::tool::{Output, Tool};

use anyhow::Context;
use clap::ValueEnum;
use serde_json::json;
use uuid::Uuid;

#[derive(clap::Parser, Debug)]
#[command(about = "Generate v3 UUIDs")]
pub struct UUID3Generator {
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

impl Tool for UUID3Generator {
    fn execute(&self) -> anyhow::Result<Option<Output>> {
        let name = self
            .name
            .as_ref()
            .context("name is required for v3 UUIDs")?;

        let namespace = self
            .ns_type
            .get_namespace(&self.ns_value)
            .context("Could not generate namespace")?;

        Ok(Some(Output::JsonValue(json!(
            Uuid::new_v3(&namespace, name.as_bytes()).to_string()
        ))))
    }
}

#[derive(ValueEnum, Clone, Debug)]
pub enum NamespaceType {
    URL,
    DNS,
    OID,
    X500,
    Custom,
}

impl NamespaceType {
    pub fn get_namespace(&self, ns_value: &Option<String>) -> anyhow::Result<Uuid> {
        match self {
            NamespaceType::URL => Ok(uuid::Uuid::NAMESPACE_URL),
            NamespaceType::DNS => Ok(uuid::Uuid::NAMESPACE_DNS),
            NamespaceType::OID => Ok(uuid::Uuid::NAMESPACE_OID),
            NamespaceType::X500 => Ok(uuid::Uuid::NAMESPACE_X500),
            NamespaceType::Custom => {
                let namespace_value = ns_value
                    .as_ref()
                    .context("namespace value is required for custom namespaces")?;

                Uuid::parse_str(namespace_value).context("invalid custom namespace UUID format")
            }
        }
    }
}

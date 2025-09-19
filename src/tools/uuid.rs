use crate::tool::{Output, Tool};
use clap::{Command, CommandFactory, Parser, Subcommand, ValueEnum};
use uuid::Uuid;

#[derive(Parser, Debug)]
#[command(name = "uuid")]
pub struct UUIDTool {
    #[command(subcommand)]
    command: UUIDCommand,
}

#[derive(ValueEnum, Clone, Debug)]
enum Namespace {
    /// DNS namespace
    DNS,
    /// URL namespace
    URL,
    /// ISO OID namespace
    OID,
    /// X.500 DN namespace
    X500,
}

impl Namespace {
    fn to_uuid(&self) -> Uuid {
        match self {
            Namespace::DNS => Uuid::NAMESPACE_DNS,
            Namespace::URL => Uuid::NAMESPACE_URL,
            Namespace::OID => Uuid::NAMESPACE_OID,
            Namespace::X500 => Uuid::NAMESPACE_X500,
        }
    }
}

#[derive(Subcommand, Debug)]
enum UUIDCommand {
    /// Generate UUID v1 (timestamp-based)
    V1 {
        /// Number of UUIDs to generate
        #[arg(short = 'c', long = "count", default_value = "1")]
        quantity: usize,
    },
    /// Generate UUID v3 (namespace + MD5 hash)
    V3 {
        /// Namespace to use
        #[arg(short, long)]
        namespace: Namespace,
        /// Name to hash
        #[arg(short = 'N', long)]
        name: String,
        /// Number of UUIDs to generate
        #[arg(short = 'c', long = "count", default_value = "1")]
        quantity: usize,
    },
    /// Generate UUID v4 (random)
    V4 {
        /// Number of UUIDs to generate
        #[arg(short = 'c', long = "count", default_value = "1")]
        quantity: usize,
    },
    /// Generate UUID v5 (namespace + SHA-1 hash)
    V5 {
        /// Namespace to use
        #[arg(short, long)]
        namespace: Namespace,
        /// Name to hash
        #[arg(short = 'N', long)]
        name: String,
        /// Number of UUIDs to generate
        #[arg(short = 'c', long = "count", default_value = "1")]
        quantity: usize,
    },
}

impl Tool for UUIDTool {
    fn cli() -> Command {
        UUIDTool::command()
    }

    fn execute(&self) -> anyhow::Result<Option<Output>> {
        let uuids: Vec<String> = match &self.command {
            UUIDCommand::V1 { quantity } => (0..*quantity)
                .map(|_| Uuid::now_v1(&[0, 1, 2, 3, 4, 5]).to_string())
                .collect(),
            UUIDCommand::V3 {
                namespace,
                name,
                quantity,
            } => {
                let ns_uuid = namespace.to_uuid();

                (0..*quantity)
                    .map(|_| Uuid::new_v3(&ns_uuid, name.as_bytes()).to_string())
                    .collect()
            }
            UUIDCommand::V4 { quantity } => {
                (0..*quantity).map(|_| Uuid::new_v4().to_string()).collect()
            }
            UUIDCommand::V5 {
                namespace,
                name,
                quantity,
            } => {
                let ns_uuid = namespace.to_uuid();

                (0..*quantity)
                    .map(|_| Uuid::new_v5(&ns_uuid, name.as_bytes()).to_string())
                    .collect()
            }
        };

        Ok(Some(Output::JsonValue(serde_json::json!(uuids))))
    }
}

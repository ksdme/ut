use crate::{
    args::StringInput,
    tool::{Output, Tool},
};
use anyhow::Context;
use clap::{Command, CommandFactory, Parser, Subcommand};
use ulid::Ulid;

#[derive(Parser, Debug)]
#[command(
    name = "ulid",
    about = "Generate and manipulate ULIDs (Universally Unique Lexicographically Sortable Identifiers)"
)]
pub struct ULIDTool {
    #[command(subcommand)]
    command: ULIDCommand,
}

#[derive(Subcommand, Debug)]
enum ULIDCommand {
    /// Generate new ULIDs (default)
    #[clap(visible_alias = "g")]
    Generate {
        /// Number of ULIDs to generate
        #[arg(short = 'c', long = "count", default_value = "1")]
        quantity: usize,
    },

    /// Parse and inspect a ULID
    Parse {
        /// ULID string to parse (use "-" for stdin)
        ulid: StringInput,
    },

    /// Validate a ULID string
    Validate {
        /// ULID string to validate (use "-" for stdin)
        ulid: StringInput,
    },

    /// Convert ULID to UUID
    ToUUID {
        /// ULID to convert (use "-" for stdin)
        ulid: StringInput,
    },

    /// Convert UUID to ULID
    FromUUID {
        /// UUID to convert (use "-" for stdin)
        uuid: StringInput,
    },
}

impl Tool for ULIDTool {
    fn cli() -> Command {
        ULIDTool::command()
    }

    fn execute(&self) -> anyhow::Result<Option<Output>> {
        let result = match &self.command {
            ULIDCommand::Generate { quantity } => {
                let ulids: Vec<String> = (0..*quantity).map(|_| Ulid::new().to_string()).collect();
                serde_json::json!(ulids)
            }

            ULIDCommand::Parse { ulid } => {
                let parsed = Ulid::from_string(ulid.as_ref()).context("Invalid ULID format")?;

                let timestamp_ms = parsed.timestamp_ms();
                let datetime_secs = timestamp_ms / 1000;

                // Convert to ISO 8601 format using jiff
                let datetime_str = jiff::Timestamp::from_second(datetime_secs as i64)
                    .map(|ts| ts.to_string())
                    .unwrap_or_else(|_| "Invalid timestamp".to_string());

                serde_json::json!({
                    "datetime": datetime_str,
                    "timestamp_ms": timestamp_ms,
                    "bytes": parsed.to_bytes(),
                })
            }

            ULIDCommand::Validate { ulid } => {
                // TODO: Also use proper exit code.
                serde_json::json!(if Ulid::from_string(ulid.as_ref()).is_ok() {
                    "valid"
                } else {
                    "invalid"
                })
            }

            ULIDCommand::ToUUID { ulid } => {
                let parsed = Ulid::from_string(ulid.as_ref()).context("Invalid ULID format")?;
                let uuid: uuid::Uuid = parsed.into();
                serde_json::json!({
                    "uuid": uuid.to_string(),
                })
            }

            ULIDCommand::FromUUID { uuid } => {
                let parsed_uuid =
                    uuid::Uuid::parse_str(uuid.as_ref()).context("Invalid UUID format")?;

                // Convert UUID bytes to ULID
                let uuid_bytes = parsed_uuid.as_bytes();
                let ulid = Ulid::from_bytes(*uuid_bytes);

                serde_json::json!({
                    "ulid": ulid.to_string(),
                })
            }
        };

        Ok(Some(Output::JsonValue(result)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_single() {
        let tool = ULIDTool {
            command: ULIDCommand::Generate { quantity: 1 },
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };

        let ulids = val.as_array().unwrap();
        assert_eq!(ulids.len(), 1);

        // Verify it's a valid ULID format (26 characters)
        let ulid_str = ulids[0].as_str().unwrap();
        assert_eq!(ulid_str.len(), 26);
        assert!(Ulid::from_string(ulid_str).is_ok());
    }

    #[test]
    fn test_generate_multiple() {
        let tool = ULIDTool {
            command: ULIDCommand::Generate { quantity: 5 },
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };

        let ulids = val.as_array().unwrap();
        assert_eq!(ulids.len(), 5);

        // Verify all are valid ULIDs
        for ulid in ulids {
            let ulid_str = ulid.as_str().unwrap();
            assert!(Ulid::from_string(ulid_str).is_ok());
        }
    }

    #[test]
    fn test_validate_valid() {
        let valid_ulid = Ulid::new().to_string();
        let tool = ULIDTool {
            command: ULIDCommand::Validate {
                ulid: StringInput(valid_ulid),
            },
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };

        assert_eq!(val.as_str().unwrap(), "valid");
    }

    #[test]
    fn test_validate_invalid() {
        let tool = ULIDTool {
            command: ULIDCommand::Validate {
                ulid: StringInput("invalid-ulid".to_string()),
            },
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };

        assert_eq!(val.as_str().unwrap(), "invalid");
    }

    #[test]
    fn test_parse() {
        let ulid = Ulid::new();

        let tool = ULIDTool {
            command: ULIDCommand::Parse {
                ulid: StringInput(ulid.to_string()),
            },
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };

        assert!(val["timestamp_ms"].as_u64().is_some());
        assert!(val["datetime"].as_str().is_some());
    }

    #[test]
    fn test_ulid_to_uuid_conversion() {
        let ulid = Ulid::new();

        let tool = ULIDTool {
            command: ULIDCommand::ToUUID {
                ulid: StringInput(ulid.to_string()),
            },
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };

        let uuid_str = val["uuid"].as_str().unwrap();
        assert!(uuid::Uuid::parse_str(uuid_str).is_ok());
    }

    #[test]
    fn test_uuid_to_ulid_conversion() {
        let uuid = uuid::Uuid::new_v4();

        let tool = ULIDTool {
            command: ULIDCommand::FromUUID {
                uuid: StringInput(uuid.to_string()),
            },
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };

        let ulid_str = val["ulid"].as_str().unwrap();
        assert!(Ulid::from_string(ulid_str).is_ok());
    }

    #[test]
    fn test_roundtrip_ulid_uuid_ulid() {
        let original_ulid = Ulid::new();
        let original_ulid_str = original_ulid.to_string();

        // Convert to UUID
        let to_uuid_tool = ULIDTool {
            command: ULIDCommand::ToUUID {
                ulid: StringInput(original_ulid_str.clone()),
            },
        };
        let uuid_result = to_uuid_tool.execute().unwrap().unwrap();
        let Output::JsonValue(uuid_val) = uuid_result else {
            unreachable!()
        };
        let uuid_str = uuid_val["uuid"].as_str().unwrap().to_string();

        // Convert back to ULID
        let from_uuid_tool = ULIDTool {
            command: ULIDCommand::FromUUID {
                uuid: StringInput(uuid_str.to_string()),
            },
        };
        let ulid_result = from_uuid_tool.execute().unwrap().unwrap();
        let Output::JsonValue(ulid_val) = ulid_result else {
            unreachable!()
        };
        let final_ulid_str = ulid_val["ulid"].as_str().unwrap();

        // Should match original
        assert_eq!(final_ulid_str, original_ulid_str);
    }
}

use crate::tool::{Output, Tool};
use anyhow::Context;
use clap::{Command, CommandFactory, Parser, Subcommand};
use ulid::Ulid;

#[derive(Parser, Debug)]
#[command(
    name = "ulid",
    about = "Generate and manipulate ULIDs (Universally Unique Lexicographically Sortable Identifiers)"
)]
pub struct UlidTool {
    #[command(subcommand)]
    command: Option<UlidCommand>,

    /// Number of ULIDs to generate (when no subcommand specified)
    #[arg(short = 'c', long = "count", default_value = "1")]
    quantity: usize,
}

#[derive(Subcommand, Debug)]
enum UlidCommand {
    /// Generate new ULIDs (default)
    Generate {
        /// Number of ULIDs to generate
        #[arg(short = 'c', long = "count", default_value = "1")]
        quantity: usize,
    },

    /// Parse and inspect a ULID
    Parse {
        /// ULID string to parse
        ulid: String,
    },

    /// Validate a ULID string
    Validate {
        /// ULID string to validate
        ulid: String,
    },

    /// Convert ULID to UUID
    ToUuid {
        /// ULID to convert
        ulid: String,
    },

    /// Convert UUID to ULID
    FromUuid {
        /// UUID to convert
        uuid: String,
    },
}

impl Tool for UlidTool {
    fn cli() -> Command {
        UlidTool::command()
    }

    fn execute(&self) -> anyhow::Result<Option<Output>> {
        let result = match &self.command {
            None => {
                // Default behavior: generate ULIDs
                let ulids: Vec<String> = (0..self.quantity)
                    .map(|_| Ulid::new().to_string())
                    .collect();
                serde_json::json!(ulids)
            }

            Some(UlidCommand::Generate { quantity }) => {
                let ulids: Vec<String> = (0..*quantity)
                    .map(|_| Ulid::new().to_string())
                    .collect();
                serde_json::json!(ulids)
            }

            Some(UlidCommand::Parse { ulid }) => {
                let parsed = Ulid::from_string(ulid).context("Invalid ULID format")?;

                let timestamp_ms = parsed.timestamp_ms();
                let datetime_secs = timestamp_ms / 1000;

                // Convert to ISO 8601 format using jiff
                let datetime_str = jiff::Timestamp::from_second(datetime_secs as i64)
                    .map(|ts| ts.to_string())
                    .unwrap_or_else(|_| "Invalid timestamp".to_string());

                serde_json::json!({
                    "ulid": ulid,
                    "timestamp_ms": timestamp_ms,
                    "datetime": datetime_str,
                    "bytes": parsed.to_bytes(),
                })
            }

            Some(UlidCommand::Validate { ulid }) => {
                let is_valid = Ulid::from_string(ulid).is_ok();
                serde_json::json!({
                    "ulid": ulid,
                    "valid": is_valid,
                })
            }

            Some(UlidCommand::ToUuid { ulid }) => {
                let parsed = Ulid::from_string(ulid).context("Invalid ULID format")?;
                let uuid: uuid::Uuid = parsed.into();
                serde_json::json!({
                    "ulid": ulid,
                    "uuid": uuid.to_string(),
                })
            }

            Some(UlidCommand::FromUuid { uuid }) => {
                let parsed_uuid = uuid::Uuid::parse_str(uuid).context("Invalid UUID format")?;

                // Convert UUID bytes to ULID
                let uuid_bytes = parsed_uuid.as_bytes();
                let ulid = Ulid::from_bytes(*uuid_bytes);

                serde_json::json!({
                    "uuid": uuid,
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
        let tool = UlidTool {
            command: None,
            quantity: 1,
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
        let tool = UlidTool {
            command: Some(UlidCommand::Generate { quantity: 5 }),
            quantity: 1,
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
        let tool = UlidTool {
            command: Some(UlidCommand::Validate { ulid: valid_ulid }),
            quantity: 1,
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };

        assert_eq!(val["valid"].as_bool().unwrap(), true);
    }

    #[test]
    fn test_validate_invalid() {
        let tool = UlidTool {
            command: Some(UlidCommand::Validate {
                ulid: "invalid-ulid".to_string(),
            }),
            quantity: 1,
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };

        assert_eq!(val["valid"].as_bool().unwrap(), false);
    }

    #[test]
    fn test_parse() {
        let ulid = Ulid::new();
        let ulid_str = ulid.to_string();

        let tool = UlidTool {
            command: Some(UlidCommand::Parse {
                ulid: ulid_str.clone(),
            }),
            quantity: 1,
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };

        assert_eq!(val["ulid"].as_str().unwrap(), ulid_str);
        assert!(val["timestamp_ms"].as_u64().is_some());
        assert!(val["datetime"].as_str().is_some());
    }

    #[test]
    fn test_ulid_to_uuid_conversion() {
        let ulid = Ulid::new();
        let ulid_str = ulid.to_string();

        let tool = UlidTool {
            command: Some(UlidCommand::ToUuid {
                ulid: ulid_str.clone(),
            }),
            quantity: 1,
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };

        assert_eq!(val["ulid"].as_str().unwrap(), ulid_str);
        let uuid_str = val["uuid"].as_str().unwrap();
        assert!(uuid::Uuid::parse_str(uuid_str).is_ok());
    }

    #[test]
    fn test_uuid_to_ulid_conversion() {
        let uuid = uuid::Uuid::new_v4();
        let uuid_str = uuid.to_string();

        let tool = UlidTool {
            command: Some(UlidCommand::FromUuid {
                uuid: uuid_str.clone(),
            }),
            quantity: 1,
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };

        assert_eq!(val["uuid"].as_str().unwrap(), uuid_str);
        let ulid_str = val["ulid"].as_str().unwrap();
        assert!(Ulid::from_string(ulid_str).is_ok());
    }

    #[test]
    fn test_roundtrip_ulid_uuid_ulid() {
        let original_ulid = Ulid::new();
        let original_ulid_str = original_ulid.to_string();

        // Convert to UUID
        let to_uuid_tool = UlidTool {
            command: Some(UlidCommand::ToUuid {
                ulid: original_ulid_str.clone(),
            }),
            quantity: 1,
        };
        let uuid_result = to_uuid_tool.execute().unwrap().unwrap();
        let Output::JsonValue(uuid_val) = uuid_result else {
            unreachable!()
        };
        let uuid_str = uuid_val["uuid"].as_str().unwrap().to_string();

        // Convert back to ULID
        let from_uuid_tool = UlidTool {
            command: Some(UlidCommand::FromUuid { uuid: uuid_str }),
            quantity: 1,
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

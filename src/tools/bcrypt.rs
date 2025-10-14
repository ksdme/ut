use crate::args::StringInput;
use crate::tool::{Output, Tool};
use anyhow::{Context, Result};
use clap::{Command, CommandFactory, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "bcrypt", about = "bcrypt hashing and verification utilities")]
pub struct BcryptTool {
    #[command(subcommand)]
    command: BcryptCommand,
}

#[derive(Subcommand, Debug)]
enum BcryptCommand {
    /// Hash a password using bcrypt
    Hash {
        /// Password to hash
        password: StringInput,

        /// Cost factor (4-31, default: 12). Higher values are more secure but slower
        #[arg(short, long, default_value = "12")]
        cost: u32,
    },
    /// Verify a password against a bcrypt hash
    Verify {
        /// Password to verify
        password: StringInput,

        /// Bcrypt hash to verify against
        hash: String,
    },
}

impl Tool for BcryptTool {
    fn cli() -> Command {
        BcryptTool::command()
    }

    fn execute(&self) -> Result<Option<Output>> {
        match &self.command {
            BcryptCommand::Hash { password, cost } => {
                // Validate cost
                if *cost < 4 || *cost > 31 {
                    anyhow::bail!("Cost must be between 4 and 31");
                }

                Ok(Some(Output::JsonValue(serde_json::json!(
                    bcrypt::hash(password.as_ref(), *cost).context("Failed to hash password")?
                ))))
            }
            BcryptCommand::Verify { password, hash } => {
                let is_valid =
                    bcrypt::verify(password.as_ref(), hash).context("Failed to verify password")?;

                // TODO: Also use proper exit code.
                Ok(Some(Output::JsonValue(serde_json::json!(if is_valid {
                    "valid"
                } else {
                    "invalid"
                }))))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::args::StringInput;

    #[test]
    fn test_hash_default_cost() {
        let tool = BcryptTool {
            command: BcryptCommand::Hash {
                password: StringInput("test_password".to_string()),
                cost: 12,
            },
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            panic!("Expected JsonValue output");
        };

        let hash = val.as_str().unwrap();
        // Bcrypt hashes start with $2b$ or similar and are 60 chars long
        assert!(hash.starts_with("$2"));
        assert_eq!(hash.len(), 60);
    }

    #[test]
    fn test_hash_custom_cost() {
        let tool = BcryptTool {
            command: BcryptCommand::Hash {
                password: StringInput("test_password".to_string()),
                cost: 8,
            },
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            panic!("Expected JsonValue output");
        };

        let hash = val.as_str().unwrap();
        assert!(hash.starts_with("$2"));
        assert_eq!(hash.len(), 60);
    }

    #[test]
    fn test_hash_cost_too_low() {
        let tool = BcryptTool {
            command: BcryptCommand::Hash {
                password: StringInput("test_password".to_string()),
                cost: 3,
            },
        };
        let result = tool.execute();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Cost must be between 4 and 31")
        );
    }

    #[test]
    fn test_hash_cost_too_high() {
        let tool = BcryptTool {
            command: BcryptCommand::Hash {
                password: StringInput("test_password".to_string()),
                cost: 32,
            },
        };
        let result = tool.execute();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Cost must be between 4 and 31")
        );
    }

    #[test]
    fn test_verify_correct_password() {
        // First hash a password
        let hash_tool = BcryptTool {
            command: BcryptCommand::Hash {
                password: StringInput("correct_password".to_string()),
                cost: 6, // Use lower cost for faster tests
            },
        };
        let hash_result = hash_tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = hash_result else {
            panic!("Expected JsonValue output");
        };
        let hash = val.as_str().unwrap().to_string();

        // Now verify the correct password
        let verify_tool = BcryptTool {
            command: BcryptCommand::Verify {
                password: StringInput("correct_password".to_string()),
                hash,
            },
        };
        let verify_result = verify_tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = verify_result else {
            panic!("Expected JsonValue output");
        };
        assert_eq!(val.as_str().unwrap(), "valid");
    }

    #[test]
    fn test_verify_incorrect_password() {
        // First hash a password
        let hash_tool = BcryptTool {
            command: BcryptCommand::Hash {
                password: StringInput("correct_password".to_string()),
                cost: 6, // Use lower cost for faster tests
            },
        };
        let hash_result = hash_tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = hash_result else {
            panic!("Expected JsonValue output");
        };
        let hash = val.as_str().unwrap().to_string();

        // Now verify an incorrect password
        let verify_tool = BcryptTool {
            command: BcryptCommand::Verify {
                password: StringInput("wrong_password".to_string()),
                hash,
            },
        };
        let verify_result = verify_tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = verify_result else {
            panic!("Expected JsonValue output");
        };
        assert_eq!(val.as_str().unwrap(), "invalid");
    }

    #[test]
    fn test_verify_invalid_hash_format() {
        let verify_tool = BcryptTool {
            command: BcryptCommand::Verify {
                password: StringInput("test_password".to_string()),
                hash: "not_a_valid_bcrypt_hash".to_string(),
            },
        };
        let result = verify_tool.execute();
        assert!(result.is_err());
    }

    #[test]
    fn test_hash_empty_password() {
        let tool = BcryptTool {
            command: BcryptCommand::Hash {
                password: StringInput("".to_string()),
                cost: 6,
            },
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            panic!("Expected JsonValue output");
        };

        let hash = val.as_str().unwrap();
        assert!(hash.starts_with("$2"));
        assert_eq!(hash.len(), 60);
    }

    #[test]
    fn test_verify_empty_password() {
        // Hash an empty password
        let hash_tool = BcryptTool {
            command: BcryptCommand::Hash {
                password: StringInput("".to_string()),
                cost: 6,
            },
        };
        let hash_result = hash_tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = hash_result else {
            panic!("Expected JsonValue output");
        };
        let hash = val.as_str().unwrap().to_string();

        // Verify with empty password
        let verify_tool = BcryptTool {
            command: BcryptCommand::Verify {
                password: StringInput("".to_string()),
                hash,
            },
        };
        let verify_result = verify_tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = verify_result else {
            panic!("Expected JsonValue output");
        };
        assert_eq!(val.as_str().unwrap(), "valid");
    }

    #[test]
    fn test_hash_special_characters() {
        let tool = BcryptTool {
            command: BcryptCommand::Hash {
                password: StringInput("p@ssw0rd!#$%^&*()".to_string()),
                cost: 6,
            },
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            panic!("Expected JsonValue output");
        };

        let hash = val.as_str().unwrap();
        assert!(hash.starts_with("$2"));
        assert_eq!(hash.len(), 60);
    }

    #[test]
    fn test_hash_unicode_characters() {
        let tool = BcryptTool {
            command: BcryptCommand::Hash {
                password: StringInput("密码🔒".to_string()),
                cost: 6,
            },
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            panic!("Expected JsonValue output");
        };

        let hash = val.as_str().unwrap();
        assert!(hash.starts_with("$2"));
        assert_eq!(hash.len(), 60);
    }

    #[test]
    fn test_verify_with_known_hash() {
        // Generate a hash and verify it with the known password
        let password = "test123";
        let hash_tool = BcryptTool {
            command: BcryptCommand::Hash {
                password: StringInput(password.to_string()),
                cost: 6,
            },
        };
        let hash_result = hash_tool.execute().unwrap().unwrap();
        let Output::JsonValue(hash_val) = hash_result else {
            panic!("Expected JsonValue output");
        };
        let hash = hash_val.as_str().unwrap().to_string();

        // Now verify with the correct password
        let verify_tool = BcryptTool {
            command: BcryptCommand::Verify {
                password: StringInput(password.to_string()),
                hash: hash.clone(),
            },
        };
        let verify_result = verify_tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = verify_result else {
            panic!("Expected JsonValue output");
        };
        assert_eq!(val.as_str().unwrap(), "valid");

        // Verify with wrong password
        let verify_tool2 = BcryptTool {
            command: BcryptCommand::Verify {
                password: StringInput("wrongpassword".to_string()),
                hash,
            },
        };
        let verify_result2 = verify_tool2.execute().unwrap().unwrap();

        let Output::JsonValue(val2) = verify_result2 else {
            panic!("Expected JsonValue output");
        };
        assert_eq!(val2.as_str().unwrap(), "invalid");
    }

    #[test]
    fn test_same_password_different_hashes() {
        // Hash the same password twice
        let tool1 = BcryptTool {
            command: BcryptCommand::Hash {
                password: StringInput("same_password".to_string()),
                cost: 6,
            },
        };
        let result1 = tool1.execute().unwrap().unwrap();

        let tool2 = BcryptTool {
            command: BcryptCommand::Hash {
                password: StringInput("same_password".to_string()),
                cost: 6,
            },
        };
        let result2 = tool2.execute().unwrap().unwrap();

        let Output::JsonValue(val1) = result1 else {
            panic!("Expected JsonValue output");
        };
        let Output::JsonValue(val2) = result2 else {
            panic!("Expected JsonValue output");
        };

        // Hashes should be different (due to random salt)
        assert_ne!(val1.as_str().unwrap(), val2.as_str().unwrap());

        // But both should verify the same password
        let hash1 = val1.as_str().unwrap().to_string();
        let verify_tool = BcryptTool {
            command: BcryptCommand::Verify {
                password: StringInput("same_password".to_string()),
                hash: hash1,
            },
        };
        let verify_result = verify_tool.execute().unwrap().unwrap();
        let Output::JsonValue(val) = verify_result else {
            panic!("Expected JsonValue output");
        };
        assert_eq!(val.as_str().unwrap(), "valid");
    }
}

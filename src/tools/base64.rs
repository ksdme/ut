use crate::{
    args::StringInput,
    tool::{Output, Tool},
};
use anyhow::Context;
use base64::{Engine as _, engine::general_purpose};
use clap::{Command, CommandFactory, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "base64", about = "Base64 encode and decode utilities")]
pub struct Base64Tool {
    #[command(subcommand)]
    command: Base64Command,
}

#[derive(Subcommand, Debug)]
enum Base64Command {
    /// Base64 encode contents
    Encode {
        /// Input to encode
        text: StringInput,
        /// Encode with urlsafe character set
        #[arg(long)]
        urlsafe: bool,
    },
    /// Base64 decode contents
    Decode {
        /// Input to decode
        text: StringInput,
        /// Decode with urlsafe character set
        #[arg(long)]
        urlsafe: bool,
    },
}

impl Tool for Base64Tool {
    fn cli() -> Command {
        Base64Tool::command()
    }

    fn execute(&self) -> anyhow::Result<Option<Output>> {
        match &self.command {
            Base64Command::Encode { text, urlsafe } => {
                let encoded = if *urlsafe {
                    general_purpose::URL_SAFE.encode(&text.0)
                } else {
                    general_purpose::STANDARD.encode(&text.0)
                };

                Ok(Some(Output::JsonValue(serde_json::json!(encoded))))
            }
            Base64Command::Decode { text, urlsafe } => {
                let engine = if *urlsafe {
                    &general_purpose::URL_SAFE
                } else {
                    &general_purpose::STANDARD
                };

                Ok(Some(Output::Bytes(
                    engine.decode(&text.0).context("Could not decode base64")?,
                )))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::args::StringInput;

    #[test]
    fn test_encode_standard() {
        let tool = Base64Tool {
            command: Base64Command::Encode {
                text: StringInput("Hello, World!".to_string()),
                urlsafe: false,
            },
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };
        assert_eq!(val.as_str().unwrap(), "SGVsbG8sIFdvcmxkIQ==");
    }

    #[test]
    fn test_encode_urlsafe() {
        let tool = Base64Tool {
            command: Base64Command::Encode {
                text: StringInput("Hello>>World??".to_string()),
                urlsafe: true,
            },
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };
        // URL-safe encoding uses - and _ instead of + and /
        assert_eq!(val.as_str().unwrap(), "SGVsbG8-PldvcmxkPz8=");
    }

    #[test]
    fn test_encode_empty_string() {
        let tool = Base64Tool {
            command: Base64Command::Encode {
                text: StringInput("".to_string()),
                urlsafe: false,
            },
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };
        assert_eq!(val.as_str().unwrap(), "");
    }

    #[test]
    fn test_encode_binary_data() {
        let tool = Base64Tool {
            command: Base64Command::Encode {
                text: StringInput("\x00\x01\x02\x03".to_string()),
                urlsafe: false,
            },
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };
        assert_eq!(val.as_str().unwrap(), "AAECAw==");
    }

    #[test]
    fn test_decode_standard() {
        let tool = Base64Tool {
            command: Base64Command::Decode {
                text: StringInput("SGVsbG8sIFdvcmxkIQ==".to_string()),
                urlsafe: false,
            },
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::Bytes(bytes) = result else {
            unreachable!()
        };
        assert_eq!(String::from_utf8(bytes).unwrap(), "Hello, World!");
    }

    #[test]
    fn test_decode_urlsafe() {
        let tool = Base64Tool {
            command: Base64Command::Decode {
                text: StringInput("SGVsbG8-PldvcmxkPz8=".to_string()),
                urlsafe: true,
            },
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::Bytes(bytes) = result else {
            unreachable!()
        };
        assert_eq!(String::from_utf8(bytes).unwrap(), "Hello>>World??");
    }

    #[test]
    fn test_decode_empty_string() {
        let tool = Base64Tool {
            command: Base64Command::Decode {
                text: StringInput("".to_string()),
                urlsafe: false,
            },
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::Bytes(bytes) = result else {
            unreachable!()
        };
        assert_eq!(bytes, Vec::<u8>::new());
    }

    #[test]
    fn test_decode_binary_data() {
        let tool = Base64Tool {
            command: Base64Command::Decode {
                text: StringInput("AAECAw==".to_string()),
                urlsafe: false,
            },
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::Bytes(bytes) = result else {
            unreachable!()
        };
        assert_eq!(bytes, vec![0u8, 1, 2, 3]);
    }

    #[test]
    fn test_decode_invalid_base64() {
        let tool = Base64Tool {
            command: Base64Command::Decode {
                text: StringInput("Not valid base64!!!".to_string()),
                urlsafe: false,
            },
        };
        let result = tool.execute();

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Could not decode base64")
        );
    }

    #[test]
    fn test_encode_decode_roundtrip_standard() {
        let original = "The quick brown fox jumps over the lazy dog";

        let encode_tool = Base64Tool {
            command: Base64Command::Encode {
                text: StringInput(original.to_string()),
                urlsafe: false,
            },
        };
        let encoded = encode_tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = encoded else {
            unreachable!()
        };
        let encoded_str = val.as_str().unwrap().to_string();

        let decode_tool = Base64Tool {
            command: Base64Command::Decode {
                text: StringInput(encoded_str),
                urlsafe: false,
            },
        };
        let decoded = decode_tool.execute().unwrap().unwrap();

        let Output::Bytes(bytes) = decoded else {
            unreachable!()
        };
        assert_eq!(String::from_utf8(bytes).unwrap(), original);
    }

    #[test]
    fn test_encode_decode_roundtrip_urlsafe() {
        let original = "Special chars: +/=?&";

        let encode_tool = Base64Tool {
            command: Base64Command::Encode {
                text: StringInput(original.to_string()),
                urlsafe: true,
            },
        };
        let encoded = encode_tool.execute().unwrap().unwrap();
        let Output::JsonValue(val) = encoded else {
            unreachable!()
        };
        let encoded_str = val.as_str().unwrap().to_string();

        let decode_tool = Base64Tool {
            command: Base64Command::Decode {
                text: StringInput(encoded_str),
                urlsafe: true,
            },
        };
        let decoded = decode_tool.execute().unwrap().unwrap();

        let Output::Bytes(bytes) = decoded else {
            unreachable!()
        };
        assert_eq!(String::from_utf8(bytes).unwrap(), original);
    }
}

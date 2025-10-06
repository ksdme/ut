use crate::tool::{Output, Tool};
use anyhow::Context;
use clap::{Command, CommandFactory, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "url", about = "URL encode and decode utilities")]
pub struct UrlTool {
    #[command(subcommand)]
    command: UrlCommand,
}

#[derive(Subcommand, Debug)]
enum UrlCommand {
    /// URL encode text
    Encode {
        /// Text to URL encode
        text: String,
    },
    /// URL decode text
    Decode {
        /// Text to URL decode
        text: String,
    },
}

impl Tool for UrlTool {
    fn cli() -> Command {
        UrlTool::command()
    }

    fn execute(&self) -> anyhow::Result<Option<Output>> {
        let result = match &self.command {
            UrlCommand::Encode { text } => urlencoding::encode(text).into_owned(),
            UrlCommand::Decode { text } => urlencoding::decode(text)
                .context("Could not decode")?
                .into_owned(),
        };

        Ok(Some(Output::JsonValue(serde_json::json!(result))))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_simple() {
        let tool = UrlTool {
            command: UrlCommand::Encode {
                text: "hello world".to_string(),
            },
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };
        assert_eq!(val.as_str().unwrap(), "hello%20world");
    }

    #[test]
    fn test_encode_special_chars() {
        let tool = UrlTool {
            command: UrlCommand::Encode {
                text: "hello@world.com?key=value&foo=bar".to_string(),
            },
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };
        assert_eq!(
            val.as_str().unwrap(),
            "hello%40world.com%3Fkey%3Dvalue%26foo%3Dbar"
        );
    }

    #[test]
    fn test_encode_unicode() {
        let tool = UrlTool {
            command: UrlCommand::Encode {
                text: "Hello 世界".to_string(),
            },
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };
        assert_eq!(val.as_str().unwrap(), "Hello%20%E4%B8%96%E7%95%8C");
    }

    #[test]
    fn test_encode_empty_string() {
        let tool = UrlTool {
            command: UrlCommand::Encode {
                text: "".to_string(),
            },
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };
        assert_eq!(val.as_str().unwrap(), "");
    }

    #[test]
    fn test_encode_already_encoded() {
        let tool = UrlTool {
            command: UrlCommand::Encode {
                text: "hello%20world".to_string(),
            },
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };
        assert_eq!(val.as_str().unwrap(), "hello%2520world");
    }

    #[test]
    fn test_decode_simple() {
        let tool = UrlTool {
            command: UrlCommand::Decode {
                text: "hello%20world".to_string(),
            },
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };
        assert_eq!(val.as_str().unwrap(), "hello world");
    }

    #[test]
    fn test_decode_special_chars() {
        let tool = UrlTool {
            command: UrlCommand::Decode {
                text: "hello%40world.com%3Fkey%3Dvalue%26foo%3Dbar".to_string(),
            },
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };
        assert_eq!(val.as_str().unwrap(), "hello@world.com?key=value&foo=bar");
    }

    #[test]
    fn test_decode_unicode() {
        let tool = UrlTool {
            command: UrlCommand::Decode {
                text: "Hello%20%E4%B8%96%E7%95%8C".to_string(),
            },
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };
        assert_eq!(val.as_str().unwrap(), "Hello 世界");
    }

    #[test]
    fn test_decode_empty_string() {
        let tool = UrlTool {
            command: UrlCommand::Decode {
                text: "".to_string(),
            },
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };
        assert_eq!(val.as_str().unwrap(), "");
    }

    #[test]
    fn test_decode_plus_sign() {
        let tool = UrlTool {
            command: UrlCommand::Decode {
                text: "hello+world".to_string(),
            },
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };
        assert_eq!(val.as_str().unwrap(), "hello+world");
    }

    #[test]
    fn test_decode_partial_encoding() {
        let tool = UrlTool {
            command: UrlCommand::Decode {
                text: "hello%ZZworld".to_string(),
            },
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };
        // Invalid percent encoding is returned as-is
        assert_eq!(val.as_str().unwrap(), "hello%ZZworld");
    }

    #[test]
    fn test_encode_decode_roundtrip() {
        let original = "Hello World! @#$%^&*()";

        let encode_tool = UrlTool {
            command: UrlCommand::Encode {
                text: original.to_string(),
            },
        };
        let encoded = encode_tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = encoded else {
            unreachable!()
        };
        let encoded_str = val.as_str().unwrap().to_string();

        let decode_tool = UrlTool {
            command: UrlCommand::Decode { text: encoded_str },
        };
        let decoded = decode_tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = decoded else {
            unreachable!()
        };
        assert_eq!(val.as_str().unwrap(), original);
    }
}

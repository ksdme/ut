use crate::args::StringInput;
use crate::tool::{Output, Tool};
use anyhow::Context;
use clap::{Command, CommandFactory, Parser, Subcommand};
use url::Url;

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
        /// Text to URL encode (use "-" for stdin)
        text: StringInput,
    },
    /// URL decode text
    Decode {
        /// Text to URL decode (use "-" for stdin)
        text: StringInput,
    },
    /// Parse URL into its components
    Parse {
        /// URL to parse (use "-" for stdin)
        url: StringInput,
    },
}

impl Tool for UrlTool {
    fn cli() -> Command {
        UrlTool::command()
    }

    fn execute(&self) -> anyhow::Result<Option<Output>> {
        match &self.command {
            UrlCommand::Encode { text } => {
                let result = urlencoding::encode(text.as_ref()).into_owned();
                Ok(Some(Output::JsonValue(serde_json::json!(result))))
            }
            UrlCommand::Decode { text } => {
                let result = urlencoding::decode(text.as_ref())
                    .context("Could not decode")?
                    .into_owned();
                Ok(Some(Output::JsonValue(serde_json::json!(result))))
            }
            UrlCommand::Parse { url } => {
                let parsed = Url::parse(url.as_ref()).context("Could not parse URL")?;

                // Build query params as a JSON object
                let query_params: serde_json::Map<String, serde_json::Value> = parsed
                    .query_pairs()
                    .map(|(k, v)| (k.into_owned(), serde_json::json!(v)))
                    .collect();

                let result = serde_json::json!({
                    "scheme": parsed.scheme(),
                    "host": parsed.host_str(),
                    "port": parsed.port_or_known_default(),
                    "path": parsed.path(),
                    "query": parsed.query(),
                    "query_params": query_params,
                    "fragment": parsed.fragment(),
                    "username": parsed.username(),
                    "password": parsed.password(),
                });

                Ok(Some(Output::JsonValue(result)))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::args::StringInput;

    #[test]
    fn test_encode_simple() {
        let tool = UrlTool {
            command: UrlCommand::Encode {
                text: StringInput("hello world".to_string()),
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
                text: StringInput("hello@world.com?key=value&foo=bar".to_string()),
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
                text: StringInput("Hello 世界".to_string()),
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
                text: StringInput("".to_string()),
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
                text: StringInput("hello%20world".to_string()),
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
                text: StringInput("hello%20world".to_string()),
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
                text: StringInput("hello%40world.com%3Fkey%3Dvalue%26foo%3Dbar".to_string()),
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
                text: StringInput("Hello%20%E4%B8%96%E7%95%8C".to_string()),
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
                text: StringInput("".to_string()),
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
                text: StringInput("hello+world".to_string()),
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
                text: StringInput("hello%ZZworld".to_string()),
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
                text: StringInput(original.to_string()),
            },
        };
        let encoded = encode_tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = encoded else {
            unreachable!()
        };
        let encoded_str = val.as_str().unwrap().to_string();

        let decode_tool = UrlTool {
            command: UrlCommand::Decode {
                text: StringInput(encoded_str),
            },
        };
        let decoded = decode_tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = decoded else {
            unreachable!()
        };
        assert_eq!(val.as_str().unwrap(), original);
    }

    #[test]
    fn test_parse_basic_url() {
        let tool = UrlTool {
            command: UrlCommand::Parse {
                url: StringInput("https://example.com/path".to_string()),
            },
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };
        assert_eq!(val["scheme"], "https");
        assert_eq!(val["host"], "example.com");
        assert_eq!(val["port"], 443);
        assert_eq!(val["path"], "/path");
        assert!(val["query"].is_null());
        assert!(val["fragment"].is_null());
    }

    #[test]
    fn test_parse_url_with_query_params() {
        let tool = UrlTool {
            command: UrlCommand::Parse {
                url: StringInput("https://example.com/search?key1=value1&key2=value2".to_string()),
            },
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };
        assert_eq!(val["query"], "key1=value1&key2=value2");
        assert_eq!(val["query_params"]["key1"], "value1");
        assert_eq!(val["query_params"]["key2"], "value2");
    }

    #[test]
    fn test_parse_url_with_fragment() {
        let tool = UrlTool {
            command: UrlCommand::Parse {
                url: StringInput("https://example.com/page#section".to_string()),
            },
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };
        assert_eq!(val["fragment"], "section");
    }

    #[test]
    fn test_parse_url_with_credentials() {
        let tool = UrlTool {
            command: UrlCommand::Parse {
                url: StringInput("https://user:pass@example.com/".to_string()),
            },
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };
        assert_eq!(val["username"], "user");
        assert_eq!(val["password"], "pass");
    }

    #[test]
    fn test_parse_url_with_port() {
        let tool = UrlTool {
            command: UrlCommand::Parse {
                url: StringInput("http://localhost:8080/api".to_string()),
            },
        };
        let result = tool.execute().unwrap().unwrap();

        let Output::JsonValue(val) = result else {
            unreachable!()
        };
        assert_eq!(val["host"], "localhost");
        assert_eq!(val["port"], 8080);
        assert_eq!(val["scheme"], "http");
    }

    #[test]
    fn test_parse_invalid_url() {
        let tool = UrlTool {
            command: UrlCommand::Parse {
                url: StringInput("not-a-valid-url".to_string()),
            },
        };
        let result = tool.execute();
        assert!(result.is_err());
    }
}

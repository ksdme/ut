use clap::{Command, CommandFactory, Parser, Subcommand};
use nom::{
    IResult,
    branch::alt,
    bytes::complete::{tag, take_until, take_while1},
    character::complete::{char, digit1},
    combinator::{map, opt, recognize},
    multi::many0,
    sequence::{delimited, preceded, tuple},
};
use serde_json::{Value, json};

use crate::tool::{Output, Tool};

#[derive(Parser, Debug)]
#[command(name = "json", about = "JSON utilities")]
pub struct JsonTool {
    #[command(subcommand)]
    command: JsonCommand,
}

#[derive(Subcommand, Debug)]
enum JsonCommand {
    /// Build JSON from key-value pairs with dot notation and array support
    Builder {
        /// Key-value pairs in the format key=value (e.g., a.b.c=hello, "a.b[].c"=1 or "a.b[2].c"=false)
        #[arg(required = true)]
        inputs: Vec<String>,
    },
}

impl Tool for JsonTool {
    fn cli() -> Command {
        JsonTool::command()
    }

    fn execute(&self) -> anyhow::Result<Option<Output>> {
        match &self.command {
            JsonCommand::Builder { inputs } => {
                let mut root = json!({});

                for input in inputs {
                    let (path_parts, value) = parse_input(input)?;
                    set_nested_value(&mut root, path_parts, value)?;
                }

                let serialized = serde_json::to_string_pretty(&root)?;
                Ok(Some(Output::Text(serialized)))
            }
        }
    }
}

fn parse_input(input: &str) -> anyhow::Result<(Vec<PathPart>, Value)> {
    // Two-stage parsing:
    // 1. First, split input into key=value (input_parser)
    //    This extracts the raw key string and parses the value
    // 2. Then, parse the key string into path parts (path_parser)
    //    This allows better error messages - we know which input failed and why

    match input_parser(input) {
        Ok((remaining, (path_str, value))) => {
            if !remaining.is_empty() {
                return Err(anyhow::anyhow!(
                    "Failed to parse input completely, remaining: '{}'",
                    remaining
                ));
            }

            // Now parse the path string into structured path parts
            let (path_remaining, path_parts) = path_parser(&path_str)
                .map_err(|e| anyhow::anyhow!("Failed to parse path '{}': {}", path_str, e))?;

            if !path_remaining.is_empty() {
                return Err(anyhow::anyhow!(
                    "Failed to parse path completely, remaining: '{}'",
                    path_remaining
                ));
            }

            Ok((path_parts, value))
        }
        Err(e) => Err(anyhow::anyhow!("Failed to parse input '{}': {}", input, e)),
    }
}

// Parse the input at the '=' separator level
// Extracts the raw key string (before '=') and parses the value (after '=')
// The key string is returned as-is for later path parsing
fn input_parser(input: &str) -> IResult<&str, (String, Value)> {
    let (input, key) = parse_key(input)?;
    let (input, _) = char('=')(input)?;
    let (input, value) = json_value(input)?;

    Ok((input, (key, value)))
}

// Parse a key before the '=' separator
// This extracts the key string but doesn't interpret it as a path yet
fn parse_key(input: &str) -> IResult<&str, String> {
    alt((
        quoted_key,
        // Unquoted key - everything before '='
        map(take_while1(|c: char| c != '='), |s: &str| s.to_string()),
    ))(input)
}

// Parse a quoted key (e.g., "hello world" in path)
fn quoted_key(input: &str) -> IResult<&str, String> {
    map(
        delimited(char('"'), take_until("\""), char('"')),
        |s: &str| s.to_string(),
    )(input)
}

// Parse any JSON value with type detection
// Tries parsers in order: quoted string, boolean, null, float, integer, unquoted string
fn json_value(input: &str) -> IResult<&str, Value> {
    let input = input.trim();

    alt((
        quoted_string,
        boolean,
        null,
        float, // Try float before integer (more specific)
        integer,
        unquoted_string, // Fallback to string
    ))(input)
}

// Parse a quoted string value (e.g., "hello world")
fn quoted_string(input: &str) -> IResult<&str, Value> {
    map(
        delimited(char('"'), take_until("\""), char('"')),
        |s: &str| json!(s),
    )(input)
}

// Parse a boolean value (true or false)
fn boolean(input: &str) -> IResult<&str, Value> {
    alt((
        map(tag("true"), |_| json!(true)),
        map(tag("false"), |_| json!(false)),
    ))(input)
}

// Parse a null value
fn null(input: &str) -> IResult<&str, Value> {
    map(tag("null"), |_| json!(null))(input)
}

// Parse a floating point number (e.g., 3.14, -2.5, 1.5e10)
fn float(input: &str) -> IResult<&str, Value> {
    map(
        recognize(tuple((
            opt(char('-')),
            digit1,
            char('.'),
            digit1,
            opt(tuple((
                alt((char('e'), char('E'))),
                opt(alt((char('+'), char('-')))),
                digit1,
            ))),
        ))),
        |s: &str| json!(s.parse::<f64>().unwrap()),
    )(input)
}

// Parse an integer (e.g., 42, -10)
fn integer(input: &str) -> IResult<&str, Value> {
    map(recognize(tuple((opt(char('-')), digit1))), |s: &str| {
        json!(s.parse::<i64>().unwrap())
    })(input)
}

// Parse an unquoted string (fallback - consumes rest of input)
fn unquoted_string(input: &str) -> IResult<&str, Value> {
    map(nom::combinator::rest, |s: &str| json!(s))(input)
}

// Parse a complete path into PathPart components
// Examples: "a.b.c" → [Key("a"), Key("b"), Key("c")]
//           "a[0].b" → [Key("a"), ArrayIndex(0), Key("b")]
//           "a[].b" → [Key("a"), ArrayAppend, Key("b")]
fn path_parser(input: &str) -> IResult<&str, Vec<PathPart>> {
    let (input, first) = path_part(input)?;
    let (input, rest) = many0(alt((
        // Array access without dot: [0] or []
        alt((array_append, array_index)),
        // Key access with dot: .key
        preceded(char('.'), path_part),
    )))(input)?;

    let mut parts = vec![first];
    parts.extend(rest);

    Ok((input, parts))
}

// Parse a single path part (just a key)
// Array access is handled separately in path_parser
fn path_part(input: &str) -> IResult<&str, PathPart> {
    map(key, PathPart::Key)(input)
}

// Parse array append notation []
fn array_append(input: &str) -> IResult<&str, PathPart> {
    map(tag("[]"), |_| PathPart::ArrayAppend)(input)
}

// Parse array index notation [N] where N is a number
fn array_index(input: &str) -> IResult<&str, PathPart> {
    map(delimited(char('['), digit1, char(']')), |s: &str| {
        PathPart::ArrayIndex(s.parse().unwrap())
    })(input)
}

// Parse a key (quoted or unquoted) in a path
fn key(input: &str) -> IResult<&str, String> {
    alt((quoted_key, unquoted_key))(input)
}

// Parse an unquoted key in a path (alphanumeric, underscore, hyphen, spaces)
// Stops at '[' or '.' to allow array access without dots
fn unquoted_key(input: &str) -> IResult<&str, String> {
    map(
        take_while1(|c: char| {
            (c.is_alphanumeric() || c == '_' || c == '-' || c == ' ') && c != '[' && c != '.'
        }),
        |s: &str| s.to_string(),
    )(input)
}

// Set a value in the JSON structure at the specified path
// Creates intermediate objects/arrays as needed based on the path
fn set_nested_value(root: &mut Value, parts: Vec<PathPart>, value: Value) -> anyhow::Result<()> {
    if parts.is_empty() {
        return Err(anyhow::anyhow!("Empty path"));
    }

    let mut current = root;

    for (i, part) in parts.iter().enumerate() {
        let is_last = i == parts.len() - 1;

        // Determine what the next container should be (if not last)
        let next_is_array = !is_last
            && matches!(
                parts.get(i + 1),
                Some(PathPart::ArrayIndex(_)) | Some(PathPart::ArrayAppend)
            );

        match part {
            PathPart::Key(key) => {
                if !current.is_object() {
                    *current = json!({});
                }

                let obj = current.as_object_mut().unwrap();

                if is_last {
                    obj.insert(key.clone(), value.clone());
                    return Ok(());
                }

                if !obj.contains_key(key) {
                    if next_is_array {
                        obj.insert(key.clone(), json!([]));
                    } else {
                        obj.insert(key.clone(), json!({}));
                    }
                }

                current = obj.get_mut(key).unwrap();
            }
            PathPart::ArrayIndex(idx) => {
                if !current.is_array() {
                    *current = json!([]);
                }

                let arr = current.as_array_mut().unwrap();

                // Extend array with nulls if necessary
                while arr.len() <= *idx {
                    arr.push(json!(null));
                }

                if is_last {
                    arr[*idx] = value.clone();
                    return Ok(());
                }

                if arr[*idx].is_null() {
                    if next_is_array {
                        arr[*idx] = json!([]);
                    } else {
                        arr[*idx] = json!({});
                    }
                }

                current = &mut arr[*idx];
            }
            PathPart::ArrayAppend => {
                if !current.is_array() {
                    *current = json!([]);
                }

                let arr = current.as_array_mut().unwrap();

                if is_last {
                    arr.push(value.clone());
                    return Ok(());
                }

                if next_is_array {
                    arr.push(json!([]));
                } else {
                    arr.push(json!({}));
                }

                let last_idx = arr.len() - 1;
                current = &mut arr[last_idx];
            }
        }
    }

    Ok(())
}

/// Represents a single component in a JSON path
#[derive(Debug, Clone)]
enum PathPart {
    /// An object key (e.g., "name" in a.name)
    Key(String),
    /// An array index (e.g., 0 in a.[0])
    ArrayIndex(usize),
    /// Array append operation (e.g., [] in a.[])
    ArrayAppend,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_path() {
        let (_, parts) = path_parser("a.b.c").unwrap();
        assert_eq!(parts.len(), 3);

        let (_, parts) = path_parser("a[0].b").unwrap();
        assert_eq!(parts.len(), 3);

        let (_, parts) = path_parser("a[].b").unwrap();
        assert_eq!(parts.len(), 3);
    }

    #[test]
    fn test_simple_nested() {
        let tool = JsonTool {
            command: JsonCommand::Builder {
                inputs: vec!["a.b.c=hello".to_string()],
            },
        };
        let result = tool.execute().unwrap().unwrap();
        if let Output::Text(text) = result {
            let value: Value = serde_json::from_str(&text).unwrap();
            assert_eq!(value["a"]["b"]["c"], "hello");
        } else {
            panic!("Expected Text output");
        }
    }

    #[test]
    fn test_boolean_value() {
        let tool = JsonTool {
            command: JsonCommand::Builder {
                inputs: vec!["k.d.l=true".to_string()],
            },
        };
        let result = tool.execute().unwrap().unwrap();
        if let Output::Text(text) = result {
            let value: Value = serde_json::from_str(&text).unwrap();
            assert_eq!(value["k"]["d"]["l"], true);
        } else {
            panic!("Expected Text output");
        }
    }

    #[test]
    fn test_array_append() {
        let tool = JsonTool {
            command: JsonCommand::Builder {
                inputs: vec!["a.b[].c=1".to_string(), "a.b[].c=2".to_string()],
            },
        };
        let result = tool.execute().unwrap().unwrap();
        if let Output::Text(text) = result {
            let value: Value = serde_json::from_str(&text).unwrap();
            assert_eq!(value["a"]["b"][0]["c"], 1);
            assert_eq!(value["a"]["b"][1]["c"], 2);
        } else {
            panic!("Expected Text output");
        }
    }

    #[test]
    fn test_array_index() {
        let tool = JsonTool {
            command: JsonCommand::Builder {
                inputs: vec!["a.b[3].c=hello".to_string()],
            },
        };
        let result = tool.execute().unwrap().unwrap();
        if let Output::Text(text) = result {
            let value: Value = serde_json::from_str(&text).unwrap();
            assert_eq!(value["a"]["b"][3]["c"], "hello");
            assert_eq!(value["a"]["b"][0], Value::Null);
        } else {
            panic!("Expected Text output");
        }
    }

    #[test]
    fn test_quoted_key() {
        let tool = JsonTool {
            command: JsonCommand::Builder {
                inputs: vec![r#""hello world"=test"#.to_string()],
            },
        };
        let result = tool.execute().unwrap().unwrap();
        if let Output::Text(text) = result {
            let value: Value = serde_json::from_str(&text).unwrap();
            assert_eq!(value["hello world"], "test");
        } else {
            panic!("Expected Text output");
        }
    }
}

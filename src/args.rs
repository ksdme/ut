use std::io::{self, Read};
use std::str::FromStr;

/// A type for clap argument parsing that supports reading from stdin
/// when the value is "-" and allows escaping "-" with "\-".
#[derive(Debug, Clone)]
pub struct StringInput(pub String);

impl FromStr for StringInput {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "-" {
            // Read from stdin
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;
            Ok(StringInput(buffer))
        } else if s == r"\-" {
            // Escaped dash becomes literal dash
            Ok(StringInput("-".to_string()))
        } else {
            // Regular string
            Ok(StringInput(s.to_string()))
        }
    }
}

impl AsRef<str> for StringInput {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for StringInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

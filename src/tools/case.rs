use crate::args::StringInput;
use crate::tool::{Output, Tool};
use clap::{Command, CommandFactory, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "case", about = "Convert text between different case formats")]
pub struct CaseTool {
    #[command(subcommand)]
    command: CaseCommand,
}

#[derive(Subcommand, Debug)]
enum CaseCommand {
    /// Convert text to lowercase
    Lower {
        /// Text to convert (use "-" for stdin)
        text: StringInput,
    },
    /// Convert text to UPPERCASE
    Upper {
        /// Text to convert (use "-" for stdin)
        text: StringInput,
    },
    /// Convert text to camelCase
    Camel {
        /// Text to convert (use "-" for stdin)
        text: StringInput,
    },
    /// Convert text to Title Case
    Title {
        /// Text to convert (use "-" for stdin)
        text: StringInput,
    },
    /// Convert text to CONSTANT_CASE
    Constant {
        /// Text to convert (use "-" for stdin)
        text: StringInput,
    },
    /// Convert text to Header-Case
    Header {
        /// Text to convert (use "-" for stdin)
        text: StringInput,
    },
    /// Convert text to sentence case
    Sentence {
        /// Text to convert (use "-" for stdin)
        text: StringInput,
    },
    /// Convert text to snake_case
    Snake {
        /// Text to convert (use "-" for stdin)
        text: StringInput,
    },
}

impl Tool for CaseTool {
    fn cli() -> Command {
        CaseTool::command()
    }

    fn execute(&self) -> anyhow::Result<Option<Output>> {
        let result = match &self.command {
            CaseCommand::Lower { text } => to_lowercase(text.as_ref()),
            CaseCommand::Upper { text } => to_uppercase(text.as_ref()),
            CaseCommand::Camel { text } => to_camel_case(text.as_ref()),
            CaseCommand::Title { text } => to_title_case(text.as_ref()),
            CaseCommand::Constant { text } => to_constant_case(text.as_ref()),
            CaseCommand::Header { text } => to_header_case(text.as_ref()),
            CaseCommand::Sentence { text } => to_sentence_case(text.as_ref()),
            CaseCommand::Snake { text } => to_snake_case(text.as_ref()),
        };

        Ok(Some(Output::JsonValue(serde_json::json!(result))))
    }
}

// lowercase
fn to_lowercase(text: &str) -> String {
    text.to_lowercase()
}

// UPPERCASE
fn to_uppercase(text: &str) -> String {
    text.to_uppercase()
}

// camelCase
fn to_camel_case(text: &str) -> String {
    let words = split_words(text);
    if words.is_empty() {
        return String::new();
    }

    let mut result = words[0].to_lowercase();
    for word in &words[1..] {
        if !word.is_empty() {
            result.push_str(&capitalize_first(word));
        }
    }
    result
}

// Title Case
fn to_title_case(text: &str) -> String {
    split_words(text)
        .iter()
        .map(|word| capitalize_first(word))
        .collect::<Vec<_>>()
        .join(" ")
}

// CONSTANT_CASE
fn to_constant_case(text: &str) -> String {
    split_words(text)
        .iter()
        .map(|word| word.to_uppercase())
        .collect::<Vec<_>>()
        .join("_")
}

// header-case
fn to_header_case(text: &str) -> String {
    split_words(text)
        .iter()
        .map(|word| capitalize_first(word))
        .collect::<Vec<_>>()
        .join("-")
}

// Sentence case
fn to_sentence_case(text: &str) -> String {
    let words = split_words(text);
    if words.is_empty() {
        return String::new();
    }

    let mut result = capitalize_first(&words[0]);
    for word in &words[1..] {
        if !word.is_empty() {
            result.push(' ');
            result.push_str(&word.to_lowercase());
        }
    }
    result
}

// snake_case
fn to_snake_case(text: &str) -> String {
    split_words(text)
        .iter()
        .map(|word| word.to_lowercase())
        .collect::<Vec<_>>()
        .join("_")
}

// Splits a string into a sequence of words based on the whitespace, hyphens,
// underscore and casing boundaries.
fn split_words(text: &str) -> Vec<String> {
    let mut chars = text.chars().peekable();

    let mut words = Vec::new();
    let mut current_word = String::new();

    while let Some(ch) = chars.next() {
        // Split on explicit separators (space, underscore, hyphen)
        if ch.is_whitespace() || ch == '_' || ch == '-' || ch == '.' {
            if !current_word.is_empty() {
                words.push(current_word.clone());
                current_word.clear();
            }
        // Split on camelCase boundaries (uppercase followed by lowercase)
        // Example: "XMLParser" -> ["XML", "Parser"]
        } else if ch.is_uppercase() && !current_word.is_empty() {
            // Check if this uppercase letter starts a new word
            // (uppercase followed by lowercase indicates word boundary)
            if chars.peek().map_or(false, |&next| next.is_lowercase()) {
                words.push(current_word.clone());
                current_word.clear();
            }
            current_word.push(ch);
        } else {
            current_word.push(ch);
        }
    }

    // Add the final word if it exists
    if !current_word.is_empty() {
        words.push(current_word);
    }

    words
}

pub fn capitalize_first(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lowercase() {
        assert_eq!(to_lowercase("Hello World"), "hello world");
        assert_eq!(to_lowercase("HELLO WORLD"), "hello world");
        assert_eq!(to_lowercase("hELLo WoRLd"), "hello world");
    }

    #[test]
    fn test_uppercase() {
        assert_eq!(to_uppercase("Hello World"), "HELLO WORLD");
        assert_eq!(to_uppercase("hello world"), "HELLO WORLD");
        assert_eq!(to_uppercase("hELLo WoRLd"), "HELLO WORLD");
    }

    #[test]
    fn test_camel_case() {
        assert_eq!(to_camel_case("hello world"), "helloWorld");
        assert_eq!(to_camel_case("Hello World"), "helloWorld");
        assert_eq!(to_camel_case("HELLO_WORLD"), "helloWorld");
        assert_eq!(to_camel_case("hello-world"), "helloWorld");
        assert_eq!(to_camel_case("HelloWorld"), "helloWorld");
        assert_eq!(to_camel_case("single"), "single");
        assert_eq!(to_camel_case(""), "");
    }

    #[test]
    fn test_title_case() {
        assert_eq!(to_title_case("hello world"), "Hello World");
        assert_eq!(to_title_case("HELLO WORLD"), "Hello World");
        assert_eq!(to_title_case("hello_world"), "Hello World");
        assert_eq!(to_title_case("hello-world"), "Hello World");
        assert_eq!(to_title_case("helloWorld"), "Hello World");
    }

    #[test]
    fn test_constant_case() {
        assert_eq!(to_constant_case("hello world"), "HELLO_WORLD");
        assert_eq!(to_constant_case("Hello World"), "HELLO_WORLD");
        assert_eq!(to_constant_case("helloWorld"), "HELLO_WORLD");
        assert_eq!(to_constant_case("hello-world"), "HELLO_WORLD");
        assert_eq!(to_constant_case("HELLO_WORLD"), "HELLO_WORLD");
    }

    #[test]
    fn test_header_case() {
        assert_eq!(to_header_case("hello world"), "Hello-World");
        assert_eq!(to_header_case("Hello World"), "Hello-World");
        assert_eq!(to_header_case("helloWorld"), "Hello-World");
        assert_eq!(to_header_case("hello_world"), "Hello-World");
        assert_eq!(to_header_case("HELLO_WORLD"), "Hello-World");
    }

    #[test]
    fn test_sentence_case() {
        assert_eq!(to_sentence_case("hello world"), "Hello world");
        assert_eq!(to_sentence_case("HELLO WORLD"), "Hello world");
        assert_eq!(to_sentence_case("helloWorld"), "Hello world");
        assert_eq!(to_sentence_case("hello_world"), "Hello world");
        assert_eq!(to_sentence_case("hello-world"), "Hello world");
    }

    #[test]
    fn test_snake_case() {
        assert_eq!(to_snake_case("hello world"), "hello_world");
        assert_eq!(to_snake_case("Hello World"), "hello_world");
        assert_eq!(to_snake_case("helloWorld"), "hello_world");
        assert_eq!(to_snake_case("HELLO_WORLD"), "hello_world");
        assert_eq!(to_snake_case("hello-world"), "hello_world");
    }

    #[test]
    fn test_split_words() {
        assert_eq!(split_words("hello world"), vec!["hello", "world"]);
        assert_eq!(split_words("helloWorld"), vec!["hello", "World"]);
        assert_eq!(split_words("hello_world"), vec!["hello", "world"]);
        assert_eq!(split_words("hello-world"), vec!["hello", "world"]);
        assert_eq!(split_words("HTTPSConnection"), vec!["HTTPS", "Connection"]);
        assert_eq!(split_words("XMLParser"), vec!["XML", "Parser"]);
        assert_eq!(split_words("single"), vec!["single"]);
        assert_eq!(split_words(""), Vec::<String>::new());
    }

    #[test]
    fn test_capitalize_first() {
        assert_eq!(capitalize_first("hello"), "Hello");
        assert_eq!(capitalize_first("HELLO"), "Hello");
        assert_eq!(capitalize_first("h"), "H");
        assert_eq!(capitalize_first(""), "");
    }
}

use crate::args::StringInput;
use crate::tool::{Output, Tool};
use clap::{Command, CommandFactory, Parser, Subcommand};
use md5::Md5;
use sha1::Sha1;
use sha2::{Digest, Sha224, Sha256, Sha384, Sha512};

#[derive(Parser, Debug)]
#[command(name = "hash")]
#[command(about = "Generate hash digests using various algorithms")]
pub struct HashTool {
    #[command(subcommand)]
    command: HashCommand,
}

#[derive(Subcommand, Debug)]
enum HashCommand {
    /// Generate MD5 hash
    Md5 {
        /// Input to hash (use "-" for stdin)
        input: StringInput,
    },
    /// Generate SHA-1 hash
    Sha1 {
        /// Input to hash (use "-" for stdin)
        input: StringInput,
    },
    /// Generate SHA-224 hash
    Sha224 {
        /// Input to hash (use "-" for stdin)
        input: StringInput,
    },
    /// Generate SHA-256 hash
    Sha256 {
        /// Input to hash (use "-" for stdin)
        input: StringInput,
    },
    /// Generate SHA-384 hash
    Sha384 {
        /// Input to hash (use "-" for stdin)
        input: StringInput,
    },
    /// Generate SHA-512 hash
    Sha512 {
        /// Input to hash (use "-" for stdin)
        input: StringInput,
    },
}

impl Tool for HashTool {
    fn cli() -> Command {
        HashTool::command()
    }

    fn execute(&self) -> anyhow::Result<Option<Output>> {
        let hash = match &self.command {
            HashCommand::Md5 { input } => {
                let mut hasher = Md5::new();
                hasher.update(input.as_ref().as_bytes());
                format!("{:x}", hasher.finalize())
            }
            HashCommand::Sha1 { input } => {
                let mut hasher = Sha1::new();
                hasher.update(input.as_ref().as_bytes());
                format!("{:x}", hasher.finalize())
            }
            HashCommand::Sha224 { input } => {
                let mut hasher = Sha224::new();
                hasher.update(input.as_ref().as_bytes());
                format!("{:x}", hasher.finalize())
            }
            HashCommand::Sha256 { input } => {
                let mut hasher = Sha256::new();
                hasher.update(input.as_ref().as_bytes());
                format!("{:x}", hasher.finalize())
            }
            HashCommand::Sha384 { input } => {
                let mut hasher = Sha384::new();
                hasher.update(input.as_ref().as_bytes());
                format!("{:x}", hasher.finalize())
            }
            HashCommand::Sha512 { input } => {
                let mut hasher = Sha512::new();
                hasher.update(input.as_ref().as_bytes());
                format!("{:x}", hasher.finalize())
            }
        };

        Ok(Some(Output::JsonValue(serde_json::json!(hash))))
    }
}

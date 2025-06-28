use clap::{CommandFactory, FromArgMatches};

use crate::tool::Tool;
use anyhow::{Context, anyhow};

mod tool;
pub mod tools;

// This way of building main is not ideal.
macro_rules! toolbox {
    ($cmd:ident, $(($tool:path, $name:literal, $($alias:literal),*)),+) => {
        {
            // Register the tools.
            $(
                $cmd = $cmd.subcommand(
                    <$tool>::command()
                    .name($name)
                    $(.alias($alias))*
                );
            )*

            // Parse args.
            let matches = $cmd.get_matches();
            let (subcommand_name, subcommand_matches) = matches
                .subcommand()
                .context("Could not determine subcommand")?;

            // Run the specific tool.
            match subcommand_name {
                $(
                    $name => {
                        let output = <$tool>::from_arg_matches(subcommand_matches)
                            .context("Could not initialize the tool")?
                            .execute()?;

                        println!("{:?}", output);
                        Ok(())
                    }
                )*
                _ => {
                    Err(anyhow!("Unknown subcommand"))
                }
            }
        }
    };
}

fn main() -> anyhow::Result<()> {
    let mut cli = clap::builder::Command::new("ut").about("a utility toolkit");
    toolbox!(
        cli,
        (tools::crypto::hash::Hash, "hash",),
        (
            tools::crypto::token::TokenGenerator,
            "token",
            "secret",
            "password"
        ),
        (tools::crypto::ulid::ULIDGenerator, "ulid",),
        (tools::crypto::uuid1::UUID1Generator, "uuid-v1", "uuid1"),
        (tools::crypto::uuid3::UUID3Generator, "uuid-v3", "uuid3"),
        (
            tools::crypto::uuid4::UUID4Generator,
            "uuid-v4",
            "uuid",
            "uuid4"
        ),
        (tools::crypto::uuid5::UUID5Generator, "uuid-v5", "uuid5")
    )
}

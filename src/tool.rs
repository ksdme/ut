// Represents a tool under ut.
pub trait Tool {
    // The contribution of this tool to the ut CLI. The clap::Command
    // returned here will be set up as a subcommand on the ut binary.
    fn cli() -> clap::Command;

    // Run the tool. All the context that the tool requires should be
    // using the cli above.
    fn execute(&self) -> anyhow::Result<Option<Output>>;
}

#[derive(Debug)]
pub enum Output {
    Bytes(Vec<u8>),
    JsonValue(serde_json::Value),
}

/// Represents a tool on ut.
pub trait Tool {
    fn execute(&self) -> anyhow::Result<Option<Output>>;
}

#[derive(Debug)]
pub enum Output {
    JsonValue(serde_json::Value),
}

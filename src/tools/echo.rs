use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct EchoParams {
    pub message: String,
}

pub fn format_message(message: &str) -> String {
    format!("Echo: {message}")
}

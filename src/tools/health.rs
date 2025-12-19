use schemars::JsonSchema;
use serde::Deserialize;

use crate::server::ServerState;

#[derive(Debug, Deserialize, JsonSchema, Default)]
pub struct HealthParams {
    #[serde(default)]
    pub verbose: bool,
}

pub fn health_summary(state: &ServerState, verbose: bool) -> String {
    let uptime = state.uptime();
    let active = state.transports.active_endpoints();
    let transports = if active.is_empty() {
        "none".to_string()
    } else {
        active.join(", ")
    };
    if verbose {
        format!(
            "status: ok\nversion: {}\nuptime_seconds: {}\ntransports: {}",
            state.version,
            uptime.as_secs(),
            transports
        )
    } else {
        format!(
            "ok (v{}, uptime {}s, transports: {})",
            state.version,
            uptime.as_secs(),
            transports
        )
    }
}

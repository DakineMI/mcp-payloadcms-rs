use clap::Parser;

use mcp_payloadcms_rs::{cli, error::ServiceResult, server};

#[tokio::main]
async fn main() -> ServiceResult<()> {
    server::start_server(cli::CommandArguments::parse()).await
}

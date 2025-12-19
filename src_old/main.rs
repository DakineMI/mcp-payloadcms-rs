use clap::Parser;

use mcp_server_template_rs::{cli, error::ServiceResult, server};

#[tokio::main]
async fn main() -> ServiceResult<()> {
    server::start_server(cli::CommandArguments::parse()).await
}

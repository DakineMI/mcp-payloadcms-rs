use clap::Parser;

#[derive(Parser, Debug)]
#[command(name =  env!("CARGO_PKG_NAME"))]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "MCP server for Payload CMS",
long_about = None)]
pub struct CommandArguments {}

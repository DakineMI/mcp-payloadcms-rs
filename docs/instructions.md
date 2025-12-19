# MCP Server (library)

This crate exposes a pluggable, multi-transport MCP server. Defaults include:

Tools:
- `echo`: Echo a message back to the caller (`{ "message": "hi" }`).
- `health`: Report version, uptime, and active transports. Optional `verbose` flag.

Transports: stdio, TCP (`MCP_TCP_ADDR`), Unix socket (`MCP_UNIX_PATH`, unix only), streamable HTTP+SSE (`MCP_HTTP_ADDR`), dedicated SSE (`MCP_SSE_ADDR`), and websockets (`MCP_WS_ADDR`). Toggle via `MCP_ENABLE_*` env vars.

Add your own tools by extending `tools.rs` (define schemas, add to `tool_definitions()`, and dispatch in `run_tool`). Instructions are served via the `file://instructions` resource and returned from initialize.

Notes:
- At least one transport must be enabled; otherwise the server exits early with an error.
- Logging is enabled via `tracing_subscriber` (defaults to `info`). Override with `RUST_LOG` as needed. Startup logs include the active transports and bind targets.

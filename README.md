# MCP PayloadCMS (Rust) — Local-only

This is a local-only Rust implementation of the payload MCP tooling and server.

Build
  cd mcp-payloadcms-rs
  cargo build --release

Run (dev)
  cargo run

Configuration (optional)
  `SOFTWARE_PLANNER_DATA_PATH`  - Full path to the JSON data file (e.g. `/tmp/mcp-data.json`)
  `SOFTWARE_PLANNER_DATA_DIR`   - Directory path where `data.json` will be stored (e.g. `/tmp/mcp-data`)

Default storage location (if no env var set):
  `~/.software-planning-tool/data.json`

Endpoints / Tools
- The MCP server exposes tools via RMCP (tools include `echo`, `validate`, `query`, `mcp_query`, `generate_template`, `generate_collection`, `generate_field`, `scaffold_project`).
- Resources: `payload://templates`, `payload://scaffold`.

Notes
- All operations are local-only and do not contact Redis or external services.
- `scaffold_project` writes files to disk; provide an `outputDir` option to choose location or it will create a temporary directory.

## Payload CMS MCP Server Tools

- `validate`: Validate Payload CMS code for collections, fields, globals, or config. Provide `code` and `file_type` (`collection`, `field`, `global`, `config`).
- `query`: Search validation rules and best practices. Provide `query` text and optional `file_type`.
- `mcp_query`: Run SQL-like queries over validation rules. Provide `sql`.
- `generate_template`: Generate code templates (`collection`, `field`, `global`, `config`, `access-control`, `hook`, `endpoint`, `plugin`, `block`, `migration`) with an `options` object.
- `generate_collection`: Convenience to generate a collection template; supply `slug` and optional `fields`, `auth`, `timestamps`, `admin`, `hooks`, `access`, `versions`.
- `generate_field`: Convenience to generate a field template; supply `name`, `type`, and optional flags.
- `scaffold_project`: Scaffold a full Payload CMS project; supply `project_name` and optional config matching Payload’s collections/globals/blocks/plugins.

All results are returned as JSON. Use `mcp_query` for ad-hoc inspection of the validation rule catalog. Use `scaffold_project` to get a file structure you can write to disk.

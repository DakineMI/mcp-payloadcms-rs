# MCP PayloadCMS Development Server

A comprehensive Model Context Protocol (MCP) server that accelerates PayloadCMS development through intelligent code generation, validation, and project scaffolding.

## 🚀 Features

### Code Generation
- **Collection Generation**: Complete Payload collections with fields, hooks, and access control
- **Field Templates**: All Payload field types with proper TypeScript typing
- **Global Configuration**: Global settings and configurations
- **Hook Scaffolding**: Ready-to-use hook templates for common patterns
- **Access Control**: Pre-built access control functions and patterns

### Validation & Quality Assurance
- **Code Validation**: Syntax and structure validation for Payload code
- **Type Checking**: TypeScript compatibility verification
- **Best Practices**: Payload CMS best practices enforcement
- **Security Checks**: Common security vulnerabilities detection

### Intelligent Querying
- **Rule Database**: 200+ validation rules organized by category
- **SQL-like Queries**: Advanced filtering and searching of validation rules
- **Context-Aware Suggestions**: Smart recommendations based on code patterns
- **Documentation Integration**: Automatic linking to Payload documentation

### Project Scaffolding
- **Quick Start**: Generate complete Payload projects with best practices
- **Modular Architecture**: Well-structured project layouts
- **Configuration Templates**: Optimized Payload configurations
- **Development Tools**: Integrated testing and development utilities

## 🛠️ MCP Tools

### Code Generation Tools
- `generate_collection` - Generate complete Payload collections
- `generate_field` - Generate individual field configurations
- `generate_template` - Generate components from templates
- `generate_global` - Generate global configurations

### Validation Tools
- `validate_payload_code` - Validate Payload code snippets
- `validate_project` - Validate entire project configurations
- `check_best_practices` - Check adherence to Payload best practices

### Query & Analysis Tools
- `query_validation_rules` - Query validation rules with filters
- `get_validation_rules_with_examples` - Get rules with code examples
- `get_categories` - List all validation categories
- `execute_sql_query` - SQL queries against rule database

### Project Management Tools
- `scaffold_project` - Create new Payload projects
- `analyze_project` - Analyze existing project structure
- `migrate_project` - Migration helpers for version upgrades

## 📋 Installation

### Prerequisites
- Rust 1.70+
- Payload CMS knowledge (recommended)

### Build from Source
```bash
git clone <repository>
cd mcp-payloadcms-rs
cargo build --release
```

### MCP Client Integration
Add to your MCP client configuration:

```json
{
  "mcpServers": {
    "payloadcms": {
      "command": "cargo",
      "args": ["run", "--bin", "mcp-payloadcms-rs"],
      "env": {
        "RUST_LOG": "info"
      }
    }
  }
}
```

## 🎯 Usage Examples

### Generate a Complete Collection
```json
{
  "method": "tools/call",
  "params": {
    "name": "generate_collection",
    "arguments": {
      "slug": "blog-posts",
      "auth": true,
      "timestamps": true,
      "fields": [
        {
          "name": "title",
          "type": "text",
          "required": true,
          "localized": true
        },
        {
          "name": "content",
          "type": "richText",
          "required": true
        },
        {
          "name": "tags",
          "type": "relationship",
          "relationTo": "tags",
          "hasMany": true
        }
      ],
      "admin": {
        "useAsTitle": "title",
        "defaultColumns": ["title", "status", "updatedAt"]
      },
      "hooks": true,
      "access": true,
      "versions": true
    }
  }
}
```

### Validate Generated Code
```json
{
  "method": "tools/call",
  "params": {
    "name": "validate_payload_code",
    "arguments": {
      "code": "export const Users = {\n  slug: 'users',\n  fields: [\n    { name: 'email', type: 'email', required: true }\n  ]\n}",
      "file_type": "collection"
    }
  }
}
```

### Query Validation Rules
```json
{
  "method": "tools/call",
  "params": {
    "name": "query_validation_rules",
    "arguments": {
      "query": "security",
      "category": "access",
      "severity": "high"
    }
  }
}
```

## 🔍 Supported Payload Features

### Field Types
- `text`, `textarea`, `richText`
- `number`, `email`, `date`, `checkbox`
- `select`, `radio`, `relationship`
- `upload`, `array`, `blocks`, `tabs`
- `group`, `row`, `collapsible`

### Advanced Features
- **Authentication**: User registration and login
- **Access Control**: Field and document-level permissions
- **Hooks**: Before/after operation hooks
- **Versions**: Document versioning and drafts
- **Localization**: Multi-language content support
- **SEO**: Meta fields and search optimization

### Integrations
- **Database**: MongoDB, PostgreSQL support
- **File Storage**: Local and cloud storage (AWS S3, etc.)
- **Email**: SMTP and service integrations
- **Search**: Full-text search capabilities

## 🏗️ Architecture

### Core Components
- **Generator Engine**: Template-based code generation
- **Validator System**: Rule-based validation pipeline
- **Query Processor**: SQL-like rule database queries
- **Scaffolder**: Project initialization and structure
- **Type System**: Complete TypeScript integration

### MCP Integration
- **Tool Registration**: Dynamic tool discovery and registration
- **Parameter Validation**: JSON Schema validation for all inputs
- **Error Handling**: Comprehensive error reporting and recovery
- **Async Processing**: Tokio-based concurrent operations
- **Resource Management**: Efficient memory and CPU usage

## 🧪 Testing

Run the test suite:
```bash
cargo test
```

Run doctests:
```bash
cargo test --doc
```

## 📚 Documentation

### MCP Tool Reference
See `src/docs/instructions.md` for complete MCP tool documentation with examples.

### Validation Rules
The server includes 200+ validation rules covering:
- **Syntax**: Code structure and formatting
- **Types**: TypeScript type safety
- **Security**: Common vulnerabilities and best practices
- **Performance**: Optimization recommendations
- **Maintainability**: Code quality and structure

### Best Practices
- Always validate generated code before production use
- Use the scaffolding tools for consistent project structure
- Leverage the query tools to understand validation requirements
- Review access control configurations carefully

## 🤝 Contributing

### Development Setup
```bash
git clone <repository>
cd mcp-payloadcms-rs
cargo build
cargo test
```

### Adding New Tools
1. Define the tool parameters in `src/payload_tools/mcp.rs`
2. Implement the logic in the appropriate module
3. Add comprehensive tests
4. Update documentation in `src/docs/instructions.md`

### Code Standards
- Comprehensive error handling
- Full test coverage
- Clear documentation
- TypeScript-first approach
- Payload best practices compliance

## 📄 License

MIT License - see LICENSE file for details.

## 🙏 Acknowledgments

- [Payload CMS](https://payloadcms.com/) for the excellent headless CMS
- [Model Context Protocol](https://modelcontextprotocol.io/) for the AI integration standard
- The Rust community for outstanding tooling and libraries

---

**Transform your PayloadCMS development workflow with AI-powered assistance.**
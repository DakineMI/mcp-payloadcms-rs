# MCP PayloadCMS Server Instructions

## Overview

The MCP PayloadCMS server provides comprehensive tooling for PayloadCMS development through the Model Context Protocol. It enables AI assistants to generate Payload components, validate configurations, query validation rules, and scaffold new projects.

## Available MCP Tools

### 1. `echo` - Test Tool
**Purpose:** Simple echo functionality for testing MCP connectivity

**Parameters:**
```json
{
  "message": "string" // Message to echo back
}
```

**Example:**
```json
{
  "name": "echo",
  "arguments": {
    "message": "Hello PayloadCMS!"
  }
}
```

---

### 2. `validate_payload_code` - Code Validation
**Purpose:** Validate PayloadCMS code snippets for syntax and structure

**Parameters:**
```json
{
  "code": "string",        // Payload code to validate
  "file_type": "FileType"  // Type of Payload file (collection, global, etc.)
}
```

**Supported File Types:**
- `collection` - Collection configuration
- `global` - Global configuration
- `field` - Field definition
- `hook` - Hook function
- `access` - Access control function

**Example:**
```json
{
  "name": "validate_payload_code",
  "arguments": {
    "code": "export const Users = {\n  slug: 'users',\n  fields: [\n    { name: 'email', type: 'email', required: true }\n  ]\n}",
    "file_type": "collection"
  }
}
```

---

### 3. `query_validation_rules` - Rule Querying
**Purpose:** Query and filter validation rules with advanced search capabilities

**Parameters:**
```json
{
  "query": "string",           // Search query (optional)
  "file_type": "FileType",      // Filter by file type (optional)
  "category": "string",         // Filter by category (optional)
  "severity": "string"          // Filter by severity level (optional)
}
```

**Query Examples:**
```json
// Get all rules
{
  "name": "query_validation_rules",
  "arguments": {}
}

// Search for specific rules
{
  "name": "query_validation_rules",
  "arguments": {
    "query": "field validation"
  }
}

// Get rules for collections only
{
  "name": "query_validation_rules",
  "arguments": {
    "file_type": "collection",
    "category": "structure"
  }
}
```

---

### 4. `get_validation_rules_with_examples` - Rules with Examples
**Purpose:** Retrieve validation rules with practical code examples

**Parameters:** None required

**Example:**
```json
{
  "name": "get_validation_rules_with_examples",
  "arguments": {}
}
```

---

### 5. `get_categories` - Validation Categories
**Purpose:** Get list of all available validation rule categories

**Parameters:** None required

**Example:**
```json
{
  "name": "get_categories",
  "arguments": {}
}
```

---

### 6. `generate_template` - Component Generation
**Purpose:** Generate PayloadCMS components from templates

**Parameters:**
```json
{
  "template_type": "TemplateType",  // Type of template to generate
  "options": "object"               // Template-specific options
}
```

**Supported Template Types:**
- `collection` - Collection configuration
- `global` - Global configuration
- `field` - Field definition
- `hook` - Hook function
- `access` - Access control function

**Example - Generate Collection:**
```json
{
  "name": "generate_template",
  "arguments": {
    "template_type": "collection",
    "options": {
      "slug": "posts",
      "fields": [
        { "name": "title", "type": "text", "required": true },
        { "name": "content", "type": "richText" },
        { "name": "author", "type": "relationship", "relationTo": "users" }
      ],
      "admin": {
        "useAsTitle": "title"
      }
    }
  }
}
```

---

### 7. `execute_sql_query` - SQL Query Execution
**Purpose:** Execute SQL queries against Payload's database validation rules

**Parameters:**
```json
{
  "sql": "string"  // SQL query to execute
}
```

**Supported Queries:**
```sql
-- Get all validation rules
SELECT * FROM validation_rules;

-- Search by category
SELECT * FROM validation_rules WHERE category = 'security';

-- Count rules by severity
SELECT severity, COUNT(*) as count
FROM validation_rules
GROUP BY severity;
```

**Example:**
```json
{
  "name": "execute_sql_query",
  "arguments": {
    "sql": "SELECT category, COUNT(*) as count FROM validation_rules GROUP BY category"
  }
}
```

---

### 8. `generate_collection` - Collection Generation
**Purpose:** Generate complete Payload collections with fields and configuration

**Parameters:**
```json
{
  "slug": "string",           // Collection slug
  "fields": "array",          // Array of field definitions (optional)
  "auth": "boolean",          // Enable authentication (optional)
  "timestamps": "boolean",    // Add timestamps (optional)
  "admin": "object",          // Admin configuration (optional)
  "hooks": "boolean",         // Add hook placeholders (optional)
  "access": "boolean",        // Add access control (optional)
  "versions": "boolean"       // Enable versioning (optional)
}
```

**Example:**
```json
{
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
```

---

### 9. `generate_field` - Field Generation
**Purpose:** Generate individual Payload field configurations

**Parameters:**
```json
{
  "name": "string",           // Field name
  "type": "string",           // Field type (text, number, email, etc.)
  "required": "boolean",      // Field requirement (optional)
  "unique": "boolean",        // Uniqueness constraint (optional)
  "localized": "boolean",     // Localization support (optional)
  "access": "boolean",        // Access control (optional)
  "admin": "object",          // Admin configuration (optional)
  "validation": "boolean",    // Validation rules (optional)
  "default_value": "any"      // Default value (optional)
}
```

**Common Field Types:**
- `text` - Text input
- `textarea` - Multi-line text
- `richText` - Rich text editor
- `number` - Numeric input
- `email` - Email validation
- `date` - Date picker
- `upload` - File upload
- `relationship` - Relation to other collections

**Example:**
```json
{
  "name": "generate_field",
  "arguments": {
    "name": "email",
    "type": "email",
    "required": true,
    "unique": true,
    "admin": {
      "description": "User's email address"
    }
  }
}
```

---

## Usage Patterns

### 1. Project Initialization
```json
// Start with scaffolding
{
  "name": "scaffolding_scaffold_project",
  "arguments": {
    "name": "my-payload-project",
    "template": "basic"
  }
}

// Generate core collections
{
  "name": "generate_collection",
  "arguments": {
    "slug": "users",
    "auth": true,
    "fields": [
      {"name": "email", "type": "email", "required": true, "unique": true},
      {"name": "name", "type": "text", "required": true}
    ]
  }
}
```

### 2. Development Workflow
```json
// Generate a new field
{
  "name": "generate_field",
  "arguments": {
    "name": "description",
    "type": "textarea",
    "required": false,
    "admin": {"description": "Optional description"}
  }
}

// Validate the generated code
{
  "name": "validate_payload_code",
  "arguments": {
    "code": "export const description = {\n  name: 'description',\n  type: 'textarea',\n  required: false,\n  admin: { description: 'Optional description' }\n};",
    "file_type": "field"
  }
}
```

### 3. Code Review and Validation
```json
// Check all validation rules
{
  "name": "get_validation_rules_with_examples",
  "arguments": {}
}

// Query specific rules
{
  "name": "query_validation_rules",
  "arguments": {
    "query": "security",
    "category": "access"
  }
}
```

### 4. Advanced Querying
```json
// SQL-based rule analysis
{
  "name": "execute_sql_query",
  "arguments": {
    "sql": "SELECT category, severity, COUNT(*) as count FROM validation_rules GROUP BY category, severity ORDER BY count DESC"
  }
}
```

## Error Handling

The server provides detailed error messages for common issues:

- **Validation Errors:** Invalid Payload code structure
- **Type Errors:** Incorrect field types or configurations
- **Query Errors:** Invalid SQL syntax or rule queries
- **Generation Errors:** Template rendering failures

## Best Practices

1. **Start with Validation:** Always validate generated code before using in production
2. **Use Templates:** Leverage built-in templates for consistent code generation
3. **Query Rules:** Use validation rule queries to understand requirements
4. **Test Incrementally:** Generate and validate components step-by-step
5. **Review Access Control:** Pay special attention to access control configurations

## Integration with Payload

This MCP server is designed to work alongside Payload's development workflow:

1. **Code Generation:** Generate components faster than manual coding
2. **Validation:** Ensure generated code meets Payload standards
3. **Querying:** Understand validation rules and best practices
4. **Scaffolding:** Quickly set up new Payload projects

## Contributing

When adding new tools or modifying existing ones:

1. Update this instructions file with new tool documentation
2. Add comprehensive examples for all parameters
3. Include error cases and edge conditions
4. Update the usage patterns section if needed

## Support

For issues or questions about the MCP PayloadCMS server:

- Check the validation rules documentation
- Review the generated code examples
- Test with simple cases first
- Use the echo tool for connectivity testing
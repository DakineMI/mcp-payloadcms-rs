use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum TemplateType {
    Collection,
    Field,
    Global,
    Config,
    AccessControl,
    Hook,
    Endpoint,
    Plugin,
    Block,
    Migration,
}

pub fn generate_template(template_type: TemplateType, options: &Value) -> Result<String, String> {
    let map = options
        .as_object()
        .ok_or_else(|| "Template options must be an object".to_string())?;

    match template_type {
        TemplateType::Collection => generate_collection_template(map),
        TemplateType::Field => generate_field_template(map),
        TemplateType::Global => generate_global_template(map),
        TemplateType::Config => generate_config_template(map),
        TemplateType::AccessControl => generate_access_control_template(map),
        TemplateType::Hook => generate_hook_template(map),
        TemplateType::Endpoint => generate_endpoint_template(map),
        TemplateType::Plugin => generate_plugin_template(map),
        TemplateType::Block => generate_block_template(map),
        TemplateType::Migration => generate_migration_template(map),
    }
}

fn get_string(map: &Map<String, Value>, key: &str) -> Option<String> {
    map.get(key).and_then(|v| v.as_str()).map(|s| s.to_string())
}

fn get_bool(map: &Map<String, Value>, key: &str, default: bool) -> bool {
    map.get(key).and_then(|v| v.as_bool()).unwrap_or(default)
}

fn get_array<'a>(map: &'a Map<String, Value>, key: &str) -> Option<&'a Vec<Value>> {
    map.get(key).and_then(|v| v.as_array())
}

fn value_to_literal(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => format!("'{}'", s.replace('\'', "\\'")),
        Value::Array(arr) => {
            let parts: Vec<String> = arr.iter().map(value_to_literal).collect();
            format!("[{}]", parts.join(", "))
        }
        Value::Object(obj) => {
            let parts: Vec<String> = obj
                .iter()
                .map(|(k, v)| format!("{}: {}", k, value_to_literal(v)))
                .collect();
            format!("{{{}}}", parts.join(", "))
        }
    }
}

fn generate_collection_template(options: &Map<String, Value>) -> Result<String, String> {
    let slug = get_string(options, "slug").ok_or("Collection slug is required")?;
    let fields = get_array(options, "fields").cloned().unwrap_or_default();
    let auth = get_bool(options, "auth", false);
    let timestamps = get_bool(options, "timestamps", true);
    let hooks = get_bool(options, "hooks", false);
    let access = get_bool(options, "access", false);
    let versions = get_bool(options, "versions", false);

    let admin = options
        .get("admin")
        .and_then(|v| v.as_object())
        .cloned()
        .unwrap_or_default();

    let fields_code = if fields.is_empty() {
        String::new()
    } else {
        let mut lines = Vec::new();
        for field in fields {
            lines.push(generate_field_template_from_value(&field)?);
        }
        lines.join(",\n    ")
    };

    let admin_code = if admin.is_empty() {
        String::new()
    } else {
        let mut admin_parts = String::new();
        if let Some(title) = admin.get("useAsTitle").and_then(|v| v.as_str()) {
            admin_parts.push_str(&format!("\n    useAsTitle: '{title}',"));
        }
        if let Some(cols) = admin.get("defaultColumns").and_then(|v| v.as_array()) {
            let cols = cols
                .iter()
                .filter_map(|v| v.as_str())
                .map(|c| format!("'{c}'"))
                .collect::<Vec<_>>()
                .join(", ");
            admin_parts.push_str(&format!("\n    defaultColumns: [{cols}],"));
        }
        if let Some(group) = admin.get("group").and_then(|v| v.as_str()) {
            admin_parts.push_str(&format!("\n    group: '{group}',"));
        }

        format!("\n  admin: {{{}\n  }},", admin_parts)
    };

    let hooks_code = if hooks {
        "\n  hooks: {\n    beforeOperation: [\n      // Add your hooks here\n    ],\n    afterOperation: [\n      // Add your hooks here\n    ],\n  },"
            .to_string()
    } else {
        String::new()
    };

    let access_code = if access {
        "\n  access: {\n    read: () => true,\n    update: () => true,\n    create: () => true,\n    delete: () => true,\n  },"
            .to_string()
    } else {
        String::new()
    };

    let auth_code = if auth {
        "\n  auth: {\n    useAPIKey: true,\n    tokenExpiration: 7200,\n  },"
            .to_string()
    } else {
        String::new()
    };

    let versions_code = if versions {
        "\n  versions: {\n    drafts: true,\n  },".to_string()
    } else {
        String::new()
    };

    Ok(format!(
        "import {{ CollectionConfig }} from 'payload/types';\n\nconst {}: CollectionConfig = {{\n  slug: '{}',{}{}{}{}{}\n  {}fields: [\n    {}\n  ],\n}};\n\nexport default {};",
        capitalize(&slug),
        slug,
        admin_code,
        auth_code,
        access_code,
        hooks_code,
        versions_code,
        if timestamps { "timestamps: true,\n  " } else { "" },
        fields_code,
        capitalize(&slug)
    ))
}

fn generate_field_template(options: &Map<String, Value>) -> Result<String, String> {
    generate_field_template_from_value(&Value::Object(options.clone()))
}

fn generate_field_template_from_value(value: &Value) -> Result<String, String> {
    let map = value
        .as_object()
        .ok_or_else(|| "Field options must be an object".to_string())?;
    let name = get_string(map, "name").ok_or("Field name is required")?;
    let field_type = get_string(map, "type").ok_or("Field type is required")?;

    let required = get_bool(map, "required", false);
    let unique = get_bool(map, "unique", false);
    let localized = get_bool(map, "localized", false);
    let access = get_bool(map, "access", false);
    let validation = get_bool(map, "validation", false);
    let default_value = map.get("defaultValue");
    let admin = map
        .get("admin")
        .and_then(|v| v.as_object())
        .cloned()
        .unwrap_or_default();

    let admin_code = if admin.is_empty() {
        String::new()
    } else {
        let mut admin_parts = String::new();
        if let Some(description) = admin.get("description").and_then(|v| v.as_str()) {
            admin_parts.push_str(&format!("\n      description: '{description}',"));
        }
        if admin
            .get("readOnly")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            admin_parts.push_str("\n      readOnly: true,");
        }

        format!("\n    admin: {{{}\n    }},", admin_parts)
    };

    let access_code = if access {
        "\n    access: {\n      read: () => true,\n      update: () => true,\n    },"
            .to_string()
    } else {
        String::new()
    };

    let validation_code = if validation {
        "\n    validate: (value) => {\n      if (value === undefined || value === null) {\n        return 'Value is required';\n      }\n      return true;\n    },"
            .to_string()
    } else {
        String::new()
    };

    let default_value_code = default_value.map(|v| {
        format!(
            "\n    defaultValue: {},",
            if v.is_string() {
                value_to_literal(v)
            } else {
                value_to_literal(v)
            }
        )
    });

    let field_specific = match field_type.as_str() {
        "text" | "textarea" | "email" | "code" => "\n    minLength: 1,\n    maxLength: 255,".to_string(),
        "number" => "\n    min: 0,\n    max: 1000,".to_string(),
        "select" => "\n    options: [\n      { label: 'Option 1', value: 'option1' },\n      { label: 'Option 2', value: 'option2' },\n    ],\n    hasMany: false,".to_string(),
        "relationship" => "\n    relationTo: 'collection-name',\n    hasMany: false,".to_string(),
        "array" => "\n    minRows: 0,\n    maxRows: 10,\n    fields: [\n      {\n        name: 'subField',\n        type: 'text',\n        required: true,\n      },\n    ],".to_string(),
        "blocks" => "\n    blocks: [\n      {\n        slug: 'block-name',\n        fields: [\n          {\n            name: 'blockField',\n            type: 'text',\n            required: true,\n          },\n        ],\n      },\n    ],".to_string(),
        _ => String::new(),
    };

    let default_and_specific = default_value_code.unwrap_or_default() + &field_specific;

    Ok(format!(
        "{{\n    name: '{name}',\n    type: '{field_type}',{required}{unique}{localized}{admin}{access}{validation}{default_and_specific}\n  }}",
        name = name,
        field_type = field_type,
        required = if required { "\n    required: true," } else { "" },
        unique = if unique { "\n    unique: true," } else { "" },
        localized = if localized { "\n    localized: true," } else { "" },
        admin = admin_code,
        access = access_code,
        validation = validation_code,
        default_and_specific = default_and_specific
    ))
}

fn generate_global_template(options: &Map<String, Value>) -> Result<String, String> {
    let slug = get_string(options, "slug").ok_or("Global slug is required")?;
    let fields = get_array(options, "fields").cloned().unwrap_or_default();
    let access = get_bool(options, "access", false);
    let versions = get_bool(options, "versions", false);
    let admin = options
        .get("admin")
        .and_then(|v| v.as_object())
        .cloned()
        .unwrap_or_default();

    let fields_code = if fields.is_empty() {
        String::new()
    } else {
        let mut lines = Vec::new();
        for field in fields {
            lines.push(generate_field_template_from_value(&field)?);
        }
        lines.join(",\n    ")
    };

    let admin_code = if admin.is_empty() {
        String::new()
    } else {
        admin
            .get("group")
            .and_then(|v| v.as_str())
            .map(|group| format!("\n  admin: {{\n    group: '{group}',\n  }},"))
            .unwrap_or_default()
    };

    let access_code = if access {
        "\n  access: {\n    read: () => true,\n    update: () => true,\n  },"
            .to_string()
    } else {
        String::new()
    };

    let versions_code = if versions {
        "\n  versions: {\n    drafts: true,\n  },".to_string()
    } else {
        String::new()
    };

    Ok(format!(
        "import {{ GlobalConfig }} from 'payload/types';\n\nconst {}: GlobalConfig = {{\n  slug: '{}',{}{}{}\n  fields: [\n    {}\n  ],\n}};\n\nexport default {};",
        capitalize(&slug),
        slug,
        admin_code,
        access_code,
        versions_code,
        fields_code,
        capitalize(&slug)
    ))
}

fn generate_config_template(options: &Map<String, Value>) -> Result<String, String> {
    let server_url = get_string(options, "serverURL").unwrap_or_else(|| "http://localhost:3000".to_string());
    let collections = get_array(options, "collections").cloned().unwrap_or_default();
    let globals = get_array(options, "globals").cloned().unwrap_or_default();
    let plugins = get_array(options, "plugins").cloned().unwrap_or_default();
    let db = get_string(options, "db").unwrap_or_else(|| "mongodb".to_string());
    let _typescript = get_bool(options, "typescript", true);

    let collections_code = if collections.is_empty() {
        String::new()
    } else {
        collections
            .iter()
            .filter_map(|c| c.as_str())
            .map(|c| format!("import {} from './collections/{}';", capitalize(c), c))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let globals_code = if globals.is_empty() {
        String::new()
    } else {
        globals
            .iter()
            .filter_map(|g| g.as_str())
            .map(|g| format!("import {} from './globals/{}';", capitalize(g), g))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let plugins_imports = plugins
        .iter()
        .filter_map(|p| p.as_str())
        .map(|plugin| match plugin {
            "form-builder" => "import formBuilder from '@payloadcms/plugin-form-builder';".to_string(),
            "seo" => "import seoPlugin from '@payloadcms/plugin-seo';".to_string(),
            "nested-docs" => "import nestedDocs from '@payloadcms/plugin-nested-docs';".to_string(),
            other => format!("import {} from '@payloadcms/plugin-{}';", other, other),
        })
        .collect::<Vec<_>>()
        .join("\n");

    let plugins_init = if plugins.is_empty() {
        String::new()
    } else {
        let mut parts = Vec::new();
        for plugin in plugins.iter().filter_map(|p| p.as_str()) {
            let code = match plugin {
                "form-builder" => "formBuilder({\n      formOverrides: {\n        admin: {\n          group: 'Content',\n        },\n      },\n      formSubmissionOverrides: {\n        admin: {\n          group: 'Content',\n        },\n      },\n      redirectRelationships: ['pages'],\n    }),"
                    .to_string(),
                "seo" => "seoPlugin(),".to_string(),
                "nested-docs" => "nestedDocs({\n      collections: ['pages'],\n    }),"
                    .to_string(),
                other => format!("{}(),", other),
            };
            parts.push(code);
        }
        format!("\n  plugins: [\n    {}\n  ],", parts.join("\n    "))
    };

    let collections_init = if collections.is_empty() {
        String::new()
    } else {
        let list = collections
            .iter()
            .filter_map(|c| c.as_str())
            .map(|c| format!("{},", capitalize(c)))
            .collect::<Vec<_>>()
            .join("\n    ");
        format!("\n  collections: [\n    {}\n  ],", list)
    };

    let globals_init = if globals.is_empty() {
        String::new()
    } else {
        let list = globals
            .iter()
            .filter_map(|g| g.as_str())
            .map(|g| format!("{},", capitalize(g)))
            .collect::<Vec<_>>()
            .join("\n    ");
        format!("\n  globals: [\n    {}\n  ],", list)
    };

    let db_imports = if db == "postgres" {
        "import { postgresAdapter } from '@payloadcms/db-postgres';"
    } else {
        "import { mongooseAdapter } from '@payloadcms/db-mongoose';"
    };

    let db_code = if db == "postgres" {
        "\n  db: postgresAdapter({\n    pool: {\n      connectionString: process.env.DATABASE_URI,\n    },\n  }),"
    } else {
        "\n  db: mongooseAdapter({\n    url: process.env.MONGODB_URI,\n  }),"
    };

    let admin = options
        .get("admin")
        .and_then(|v| v.as_object())
        .cloned()
        .unwrap_or_default();

    let bundler_imports = match admin.get("bundler").and_then(|v| v.as_str()) {
        Some("vite") => "import { viteBundler } from '@payloadcms/bundler-vite';",
        _ => "import { webpackBundler } from '@payloadcms/bundler-webpack';",
    };

    let admin_init = if admin.is_empty() {
        String::new()
    } else {
        let user = admin
            .get("user")
            .and_then(|v| v.as_str())
            .unwrap_or("users");
        let bundler = match admin.get("bundler").and_then(|v| v.as_str()) {
            Some("vite") => "viteBundler()",
            _ => "webpackBundler()",
        };
        format!(
            "\n  admin: {{\n    user: '{user}',\n    bundler: {bundler},\n    meta: {{\n      titleSuffix: '- Payload CMS',\n      favicon: '/assets/favicon.ico',\n      ogImage: '/assets/og-image.jpg',\n    }},\n  }},"
        )
    };

    let mut imports_section = format!(
        "import path from 'path';\nimport {{ buildConfig }} from 'payload/config';\n{}\n{}",
        db_imports, bundler_imports
    );
    if !collections_code.is_empty() {
        imports_section.push_str(&format!("\n{collections_code}"));
    }
    if !globals_code.is_empty() {
        imports_section.push_str(&format!("\n{globals_code}"));
    }
    if !plugins_imports.is_empty() {
        imports_section.push_str(&format!("\n{plugins_imports}"));
    }

    Ok(format!(
        "{}\n\nexport default buildConfig({{\n  serverURL: '{}',{}{}{}{}{}\n  typescript: {{\n    outputFile: path.resolve(__dirname, 'payload-types.ts'),\n  }},\n  graphQL: {{\n    schemaOutputFile: path.resolve(__dirname, 'generated-schema.graphql'),\n  }},\n  cors: ['http://localhost:3000'],\n  csrf: [\n    'http://localhost:3000',\n  ],\n}});",
        imports_section,
        server_url,
        admin_init,
        db_code,
        plugins_init,
        collections_init,
        globals_init
    ))
}

fn generate_access_control_template(options: &Map<String, Value>) -> Result<String, String> {
    let name = get_string(options, "name").unwrap_or_else(|| "default".to_string());
    let roles = options
        .get("roles")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_else(|| vec![json!("admin"), json!("editor"), json!("user")]);

    let roles_union = roles
        .iter()
        .filter_map(|r| r.as_str())
        .map(|r| format!("'{}'", r))
        .collect::<Vec<_>>()
        .join(" | ");

    Ok(format!(
        "import {{ Access }} from 'payload/types';\n\ntype Role = {roles_union};\n\nexport const {name}Access: Access = ({{ req }}) => {{\n  if (!req.user) {{\n    return false;\n  }}\n\n  if (req.user.role === 'admin') {{\n    return true;\n  }}\n\n  if (req.user.role === 'editor') {{\n    return {{\n      read: true,\n      update: true,\n      create: true,\n      delete: false,\n    }};\n  }}\n\n  if (req.user.role === 'user') {{\n    return {{\n      read: {{\n        and: [\n          {{\n            createdBy: {{\n              equals: req.user.id,\n            }},\n          }},\n        ],\n      }},\n      update: {{\n        createdBy: {{\n          equals: req.user.id,\n        }},\n      }},\n      create: true,\n      delete: {{\n        createdBy: {{\n          equals: req.user.id,\n        }},\n      }},\n    }};\n  }}\n\n  return false;\n}};"
    ))
}

fn generate_hook_template(options: &Map<String, Value>) -> Result<String, String> {
    let template_type = get_string(options, "type").unwrap_or_else(|| "collection".to_string());
    let name = get_string(options, "name").unwrap_or_else(|| "default".to_string());
    let operation = get_string(options, "operation").unwrap_or_else(|| "create".to_string());
    let timing = get_string(options, "timing").unwrap_or_else(|| "before".to_string());
    let timing_type = if timing == "before" {
        "BeforeOperation"
    } else {
        "AfterOperation"
    };

    Ok(format!(
        "import {{ {} }} from 'payload/types';\n\nexport const {}{}Hook: {} = async ({{ \n  req, \n  data, \n  operation,{}\n  {}{}\n}}) => {{\n  console.log(`{} {} operation on {} {}`);\n  {} \n}};",
        timing_type,
        timing,
        capitalize(&operation),
        timing_type,
        if timing == "after" { "\n  doc," } else { "" },
        if timing == "after" { "previousDoc,\n" } else { "" },
        "",
        timing,
        operation,
        template_type,
        name,
        if timing == "before" {
            "return data;"
        } else {
            "return doc;"
        }
    ))
}

fn generate_endpoint_template(options: &Map<String, Value>) -> Result<String, String> {
    let path = get_string(options, "path").unwrap_or_else(|| "/api/custom".to_string());
    let method = get_string(options, "method").unwrap_or_else(|| "get".to_string());
    let auth = get_bool(options, "auth", true);

    let handler_name = format!(
        "{}{}",
        method,
        path.replace('/', "_")
            .trim_matches('_')
            .replace("__", "_")
    );

    Ok(format!(
        "import {{ Payload }} from 'payload';\nimport {{ Request, Response }} from 'express';\n\nexport const {} = async (req: Request, res: Response, payload: Payload) => {{\n  try {{\n    {}    const result = {{\n      message: 'Success',\n      timestamp: new Date().toISOString(),\n    }};\n\n    return res.status(200).json(result);\n  }} catch (error) {{\n    console.error(`Error in {} endpoint:`, error);\n    return res.status(500).json({{\n      message: 'Internal Server Error',\n      error: error.message,\n    }});\n  }}\n}};\n\nexport default {{\n  path: '{}',\n  method: '{}',\n  handler: {},\n}};",
        handler_name,
        if auth {
            "if (!req.user) {\n      return res.status(401).json({\n        message: 'Unauthorized',\n      });\n    }\n\n    "
        } else {
            ""
        },
        path,
        path,
        method,
        handler_name
    ))
}

fn generate_plugin_template(options: &Map<String, Value>) -> Result<String, String> {
    let name = get_string(options, "name").unwrap_or_else(|| "custom-plugin".to_string());
    let collections = get_array(options, "collections").cloned().unwrap_or_default();
    let globals = get_array(options, "globals").cloned().unwrap_or_default();
    let endpoints = get_array(options, "endpoints").cloned().unwrap_or_default();

    let plugin_type_name = sanitize_identifier(&name);

    let collections_code = if collections.is_empty() {
        "// No collections to add".to_string()
    } else {
        format!(
            "\n      const collections = [\n        {}\n      ];\n      \n      config.collections = [\n        ...(config.collections || []),\n        ...collections,\n      ];",
            collections
                .iter()
                .filter_map(|c| c.as_str())
                .map(|c| format!(
                    "{{\n          slug: '{}',\n        }}",
                    c
                ))
                .collect::<Vec<_>>()
                .join(",\n        ")
        )
    };

    let globals_code = if globals.is_empty() {
        "// No globals to add".to_string()
    } else {
        format!(
            "\n      const globals = [\n        {}\n      ];\n      \n      config.globals = [\n        ...(config.globals || []),\n        ...globals,\n      ];",
            globals
                .iter()
                .filter_map(|g| g.as_str())
                .map(|g| format!(
                    "{{\n          slug: '{}',\n        }}",
                    g
                ))
                .collect::<Vec<_>>()
                .join(",\n        ")
        )
    };

    let endpoints_code = if endpoints.is_empty() {
        "// No endpoints to add".to_string()
    } else {
        format!(
            "\n      const endpoints = [\n        {}\n      ];\n      \n      config.endpoints = [\n        ...(config.endpoints || []),\n        ...endpoints,\n      ];",
            endpoints
                .iter()
                .filter_map(|e| e.as_str())
                .map(|e| format!(
                    "{{\n          path: '/{}',\n          method: 'get',\n          handler: async (req, res) => {{\n            res.status(200).json({{ message: '{} endpoint' }});\n          }},\n        }}",
                    e, e
                ))
                .collect::<Vec<_>>()
                .join(",\n        ")
        )
    };

    Ok(format!(
        "import {{ Config, Plugin }} from 'payload/config';\n\nexport interface {}PluginOptions {{\n  enabled?: boolean;\n}}\n\nexport const {}Plugin = (options: {}PluginOptions = {{}}): Plugin => {{\n  return {{\n    name: '{}',\n    config: (incomingConfig: Config): Config => {{\n      const {{ enabled = true }} = options;\n      \n      if (!enabled) {{\n        return incomingConfig;\n      }}\n      \n      const config = {{ ...incomingConfig }};{}\n      {}\n      {}\n      return config;\n    }},\n  }};\n}};\n\nexport default {}Plugin;",
        plugin_type_name,
        sanitize_identifier(&name),
        plugin_type_name,
        name,
        collections_code,
        globals_code,
        endpoints_code,
        sanitize_identifier(&name)
    ))
}

fn generate_block_template(options: &Map<String, Value>) -> Result<String, String> {
    let name = get_string(options, "name").unwrap_or_else(|| "custom-block".to_string());
    let fields = get_array(options, "fields").cloned().unwrap_or_default();
    let image_field = get_bool(options, "imageField", true);
    let content_field = get_bool(options, "contentField", true);

    let fields_code = if fields.is_empty() {
        String::new()
    } else {
        let mut parts = Vec::new();
        for field in fields {
            parts.push(generate_field_template_from_value(&field)?);
        }
        parts.join(",\n    ")
    };

    let image_code = if image_field {
        "{
    name: 'image',
    type: 'upload',
    relationTo: 'media',
    required: true,
    admin: {
      description: 'Add an image to this block',
    },
  },"
            .to_string()
    } else {
        String::new()
    };

    let content_code = if content_field {
        "{
    name: 'content',
    type: 'richText',
    required: true,
    admin: {
      description: 'Add content to this block',
    },
  },"
            .to_string()
    } else {
        String::new()
    };

    Ok(format!(
        "import {{ Block }} from 'payload/types';\n\nexport const {}Block: Block = {{\n  slug: '{}',\n  labels: {{\n    singular: '{}',\n    plural: '{}s',\n  }},\n  fields: [\n    {}\n    {}\n    {}\n  ],\n}};\n\nexport default {}Block;",
        sanitize_identifier(&name),
        name,
        capitalize_words(&name.replace('-', " ")),
        capitalize_words(&name.replace('-', " ")),
        image_code,
        content_code,
        fields_code,
        sanitize_identifier(&name)
    ))
}

fn generate_migration_template(options: &Map<String, Value>) -> Result<String, String> {
    let name = get_string(options, "name").unwrap_or_else(|| "custom-migration".to_string());
    let collection = get_string(options, "collection").unwrap_or_default();
    let operation = get_string(options, "operation").unwrap_or_else(|| "update".to_string());

    let body = if collection.is_empty() {
        "// Add your migration logic here\n    // This could be schema changes, data transformations, etc.\n    ".to_string()
    } else if operation == "delete" {
        format!(
            "// Get the collection\n    const collection = '{collection}';\n    \n    const docs = await payload.find({{\n      collection,\n      limit: 100,\n    }});\n    \n    console.log(`Found ${{docs.docs.length}} documents to migrate`);\n    \n    for (const doc of docs.docs) {{\n      await payload.delete({{\n        collection,\n        id: doc.id,\n      }});\n    }}\n    ",
        )
    } else {
        format!(
            "// Get the collection\n    const collection = '{collection}';\n    \n    const docs = await payload.find({{\n      collection,\n      limit: 100,\n    }});\n    \n    console.log(`Found ${{docs.docs.length}} documents to migrate`);\n    \n    for (const doc of docs.docs) {{\n      await payload.update({{\n        collection,\n        id: doc.id,\n        data: {{\n          migratedAt: new Date().toISOString(),\n        }},\n      }});\n    }}\n    ",
        )
    };

    Ok(format!(
        "import {{ Payload }} from 'payload';\n\nexport const {}Migration = async (payload: Payload) => {{\n  try {{\n    console.log('Starting migration: {}');\n    \n    {}    console.log('Migration completed successfully: {}');\n    return {{ success: true }};\n  }} catch (error) {{\n    console.error('Migration failed:', error);\n    return {{ success: false, error: error.message }};\n  }}\n}};\n\nexport default {}Migration;",
        sanitize_identifier(&name),
        name,
        body,
        name,
        sanitize_identifier(&name)
    ))
}

fn capitalize(value: &str) -> String {
    let mut chars = value.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

fn capitalize_words(value: &str) -> String {
    value
        .split_whitespace()
        .map(|word| capitalize(word))
        .collect::<Vec<_>>()
        .join(" ")
}

fn sanitize_identifier(value: &str) -> String {
    let mut out = String::new();
    for (idx, ch) in value.chars().enumerate() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            if idx == 0 && ch.is_ascii_digit() {
                out.push('_');
            }
            out.push(ch);
        } else if ch == '-' {
            out.push('_');
        }
    }
    if out.is_empty() {
        "_plugin".to_string()
    } else {
        out
    }
}

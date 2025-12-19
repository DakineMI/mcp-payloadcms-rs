use std::collections::HashMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::tools::generator::{generate_template, TemplateType};

#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FieldOption {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: String,
    pub required: Option<bool>,
    pub unique: Option<bool>,
    pub localized: Option<bool>,
    pub validation: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CollectionOption {
    pub name: String,
    pub fields: Option<Vec<FieldOption>>,
    pub auth: Option<bool>,
    pub timestamps: Option<bool>,
    pub admin: Option<AdminOption>,
    pub versions: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GlobalOption {
    pub name: String,
    pub fields: Option<Vec<FieldOption>>,
    pub versions: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BlockOption {
    pub name: String,
    pub fields: Option<Vec<FieldOption>>,
    pub image_field: Option<bool>,
    pub content_field: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AdminOption {
    pub user: Option<String>,
    pub bundler: Option<String>,
    pub use_as_title: Option<String>,
    pub group: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ScaffoldOptions {
    pub project_name: String,
    pub description: Option<String>,
    pub server_url: Option<String>,
    pub database: Option<String>,
    pub auth: Option<bool>,
    pub admin: Option<AdminOption>,
    pub collections: Option<Vec<CollectionOption>>,
    pub globals: Option<Vec<GlobalOption>>,
    pub blocks: Option<Vec<BlockOption>>,
    pub plugins: Option<Vec<String>>,
    pub typescript: Option<bool>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ScaffoldFile {
    File(String),
    Directory(ScaffoldFileStructure),
}

pub type ScaffoldFileStructure = HashMap<String, ScaffoldFile>;

pub fn scaffold_project(options: &ScaffoldOptions) -> ScaffoldFileStructure {
    let description = options
        .description
        .clone()
        .unwrap_or_else(|| "A Payload CMS 3 project".to_string());
    let server_url = options
        .server_url
        .clone()
        .unwrap_or_else(|| "http://localhost:3000".to_string());
    let database = options
        .database
        .clone()
        .unwrap_or_else(|| "mongodb".to_string());
    let typescript = options.typescript.unwrap_or(true);

    let mut root = ScaffoldFileStructure::new();

    root.insert(
        "package.json".to_string(),
        ScaffoldFile::File(generate_package_json(
            &options.project_name,
            &description,
            &database,
            typescript,
            options.plugins.clone().unwrap_or_default(),
        )),
    );
    root.insert(
        "tsconfig.json".to_string(),
        ScaffoldFile::File(generate_ts_config()),
    );
    root.insert(
        ".env".to_string(),
        ScaffoldFile::File(generate_env_file(&database)),
    );
    root.insert(
        ".env.example".to_string(),
        ScaffoldFile::File(generate_env_file(&database)),
    );
    root.insert(
        ".gitignore".to_string(),
        ScaffoldFile::File(generate_gitignore()),
    );
    root.insert(
        "README.md".to_string(),
        ScaffoldFile::File(generate_readme(
            &options.project_name,
            &description,
        )),
    );

    // src directory
    let mut src = ScaffoldFileStructure::new();
    src.insert(
        "payload.config.ts".to_string(),
        ScaffoldFile::File(generate_payload_config(
            &options.project_name,
            &server_url,
            &database,
            options.admin.as_ref(),
            typescript,
        )),
    );

    // collections
    let mut collections_dir = ScaffoldFileStructure::new();
    if let Some(collections) = &options.collections {
        for collection in collections {
            let mut field_values = Vec::new();
            if let Some(fields) = &collection.fields {
                for field in fields {
                    if let Ok(val) = serde_json::to_value(field) {
                        field_values.push(val);
                    }
                }
            }

            let mut opts = serde_json::Map::new();
            opts.insert("slug".to_string(), json!(collection.name));
            opts.insert("fields".to_string(), Value::Array(field_values));
            opts.insert("auth".to_string(), json!(collection.auth.unwrap_or(false)));
            opts.insert(
                "timestamps".to_string(),
                json!(collection.timestamps.unwrap_or(true)),
            );
            if let Some(admin) = &collection.admin {
                if let Ok(val) = serde_json::to_value(admin) {
                    opts.insert("admin".to_string(), val);
                }
            }
            opts.insert(
                "versions".to_string(),
                json!(collection.versions.unwrap_or(false)),
            );
            opts.insert("access".to_string(), json!(true));
            opts.insert("hooks".to_string(), json!(true));

            let code = match generate_template(TemplateType::Collection, &Value::Object(opts)) {
                Ok(code) => code,
                Err(err) => format!("// Failed to generate collection: {err}"),
            };

            collections_dir.insert(
                format!("{}.ts", collection.name),
                ScaffoldFile::File(code),
            );
        }
    }
    src.insert("collections".to_string(), ScaffoldFile::Directory(collections_dir));

    // globals
    let mut globals_dir = ScaffoldFileStructure::new();
    if let Some(globals) = &options.globals {
        for global in globals {
            let mut field_values = Vec::new();
            if let Some(fields) = &global.fields {
                for field in fields {
                    if let Ok(val) = serde_json::to_value(field) {
                        field_values.push(val);
                    }
                }
            }

            let mut opts = serde_json::Map::new();
            opts.insert("slug".to_string(), json!(global.name));
            opts.insert("fields".to_string(), Value::Array(field_values));
            opts.insert(
                "versions".to_string(),
                json!(global.versions.unwrap_or(false)),
            );
            opts.insert("access".to_string(), json!(true));

            let code = match generate_template(TemplateType::Global, &Value::Object(opts)) {
                Ok(code) => code,
                Err(err) => format!("// Failed to generate global: {err}"),
            };

            globals_dir.insert(format!("{}.ts", global.name), ScaffoldFile::File(code));
        }
    }
    src.insert("globals".to_string(), ScaffoldFile::Directory(globals_dir));

    // blocks
    let mut blocks_dir = ScaffoldFileStructure::new();
    if let Some(blocks) = &options.blocks {
        for block in blocks {
            let mut field_values = Vec::new();
            if let Some(fields) = &block.fields {
                for field in fields {
                    if let Ok(val) = serde_json::to_value(field) {
                        field_values.push(val);
                    }
                }
            }

            let mut opts = serde_json::Map::new();
            opts.insert("name".to_string(), json!(block.name));
            opts.insert("fields".to_string(), Value::Array(field_values));
            opts.insert(
                "imageField".to_string(),
                json!(block.image_field.unwrap_or(true)),
            );
            opts.insert(
                "contentField".to_string(),
                json!(block.content_field.unwrap_or(true)),
            );

            let code = match generate_template(TemplateType::Block, &Value::Object(opts)) {
                Ok(code) => code,
                Err(err) => format!("// Failed to generate block: {err}"),
            };

            blocks_dir.insert(format!("{}.ts", block.name), ScaffoldFile::File(code));
        }
    }
    src.insert("blocks".to_string(), ScaffoldFile::Directory(blocks_dir));

    // access
    let mut access_dir = ScaffoldFileStructure::new();
    access_dir.insert(
        "index.ts".to_string(),
        ScaffoldFile::File(generate_access_index()),
    );
    src.insert("access".to_string(), ScaffoldFile::Directory(access_dir));

    // hooks
    let mut hooks_dir = ScaffoldFileStructure::new();
    hooks_dir.insert(
        "index.ts".to_string(),
        ScaffoldFile::File(generate_hooks_index()),
    );
    src.insert("hooks".to_string(), ScaffoldFile::Directory(hooks_dir));

    // endpoints
    let mut endpoints_dir = ScaffoldFileStructure::new();
    endpoints_dir.insert(
        "index.ts".to_string(),
        ScaffoldFile::File(generate_endpoints_index()),
    );
    src.insert("endpoints".to_string(), ScaffoldFile::Directory(endpoints_dir));

    // server
    src.insert("server.ts".to_string(), ScaffoldFile::File(generate_server()));

    root.insert("src".to_string(), ScaffoldFile::Directory(src));

    root
}

pub fn validate_scaffold_options(options: &ScaffoldOptions) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    if options.project_name.trim().is_empty() {
        errors.push("Project name is required".to_string());
    }

    if let Some(server_url) = &options.server_url {
        if !(server_url.starts_with("http://") || server_url.starts_with("https://")) {
            errors.push("Server URL must be a valid URL".to_string());
        }
    }

    if let Some(database) = &options.database {
        if database != "mongodb" && database != "postgres" {
            errors.push("Database must be either 'mongodb' or 'postgres'".to_string());
        }
    }

    if let Some(collections) = &options.collections {
        for collection in collections {
            if collection.name.trim().is_empty() {
                errors.push("Collection name is required".to_string());
            }
            if let Some(fields) = &collection.fields {
                for field in fields {
                    if field.name.trim().is_empty() {
                        errors.push("Field name is required".to_string());
                    }
                    if field.field_type.trim().is_empty() {
                        errors.push("Field type is required".to_string());
                    }
                }
            }
        }
    }

    if let Some(globals) = &options.globals {
        for global in globals {
            if global.name.trim().is_empty() {
                errors.push("Global name is required".to_string());
            }
            if let Some(fields) = &global.fields {
                for field in fields {
                    if field.name.trim().is_empty() {
                        errors.push("Field name is required".to_string());
                    }
                    if field.field_type.trim().is_empty() {
                        errors.push("Field type is required".to_string());
                    }
                }
            }
        }
    }

    if let Some(blocks) = &options.blocks {
        for block in blocks {
            if block.name.trim().is_empty() {
                errors.push("Block name is required".to_string());
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn generate_package_json(
    project_name: &str,
    description: &str,
    database: &str,
    typescript: bool,
    plugins: Vec<String>,
) -> String {
    let db_dependency = if database == "mongodb" {
        "\"@payloadcms/db-mongodb\": \"^1.0.0\","
    } else {
        "\"@payloadcms/db-postgres\": \"^1.0.0\","
    };

    let plugin_dependencies = plugins
        .iter()
        .filter_map(|plugin| match plugin.as_str() {
            "seo" => Some("\"@payloadcms/plugin-seo\": \"^1.0.0\","),
            "nested-docs" => Some("\"@payloadcms/plugin-nested-docs\": \"^1.0.0\","),
            "form-builder" => Some("\"@payloadcms/plugin-form-builder\": \"^1.0.0\","),
            "cloud" => Some("\"@payloadcms/plugin-cloud\": \"^1.0.0\","),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n    ");

    format!(
        "{{\n  \"name\": \"{}\",\n  \"description\": \"{}\",\n  \"version\": \"1.0.0\",\n  \"main\": \"dist/server.js\",\n  \"license\": \"MIT\",\n  \"scripts\": {{\n    \"dev\": \"cross-env PAYLOAD_CONFIG_PATH=src/payload.config.ts nodemon\",\n    \"build:payload\": \"cross-env PAYLOAD_CONFIG_PATH=src/payload.config.ts payload build\",\n    \"build:server\": \"{}\",\n    \"build\": \"yarn build:payload && yarn build:server\",\n    \"start\": \"cross-env PAYLOAD_CONFIG_PATH=dist/payload.config.js NODE_ENV=production node dist/server.js\",\n    \"generate:types\": \"cross-env PAYLOAD_CONFIG_PATH=src/payload.config.ts payload generate:types\",\n    \"generate:graphQLSchema\": \"cross-env PAYLOAD_CONFIG_PATH=src/payload.config.ts payload generate:graphQLSchema\"\n  }},\n  \"dependencies\": {{\n    \"payload\": \"^2.0.0\",\n    {}\n    \"@payloadcms/richtext-lexical\": \"^1.0.0\",\n    {}\n    \"dotenv\": \"^16.0.0\",\n    \"express\": \"^4.17.1\"\n  }},\n  \"devDependencies\": {{\n    {}\n    \"cross-env\": \"^7.0.3\",\n    \"nodemon\": \"^2.0.6\",\n    {}\n    \"payload-types\": \"file:src/payload-types.ts\"\n  }}\n}}",
        project_name
            .to_lowercase()
            .replace(|c: char| !c.is_ascii_alphanumeric() && c != '-', "-"),
        description,
        if typescript {
            "tsc"
        } else {
            "copyfiles src/* dist/"
        },
        db_dependency,
        plugin_dependencies,
        if typescript {
            "\"typescript\": \"^5.0.0\",\n    \"@types/express\": \"^4.17.9\","
        } else {
            ""
        },
        if typescript { "" } else { "\"copyfiles\": \"^2.4.1\"," }
    )
}

fn generate_ts_config() -> String {
    "{\n  \"compilerOptions\": {\n    \"target\": \"es2020\",\n    \"module\": \"commonjs\",\n    \"moduleResolution\": \"node\",\n    \"esModuleInterop\": true,\n    \"strict\": true,\n    \"outDir\": \"dist\",\n    \"rootDir\": \"src\",\n    \"skipLibCheck\": true,\n    \"sourceMap\": true,\n    \"declaration\": true,\n    \"jsx\": \"react\",\n    \"baseUrl\": \".\",\n    \"paths\": {\n      \"payload/generated-types\": [\"src/payload-types.ts\"]\n    }\n  },\n  \"include\": [\"src\"],\n  \"exclude\": [\"node_modules\", \"dist\"]\n}"
        .to_string()
}

fn generate_env_file(database: &str) -> String {
    format!(
        "# Server\nPORT=3000\nNODE_ENV=development\n\n# Database\n{}\n\n# Payload\nPAYLOAD_SECRET=your-payload-secret-key-here\nPAYLOAD_PUBLIC_SERVER_URL=http://localhost:3000",
        if database == "mongodb" {
            "MONGODB_URI=mongodb://localhost:27017/payload-cms-3-project"
        } else {
            "DATABASE_URI=postgres://postgres:postgres@localhost:5432/payload-cms-3-project"
        }
    )
}

fn generate_gitignore() -> String {
    "# dependencies\n/node_modules\n\n# build\n/dist\n/build\n\n# misc\n.DS_Store\n.env\n.env.local\n.env.development.local\n.env.test.local\n.env.production.local\n\n# logs\nnpm-debug.log*\nyarn-debug.log*\nyarn-error.log*\n\n# payload\n/src/payload-types.ts"
        .to_string()
}

fn generate_readme(project_name: &str, description: &str) -> String {
    format!(
        "# {}\n\n{}\n\n## Getting Started\n\n### Development\n\n1. Clone this repository\n2. Install dependencies with `yarn` or `npm install`\n3. Copy `.env.example` to `.env` and configure your environment variables\n4. Start the development server with `yarn dev` or `npm run dev`\n5. Visit http://localhost:3000/admin to access the admin panel\n\n### Production\n\n1. Build the project with `yarn build` or `npm run build`\n2. Start the production server with `yarn start` or `npm start`\n\n## Features\n\n- Payload CMS 3.0\n- TypeScript\n- Express server\n- Admin panel\n- API endpoints\n- GraphQL API\n\n## Project Structure\n\n- `/src` - Source code\n  - `/collections` - Collection definitions\n  - `/globals` - Global definitions\n  - `/blocks` - Block definitions\n  - `/access` - Access control functions\n  - `/hooks` - Hook functions\n  - `/endpoints` - Custom API endpoints\n  - `payload.config.ts` - Payload configuration\n  - `server.ts` - Express server\n\n## License\n\nMIT",
        project_name, description
    )
}

fn generate_payload_config(
    project_name: &str,
    server_url: &str,
    database: &str,
    admin: Option<&AdminOption>,
    typescript: bool,
) -> String {
    let mut opts = serde_json::Map::new();
    opts.insert("projectName".to_string(), json!(project_name));
    opts.insert("serverURL".to_string(), json!(server_url));
    opts.insert("db".to_string(), json!(database));
    opts.insert("typescript".to_string(), json!(typescript));
    opts.insert("csrf".to_string(), json!(true));
    opts.insert("rateLimit".to_string(), json!(true));
    if let Some(admin) = admin {
        if let Ok(val) = serde_json::to_value(admin) {
            opts.insert("admin".to_string(), val);
        }
    }

    generate_template(TemplateType::Config, &Value::Object(opts))
        .unwrap_or_else(|err| format!("// Failed to generate config: {err}"))
}

fn generate_access_index() -> String {
    "// Export all access control functions\nexport * from './isAdmin';\nexport * from './isAdminOrEditor';\nexport * from './isAdminOrSelf';\n\nexport const isAdmin = ({ req }) => {\n  return req.user?.role === 'admin';\n};\n\nexport const isAdminOrEditor = ({ req }) => {\n  return ['admin', 'editor'].includes(req.user?.role);\n};\n\nexport const isAdminOrSelf = ({ req }) => {\n  const { user } = req;\n  \n  if (!user) return false;\n  if (user.role === 'admin') return true;\n  \n  const id = req.params?.id;\n  if (id && user.id === id) return true;\n  \n  return false;\n};"
        .to_string()
}

fn generate_hooks_index() -> String {
    "// Export all hook functions\nexport * from './populateCreatedBy';\nexport * from './formatSlug';\n\nexport const populateCreatedBy = ({ req }) => {\n  return {\n    createdBy: req.user?.id,\n  };\n};\n\nexport const formatSlug = ({ value }) => {\n  if (!value) return '';\n  \n  return value\n    .toLowerCase()\n    .replace(/ /g, '-')\n    .replace(/[^\\w-]+/g, '');\n};"
        .to_string()
}

fn generate_endpoints_index() -> String {
    "import { Payload } from 'payload';\nimport { Request, Response } from 'express';\n\nexport const registerEndpoints = (payload: Payload): void => {\n  payload.router.get('/api/health', (req: Request, res: Response) => {\n    res.status(200).json({\n      status: 'ok',\n      message: 'API is healthy',\n      timestamp: new Date().toISOString(),\n    });\n  });\n  \n  payload.router.get('/api/custom-data', async (req: Request, res: Response) => {\n    try {\n      res.status(200).json({\n        message: 'Custom data endpoint',\n      });\n    } catch (error) {\n      res.status(500).json({\n        message: 'Error fetching data',\n        error: error.message,\n      });\n    }\n  });\n};"
        .to_string()
}

fn generate_server() -> String {
    "import express from 'express';\nimport payload from 'payload';\nimport { registerEndpoints } from './endpoints';\nimport path from 'path';\n\nrequire('dotenv').config();\n\nconst app = express();\n\napp.get('/', (_, res) => {\n  res.redirect('/admin');\n});\n\nconst start = async () => {\n  await payload.init({\n    secret: process.env.PAYLOAD_SECRET || 'your-payload-secret-key-here',\n    express: app,\n    onInit: () => {\n      payload.logger.info(`Payload Admin URL: ${payload.getAdminURL()}`);\n    },\n  });\n\n  registerEndpoints(payload);\n\n  app.get('/api/custom-route', (req, res) => {\n    res.json({ message: 'Custom route' });\n  });\n\n  app.use('/public', express.static(path.resolve(__dirname, '../public')));\n\n  const PORT = process.env.PORT || 3000;\n  app.listen(PORT, () => {\n    payload.logger.info(`Server started on port ${PORT}`);\n  });\n};\n\nstart();"
        .to_string()
}

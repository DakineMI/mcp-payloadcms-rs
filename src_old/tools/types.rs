use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum FileType {
    Collection,
    Field,
    Global,
    Config,
}

impl FileType {
    pub fn as_str(&self) -> &'static str {
        match self {
            FileType::Collection => "collection",
            FileType::Field => "field",
            FileType::Global => "global",
            FileType::Config => "config",
        }
    }
}

impl fmt::Display for FileType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for FileType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "collection" => Ok(FileType::Collection),
            "field" => Ok(FileType::Field),
            "global" => Ok(FileType::Global),
            "config" => Ok(FileType::Config),
            _ => Err(format!("Unknown file type: {s}")),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct ValidationError {
    pub message: String,
    pub path: Option<String>,
    pub line: Option<usize>,
    pub column: Option<usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct Suggestion {
    pub message: String,
    pub code: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct Reference {
    pub title: String,
    pub url: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub suggestions: Vec<Suggestion>,
    pub references: Vec<Reference>,
}

impl ValidationResult {
    pub fn ok() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            suggestions: Vec::new(),
            references: Vec::new(),
        }
    }

    pub fn with_errors(errors: Vec<String>) -> Self {
        Self {
            is_valid: errors.is_empty(),
            errors,
            warnings: Vec::new(),
            suggestions: Vec::new(),
            references: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct SqlQueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct Examples {
    pub valid: Vec<String>,
    pub invalid: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct ValidationRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub file_types: Vec<FileType>,
    pub examples: Examples,
}

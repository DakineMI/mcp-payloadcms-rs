use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Todo {
    pub id: String,
    pub title: String,
    pub description: String,
    pub complexity: u8,
    pub code_example: Option<String>,
    pub is_complete: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Goal {
    pub id: String,
    pub description: String,
    pub created_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ImplementationPlan {
    pub goal_id: String,
    pub todos: Vec<Todo>,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct StorageData {
    pub goals: HashMap<String, Goal>,
    pub plans: HashMap<String, ImplementationPlan>,
}

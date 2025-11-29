use chrono::Utc;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;
use thiserror::Error;
use ulid::Ulid;

use crate::types::{Goal, ImplementationPlan, StorageData, Todo};

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Plan not found: {0}")]
    PlanNotFound(String),
    #[error("Todo not found: {0}")]
    TodoNotFound(String),
}

pub struct Storage {
    storage_path: PathBuf,
    data: StorageData,
}

impl Storage {
    pub fn new() -> Self {
        let home = dirs::home_dir().expect("couldn't find home dir");
        let data_dir = home.join(".software-planning-tool");
        let storage_path = data_dir.join("data.json");
        Self {
            storage_path,
            data: StorageData::default(),
        }
    }

    // `with_path` helper was only used for testing; removed at user request.

    pub fn initialize(&mut self) -> Result<(), StorageError> {
        let data_dir = self.storage_path.parent().unwrap();
        fs::create_dir_all(data_dir)?;

        if self.storage_path.exists() {
            let mut file = File::open(&self.storage_path)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            self.data = serde_json::from_str(&contents)?;
        } else {
            self.save()?;
        }

        Ok(())
    }

    pub fn snapshot_for_save(&self) -> (PathBuf, StorageData) {
        (self.storage_path.clone(), self.data.clone())
    }

    pub async fn save_snapshot_async(
        storage_path: PathBuf,
        data: StorageData,
    ) -> Result<(), StorageError> {
        let inner_res: Result<(), StorageError> =
            tokio::task::spawn_blocking(move || -> Result<(), StorageError> {
                if let Some(parent) = storage_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }

                let temp = storage_path.with_extension("tmp");
                let mut f = std::fs::File::create(&temp)?;
                let content = serde_json::to_string_pretty(&data)?;
                f.write_all(content.as_bytes())?;
                f.sync_all()?;
                std::fs::rename(temp, &storage_path)?;
                Ok(())
            })
            .await
            .map_err(|e| {
                StorageError::Io(std::io::Error::other(format!(
                    "spawn_blocking failed: {}",
                    e
                )))
            })?;
        inner_res?;
        Ok(())
    }

    /// Persist the current storage content synchronously to disk using a
    /// temporary file and an atomic rename to avoid partial writes.
    pub fn save(&self) -> Result<(), StorageError> {
        if let Some(parent) = self.storage_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let temp = self.storage_path.with_extension("tmp");
        let mut f = File::create(&temp)?;
        let content = serde_json::to_string_pretty(&self.data)?;
        f.write_all(content.as_bytes())?;
        f.sync_all()?;
        fs::rename(temp, &self.storage_path)?;
        Ok(())
    }

    /// Persist the current storage content asynchronously using a blocking
    /// task, which delegates to the synchronous `save()` implementation.
    pub async fn save_async(&self) -> Result<(), StorageError> {
        // Snapshot storage_path and data to avoid holding the borrow and to
        // ensure we move owned values into the blocking task.
        let (path, data) = self.snapshot_for_save();
        Storage::save_snapshot_async(path, data).await
    }

    // done

    pub fn create_goal(&mut self, description: String) -> Result<Goal, StorageError> {
        let goal = Goal {
            id: Ulid::new().to_string(),
            description,
            created_at: Utc::now().to_rfc3339(),
        };
        self.data.goals.insert(goal.id.clone(), goal.clone());
        // No immediate disk write; callers should call save_async() to persist changes.
        Ok(goal)
    }

    pub fn get_goal(&self, id: &str) -> Option<Goal> {
        self.data.goals.get(id).cloned()
    }

    pub fn create_plan(&mut self, goal_id: String) -> Result<ImplementationPlan, StorageError> {
        let plan = ImplementationPlan {
            goal_id: goal_id.clone(),
            todos: vec![],
            updated_at: Utc::now().to_rfc3339(),
        };
        self.data.plans.insert(goal_id.clone(), plan.clone());
        // No immediate disk write; callers should call save_async() to persist changes.
        Ok(plan)
    }

    pub fn get_plan(&self, goal_id: &str) -> Option<ImplementationPlan> {
        self.data.plans.get(goal_id).cloned()
    }

    pub fn add_todo(&mut self, goal_id: &str, todo: Todo) -> Result<Todo, StorageError> {
        // Scope the mutable borrow so we can call `self.save()` after it is dropped
        let added = {
            let plan = self
                .data
                .plans
                .get_mut(goal_id)
                .ok_or(StorageError::PlanNotFound(goal_id.to_string()))?;

            let mut todo = todo;
            todo.id = Ulid::new().to_string();
            let now = Utc::now().to_rfc3339();
            todo.created_at = now.clone();
            todo.updated_at = now.clone();
            todo.is_complete = false;

            plan.todos.push(todo.clone());
            plan.updated_at = Utc::now().to_rfc3339();
            todo
        };

        // No immediate disk write; callers should call save_async() to persist changes.
        Ok(added)
    }

    pub fn remove_todo(&mut self, goal_id: &str, todo_id: &str) -> Result<(), StorageError> {
        // Narrow the borrow on plan so we can call save() afterwards
        let removed = {
            let plan = self
                .data
                .plans
                .get_mut(goal_id)
                .ok_or(StorageError::PlanNotFound(goal_id.to_string()))?;
            let before = plan.todos.len();
            plan.todos.retain(|t| t.id != todo_id);
            if plan.todos.len() == before {
                return Err(StorageError::TodoNotFound(todo_id.to_string()));
            }
            plan.updated_at = Utc::now().to_rfc3339();
            true
        };

        if removed {
            // No immediate disk write; callers should call save_async() to persist changes.
            Ok(())
        } else {
            Err(StorageError::TodoNotFound(todo_id.to_string()))
        }
    }

    pub fn update_todo_status(
        &mut self,
        goal_id: &str,
        todo_id: &str,
        is_complete: bool,
    ) -> Result<Todo, StorageError> {
        let updated = {
            let plan = self
                .data
                .plans
                .get_mut(goal_id)
                .ok_or(StorageError::PlanNotFound(goal_id.to_string()))?;
            let todo = plan
                .todos
                .iter_mut()
                .find(|t| t.id == todo_id)
                .ok_or(StorageError::TodoNotFound(todo_id.to_string()))?;
            todo.is_complete = is_complete;
            todo.updated_at = Utc::now().to_rfc3339();
            plan.updated_at = Utc::now().to_rfc3339();
            todo.clone()
        };
        // No immediate disk write; callers should call save_async() to persist changes.
        Ok(updated)
    }

    pub fn get_todos(&self, goal_id: &str) -> Result<Vec<Todo>, StorageError> {
        let plan = self
            .data
            .plans
            .get(goal_id)
            .ok_or(StorageError::PlanNotFound(goal_id.to_string()))?;
        Ok(plan.todos.clone())
    }
}

impl Default for Storage {
    fn default() -> Self {
        Self::new()
    }
}

// Tests removed per user request

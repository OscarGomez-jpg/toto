use crate::domain::task::Task;
use std::error::Error;

use chrono::{DateTime, Utc};

/// Defines the interface for persisting task data.
/// 
/// This port is implemented by outbound adapters (like SQLite) to store
/// and retrieve domain entities.
#[cfg_attr(test, mockall::automock)]
pub trait TaskRepository: Send + Sync {
    /// Persists a new local task.
    fn add(
        &self,
        content: String,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> Result<String, Box<dyn Error>>;

    /// Retrieves all persisted tasks from the store.
    fn get_all(&self) -> Result<Vec<Task>, Box<dyn Error>>;

    /// Updates the completion status of a task.
    fn toggle_completed(&self, id: String) -> Result<(), Box<dyn Error>>;

    /// Updates the important status of a task.
    fn toggle_important(&self, id: String) -> Result<(), Box<dyn Error>>;

    /// Updates the content and metadata of an existing task.
    fn update_content(
        &self,
        id: String,
        content: String,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> Result<(), Box<dyn Error>>;

    /// Deletes a task from the store.
    fn remove(&self, id: String) -> Result<bool, Box<dyn Error>>;

    /// Removes all tasks marked as completed and returns the number of deleted items.
    fn clear_completed(&self) -> Result<usize, Box<dyn Error>>;

    /// Adjusts the relative position of a task in the list.
    fn move_task(&self, id: String, delta: i32) -> Result<(), Box<dyn Error>>;

    /// Inserts or updates a task that originated from an external source (e.g., Jira).
    fn upsert_from_external(&self, task: Task) -> Result<(), Box<dyn Error>>;
}

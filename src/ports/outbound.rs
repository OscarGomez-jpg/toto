use crate::domain::task::Task;
use std::error::Error;

pub trait TaskRepository: Send + Sync {
    fn add(&self, content: String) -> Result<String, Box<dyn Error>>;
    fn get_all(&self) -> Result<Vec<Task>, Box<dyn Error>>;
    fn toggle_completed(&self, id: String) -> Result<(), Box<dyn Error>>;
    fn toggle_important(&self, id: String) -> Result<(), Box<dyn Error>>;
    fn update_content(&self, id: String, content: String) -> Result<(), Box<dyn Error>>;
    fn remove(&self, id: String) -> Result<bool, Box<dyn Error>>;
    fn clear_completed(&self) -> Result<usize, Box<dyn Error>>;
    fn move_task(&self, id: String, delta: i32) -> Result<(), Box<dyn Error>>;
}

use crate::domain::task::{Task, TaskSource};
use crate::ports::outbound::TaskRepository;
use chrono::{DateTime, Utc};
use directories::ProjectDirs;
use log::{debug, info};
use rusqlite::{params, Connection, OptionalExtension};
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use uuid::Uuid;

pub struct SqliteRepository {
    conn: Mutex<Connection>,
}

impl SqliteRepository {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let db_path = Self::get_db_path();
        debug!("Opening database at {:?}", db_path);
        Self::migrate_files_to_data_dir(&db_path);

        let conn = Connection::open(&db_path)?;
        Self::setup_db(&conn)?;

        let repo = SqliteRepository {
            conn: Mutex::new(conn),
        };
        Ok(repo)
    }

    pub fn new_in_memory() -> Result<Self, Box<dyn Error>> {
        let conn = Connection::open_in_memory()?;
        Self::setup_db(&conn)?;
        Ok(SqliteRepository {
            conn: Mutex::new(conn),
        })
    }

    fn setup_db(conn: &Connection) -> Result<(), Box<dyn Error>> {
        // Ensure the table exists
        conn.execute(
            "CREATE TABLE IF NOT EXISTS todo (
                id TEXT PRIMARY KEY,
                external_id TEXT,
                source TEXT DEFAULT 'Local',
                title TEXT DEFAULT '',
                description TEXT DEFAULT '',
                important INTEGER DEFAULT 0,
                completed INTEGER DEFAULT 0,
                start_date TEXT,
                end_date TEXT,
                position INTEGER
            )",
            [],
        )?;

        // Ensure all required columns exist (for migrations)
        let columns: Vec<String> = conn
            .prepare("SELECT name FROM pragma_table_info('todo')")?
            .query_map([], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()?;

        if !columns.contains(&"title".to_string()) {
            info!("Adding 'title' column to todo table");
            conn.execute("ALTER TABLE todo ADD COLUMN title TEXT DEFAULT ''", [])?;
        }
        if columns.contains(&"content".to_string()) && !columns.contains(&"description".to_string())
        {
            info!("Renaming 'content' to 'description'");
            conn.execute("ALTER TABLE todo RENAME COLUMN content TO description", [])?;
        }
        if !columns.contains(&"description".to_string())
            && !columns.contains(&"content".to_string())
        {
            info!("Adding 'description' column to todo table");
            conn.execute(
                "ALTER TABLE todo ADD COLUMN description TEXT DEFAULT ''",
                [],
            )?;
        }
        if !columns.contains(&"completed".to_string()) {
            info!("Adding 'completed' column to todo table");
            conn.execute(
                "ALTER TABLE todo ADD COLUMN completed INTEGER DEFAULT 0",
                [],
            )?;
        }
        if !columns.contains(&"position".to_string()) {
            info!("Adding 'position' column to todo table");
            conn.execute("ALTER TABLE todo ADD COLUMN position INTEGER", [])?;
        }
        if !columns.contains(&"start_date".to_string()) {
            info!("Adding 'start_date' column to todo table");
            conn.execute("ALTER TABLE todo ADD COLUMN start_date TEXT", [])?;
        }
        if !columns.contains(&"end_date".to_string()) {
            info!("Adding 'end_date' column to todo table");
            conn.execute("ALTER TABLE todo ADD COLUMN end_date TEXT", [])?;
        }
        if !columns.contains(&"external_id".to_string()) {
            info!("Adding 'external_id' column to todo table");
            conn.execute("ALTER TABLE todo ADD COLUMN external_id TEXT", [])?;
        }
        if !columns.contains(&"source".to_string()) {
            info!("Adding 'source' column to todo table");
            conn.execute(
                "ALTER TABLE todo ADD COLUMN source TEXT DEFAULT 'Local'",
                [],
            )?;
        }
        if !columns.contains(&"features".to_string()) {
            info!("Adding 'features' column to todo table");
            conn.execute("ALTER TABLE todo ADD COLUMN features TEXT", [])?;
        }

        Ok(())
    }

    fn get_db_path() -> PathBuf {
        if let Some(proj_dirs) = ProjectDirs::from("", "", "toto") {
            let data_dir = proj_dirs.data_dir();
            let _ = fs::create_dir_all(data_dir);
            data_dir.join("todo.db")
        } else {
            PathBuf::from("todo.db")
        }
    }

    fn migrate_files_to_data_dir(target_db_path: &Path) {
        let local_db = Path::new("todo.db");
        if local_db.exists() && local_db != target_db_path {
            info!(
                "Migrating database file from {:?} to {:?}",
                local_db, target_db_path
            );
            let _ = fs::rename(local_db, target_db_path);
        }
    }

    fn get_task_features(
        &self,
        id: &str,
    ) -> Result<serde_json::Map<String, serde_json::Value>, Box<dyn Error>> {
        let conn = self.conn.lock().unwrap();
        let features_json: Option<String> = conn
            .query_row(
                "SELECT features FROM todo WHERE id = ?",
                params![id],
                |row| row.get(0),
            )
            .optional()?
            .flatten();

        match features_json {
            Some(json_str) => {
                let map = serde_json::from_str(&json_str)?;
                Ok(map)
            }
            None => Ok(serde_json::Map::new()),
        }
    }

    fn save_task_features(
        &self,
        id: &str,
        features: serde_json::Map<String, serde_json::Value>,
    ) -> Result<(), Box<dyn Error>> {
        let conn = self.conn.lock().unwrap();
        let json_str = serde_json::to_string(&features)?;
        conn.execute(
            "UPDATE todo SET features = ? WHERE id = ?",
            params![json_str, id],
        )?;
        Ok(())
    }
}

impl TaskRepository for SqliteRepository {
    fn add(
        &self,
        title: String,
        description: String,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> Result<String, Box<dyn Error>> {
        debug!("Adding local task: {} - {}", title, description);
        let conn = self.conn.lock().unwrap();
        let id = Uuid::new_v4().to_string();
        let max_pos: i64 =
            conn.query_row("SELECT IFNULL(MAX(position), 0) FROM todo", [], |row| {
                row.get(0)
            })?;

        let features = serde_json::Map::new();
        let features_json = serde_json::to_string(&features)?;

        conn.execute(
            "INSERT INTO todo (id, title, description, position, start_date, end_date, source, features) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                id,
                title,
                description,
                max_pos + 1,
                start_date.map(|d| d.to_rfc3339()),
                end_date.map(|d| d.to_rfc3339()),
                "Local",
                features_json
            ],
        )?;
        Ok(id)
    }

    fn get_all(&self) -> Result<Vec<Task>, Box<dyn Error>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, title, description, important, completed, start_date, end_date, external_id, source, features FROM todo ORDER BY completed ASC, position DESC, id DESC")?;
        let todo_iter = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let title: String = row.get(1)?;
            let description: String = row.get(2)?;
            let start_date_str: Option<String> = row.get(5)?;
            let end_date_str: Option<String> = row.get(6)?;
            let external_id: Option<String> = row.get(7)?;
            let source_str: String = row.get(8)?;
            let features_json: Option<String> = row.get(9)?;

            let start_date = start_date_str.and_then(|s| {
                DateTime::parse_from_rfc3339(&s)
                    .ok()
                    .map(|d| d.with_timezone(&Utc))
            });
            let end_date = end_date_str.and_then(|s| {
                DateTime::parse_from_rfc3339(&s)
                    .ok()
                    .map(|d| d.with_timezone(&Utc))
            });

            let source = match source_str.as_str() {
                "Jira" => TaskSource::Jira,
                _ => TaskSource::Local,
            };

            let mut builder = crate::domain::task::TaskBuilder::new(id)
                .with_metadata(title, description)
                .with_status(row.get::<_, i32>(4)? != 0, row.get::<_, i32>(3)? != 0)
                .with_schedule(start_date, end_date)
                .with_external(external_id, source);

            if let Some(json_str) = features_json {
                if let Ok(features_map) =
                    serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(&json_str)
                {
                    if let Some(tags_val) = features_map.get("tags") {
                        if let Ok(tags) = serde_json::from_value::<Vec<String>>(tags_val.clone()) {
                            builder = builder.with_tags(tags);
                        }
                    }
                    if let Some(relations_val) = features_map.get("relations") {
                        if let Ok(relations) = serde_json::from_value::<
                            Vec<crate::domain::task::TaskRelation>,
                        >(relations_val.clone())
                        {
                            builder = builder.with_relations(relations);
                        }
                    }
                }
            }

            Ok(builder.build())
        })?;

        let mut tasks = Vec::new();
        for task in todo_iter {
            tasks.push(task?);
        }
        Ok(tasks)
    }

    fn toggle_completed(&self, id: String) -> Result<(), Box<dyn Error>> {
        debug!("Toggling completed for task: {}", id);
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE todo SET completed = 1 - completed WHERE id = ?",
            params![id],
        )?;
        Ok(())
    }

    fn toggle_important(&self, id: String) -> Result<(), Box<dyn Error>> {
        debug!("Toggling important for task: {}", id);
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE todo SET important = 1 - important WHERE id = ?",
            params![id],
        )?;
        Ok(())
    }

    fn update_task(
        &self,
        id: String,
        title: String,
        description: String,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> Result<(), Box<dyn Error>> {
        debug!("Updating task: {}", id);
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE todo SET title = ?, description = ?, start_date = ?, end_date = ? WHERE id = ?",
            params![
                title,
                description,
                start_date.map(|d| d.to_rfc3339()),
                end_date.map(|d| d.to_rfc3339()),
                id
            ],
        )?;
        Ok(())
    }

    fn remove(&self, id: String) -> Result<bool, Box<dyn Error>> {
        info!("Removing task: {}", id);
        let conn = self.conn.lock().unwrap();
        let rows = conn.execute("DELETE FROM todo WHERE id = ?", params![id])?;
        Ok(rows > 0)
    }

    fn clear_completed(&self) -> Result<usize, Box<dyn Error>> {
        info!("Clearing all completed tasks");
        let conn = self.conn.lock().unwrap();
        let rows = conn.execute("DELETE FROM todo WHERE completed = 1", [])?;
        Ok(rows)
    }

    fn move_task(&self, id: String, delta: i32) -> Result<(), Box<dyn Error>> {
        let conn = self.conn.lock().unwrap();
        let current_pos: i64 = conn.query_row(
            "SELECT position FROM todo WHERE id = ?",
            params![id],
            |row| row.get(0),
        )?;

        let target_id: Option<String> = if delta > 0 {
            conn.query_row(
                "SELECT id FROM todo WHERE position > ? ORDER BY position ASC LIMIT 1",
                params![current_pos],
                |row| row.get(0),
            )
            .ok()
        } else {
            conn.query_row(
                "SELECT id FROM todo WHERE position < ? ORDER BY position DESC LIMIT 1",
                params![current_pos],
                |row| row.get(0),
            )
            .ok()
        };

        if let Some(tid) = target_id {
            let target_pos: i64 = conn.query_row(
                "SELECT position FROM todo WHERE id = ?",
                params![tid],
                |row| row.get(0),
            )?;
            conn.execute(
                "UPDATE todo SET position = ? WHERE id = ?",
                params![target_pos, id],
            )?;
            conn.execute(
                "UPDATE todo SET position = ? WHERE id = ?",
                params![current_pos, tid],
            )?;
        }

        Ok(())
    }

    fn add_tag(&self, id: String, tag: String) -> Result<(), Box<dyn Error>> {
        let mut features = self.get_task_features(&id)?;
        let tags_val = features
            .entry("tags".to_string())
            .or_insert(serde_json::json!([]));
        if let Some(tags_array) = tags_val.as_array_mut() {
            let tag_json = serde_json::json!(tag);
            if !tags_array.contains(&tag_json) {
                tags_array.push(tag_json);
            }
        }
        self.save_task_features(&id, features)
    }

    fn remove_tag(&self, id: String, tag: String) -> Result<(), Box<dyn Error>> {
        let mut features = self.get_task_features(&id)?;
        if let Some(tags_val) = features.get_mut("tags") {
            if let Some(tags_array) = tags_val.as_array_mut() {
                tags_array.retain(|t| t.as_str() != Some(&tag));
            }
        }
        self.save_task_features(&id, features)
    }

    fn add_relation(
        &self,
        id: String,
        relation: crate::domain::task::TaskRelation,
    ) -> Result<(), Box<dyn Error>> {
        let mut features = self.get_task_features(&id)?;
        let relations_val = features
            .entry("relations".to_string())
            .or_insert(serde_json::json!([]));
        if let Some(relations_array) = relations_val.as_array_mut() {
            let rel_json = serde_json::to_value(&relation)?;
            // Check if already exists
            let exists = relations_array.iter().any(|r| {
                r.get("target_id") == rel_json.get("target_id")
                    && r.get("relation_type") == rel_json.get("relation_type")
            });
            if !exists {
                relations_array.push(rel_json);
            }
        }
        self.save_task_features(&id, features)
    }

    fn remove_relation(&self, id: String, target_id: String) -> Result<(), Box<dyn Error>> {
        let mut features = self.get_task_features(&id)?;
        if let Some(relations_val) = features.get_mut("relations") {
            if let Some(relations_array) = relations_val.as_array_mut() {
                relations_array
                    .retain(|r| r.get("target_id").and_then(|v| v.as_str()) != Some(&target_id));
            }
        }
        self.save_task_features(&id, features)
    }

    fn upsert_from_external(&self, task: Task) -> Result<(), Box<dyn Error>> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;

        let source_str = match task.source() {
            TaskSource::Jira => "Jira",
            TaskSource::Local => "Local",
        };

        // 1. Try to find existing task by external_id AND source
        let existing_data: Option<(String, Option<String>)> = tx
            .query_row(
                "SELECT id, features FROM todo WHERE external_id = ? AND source = ?",
                params![task.external_id(), source_str],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .ok();

        if let Some((id, existing_features_json)) = existing_data {
            // 2. Update existing - Merge features
            let features_map: serde_json::Map<String, serde_json::Value> = existing_features_json
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default();

            // For now, we just overwrite title/desc/dates, but we keep other features like tags/relations
            // In the future, we might want a more sophisticated merge

            debug!(
                "Updating existing external task: {} (ID: {}, Source: {})",
                task.title(),
                id,
                source_str
            );
            tx.execute(
                "UPDATE todo SET title = ?, description = ?, completed = ?, start_date = ?, end_date = ?, features = ? WHERE id = ?",
                params![
                    task.title(),
                    task.description(),
                    if task.is_completed() { 1 } else { 0 },
                    task.start_date().map(|d| d.to_rfc3339()),
                    task.end_date().map(|d| d.to_rfc3339()),
                    serde_json::to_string(&features_map)?,
                    id
                ],
            )?;
        } else {
            // 3. Insert new
            debug!(
                "Inserting new external task: {} (Source: {})",
                task.title(),
                source_str
            );
            let id = Uuid::new_v4().to_string();
            let max_pos: i64 =
                tx.query_row("SELECT IFNULL(MAX(position), 0) FROM todo", [], |row| {
                    row.get(0)
                })?;

            // Serialize features if any
            let features_map = serde_json::Map::new(); // External tasks start with no extra features for now

            tx.execute(
                "INSERT INTO todo (id, external_id, source, title, description, important, completed, start_date, end_date, position, features) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                params![
                    id,
                    task.external_id(),
                    source_str,
                    task.title(),
                    task.description(),
                    if task.is_important() { 1 } else { 0 },
                    if task.is_completed() { 1 } else { 0 },
                    task.start_date().map(|d| d.to_rfc3339()),
                    task.end_date().map(|d| d.to_rfc3339()),
                    max_pos + 1,
                    serde_json::to_string(&features_map)?
                ],
            )?;
        }

        tx.commit()?;
        Ok(())
    }
}

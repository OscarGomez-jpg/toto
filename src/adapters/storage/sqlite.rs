use crate::domain::task::Task;
use crate::ports::outbound::TaskRepository;
use chrono::{DateTime, Utc};
use directories::ProjectDirs;
use rusqlite::{params, Connection};
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
        Self::migrate_files_to_data_dir(&db_path);

        let conn = Connection::open(&db_path)?;

        // 1. Check if we need to migrate id from INTEGER to TEXT
        let id_type: String = conn
            .query_row(
                "SELECT type FROM pragma_table_info('todo') WHERE name='id'",
                [],
                |row| row.get(0),
            )
            .unwrap_or_else(|_| "INTEGER".to_string());

        if id_type == "INTEGER" {
            // Migration from very old numeric ID schema
            conn.execute_batch(
                "BEGIN TRANSACTION;
                 CREATE TABLE todo_new (
                    id TEXT PRIMARY KEY,
                    content TEXT NOT NULL,
                    important INTEGER DEFAULT 0,
                    completed INTEGER DEFAULT 0,
                    start_date TEXT,
                    end_date TEXT,
                    position INTEGER
                 );
                 INSERT INTO todo_new (id, content, important, completed, position)
                 SELECT CAST(id AS TEXT), content, important, completed, id FROM todo;
                 DROP TABLE todo;
                 ALTER TABLE todo_new RENAME TO todo;
                 COMMIT;",
            )?;
        }

        // 2. Ensure all required columns exist
        let columns: Vec<String> = conn
            .prepare("SELECT name FROM pragma_table_info('todo')")?
            .query_map([], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()?;

        if !columns.contains(&"completed".to_string()) {
            conn.execute(
                "ALTER TABLE todo ADD COLUMN completed INTEGER DEFAULT 0",
                [],
            )?;
        }
        if !columns.contains(&"position".to_string()) {
            conn.execute("ALTER TABLE todo ADD COLUMN position INTEGER", [])?;
        }
        if !columns.contains(&"start_date".to_string()) {
            conn.execute("ALTER TABLE todo ADD COLUMN start_date TEXT", [])?;
        }
        if !columns.contains(&"end_date".to_string()) {
            // If we have an old due_date column, migrate its data to end_date
            conn.execute("ALTER TABLE todo ADD COLUMN end_date TEXT", [])?;
            if columns.contains(&"due_date".to_string()) {
                conn.execute(
                    "UPDATE todo SET end_date = due_date WHERE end_date IS NULL",
                    [],
                )?;
            }
        }

        conn.execute(
            "CREATE TABLE IF NOT EXISTS todo (
                id TEXT PRIMARY KEY,
                content TEXT NOT NULL,
                important INTEGER DEFAULT 0,
                completed INTEGER DEFAULT 0,
                start_date TEXT,
                end_date TEXT,
                position INTEGER
            )",
            [],
        )?;

        let repo = SqliteRepository {
            conn: Mutex::new(conn),
        };
        Ok(repo)
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
            let _ = fs::rename(local_db, target_db_path);
        }
    }
}

impl TaskRepository for SqliteRepository {
    fn add(
        &self,
        content: String,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> Result<String, Box<dyn Error>> {
        let conn = self.conn.lock().unwrap();
        let id = Uuid::new_v4().to_string();
        let max_pos: i64 =
            conn.query_row("SELECT IFNULL(MAX(position), 0) FROM todo", [], |row| {
                row.get(0)
            })?;
        conn.execute(
            "INSERT INTO todo (id, content, position, start_date, end_date) VALUES (?, ?, ?, ?, ?)",
            params![
                id,
                content,
                max_pos + 1,
                start_date.map(|d| d.to_rfc3339()),
                end_date.map(|d| d.to_rfc3339())
            ],
        )?;
        Ok(id)
    }

    fn get_all(&self) -> Result<Vec<Task>, Box<dyn Error>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, content, important, completed, start_date, end_date FROM todo ORDER BY completed ASC, position DESC, id DESC")?;
        let todo_iter = stmt.query_map([], |row| {
            let start_date_str: Option<String> = row.get(4)?;
            let end_date_str: Option<String> = row.get(5)?;

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

            let mut task = Task::new(row.get(0)?, row.get(1)?);
            task.important = row.get::<_, i32>(2)? != 0;
            task.completed = row.get::<_, i32>(3)? != 0;
            task.start_date = start_date;
            task.end_date = end_date;

            Ok(task)
        })?;

        let mut tasks = Vec::new();
        for task in todo_iter {
            tasks.push(task?);
        }
        Ok(tasks)
    }

    fn toggle_completed(&self, id: String) -> Result<(), Box<dyn Error>> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE todo SET completed = 1 - completed WHERE id = ?",
            params![id],
        )?;
        Ok(())
    }

    fn toggle_important(&self, id: String) -> Result<(), Box<dyn Error>> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE todo SET important = 1 - important WHERE id = ?",
            params![id],
        )?;
        Ok(())
    }

    fn update_content(
        &self,
        id: String,
        content: String,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> Result<(), Box<dyn Error>> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE todo SET content = ?, start_date = ?, end_date = ? WHERE id = ?",
            params![
                content,
                start_date.map(|d| d.to_rfc3339()),
                end_date.map(|d| d.to_rfc3339()),
                id
            ],
        )?;
        Ok(())
    }

    fn remove(&self, id: String) -> Result<bool, Box<dyn Error>> {
        let conn = self.conn.lock().unwrap();
        let rows = conn.execute("DELETE FROM todo WHERE id = ?", params![id])?;
        Ok(rows > 0)
    }

    fn clear_completed(&self) -> Result<usize, Box<dyn Error>> {
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

        // Find the task to swap with
        let target_id: Option<String> = if delta > 0 {
            // Move up (increase position)
            conn.query_row(
                "SELECT id FROM todo WHERE position > ? ORDER BY position ASC LIMIT 1",
                params![current_pos],
                |row| row.get(0),
            )
            .ok()
        } else {
            // Move down (decrease position)
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
}

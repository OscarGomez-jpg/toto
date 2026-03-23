use rusqlite::{params, Connection};
use std::path::{Path, PathBuf};
use directories::ProjectDirs;
use std::fs;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct Line {
    pub id: i64,
    pub content: String,
    pub important: bool,
    pub completed: bool,
    pub due_date: Option<DateTime<Utc>>,
}

pub struct TodoList {
    conn: Connection,
    dirty: bool,
}

impl TodoList {
    fn get_db_path() -> PathBuf {
        if let Some(proj_dirs) = ProjectDirs::from("", "", "toto") {
            let data_dir = proj_dirs.data_dir();
            fs::create_dir_all(data_dir).expect("Failed to create data directory");
            data_dir.join("todo.db")
        } else {
            PathBuf::from("todo.db")
        }
    }

    pub fn load() -> Self {
        let db_path = Self::get_db_path();
        Self::migrate_files_to_data_dir(&db_path);

        let conn = Connection::open(&db_path).expect("Failed to open database");

        conn.execute(
            "CREATE TABLE IF NOT EXISTS todo (
                id INTEGER PRIMARY KEY,
                content TEXT NOT NULL,
                important INTEGER DEFAULT 0,
                completed INTEGER DEFAULT 0,
                due_date TEXT
            )",
            [],
        )
        .expect("Failed to create table");

        // Simple migration check for existing tables (if columns are missing)
        let _ = conn.execute("ALTER TABLE todo ADD COLUMN completed INTEGER DEFAULT 0", []);
        let _ = conn.execute("ALTER TABLE todo ADD COLUMN due_date TEXT", []);

        let mut list = TodoList { conn, dirty: false };
        list.migrate_from_json(&db_path);
        list
    }

    fn migrate_files_to_data_dir(target_db_path: &Path) {
        let local_db = Path::new("todo.db");
        if local_db.exists() && local_db != target_db_path {
            let _ = fs::rename(local_db, target_db_path);
        }
        
        let local_json = Path::new("todo.json");
        if local_json.exists() {
            let target_json = target_db_path.with_extension("json");
            let _ = fs::rename(local_json, target_json);
        }
    }

    fn migrate_from_json(&mut self, db_path: &Path) {
        let json_path = db_path.with_extension("json");
        if json_path.exists() {
            if let Ok(contents) = fs::read_to_string(&json_path) {
                #[derive(serde::Deserialize)]
                struct OldLine { content: String }
                #[derive(serde::Deserialize)]
                struct OldList { list: Vec<OldLine> }

                if let Ok(old_data) = serde_json::from_str::<OldList>(&contents) {
                    for line in old_data.list {
                        let _ = self.add_line(line.content);
                    }
                    let _ = fs::remove_file(json_path);
                }
            }
        }
    }

    pub fn add_line(&mut self, content: String) -> i64 {
        self.conn.execute(
            "INSERT INTO todo (content) VALUES (?)",
            params![content],
        ).expect("Failed to insert todo");
        self.dirty = true;
        self.conn.last_insert_rowid()
    }

    pub fn get_all(&self) -> Vec<Line> {
        let mut stmt = self.conn.prepare("SELECT id, content, important, completed, due_date FROM todo ORDER BY completed ASC, important DESC, id DESC").expect("Failed to prepare statement");
        let todo_iter = stmt.query_map([], |row| {
            let due_date_str: Option<String> = row.get(4)?;
            let due_date = due_date_str.and_then(|s| s.parse::<DateTime<Utc>>().ok());

            Ok(Line {
                id: row.get(0)?,
                content: row.get(1)?,
                important: row.get::<_, i32>(2)? != 0,
                completed: row.get::<_, i32>(3)? != 0,
                due_date,
            })
        }).expect("Failed to query todos");

        todo_iter.map(|item| item.unwrap()).collect()
    }

    pub fn toggle_completed(&mut self, id: i64) {
        self.conn.execute(
            "UPDATE todo SET completed = 1 - completed WHERE id = ?",
            params![id],
        ).expect("Failed to toggle completed");
        self.dirty = true;
    }

    pub fn toggle_important(&mut self, id: i64) {
        self.conn.execute(
            "UPDATE todo SET important = 1 - important WHERE id = ?",
            params![id],
        ).expect("Failed to toggle important");
        self.dirty = true;
    }

    pub fn update_content(&mut self, id: i64, content: String) {
        self.conn.execute(
            "UPDATE todo SET content = ? WHERE id = ?",
            params![content, id],
        ).expect("Failed to update content");
        self.dirty = true;
    }

    pub fn remove(&mut self, id: i64) -> String {
        let rows = self.conn.execute("DELETE FROM todo WHERE id = ?", params![id]).expect("Failed to delete todo");
        if rows > 0 {
            self.dirty = true;
            format!("Task {} removed successfully", id)
        } else {
            format!("Task {} not found", id)
        }
    }

    pub fn dirty(&self) -> bool {
        self.dirty
    }
}

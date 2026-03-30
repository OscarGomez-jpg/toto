use crate::adapters::tui::app::{Action, CurrentScreen};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyConfig {
    pub key: KeyCode,
    pub modifiers: KeyModifiers,
}

impl From<(KeyCode, KeyModifiers)> for KeyConfig {
    fn from((key, modifiers): (KeyCode, KeyModifiers)) -> Self {
        Self { key, modifiers }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraConfig {
    pub enabled: bool,
    pub domain: String,
    pub email: String,
    pub api_token: String,
    pub projects: Vec<String>,
}

impl Default for JiraConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            domain: "lozanop513.atlassian.net".to_string(),
            email: "".to_string(),
            api_token: "".to_string(),
            projects: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub keybindings: HashMap<CurrentScreen, Vec<(KeyConfig, Action)>>,
    #[serde(default)]
    pub jira: JiraConfig,
}

impl Default for Config {
    fn default() -> Self {
        let mut keybindings = HashMap::new();

        // Main Screen
        let main_keys = vec![
            (KeyConfig::from((KeyCode::Char('q'), KeyModifiers::empty())), Action::Quit),
            (KeyConfig::from((KeyCode::Char('c'), KeyModifiers::CONTROL)), Action::Quit),
            (KeyConfig::from((KeyCode::Char('a'), KeyModifiers::empty())), Action::Add),
            (KeyConfig::from((KeyCode::Char('e'), KeyModifiers::empty())), Action::Edit),
            (KeyConfig::from((KeyCode::Char('v'), KeyModifiers::empty())), Action::ToggleGantt),
            (KeyConfig::from((KeyCode::Char('x'), KeyModifiers::empty())), Action::Delete),
            (KeyConfig::from((KeyCode::Char('d'), KeyModifiers::empty())), Action::Delete),
            (KeyConfig::from((KeyCode::Char('d'), KeyModifiers::SHIFT)), Action::Delete),
            (KeyConfig::from((KeyCode::Char('d'), KeyModifiers::CONTROL)), Action::PageDown),
            (KeyConfig::from((KeyCode::Char('u'), KeyModifiers::CONTROL)), Action::PageUp),
            (KeyConfig::from((KeyCode::Char('l'), KeyModifiers::CONTROL)), Action::ClearCompleted),
            (KeyConfig::from((KeyCode::Char('/'), KeyModifiers::empty())), Action::Search),
            (KeyConfig::from((KeyCode::Char('c'), KeyModifiers::empty())), Action::ToggleCompleted),
            (KeyConfig::from((KeyCode::Enter, KeyModifiers::empty())), Action::ToggleCompleted),
            (KeyConfig::from((KeyCode::Char('i'), KeyModifiers::empty())), Action::ToggleImportant),
            (KeyConfig::from((KeyCode::Down, KeyModifiers::empty())), Action::MoveDown),
            (KeyConfig::from((KeyCode::Char('j'), KeyModifiers::empty())), Action::MoveDown),
            (KeyConfig::from((KeyCode::Up, KeyModifiers::empty())), Action::MoveUp),
            (KeyConfig::from((KeyCode::Char('k'), KeyModifiers::empty())), Action::MoveUp),
            (KeyConfig::from((KeyCode::Down, KeyModifiers::SHIFT)), Action::MoveTaskDown),
            (KeyConfig::from((KeyCode::Char('j'), KeyModifiers::SHIFT)), Action::MoveTaskDown),
            (KeyConfig::from((KeyCode::Up, KeyModifiers::SHIFT)), Action::MoveTaskUp),
            (KeyConfig::from((KeyCode::Char('k'), KeyModifiers::SHIFT)), Action::MoveTaskUp),
            (KeyConfig::from((KeyCode::Char('g'), KeyModifiers::empty())), Action::MoveToTop),
            (KeyConfig::from((KeyCode::Char('G'), KeyModifiers::SHIFT)), Action::MoveToBottom),
            (KeyConfig::from((KeyCode::Home, KeyModifiers::empty())), Action::MoveToTop),
            (KeyConfig::from((KeyCode::End, KeyModifiers::empty())), Action::MoveToBottom),
            (KeyConfig::from((KeyCode::PageUp, KeyModifiers::empty())), Action::PageUp),
            (KeyConfig::from((KeyCode::PageDown, KeyModifiers::empty())), Action::PageDown),
            (KeyConfig::from((KeyCode::Esc, KeyModifiers::empty())), Action::Esc),
            (KeyConfig::from((KeyCode::Char('s'), KeyModifiers::CONTROL)), Action::SyncJira),
        ];
        keybindings.insert(CurrentScreen::Main, main_keys);

        // Gantt Screen
        let gantt_keys = vec![
            (KeyConfig::from((KeyCode::Char('q'), KeyModifiers::empty())), Action::ToggleGantt),
            (KeyConfig::from((KeyCode::Char('v'), KeyModifiers::empty())), Action::ToggleGantt),
            (KeyConfig::from((KeyCode::Esc, KeyModifiers::empty())), Action::ToggleGantt),
            (KeyConfig::from((KeyCode::Down, KeyModifiers::empty())), Action::MoveDown),
            (KeyConfig::from((KeyCode::Char('j'), KeyModifiers::empty())), Action::MoveDown),
            (KeyConfig::from((KeyCode::Up, KeyModifiers::empty())), Action::MoveUp),
            (KeyConfig::from((KeyCode::Char('k'), KeyModifiers::empty())), Action::MoveUp),
        ];
        keybindings.insert(CurrentScreen::Gantt, gantt_keys);

        // Adding/Editing Screen
        let input_keys = vec![
            (KeyConfig::from((KeyCode::Tab, KeyModifiers::empty())), Action::Tab),
            (KeyConfig::from((KeyCode::BackTab, KeyModifiers::SHIFT)), Action::BackTab),
            (KeyConfig::from((KeyCode::Enter, KeyModifiers::empty())), Action::Enter),
            (KeyConfig::from((KeyCode::Esc, KeyModifiers::empty())), Action::Esc),
            (KeyConfig::from((KeyCode::Char('c'), KeyModifiers::CONTROL)), Action::Esc),
            (KeyConfig::from((KeyCode::Left, KeyModifiers::empty())), Action::MoveDateLeft),
            (KeyConfig::from((KeyCode::Right, KeyModifiers::empty())), Action::MoveDateRight),
            (KeyConfig::from((KeyCode::Up, KeyModifiers::empty())), Action::MoveDateUp),
            (KeyConfig::from((KeyCode::Down, KeyModifiers::empty())), Action::MoveDateDown),
            (KeyConfig::from((KeyCode::Char(' '), KeyModifiers::empty())), Action::SelectDate),
        ];
        keybindings.insert(CurrentScreen::Adding, input_keys.clone());
        keybindings.insert(CurrentScreen::Editing, input_keys);

        // Searching Screen
        let search_keys = vec![
            (KeyConfig::from((KeyCode::Enter, KeyModifiers::empty())), Action::Esc),
            (KeyConfig::from((KeyCode::Esc, KeyModifiers::empty())), Action::Esc),
        ];
        keybindings.insert(CurrentScreen::Searching, search_keys);

        // ConfirmingDelete Screen
        let confirm_keys = vec![
            (KeyConfig::from((KeyCode::Char('y'), KeyModifiers::empty())), Action::ConfirmDelete),
            (KeyConfig::from((KeyCode::Enter, KeyModifiers::empty())), Action::ConfirmDelete),
            (KeyConfig::from((KeyCode::Char('n'), KeyModifiers::empty())), Action::Esc),
            (KeyConfig::from((KeyCode::Esc, KeyModifiers::empty())), Action::Esc),
        ];
        keybindings.insert(CurrentScreen::ConfirmingDelete, confirm_keys);

        Config { 
            keybindings,
            jira: JiraConfig::default(),
        }
    }
}

impl Config {
    pub fn load() -> Self {
        if let Some(config_path) = get_config_path() {
            if let Ok(content) = fs::read_to_string(&config_path) {
                if let Ok(config) = toml::from_str(&content) {
                    return config;
                }
            } else {
                // Try to save default config if it doesn't exist
                let _ = save_default_config(&config_path);
            }
        }
        Config::default()
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(config_path) = get_config_path() {
            let content = toml::to_string_pretty(&self)?;
            fs::write(config_path, content)?;
        }
        Ok(())
    }

    pub fn get_action(&self, screen: &CurrentScreen, event: &KeyEvent) -> Option<Action> {
        if let Some(bindings) = self.keybindings.get(screen) {
            for (key_config, action) in bindings {
                if key_config.key == event.code && key_config.modifiers == event.modifiers {
                    let action: &Action = action;
                    return Some(action.clone());
                }
            }
        }
        None
    }
}

fn get_config_path() -> Option<PathBuf> {
    ProjectDirs::from("", "", "toto").map(|proj_dirs| {
        let config_dir = proj_dirs.config_dir();
        if !config_dir.exists() {
            let _ = fs::create_dir_all(config_dir);
        }
        config_dir.join("config.toml")
    })
}

fn save_default_config(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::default();
    let content = toml::to_string_pretty(&config)?;
    fs::write(path, content)?;
    Ok(())
}

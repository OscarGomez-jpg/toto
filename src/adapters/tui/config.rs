use crate::adapters::tui::app::{Action, CurrentScreen};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Represents a specific key and its modifiers.
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

/// Configuration settings for Jira integration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraConfig {
    /// Whether Jira synchronization is enabled.
    pub enabled: bool,
    /// Your Jira domain (e.g., your-name.atlassian.net).
    pub domain: String,
    /// The email associated with your Jira account.
    pub email: String,
    /// Your Atlassian API token.
    pub api_token: String,
    /// List of project keys to synchronize (e.g., ["PROJ", "TASK"]).
    pub projects: Vec<String>,
    /// List of labels to filter issues by.
    pub labels: Vec<String>,
}

impl Default for JiraConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            domain: "lozanop513.atlassian.net".to_string(),
            email: "".to_string(),
            api_token: "".to_string(),
            projects: vec![],
            labels: vec![],
        }
    }
}

/// The root configuration structure for the application.
///
/// This is typically loaded from and saved to a `config.toml` file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// A map of screen states to their specific key-to-action bindings.
    pub keybindings: HashMap<CurrentScreen, Vec<(KeyConfig, Action)>>,
    /// Jira connection settings.
    #[serde(default)]
    pub jira: JiraConfig,
}

impl Default for Config {
    fn default() -> Self {
        let mut keybindings = HashMap::new();

        // Main Screen
        let main_keys = vec![
            (
                KeyConfig::from((KeyCode::Char('q'), KeyModifiers::empty())),
                Action::Quit,
            ),
            (
                KeyConfig::from((KeyCode::Char('c'), KeyModifiers::CONTROL)),
                Action::Quit,
            ),
            (
                KeyConfig::from((KeyCode::Char('a'), KeyModifiers::empty())),
                Action::Add,
            ),
            (
                KeyConfig::from((KeyCode::Char('e'), KeyModifiers::empty())),
                Action::Edit,
            ),
            (
                KeyConfig::from((KeyCode::Char('v'), KeyModifiers::empty())),
                Action::ToggleGantt,
            ),
            (
                KeyConfig::from((KeyCode::Char('x'), KeyModifiers::empty())),
                Action::Delete,
            ),
            (
                KeyConfig::from((KeyCode::Char('d'), KeyModifiers::empty())),
                Action::Delete,
            ),
            (
                KeyConfig::from((KeyCode::Char('d'), KeyModifiers::SHIFT)),
                Action::Delete,
            ),
            (
                KeyConfig::from((KeyCode::Char('d'), KeyModifiers::CONTROL)),
                Action::PageDown,
            ),
            (
                KeyConfig::from((KeyCode::Char('u'), KeyModifiers::CONTROL)),
                Action::PageUp,
            ),
            (
                KeyConfig::from((KeyCode::Char('l'), KeyModifiers::CONTROL)),
                Action::ClearCompleted,
            ),
            (
                KeyConfig::from((KeyCode::Char('/'), KeyModifiers::empty())),
                Action::Search,
            ),
            (
                KeyConfig::from((KeyCode::Char('c'), KeyModifiers::empty())),
                Action::ToggleCompleted,
            ),
            (
                KeyConfig::from((KeyCode::Enter, KeyModifiers::empty())),
                Action::ToggleCompleted,
            ),
            (
                KeyConfig::from((KeyCode::Char('i'), KeyModifiers::empty())),
                Action::ToggleImportant,
            ),
            (
                KeyConfig::from((KeyCode::Down, KeyModifiers::empty())),
                Action::MoveDown,
            ),
            (
                KeyConfig::from((KeyCode::Char('j'), KeyModifiers::empty())),
                Action::MoveDown,
            ),
            (
                KeyConfig::from((KeyCode::Up, KeyModifiers::empty())),
                Action::MoveUp,
            ),
            (
                KeyConfig::from((KeyCode::Char('k'), KeyModifiers::empty())),
                Action::MoveUp,
            ),
            (
                KeyConfig::from((KeyCode::Down, KeyModifiers::SHIFT)),
                Action::MoveTaskDown,
            ),
            (
                KeyConfig::from((KeyCode::Char('j'), KeyModifiers::SHIFT)),
                Action::MoveTaskDown,
            ),
            (
                KeyConfig::from((KeyCode::Up, KeyModifiers::SHIFT)),
                Action::MoveTaskUp,
            ),
            (
                KeyConfig::from((KeyCode::Char('k'), KeyModifiers::SHIFT)),
                Action::MoveTaskUp,
            ),
            (
                KeyConfig::from((KeyCode::Char('g'), KeyModifiers::empty())),
                Action::MoveToTop,
            ),
            (
                KeyConfig::from((KeyCode::Char('G'), KeyModifiers::SHIFT)),
                Action::MoveToBottom,
            ),
            (
                KeyConfig::from((KeyCode::Home, KeyModifiers::empty())),
                Action::MoveToTop,
            ),
            (
                KeyConfig::from((KeyCode::End, KeyModifiers::empty())),
                Action::MoveToBottom,
            ),
            (
                KeyConfig::from((KeyCode::PageUp, KeyModifiers::empty())),
                Action::PageUp,
            ),
            (
                KeyConfig::from((KeyCode::PageDown, KeyModifiers::empty())),
                Action::PageDown,
            ),
            (
                KeyConfig::from((KeyCode::Esc, KeyModifiers::empty())),
                Action::Esc,
            ),
            (
                KeyConfig::from((KeyCode::Char('s'), KeyModifiers::CONTROL)),
                Action::SyncJira,
            ),
            (
                KeyConfig::from((KeyCode::Char('h'), KeyModifiers::empty())),
                Action::ToggleHelp,
            ),
        ];
        keybindings.insert(CurrentScreen::Main, main_keys);

        // Gantt Screen
        let gantt_keys = vec![
            (
                KeyConfig::from((KeyCode::Char('q'), KeyModifiers::empty())),
                Action::ToggleGantt,
            ),
            (
                KeyConfig::from((KeyCode::Char('v'), KeyModifiers::empty())),
                Action::ToggleGantt,
            ),
            (
                KeyConfig::from((KeyCode::Esc, KeyModifiers::empty())),
                Action::ToggleGantt,
            ),
            (
                KeyConfig::from((KeyCode::Down, KeyModifiers::empty())),
                Action::MoveDown,
            ),
            (
                KeyConfig::from((KeyCode::Char('j'), KeyModifiers::empty())),
                Action::MoveDown,
            ),
            (
                KeyConfig::from((KeyCode::Up, KeyModifiers::empty())),
                Action::MoveUp,
            ),
            (
                KeyConfig::from((KeyCode::Char('k'), KeyModifiers::empty())),
                Action::MoveUp,
            ),
            (
                KeyConfig::from((KeyCode::Char('h'), KeyModifiers::empty())),
                Action::ToggleHelp,
            ),
        ];
        keybindings.insert(CurrentScreen::Gantt, gantt_keys);

        // Adding/Editing Screen
        let input_keys = vec![
            (
                KeyConfig::from((KeyCode::Tab, KeyModifiers::empty())),
                Action::Tab,
            ),
            (
                KeyConfig::from((KeyCode::BackTab, KeyModifiers::SHIFT)),
                Action::BackTab,
            ),
            (
                KeyConfig::from((KeyCode::Enter, KeyModifiers::empty())),
                Action::Enter,
            ),
            (
                KeyConfig::from((KeyCode::Esc, KeyModifiers::empty())),
                Action::Esc,
            ),
            (
                KeyConfig::from((KeyCode::Char('c'), KeyModifiers::CONTROL)),
                Action::Esc,
            ),
            (
                KeyConfig::from((KeyCode::Left, KeyModifiers::empty())),
                Action::MoveDateLeft,
            ),
            (
                KeyConfig::from((KeyCode::Right, KeyModifiers::empty())),
                Action::MoveDateRight,
            ),
            (
                KeyConfig::from((KeyCode::Up, KeyModifiers::empty())),
                Action::MoveDateUp,
            ),
            (
                KeyConfig::from((KeyCode::Down, KeyModifiers::empty())),
                Action::MoveDateDown,
            ),
            (
                KeyConfig::from((KeyCode::Char(' '), KeyModifiers::empty())),
                Action::SelectDate,
            ),
            (
                KeyConfig::from((KeyCode::Char('h'), KeyModifiers::CONTROL)),
                Action::ToggleHelp,
            ),
        ];
        keybindings.insert(CurrentScreen::Adding, input_keys.clone());
        keybindings.insert(CurrentScreen::Editing, input_keys);

        // Searching Screen
        let search_keys = vec![
            (
                KeyConfig::from((KeyCode::Enter, KeyModifiers::empty())),
                Action::Esc,
            ),
            (
                KeyConfig::from((KeyCode::Esc, KeyModifiers::empty())),
                Action::Esc,
            ),
            (
                KeyConfig::from((KeyCode::Char('h'), KeyModifiers::CONTROL)),
                Action::ToggleHelp,
            ),
        ];
        keybindings.insert(CurrentScreen::Searching, search_keys);

        // ConfirmingDelete Screen
        let confirm_keys = vec![
            (
                KeyConfig::from((KeyCode::Char('y'), KeyModifiers::empty())),
                Action::ConfirmDelete,
            ),
            (
                KeyConfig::from((KeyCode::Enter, KeyModifiers::empty())),
                Action::ConfirmDelete,
            ),
            (
                KeyConfig::from((KeyCode::Char('n'), KeyModifiers::empty())),
                Action::Esc,
            ),
            (
                KeyConfig::from((KeyCode::Esc, KeyModifiers::empty())),
                Action::Esc,
            ),
            (
                KeyConfig::from((KeyCode::Char('h'), KeyModifiers::empty())),
                Action::ToggleHelp,
            ),
        ];
        keybindings.insert(CurrentScreen::ConfirmingDelete, confirm_keys);

        // JiraConfiguring Screen
        let jira_keys = vec![
            (
                KeyConfig::from((KeyCode::Tab, KeyModifiers::empty())),
                Action::Tab,
            ),
            (
                KeyConfig::from((KeyCode::BackTab, KeyModifiers::SHIFT)),
                Action::BackTab,
            ),
            (
                KeyConfig::from((KeyCode::Enter, KeyModifiers::empty())),
                Action::Enter,
            ),
            (
                KeyConfig::from((KeyCode::Esc, KeyModifiers::empty())),
                Action::Esc,
            ),
            (
                KeyConfig::from((KeyCode::Char('h'), KeyModifiers::CONTROL)),
                Action::ToggleHelp,
            ),
        ];
        keybindings.insert(CurrentScreen::JiraConfiguring, jira_keys);

        // Help Screen
        let help_keys = vec![
            (
                KeyConfig::from((KeyCode::Char('q'), KeyModifiers::empty())),
                Action::ToggleHelp,
            ),
            (
                KeyConfig::from((KeyCode::Esc, KeyModifiers::empty())),
                Action::ToggleHelp,
            ),
            (
                KeyConfig::from((KeyCode::Char('h'), KeyModifiers::empty())),
                Action::ToggleHelp,
            ),
        ];
        keybindings.insert(CurrentScreen::Help, help_keys);

        Config {
            keybindings,
            jira: JiraConfig::default(),
        }
    }
}

impl Config {
    /// Loads configuration from the standard user config path.
    /// If the file does not exist, it saves the default configuration first.
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

    /// Persists the current configuration to the user's config file.
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(config_path) = get_config_path() {
            let content = toml::to_string_pretty(&self)?;
            fs::write(config_path, content)?;
        }
        Ok(())
    }

    /// Maps a raw `KeyEvent` to a logical `Action` based on the current screen state.
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

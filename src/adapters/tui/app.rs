use crate::domain::command::{
    MoveTaskCommand, RemoveTaskCommand, ToggleCompletedCommand, ToggleImportantCommand,
};
use crate::domain::task::Task;
use crate::ports::inbound::TaskServicePort;
use chrono::{DateTime, Datelike, Utc};
use ratatui::widgets::ListState;
use std::sync::Arc;
use time::Month;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum CurrentScreen {
    Main,
    Adding,
    Editing,
    Searching,
    ConfirmingDelete,
    Gantt,
    JiraConfiguring,
    Help,
}

#[derive(PartialEq)]
pub enum InputFocus {
    Title,
    Description,
    StartDate,
    EndDate,
    JiraDomain,
    JiraEmail,
    JiraToken,
    JiraProjects,
    JiraLabels,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum Action {
    Quit,
    Add,
    Edit,
    Delete,
    ConfirmDelete,
    ToggleCompleted,
    ToggleImportant,
    ToggleGantt,
    ToggleHelp,
    MoveUp,
    MoveDown,
    MoveTaskUp,
    MoveTaskDown,
    MoveToTop,
    MoveToBottom,
    PageUp,
    PageDown,
    Search,
    SyncJira,
    ClearCompleted,
    Esc,
    Enter,
    Tab,
    BackTab,
    MoveDateLeft,
    MoveDateRight,
    MoveDateUp,
    MoveDateDown,
    SelectDate,
    Macro(Vec<Action>),
}

pub struct App {
    pub task_service: Arc<dyn TaskServicePort>,
    pub current_screen: CurrentScreen,
    pub input_focus: InputFocus,
    pub list_state: ListState,
    pub list_offset: usize,
    pub title_input: String,
    pub description_input: String,
    pub start_date_input: String,
    pub end_date_input: String,
    pub search_query: String,
    pub editing_id: Option<String>,
    pub selected_date: chrono::NaiveDate,
    pub jira_domain_input: String,
    pub jira_email_input: String,
    pub jira_api_token_input: String,
    pub jira_projects_input: String,
    pub jira_labels_input: String,
    pub ticks: u64,
}

impl App {
    pub fn new(task_service: Arc<dyn TaskServicePort>) -> App {
        let mut list_state = ListState::default();
        if let Ok(tasks) = task_service.get_all_tasks() {
            if !tasks.is_empty() {
                list_state.select(Some(0));
            }
        }

        App {
            task_service,
            current_screen: CurrentScreen::Main,
            input_focus: InputFocus::Title,
            list_state,
            list_offset: 0,
            title_input: String::new(),
            description_input: String::new(),
            start_date_input: String::new(),
            end_date_input: String::new(),
            search_query: String::new(),
            editing_id: None,
            selected_date: Utc::now().date_naive(),
            jira_domain_input: String::new(),
            jira_email_input: String::new(),
            jira_api_token_input: String::new(),
            jira_projects_input: String::new(),
            jira_labels_input: String::new(),
            ticks: 0,
        }
    }

    pub fn on_tick(&mut self) {
        self.ticks += 1;
    }

    pub fn get_filtered_items(&self) -> Vec<Task> {
        let items = self.task_service.get_all_tasks().unwrap_or_default();
        let mut filtered: Vec<Task> = if self.search_query.is_empty() {
            items
        } else {
            let query = self.search_query.to_lowercase();
            items
                .into_iter()
                .filter(|item| {
                    item.title().to_lowercase().contains(&query)
                        || item.description().to_lowercase().contains(&query)
                })
                .collect()
        };

        // Consistent sorting for UI and Actions
        filtered.sort_by(|a, b| {
            match (a.start_date(), b.start_date()) {
                (Some(da), Some(db)) => da.cmp(&db),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            }
            .then_with(|| match (a.end_date(), b.end_date()) {
                (Some(da), Some(db)) => da.cmp(&db),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            })
            .then_with(|| a.title().cmp(&b.title()))
            .then_with(|| a.id.cmp(&b.id))
        });

        filtered
    }

    pub fn next(&mut self) {
        let items = self.get_filtered_items();
        if items.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let items = self.get_filtered_items();
        if items.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn move_to_top(&mut self) {
        let items = self.get_filtered_items();
        if !items.is_empty() {
            self.list_state.select(Some(0));
        }
    }

    pub fn move_to_bottom(&mut self) {
        let items = self.get_filtered_items();
        if !items.is_empty() {
            self.list_state.select(Some(items.len() - 1));
        }
    }

    pub fn page_up(&mut self) {
        let items = self.get_filtered_items();
        if items.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => i.saturating_sub(10),
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn page_down(&mut self) {
        let items = self.get_filtered_items();
        if items.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => (i + 10).min(items.len() - 1),
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn move_task_up(&mut self) {
        let items = self.get_filtered_items();
        if let Some(i) = self.list_state.selected() {
            if i > 0 && i < items.len() {
                let id = items[i].id.clone();
                let cmd = Box::new(MoveTaskCommand { id, delta: 1 });
                let _ = self.task_service.execute_command(cmd);
                self.list_state.select(Some(i - 1));
            }
        }
    }

    pub fn move_task_down(&mut self) {
        let items = self.get_filtered_items();
        if let Some(i) = self.list_state.selected() {
            if i < items.len() - 1 {
                let id = items[i].id.clone();
                let cmd = Box::new(MoveTaskCommand { id, delta: -1 });
                let _ = self.task_service.execute_command(cmd);
                self.list_state.select(Some(i + 1));
            }
        }
    }

    pub fn remove_selected(&mut self) {
        let items = self.get_filtered_items();
        if let Some(i) = self.list_state.selected() {
            if i < items.len() {
                let id = items[i].id.clone();
                let cmd = Box::new(RemoveTaskCommand { id });
                let _ = self.task_service.execute_command(cmd);

                let new_items = self.get_filtered_items();
                if new_items.is_empty() {
                    self.list_state.select(None);
                } else if i >= new_items.len() {
                    self.list_state.select(Some(new_items.len() - 1));
                }
            }
        }
    }

    pub fn toggle_completed(&mut self) {
        let items = self.get_filtered_items();
        if let Some(i) = self.list_state.selected() {
            if i < items.len() {
                let cmd = Box::new(ToggleCompletedCommand {
                    id: items[i].id.clone(),
                });
                let _ = self.task_service.execute_command(cmd);
            }
        }
    }

    pub fn toggle_important(&mut self) {
        let items = self.get_filtered_items();
        if let Some(i) = self.list_state.selected() {
            if i < items.len() {
                let cmd = Box::new(ToggleImportantCommand {
                    id: items[i].id.clone(),
                });
                let _ = self.task_service.execute_command(cmd);
            }
        }
    }

    pub fn next_field(&mut self) {
        self.input_focus = match self.input_focus {
            InputFocus::Title => InputFocus::Description,
            InputFocus::Description => InputFocus::StartDate,
            InputFocus::StartDate => InputFocus::EndDate,
            InputFocus::EndDate => InputFocus::Title,
            InputFocus::JiraDomain => InputFocus::JiraEmail,
            InputFocus::JiraEmail => InputFocus::JiraToken,
            InputFocus::JiraToken => InputFocus::JiraProjects,
            InputFocus::JiraProjects => InputFocus::JiraLabels,
            InputFocus::JiraLabels => InputFocus::JiraDomain,
        };
        self.sync_selected_date();
    }

    pub fn sync_selected_date(&mut self) {
        let date_str = match self.input_focus {
            InputFocus::StartDate => &self.start_date_input,
            InputFocus::EndDate => &self.end_date_input,
            _ => return,
        };

        if let Ok(date) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
            self.selected_date = date;
        } else {
            self.selected_date = Utc::now().date_naive();
        }
    }

    pub fn parse_start_date(&self) -> Option<DateTime<Utc>> {
        if self.start_date_input.is_empty() {
            return None;
        }
        let s = if self.start_date_input.len() == 10 {
            format!("{}T00:00:00Z", self.start_date_input)
        } else {
            self.start_date_input.clone()
        };
        DateTime::parse_from_rfc3339(&s)
            .ok()
            .map(|d| d.with_timezone(&Utc))
    }

    pub fn parse_end_date(&self) -> Option<DateTime<Utc>> {
        if self.end_date_input.is_empty() {
            return None;
        }
        let s = if self.end_date_input.len() == 10 {
            format!("{}T23:59:59Z", self.end_date_input)
        } else {
            self.end_date_input.clone()
        };
        DateTime::parse_from_rfc3339(&s)
            .ok()
            .map(|d| d.with_timezone(&Utc))
    }

    pub fn move_date_left(&mut self) {
        self.selected_date = self.selected_date.pred_opt().unwrap_or(self.selected_date);
    }

    pub fn move_date_right(&mut self) {
        self.selected_date = self.selected_date.succ_opt().unwrap_or(self.selected_date);
    }

    pub fn move_date_up(&mut self) {
        if let Some(d) = self.selected_date.checked_sub_days(chrono::Days::new(7)) {
            self.selected_date = d;
        }
    }

    pub fn move_date_down(&mut self) {
        if let Some(d) = self.selected_date.checked_add_days(chrono::Days::new(7)) {
            self.selected_date = d;
        }
    }

    pub fn select_date(&mut self) {
        let date_str = self.selected_date.format("%Y-%m-%d").to_string();
        match self.input_focus {
            InputFocus::StartDate => self.start_date_input = date_str,
            InputFocus::EndDate => self.end_date_input = date_str,
            _ => {}
        }
    }

    pub fn get_time_date(&self) -> time::Date {
        let year = self.selected_date.year();
        let month = self.selected_date.month() as u8;
        let day = self.selected_date.day() as u8;

        time::Date::from_calendar_date(year, Month::try_from(month).unwrap_or(Month::January), day)
            .unwrap_or_else(|_| time::Date::from_calendar_date(2026, Month::January, 1).unwrap())
    }
}

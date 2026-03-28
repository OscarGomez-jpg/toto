use crate::domain::task::Task;
use crate::ports::inbound::TaskServicePort;
use ratatui::widgets::ListState;
use std::sync::Arc;
use unicode_segmentation::UnicodeSegmentation;

#[derive(PartialEq)]
pub enum CurrentScreen {
    Main,
    Adding,
    Editing,
    Searching,
    ConfirmingDelete,
}

pub struct App {
    pub task_service: Arc<dyn TaskServicePort>,
    pub current_screen: CurrentScreen,
    pub list_state: ListState,
    pub input: String,
    pub search_query: String,
    pub editing_id: Option<String>,
    pub ticks: u64,
    pub scroll_offset: usize,
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
            list_state,
            input: String::new(),
            search_query: String::new(),
            editing_id: None,
            ticks: 0,
            scroll_offset: 0,
        }
    }

    pub fn on_tick(&mut self) {
        self.ticks += 1;
    }

    pub fn get_filtered_items(&self) -> Vec<Task> {
        let items = self.task_service.get_all_tasks().unwrap_or_default();
        if self.search_query.is_empty() {
            items
        } else {
            let query = self.search_query.to_lowercase();
            items
                .into_iter()
                .filter(|item| item.content.to_lowercase().contains(&query))
                .collect()
        }
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
                let _ = self.task_service.move_task(id, 1); // 1 is up in our position DESC order
                self.list_state.select(Some(i - 1));
            }
        }
    }

    pub fn move_task_down(&mut self) {
        let items = self.get_filtered_items();
        if let Some(i) = self.list_state.selected() {
            if i < items.len() - 1 {
                let id = items[i].id.clone();
                let _ = self.task_service.move_task(id, -1); // -1 is down in our position DESC order
                self.list_state.select(Some(i + 1));
            }
        }
    }

    pub fn remove_selected(&mut self) {
        let items = self.get_filtered_items();
        if let Some(i) = self.list_state.selected() {
            if i < items.len() {
                let id = items[i].id.clone();
                let _ = self.task_service.remove_task(id);

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
                let _ = self.task_service.toggle_completed(items[i].id.clone());
            }
        }
    }

    pub fn toggle_important(&mut self) {
        let items = self.get_filtered_items();
        if let Some(i) = self.list_state.selected() {
            if i < items.len() {
                let _ = self.task_service.toggle_important(items[i].id.clone());
            }
        }
    }

    pub fn cursor_position(&self) -> usize {
        self.input.graphemes(true).count()
    }
}

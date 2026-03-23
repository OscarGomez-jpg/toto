use ratatui::widgets::ListState;
use crate::list::{TodoList, Line};

#[derive(PartialEq)]
pub enum CurrentScreen {
    Main,
    Adding,
    Editing,
    Searching,
    ConfirmingDelete,
}

pub struct App {
    pub todo_list: TodoList,
    pub current_screen: CurrentScreen,
    pub list_state: ListState,
    pub input: String,
    pub search_query: String,
    pub editing_id: Option<i64>,
}

impl App {
    pub fn new() -> App {
        let todo_list = TodoList::load();
        let mut list_state = ListState::default();
        if !todo_list.get_all().is_empty() {
            list_state.select(Some(0));
        }

        App {
            todo_list,
            current_screen: CurrentScreen::Main,
            list_state,
            input: String::new(),
            search_query: String::new(),
            editing_id: None,
        }
    }

    pub fn get_filtered_items(&self) -> Vec<Line> {
        let items = self.todo_list.get_all();
        if self.search_query.is_empty() {
            items
        } else {
            let query = self.search_query.to_lowercase();
            items.into_iter()
                .filter(|item| item.content.to_lowercase().contains(&query))
                .collect()
        }
    }

    pub fn next(&mut self) {
        let items = self.get_filtered_items();
        if items.is_empty() { return; }
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
        if items.is_empty() { return; }
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

    pub fn remove_selected(&mut self) {
        let items = self.get_filtered_items();
        if let Some(i) = self.list_state.selected() {
            if i < items.len() {
                let id = items[i].id;
                self.todo_list.remove(id);
                
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
                self.todo_list.toggle_completed(items[i].id);
            }
        }
    }

    pub fn toggle_important(&mut self) {
        let items = self.get_filtered_items();
        if let Some(i) = self.list_state.selected() {
            if i < items.len() {
                self.todo_list.toggle_important(items[i].id);
            }
        }
    }
}

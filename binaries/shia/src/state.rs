use std::thread::JoinHandle;
use tui_textarea::TextArea;

use crate::prelude::*;

pub struct AppState<'a> {
    pub input_textarea: TextArea<'a>,
    pub tabs: Vec<TabState>,
    pub tab_index: usize,
    pub scroll: u16,
}

pub struct TabState {
    pub title: String,
    pub page: PageState,
}

pub struct PageState {
    pub content: Arc<RwLock<String>>,
}

impl <'a>AppState<'a> {
    pub fn on_tick(&mut self) {
        self.scroll += 1;
        self.scroll %= 10;
    }
}

impl Default for PageState {
    fn default() -> Self {
        Self {
            content: Arc::new(RwLock::new(String::new())),
        }
    }
}

impl Default for TabState {
    fn default() -> Self {
        Self {
            title: "<Unamed session>".to_string(),
            page: PageState::default(),
        }
    }
}

impl <'a>Default for AppState<'a> {
    fn default() -> Self {
        Self {
            input_textarea: TextArea::new(vec![" corbe   entropie   main ≢  ?4 ~1      ".to_string()]),
            tabs: vec![TabState::default(),TabState::default(),TabState::default()],
            tab_index: 0,
            scroll: 0,
        }
    }
}
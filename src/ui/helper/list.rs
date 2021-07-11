use tui::widgets::ListState;

pub struct StatefulList {
    pub state: ListState,
    pub items: Vec<String>,
}

impl StatefulList {
    pub fn with_items<T: ToString>(items: Vec<T>) -> StatefulList {
        StatefulList {
            state: ListState::default(),
            items: items.iter().map(|item| item.to_string()).collect(),
        }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) if i >= (self.items.len() - 1) => 0,
            Some(i) => i + 1,
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) if i == 0 => self.items.len() - 1,
            Some(i) => i - 1,
            None => 0,
        };
        self.state.select(Some(i));
    }
}

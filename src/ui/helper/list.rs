use tui::widgets::ListState;

/// This is a helper struct for tui-rs
/// It's a simple wrapper manage a list of strings where items can be selected.
pub struct StatefulList {
    pub state: ListState,
    pub items: Vec<String>,
}

impl StatefulList {
    /// Create the list from a vector of things that can be converted in to Strings.
    pub fn with_items<T: ToString>(items: Vec<T>) -> StatefulList {
        let mut list = StatefulList {
            state: ListState::default(),
            items: items.iter().map(|item| item.to_string()).collect(),
        };
        // Select the first element by default.
        if list.items.len() > 0 {
            list.state.select(Some(0));
        }

        list
    }

    /// Select the next item in the list.
    /// If there are no more items, we start at the first item.
    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) if i >= (self.items.len() - 1) => 0,
            Some(i) => i + 1,
            None => 0,
        };
        self.state.select(Some(i));
    }

    /// Select the previous item in the list.
    /// If there are no more items, we go to the the last item of the list.
    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) if i == 0 => self.items.len() - 1,
            Some(i) => i - 1,
            None => 0,
        };
        self.state.select(Some(i));
    }
}

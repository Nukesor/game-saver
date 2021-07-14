use tui::widgets::ListState;

use super::files::SaveFile;

pub trait StatefulList {
    type Item;

    fn get_state(&mut self) -> &mut ListState;
    fn get_items(&mut self) -> &Vec<Self::Item>;
}

/// This is a helper struct for tui-rs
/// It's a simple wrapper manage a list of strings where items can be selected.
pub struct StringList {
    pub state: ListState,
    pub items: Vec<String>,
}

impl StringList {
    /// Create the list from a vector of things that can be converted in to Strings.
    pub fn with_items<T: ToString>(items: Vec<T>) -> StringList {
        let mut list = StringList {
            state: ListState::default(),
            items: items.iter().map(|item| item.to_string()).collect(),
        };
        list.autoselect_first();

        list
    }

    /// If something is selected, return the selected item.
    pub fn get_selected(&self) -> Option<String> {
        let selected = self.state.selected()?;
        self.items.get(selected).cloned()
    }
}

impl StatefulList for StringList {
    type Item = String;

    fn get_state(&mut self) -> &mut ListState {
        &mut self.state
    }

    fn get_items(&mut self) -> &Vec<Self::Item> {
        &self.items
    }
}

/// This is a helper struct for tui-rs
/// It's a wrapper that manages a list of savegame file infos where items can be selected.
pub struct SaveList {
    pub state: ListState,
    pub items: Vec<SaveFile>,
}

impl SaveList {
    /// Create the list from a vector of things that can be converted in to Strings.
    pub fn with_items(items: Vec<SaveFile>) -> SaveList {
        let mut list = SaveList {
            state: ListState::default(),
            items,
        };
        list.autoselect_first();

        list
    }

    /// If something is selected, return the selected item.
    pub fn get_selected(&self) -> Option<SaveFile> {
        let selected = self.state.selected()?;
        self.items.get(selected).cloned()
    }
}

impl StatefulList for SaveList {
    type Item = SaveFile;
    fn get_state(&mut self) -> &mut ListState {
        &mut self.state
    }

    fn get_items(&mut self) -> &Vec<Self::Item> {
        &self.items
    }
}

pub trait Navigate {
    /// Respect any previous state, as long as it's valid.
    /// Otherwise autoselect the first entry if possible.
    fn focus(&mut self);

    /// Autoselect the first entry if possible.
    fn autoselect_first(&mut self);

    /// Select the next item in the list.
    /// If there are no more items, we start at the first item.
    fn next(&mut self);

    /// Select the previous item in the list.
    /// If there are no more items, we go to the the last item of the list.
    fn previous(&mut self);
}

impl<I, T> Navigate for T
where
    T: StatefulList<Item = I>,
{
    fn focus(&mut self) {
        // Don't change state, if it's valid
        if let Some(selected) = self.get_state().selected() {
            if self.get_items().len() > selected {
                return;
            }
        }

        self.autoselect_first()
    }

    fn autoselect_first(&mut self) {
        if self.get_items().is_empty() {
            // Remove selection, if no elements exist.
            self.get_state().select(None)
        } else {
            // Select the first element, if there are any elements
            self.get_state().select(Some(0))
        }
    }

    fn next(&mut self) {
        if self.get_items().is_empty() {
            self.get_state().select(None);
            return;
        }
        let i = match self.get_state().selected() {
            Some(i) if i >= (self.get_items().len() - 1) => 0,
            Some(i) => i + 1,
            None => 0,
        };
        self.get_state().select(Some(i));
    }

    fn previous(&mut self) {
        if self.get_items().is_empty() {
            self.get_state().select(None);
            return;
        }
        let i = match self.get_state().selected() {
            Some(i) if i == 0 => self.get_items().len() - 1,
            Some(i) => i - 1,
            None => 0,
        };
        self.get_state().select(Some(i));
    }
}

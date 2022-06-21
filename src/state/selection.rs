use std::collections::HashSet;

use yew::virtual_dom::Key;

use crate::props::ExtractKeyFn;

pub struct Selection {
    multiselect: bool,
    selection: Option<Key>, // used for single row
    selection_map: HashSet<Key>, // used for multiselect
}

impl Selection  {

    pub fn new() -> Self {
        Self {
            multiselect: false,
            selection: None,
            selection_map: HashSet::new(),
        }
    }

    pub fn multiselect(mut self, multiselect: bool) -> Self {
        self.multiselect = multiselect;
        self
    }

    pub fn clear(&mut self) {
        self.selection = None;
        self.selection_map = HashSet::new();
    }

    pub fn select(&mut self, key: &Key) {
        match self.multiselect {
            false => self.selection = Some(key.clone()),
            true => {
                let mut map = HashSet::new();
                map.insert(key.clone());
                self.selection_map = map;
            }
        }
    }

    pub fn toggle(&mut self, key: &Key) {
        match self.multiselect {
            false => {
                if let Some(current) = &self.selection {
                    if current == key {
                        self.selection = None;
                    } else {
                        self.selection = Some(key.clone());
                    }
                } else {
                    self.selection = Some(key.clone());
                }
            }
            true => {
                if self.selection_map.contains(key) {
                    self.selection_map.remove(key);
                } else {
                   self.selection_map.insert(key.clone());
                }
            }
        }
    }

    pub fn is_selected(&self, key: &Key) -> bool {
        match self.multiselect {
            false => {
                if let Some(current) = &self.selection {
                    if current == key {
                        return true;
                    }
                }
            }
            true => {
                if self.selection_map.contains(key) {
                    return true;
                }
            }
        }
        false
    }

    pub fn selected_keys<T>(&self, data: &Vec<T>, extract_key: Option<&ExtractKeyFn<T>>) -> Vec<Key> {

        let mut keys = Vec::new();
        for (i, record) in data.iter().enumerate() {
            let key = match &extract_key {
                None => Key::from(i),
                Some(extract_fn) => extract_fn.apply(record),
            };
            if self.is_selected(&key) {
                keys.push(key);
            }
        }
        keys
    }
}

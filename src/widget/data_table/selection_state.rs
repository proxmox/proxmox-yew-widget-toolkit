use std::collections::HashSet;

use yew::virtual_dom::Key;

use crate::state::DataStore;

use super::{DataTable, RowSelectionStatus};

pub(super) struct TableSelection {
    pub status: RowSelectionStatus,
    // Full coverage alone does not opt into extending the selection to new rows.
    all_rows_chosen: bool,
}

impl Default for TableSelection {
    fn default() -> Self {
        Self {
            status: RowSelectionStatus::Nothing,
            all_rows_chosen: false,
        }
    }
}

impl TableSelection {
    pub fn select_all<S: DataStore>(&mut self, props: &DataTable<S>) {
        self.all_rows_chosen = false;
        let Some(selection) = &props.selection else {
            self.status = RowSelectionStatus::Nothing;
            return;
        };
        let record_count = props.store.filtered_data_len();
        if !selection.is_multiselect() || record_count == 0 {
            self.update_status(props);
            return;
        }

        let keys = (0..record_count)
            .filter_map(|pos| props.store.lookup_filtered_record_key(pos))
            .collect();
        selection.bulk_select(keys);
        self.status = RowSelectionStatus::All;
        self.all_rows_chosen = true;
    }

    pub fn clear<S: DataStore>(&mut self, props: &DataTable<S>) {
        self.all_rows_chosen = false;
        if let Some(selection) = &props.selection {
            selection.clear();
        }
        self.update_status(props);
    }

    pub fn replace<S: DataStore>(&mut self, props: &DataTable<S>) {
        self.all_rows_chosen = false;
        self.update_status(props);
    }

    pub fn update_status<S: DataStore>(&mut self, props: &DataTable<S>) {
        let record_count = props.store.filtered_data_len();
        self.status = match &props.selection {
            Some(selection) if record_count > 0 => {
                if record_count == selection.len() {
                    RowSelectionStatus::All
                } else if !selection.is_empty() {
                    RowSelectionStatus::Some
                } else {
                    RowSelectionStatus::Nothing
                }
            }
            _ => RowSelectionStatus::Nothing,
        };
        self.all_rows_chosen &= self.status == RowSelectionStatus::All
            && props.selection.as_ref().is_some_and(|selection| {
                selection.is_multiselect()
                    && (0..record_count).all(|pos| {
                        props
                            .store
                            .lookup_filtered_record_key(pos)
                            .is_some_and(|key| selection.contains(&key))
                    })
            });
    }

    pub fn data_changed<S: DataStore>(&mut self, props: &DataTable<S>) {
        if self.all_rows_chosen && props.store.filtered_data_len() > 0 {
            self.select_all(props);
        } else if let Some(selection) = &props.selection {
            let mut selection = selection.write();
            if selection.is_multiselect() {
                let mut keys: HashSet<Key> = HashSet::new();
                for (_pos, node) in props.store.filtered_data() {
                    let key = props.store.extract_key(&node.record());
                    if selection.contains(&key) {
                        keys.insert(key);
                    }
                }
                selection.bulk_select(keys);
            } else if let Some(key) = selection.selected_key() {
                if props.store.filtered_record_pos(&key).is_none() {
                    selection.clear();
                }
            }
        }
        self.update_status(props);
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::rc::Rc;

    use crate::props::ExtractKeyFn;
    use crate::state::{Selection, Store};

    use super::*;

    fn table(rows: Vec<u32>, multiselect: bool) -> (TableSelection, DataTable<Store<u32>>) {
        let store = Store::with_extract_key(ExtractKeyFn::new(|v: &u32| Key::from(*v as u64)));
        store.set_data(rows);
        let props = DataTable::new(Rc::new(Vec::new()), store)
            .selection(Selection::new().multiselect(multiselect));
        (TableSelection::default(), props)
    }

    fn assert_keys(props: &DataTable<Store<u32>>, keys: &[u32]) {
        let selected: HashSet<_> = props
            .selection
            .as_ref()
            .unwrap()
            .selected_keys()
            .into_iter()
            .collect();
        assert_eq!(
            selected,
            keys.iter().map(|key| Key::from(*key as u64)).collect()
        );
    }

    #[test]
    fn select_all_extends_to_added_rows() {
        let (mut state, props) = table(vec![1, 2], true);
        state.select_all(&props);
        props.store.set_data(vec![1, 2, 3]);
        state.data_changed(&props);
        assert_keys(&props, &[1, 2, 3]);
        assert_eq!(state.status, RowSelectionStatus::All);
    }

    #[test]
    fn empty_filter_cancels_select_all_with_or_without_clear() {
        for clear in [false, true] {
            let (mut state, props) = table(vec![1, 2], true);
            state.select_all(&props);
            props.store.set_filter(|_: &u32| false);
            state.data_changed(&props);
            assert_eq!(state.status, RowSelectionStatus::Nothing);
            assert!(!state.all_rows_chosen);
            if clear {
                props.selection.as_ref().unwrap().clear();
                state.update_status(&props);
            }
            props.store.set_filter(|_: &u32| true);
            state.data_changed(&props);
            assert_keys(&props, &[]);
        }
    }

    #[test]
    fn empty_table_cannot_establish_select_all() {
        let (mut state, props) = table(vec![], true);
        state.select_all(&props);
        assert!(!state.all_rows_chosen);
        props.store.set_data(vec![1, 2]);
        state.data_changed(&props);
        assert_keys(&props, &[]);
    }

    #[test]
    fn clear_and_deselection_cancel_select_all() {
        for clear in [false, true] {
            let (mut state, props) = table(vec![1, 2], true);
            state.select_all(&props);
            if clear {
                state.clear(&props);
            } else {
                props.selection.as_ref().unwrap().toggle(Key::from(2u64));
                state.update_status(&props);
            }
            props.store.set_data(vec![1, 2, 3]);
            state.data_changed(&props);
            assert_keys(&props, if clear { &[] } else { &[1] });
        }
    }

    #[test]
    fn replacing_or_removing_selection_cancels_select_all() {
        let (mut state, mut props) = table(vec![1, 2], true);
        state.select_all(&props);
        let replacement = Selection::new().multiselect(true);
        replacement.bulk_select([Key::from(1u64), Key::from(2u64)].into());
        props.selection = Some(replacement);
        state.replace(&props);
        props.store.set_data(vec![1, 2, 3]);
        state.data_changed(&props);
        assert_keys(&props, &[1, 2]);
        state.select_all(&props);
        props.selection = None;
        state.replace(&props);
        assert!(!state.all_rows_chosen);
        assert_eq!(state.status, RowSelectionStatus::Nothing);
    }

    #[test]
    fn single_row_selection_does_not_extend_when_filter_widens() {
        for multiselect in [false, true] {
            let (mut state, props) = table(vec![1, 2], multiselect);
            props.store.set_filter(|row: &u32| *row == 1);
            state.data_changed(&props);
            props.selection.as_ref().unwrap().select(Key::from(1u64));
            state.update_status(&props);
            assert_eq!(state.status, RowSelectionStatus::All);
            props.store.set_filter(|_: &u32| true);
            state.data_changed(&props);
            assert_keys(&props, &[1]);
        }
    }

    #[test]
    fn select_all_does_not_expand_single_selection() {
        let (mut state, props) = table(vec![1], false);
        props.selection.as_ref().unwrap().select(Key::from(1u64));
        state.select_all(&props);
        props.store.set_data(vec![1, 2]);
        state.data_changed(&props);
        assert_keys(&props, &[1]);
    }

    #[test]
    fn batched_external_updates_do_not_establish_select_all() {
        let (mut state, props) = table(vec![1, 2], true);
        let selection = props.selection.as_ref().unwrap();
        let notifications = Rc::new(Cell::new(0));
        let _observer = selection.add_listener({
            let notifications = notifications.clone();
            move |_| notifications.set(notifications.get() + 1)
        });
        {
            let mut selection = selection.write();
            selection.select(Key::from(1u64));
            selection.select(Key::from(2u64));
        }
        assert_eq!(notifications.get(), 1);
        state.update_status(&props);
        props.store.set_data(vec![1, 2, 3]);
        state.data_changed(&props);
        assert_keys(&props, &[1, 2]);

        state.select_all(&props);
        {
            let mut selection = selection.write();
            selection.clear();
            selection.select(Key::from(2u64));
        }
        state.update_status(&props);
        props.store.set_data(vec![1, 2, 3, 4]);
        state.data_changed(&props);
        assert_keys(&props, &[2]);
    }

    #[test]
    fn foreign_keys_cannot_preserve_select_all_intent() {
        let (mut state, props) = table(vec![1, 2], true);
        state.select_all(&props);
        props
            .selection
            .as_ref()
            .unwrap()
            .bulk_select([Key::from(1u64), Key::from(99u64)].into());
        state.update_status(&props);
        props.store.set_data(vec![1, 2, 3]);
        state.data_changed(&props);
        assert_keys(&props, &[1]);
    }
}

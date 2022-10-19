use std::rc::Rc;
use std::cell::RefCell;

use std::ops::Range;

use yew::html::IntoPropValue;
use yew::virtual_dom::Key;
//use crate::props::{ExtractKeyFn, IntoExtractKeyFn};
use crate::props::{ExtractKeyFn, FilterFn, IntoFilterFn, SorterFn, IntoSorterFn};
use super::TreeNode;

pub fn optional_list_rc_ptr_eq<T>(a: &Option<Vec<Rc<T>>>, b: &Option<Vec<Rc<T>>>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => {
            if a.len() != b.len() { return false; }
            for i in 0..a.len() {
                if !Rc::ptr_eq(&a[i], &b[i]) { return false; }
            }
            true
        }
        (None, None) => true,
        _ => false,
    }
}

pub struct TreeFilter<T> {
    data: Option<Vec<Rc<RefCell<TreeNode<T>>>>>,
    filtered_data: Vec<Rc<RefCell<TreeNode<T>>>>,
    sorter: Option<SorterFn<TreeNode<T>>>,
    filter: Option<FilterFn<TreeNode<T>>>,
    cursor: Option<usize>,
    extract_key: ExtractKeyFn<T>,
}

pub fn flatten_tree_children<T>(
    list: &mut Vec<Rc<RefCell<TreeNode<T>>>>,
    level: usize,
    parent: Option<usize>,
    children: &[Rc<RefCell<TreeNode<T>>>],
    sorter: &Option<SorterFn<TreeNode<T>>>,
    filter: &Option<FilterFn<TreeNode<T>>>,
) {

    let mut children: Vec<Rc<RefCell<TreeNode<T>>>> = children.iter()
        .filter(|item| match filter {
            Some(filter) => filter.apply(0, &item.borrow()), // fixme: remove fiter record_num param
            None => true,
        })
        .map(|child| {
            {
                let mut child = child.borrow_mut();
                child.level = level;
                child.parent = parent;
            }
            Rc::clone(child)
        })
        .collect();

    if let Some(sorter) = sorter {
        children.sort_by(|a, b| { sorter.cmp(&a.borrow(), &b.borrow()) });
    }

    for subtree in children {
        let subtree_clone = subtree.clone();
        let subtree = subtree.borrow();
        if subtree.expanded {
            list.push(subtree_clone);
            if let Some(subtree_children) = &subtree.children {
                flatten_tree_children(list, level + 1,  Some(list.len() - 1), subtree_children, sorter, filter);
            }
        } else {
            list.push(subtree_clone);
        }
    }
}

pub struct TreeFilterIterator<'a, T> {
    data: &'a TreeFilter<T>,
    pos: usize,
    range: Option<Range<usize>>,
}

impl <T> TreeFilter<T> {

    pub fn new(extract_key: ExtractKeyFn<T>) -> Self {
        Self {
            data: None,
            filtered_data: Vec::new(),
            sorter: None,
            filter: None,
            cursor: None,
            extract_key,
        }
    }

    pub fn data_list(mut self, list: impl IntoPropValue<Option<Rc<Vec<Rc<T>>>>>) -> Self {
        self.set_data_list(list);
        self
    }

    pub fn set_data_list(&mut self, list: impl IntoPropValue<Option<Rc<Vec<Rc<T>>>>>) {
        let list = list.into_prop_value();
        let children = list.map(|data| {
            data.iter()
                .map(|record| TreeNode {
                    record: record.clone(),
                    expanded: false,
                    children: None,
                    parent: None,
                    level: 0,
                })
                .fold(Vec::new(), |mut acc, node| { acc.push(Rc::new(RefCell::new(node))); acc })
        });
        self.set_data(children);
    }

    pub fn data(mut self, data: impl IntoPropValue<Option<Vec<Rc<RefCell<TreeNode<T>>>>>>) -> Self {
        self.set_data(data);
        self
    }

    pub fn set_data(&mut self, data: impl IntoPropValue<Option<Vec<Rc<RefCell<TreeNode<T>>>>>>) {
        let new_data = data.into_prop_value();
        if optional_list_rc_ptr_eq(&self.data, &new_data) { return; }

        self.data = new_data;
        self.update_filtered_data();
    }

    pub fn sorter(mut self, sorter: impl IntoSorterFn<TreeNode<T>>) -> Self {
        self.set_sorter(sorter);
        self
    }

    pub fn set_sorter(&mut self, sorter: impl IntoSorterFn<TreeNode<T>>) {
        self.sorter = sorter.into_sorter_fn();
        self.update_filtered_data();
    }

    pub fn filter(mut self, filter: impl IntoFilterFn<TreeNode<T>>) -> Self {
        self.set_filter(filter);
        self
    }

    pub fn set_filter(&mut self, filter: impl IntoFilterFn<TreeNode<T>>) {
        self.filter = filter.into_filter_fn();
        self.update_filtered_data();
    }

    pub fn lookup_filtered_record(&self, cursor: usize) -> Option<&Rc<RefCell<TreeNode<T>>>> {
        self.filtered_data.get(cursor)
    }

    pub fn filtered_record_pos(&self, key: &Key) -> Option<usize> {
        self.filtered_data.iter().position(|item| key == &self.extract_key.apply(&item.borrow().record))
    }

    pub fn filtered_data_len(&self) -> usize {
        self.filtered_data.len()
    }

    fn update_filtered_data(&mut self) {

        let old_cursor_record_key = if let Some(cursor) = self.cursor {
            self.lookup_filtered_record(cursor)
                .map(|item| self.extract_key.apply(&item.borrow().record))
        } else {
            None
        };

        let data = match &self.data {
            None => {
                self.cursor = None;
                self.filtered_data = Vec::new();
                return;
            }
            Some(data) => data,
        };

        let mut list = Vec::new();
        flatten_tree_children(&mut list, 0, None, data, &self.sorter, &self.filter);

        let new_cursor = match &old_cursor_record_key {
            Some(record_key) => self.filtered_record_pos(record_key),
            None => None,
        };

        self.cursor = new_cursor;
        self.filtered_data = list;

    }

    pub fn get_cursor(&self) -> Option<usize> {
        self.cursor
    }

    pub fn cursor(mut self, cursor: Option<usize>) -> Self {
        self.set_cursor(cursor);
        self
    }

    pub fn set_cursor(&mut self, cursor: Option<usize>) {
        self.cursor = match cursor {
            Some(c) => {
                let len = self.filtered_data_len();
                if c < len {
                    Some(c)
                } else {
                    None
                }
            }
            None => None,
        }
    }

    pub fn cursor_down(&mut self) {
        let len = self.filtered_data_len();
        if len == 0 {
            self.cursor = None;
            return;
        }
        self.cursor = match self.cursor {
            Some(n) => if (n + 1) < len { Some(n + 1) }  else { Some(0) },
            None => Some(0),
        };
    }

    pub fn cursor_up(&mut self) {
        let len = self.filtered_data_len();
        if len == 0 {
            self.cursor = None;
            return;
        }

        self.cursor = match self.cursor {
            Some(n) => if n > 0 { Some(n - 1) } else { Some(len - 1) },
            None => Some(len - 1),
        };
    }

    pub fn filtered_data(&self) -> TreeFilterIterator<T> {
        TreeFilterIterator {
            range: None,
            pos: 0,
            data: self,
        }
    }

    pub fn filtered_data_range(&self, range: Range<usize>) -> TreeFilterIterator<T> {
        TreeFilterIterator {
            pos: range.start,
            range: Some(range),
            data: self,
        }
    }
}

impl <'a, T> Iterator for TreeFilterIterator<'a, T> {
    type Item = (usize, &'a Rc<RefCell<TreeNode<T>>>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.data.data.is_none() {
            return None;
        };

        if let Some(range) = &self.range {
            if range.end <= self.pos {
                return None;
            }
        }

        if self.data.filtered_data.len() <= self.pos {
            return None;
        }

        let pos = self.pos;
        self.pos += 1;

        Some((pos, &self.data.filtered_data[pos]))
    }
}

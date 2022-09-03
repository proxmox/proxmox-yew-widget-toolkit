use std::rc::Rc;
use std::ops::Range;

use yew::html::IntoPropValue;

//use crate::props::{ExtractKeyFn, IntoExtractKeyFn};
use crate::props::{FilterFn, IntoFilterFn, SorterFn, IntoSorterFn};

fn my_data_eq_fn<T>(a: &Option<Rc<Vec<T>>>, b: &Option<Rc<Vec<T>>>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => Rc::ptr_eq(a, b),
        (None, None) => true,
        _ => false,
    }
}

pub struct DataFilter<T> {
    data: Option<Rc<Vec<T>>>,
    filtered_data: Vec<usize>,
   // extract_key: Option<ExtractKeyFn<T>>,
    sorter: Option<SorterFn<T>>,
    filter: Option<FilterFn<T>>,
    cursor: Option<usize>,
}

pub struct DataFilterIterator<'a, T> {
    data: &'a DataFilter<T>,
    pos: usize,
    range: Option<Range<usize>>,
}

impl <T> DataFilter<T> {

    pub fn new() -> Self {
        Self {
            data: None,
            filtered_data: Vec::new(),
            //extract_key: None,
            sorter: None,
            filter: None,
            cursor: None,
        }
    }

    pub fn data(mut self, data: impl IntoPropValue<Option<Rc<Vec<T>>>>) -> Self {
        self.set_data(data);
        self
    }

    pub fn set_data(&mut self, data: impl IntoPropValue<Option<Rc<Vec<T>>>>) {
        let new_data = data.into_prop_value();
        if my_data_eq_fn(&self.data, &new_data) { return; }

        self.data = new_data;
        self.update_filtered_data();
    }

    pub fn lookup_record(&self, record_num: usize) -> Option<&T> {
        self.data.as_ref().map(|data| data.get(record_num)).flatten()
    }

    pub fn lookup_filtered_record(&self, cursor: usize) -> Option<(usize, &T)> {
        let n = match self.unfiltered_pos(cursor) {
            Some(n) => n,
            None => return None,
        };

        self.lookup_record(n).map(|item| (n, item))
    }

    /*
    pub fn extract_key(mut self, extract_fn: impl IntoExtractKeyFn<T>) -> Self {
        self.set_extract_key(extract_fn);
        self
    }

    pub fn set_extract_key(&mut self, extract_fn: impl IntoExtractKeyFn<T>) {
        self.extract_key = extract_fn.into_extract_key_fn();
    }
*/

    pub fn sorter(mut self, sorter: impl IntoSorterFn<T>) -> Self {
        self.set_sorter(sorter);
        self
    }

    pub fn set_sorter(&mut self, sorter: impl IntoSorterFn<T>) {
        self.sorter = sorter.into_sorter_fn();
        self.update_filtered_data();
    }

    pub fn filter(mut self, filter: impl IntoFilterFn<T>) -> Self {
        self.set_filter(filter);
        self
    }

    pub fn set_filter(&mut self, filter: impl IntoFilterFn<T>) {
        self.filter = filter.into_filter_fn();
        self.update_filtered_data();
    }

    pub fn filtered_data(&self) -> DataFilterIterator<T> {
        DataFilterIterator {
            range: None,
            pos: 0,
            data: self,
        }
    }

    pub fn filtered_data_range(&self, range: Range<usize>) -> DataFilterIterator<T> {
        DataFilterIterator {
            pos: range.start,
            range: Some(range),
            data: self,
        }
    }

    pub fn unfiltered_pos(&self, cursor: usize) -> Option<usize> {
        self.filtered_data.get(cursor).map(|n| *n)
    }

    pub fn filtered_pos(&self, record_num: usize) -> Option<usize> {
        self.filtered_data.iter().position(|n| *n == record_num)
    }

    pub fn filtered_data_len(&self) -> usize {
        self.filtered_data.len()
    }

    fn update_filtered_data(&mut self) {

        let old_cursor_record_num = self
            .cursor.map(|cursor| self.unfiltered_pos(cursor))
            .flatten();

        self.cursor = None;


        let data = match &self.data {
            None => {
                self.filtered_data = Vec::new();
                return;
            }
            Some(data) => data,
        };

        let mut filtered_data: Vec<(usize, &T)> = if let Some(filter) = &self.filter {
            data.iter().enumerate().filter(|(n, item)| {
                filter.apply(*n, item)
            }).collect()
        } else {
            data.iter().enumerate().collect()
        };

        if let Some(sorter) = &self.sorter {
            filtered_data.sort_by(|a, b| { sorter.cmp(a.1, b.1) });
        }

        self.filtered_data = filtered_data.into_iter().map(|item| item.0).collect();

        self.cursor = match old_cursor_record_num {
            Some(n) => self.filtered_pos(n),
            None => None,
        };
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
            Some(n) => if (n + 1) < len { Some(n + 1) }  else { None },
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
            Some(n) => if n > 0 { Some(n - 1) } else { None },
            None => Some(len - 1),
        };
    }
}

impl <'a, T> Iterator for DataFilterIterator<'a, T> {
    type Item = (usize, usize, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        let data = match &self.data.data {
            Some(data) => data,
            None => return None,
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
        let n = self.data.filtered_data[pos];
        self.pos += 1;

        Some((pos, n, &data[n]))
    }
}

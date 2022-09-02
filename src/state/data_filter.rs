use std::rc::Rc;

use yew::html::IntoPropValue;

use crate::props::{ExtractKeyFn, FilterFn, IntoFilterFn, SorterFn, IntoSorterFn};

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
    extract_key: Option<ExtractKeyFn<T>>,
    sorter: Option<SorterFn<T>>,
    filter: Option<FilterFn<T>>,
}

pub struct DataFilterIterator<'a, T> {
    data: &'a DataFilter<T>,
    pos: usize,
}

impl <T> DataFilter<T> {

    pub fn new() -> Self {
        Self {
            data: None,
            filtered_data: Vec::new(),
            extract_key: None,
            sorter: None,
            filter: None,
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

    pub fn extract_key(mut self, extract_fn: impl Into<ExtractKeyFn<T>>) -> Self {
        self.set_extract_key(extract_fn);
        self
    }

    pub fn set_extract_key(&mut self, extract_fn: impl Into<ExtractKeyFn<T>>) {
        self.extract_key = Some(extract_fn.into());
    }

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
            pos: 0,
            data: self,
        }
    }

    fn update_filtered_data(&mut self) {
        let data = match &self.data {
            None => {
                self.filtered_data = Vec::new();
                return;
            }
            Some(data) => data,
        };

        let mut filtered_data: Vec<(usize, &T)> = if let Some(filter) = &self.filter {
            data.iter().enumerate().filter(|(_n, item)| {
                filter.apply(item)
            }).collect()
        } else {
            data.iter().enumerate().collect()
        };

        if let Some(sorter) = &self.sorter {
            filtered_data.sort_by(|a, b| { sorter.cmp(a.1, b.1) });
        }

        self.filtered_data = filtered_data.into_iter().map(|item| item.0).collect();
    }
}

impl <'a, T> Iterator for DataFilterIterator<'a, T> {
    // we will be counting with usize
    type Item = &'a T;

    // next() is the only required method
    fn next(&mut self) -> Option<Self::Item> {
        let data = match &self.data.data {
            Some(data) => data,
            None => return None,
        };

        if self.data.filtered_data.len() <= self.pos {
            return None;
        }

        let n = self.data.filtered_data[self.pos];
        self.pos += 1;

        Some(&data[n])
    }
}

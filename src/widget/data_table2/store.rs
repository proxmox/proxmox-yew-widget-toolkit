use std::rc::Rc;
use std::cell::{Ref, RefMut, RefCell};
use std::ops::Range;
use std::ops::{Deref, DerefMut};

use derivative::Derivative;

use yew::virtual_dom::Key;

use crate::props::{FilterFn, IntoSorterFn, IntoFilterFn, SorterFn, ExtractKeyFn};
use crate::state::{DataCollection, DataNode, DataNodeDerefGuard};

use super::ExtractPrimaryKey;

/// Shared list store.
#[derive(Derivative)]
#[derivative(Clone(bound=""), PartialEq(bound=""))]
pub struct Store<T: 'static> {
    #[derivative(PartialEq(compare_with="store_state_equal::<T>"))]
    inner: Rc<RefCell<StoreState<T>>>,
}

impl<T: ExtractPrimaryKey + 'static> Store<T> {

    /// Creates a new instance for types implementing [ExtractPrimaryKey].
    ///
    /// Use [Self::with_extract_key] for types which does not
    /// implement [ExtractPrimaryKey].
    pub fn new() -> Self {
        let extract_key = ExtractKeyFn::new(|data: &T| data.extract_key());
        Self {
            inner: Rc::new(RefCell::new(StoreState::new(extract_key))),
        }
    }
}

impl<T: 'static> Store<T> {

    /// Creates a new instance with the specifies extract key function.
    pub fn with_extract_key(extract_key: impl Into<ExtractKeyFn<T>>) -> Self {
        Self {
            inner: Rc::new(RefCell::new(StoreState::new(extract_key.into()))),
        }
    }

    pub fn set_data(&self, data: Vec<T>) {
        self.inner.borrow_mut().set_data(data);
    }

    pub fn filtered_data<'a>(&'a self) -> Box<dyn Iterator<Item=(usize, Box<dyn DataNode<T> + 'a>)> + 'a> {
        Box::new(StoreIterator {
            range: None,
            pos: 0,
            state: self.inner.borrow(),
        })
    }

    pub fn read(&self) -> StoreReadGuard<T> {
        StoreReadGuard {
            state: self.inner.borrow(),
        }
    }

    pub fn write(&self) -> StoreWriteGuard<T> {
        StoreWriteGuard {
            state: self.inner.borrow_mut(),
        }
    }
}

pub struct StoreWriteGuard<'a, T> {
    state: RefMut<'a, StoreState<T>>,
}

impl<T> Deref for StoreWriteGuard<'_, T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.state.data
    }
}

impl<'a, T> DerefMut for StoreWriteGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state.data
    }
}

pub struct StoreReadGuard<'a, T> {
    state: Ref<'a, StoreState<T>>,
}

impl<T> Deref for StoreReadGuard<'_, T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.state.data
    }
}

pub struct StoreNodeRef<'a, T>(Ref<'a, T>);

impl<'a, T> DataNode<T> for StoreNodeRef<'a, T> {
    fn record(&self) -> DataNodeDerefGuard<T> {
        let guard: Box<dyn Deref<Target = T>> = Box::new(&*self.0);
        DataNodeDerefGuard { guard: guard }
    }
    fn level(&self) -> usize { 0 }
    fn expanded(&self) -> bool { false }
    fn parent(&self) -> Option<Box<dyn DataNode<T> + '_>> { None }
}

impl<T> DataCollection<T> for Store<T> {

    fn extract_key(&self, data: &T) -> Key {
        self.inner.borrow().extract_key.apply(data)
    }

    fn set_sorter(&self, sorter: impl IntoSorterFn<T>) {
        self.inner.borrow_mut().set_sorter(sorter);
    }

    fn set_filter(&self, filter: impl IntoFilterFn<T>) {
        self.inner.borrow_mut().set_filter(filter);
    }

    fn lookup_filtered_record_key(&self, cursor: usize) -> Option<Key> {
        self.inner.borrow().lookup_filtered_record_key(cursor)
    }

    fn filtered_record_pos(&self, key: &Key) -> Option<usize> {
        self.inner.borrow().filtered_record_pos(key)
    }

    fn filtered_data_len(&self) -> usize {
        self.inner.borrow().filtered_data_len()
    }

    fn filtered_data<'a>(
        &'a self,
    ) -> Box<dyn Iterator<Item=(usize, Box<dyn DataNode<T> + 'a>)> + 'a> {

        Box::new(StoreIterator {
            range: None,
            pos: 0,
            state: self.inner.borrow(),
         })
    }

    fn filtered_data_range<'a>(
        &'a self,
        range: Range<usize>,
    ) -> Box<dyn Iterator<Item=(usize, Box<dyn DataNode<T> + 'a>)> + 'a> {

        Box::new(StoreIterator {
            pos: range.start,
            range: Some(range),
            state: self.inner.borrow(),
        })
    }

    fn get_cursor(&self) -> Option<usize> {
        self.inner.borrow().get_cursor()
    }

    fn set_cursor(&self, cursor: Option<usize>) {
        self.inner.borrow_mut().set_cursor(cursor)
    }
}

fn store_state_equal<T>(
    me: &Rc<RefCell<StoreState<T>>>,
    other: &Rc<RefCell<StoreState<T>>>
) -> bool {
    Rc::ptr_eq(&me, &other) &&
        me.borrow().version == other.borrow().version
}

#[repr(transparent)]
struct Wrapper<'a, T>(&'a T);

pub struct StoreState<T> {
    extract_key: ExtractKeyFn<T>,

    version: usize,

    data: Vec<T>,

    filtered_data: Vec<usize>,

    sorter: Option<SorterFn<T>>,
    filter: Option<FilterFn<T>>,
    cursor: Option<usize>,
}

impl<T: 'static> StoreState<T> {

    fn new(extract_key: ExtractKeyFn<T>) -> Self {
        Self {
            version: 0,
            data: Vec::new(),
            extract_key,
            filtered_data: Vec::new(),
            sorter: None,
            filter: None,
            cursor: None,
        }
    }

    fn set_sorter(&mut self, sorter: impl IntoSorterFn<T>) {
        self.sorter = sorter.into_sorter_fn();
        self.update_filtered_data();
    }

    fn set_filter(&mut self, filter: impl IntoFilterFn<T>) {
        self.filter = filter.into_filter_fn();
        self.update_filtered_data();
    }

    fn set_data(&mut self, data: Vec<T>) {
        self.version += 1;
        self.data = data;
        self.update_filtered_data();
    }

    fn update_filtered_data(&mut self) {

        let old_cursor_record_key = if let Some(cursor) = self.cursor {
            self.lookup_filtered_record_key(cursor)
        } else {
            None
        };

        self.filtered_data = self.data.iter().enumerate()
            .filter(|(_, record)| match &self.filter {
                Some(filter) => filter.apply(0, record), // fixme: remove fiter record_num param
                None => true,
            })
            .map(|(n, _record)| n)
            .collect();

        if let Some(sorter) = &self.sorter {
            self.filtered_data.sort_by(|a, b| {
                sorter.cmp(&self.data[*a], &self.data[*b])
            });
        }

        self.cursor = match &old_cursor_record_key {
            Some(record_key) => self.filtered_record_pos(record_key),
            None => None,
        };
    }

    fn lookup_filtered_record_key(&self, cursor: usize) -> Option<Key> {
        let n = match self.filtered_data.get(cursor) {
            Some(n) => *n,
            None => return None,
        };

        let record = match self.data.get(n) {
            Some(record) => record,
            None => return None,
        };

        Some(self.extract_key.apply(record))
    }

    fn filtered_record_pos(&self, key: &Key) -> Option<usize> {
        self.filtered_data.iter()
            .position(|n| key == &self.extract_key.apply(&self.data[*n]))
    }

    fn filtered_data_len(&self) -> usize {
        self.filtered_data.len()
    }

    /*
    fn filtered_data<'a>(&'a self) -> Box<dyn Iterator<Item =(usize, Rc<dyn DataNode<T> + 'a>)> + 'a> {
        Box::new(self.filtered_data.iter().enumerate().map(|(i, node_id)| {
            let record = &self.data[*node_id];
            let wrapper: Rc<dyn DataNode<T>> = Rc::new(Wrapper(record));
            (i, wrapper)
        }))
    }
     */
    fn get_cursor(&self) -> Option<usize> {
        self.cursor
    }

    fn set_cursor(&mut self, cursor: Option<usize>) {
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
}

impl<'a, T> Deref for Wrapper<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.0
    }
}

impl<'a, T> DataNode<T> for Wrapper<'a, T> {
    fn record(&self) -> DataNodeDerefGuard<T> {
        let guard: Box<dyn Deref<Target = T>> = Box::new(self.0);
        DataNodeDerefGuard { guard: guard }
    }
    fn level(&self) -> usize { 0 }
    fn expanded(&self) -> bool { false }
    fn parent(&self) -> Option<Box<dyn DataNode<T> + '_>> { None }
}

pub struct StoreIterator<'a, T> {
    state: Ref<'a, StoreState<T>>,
    pos: usize,
    range: Option<Range<usize>>,
}

impl <'a, T: 'static> Iterator for StoreIterator<'a, T> where Self: 'a {
    type Item = (usize, Box<dyn DataNode<T> + 'a>);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(range) = &self.range {
            if range.end <= self.pos {
                return None;
            }
        }

        if self.state.filtered_data.len() <= self.pos {
            return None;
        }

        let pos = self.pos;
        self.pos += 1;

        let node_id = self.state.filtered_data[pos];

        let myref: Ref<'a, T> = Ref::map(Ref::clone(&self.state), |state| &state.data[node_id]);

        let node = Box::new(StoreNodeRef(myref));

        Some((pos, node))
    }
}

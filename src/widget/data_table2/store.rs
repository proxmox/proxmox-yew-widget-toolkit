use std::rc::Rc;
use std::cell::{Ref, RefMut, RefCell};
use std::ops::Range;
use std::ops::{Deref, DerefMut};

use derivative::Derivative;

use crate::props::ExtractKeyFn;
use crate::state::{DataCollection, DataNode, DataNodeDerefGuard};

use super::ExtractPrimaryKey;


/// Shared list store.
#[derive(Derivative)]
#[derivative(Clone(bound=""), PartialEq(bound=""))]
pub struct Store<T: 'static> {
    extract_key: ExtractKeyFn<T>,
    #[derivative(PartialEq(compare_with="store_state_equal::<T>"))]
    inner: Rc<RefCell<StoreState<T>>>,
}

impl<T: ExtractPrimaryKey + 'static> Store<T> {

    /// Creates a new instance for types implementing [ExtractPrimaryKey].
    ///
    /// Use [Self::with_extract_key] for types which does not
    /// implement [ExtractPrimaryKey].
    pub fn new() -> Self {
        Self {
            extract_key: ExtractKeyFn::new(|data: &T| data.extract_key()),
            inner: Rc::new(RefCell::new(StoreState::new())),
        }
    }
}

impl<T: 'static> Store<T> {

    /// Creates a new instance with the specifies extract key function.
    pub fn with_extract_key(extract_key: impl Into<ExtractKeyFn<T>>) -> Self {
        Self {
            extract_key: extract_key.into(),
            inner: Rc::new(RefCell::new(StoreState::new())),
        }
    }

    pub fn set_data(&self, data: Vec<T>) {
        self.inner.borrow_mut().set_data(data);
    }

    pub fn filtered_data<'a>(&'a self) -> Box<dyn Iterator<Item=(usize, Rc<dyn DataNode<T> + 'a>)> + 'a> {
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

/*
impl DataCollection<T> for Store<T> {

    fn filtered_data<'a>(&'a self) -> Box<dyn Iterator<Item=(usize, Rc<dyn DataNode<T>>)> + 'a> {
        Box::new(StoreIterator {
            range: None,
            pos: 0,
            state: self.inner.borrow(),
         })
    }

    fn filtered_data_range<'a>(&'a self, range: Range<usize>) -> Box<dyn Iterator<Item=(usize, Rc<dyn DataNode>>)> + 'a> {
        Box::new(StoreIterator {
            pos: range.start,
            range: Some(range),
            state: self.inner.borrow(),
        })
    }
}
 */

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
    version: usize,

    data: Vec<T>,

    filtered_data: Vec<usize>,
}

impl<T: 'static> StoreState<T> {

    fn new() -> Self {
        Self {
            version: 0,
            data: Vec::new(),
            filtered_data: Vec::new(),
        }
    }

    fn set_data(&mut self, data: Vec<T>) {
        self.version += 1;
        self.data = data;
        //self.filtered_data = data.into_iter().map(|item| Rc::new(Entry(item))).collect();
    }

    fn filtered_data<'a>(&'a self) -> Box<dyn Iterator<Item =(usize, Rc<dyn DataNode<T> + 'a>)> + 'a> {
        Box::new(self.filtered_data.iter().enumerate().map(|(i, node_id)| {
            let record = &self.data[*node_id];
            let wrapper: Rc<dyn DataNode<T>> = Rc::new(Wrapper(record));
            (i, wrapper)
        }))
    }

    /*
    fn filtered_data<'a>(&'a self) -> Box<dyn Iterator<Item=(usize, Rc<dyn DataNode<T>>)> + 'a> {
        Box::new(StoreStateIterator {
            range: None,
            pos: 0,
            state: self,
        })
    }
     */
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
    type Item = (usize, Rc<dyn DataNode<T> + 'a>);

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

        let node = Rc::new(StoreNodeRef(myref));

        Some((pos, node))
    }
}

use std::rc::Rc;
use std::cell::RefCell;
use std::ops::{Deref, Range};

use yew::virtual_dom::Key;

use crate::props::{IntoFilterFn, IntoSorterFn};

pub trait DataNode<T> {
    fn record(&self) -> DataNodeDerefGuard<T>;
    fn level(&self) -> usize;
    fn expanded(&self) -> bool;
    fn parent(&self) -> Option<Box<dyn DataNode<T> + '_>>;
}

pub trait DataCollection<T> {
    fn extract_key(&self, data: &T) -> Key;
    fn set_sorter(&mut self, sorter: impl IntoSorterFn<T>);
    fn set_filter(&mut self, filter: impl IntoFilterFn<T>);
    fn lookup_filtered_record_key(&self, cursor: usize) -> Option<Key>;
    fn filtered_record_pos(&self, key: &Key) -> Option<usize>;
    fn filtered_data_len(&self) -> usize;

    fn filtered_data<'a>(&'a self) ->
        Box<dyn Iterator<Item=(usize, Rc<dyn DataNode<T> + 'a>)> + 'a> { todo!(); }
    fn filtered_data_range<'a>(&'a self, range: Range<usize>) ->
        Box<dyn Iterator<Item=(usize, Rc<dyn DataNode<T> + 'a>)> + 'a>;

    // Cursor
    fn get_cursor(&self) -> Option<usize>;
    fn set_cursor(&mut self, cursor: Option<usize>);

    fn cursor_down(&mut self) {
        let len = self.filtered_data_len();
        if len == 0 {
            self.set_cursor(None);
            return;
        }
        self.set_cursor(match self.get_cursor() {
            Some(n) => if (n + 1) < len { Some(n + 1) }  else { Some(0) },
            None => Some(0),
        });
    }

    fn cursor_up(&mut self) {
        let len = self.filtered_data_len();
        if len == 0 {
            self.set_cursor(None);
            return;
        }

        self.set_cursor(match self.get_cursor() {
            Some(n) => if n > 0 { Some(n - 1) } else { Some(len - 1) },
            None => Some(len - 1),
        });
    }

}

pub struct DataNodeDerefGuard<'a, T> {
    pub(crate) guard: Box<(dyn Deref<Target=T> + 'a)>,
}

impl<T> Deref for DataNodeDerefGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.guard
     }
}

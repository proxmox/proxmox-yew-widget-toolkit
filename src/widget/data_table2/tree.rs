use std::rc::Rc;
use std::cell::RefCell;

use serde::{Serialize, Deserialize};

use super::{DataNode, DataNodeDerefGuard};

impl<T> DataNode<T> for RefCell<TreeNode<T>> {

    fn record(&self) -> DataNodeDerefGuard<T> {
        // Note: we use the Rc<T> as deref guard
        let record = self.borrow().record.clone();
        DataNodeDerefGuard { guard: Box::new(record) }
    }
  
    fn level(&self) -> usize {
        self.borrow().level
    }
    
    fn expanded(&self) -> bool {
        self.borrow().expanded
    }
    
    fn parent(&self) -> Option<Box<dyn DataNode<T> + '_>> {
        // fixme: howto implement?
        todo!();
    }
}

#[derive(Serialize, Deserialize)]
pub struct TreeNode<T> {
    pub record: Rc<T>,
    #[serde(default)]
    pub expanded: bool,
    #[serde(default)]
    pub children: Option<Vec<Rc<RefCell<TreeNode<T>>>>>,

    // Note: flatten_tree_children sets below attributes, so they are
    // only available in the filtered_data list.
    #[serde(skip)]
    pub parent: Option<usize>,
    #[serde(skip)]
    pub level: usize,
}

impl<T> AsRef<T> for TreeNode<T> {
    fn as_ref(&self) -> &T {
        &self.record
    }
}

impl<T> std::borrow::Borrow<T> for &TreeNode<T> {
    fn borrow(&self) -> &T {
        &self.record
    }
}

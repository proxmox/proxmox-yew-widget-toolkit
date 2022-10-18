use std::rc::Rc;
use serde::{Serialize, Deserialize};

//use yew::prelude::*;
//use yew::html::IntoPropValue;

#[derive(Serialize, Deserialize)]
pub struct TreeNode<T> {
    pub record: Rc<T>,
    #[serde(default)]
    pub expanded: bool,
    #[serde(default)]
    pub children: Option<Vec<Rc<TreeNode<T>>>>,
}

pub struct FlatTreeNode<T> {
    pub parent: Option<usize>,
    pub level: usize,
    pub node: Rc<TreeNode<T>>,
}

impl<T> AsRef<T> for FlatTreeNode<T> {
    fn as_ref(&self) -> &T {
        &self.node.record
    }
}

impl<T> std::borrow::Borrow<T> for &FlatTreeNode<T> {
    fn borrow(&self) -> &T {
        &self.node.record
    }
}

pub trait ToFlatNodeList<T> {
    fn to_flat_node_list(&self) -> Vec<FlatTreeNode<T>>;
}

impl<T> ToFlatNodeList<T> for &Rc<TreeNode<T>> {
    fn to_flat_node_list(&self) -> Vec<FlatTreeNode<T>> {
        flatten_tree(Rc::clone(self))
    }
}

impl<T> ToFlatNodeList<T> for &Vec<Rc<T>> {
    fn to_flat_node_list(&self) -> Vec<FlatTreeNode<T>> {
        self.iter().map(|record| {
            let node = TreeNode {
                record: record.clone(),
                expanded: false,
                children: None,
            };
            FlatTreeNode { parent: None, level: 0, node: Rc::new(node) }
        }).collect()
    }
}

impl<T> ToFlatNodeList<T> for Option<Rc<Vec<Rc<T>>>> {
    fn to_flat_node_list(&self) -> Vec<FlatTreeNode<T>> {
        match self {
            Some(list) => list.as_ref().to_flat_node_list(),
            None => Vec::new(),
        }
    }
}

fn flatten_tree_children<T>(
    list: &mut Vec<FlatTreeNode<T>>,
    level: usize,
    parent: Option<usize>,
    children: &[Rc<TreeNode<T>>],
) {
    for tree in children {
        list.push(FlatTreeNode { parent, level, node: tree.clone() });
        if let Some(children) = &tree.children {
            flatten_tree_children(list, level + 1, Some(list.len() - 1), children);
        }
    }
}

pub fn flatten_tree<T>(
    tree: Rc<TreeNode<T>>,
) -> Vec<FlatTreeNode<T>> {
    let mut list = Vec::new();
    flatten_tree_children(&mut list, 0, None, &[ tree ]);
    list
}

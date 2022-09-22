use derivative::Derivative;

use yew::prelude::*;
use yew::virtual_dom::Key;

use super::DataTableColumn;

#[derive(Derivative)]
#[derivative(Clone(bound=""), PartialEq(bound=""))]
pub enum Header<T: 'static> {
    Single(DataTableColumn<T>),
    Group(HeaderGroup<T>),
}

impl<T: 'static> From<DataTableColumn<T>> for Header<T> {
    fn from(column: DataTableColumn<T>) -> Self {
        Self::Single(column)
    }
}

impl<T: 'static> From<HeaderGroup<T>> for Header<T> {
    fn from(group: HeaderGroup<T>) -> Self {
        Self::Group(group)
    }
}

impl<T: 'static> Header<T> {

    pub(crate) fn extract_column_list(&self, list: &mut Vec<DataTableColumn<T>>) {
        match self {
            Header::Single(column) => list.push(column.clone()),
            Header::Group(group) => group.extract_column_list(list),
        }
    }
}

#[derive(Derivative)]
#[derivative(Clone(bound=""), PartialEq(bound=""))]
pub struct HeaderGroup<T: 'static> {
    /// The name dispayed in the header.
    pub name: AttrValue,
    /// Unique Column Key
    pub key: Option<Key>,
    pub children: Vec<Header<T>>,
}


impl<T: 'static> HeaderGroup<T> {

    /// Create a new instance.
    pub fn  new(name: impl Into<AttrValue>) -> Self {
        Self {
            name: name.into(),
            key: None,
            children: Vec::new()
        }
    }

    /// Builder style method to set the yew `key` property
    pub fn key(mut self, key: impl Into<Key>) -> Self {
        self.key = Some(key.into());
        self
    }

    pub fn with_child(mut self, header: impl Into<Header<T>>) -> Self {
        self.add_child(header);
        self
    }

    pub fn add_child(&mut self, header: impl Into<Header<T>>) {
        self.children.push(header.into())
    }

    /// Builder style method to add multiple children
    pub fn children(mut self, child: impl IntoIterator<Item = Header<T>>) -> Self {
        self.add_children(child);
        self
    }

    /// Method to add multiple children.
    pub fn add_children(&mut self, children: impl IntoIterator<Item = Header<T>>) {
        self.children.extend(children);
    }

    /// Method to set the children property.
    pub fn set_children(&mut self, children: impl IntoIterator<Item = Header<T>>) {
        self.children.clear();
        self.children.extend(children);
    }

    pub(crate) fn extract_column_list(&self, list: &mut Vec<DataTableColumn<T>>) {
        for child in &self.children {
            child.extract_column_list(list);
        }
    }
}

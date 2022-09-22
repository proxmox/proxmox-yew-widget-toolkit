use std::rc::Rc;
use std::ops::Range;
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


#[derive(Derivative)]
#[derivative(Clone(bound=""), PartialEq(bound=""))]
pub enum IndexedHeader<T: 'static> {
    Single(Rc<IndexedHeaderSingle<T>>),
    Group(Rc<IndexedHeaderGroup<T>>),
}

#[derive(Derivative)]
#[derivative(Clone(bound=""), PartialEq(bound=""))]
pub struct IndexedHeaderSingle<T: 'static> {
    pub cell_idx: usize,
    pub parent: Option<usize>,
    pub column: DataTableColumn<T>,
}

#[derive(Derivative)]
#[derivative(Clone(bound=""), PartialEq(bound=""))]
pub struct IndexedHeaderGroup<T: 'static> {
    pub cell_idx: usize,
    pub parent: Option<usize>,
    pub colspan: usize,
    pub cell_count: usize,
    /// The name dispayed in the header.
    pub name: AttrValue,
    /// Unique Column Key
    pub key: Option<Key>,
    pub children: Vec<IndexedHeader<T>>,
}

impl<T: 'static> IndexedHeaderGroup<T> {
    pub fn extract_column_list(&self, list: &mut Vec<DataTableColumn<T>>) {
        for child in &self.children {
            child.extract_column_list(list);
        }
    }
}

impl<T: 'static> IndexedHeader<T> {

    pub fn convert_header_list(
        list: &[Header<T>],
        cell_idx: usize,
        parent: Option<usize>,
    ) -> (Vec<IndexedHeader<T>>, usize, usize) {
        let mut span = 0;
        let mut cells = 0;
        let mut cell_idx = cell_idx;

        let mut indexed_list = Vec::new();

        for child in list {
            match child {
                Header::Single(column) => {
                    let cell = Self::convert_column(column, cell_idx, parent);
                    indexed_list.push(IndexedHeader::Single(Rc::new(cell)));
                    span += 1;
                    cell_idx += 1;
                    cells += 1;
                }
                Header::Group(group) => {
                    let indexed_group = Self::convert_group(group, cell_idx, parent);
                    span += indexed_group.colspan;
                    cell_idx += indexed_group.cell_count;
                    cells += indexed_group.cell_count;
                    indexed_list.push(Self::Group(Rc::new(indexed_group)));
                }
            }
        }
        (indexed_list, span, cells)
    }

    pub fn convert_column(column: &DataTableColumn<T>, cell_idx: usize, parent: Option<usize>) -> IndexedHeaderSingle<T> {
        IndexedHeaderSingle {
            cell_idx,
            parent,
            column: column.clone(),
        }
    }

    pub fn convert_group(group: &HeaderGroup<T>, cell_idx: usize, parent: Option<usize>) -> IndexedHeaderGroup<T> {

        let (children, span, cells) = Self::convert_header_list(&group.children, cell_idx + 1, Some(cell_idx));

        let cell_count = cells + 1;
        let colspan = span.max(1); // at least one column for the group header

        let indexed_group = IndexedHeaderGroup {
            parent,
            cell_idx,
            colspan,
            cell_count,
            name: group.name.clone(),
            key: group.key.clone(),
            children,
        };

        indexed_group
    }

    pub fn cell_range(&self) -> Range<usize> {
       match self {
           Self::Single(single) => single.cell_idx..(single.cell_idx+1),
           Self::Group(group) => group.cell_idx..(group.cell_idx + group.cell_count),
       }
    }

    pub fn extract_cell_list(&self, list: &mut Vec<IndexedHeader<T>>) {
        match self {
            Self::Single(single) => list.push(Self::Single(single.clone())),
            Self::Group(group) => {
                list.push(Self::Group(group.clone()));
                for child in &group.children {
                    child.extract_cell_list(list);
                }
            }
        }
    }

    pub fn extract_column_list(&self, list: &mut Vec<DataTableColumn<T>>) {
        match self {
            Self::Single(single) => list.push(single.column.clone()),
            Self::Group(group) => group.extract_column_list(list),
        }
    }

}

use std::rc::Rc;
use std::ops::Range;
use derivative::Derivative;

use yew::prelude::*;
use yew::virtual_dom::Key;

use super::DataTableColumn;

/// DataTable header definition.
///
/// This structure makes it possible to describe a nested header
/// hierachy, where a parent header can group one or more child
/// headers.
#[derive(Derivative)]
#[derivative(Clone(bound=""), PartialEq(bound=""))]
pub enum DataTableHeader<T: 'static> {
    Single(DataTableColumn<T>),
    Group(DataTableHeaderGroup<T>),
}

impl<T: 'static> From<DataTableColumn<T>> for DataTableHeader<T> {
    fn from(column: DataTableColumn<T>) -> Self {
        Self::Single(column)
    }
}

impl<T: 'static> From<DataTableHeaderGroup<T>> for DataTableHeader<T> {
    fn from(group: DataTableHeaderGroup<T>) -> Self {
        Self::Group(group)
    }
}

impl<T: 'static> DataTableHeader<T> {

    pub(crate) fn extract_column_list(&self, list: &mut Vec<DataTableColumn<T>>) {
        match self {
            DataTableHeader::Single(column) => list.push(column.clone()),
            DataTableHeader::Group(group) => group.extract_column_list(list),
        }
    }
}

/// Group of [headers](DataTableHeader).
#[derive(Derivative)]
#[derivative(Clone(bound=""), PartialEq(bound=""))]
pub struct DataTableHeaderGroup<T: 'static> {
    /// The name dispayed in the header.
    pub name: AttrValue,
    /// Unique Column Key
    pub key: Option<Key>,
    pub children: Vec<DataTableHeader<T>>,
    pub hidden: bool,
}


impl<T: 'static> DataTableHeaderGroup<T> {

    /// Create a new instance.
    pub fn  new(name: impl Into<AttrValue>) -> Self {
        Self {
            name: name.into(),
            key: None,
            hidden: false,
            children: Vec::new(),
        }
    }

    /// Builder style method to set the yew `key` property
    pub fn key(mut self, key: impl Into<Key>) -> Self {
        self.key = Some(key.into());
        self
    }

    pub fn with_child(mut self, header: impl Into<DataTableHeader<T>>) -> Self {
        self.add_child(header);
        self
    }

    pub fn add_child(&mut self, header: impl Into<DataTableHeader<T>>) {
        self.children.push(header.into())
    }

    /// Builder style method to add multiple children
    pub fn children(mut self, child: impl IntoIterator<Item = DataTableHeader<T>>) -> Self {
        self.add_children(child);
        self
    }

    /// Method to add multiple children.
    pub fn add_children(&mut self, children: impl IntoIterator<Item = DataTableHeader<T>>) {
        self.children.extend(children);
    }

    /// Method to set the children property.
    pub fn set_children(&mut self, children: impl IntoIterator<Item = DataTableHeader<T>>) {
        self.children.clear();
        self.children.extend(children);
    }

    /// Builder style method to set the hidden flag.
    pub fn hidden(mut self, hidden: bool) -> Self {
        self.set_hidden(hidden);
        self
    }

    /// Method to set the hidden flag.
    pub fn set_hidden(&mut self, hidden: bool) {
        self.hidden = hidden;
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
    pub start_col: usize,
    pub parent: Option<usize>,
    pub column: DataTableColumn<T>,
}

#[derive(Derivative)]
#[derivative(Clone(bound=""), PartialEq(bound=""))]
pub struct IndexedHeaderGroup<T: 'static> {
    pub cell_idx: usize,
    pub start_col: usize,
    pub parent: Option<usize>,
    pub colspan: usize,
    pub cell_count: usize,

    pub name: AttrValue,
    pub key: Option<Key>,
    pub children: Vec<IndexedHeader<T>>,
    pub hidden: bool,
}

impl<T: 'static> IndexedHeaderGroup<T> {
    pub fn extract_column_list(&self, list: &mut Vec<Rc<IndexedHeaderSingle<T>>>) {
        for child in &self.children {
            child.extract_column_list(list);
        }
    }
}

pub fn create_indexed_header_list<T: 'static>(list: &[DataTableHeader<T>]) -> Vec<IndexedHeader<T>> {
    IndexedHeader::convert_header_list(list, 0, 0, None).0
}

impl<T: 'static> IndexedHeader<T> {

    pub fn convert_header_list(
        list: &[DataTableHeader<T>],
        cell_idx: usize,
        start_col: usize,
        parent: Option<usize>,
    ) -> (Vec<IndexedHeader<T>>, usize, usize) {
        let mut span = 0;
        let mut cells = 0;
        let mut cell_idx = cell_idx;

        let mut indexed_list = Vec::new();

        for child in list {
            match child {
                DataTableHeader::Single(column) => {
                    let cell = Self::convert_column(column, cell_idx, start_col + span, parent);
                    indexed_list.push(IndexedHeader::Single(Rc::new(cell)));
                    span += 1;
                    cell_idx += 1;
                    cells += 1;
                }
                DataTableHeader::Group(group) => {
                    let indexed_group = Self::convert_group(group, cell_idx, start_col + span, parent);
                    span += indexed_group.colspan;
                    cell_idx += indexed_group.cell_count;
                    cells += indexed_group.cell_count;
                    indexed_list.push(Self::Group(Rc::new(indexed_group)));
                }
            }
        }
        (indexed_list, span, cells)
    }

    pub fn convert_column(
        column: &DataTableColumn<T>,
        cell_idx: usize,
        start_col: usize,
        parent: Option<usize>,
    ) -> IndexedHeaderSingle<T> {
        IndexedHeaderSingle {
            cell_idx,
            start_col,
            parent,
            column: column.clone(),
        }
    }

    pub fn convert_group(
        group: &DataTableHeaderGroup<T>,
        cell_idx: usize,
        start_col: usize,
        parent: Option<usize>,
    ) -> IndexedHeaderGroup<T> {

        let (children, span, cells) = Self::convert_header_list(
            &group.children,
            cell_idx + 1,
            start_col,
            Some(cell_idx),
        );

        let cell_count = cells + 1;
        let colspan = span.max(1); // at least one column for the group header

        let indexed_group = IndexedHeaderGroup {
            parent,
            cell_idx,
            start_col,
            colspan,
            cell_count,
            name: group.name.clone(),
            key: group.key.clone(),
            hidden: group.hidden,
            children,
        };

        indexed_group
    }

    pub fn lookup_cell(headers: &[IndexedHeader<T>], cell_idx: usize) -> Option<&IndexedHeader<T>> {
        for header in headers {
            match header {
                IndexedHeader::Single(single) => {
                    if single.cell_idx == cell_idx { return Some(header); }
                }
                IndexedHeader::Group(group) => {
                    if group.cell_idx == cell_idx { return Some(header); }
                    if let Some(cell) = Self::lookup_cell(&group.children, cell_idx) {
                        return Some(cell);
                    }
                }
            }
        }
        None
    }

    pub fn parent(&self) -> Option<usize> {
        match self {
            Self::Single(single) => single.parent,
            Self::Group(group) => group.parent,
        }
    }

    pub fn cell_idx(&self) -> usize {
        match self {
            Self::Single(single) => single.cell_idx,
            Self::Group(group) => group.cell_idx,
        }
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

    pub fn extract_column_list(&self, list: &mut Vec<Rc<IndexedHeaderSingle<T>>>) {
        match self {
            Self::Single(single) => list.push(Rc::clone(&single)),
            Self::Group(group) => group.extract_column_list(list),
        }
    }

}

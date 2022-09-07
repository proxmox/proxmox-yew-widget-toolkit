use derivative::Derivative;
use yew::virtual_dom::VNode;
use yew::html::IntoPropValue;

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
    /*
    fn grid_size(&self) -> (usize, usize) /* (rows, cols) */ {
        match self {
            Header::Single(_header) => (1, 1),
            Header::Group(group) => group.grid_size(),
        }
    }
     */
}

#[derive(Derivative)]
#[derivative(Clone(bound=""), PartialEq(bound=""))]
pub struct HeaderGroup<T: 'static> {
    pub content: Option<VNode>,
    pub children: Vec<Header<T>>,
}


impl<T: 'static> HeaderGroup<T> {

    /// Create a new instance.
    pub fn  new() -> Self {
        Self { content: None, children: Vec::new() }
    }

    /// Builder style method to set the header text
    pub fn content(mut self, content: impl IntoPropValue<Option<VNode>>) -> Self {
        self.set_content(content);
        self
    }

    /// Method to set the header text
    pub fn set_content(&mut self, content: impl IntoPropValue<Option<VNode>>) {
        self.content = content.into_prop_value();
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

    /*
    fn grid_size(&self) -> (usize, usize) /* (rows, cols) */ {
        let mut rows = 0;
        let mut cols = 0;

        for child in &self.children {
            let (child_rows, child_cols) = child.grid_size();
            cols += child_cols;
            if child_rows > rows { rows = child_rows; }
        }

        if self.content.is_some() {
            rows += 1;
        }

        (rows, cols)
    }
     */
}

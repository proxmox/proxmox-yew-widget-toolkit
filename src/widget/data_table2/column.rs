use derivative::Derivative;
use yew::prelude::*;
use yew::html::IntoPropValue;
use yew::virtual_dom::Key;

use crate::props::{SorterFn, IntoSorterFn, RenderFn};
use super::TreeNode;

#[derive(Properties)]
#[derive(Derivative)]
#[derivative(Clone(bound=""), PartialEq(bound=""))]
pub struct DataTableColumn<T> {
    /// Width passed to CSS grid-template-columns.
    #[prop_or(AttrValue::Static("auto"))]
    pub width: AttrValue,
    /// The name dispayed in the header.
    pub name: AttrValue,
    /// Unique Column Key
    pub key: Option<Key>,
    /// Horizontal table cell justification.
    #[prop_or(AttrValue::Static("left"))]
    pub justify: AttrValue, // left, center, right, justify
    /// Render function (returns cell content)
    pub render_node: RenderFn<TreeNode<T>>,
    /// Sorter function.
    ///
    /// Need to be set to enable column sorting.
    pub sorter: Option<SorterFn<T>>,
    /// Sort order
    ///
    /// - `Some(true)`: Ascending
    /// - `Some(false)`: Descending
    /// - `None`: do not sort this columns
    pub sort_order: Option<bool>,
    /// Hide column
    #[prop_or_default]
    pub hidden: bool,
}

impl<T: 'static> DataTableColumn<T> {

    /// Creates a new instance.
    pub fn new(name: impl Into<AttrValue>) -> Self {
        yew::props!(Self {
            name: name.into(),
            render_node: RenderFn::new(|_| html!{ "-" }),
        })
    }

    /// Builder style method to set the yew `key` property
    pub fn key(mut self, key: impl Into<Key>) -> Self {
        self.key = Some(key.into());
        self
    }

    /// Builder style method to set the column width.
    pub fn width(mut self, width: impl Into<AttrValue>) -> Self {
        self.set_width(width);
        self
    }

    /// Method to set the column width.
    pub fn set_width(&mut self, width: impl Into<AttrValue>) {
        self.width = width.into();
    }

    /// Builder style method to set the column width as flex fraction.
    pub fn flex(mut self, flex: usize) -> Self {
        self.set_flex(flex);
        self
    }

    /// Method to set the column width as flex fraction.
    pub fn set_flex(&mut self, flex: usize) {
        self.set_width(format!("{flex}fr"));
    }

    /// Builder style method to set a fixed column width.
    pub fn fixed(mut self, size: usize) -> Self {
        self.set_fixed(size);
        self
    }

    /// Method to set a fixed column width.
    pub fn set_fixed(&mut self, size: usize) {
        self.set_width(format!("{size}px"));
    }

    /// Builder style method to set the column width as percentage.
    pub fn percentage(mut self, percentage: usize) -> Self {
        self.set_percentage(percentage);
        self
    }

    /// Method to set the column width as percentage.
    pub fn set_percentage(&mut self, percentage: usize) {
        self.set_width(format!("{percentage}%"));
    }

    /// Builder style method to set the horizontal cell justification.
    pub fn justify(mut self, justify: impl Into<AttrValue>) -> Self {
        self.set_justify(justify);
        self
    }

    /// Method to set the horizontal cell justification.
    pub fn set_justify(&mut self, justify: impl Into<AttrValue>) {
        self.justify = justify.into();
    }

    /// Builder style method to set the render function.
    pub fn render(mut self, render: impl Into<RenderFn<T>>) -> Self {
        let render = render.into();
        self.render_node(move |node: &TreeNode<T>| {
            let data: &T = node.as_ref();
            render.apply(data)
        })
    }

    /// Builder style method to set the render function.
    pub fn render_node(mut self, render: impl Into<RenderFn<TreeNode<T>>>) -> Self {
        self.render_node = render.into();
        self
    }

    /// Builder style method to set the sort function.
    pub fn sorter(mut self, sorter: impl IntoSorterFn<T>) -> Self {
        self.sorter = sorter.into_sorter_fn();
        self
    }

    /// Builder style method to set the sort order
    pub fn sort_order(mut self, order: impl IntoPropValue<Option<bool>>) -> Self {
        self.sort_order = order.into_prop_value();
        self
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
}

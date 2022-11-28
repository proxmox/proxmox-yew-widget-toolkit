use std::rc::Rc;

use derivative::Derivative;
use indexmap::IndexMap;
use yew::prelude::*;
use yew::html::{IntoPropValue, IntoEventCallback};

use yew::virtual_dom::Key;

use crate::props::{SorterFn, IntoSorterFn, RenderFn};
use crate::state::DataNode;

use super::DataTableMouseEvent;

/// Column render function arguments.
pub struct DataTableColumnRenderArgs<'a, T> {
    // The data node.
    pub(crate) node: &'a dyn DataNode<T>,
    // Row index.
    pub(crate) row_index: usize,
    // Column index.
    pub(crate) column_index: usize,
    // Select flag is set when the cell is selected.
    pub(crate) selected: bool,

    /// Cell class. This attribute may be modified to change the
    /// appearance of the cell.
    pub class: Classes,
    /// Additional cell attributes (style, colspan, ...)
    pub attributes: IndexMap<AttrValue, AttrValue>,
}

impl<'a, T> DataTableColumnRenderArgs<'a, T> {

    /// Return the data node.
    pub fn node(&self) -> &dyn DataNode<T> {
        self.node
    }

    /// Returns the row index.
    pub fn row_index(&self) -> usize {
        self.row_index
    }

    /// Returns the column index.
    pub fn columns_index(&self) -> usize {
        self.column_index
    }

    /// Returns if the cell is selected.
    pub fn is_selected(&self) -> bool {
        self.selected
    }

    /// Method to set additional html attributes on the table cell
    ///
    /// Value 'None' removes the attribute.
    pub fn set_attribute(
        &mut self,
        key: impl Into<AttrValue>,
        value: impl IntoPropValue<Option<AttrValue>>,
    ) {
        if let Some(value) = value.into_prop_value() {
            self.attributes.insert(key.into(), value);
        } else {
            self.attributes.remove(&key.into());
        }
    }

    /// Method to add a CSS class to the table cell
    pub fn add_class(&mut self, class: impl Into<Classes>) {
        self.class.push(class);
    }
}

#[derive(Derivative)]
#[derivative(Clone(bound=""), PartialEq(bound=""))]
pub struct DataTableRenderCell<T>(
    #[derivative(PartialEq(compare_with="Rc::ptr_eq"))]
    Rc<dyn Fn(&mut DataTableColumnRenderArgs<T>) -> Html>
);

impl<T> DataTableRenderCell<T> {
    /// Creates a new instance.
    pub fn new(renderer: impl 'static + Fn(&mut DataTableColumnRenderArgs<T>) -> Html) -> Self {
        Self(Rc::new(renderer))
    }
    /// Apply the render function
    pub fn apply(&self, args: &mut DataTableColumnRenderArgs<T>) -> Html {
        (self.0)(args)
    }
}

impl<T, F: 'static + Fn(&mut DataTableColumnRenderArgs<T>) -> Html> From<F> for DataTableRenderCell<T> {
    fn from(f: F) -> Self {
        DataTableRenderCell::new(f)
    }
}

/// DataTable column properties.
#[derive(Properties)]
#[derive(Derivative)]
#[derivative(Clone(bound=""), PartialEq(bound=""))]
pub struct DataTableColumn<T> {
    /// Width passed to CSS grid-template-columns.
    #[prop_or(AttrValue::Static("auto"))]
    pub width: AttrValue,
    /// The name dispayed in the header.
    pub name: AttrValue,
    /// Optional ARIA label.
    pub aria_label: Option<AttrValue>,
    /// Unique Column Key
    pub key: Option<Key>,
    /// Horizontal table cell justification.
    #[prop_or(AttrValue::Static("left"))]
    pub justify: AttrValue, // left, center, right, justify
    /// Render function (returns cell content)
    pub render_cell: DataTableRenderCell<T>,
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

    /// Cell click callback (parameter is the record key.)
    pub on_cell_click: Option<Callback<DataTableMouseEvent>>,

}

impl<T: 'static> DataTableColumn<T> {

    /// Creates a new instance.
    pub fn new(name: impl Into<AttrValue>) -> Self {
        yew::props!(Self {
            name: name.into(),
            render_cell: DataTableRenderCell::new(|_| html!{ "-" }),
        })
    }

    /// Builder style method to set the yew `key` property
    pub fn key(mut self, key: impl Into<Key>) -> Self {
        self.key = Some(key.into());
        self
    }

    /// Builder style method to set the html aria-label attribute
    pub fn aria_label(mut self, label: impl IntoPropValue<Option<AttrValue>>) -> Self {
        self.set_aria_label(label);
        self
    }

    /// Method to set the html aria-label attribute
    pub fn set_aria_label(&mut self, label: impl IntoPropValue<Option<AttrValue>>) {
        self.aria_label = label.into_prop_value();
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
    pub fn render(self, render: impl Into<RenderFn<T>>) -> Self {
        let render = render.into();
        self.render_cell(move |args: &mut DataTableColumnRenderArgs<T>| {
            render.apply(&*args.node.record())
        })
    }

    /// Builder style method to set the cell render function.
    pub fn render_cell(mut self, render: impl Into<DataTableRenderCell<T>>) -> Self {
        self.render_cell = render.into();
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

    /// Builder style method to set the cell click callback.
    pub fn on_cell_click(mut self, cb: impl IntoEventCallback<DataTableMouseEvent>) -> Self {
        self.on_cell_click = cb.into_event_callback();
        self
    }
}

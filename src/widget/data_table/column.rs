use derivative::Derivative;

use yew::prelude::*;
use yew::html::IntoPropValue;

use yew::virtual_dom::Key;

use crate::props::{
    SorterFn, IntoSorterFn, RenderFn,
    CallbackMut, IntoEventCallbackMut,
};

use super::{
    DataTableKeyboardEvent, DataTableHeaderKeyboardEvent, DataTableMouseEvent,
    DataTableCellRenderer, DataTableCellRenderArgs, DataTableHeaderRenderer,
};

/// DataTable column properties.
#[derive(Properties)]
#[derive(Derivative)]
#[derivative(Clone(bound=""), PartialEq(bound=""))]
pub struct DataTableColumn<T: 'static> {
    /// Width passed to CSS grid-template-columns.
    #[prop_or(AttrValue::Static("auto"))]
    pub width: AttrValue,
    /// The name dispayed in the header (Also used as aria-label).
    pub name: AttrValue,
    /// Unique Column Key
    pub key: Option<Key>,
    /// Horizontal table cell justification.
    #[prop_or(AttrValue::Static("left"))]
    pub justify: AttrValue, // left, center, right, justify
    /// Render function (returns cell content)
    pub render_cell: DataTableCellRenderer<T>,
    /// Rendert function for Header content. If set, this is used instead of `name`.
    pub render_header: Option<DataTableHeaderRenderer<T>>,
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
    /// Resizable flag.
    #[prop_or(true)]
    pub resizable: bool,

    /// Show menu flag.
    #[prop_or(true)]
    pub show_menu: bool,

    /// Cell click callback
    pub on_cell_click: Option<CallbackMut<DataTableMouseEvent>>,
    /// Cell keydown callback
    pub on_cell_keydown: Option<CallbackMut<DataTableKeyboardEvent>>,

    pub on_header_keydown: Option<CallbackMut<DataTableHeaderKeyboardEvent<T>>>,
}

impl<T: 'static> DataTableColumn<T> {

    /// Creates a new instance.
    pub fn new(name: impl Into<AttrValue>) -> Self {
        yew::props!(Self {
            name: name.into(),
            render_cell: DataTableCellRenderer::new(|_| html!{ "-" }),
        })
    }

    /// Genertates a column which shows a checkbox indication the
    /// selection status.
    pub fn selection_indicator() -> Self {
        Self::new ("selection indicator")
            .width("2.5em")
            .resizable(false)
            .show_menu(false)
            .render_header(super::render_selection_header)
            .render_cell(super::render_selection_indicator)
            .on_header_keydown(|event: &mut DataTableHeaderKeyboardEvent<T>| {
                if event.key() == " " {
                    event.stop_propagation();
                    event.prevent_default();
                    event.send_toggle_select_all();
                }
            })
            .on_cell_keydown(|event: &mut DataTableKeyboardEvent| {
                if event.key() == " " {
                    event.stop_propagation();
                    event.prevent_default();
                    if let Some(selection) = &event.selection {
                        selection.toggle(event.record_key.clone());
                    }
                }
            })
    }

    /// Genertates a column which shows the now number.
    pub fn row_number() -> Self {
        Self::new("Row")
            .fixed(60)
            .render_cell(super::render_row_number)
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
    pub fn render(self, render: impl Into<RenderFn<T>>) -> Self {
        let render = render.into();
        self.render_cell(move |args: &mut DataTableCellRenderArgs<T>| {
            render.apply(args.record)
        })
    }

    /// Builder style method to set the cell render function.
    pub fn render_cell(mut self, render: impl Into<DataTableCellRenderer<T>>) -> Self {
        self.render_cell = render.into();
        self
    }

    /// Builder style method to set the header render function
    pub fn render_header(mut self, render: impl Into<DataTableHeaderRenderer<T>>) -> Self {
        self.render_header = Some(render.into());
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

    /// Builder style method to set the resizable flag.
    pub fn resizable(mut self, hidden: bool) -> Self {
        self.set_resizable(hidden);
        self
    }

    /// Method to set the resizable flag.
    pub fn set_resizable(&mut self, resizable: bool) {
        self.resizable = resizable;
    }

    /// Builder style method to set the show_menu flag.
    pub fn show_menu(mut self, hidden: bool) -> Self {
        self.set_show_menu(hidden);
        self
    }

    /// Method to set the show_menu flag.
    pub fn set_show_menu(&mut self, show_menu: bool) {
        self.show_menu = show_menu;
    }

    /// Builder style method to set the cell click callback.
    pub fn on_cell_click(mut self, cb: impl IntoEventCallbackMut<DataTableMouseEvent>) -> Self {
        self.on_cell_click = cb.into_event_cb_mut();
        self
    }

    /// Builder style method to set the cell keydown callback.
    pub fn on_cell_keydown(mut self, cb: impl IntoEventCallbackMut<DataTableKeyboardEvent>) -> Self {
        self.on_cell_keydown = cb.into_event_cb_mut();
        self

    }

    /// Builder style method to set the header keydown callback.
    pub fn on_header_keydown(mut self, cb: impl IntoEventCallbackMut<DataTableHeaderKeyboardEvent<T>>) -> Self {
        self.on_header_keydown = cb.into_event_cb_mut();
        self
    }
}

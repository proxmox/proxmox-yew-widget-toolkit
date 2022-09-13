use derivative::Derivative;
use yew::prelude::*;

use crate::props::{SorterFn, IntoSorterFn, RenderFn};

#[derive(Clone, PartialEq)]
pub enum DataTableColumnWidth {
    Auto,
    Fixed(usize),
    Flex(usize),
}

impl From<usize> for DataTableColumnWidth {
    fn from(value: usize) -> Self {
        Self::Fixed(value)
    }
}

#[derive(Properties)]
#[derive(Derivative)]
#[derivative(Clone(bound=""), PartialEq(bound=""))]
pub struct DataTableColumn<T> {
    #[prop_or(DataTableColumnWidth::Auto)]
    pub width: DataTableColumnWidth,
    pub name: String,
    #[prop_or(String::from("left"))]
    pub justify: String, // left, center, right, justify
    pub render: RenderFn<T>,
    pub sorter: Option<SorterFn<T>>,
}

impl<T> DataTableColumn<T> {

    pub fn new(name: impl Into<String>) -> Self {
        yew::props!(Self {
            name: name.into(),
            render: RenderFn::new(|_| html!{ "-" }),
        })
    }

    pub fn width(mut self, width: impl Into<DataTableColumnWidth>) -> Self {
        self.set_width(width);
        self
    }

    pub fn set_width(&mut self, width: impl Into<DataTableColumnWidth>) {
        self.width = width.into();
    }

    pub fn flex(mut self, flex: usize) -> Self {
        self.set_flex(flex);
        self
    }

    pub fn set_flex(&mut self, flex: usize) {
        self.set_width(DataTableColumnWidth::Flex(flex));
    }

    pub fn justify(mut self, justify: impl Into<String>) -> Self {
        self.set_justify(justify);
        self
    }

    pub fn set_justify(&mut self, justify: impl Into<String>) {
        self.justify = justify.into();
    }

    pub fn render(mut self, render: impl Into<RenderFn<T>>) -> Self {
        self.render = render.into();
        self
    }

    pub fn sorter(mut self, sorter: impl IntoSorterFn<T>) -> Self {
        self.sorter = sorter.into_sorter_fn();
        self
    }
}

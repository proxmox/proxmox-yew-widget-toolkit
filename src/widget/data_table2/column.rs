use derivative::Derivative;
use yew::prelude::*;

use crate::props::{SorterFn, IntoSorterFn, RenderFn};


#[derive(Properties)]
#[derive(Derivative)]
#[derivative(Clone(bound=""), PartialEq(bound=""))]
pub struct DataTableColumn<T> {
    #[prop_or(String::from("100px"))]
    pub width: String,
    pub name: String,
    #[prop_or(String::from("flex-start"))]
    pub justify: String, // flex-start, flex-end, center
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

    pub fn width(mut self, width: impl Into<String>) -> Self {
        self.set_width(width);
        self
    }

    pub fn set_width(&mut self, width: impl Into<String>) {
        self.width = width.into();
    }

    pub fn flex(mut self, flex: usize) -> Self {
        self.set_flex(flex);
        self
    }

    pub fn set_flex(&mut self, flex: usize) {
        self.set_width(format!("{}fr", flex));
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

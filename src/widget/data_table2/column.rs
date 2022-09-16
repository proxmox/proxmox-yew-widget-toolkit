use std::cmp::Ordering;

use derivative::Derivative;
use yew::prelude::*;

use crate::props::{SorterFn, IntoSorterFn, RenderFn};

#[derive(Properties)]
#[derive(Derivative)]
#[derivative(Clone(bound=""), PartialEq(bound=""))]
pub struct DataTableColumn<T> {
    #[prop_or(AttrValue::Static("auto"))]
    pub width: AttrValue,
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

    pub fn width(mut self, width: impl Into<AttrValue>) -> Self {
        self.set_width(width);
        self
    }

    pub fn set_width(&mut self, width: impl Into<AttrValue>) {
        self.width = width.into();
    }

    pub fn flex(mut self, flex: usize) -> Self {
        self.set_flex(flex);
        self
    }

    pub fn set_flex(&mut self, flex: usize) {
        self.set_width(format!("{flex}fr"));
    }

    pub fn fixed(mut self, size: usize) -> Self {
        self.set_fixed(size);
        self
    }

    pub fn set_fixed(&mut self, size: usize) {
        self.set_width(format!("{size}px"));
    }

    pub fn percentage(mut self, percentage: usize) -> Self {
        self.set_percentage(percentage);
        self
    }

    pub fn set_percentage(&mut self, percentage: usize) {
        self.set_width(format!("{percentage}%"));
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

pub(crate) fn create_combined_sorter_fn<T: 'static>(
    sorters: &[(usize, bool)],
    columns: &[DataTableColumn<T>]
) -> SorterFn<T> {
    let sorters: Vec<(SorterFn<T>, bool)> = sorters
        .iter()
        .filter_map(|(sort_idx, ascending)| {
            match &columns[*sort_idx].sorter {
                None => None,
                Some(sorter) => Some((sorter.clone(), *ascending)),
            }
        })
        .collect();

    SorterFn::new(move |a: &T, b: &T| {
        for (sort_fn, ascending) in &sorters {
            match if *ascending {
                sort_fn.cmp(a, b)
            } else {
                sort_fn.cmp(b, a)
            } {
                Ordering::Equal => { /* continue */ },
                other => return other,
            }
        }
        Ordering::Equal
    })
}

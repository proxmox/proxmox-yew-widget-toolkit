use std::cmp::Ordering;

use derivative::Derivative;
use yew::prelude::*;

use crate::props::{SorterFn, IntoSorterFn, RenderFn};

#[derive(Properties)]
#[derive(Derivative)]
#[derivative(Clone(bound=""), PartialEq(bound=""))]
pub struct DataTableColumn<T> {
    /// Width passed to CSS grid-template-columns.
    #[prop_or(AttrValue::Static("auto"))]
    pub width: AttrValue,
    /// The name dispayed in the header.
    pub name: String,
    /// Horizontal table cell justification.
    #[prop_or(String::from("left"))]
    pub justify: String, // left, center, right, justify
    /// Render function (returns cell content)
    pub render: RenderFn<T>,
    /// Sorter function.
    ///
    /// Need to be set to enable column sorting.
    pub sorter: Option<SorterFn<T>>,
}

impl<T> DataTableColumn<T> {

    /// Creates a new instance.
    pub fn new(name: impl Into<String>) -> Self {
        yew::props!(Self {
            name: name.into(),
            render: RenderFn::new(|_| html!{ "-" }),
        })
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
    pub fn justify(mut self, justify: impl Into<String>) -> Self {
        self.set_justify(justify);
        self
    }

    /// Method to set the horizontal cell justification.
    pub fn set_justify(&mut self, justify: impl Into<String>) {
        self.justify = justify.into();
    }

    /// Builder style method to set the render function.
    pub fn render(mut self, render: impl Into<RenderFn<T>>) -> Self {
        self.render = render.into();
        self
    }

    /// Builder style method to set the sort function.
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

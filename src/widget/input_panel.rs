use std::borrow::Cow;

use yew::prelude::*;
use yew::virtual_dom::{Listeners, VList, VTag};
use yew::html::IntoPropValue;

use pwt_macros::widget;

use crate::prelude::*;

/// Layout widget for forms with one or two columns.
///
/// This container show input fields with labels at different regions
/// (left, right, advanced).
#[widget(pwt=crate, @element, @container)]
#[derive(Properties, PartialEq, Clone)]
pub struct InputPanel {
    /// Spacing between fields
    #[prop_or_default]
    pub gap: usize,
    /// Flag to show the advanced region.
    #[prop_or_default]
    pub show_advanced: bool,
    /// Flag to show two columns (usually autodetected).
    #[prop_or_default]
    two_column: bool, // autodetected

    #[prop_or_default]
    left_count: usize,
    #[prop_or_default]
    right_count: usize,
}

impl InputPanel {

    /// Creates a new instance.
    pub fn new() -> Self {
        yew::props!(Self {})
    }

    /// Builder style method to set the field spacing.
    pub fn gap(mut self, gap: usize) -> Self {
        self.set_gap(gap);
        self
    }

    /// Method to set the field spacing.
    pub fn set_gap(&mut self, gap: usize) {
        self.gap = gap;
    }

    /// Builder style method to set the show_advanced flag.
    pub fn show_advanced(mut self, show_advanced: bool) -> Self {
        self.set_show_advanced(show_advanced);
        self
    }

    /// Method to set the show_advanced flag.
    pub fn set_show_advanced(&mut self, show_advanced: bool) {
        self.show_advanced = show_advanced;
    }

    pub fn with_spacer(mut self) -> Self {
        self.add_spacer(false);
        self
    }

    pub fn with_advanced_spacer(mut self) -> Self {
        self.add_spacer(true);
        self
    }

    pub fn add_spacer(&mut self, advanced: bool) {
        self.left_count += 1;
        self.right_count += 1;

        let visible = if advanced { self.show_advanced } else { true };

        let style = if visible {
            format!("grid-row: {}; grid-column-start: 1; grid-column-end: -1;", self.left_count)
        } else {
            format!("grid-row: {}; display: none;", self.left_count)
        };

        self.add_child(html!{
            <hr class="pwt-w-100" {style}/>
        });
    }

    /// Builder style method to add a field with label at the left column.
    pub fn with_field(
        mut self,
        label: impl IntoPropValue<AttrValue>,
        field: impl FieldBuilder,
    ) -> Self {
        self.add_field(false, label, field);
        self
    }

    pub fn with_advanced_field(
        mut self,
        label: impl IntoPropValue<AttrValue>,
        field: impl FieldBuilder,
    ) -> Self {
        self.add_field(true, label, field);
        self
    }

    /// Method to add a field with label at the left column.
    pub fn add_field(
        &mut self,
        advanced: bool,
        label: impl IntoPropValue<AttrValue>,
        field: impl FieldBuilder,
    ) {
        self.left_count += 1;

        let visible = if advanced { self.show_advanced } else { true };

        let style = if visible {
            format!("grid-row: {};", self.left_count)
        } else {
            format!("grid-row: {}; display: none;", self.left_count)
        };

        let label_id = crate::widget::get_unique_element_id();
        let class = classes!(
            "pwt-grid-column-1",
            "pwt-white-space-nowrap",
            field.is_disabled().then(|| Some("pwt-label-disabled")),
        );
        self.add_child(html! {
            <label id={label_id.clone()} {class} style={style.clone()}>
                {label.into_prop_value()}
            </label>
        });

        let field = field.label_id(label_id);

        self.add_child(html!{
            <div {style}>{field.into()}</div>
        });
    }

    /// Builder style method to add a field with label at the right column.
    pub fn with_right_field(
        mut self,
        label: impl IntoPropValue<AttrValue>,
        field: impl FieldBuilder,
    ) -> Self {
        self.add_right_field(false, label, field);
        self
    }

    /// Method to add a field with label at the right column.
    pub fn add_right_field(
        &mut self,
        advanced: bool,
        label: impl IntoPropValue<AttrValue>,
        field: impl FieldBuilder,
    ) {
        self.two_column = true;
        self.right_count += 1;

        let visible = if advanced { self.show_advanced } else { true };

        let style = if visible {
            format!("grid-row: {};", self.right_count)
        } else {
            format!("grid-row: {}; display: none;", self.right_count)
        };

        let class = classes!(
            "pwt-grid-column-3",
            "pwt-white-space-nowrap",
            "pwt-text-end",
            field.is_disabled().then(|| Some("pwt-label-disabled")),
        );
        self.add_child(html! {
            <label {class} style={style.clone()}>
                {label.into_prop_value()}
            </label>
        });

        self.add_child(html!{
            <div {style}>{field.into()}</div>
        });
    }

    /// Builder style method to add a large field spanning both columns.
    pub fn with_large_field(
        mut self,
        label: impl IntoPropValue<AttrValue>,
        field: impl FieldBuilder,
    ) -> Self {
        self.add_large_field(false, label, field);
        self
    }

    /// Method to add a large field spanning both columns.
    pub fn add_large_field(
        &mut self,
        advanced: bool,
        label: impl IntoPropValue<AttrValue>,
        field: impl FieldBuilder,
    ) {
        self.two_column = true;

        if self.right_count <= self.left_count {
            self.right_count = self.left_count;
        } else {
            self.left_count = self.right_count;
        }

        self.left_count += 1;
        self.right_count += 1;

        let visible = if advanced { self.show_advanced } else { true };

        let style = if visible {
            format!("grid-row: {};", self.left_count)
        } else {
            format!("grid-row: {}; display: none;", self.left_count)
        };

        let class = classes!(
            "pwt-grid-column-1",
            "pwt-white-space-nowrap",
            field.is_disabled().then(|| Some("pwt-label-disabled")),
        );
        self.add_child(html! {
            <label {class} style={style.clone()}>{label.into_prop_value()}</label>
        });

        // fixme: label_id?
        self.add_child(html!{
            <div class="pwt-fill-grid-row" {style}>{field.into()}</div>
        });
    }
}

impl Into<VTag> for InputPanel {
    fn into(mut self) -> VTag {
        if self.two_column {
            self.add_class("pwt-form-grid-col4")
        } else {
            self.add_class("pwt-form-grid-col2")
        }

        if self.gap > 0 {
            self.add_class(format!("pwt-gap-{}", self.gap));
        };

        let attributes = self.std_props.cumulate_attributes(None::<&str>);

        let listeners = Listeners::Pending(
            self.listeners.listeners.into_boxed_slice()
        );

        let children = VList::with_children(self.children, None);

        VTag::__new_other(
            Cow::Borrowed("div"),
            self.std_props.node_ref,
            self.std_props.key,
            attributes,
            listeners,
            children,
        )
    }
}

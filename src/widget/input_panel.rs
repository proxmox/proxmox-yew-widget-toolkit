use std::borrow::Cow;

use yew::html::IntoPropValue;
use yew::prelude::*;
use yew::virtual_dom::{Key, Listeners, VList, VTag};

use pwt_macros::{builder, widget};

use crate::prelude::*;

enum Position {
    Left,
    Right,
    Large,
}

/// Layout widget for forms with one or two columns.
///
/// This container show input fields with labels at different regions
/// (left, right, advanced).
#[widget(pwt=crate, @element, @container)]
#[builder]
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

    #[prop_or_default]
    #[builder(IntoPropValue, into_prop_value)]
    /// A custom label width in the grid column template.
    pub label_width: Option<AttrValue>,

    #[prop_or_default]
    #[builder(IntoPropValue, into_prop_value)]
    /// A custom field width in the grid column template
    pub field_width: Option<AttrValue>,
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
            format!(
                "grid-row: {}; grid-column-start: 1; grid-column-end: -1;",
                self.left_count
            )
        } else {
            format!("grid-row: {}; display: none;", self.left_count)
        };

        let key = format!(
            "sp_{}_{}",
            self.left_count,
            if advanced { "1" } else { "0" }
        );

        self.add_child(html! {
            <hr {key} class="pwt-w-100 pwt-my-2" {style}/>
        });
    }

    /// Builder style method to add a custom child in the first column
    pub fn with_custom_child(mut self, child: impl Into<yew::virtual_dom::VNode>) -> Self {
        self.add_custom_child(child);
        self
    }

    /// Adds custom child in the first column
    pub fn add_custom_child(&mut self, child: impl Into<yew::virtual_dom::VNode>) {
        self.left_count += 1;

        let style = format!("grid-row: {}; grid-column-end: span 2", self.left_count);

        let class = classes!("pwt-grid-column-1", "pwt-align-self-center",);
        let child = child.into();
        let key = match child.key() {
            Some(key) => key.clone(),
            None => {
                log::warn!("could not extract key from custom child");
                yew::virtual_dom::Key::from(format!("cl_{}", self.left_count))
            }
        };

        self.add_child(html! {
            <div {class} {key} {style}>{child}</div>
        });
    }

    fn add_field_impl(
        &mut self,
        column: Position,
        advanced: bool,
        label: impl IntoPropValue<AttrValue>,
        field: impl FieldBuilder,
    ) {
        let (label_column, row, field_class) = match column {
            Position::Left => {
                self.left_count += 1;
                (1, self.left_count, "pwt-grid-column-2")
            }
            Position::Right => {
                self.two_column = true;
                self.right_count += 1;
                (3, self.right_count, "pwt-grid-column-4")
            }
            Position::Large => {
                self.two_column = true;

                let max = self.left_count.max(self.right_count);
                self.left_count = max + 1;
                self.right_count = max + 1;

                (1, self.left_count, "pwt-fill-grid-row")
            }
        };

        let visible = if advanced { self.show_advanced } else { true };

        let style = if visible {
            format!("grid-row: {};", row)
        } else {
            format!("grid-row: {}; display: none;", row)
        };

        let label_id = crate::widget::get_unique_element_id();
        let class = classes!(
            format!("pwt-grid-column-{}", label_column),
            "pwt-align-self-center",
            matches!(column, Position::Right).then_some("pwt-text-align-end"),
            field.is_disabled().then_some("pwt-label-disabled"),
        );

        let label = label.into_prop_value();
        let key = Key::from(format!("label_{label}"));

        self.add_child(html! {
            <label {key} id={label_id.clone()} {class} style={style.clone()}>
                {label}
            </label>
        });

        let name = field.as_input_props().name.clone();
        let field = field.label_id(label_id).into();
        let key = match field.key() {
            Some(key) => key.clone(),
            None => match name {
                Some(name) => Key::from(name.to_string()),
                None => {
                    log::warn!("could not extract key from field");
                    Key::from(format!("f_{}_{}_{}", label_column, row, advanced))
                }
            },
        };

        let class = classes!(field_class, "pwt-align-self-center");
        self.add_child(html! {
            <div {key} {class} {style}>{field}</div>
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
        self.add_field_impl(Position::Left, advanced, label, field)
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

    /// Builder style method to add a right column field in the advanced section.
    pub fn with_right_advanced_field(
        mut self,
        label: impl IntoPropValue<AttrValue>,
        field: impl FieldBuilder,
    ) -> Self {
        self.add_right_field(true, label, field);
        self
    }

    /// Method to add a field with label at the right column.
    pub fn add_right_field(
        &mut self,
        advanced: bool,
        label: impl IntoPropValue<AttrValue>,
        field: impl FieldBuilder,
    ) {
        self.add_field_impl(Position::Right, advanced, label, field)
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

    /// Builder style method to add a large field in the advanced section
    pub fn with_large_advanced_field(
        mut self,
        label: impl IntoPropValue<AttrValue>,
        field: impl FieldBuilder,
    ) -> Self {
        self.add_large_field(true, label, field);
        self
    }

    /// Method to add a large field spanning both columns.
    pub fn add_large_field(
        &mut self,
        advanced: bool,
        label: impl IntoPropValue<AttrValue>,
        field: impl FieldBuilder,
    ) {
        self.add_field_impl(Position::Large, advanced, label, field)
    }
}

impl Default for InputPanel {
    fn default() -> Self {
        Self::new()
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

        if self.label_width.is_some() || self.field_width.is_some() {
            let mut column_template = format!(
                "{} {}",
                self.label_width.unwrap_or("minmax(130px, 0.65fr)".into()),
                self.field_width.unwrap_or("minmax(200px, 1fr)".into())
            );

            if self.two_column {
                column_template = format!("{} {}", column_template, column_template);
            }

            let style = self
                .std_props
                .attributes
                .get_mut_index_map()
                .remove(&AttrValue::Static("style"))
                .map(|(style, _)| style)
                .unwrap_or("".into());

            self.std_props.attributes.get_mut_index_map().insert(
                "style".into(),
                (
                    format!("grid-template-columns: {};{}", column_template, style).into(),
                    yew::virtual_dom::ApplyAttributeAs::Attribute,
                ),
            );
        }

        let attributes = self.std_props.cumulate_attributes(None::<&str>);

        let listeners = Listeners::Pending(self.listeners.listeners.into_boxed_slice());

        let children = VList::with_children(self.children, None);

        VTag::__new_other(
            Cow::Borrowed("div"),
            self.std_props.node_ref,
            self.std_props.key,
            attributes,
            listeners,
            children.into(),
        )
    }
}

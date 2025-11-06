use std::borrow::Cow;

use yew::html::IntoPropValue;
use yew::prelude::*;
use yew::virtual_dom::{Key, Listeners, VList, VTag};

use pwt_macros::{builder, widget};

use crate::prelude::*;
use crate::props::{PwtSpace, WidgetStyleBuilder};
use crate::widget::{Container, FieldLabel, Row};

pub enum FieldPosition {
    Left,
    Right,
    Large,
}

/// Allow a component te be associated with label
///
/// [InputPanel] `add_field` functions requires components to implement this trait. The trait
/// is automatically implemented for standard fields (impl FieldBuilder).
pub trait Labelable: Into<Html> {
    /// Should return the field name (used to generate default component key)
    fn name(&self) -> Option<AttrValue>;
    /// Assign a label_id to the component
    fn set_label_id(&mut self, label_id: AttrValue);
    /// Returns if the field is diabled.
    fn disabled(&self) -> bool;
}

impl<T: FieldBuilder> Labelable for T {
    fn name(&self) -> Option<AttrValue> {
        self.as_input_props().name.clone()
    }

    fn set_label_id(&mut self, label_id: AttrValue) {
        self.set_label_id(label_id);
    }

    fn disabled(&self) -> bool {
        self.as_input_props().disabled
    }
}

/// Layout widget for forms with one or two columns.
///
/// This container show input fields with labels at different regions
/// (left, right, advanced).
#[widget(pwt=crate, @element, @container)]
#[builder]
#[derive(Properties, PartialEq, Clone)]
pub struct InputPanel {
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
    ///
    /// This is ignored by the mobile view (use field_width instead).
    pub label_width: Option<AttrValue>,

    #[prop_or_default]
    #[builder(IntoPropValue, into_prop_value)]
    /// A custom field width in the grid column template
    pub field_width: Option<AttrValue>,

    /// Layout for mobile devices.
    ///
    /// This renders all fields using a single row (ignores [FieldPosition]), and positions labels above fields.
    #[prop_or_default]
    #[builder]
    pub mobile: bool,
}

impl InputPanel {
    /// Creates a new instance.
    pub fn new() -> Self {
        yew::props!(Self {})
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
        let key = format!(
            "sp_{}_{}",
            self.left_count,
            if advanced { "1" } else { "0" }
        );

        let two_column = self.two_column;
        self.add_custom_child_impl(
            FieldPosition::Large,
            advanced,
            false,
            Container::from_tag("hr").key(key).class("pwt-w-100").into(),
        );
        // Note: do not change two_column when adding a spacer!
        self.two_column = two_column;
    }

    /// Builder style method to add a custom child in the first column
    pub fn with_custom_child(mut self, child: impl Into<Html>) -> Self {
        self.add_custom_child(child);
        self
    }

    /// Adds custom child in the first column
    pub fn add_custom_child(&mut self, child: impl Into<Html>) {
        self.add_custom_child_impl(FieldPosition::Left, false, false, child.into());
    }

    /// Builder style method to add a custom child and options
    pub fn with_custom_child_and_options(
        mut self,
        position: FieldPosition,
        advanced: bool,
        hidden: bool,
        child: impl Into<Html>,
    ) -> Self {
        self.add_custom_child_with_options(position, advanced, hidden, child);
        self
    }

    /// Method to add a custon child with options
    pub fn add_custom_child_with_options(
        &mut self,
        position: FieldPosition,
        advanced: bool,
        hidden: bool,
        child: impl Into<Html>,
    ) {
        self.add_custom_child_impl(position, advanced, hidden, child.into())
    }

    /// Builder style method to add a custom child in the second column
    pub fn with_right_custom_child(mut self, child: impl Into<Html>) -> Self {
        self.add_right_custom_child(child);
        self
    }

    /// Adds custom child in the second column
    pub fn add_right_custom_child(&mut self, child: impl Into<Html>) {
        self.add_custom_child_impl(FieldPosition::Right, false, false, child.into());
    }

    /// Builder style method to add a large custom child
    pub fn with_large_custom_child(mut self, child: impl Into<Html>) -> Self {
        self.add_large_custom_child(child);
        self
    }

    /// Adds large custom child
    pub fn add_large_custom_child(&mut self, child: impl Into<Html>) {
        self.add_custom_child_impl(FieldPosition::Large, false, false, child.into());
    }

    fn add_custom_child_impl(
        &mut self,
        column: FieldPosition,
        advanced: bool,
        hidden: bool,
        child: Html,
    ) {
        let mut visible = if advanced { self.show_advanced } else { true };
        if hidden {
            visible = false;
        }

        let style = if visible {
            let (row, start, span) = if self.mobile {
                self.left_count += 1; // ignore position
                (self.left_count, 1, -1)
            } else {
                match column {
                    FieldPosition::Left => {
                        self.left_count += 1;
                        (self.left_count, 1, 4)
                    }
                    FieldPosition::Right => {
                        self.two_column = true;
                        self.right_count += 1;
                        (self.right_count, 4, -1)
                    }
                    FieldPosition::Large => {
                        self.two_column = true;

                        let max = self.left_count.max(self.right_count);
                        self.left_count = max + 1;
                        self.right_count = max + 1;

                        (self.left_count, 1, -1)
                    }
                }
            };
            format!("grid-row: {}; grid-column: {}/{};", row, start, span)
        } else {
            "display: none;".to_string()
        };

        let key = match child.key() {
            Some(key) => key.clone(),
            None => {
                #[cfg(debug_assertions)]
                log::warn!("could not extract key from custom child, generating one");
                yew::virtual_dom::Key::from(format!("cl_{}", self.left_count))
            }
        };

        if self.mobile {
            self.add_child(
                Container::new()
                    .key(key)
                    .attribute("style", style)
                    .with_child(child),
            );
        } else {
            self.add_child(
                Container::new()
                    .key(key)
                    .class("pwt-align-self-center")
                    .attribute("style", style)
                    .with_child(child),
            );
        }
    }

    fn add_field_impl(
        &mut self,
        column: FieldPosition,
        advanced: bool,
        hidden: bool,
        label: impl Into<FieldLabel>,
        //field: impl Labelable,
        mut field: impl Labelable,
    ) {
        let mut visible = if advanced { self.show_advanced } else { true };
        if hidden {
            visible = false;
        }

        let (label_column, row, field_class) = if visible {
            if self.mobile {
                self.left_count += 1; // ignore position
                (1, self.left_count, "pwt-single-grid-row")
            } else {
                match column {
                    FieldPosition::Left => {
                        self.left_count += 1;
                        (1, self.left_count, "pwt-grid-column-2")
                    }
                    FieldPosition::Right => {
                        self.two_column = true;
                        self.right_count += 1;
                        (3, self.right_count, "pwt-grid-column-4")
                    }
                    FieldPosition::Large => {
                        self.two_column = true;

                        let max = self.left_count.max(self.right_count);
                        self.left_count = max + 1;
                        self.right_count = max + 1;

                        (1, self.left_count, "pwt-fill-grid-row")
                    }
                }
            }
        } else {
            (1, 10000, "pwt-grid-column-2")
        };

        let style = if visible {
            format!("grid-row: {};", row)
        } else {
            "display: none;".to_string()
        };

        let label_id = crate::widget::get_unique_element_id();
        let mut label: FieldLabel = label.into().id(label_id.clone());
        if label.std_props.key.is_none() {
            label.set_key(format!("label_{}", label.label));
        }

        let name = field.name();
        let is_disabled = field.disabled();
        field.set_label_id(label_id.into());
        let field = field.into();
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

        if self.mobile {
            let label_class = classes!(is_disabled.then_some("pwt-label-disabled"),);
            let field_with_label = Container::new()
                .key(key)
                .class(crate::css::FlexDirection::Column)
                .style("overflow", "hidden") // pass size constraints down
                .style("display", if visible { "flex" } else { "none" })
                .style("grid-row", visible.then(|| row.to_string()))
                .style("grid-column", "1/2")
                .with_child(label.class(label_class).padding_bottom(PwtSpace::Em(0.3)))
                .with_child(field);
            self.add_child(field_with_label);
        } else {
            self.add_child(
                label
                    .class("pwt-align-self-center")
                    .class(format!("pwt-grid-column-{}", label_column))
                    .class(is_disabled.then_some("pwt-label-disabled"))
                    .attribute("style", style.clone()),
            );

            self.add_child(
                Container::new()
                    .key(key)
                    .class("pwt-align-self-center")
                    .class(field_class)
                    .attribute("style", style)
                    .with_child(field),
            );
        }
    }

    /// Builder style method to add a field with label at the left column.
    pub fn with_field(mut self, label: impl Into<FieldLabel>, field: impl Labelable) -> Self {
        self.add_field(label, field);
        self
    }

    /// Builder style method to add a field with label
    pub fn with_field_and_options(
        mut self,
        position: FieldPosition,
        advanced: bool,
        hidden: bool,
        label: impl Into<FieldLabel>,
        field: impl Labelable,
    ) -> Self {
        self.add_field_with_options(position, advanced, hidden, label, field);
        self
    }

    pub fn with_advanced_field(
        mut self,
        label: impl Into<FieldLabel>,
        field: impl Labelable,
    ) -> Self {
        self.add_field_with_options(FieldPosition::Left, true, false, label, field);
        self
    }

    /// Method to add a field with label at the left column.
    pub fn add_field(&mut self, label: impl Into<FieldLabel>, field: impl Labelable) {
        self.add_field_impl(FieldPosition::Left, false, false, label, field)
    }

    /// Method to add a field with label
    pub fn add_field_with_options(
        &mut self,
        position: FieldPosition,
        advanced: bool,
        hidden: bool,
        label: impl Into<FieldLabel>,
        field: impl Labelable,
    ) {
        self.add_field_impl(position, advanced, hidden, label, field)
    }

    /// Builder style method to add a field with label at the right column.
    pub fn with_right_field(mut self, label: impl Into<FieldLabel>, field: impl Labelable) -> Self {
        self.add_right_field(label, field);
        self
    }

    /// Builder style method to add a right column field in the advanced section.
    pub fn with_right_advanced_field(
        mut self,
        label: impl Into<FieldLabel>,
        field: impl Labelable,
    ) -> Self {
        self.add_field_with_options(FieldPosition::Right, true, false, label, field);
        self
    }

    /// Method to add a field with label at the right column.
    pub fn add_right_field(&mut self, label: impl Into<FieldLabel>, field: impl Labelable) {
        self.add_field_impl(FieldPosition::Right, false, false, label, field)
    }

    /// Builder style method to add a large field spanning both columns.
    pub fn with_large_field(mut self, label: impl Into<FieldLabel>, field: impl Labelable) -> Self {
        self.add_large_field(false, false, label, field);
        self
    }

    /// Builder style method to add a large field in the advanced section
    pub fn with_large_advanced_field(
        mut self,
        label: impl Into<FieldLabel>,
        field: impl Labelable,
    ) -> Self {
        self.add_large_field(true, false, label, field);
        self
    }

    /// Method to add a large field spanning both columns.
    pub fn add_large_field(
        &mut self,
        advanced: bool,
        hidden: bool,
        label: impl Into<FieldLabel>,
        field: impl Labelable,
    ) {
        self.add_field_impl(FieldPosition::Large, advanced, hidden, label, field)
    }

    /// Builder style method to add a single line field.
    pub fn with_single_line_field(
        mut self,
        advanced: bool,
        hidden: bool,
        label: impl Into<FieldLabel>,
        field: impl Labelable,
    ) -> Self {
        self.add_single_line_field(advanced, hidden, label, field);
        self
    }

    /// Method to add a field with label at the left column, always use a single line
    ///
    /// This is usefull for mobile UI to render boolean inputs.
    ///
    /// For desktop layout, this simple calls `add_field_with_options(FieldPosition::Left, ...)`.
    pub fn add_single_line_field(
        &mut self,
        advanced: bool,
        hidden: bool,
        label: impl Into<FieldLabel>,
        mut field: impl Labelable,
    ) {
        if self.mobile {
            let label_id = crate::widget::get_unique_element_id();

            let is_disabled = field.disabled();
            let mut label: FieldLabel = label
                .into()
                .id(label_id.clone())
                .class(crate::css::Flex::Fill)
                .class(is_disabled.then_some("pwt-label-disabled"));
            if label.std_props.key.is_none() {
                label.set_key(format!("label_{}", label.label));
            }

            field.set_label_id(label_id.into());

            let name = field.name().map(|name| Key::from(name.to_string()));
            let field = field.into();
            let key = field.key().cloned().or(name);

            let row = Row::new()
                .key(key)
                .class(crate::css::AlignItems::Center)
                .with_child(label)
                .with_child(field);

            self.add_custom_child_impl(FieldPosition::Left, advanced, hidden, row.into());
        } else {
            self.add_field_impl(FieldPosition::Left, advanced, hidden, label, field);
        }
    }
}

impl Default for InputPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoVTag for InputPanel {
    fn into_vtag_with_ref(mut self, node_ref: NodeRef) -> VTag {
        if self.mobile {
            self.add_class("pwt-d-grid");
            self.add_class("pwt-gap-2");

            self.set_style(
                "grid-template-columns",
                self.field_width.clone().unwrap_or(AttrValue::Static("1fr")),
            )
        } else {
            if self.two_column {
                self.add_class("pwt-form-grid-col4");
            } else {
                self.add_class("pwt-form-grid-col2");
            }

            if self.label_width.is_some() || self.field_width.is_some() {
                let mut column_template = format!(
                    "{} {}",
                    self.label_width
                        .as_deref()
                        .unwrap_or("minmax(130px, 0.65fr)"),
                    self.field_width.as_deref().unwrap_or("minmax(200px, 1fr)")
                );

                if self.two_column {
                    column_template = format!(
                        "{} calc(var(--pwt-spacer-4) * 2) {}",
                        column_template, column_template
                    );
                }

                self.set_style("grid-template-columns", column_template.to_string());
            }
        }

        let attributes = self.std_props.cumulate_attributes(None::<&str>);

        let listeners = Listeners::Pending(self.listeners.listeners.into_boxed_slice());

        let children = VList::with_children(self.children, None);

        VTag::__new_other(
            Cow::Borrowed("div"),
            node_ref,
            self.std_props.key,
            attributes,
            listeners,
            children.into(),
        )
    }
}

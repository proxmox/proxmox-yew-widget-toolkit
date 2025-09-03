use yew::prelude::*;
use yew::virtual_dom::{Listeners, VTag};

use pwt_macros::widget;

use crate::props::{FieldStdProps, IntoVTag, WidgetStdProps};

/// Html Input element.
#[widget(pwt=crate, @input, @element)]
#[derive(Clone, PartialEq, Properties)]
pub struct Input {}

impl Default for Input {
    fn default() -> Self {
        Self::new()
    }
}

impl Input {
    /// Creates a new instance.
    pub fn new() -> Self {
        yew::props! {Self {}}
    }

    /// Use common properties from [WidgetStdProps]
    pub fn with_std_props(mut self, props: &WidgetStdProps) -> Self {
        self.std_props = props.clone();
        self
    }

    /// Use common properties from [FieldStdProps]
    pub fn with_input_props(mut self, props: &FieldStdProps) -> Self {
        self.input_props = props.clone();
        self
    }
}

impl IntoVTag for Input {
    fn into_vtag_with_ref(self, node_ref: NodeRef) -> VTag {
        let mut attributes = self.std_props.cumulate_attributes(None::<&str>);
        let attr_map = attributes.get_mut_index_map();
        self.input_props.cumulate_attributes(attr_map);

        let value = attr_map
            .get(&AttrValue::Static("value"))
            .map(|a| a.0.clone());
        let checked = attr_map
            .get(&AttrValue::Static("checked"))
            .is_some()
            .then_some(true);

        let listeners = Listeners::Pending(self.listeners.listeners.into_boxed_slice());

        VTag::__new_input(
            value,
            checked,
            node_ref,
            self.std_props.key,
            attributes,
            listeners,
        )
    }
}

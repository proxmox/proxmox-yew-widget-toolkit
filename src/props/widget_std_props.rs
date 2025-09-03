use std::borrow::Cow;

use yew::html::IntoPropValue;
use yew::prelude::*;
use yew::virtual_dom::{ApplyAttributeAs, Attributes, Key, Listeners, VList, VNode, VTag};

use crate::props::{CssStyles, ListenersWrapper};

/// Standard widget properties.
#[derive(PartialEq, Debug, Default, Clone)]
pub struct WidgetStdProps {
    /// The yew component key.
    pub key: Option<Key>,

    /// CSS class.
    pub class: Classes,

    /// Additional Html attributes.
    pub attributes: Attributes,

    /// Additional CSS styles
    pub styles: CssStyles,
}

impl WidgetStdProps {
    /// Method to set attributes.
    ///
    /// Note: Value 'None' removes the attribute.
    pub fn set_attribute(
        &mut self,
        key: impl Into<AttrValue>,
        value: impl IntoPropValue<Option<AttrValue>>,
    ) {
        if let Some(value) = value.into_prop_value() {
            self.attributes
                .get_mut_index_map()
                .insert(key.into(), (value, ApplyAttributeAs::Attribute));
        } else {
            self.attributes.get_mut_index_map().swap_remove(&key.into());
        }
    }

    /// Method to moves attributes over
    ///
    /// Tries to be efficient by consuming the source.
    /// Overwrites existing attributes.
    pub fn add_attributes(&mut self, attributes: Attributes) {
        let attrs = self.attributes.get_mut_index_map();
        match attributes {
            Attributes::Static(list) => {
                for (key, value, attr) in list {
                    attrs.insert((*key).into(), ((*value).into(), *attr));
                }
            }
            Attributes::Dynamic { keys, values } => {
                for (key, value) in keys.iter().zip(values.into_vec().into_iter()) {
                    if let Some((value, attr)) = value {
                        attrs.insert((*key).into(), (value, attr));
                    }
                }
            }
            Attributes::IndexMap(list) => {
                for (key, (value, attr)) in list.into_iter() {
                    attrs.insert(key, (value, attr));
                }
            }
        }
    }

    /// Helper to gather all attributes into a single [Attributes]
    /// map.
    pub fn cumulate_attributes(&self, additional_class: Option<impl Into<Classes>>) -> Attributes {
        let mut class = self.class.clone();

        if let Some(additional_class) = additional_class {
            class.push(additional_class);
        }

        let mut attributes = self.attributes.clone();
        let attr_map = attributes.get_mut_index_map();
        attr_map.insert(
            AttrValue::Static("class"),
            (class.into_prop_value(), ApplyAttributeAs::Attribute),
        );

        if !self.styles.is_empty() {
            let style = self
                .styles
                .compile_style_attribute(attr_map.get("style").map(|a| a.0.clone()));

            attr_map.insert(
                AttrValue::Static("style"),
                (style, ApplyAttributeAs::Attribute),
            );
        }

        attributes
    }

    /// Helper to create a VTag from [WidgetStdProps].
    pub fn into_vtag(
        self,
        tag: Cow<'static, str>,
        node_ref: NodeRef,
        additional_class: Option<impl Into<Classes>>,
        listeners: Option<ListenersWrapper>,
        children: Option<Vec<VNode>>,
    ) -> VTag {
        let attributes = self.cumulate_attributes(additional_class);

        let listeners = match listeners {
            None => Listeners::None,
            Some(wrapper) => Listeners::Pending(wrapper.listeners.into_boxed_slice()),
        };

        let vlist = if let Some(children) = children {
            VList::with_children(children, None)
        } else {
            VList::new()
        };

        VTag::__new_other(tag, node_ref, self.key, attributes, listeners, vlist.into())
    }
}

/// Like `Into<VTag>`, but allow to specify a [NodeRef].
///
/// This is basically an optimization to avoid an additional [NodeRef] allocation.
pub trait IntoVTag: Sized {
    /// Convienience helper.
    fn into_html_with_ref(self, node_ref: NodeRef) -> VNode {
        self.into_vtag_with_ref(node_ref).into()
    }

    /// Genertate a [VTag] with specified [NodeRef].
    fn into_vtag_with_ref(self, node_ref: NodeRef) -> VTag;

    /// Convienience helper.
    fn into_vtag(self) -> VTag {
        self.into_vtag_with_ref(NodeRef::default())
    }
}

/// Provide a IntoVTag default implementation for `T: Into<VTag>`.
///
/// While this works, it allocate one useless NodeRef. Please implement [IntoVTag]
/// to avoid that usless allocation.
impl<T: Into<VTag>> IntoVTag for T {
    fn into_vtag_with_ref(self, node_ref: NodeRef) -> VTag {
        let mut vtag = self.into_vtag();
        vtag.node_ref = node_ref;
        vtag
    }
}

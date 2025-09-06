//! Common Property types and builder traits

use std::ops::Deref;
use std::rc::Rc;

use yew::html::IntoPropValue;
use yew::prelude::*;
use yew::virtual_dom::{Key, VNode, VTag};

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

/// Trait to generate inline Html.
///
/// We use this for properties lite panel titles, where you usually just want text.
/// This adds the ability to add simple inline markup.
///
/// Note: It is still possible to use arbitrary html (we implement it for Html and `Option<Html>`)
///
/// Note: There are several implementations for `Option<T>`, so it requires type annotation
/// for `None`, i.e. `None::<&str>`. To avoid that, we implement it for the null type.
/// Simply use `()` instead of `None::<&str>`.
pub trait IntoOptionalInlineHtml {
    fn into_optional_inline_html(self) -> Option<Html>;
}

impl IntoOptionalInlineHtml for Html {
    fn into_optional_inline_html(self) -> Option<Html> {
        Some(self)
    }
}

impl IntoOptionalInlineHtml for Option<Html> {
    fn into_optional_inline_html(self) -> Option<Html> {
        self
    }
}

impl IntoOptionalInlineHtml for () {
    fn into_optional_inline_html(self) -> Option<Html> {
        None
    }
}

macro_rules! impl_into_inline_html {
    ($t:ty) => {
        impl IntoOptionalInlineHtml for $t {
            fn into_optional_inline_html(self) -> Option<Html> {
                Some(html! {self})
            }
        }

        impl IntoOptionalInlineHtml for Option<$t> {
            fn into_optional_inline_html(self) -> Option<Html> {
                self.map(|me| html! {me})
            }
        }
    };
}

impl_into_inline_html!(String);
impl_into_inline_html!(&str);
impl_into_inline_html!(AttrValue);

/// Trait which provides mutable access to the class property.
pub trait AsClassesMut {
    /// Mutable access to the class property.
    fn as_classes_mut(&mut self) -> &mut yew::Classes;
}

impl AsClassesMut for yew::Classes {
    fn as_classes_mut(&mut self) -> &mut yew::Classes {
        self
    }
}

/// Trait to create optional Key.
///
/// # Note
///
/// Yew 0.20 does not provide IntoPropValue for Key ...
///
/// - see <https://github.com/yewstack/yew/pull/2804>
/// - see <https://github.com/yewstack/yew/issues/3205>
pub trait IntoOptionalKey {
    fn into_optional_key(self) -> Option<Key>;
}

impl IntoOptionalKey for Key {
    fn into_optional_key(self) -> Option<Key> {
        Some(self)
    }
}

impl IntoOptionalKey for Option<Key> {
    fn into_optional_key(self) -> Option<Key> {
        self
    }
}

macro_rules! key_impl_from_into_prop_value {
    ($type:ty) => {
        impl IntoOptionalKey for $type {
            fn into_optional_key(self) -> Option<Key> {
                let attr: Option<AttrValue> = self.into_prop_value();
                attr.map(|me| Key::from(me.deref()))
            }
        }
        impl IntoOptionalKey for Option<$type> {
            fn into_optional_key(self) -> Option<Key> {
                let attr: Option<AttrValue> = self.into_prop_value();
                attr.map(|me| Key::from(me.deref()))
            }
        }
    };
}

key_impl_from_into_prop_value!(&'static str);
key_impl_from_into_prop_value!(Rc<str>);
key_impl_from_into_prop_value!(AttrValue);
key_impl_from_into_prop_value!(String);

mod callback_mut;
pub use callback_mut::{CallbackMut, CallbackMutScopeExt, IntoEventCallbackMut};

mod extract_key_function;
pub use extract_key_function::{ExtractKeyFn, ExtractPrimaryKey, IntoExtractKeyFn};

mod load_callback;
pub use load_callback::{set_http_get_method, IntoLoadCallback, LoadCallback};

mod submit_callback;
pub use submit_callback::{IntoSubmitCallback, SubmitCallback};

mod field_std_props;
pub use field_std_props::FieldStdProps;

mod field_builder;
pub use field_builder::FieldBuilder;

mod storage_location;
pub use storage_location::{IntoStorageLocation, StorageLocation};

mod widget_std_props;
pub use widget_std_props::WidgetStdProps;

mod widget_builder;
pub use widget_builder::WidgetBuilder;

mod css_styles;
pub use css_styles::{AsCssStylesMut, CssStyles};

mod widget_style_builder;
pub use widget_style_builder::{CssLength, WidgetStyleBuilder};

mod container_builder;
pub use container_builder::ContainerBuilder;

mod event_subscriber;
pub use event_subscriber::{EventSubscriber, ListenersWrapper};

mod border;
pub use border::CssBorderBuilder;

#[macro_use]
mod pwt_space;
pub use pwt_space::PwtSpace;

mod margin;
pub use margin::CssMarginBuilder;

mod padding;
pub use padding::CssPaddingBuilder;

mod render_function;
pub use render_function::{
    BuilderFn, IntoOptionalBuilderFn, IntoOptionalRenderFn, IntoOptionalTextRenderFn, RenderFn,
    TextRenderFn,
};

mod sorter_function;
pub use sorter_function::{IntoSorterFn, SorterFn};

mod filter_function;
pub use filter_function::{FilterFn, IntoFilterFn, IntoTextFilterFn, TextFilterFn};

/// Implement builder functions for node_ref and key properties.
///
/// ```
/// use pwt::prelude::*;
/// use pwt::impl_yew_std_props_builder;
/// use yew::virtual_dom::Key;
/// pub struct MyComponentProps {
///     pub node_ref: NodeRef,
///     pub key: Option<Key>,
/// }
///
/// impl MyComponentProps {
///     impl_yew_std_props_builder!();
///     // pub fn key(mut self, key: impl IntoOptionalKey) -> Self;
///     // pub fn set_key(&mut self, key: impl IntoOptionalKey);
/// }
/// ```
#[macro_export]
macro_rules! impl_yew_std_props_builder {
    () => {
        /// Builder style method to set the yew `key` property
        pub fn key(mut self, key: impl IntoOptionalKey) -> Self {
            self.set_key(key);
            self
        }

        /// Method to set the yew `key` property
        pub fn set_key(&mut self, key: impl IntoOptionalKey) {
            self.key = key.into_optional_key();
        }
    };
}

/// Implement builder functions for class properties.
///
/// ```
/// use pwt::prelude::*;
/// use pwt::impl_class_prop_builder;
///
/// pub struct MyComponentProps {
///     class: Classes,
/// }
///
/// impl MyComponentProps {
///     impl_class_prop_builder!();
///     // pub fn class(mut self, class: impl Into<Classes>) -> Self;
///     // pub fn add_class(&mut self, class: impl Into<Classes>);
/// }
/// ```
#[macro_export]
macro_rules! impl_class_prop_builder {
    () => {
        /// Builder style method to add a html class
        pub fn class(mut self, class: impl Into<Classes>) -> Self {
            self.add_class(class);
            self
        }

        /// Method to add a html class
        pub fn add_class(&mut self, class: impl Into<Classes>) {
            self.class.push(class);
        }
    };
}

//! Common Property types and builder traits

use std::ops::Deref;
use std::rc::Rc;

use yew::html::IntoPropValue;
use yew::prelude::*;
use yew::virtual_dom::Key;


/// Trait to generate inline Html.
///
/// We use this for properties lite panel titles, where you usually just want text.
/// This adds the ability to add simple inline markup.
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

macro_rules! impl_into_inline_html {
    ($t:ty) => {
        impl IntoOptionalInlineHtml for $t {
            fn into_optional_inline_html(self) -> Option<Html> {
                Some(html!{self})
            }
        }

        impl IntoOptionalInlineHtml for Option<$t> {
            fn into_optional_inline_html(self) -> Option<Html> {
                self.map(|me| html!{me})
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

mod field_std_props;
pub use field_std_props::FieldStdProps;

mod field_builder;
pub use field_builder::FieldBuilder;

mod widget_std_props;
pub use widget_std_props::WidgetStdProps;

mod widget_builder;
pub use widget_builder::WidgetBuilder;

mod container_builder;
pub use container_builder::ContainerBuilder;

mod event_subscriber;
pub use event_subscriber::{EventSubscriber, ListenersWrapper};

mod border;
pub use border::CssBorderBuilder;

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
pub use filter_function::{FilterFn, IntoFilterFn, TextFilterFn, IntoTextFilterFn};

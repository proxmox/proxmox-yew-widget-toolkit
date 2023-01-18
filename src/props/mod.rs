//! Common Property types and builder traits

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

mod callback_mut;
pub use callback_mut::{CallbackMut, CallbackMutScopeExt, IntoEventCallbackMut};

pub mod css;

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
pub use event_subscriber::{ListenersWrapper, EventSubscriber};

mod border;
pub use border::CssBorderBuilder;

mod margin;
pub use margin::CssMarginBuilder;

mod padding;
pub use padding::CssPaddingBuilder;

mod render_function;
pub use render_function::{RenderFn, BuilderFn, IntoOptionalBuilderFn};

mod sorter_function;
pub use sorter_function::{SorterFn, IntoSorterFn};

mod filter_function;
pub use filter_function::{FilterFn, IntoFilterFn};

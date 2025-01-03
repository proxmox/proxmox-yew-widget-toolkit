use proc_macro::TokenStream;
use syn::parse_macro_input;

mod widget;
use widget::*;

mod builder;
use builder::*;

/// Widget Macro for reusing some properties of HTML elements and automatically
/// implementing some useful traits.
///
/// With this, a struct will be decorated with some properties, depending on
/// the types given.
///
/// # Default behaviour
///
/// By default  a `std_props` property of type `pwt::props::WidgetStdProps`
/// will be added and the traits:
///
/// * `pwt::props::WidgetBuilder`
/// * `pwt::props::AsClassesMut`
/// * `pwt::props::CssMarginBuilder`
/// * `pwt::props::CssPaddingBuilder`
/// * `pwt::props::CssBorderBuilder`
///
/// will be implemented.
///
/// Also `Into<yew::Html>` as well as `yew::ToHtml` will be automatically
/// implemented.
///
/// > Note: This requires that the struct implements `Clone`.
///
/// # Types
///
/// Then there are the following available types:
/// * `input` -  this will add an `input_props` property of type `FieldStdProps` and
///   implement `FieldBuilder` on it
/// * `container` - this will add a `children` property of type `Vec<yew::Html>` and
///   implement `ContainerBuilder`
/// * `element` - this will add a `listeners` property of type `ListenersWrapper` and
///   implement `EventSubscriber`
/// * `svg` - this prevents the implemntation of `pwt::props::AsClassesMut` and the
///   `Css*Builder` traits.
///
/// # Syntax
///
/// `#[widget(crate=foo, comp=bar, @element, @svg, ...)]`
///
/// * `crate=foo` is optional and designates where to find the `pwt` crate
/// * `comp=bar` is also optional and describes the `Component` to use
/// * The desired types are prefixed with `@` and simply appended as a comma seperated list
///
/// # Examples:
///
/// ```
/// # // dummy crate here so we don't have a cyclic dependency
/// # pub mod pwt {
/// #     pub mod props {
/// #         #[derive(Clone, PartialEq, Default)]
/// #         pub struct WidgetStdProps {
/// #             pub class: yew::Classes,
/// #         }
/// #         #[derive(Clone, PartialEq, Default)]
/// #         pub struct ListenersWrapper;
/// #         pub trait WidgetBuilder {
/// #             fn as_std_props_mut(&mut self) -> &mut WidgetStdProps;
/// #         }
/// #         pub trait AsClassesMut {
/// #             fn as_classes_mut(&mut self) -> &mut yew::Classes;
/// #         }
/// #         pub trait CssBorderBuilder {}
/// #         pub trait CssMarginBuilder {}
/// #         pub trait CssPaddingBuilder {}
/// #         pub trait EventSubscriber: Sized {
/// #             fn as_listeners_mut(&mut self) -> &mut ListenersWrapper;
/// #             fn onclick(mut self, _cb: impl yew::html::IntoEventCallback<yew::MouseEvent>) -> Self {
/// #                 self
/// #             }
/// #         }
/// #     }
/// # }
/// use pwt_macros::widget;
/// use pwt::props::EventSubscriber;
/// use yew::prelude::*;
///
/// #[widget(@element)]
/// #[derive(Clone, PartialEq, Properties)]
/// struct Foo {
///     // some fields go here
/// }
///
/// impl Foo {
///     fn new() -> Self {
///         yew::props!( Self {} )
///     }
/// }
///
/// // implementing Into<VTag> necessary without comp
/// # use yew::virtual_dom::VTag;
/// impl From<Foo> for VTag {
///     //...
/// #     fn from(value: Foo) -> VTag {
/// #         VTag::new("foo")
/// #     }
/// }
///
/// // now you can use the provided methods from e.g., EventSubscriber:
/// let foo = Foo::new()
///     .onclick(|event: MouseEvent| { /* do something in onclick */});
/// ```
///
/// If the struct is implemented as `Properties` for a accompanying component
/// you can do
///
/// ```
/// # // dummy crate here so we don't have a cyclic dependency
/// # pub mod pwt {
/// #     pub mod props {
/// #         #[derive(Clone, PartialEq, Default)]
/// #         pub struct WidgetStdProps {
/// #             pub class: yew::Classes,
/// #             pub key: Option<yew::virtual_dom::Key>,
/// #         }
/// #         #[derive(Clone, PartialEq, Default)]
/// #         pub struct ListenersWrapper;
/// #         pub trait WidgetBuilder {
/// #             fn as_std_props_mut(&mut self) -> &mut WidgetStdProps;
/// #         }
/// #         pub trait AsClassesMut {
/// #             fn as_classes_mut(&mut self) -> &mut yew::Classes;
/// #         }
/// #         pub trait CssBorderBuilder {}
/// #         pub trait CssMarginBuilder {}
/// #         pub trait CssPaddingBuilder {}
/// #         pub trait EventSubscriber: Sized {
/// #             fn as_listeners_mut(&mut self) -> &mut ListenersWrapper;
/// #         }
/// #     }
/// # }
/// use pwt_macros::widget;
/// use pwt::props::EventSubscriber;
/// use yew::prelude::*;
///
/// #[widget(comp=Foo, @element)]
/// #[derive(Clone, PartialEq, Properties)]
/// struct FooProperties {
///     // some fields go here
/// }
///
/// impl FooProperties {
///     fn new() -> Self {
///         yew::props!( Self {} )
///     }
/// }
///
/// // no Into<VTag> implementing necessary here, it will use the Component for that
///
/// struct Foo {}
///
/// impl yew::Component for Foo {
///     type Message = ();
///     type Properties = FooProperties;
///
///     fn create(ctx: &Context<Self>) -> Self {
///         // implement create
/// #       let _ = ctx.props();
/// #       Foo {}
///     }
///
///     fn view(&self, ctx: &Context<Self>) -> Html {
///         // implement view
/// #       let _ = ctx.props();
/// #       html!{}
///     }
/// }
/// ```
///
#[proc_macro_attribute]
pub fn widget(attr: TokenStream, item: TokenStream) -> TokenStream {
    //eprintln!("attr: \"{}\"", attr.to_string());
    let setup = parse_macro_input!(attr as WidgetSetup);

    handle_widget_struct(&setup, item)
}

/// Builder Macro for auto-generating builder style methods
///
/// Can be applied to a struct and it's fields, to auto-generate
/// setter/builder style methods for them.
///
/// # How to use
///
/// It can be enabled by adding the `[builder]` attribute to the struct
/// and the fields for which you want to use it.
///
/// The `[builder]` attribute on the fields accept a few parameters, namely
///
/// * an "Into" type
/// * an "into" function
/// * an optional default value
///
/// For callbacks there is also the `[builder_cb]` attribute. It accepts the
/// same parameters as above, but instead of a default value you have to give
/// the inner type of the "Into" type, since we cannot automatically infer
/// that from the type of the field.
///
/// # Examples:
///
/// ```
/// # use pwt_macros::builder;
/// #[builder]
/// struct Foo {
///     #[builder]
///     /// this is some field
///     some_field: String,
/// }
/// ```
///
/// will generate code like this:
///
/// ```
/// # struct Foo {
/// #   some_field: String
/// # }
/// impl Foo {
///     /// Set `some_field`
///     ///
///     /// this is some field
///     pub fn set_some_field(&mut self, some_field: String) {
///         self.some_field = some_field;
///     }
///
///     /// Builder style method to set `some_field`
///     pub fn some_field(mut self, some_field: String) -> Self {
///         self.set_some_field(some_field);
///         self
///     }
/// }
/// ```
///
/// If the type you need is an "Into" type you can do the following:
///
/// ```
/// # use pwt_macros::builder;
/// # trait IntoPropValue<T> {
/// #   fn into_prop_value(self) -> T;
/// # }
/// #[builder]
/// struct Foo {
///     #[builder(IntoPropValue, into_prop_value)]
///     /// this is some field
///     some_field: f32,
/// }
/// ```
///
/// which will generate code like this:
///
/// ```
/// # trait IntoPropValue<T> {
/// #   fn into_prop_value(self) -> T;
/// # }
/// # struct Foo {
/// #   some_field: f32,
/// # }
/// impl Foo {
///     /// Set `some_field`
///     ///
///     /// this is some field
///     pub fn set_some_field(&mut self, some_field: impl IntoPropValue<f32>) {
///         self.some_field = some_field.into_prop_value();
///     }
///
///     /// Builder style method to set `some_field`
///     pub fn some_field(mut self, some_field: impl IntoPropValue<f32>) -> Self {
///         self.set_some_field(some_field);
///         self
///     }
/// }
/// ```
///
/// or with a default value:
///
///
/// ```
/// # trait IntoPropValue<T> {
/// #   fn into_prop_value(self) -> T;
/// # }
/// # use pwt_macros::builder;
/// #[builder]
/// struct Foo {
///     #[builder(IntoPropValue, into_prop_value, 0.0)]
///     /// this is some field
///     some_field: f32,
/// }
/// ```
///
/// which will generate code like this:
///
/// ```
/// # trait IntoPropValue<T> {
/// #   fn into_prop_value(self) -> T;
/// # }
/// # struct Foo {
/// #   some_field: f32,
/// # }
/// impl Foo {
///     /// Set `some_field`
///     ///
///     /// this is some field
///     pub fn set_some_field(&mut self, some_field: impl IntoPropValue<Option<f32>>) {
///         self.some_field = some_field.into_prop_value().unwrap_or(0.0);
///     }
///
///     /// Builder style method to set `some_field`
///     pub fn some_field(mut self, some_field: impl IntoPropValue<Option<f32>>) -> Self {
///         self.set_some_field(some_field);
///         self
///     }
/// }
/// ```
///
/// For callbacks you have to specify the parameter type of the callback:
///
/// ```
/// # struct Callback<T> {
/// #   field: T
/// # }
/// # trait IntoEventCallback<T> {
/// #   fn into_event_callback(self) -> Option<Callback<T>>;
/// # }
/// # use pwt_macros::builder;
/// #[builder]
/// struct Foo {
///     #[builder_cb(IntoEventCallback, into_event_callback, f32)]
///     /// this is some field
///     on_something: Option<Callback<f32>>,
/// }
/// ```
///
/// which will generate code like this:
///
/// ```
/// # struct Callback<T> {
/// #   field: T
/// # }
/// # trait IntoEventCallback<T> {
/// #   fn into_event_callback(self) -> Option<Callback<T>>;
/// # }
/// # struct Foo {
/// #   on_something: Option<Callback<f32>>,
/// # }
/// impl Foo {
///     /// Set `on_something`
///     ///
///     /// this is some field
///     pub fn set_on_something(&mut self, on_something: impl IntoEventCallback<f32>) {
///         self.on_something = on_something.into_event_callback();
///     }
///
///     /// Builder style method to set `on_something`
///     pub fn some_field(mut self, on_something: impl IntoEventCallback<f32>) -> Self {
///         self.set_on_something(on_something);
///         self
///     }
/// }
/// ```
///
#[proc_macro_attribute]
pub fn builder(_attr: TokenStream, item: TokenStream) -> TokenStream {
    handle_builder_struct(item)
}

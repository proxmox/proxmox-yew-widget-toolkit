use proc_macro::TokenStream;
use syn::parse_macro_input;

mod widget;
use widget::*;

mod builder;
use builder::*;

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

mod props {
    use yew::{virtual_dom::Key, Classes};

    pub trait FieldBuilder {
        fn as_input_props(&self) -> &FieldStdProps;
        fn as_input_props_mut(&mut self) -> &mut FieldStdProps;
    }
    pub trait CssBorderBuilder {}
    pub trait EventSubscriber: Sized {
        fn as_listeners_mut(&mut self) -> &mut ListenersWrapper;
    }
    pub trait CssPaddingBuilder {}
    pub trait CssMarginBuilder {}
    pub trait AsCssStylesMut {
        fn as_css_styles_mut(&mut self) -> &mut CssStyles;
    }

    /// Holds the CSS styles to set on elements
    #[derive(Clone, Default, Debug, PartialEq)]
    pub struct CssStyles {}

    pub trait WidgetStyleBuilder {}

    pub trait WidgetBuilder: Sized {
        fn as_std_props_mut(&mut self) -> &mut WidgetStdProps;
        fn as_std_props(&self) -> &WidgetStdProps;

        fn class(mut self, class: impl Into<Classes>) -> Self {
            self.add_class(class);
            self
        }
        fn add_class(&mut self, class: impl Into<Classes>) {
            self.as_std_props_mut().class.push(class);
        }
    }
    pub trait AsClassesMut {
        fn as_classes_mut(&mut self) -> &mut Classes;
    }
    #[derive(PartialEq, Default, Clone)]
    pub struct FieldStdProps {}
    #[derive(PartialEq, Default, Clone)]
    pub struct WidgetStdProps {
        pub key: Option<Key>,
        pub class: Classes,
        pub styles: CssStyles,
    }
    #[derive(PartialEq, Default, Clone)]
    pub struct ListenersWrapper {}
}
use pwt_macros::widget;
use yew::prelude::*;

#[widget(pwt=crate, comp=Wrong, @input, @element)]
#[derive(Properties, PartialEq, Clone)]
struct Foo {}

impl Foo {
    fn new() -> Self {
        yew::props!(Self {})
    }
}

struct FooComp {}
impl Component for FooComp {
    type Message = ();
    type Properties = Foo;

    fn create(_ctx: &yew::Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, _ctx: &yew::Context<Self>) -> yew::Html {
        html! {<div>{"Foo"}</div>}
    }
}

fn main() {}

use std::marker::PhantomData;

use crate::css::{AlignItems, Display};
use crate::prelude::*;

use crate::props::{EventSubscriber, FieldBuilder, WidgetBuilder};
use crate::widget::{get_unique_element_id, Container};

use pwt_macros::widget;

#[cfg(doc)]
use crate::widget::{
    form::{Boolean, Checkbox, Field},
    Row,
};

/// Field Label (right side label).
///
/// Simply put a [Field] into a [Row], and append the label to the right side.
/// This is sometimes useful for [Checkbox] or [Boolean] fields, where you have much
/// empty space on the right side.
///
/// The label is clickable a toggles the value of a boolean or checkbox field.
#[widget(pwt=crate, comp=PwtFieldLabel<F>, @element)]
#[derive(Clone, PartialEq, Properties)]
pub struct FieldLabel<F: Clone + PartialEq + FieldBuilder + Properties + 'static> {
    pub field: F,

    pub label: Html,
}

impl<F: Clone + PartialEq + Properties + FieldBuilder> FieldLabel<F> {
    pub fn new(label: impl Into<Html>, field: F) -> Self {
        yew::props! { Self { label: label.into(), field } }
    }
}

pub struct PwtFieldLabel<F> {
    label_id: AttrValue,
    _phantom: PhantomData<F>,
}

impl<F: Clone + PartialEq + Properties + FieldBuilder + 'static> Component for PwtFieldLabel<F> {
    type Message = ();
    type Properties = FieldLabel<F>;

    fn create(_ctx: &Context<Self>) -> Self {
        let label_id = AttrValue::from(get_unique_element_id());
        Self {
            label_id,
            _phantom: PhantomData::<F>,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        Container::new()
            .with_std_props(&props.std_props)
            .listeners(&props.listeners)
            .class(Display::Flex)
            .class(AlignItems::Center)
            .class("pwt-gap-2")
            .with_child(props.field.clone().label_id(self.label_id.clone()))
            .with_child(html! {
                <span class="pwt-user-select-none" id={self.label_id.clone()}>
                    {props.label.clone()}
                </span>
            })
            .into()
    }
}

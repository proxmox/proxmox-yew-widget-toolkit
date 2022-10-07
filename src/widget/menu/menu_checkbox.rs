use std::rc::Rc;

use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};
use yew::html::{IntoEventCallback, IntoPropValue};

use crate::prelude::*;
use crate::widget::{Container};


#[derive(Clone, PartialEq, Properties)]
pub struct MenuCheckbox {
    pub text: AttrValue,

    /// Name of the form field (or radio-group value).
    pub name: Option<AttrValue>,

    /// Radio group name.
    pub group: Option<AttrValue>,

    #[prop_or_default]
    pub disabled: bool,

    pub checked: Option<bool>,

    pub on_change: Option<Callback<bool>>,
}

impl MenuCheckbox {

    /// Create a new menu button
    pub fn new(text: impl Into<AttrValue>) -> Self {
        yew::props!(Self {
            text: text.into()
        })
    }

    pub fn checked(mut self, checked: impl IntoPropValue<Option<bool>>) -> Self {
        self.checked = checked.into_prop_value();
        self
    }

    /// Builder style method to set the field name.
    pub fn name(mut self, name: impl IntoPropValue<Option<AttrValue>>) -> Self {
        self.set_name(name);
        self
    }

    /// Method to set the field name.
    pub fn set_name(&mut self, name: impl IntoPropValue<Option<AttrValue>>) {
        self.name = name.into_prop_value();
    }

    /// Builder style method to set the radio group name.
    pub fn group(mut self, group: impl IntoPropValue<Option<AttrValue>>) -> Self {
        self.set_group(group);
        self
    }

    /// Method to set the radio group name.
    pub fn set_group(&mut self, group: impl IntoPropValue<Option<AttrValue>>) {
        self.group = group.into_prop_value();
    }

    /// Builder style method to set the on_change callback.
    pub fn on_change(mut self, cb: impl IntoEventCallback<bool>) -> Self {
        self.on_change = cb.into_event_callback();
        self
    }

}

pub enum Msg {
    Toggle,
}

#[doc(hidden)]
pub struct PwtMenuCheckbox {
    checked: bool,
}

impl Component for PwtMenuCheckbox {
    type Message = Msg;
    type Properties = MenuCheckbox;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();
        Self {
            checked: props.checked.unwrap_or(false),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            /*
            Msg::FormCtxUpdate(form_ctx) => {
                self.form_ctx = Some(form_ctx);
                let value = self.get_value(ctx);
                let changed = self.value != value;
                self.value = value;
                changed
            }
             */
            Msg::Toggle => {
                if props.disabled { return false; }
                //self.set_value(ctx, !self.get_value(ctx));
                self.checked = !self.checked;
                if let Some(on_change) = &props.on_change {
                    on_change.emit(self.checked);
                }

                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let checked = self.checked;

        let icon_class = classes!(
            "fa",
            "fa-fw",
            if props.group.is_some() {
                if checked { "fa-check-circle-o" } else { "fa-circle-o" }
            } else {
                if checked { "fa-check-square-o" } else { "fa-square-o" }
            },
            "pwt-menu-item-icon",
        );
        let icon = html!{<i role="none" aria-hidden="true" class={icon_class}/>};

        let onclick = ctx.link().callback(|_| Msg::Toggle);
        let onkeydown = Callback::from({
            let link = ctx.link().clone();
            move |event: KeyboardEvent| {
                if event.key_code() == 32 {
                    link.send_message(Msg::Toggle);
                }
            }
        });

        Container::new()
            .class("pwt-menu-item")
            .attribute("tabindex", (!props.disabled).then(|| "-1"))
            .attribute("disabled", props.disabled.then(|| ""))
            .onclick(onclick)
            .onkeydown(onkeydown)
            .with_child(icon)
            .with_child(html!{<i class="pwt-menu-item-indent">{&props.text}</i>})
            .into()
     }
}

impl Into<VNode> for MenuCheckbox {
    fn into(self) -> VNode {
        let comp = VComp::new::<PwtMenuCheckbox>(Rc::new(self), None);
        VNode::from(comp)
    }
}

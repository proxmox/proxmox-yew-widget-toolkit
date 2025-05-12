use std::rc::Rc;

use html::{IntoEventCallback, IntoPropValue};
use pwt_macros::builder;
use yew::virtual_dom::{Key, VComp, VNode};

use crate::prelude::*;
use crate::{impl_yew_std_props_builder, tr};

use super::MessageBox;

#[derive(Clone, PartialEq, Properties)]
#[builder]
/// A dialog that can be used to let users confirm an action before it is taken.
pub struct ConfirmDialog {
    /// Yew component `ref`.
    #[prop_or_default]
    pub node_ref: NodeRef,

    /// The yew component key.
    #[prop_or_default]
    pub key: Option<Key>,

    /// The title of the dialog.
    #[prop_or_default]
    #[builder(IntoPropValue, into_prop_value)]
    pub title: AttrValue,

    /// A message that conveys what will be confirmed.
    #[prop_or(html!(tr!("Confirm this action?")))]
    #[builder(IntoPropValue, into_prop_value)]
    pub confirm_message: Html,

    /// An icon that will be shown in the dialogs message.
    #[prop_or("fa fa-exclamation-triangle".into())]
    #[builder(Into, into)]
    pub icon_class: Classes,

    /// A callback for an action that needs to be confirmed by the user.
    #[prop_or_default]
    #[builder_cb(IntoEventCallback, into_event_callback, ())]
    pub on_confirm: Option<Callback<()>>,

    /// A callback that will trigger if the user dismisses the action.
    #[prop_or_default]
    #[builder_cb(IntoEventCallback, into_event_callback, ())]
    pub on_dismiss: Option<Callback<()>>,

    /// A callback that will trigger if the dialog is closed, regardless of whether the action was
    /// confirmed or not.
    #[prop_or_default]
    #[builder_cb(IntoEventCallback, into_event_callback, ())]
    pub on_close: Option<Callback<()>>,
}

impl ConfirmDialog {
    pub fn new(title: impl Into<AttrValue>, confirm_message: impl Into<AttrValue>) -> Self {
        yew::props!(Self {
            title: title.into(),
            confirm_message: Some(html! {confirm_message.into()})
        })
    }

    impl_yew_std_props_builder!();
}

impl Default for ConfirmDialog {
    fn default() -> Self {
        yew::props!(Self {
            title: tr!("Confirm"),
        })
    }
}

struct PwtConfirmDialog {}

impl Component for PwtConfirmDialog {
    type Message = ();
    type Properties = ConfirmDialog;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let on_confirm = props.on_confirm.clone();
        let on_dismiss = props.on_dismiss.clone();
        let on_close = props.on_close.clone();

        MessageBox::new(props.title.clone(), props.confirm_message.clone())
            .node_ref(props.node_ref.clone())
            .buttons(super::MessageBoxButtons::YesNo)
            .icon_class(props.icon_class.clone())
            .on_close(ctx.link().callback(move |confirm| {
                if confirm {
                    if let Some(on_confirm) = &on_confirm {
                        on_confirm.emit(());
                    }
                } else if let Some(on_dismiss) = &on_dismiss {
                    on_dismiss.emit(());
                }

                if let Some(on_close) = &on_close {
                    on_close.emit(());
                }
            }))
            .into()
    }
}

impl From<ConfirmDialog> for VNode {
    fn from(value: ConfirmDialog) -> Self {
        let key = value.key.clone();
        VComp::new::<PwtConfirmDialog>(Rc::new(value), key).into()
    }
}

use serde_json::json;

use web_sys::HtmlInputElement;
use gloo_events::EventListener;
use wasm_bindgen::{JsCast, JsValue, UnwrapThrowExt};

use yew::prelude::*;
use yew::html::{IntoPropValue, IntoEventCallback};
use yew::virtual_dom::{Key, VNode};

use crate::prelude::*;
use crate::props::{WidgetStdProps, FieldStdProps, RenderFn};
use crate::widget::Tooltip;
use crate::widget::form::Input;

use pwt_macros::widget;

#[widget(PwtDropdown, @input, @element)]
#[derive(Clone, PartialEq, Properties)]
pub struct Dropdown {
    #[prop_or_default]
    pub editable: bool,

    pub picker: RenderFn<Callback<Key>>,
    pub tip: Option<VNode>,

    pub on_change: Option<Callback<String>>,
    pub value: Option<String>,

    pub popup_type: Option<String>,
}

impl Dropdown {

    // Create a new instance
    pub fn new(picker: RenderFn<Callback<Key>>) -> Self {
        yew::props!{ Self { picker } }
    }

    /// Use common properties from [WidgetStdProps]
    pub fn with_std_props(mut self, props: &WidgetStdProps) -> Self {
        self.std_props = props.clone();
        self
    }

    /// Use common properties from [FieldStdProps]
    pub fn with_input_props(mut self, props: &FieldStdProps) -> Self {
        self.input_props = props.clone();
        self
    }

    /// Builder style method to set the editable flag
    pub fn editable(mut self, editable: bool) -> Self {
        self.set_editable(editable);
        self
    }

    /// Method to set the editable flag
    pub fn set_editable(&mut self, editable: bool) {
        self.editable = editable;
    }

    /// Builder style method to set the on_change callback
    pub fn on_change(mut self, cb: impl IntoEventCallback<String>) -> Self {
        self.on_change = cb.into_event_callback();
        self
    }

    /// Builder style method to set the value
    pub fn value(mut self, value: impl IntoPropValue<Option<String>>) -> Self {
        self.set_value(value);
        self
    }

    /// Method to set the value
    pub fn set_value(&mut self, value: impl IntoPropValue<Option<String>>) {
        self.value = value.into_prop_value();
    }

    /// Builder style method to set the popup_type
    pub fn popup_type(mut self, popup_type: impl IntoPropValue<Option<String>>) -> Self {
        self.set_popup_type(popup_type);
        self
    }

    /// Method to set the popup_type
    pub fn set_popup_type(&mut self, popup_type: impl IntoPropValue<Option<String>>) {
        self.popup_type = popup_type.into_prop_value();
    }

    /// Builder style method to set the tooltip
    pub fn tip(mut self, tip: impl IntoPropValue<Option<VNode>>) -> Self {
        self.set_tip(tip);
        self
    }

    /// Method to set the tooltip
    pub fn set_tip(&mut self, tip: impl IntoPropValue<Option<VNode>>) {
        self.tip = tip.into_prop_value();
    }
}

pub enum Msg {
    TogglePicker,
    HidePicker,
    ShowPicker,
    DialogClosed,
    Select(Key),
    Input(String),
}

#[doc(hidden)]
pub struct PwtDropdown {
    show: bool,
    last_show: bool, // track changes
    value: String,
    // fire on_change() event delayed, after the dialog is closed, so that
    // other widget can grep the focus after a change (if the want)
    pending_change: bool,
    mousedown_listener: Option<EventListener>,
    picker_ref: NodeRef,
    picker_id: String,
    popper: Option<JsValue>,
}

impl PwtDropdown {

    // focus the input elelent (after closing the dropdown dialog)
    // just to be sure (Dialog should do this automatically)
    fn restore_focus(&mut self, props: &Dropdown) {
        if let Some(el) = props.std_props.node_ref.cast::<web_sys::HtmlElement>() {
            let _ = el.focus();
        }
    }
}

impl Component for PwtDropdown {
    type Message = Msg;
    type Properties = Dropdown;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            show: false,
            last_show: false,
            pending_change: false,
            value: ctx.props().value.clone().unwrap_or_else(|| String::new()),
            mousedown_listener: None,
            picker_ref: NodeRef::default(),
            picker_id: crate::widget::get_unique_element_id(),
            popper: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::DialogClosed => {
                self.show = false;
                //log::info!("DialogClosed");
                self.restore_focus(props);
                if self.pending_change {
                    self.pending_change = false;
                    //log::info!("Pending Change {}", self.value);
                    if let Some(on_change) = &ctx.props().on_change {
                        on_change.emit(self.value.clone());
                    }
                }
                true
            }
            Msg::TogglePicker => {
                if props.input_props.disabled { return false; }
                //log::info!("TogglePicker");
                yew::Component::update(self, ctx, if self.show { Msg::HidePicker } else {Msg::ShowPicker})
            }
            Msg::HidePicker => {
                // Note: close_dialog() is async, so we use the
                // onclose handler (Msg::DialogClosed) to wait for
                // the real close (else restore_focus() does not work)
                if let Some(dialog_node) = self.picker_ref.get() {
                    crate::close_dialog(dialog_node);
                }
                //log::info!("HidePicker {}", self.show);
                false
            }
            Msg::ShowPicker => {
                self.show = true;
                //log::info!("ShowPicker {}", self.show);
                true
            }
            Msg::Select(key) => {
                self.value = key.to_string();
                if self.show {
                    self.pending_change = true;
                    yew::Component::update(self, ctx, Msg::HidePicker)
                } else {
                    //log::info!("Select {} {}", key, value);
                    if let Some(on_change) = &ctx.props().on_change {
                        on_change.emit(self.value.clone());
                    }

                    true
                }
            }
            Msg::Input(value) => {
                //log::info!("Input {}", value);
                if props.editable {
                    self.value = value;
                    if let Some(on_change) = &ctx.props().on_change {
                        on_change.emit(self.value.clone());
                    }
                }
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let disabled = props.input_props.disabled;

        let onclick = ctx.link().callback(|e: MouseEvent| {
            let event = e.unchecked_into::<Event>();
            event.set_cancel_bubble(false);
            Msg::TogglePicker
        });
        let trigger_onclick = onclick.clone();

        let onkeydown = Callback::from({
            let link = ctx.link().clone();
            let show = self.show;
            move |event: KeyboardEvent| {
                match event.key_code() {
                    27 => { // ESC
                        if !show { return; } // allow default (close dialog)
                        link.send_message(Msg::HidePicker);
                    }
                    40 => { // Down
                        link.send_message(Msg::ShowPicker);
                    }
                     _ => return,
                }
                event.prevent_default();
            }
        });

        let oninput = ctx.link().callback(|event: InputEvent| {
            let input: HtmlInputElement = event.target_unchecked_into();
            Msg::Input(input.value())
        });

        let link = ctx.link().clone();
        let onselect = Callback::from(move |key: Key| {
            link.send_message(Msg::Select(key));
        });

        let trigger_cls = classes!{
            "fa",
            "fa-lg",
            "pwt-input-trigger-icon",
            "fa-caret-down",
            self.show.then(|| "fa-rotate-180"),
            disabled.then(|| "disabled"),
        };

        let data_show = self.show.then(|| "");

        let value = props.value.clone().unwrap_or_else(|| self.value.clone());

        let oncancel = ctx.link().callback(|event: Event| {
            event.stop_propagation();
            event.prevent_default();
            Msg::HidePicker
        });

        let input: Html = Input::new()
            .with_std_props(&props.std_props)
            .with_input_props(&props.input_props)
            .class("pwt-input")
            .class("pwt-w-100")
            .attribute("value", value)
            .attribute("type", "text")
            .attribute("role", "combobox")
            .attribute("aria-expanded", if self.show { "true" } else { "false" })
            .attribute("aria-controls", self.picker_id.clone())
            .attribute("aria-haspopup", props.popup_type.clone())
            .oninput(oninput)
            .onkeydown(onkeydown)
            .onclick(onclick)
            .into();

        let onclose = ctx.link().callback(|_| Msg::DialogClosed);

        let input = html!{
            <div>
                <div style="position:relative;"> // allows us to position the trigger-icon relatively
                    {input}
                    <i onclick={trigger_onclick} class={trigger_cls}></i>
                </div>
                <dialog id={self.picker_id.clone()} ref={self.picker_ref.clone()} {onclose} {oncancel} class="pwt-dropdown" data-show={data_show}>
                    {props.picker.apply(&onselect)}
                </dialog>
            </div>
        };

        let mut tooltip = Tooltip::new()
            .with_child(input);

        if !self.show {
            tooltip.set_tip(props.tip.clone());
        }

        tooltip.into()
    }

    fn destroy(&mut self, _ctx: &Context<Self>) {
        // always close the dialog
        if let Some(dialog_node) = self.picker_ref.get() {
            crate::close_dialog(dialog_node);
        }
    }


    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            let props = ctx.props();
            let link = ctx.link().clone();
            let window = web_sys::window().unwrap();
            let picker_ref = self.picker_ref.clone();

            let opts = json!({
                "placement": "bottom-start",
                "strategy": "fixed",
                "modifiers": [
                    {
                        "name": "preventOverflow",
                        "options": {
                            "mainAxis": true, // true by default
                            "altAxis": true, // false by default
                        },
                    },
                    {
                        "name": "flip",
                        "options": {
                            "fallbackPlacements": [], // disable fallbacks
                        },
                    },
                ],
            });

            let opts = JsValue::from_serde(&opts).unwrap();

            if let Some(content_node) = props.std_props.node_ref.get() {
                if let Some(picker_node) = self.picker_ref.get() {
                    self.popper = Some(crate::create_popper(content_node, picker_node, &opts));
                }
            }

            self.mousedown_listener = Some(EventListener::new(
                &window,
                "mousedown",
                move |e: &Event| {
                    let e = e.dyn_ref::<web_sys::MouseEvent>().unwrap_throw();

                    if let Some(el) = picker_ref.cast::<web_sys::Element>() {
                        let x = e.client_x() as f64;
                        let y = e.client_y() as f64;

                        let rect = el.get_bounding_client_rect();
                        if x > rect.left() && x < rect.right() && y > rect.top() && y < rect.bottom() {
                            return;
                        }

                        link.send_message(Msg::HidePicker);
                    }
                },
            ));

            if props.input_props.autofocus {
                if let Some(el) = props.std_props.node_ref.cast::<web_sys::HtmlElement>() {
                    let _ = el.focus();
                }
            }
        }

        if let Some(popper) = &self.popper {
            crate::update_popper(popper);
        }

        if self.show != self.last_show {
            self.last_show = self.show;
            if let Some(dialog_node) = self.picker_ref.get() {
                if self.show {
                    crate::show_modal_dialog(dialog_node);
                    focus_selected_element(&self.picker_ref);
                } else {
                    crate::close_dialog(dialog_node);
                }
            }
        }
    }
}

// Focus selected element
// Note: this scrolls the selected element into the view.
pub fn focus_selected_element(node_ref: &NodeRef) {
    if let Some(el) = node_ref.cast::<web_sys::Element>() {
        if let Ok(Some(selected_el)) = el.query_selector(".selected") {
            let _ = selected_el.dyn_into::<web_sys::HtmlElement>().unwrap().focus();
        }
    }
}

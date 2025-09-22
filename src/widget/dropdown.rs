use html::Scope;
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;

use yew::html::{IntoEventCallback, IntoPropValue};
use yew::prelude::*;

use crate::prelude::*;
use crate::props::{IntoOptionalRenderFn, RenderFn};
use crate::widget::{Container, Input, Tooltip, Trigger};

use pwt_macros::{builder, widget};

use crate::dom::align::{AlignOptions, AutoFloatingPlacement, GrowDirection, Point};

use crate::dom::focus::{element_is_focusable, get_first_focusable, FocusTracker};

/// Parameters passed to the [Dropdown] picker callback.
#[derive(Clone)]
pub struct DropdownController {
    link: Scope<PwtDropdown>,
}

impl DropdownController {
    /// Change the [Dropdown] input element value.
    pub fn change_value(&self, value: String) {
        self.link.send_message(Msg::ChangeValue(value));
    }

    /// Convenience function to generate Callback.
    pub fn on_select_callback<S: std::fmt::Display>(&self) -> Callback<S> {
        let controller = self.clone();
        Callback::from(move |key: S| {
            controller.change_value(key.to_string());
        })
    }
}
/// Base widget to implement [Combobox](crate::widget::form::Combobox) like widgets.
///
/// # Note
///
/// This widget does not interact with a form context, so it ignore
/// form context related properties like (name, required, submit,
/// submit_empty).
#[widget(pwt=crate, comp=PwtDropdown, @input, @element)]
#[derive(Clone, PartialEq, Properties)]
#[builder]
pub struct Dropdown {
    /// Make the input editable.
    ///
    /// This will be forced to `false` if you pass a `render_value` callback.
    #[prop_or_default]
    #[builder]
    pub editable: bool,

    /// Function to generate the picker widget.
    pub picker: RenderFn<DropdownController>,

    /// Tooltip for the input
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub tip: Option<AttrValue>,

    /// Value change callback.
    #[builder_cb(IntoEventCallback, into_event_callback, String)]
    #[prop_or_default]
    pub on_change: Option<Callback<String>>,

    /// Sets the input to the provided value.
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub value: Option<String>,

    /// Flag to indicate if the value is valid.
    #[prop_or(true)]
    #[builder]
    pub valid: bool,

    /// Sets the "aria-haspopup" property.
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub popup_type: Option<AttrValue>,

    /// Display the output of this function instead of value.
    ///
    /// Note: dropdowns using this feature are not editable (editable property is ignored)!
    #[builder_cb(IntoOptionalRenderFn, into_optional_render_fn, AttrValue)]
    #[prop_or_default]
    pub render_value: Option<RenderFn<AttrValue>>,

    /// Icons to show on the left (false) or right(true) side of the input
    #[prop_or_default]
    #[builder]
    pub trigger: Vec<(Trigger, bool)>,
}

impl Dropdown {
    // Create a new instance.
    pub fn new(picker: impl Into<RenderFn<DropdownController>>) -> Self {
        yew::props! { Self { picker: picker.into() } }
    }

    /// Builder style method to add an icon.
    pub fn with_trigger(mut self, trigger: impl Into<Trigger>, right: bool) -> Self {
        self.add_trigger(trigger, right);
        self
    }

    /// Method to add an icon.
    pub fn add_trigger(&mut self, trigger: impl Into<Trigger>, right: bool) {
        self.trigger.push((trigger.into(), right));
    }
}

pub enum Msg {
    TogglePicker,
    HidePicker,
    ShowPicker,
    ChangeValue(String),
    Input(String),
    MouseDownInput,
    FocusChange(bool),
}

#[doc(hidden)]
pub struct PwtDropdown {
    show: bool,
    last_show: bool, // track changes
    value: String,
    // fire on_change() event delayed, after the popover is closed, so that
    // other widget can grep the focus after a change (if the want)
    pending_change: bool,
    change_from_input: bool,
    focus_on_field: bool,

    node_ref: NodeRef,
    input_ref: NodeRef,
    picker_ref: NodeRef,
    dropdown_ref: NodeRef,
    picker_id: String,
    picker_placer: Option<AutoFloatingPlacement>,
    focus_tracker: FocusTracker,
}

impl PwtDropdown {
    // focus the input elelent (after closing the dropdown popover)
    fn restore_focus(&mut self) {
        if let Some(el) = self.node_ref.cast::<web_sys::HtmlElement>() {
            let _ = el.focus();
        }
    }

    fn update_picker_placer(&mut self, _props: &Dropdown) {
        self.picker_placer = match AutoFloatingPlacement::new(
            self.dropdown_ref.clone(),
            self.picker_ref.clone(),
            AlignOptions::new(
                Point::BottomStart,
                Point::TopStart,
                GrowDirection::TopBottom,
            )
            .viewport_padding(5.0)
            .align_width(true),
        ) {
            Ok(placer) => Some(placer),
            Err(err) => {
                log::error!("error creating placer: {}", err.to_string());
                None
            }
        };
    }
}

impl Component for PwtDropdown {
    type Message = Msg;
    type Properties = Dropdown;

    fn create(ctx: &Context<Self>) -> Self {
        let focus_tracker = FocusTracker::new(ctx.link().callback(Msg::FocusChange));
        Self {
            show: false,
            last_show: false,
            pending_change: false,
            value: ctx.props().value.clone().unwrap_or_default(),
            focus_on_field: false,
            change_from_input: false,
            node_ref: NodeRef::default(),
            input_ref: NodeRef::default(),
            picker_ref: NodeRef::default(),
            dropdown_ref: NodeRef::default(),
            picker_id: crate::widget::get_unique_element_id(),
            picker_placer: None,
            focus_tracker,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::TogglePicker => {
                if props.input_props.disabled {
                    return false;
                }
                //log::info!("TogglePicker");
                yew::Component::update(
                    self,
                    ctx,
                    if self.show {
                        Msg::HidePicker
                    } else {
                        Msg::ShowPicker
                    },
                )
            }
            Msg::HidePicker => {
                if let Some(popover_node) = self.picker_ref.get() {
                    crate::hide_popover(popover_node);
                }
                self.show = false;
                self.restore_focus();
                if self.pending_change {
                    self.pending_change = false;
                    //log::info!("Pending Change {}", self.value);
                    if let Some(on_change) = &ctx.props().on_change {
                        on_change.emit(self.value.clone());
                    }
                }
                //log::info!("HidePicker {}", self.show);
                true
            }
            Msg::ShowPicker => {
                if props.input_props.disabled {
                    return false;
                }
                self.show = true;
                //log::info!("ShowPicker {}", self.show);
                true
            }
            Msg::ChangeValue(value) => {
                self.value = value;
                if self.show {
                    self.pending_change = true;
                    if !self.change_from_input {
                        yew::Component::update(self, ctx, Msg::HidePicker)
                    } else {
                        self.change_from_input = false;
                        true
                    }
                } else {
                    //log::info!("ChangeValue {} {}", key, value);
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
                    if self.show {
                        self.change_from_input = true;
                    }
                    if let Some(on_change) = &ctx.props().on_change {
                        on_change.emit(self.value.clone());
                    }
                }
                true
            }
            Msg::MouseDownInput => {
                if ctx.props().editable {
                    self.focus_on_field = true;
                }
                true
            }
            Msg::FocusChange(has_focus) => {
                if !has_focus {
                    self.show = false;
                }
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let disabled = props.input_props.disabled;
        let editable = props.editable;

        let onclick = ctx.link().batch_callback(move |e: MouseEvent| {
            let event = e.unchecked_into::<Event>();
            event.stop_propagation();
            // toggle on click only when the field is not editable
            if editable {
                vec![Msg::MouseDownInput, Msg::ShowPicker]
            } else {
                vec![Msg::TogglePicker]
            }
        });
        let trigger_onclick = ctx.link().callback(move |e: MouseEvent| {
            let event = e.unchecked_into::<Event>();
            event.stop_propagation();
            Msg::TogglePicker
        });

        let onkeydown = Callback::from({
            let link = ctx.link().clone();
            let show = self.show;
            move |event: KeyboardEvent| {
                match event.key().as_str() {
                    "Escape" => {
                        if !show {
                            return;
                        } // allow default (close popover)
                        link.send_message(Msg::HidePicker);
                    }
                    "ArrowDown" => {
                        link.send_message(Msg::ShowPicker);
                    }
                    _ => return,
                }
                event.prevent_default();
                event.stop_propagation();
            }
        });

        let oninput = ctx.link().callback(|event: InputEvent| {
            let input: HtmlInputElement = event.target_unchecked_into();
            Msg::Input(input.value())
        });

        let controller = DropdownController {
            link: ctx.link().clone(),
        };

        let data_show = self.show.then_some("true");

        let value = props.value.clone().unwrap_or_else(|| self.value.clone());

        let input: Html = if let Some(render_value) = &props.render_value {
            let rendered_value = if let Some(placeholder) = &props.input_props.placeholder {
                if value.is_empty() {
                    Container::new()
                        .with_child(placeholder)
                        .class("pwt-opacity-50")
                        .into()
                } else {
                    render_value.apply(&AttrValue::from(value.clone()))
                }
            } else {
                render_value.apply(&AttrValue::from(value.clone()))
            };

            Container::new()
                .listeners(&props.listeners)
                .class("pwt-flex-fill")
                .class("pwt-input-content")
                .class((!props.editable).then_some("non-editable"))
                .attribute(
                    "tabindex",
                    props.input_props.tabindex.unwrap_or(0).to_string(),
                )
                .attribute("role", "combobox")
                .attribute("aria-expanded", if self.show { "true" } else { "false" })
                .attribute("aria-controls", self.picker_id.clone())
                .attribute("aria-haspopup", props.popup_type.clone())
                .attribute("aria-required", props.input_props.required.then_some(""))
                .attribute("aria-label", props.input_props.aria_label.clone())
                .attribute("aria-labelledby", props.input_props.label_id.clone())
                .attribute("aria-live", "assertive")
                .with_child(rendered_value)
                .with_child(
                    Input::new()
                        .name(props.input_props.name.clone())
                        .disabled(props.input_props.disabled)
                        .required(props.input_props.required)
                        .onpointerdown(ctx.link().callback(|_| Msg::MouseDownInput))
                        .attribute("value", value)
                        .attribute("type", "hidden"),
                )
                .onkeydown(onkeydown)
                .into_html_with_ref(self.input_ref.clone())
        } else {
            Input::new()
                .with_input_props(&props.input_props)
                .listeners(&props.listeners)
                .class("pwt-flex-fill")
                .class((!props.editable).then_some("non-editable"))
                .attribute("value", value)
                .attribute("type", "text")
                .attribute("role", "combobox")
                .attribute("readonly", (!props.editable).then_some(""))
                .attribute("aria-expanded", if self.show { "true" } else { "false" })
                .attribute("aria-controls", self.picker_id.clone())
                .attribute("aria-haspopup", props.popup_type.clone())
                .oninput(oninput)
                .onpointerdown(ctx.link().callback(|_| Msg::MouseDownInput))
                .onkeydown(onkeydown)
                .into_html_with_ref(self.input_ref.clone())
        };

        let trigger_cls = classes! {
            "fa",
            "pwt-dropdown-icon",
            "pwt-pointer",
            if self.show { "fa-angle-up" } else { "fa-angle-down" },
            disabled.then_some("disabled"),
        };

        let mut select = Container::new()
            .class("pwt-input")
            .class("pwt-input-type-text")
            .class(self.show.then_some("picker-open"))
            .class(disabled.then_some("disabled"))
            .class("pwt-w-100")
            .class(if props.valid {
                "is-valid"
            } else {
                "is-invalid"
            })
            .onclick(onclick);

        for (trigger, right) in &props.trigger {
            if !right {
                let outer_class = "pwt-flex-fill-first-child pwt-d-flex pwt-align-self-center";
                select.add_child(html! {<div class={outer_class}>{trigger}</div>});
            }
        }

        select.add_child(input);

        for (trigger, right) in &props.trigger {
            if *right {
                let outer_class = "pwt-flex-fill-first-child pwt-d-flex pwt-align-self-center";
                select.add_child(html! {<div class={outer_class}>{trigger}</div>});
            }
        }

        select
            .add_child(html! {<i onclick={trigger_onclick} tabindex="-1" class={trigger_cls}></i>});

        let dropdown = Container::new()
            .onfocusin(self.focus_tracker.get_focus_callback(true))
            .onfocusout(self.focus_tracker.get_focus_callback(false))
            .with_child(select.into_html_with_ref(self.dropdown_ref.clone()))
            .with_child(
                Container::new()
                    .attribute("popover", "manual")
                    .attribute("tabindex", "-1")
                    .class("pwt-dialog")
                    .class("pwt-dropdown")
                    .attribute("id", self.picker_id.clone())
                    .attribute("data-show", data_show)
                    .onkeydown(ctx.link().batch_callback(|event: KeyboardEvent| {
                        if event.key() == "Escape" {
                            // handle escape ourselves since it's a non modal popover
                            event.prevent_default();
                            event.stop_propagation();
                            Some(Msg::HidePicker)
                        } else {
                            None
                        }
                    }))
                    .oncancel(ctx.link().callback(|event: Event| {
                        event.stop_propagation();
                        event.prevent_default();
                        Msg::HidePicker
                    }))
                    .with_optional_child(self.show.then(|| props.picker.apply(&controller)))
                    .into_html_with_ref(self.picker_ref.clone()),
            );

        let mut tooltip = Tooltip::new(dropdown).with_std_props(&props.std_props);

        if !self.show {
            tooltip.set_tip(props.tip.clone());
        }

        tooltip.into()
    }

    fn destroy(&mut self, _ctx: &Context<Self>) {
        // always close the popover
        if let Some(popover) = self.picker_ref.get() {
            crate::hide_popover(popover);
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            let props = ctx.props();

            self.update_picker_placer(props);

            if props.input_props.autofocus {
                if let Some(el) = self.input_ref.cast::<web_sys::HtmlElement>() {
                    let _ = el.focus();
                }
            }
        }

        if self.show != self.last_show {
            self.last_show = self.show;
            if let Some(popover_node) = self.picker_ref.get() {
                if self.show {
                    crate::show_popover(popover_node);
                    if self.focus_on_field {
                        if let Some(el) = self.input_ref.cast::<web_sys::HtmlElement>() {
                            let _ = el.focus();
                        }
                    } else {
                        focus_selected_element(&self.picker_ref);
                    }
                } else {
                    crate::hide_popover(popover_node);
                }
            }
        }

        // update picker placement after we opened/closed to cope with a bug that only seems to
        // affect webkit based browsers like Safari
        if let Some(placer) = &self.picker_placer {
            if let Err(err) = placer.update() {
                log::error!("error updating placement: {}", err.to_string());
            }
        }

        self.focus_on_field = false;
    }
}

// Focus selected element
// Note: this scrolls the selected element into the view.
pub fn focus_selected_element(node_ref: &NodeRef) {
    if let Some(el) = node_ref.cast::<web_sys::HtmlElement>() {
        if let Ok(Some(selected_el)) = el.query_selector(".selected") {
            let selected_el = selected_el.dyn_into::<web_sys::HtmlElement>().unwrap();
            if element_is_focusable(&selected_el) {
                let _ = el.focus();
            } else if let Some(focusable_el) = get_first_focusable(selected_el.into()) {
                let _ = focusable_el.focus();
            }
        }
    }
}

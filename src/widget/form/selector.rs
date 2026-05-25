use anyhow::{Error, bail};
use derivative::Derivative;
use serde_json::Value;

use yew::html::{IntoEventCallback, IntoPropValue};
use yew::prelude::*;
use yew::virtual_dom::Key;

use super::{
    IntoValidateFn, ManagedField, ManagedFieldContext, ManagedFieldMaster, ManagedFieldScopeExt,
    ManagedFieldState, ValidateFn,
};
use crate::AsyncAbortGuard;
use crate::css;
use crate::prelude::*;
use crate::props::{IntoLoadCallback, IntoOptionalRenderFn, LoadCallback, RenderFn};
use crate::state::DataStore;
use crate::state::Selection;
use crate::widget::{Container, Dropdown, DropdownController, Mask, Trigger, error_message};

use pwt_macros::{builder, widget};

/// Parameters passed to the [Selector::picker] callback.
pub struct SelectorRenderArgs<S: DataStore> {
    /// The [DataStore] used by the [Selector].
    pub store: S,
    /// The selection.
    pub selection: Selection,
    /// Dropdown controller.
    pub controller: DropdownController,
}

pub type PwtSelector<S> = ManagedFieldMaster<SelectorField<S>>;

/// Helper widget to implement [Combobox](super::Combobox) like selectors.
///
/// This helper simplifies the implementation of  [Combobox](super::Combobox) like
/// selectors with complex layouts (table, trees).
///
/// - Extends the [Dropdown] widget.
///
/// - Use a shared `DataStore` as data storage (either a
/// [Store](crate::state::Store) or
/// [TreeStore](crate::state::TreeStore)).
///
/// - Ability to load data using a [LoadCallback] (with reasonable
/// error handling).
///
/// - Handles [FormContext](super::FormContext) interaction.
///
/// # Note
///
/// Please use a trackable [LoadCallback] to avoid unnecessary
/// reloads.
#[widget(pwt=crate, comp=ManagedFieldMaster<SelectorField<S>>, @input)]
#[derive(Derivative, Properties)]
#[derivative(Clone(bound = ""), PartialEq(bound = ""))]
#[builder]
pub struct Selector<S: DataStore + 'static> {
    store: S,
    /// The default value.
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub default: Option<AttrValue>,

    /// Force value.
    ///
    /// To implement controlled components (for use without a FormContext).
    /// This is ignored if the field has a name.
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub value: Option<AttrValue>,

    /// Make the input editable.
    #[prop_or_default]
    #[builder]
    pub editable: bool,

    /// Autoselect flag.
    ///
    /// If there is no default, automatically select the first loaded
    /// data item.
    #[prop_or_default]
    #[builder]
    pub autoselect: bool,

    /// Change callback
    #[builder_cb(IntoEventCallback, into_event_callback, Key)]
    #[prop_or_default]
    pub on_change: Option<Callback<Key>>,

    /// Picker render function
    pub picker: RenderFn<SelectorRenderArgs<S>>,
    /// Validate callback.
    #[prop_or_default]
    pub validate: Option<ValidateFn<(String, S)>>,
    /// Data loader callback.
    #[prop_or_default]
    pub loader: Option<LoadCallback<S::Collection>>,

    /// Display the output of this function instead of displaying values directly.
    ///
    /// Note: selectors using this feature are not editable (editable property is ignored)!
    #[builder_cb(IntoOptionalRenderFn, into_optional_render_fn, AttrValue)]
    #[prop_or_default]
    pub render_value: Option<RenderFn<AttrValue>>,

    /// Icons to show on the left (false) or right(true) side of the input
    #[prop_or_default]
    #[builder]
    pub trigger: Vec<(Trigger, bool)>,
}

impl<S: DataStore> Selector<S> {
    /// Creates a new instance
    pub fn new(store: S, picker: impl Into<RenderFn<SelectorRenderArgs<S>>>) -> Self {
        yew::props!(Self {
            store,
            picker: picker.into(),
        })
    }

    /// Builder style method to set the validate callback
    pub fn validate(mut self, validate: impl IntoValidateFn<(String, S)>) -> Self {
        self.set_validate(validate);
        self
    }

    /// Method to set the validate callback
    pub fn set_validate(&mut self, validate: impl IntoValidateFn<(String, S)>) {
        self.validate = validate.into_validate_fn();
    }

    /// Builder style method to set the load callback.
    pub fn loader(mut self, callback: impl IntoLoadCallback<S::Collection>) -> Self {
        self.set_loader(callback);
        self
    }

    /// Method to set the load callback.
    pub fn set_loader(&mut self, callback: impl IntoLoadCallback<S::Collection>) {
        self.loader = callback.into_load_callback();
    }

    /// Builder style method to add an trigger
    pub fn with_trigger(mut self, trigger: impl Into<Trigger>, right: bool) -> Self {
        self.add_trigger(trigger, right);
        self
    }

    /// Method to add an trigger
    pub fn add_trigger(&mut self, trigger: impl Into<Trigger>, right: bool) {
        self.trigger.push((trigger.into(), right));
    }
}

pub enum Msg<S: DataStore> {
    Select(String),
    DataChange,
    LoadResult(Result<S::Collection, Error>),
    DeleteKey,
}

#[doc(hidden)]
pub struct SelectorField<S: DataStore> {
    state: ManagedFieldState,
    selection: Selection,
    load_error: Option<String>,
    _store_observer: S::Observer,
    abort_load_guard: Option<AsyncAbortGuard>,
}

impl<S: DataStore> std::ops::Deref for SelectorField<S> {
    type Target = ManagedFieldState;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl<S: DataStore> std::ops::DerefMut for SelectorField<S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state
    }
}

impl<S: DataStore + 'static> SelectorField<S> {
    fn load(&mut self, ctx: &ManagedFieldContext<Self>) {
        let props = ctx.props();
        let link = ctx.link().clone();
        self.abort_load_guard = None; // abort any previous load
        if let Some(loader) = &props.loader {
            if !props.is_disabled() {
                let loader = loader.clone();
                self.abort_load_guard = Some(AsyncAbortGuard::spawn(async move {
                    let res = loader.apply().await;
                    link.send_message(Msg::LoadResult(res));
                }));
            }
        } else {
            // if we have no loader, we simply trigger a redraw (ignoring the result)
            link.send_message(Msg::DataChange);
        }
    }
}

#[derive(PartialEq)]
pub struct ValidateClosure<S: DataStore> {
    required: bool,
    store: S,
    validate: Option<ValidateFn<(String, S)>>,
}

impl<S: DataStore + 'static> ManagedField for SelectorField<S> {
    type Message = Msg<S>;
    type Properties = Selector<S>;
    type ValidateClosure = ValidateClosure<S>;

    fn validation_args(props: &Self::Properties) -> Self::ValidateClosure {
        ValidateClosure {
            required: props.input_props.required,
            store: props.store.clone(),
            validate: props.validate.clone(),
        }
    }

    fn validator(props: &Self::ValidateClosure, value: &Value) -> Result<Value, Error> {
        let value = match value {
            Value::Null => String::new(),
            Value::String(v) => v.clone(),
            _ => return Err(Error::msg(tr!("got wrong data type."))),
        };

        if value.is_empty() {
            if props.required {
                bail!("Field may not be empty.");
            } else {
                return Ok(Value::String(String::new()));
            }
        }

        if !props.store.is_empty() {
            if let Some(validate) = &props.validate {
                validate.apply(&(value.clone(), props.store.clone()))?;
            }
        } else {
            // Return Ok if we have no data (i.e. because eof load error),
            // so that we can still edit/update other form properties.
        }

        Ok(Value::String(value))
    }

    fn create(ctx: &ManagedFieldContext<Self>) -> Self {
        let props = ctx.props();

        let value: Value = match &props.value {
            Some(value) => value.to_string().into(),
            None => Value::Null,
        };

        let default = match &props.default {
            Some(default) => default.to_string().into(),
            None => Value::Null,
        };

        let selection = Selection::new();
        if let Some(s) = value.as_str() {
            if !s.is_empty() {
                selection.select(s.to_string());
            }
        }

        let _store_observer = props
            .store
            .add_listener(ctx.link().callback(|_| Msg::DataChange));

        let mut me = Self {
            state: ManagedFieldState::new(value, default),
            selection,
            load_error: None,
            _store_observer,
            abort_load_guard: None,
        };

        me.load(ctx);

        me
    }

    fn value_changed(&mut self, ctx: &super::ManagedFieldContext<Self>) {
        let props = ctx.props();
        let key = self
            .value
            .as_str()
            .map(|s| Key::from(s))
            .unwrap_or(Key::from(""));

        self.selection.select(key.clone());

        if let Some(on_change) = &props.on_change {
            on_change.emit(key);
        }
    }

    fn update(&mut self, ctx: &ManagedFieldContext<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::DeleteKey => {
                if !props.editable {
                    ctx.link().update_value(String::new());
                }
                false
            }
            Msg::LoadResult(res) => {
                match res {
                    Ok(data) => {
                        self.load_error = None;
                        props.store.set_data(data);
                    }
                    Err(err) => {
                        props.store.clear();
                        self.load_error = Some(err.to_string());
                    }
                }
                self.abort_load_guard = None;
                true
            }
            Msg::DataChange => {
                let value = self.value.as_str().unwrap_or("").to_owned();

                if self.load_error.is_none() && value.is_empty() {
                    let mut default = props.default.clone();

                    if default.is_none() && props.autoselect {
                        if let Some((_pos, node)) = props.store.filtered_data().next() {
                            default = Some(AttrValue::from(node.key().to_string()));
                        }
                    }

                    if let Some(default) = default {
                        ctx.link().update_value(default.to_string());
                        ctx.link().update_default(default.to_string());
                    }
                }
                ctx.link().validate(); // re-evaluate
                true
            }
            Msg::Select(value) => {
                if !props.input_props.disabled {
                    ctx.link().update_value(value);
                }
                false
            }
        }
    }

    fn changed(&mut self, ctx: &ManagedFieldContext<Self>, old_props: &Self::Properties) -> bool {
        let props = ctx.props();

        let mut reload = false;

        if props.store != old_props.store {
            self._store_observer = props
                .store
                .add_listener(ctx.link().callback(|_| Msg::DataChange));
            reload = true;
        }

        if props.loader != old_props.loader {
            reload = true;
        }

        if old_props.is_disabled() && !props.is_disabled() {
            reload = true;
        }

        if props.value != old_props.value {
            ctx.link()
                .force_value(props.value.as_ref().map(|v| v.to_string()), None);
        } else if !props.editable && props.input_props.name.is_none() {
            // A controlled non-editable field cleared in place stays blank if the parent maps the
            // clear back to the same value, so re-assert. Limited to nameless fields, matching
            // the `value` prop's documented controlled use. Relies on changed() running, which
            // Yew only does when some other prop differs between renders.
            if let Some(value) = &props.value {
                if self.value.as_str().unwrap_or("").is_empty() && !value.is_empty() {
                    ctx.link().force_value(Some(value.to_string()), None);
                }
            }
        }

        if reload {
            self.load(ctx);
        }

        true
    }

    fn view(&self, ctx: &ManagedFieldContext<Self>) -> Html {
        let props = ctx.props();

        let value = self.value.as_str().unwrap_or("").to_owned();
        let validation_result = &self.result;

        let picker = {
            let picker = props.picker.clone();
            let store = props.store.clone();
            let selection = self.selection.clone();

            let load_error = self.load_error.clone();
            let is_loading = self.abort_load_guard.is_some();

            move |controller: &DropdownController| {
                if is_loading {
                    return Mask::new(Container::new().min_height(100))
                        .visible(true)
                        .into();
                }
                if let Some(load_error) = &load_error {
                    // make extra container so we can limit the inner message to the
                    // width of the input field, but no less that 400 px
                    return Container::new()
                        .class(css::Flex::Fill)
                        .class(css::Display::Flex)
                        .with_child(
                            error_message(&format!("Error: {}", load_error))
                                .padding(2)
                                .class(css::Flex::Fill)
                                .width(400),
                        )
                        .into();
                }

                if store.is_empty() {
                    return Container::new()
                        .padding(2)
                        .with_child(tr!("List does not contain any items."))
                        .into();
                }

                let render_picker_args = SelectorRenderArgs {
                    store: store.clone(),
                    selection: selection.clone(),
                    controller: controller.clone(),
                };
                picker.apply(&render_picker_args)
            }
        };

        let tip = match validation_result {
            Err(msg) => Some(msg.to_string()),
            Ok(_) => None,
        };

        let onkeydown = Callback::from({
            let link = ctx.link().clone();
            move |event: KeyboardEvent| match event.key().as_str() {
                "Delete" | "Backspace" => link.send_message(Msg::DeleteKey),
                _ => {}
            }
        });

        let mut trigger = props.trigger.clone();
        if !value.is_empty()
            && !props.editable
            && !props.is_disabled()
            && !props.input_props.required
        {
            trigger.push((
                Trigger::new("fa fa-times")
                    .tip(tr!("Clear Value"))
                    .on_activate(ctx.link().callback(|_| Msg::DeleteKey)),
                true,
            ));
        }

        Dropdown::new(picker)
            .with_std_props(&props.std_props)
            .with_input_props(&props.input_props)
            .editable(props.editable)
            .valid(validation_result.is_ok())
            .onkeydown(onkeydown)
            .on_change(ctx.link().callback(|key: String| Msg::Select(key)))
            .value(value)
            .render_value(props.render_value.clone())
            .tip(tip)
            .trigger(trigger)
            .into()
    }
}

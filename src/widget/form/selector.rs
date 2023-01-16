use anyhow::{bail, Error};
use serde_json::Value;
use derivative::Derivative;

use yew::prelude::*;
use yew::virtual_dom::Key;
use yew::html::{IntoEventCallback, IntoPropValue};

#[cfg(feature="proxmox-schema")]
use proxmox_schema::Schema;

use crate::prelude::*;
use crate::props::{RenderFn, IntoLoadCallback, LoadCallback};
use crate::state::{DataStore, Selection};
use crate::widget::{error_message, Dropdown};
use super::{FieldState, FieldStateMsg, IntoValidateFn, ValidateFn};

use pwt_macros::widget;

/// Parameters passed to the [Selector::picker] callback.
pub struct SelectorRenderArgs<S: DataStore> {
    /// The [DataStore] used by the [Selector].
    pub store: S,
    /// The selection.
    pub selection: Selection,
    /// This callback must be called by the picker to select
    /// something.
    pub on_select: Callback<Key>,
}

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
#[widget(pwt=crate, comp=PwtSelector<S>, @input, @element)]
#[derive(Derivative, Properties)]
#[derivative(Clone(bound=""), PartialEq(bound=""))]
pub struct Selector<S: DataStore + 'static> {
    store: S,
    /// The default value.
    pub default: Option<AttrValue>,

    /// Make the input editable.
    #[prop_or_default]
    pub editable: bool,
    /// Autoselect flag.
    ///
    /// If there is no default, automatically select the first loaded
    /// data item.
    #[prop_or_default]
    pub autoselect: bool,
    /// Change callback
    pub on_change: Option<Callback<Key>>,
    /// Picker render function
    pub picker: RenderFn<SelectorRenderArgs<S>>,
    /// Validate callback.
    pub validate: Option<ValidateFn<(String, S)>>,
    /// Data loader callback.
    pub loader: Option<LoadCallback<S::Collection>>,
}

impl<S: DataStore> Selector<S> {

    /// Creates a new instance
    pub fn new(
        store: S,
        picker: impl Into<RenderFn<SelectorRenderArgs<S>>>,
    ) -> Self {
        yew::props!(Self {
            store,
            picker: picker.into(),
        })
    }

    /// Builder style method to set the default item.
    pub fn default(mut self, default: impl IntoPropValue<Option<AttrValue>>) -> Self {
        self.set_default(default);
        self
    }

    /// Method to set the default item.
    pub fn set_default(&mut self, default: impl IntoPropValue<Option<AttrValue>>) {
        self.default = default.into_prop_value();
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

    /// Builder style method to set the autoselect flag
    pub fn autoselect(mut self, autoselect: bool) -> Self {
        self.set_autoselect(autoselect);
        self
    }

    /// Method to set the autoselect flag
    pub fn set_autoselect(&mut self, autoselect: bool) {
        self.autoselect = autoselect;
    }

    /// Builder style method to set the validate callback
    pub fn validate(
        mut self,
        validate: impl IntoValidateFn<(String, S)>,
    ) -> Self {
        self.set_validate(validate);
        self
    }

    /// Method to set the validate callback
    pub fn set_validate(
        &mut self,
        validate: impl IntoValidateFn<(String, S)>,
    ) {
        self.validate = validate.into_validate_fn();
    }

    /// Builder style method to set the validation schema
    #[cfg(feature="proxmox-schema")]
    pub fn schema(mut self, schema: &'static Schema) -> Self {
        self.set_schema(schema);
        self
    }

    /// Method to set the validation schema
    #[cfg(feature="proxmox-schema")]
    pub fn set_schema(&mut self, schema: &'static Schema) {
        self.validate = Some(ValidateFn::new(move |(value, _list): &(String, _)| {
            schema.parse_simple_value(&value)?;
            Ok(())
        }));
    }

    /// Builder style method to set the on_change callback
    pub fn on_change(mut self, cb: impl IntoEventCallback<Key>) -> Self {
        self.on_change = cb.into_event_callback();
        self
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
}


pub enum Msg<S: DataStore> {
    StateUpdate(FieldStateMsg),
    Select(String),
    DataChange,
    LoadResult(Result<S::Collection, Error>),
}

#[doc(hidden)]
pub struct PwtSelector<S: DataStore> {
    state: FieldState,
    selection: Selection,
    load_error: Option<String>,
    _store_observer: S::Observer,
}

fn create_selector_validation_cb<S: DataStore + 'static>(
    props: &Selector<S>,
) -> ValidateFn<Value> {
    let store = props.store.clone();
    let required = props.input_props.required;
    let validate = props.validate.clone();
    ValidateFn::new(move |value: &Value| {
        let value = match value {
            Value::Null => String::new(),
            Value::String(v) => v.clone(),
            _ => { // should not happen
                log::error!("PwtField: got wrong data type in validate!");
                String::new()
            }
        };

        if value.is_empty() {
            if required {
                bail!("Field may not be empty.");
            } else {
                return Ok(());
            }
        }

        if !store.is_empty() {
            match &validate {
                Some(cb) => cb.validate(&(value.into(), store.clone())),
                None => Ok(()),
            }
        } else  {
            bail!("no data loaded");
        }
    })
}

impl<S: DataStore + 'static> PwtSelector<S> {

    fn load(&self, ctx: &Context<Self>) {
        let props = ctx.props();
        let link = ctx.link().clone();
        if let Some(loader) = props.loader.clone() {
            wasm_bindgen_futures::spawn_local(async move {
                let res = loader.apply().await;
                link.send_message(Msg::LoadResult(res));
            });
        } else {
            // just trigger a data change to set the default value.
            link.send_message(Msg::DataChange);
        }
    }
}

impl<S: DataStore + 'static> Component for PwtSelector<S> {
    type Message = Msg<S>;
    type Properties = Selector<S>;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();

        let real_validate = create_selector_validation_cb(props);

        let on_change = match &props.on_change {
            Some(on_change) => Some(Callback::from({
                let on_change = on_change.clone();
                move |value: Value| {
                    on_change.emit(Key::from(value.as_str().unwrap_or("")));
                }
            })),
            None => None,
        };

        let state = FieldState::create(
            ctx,
            &props.input_props,
            ctx.link().callback(Msg::StateUpdate),
            on_change,
            real_validate.clone(),
        );

        let default = props.default.as_deref().unwrap_or("").to_string();
        let selection = Selection::new();
        if !default.is_empty() {
            selection.select(default.clone());
        }

        let _store_observer = props.store.add_listener(ctx.link().callback(|_| {
            Msg::DataChange
        }));

        let mut me = Self {
            state,
            selection,
            load_error: None,
            _store_observer,
        };

        if props.input_props.name.is_some() {
            me.state.register_field(&props.input_props, default.clone(), default, false);
        } else {
            me.state.force_value(default, None);
        }

        me.load(ctx);

        me
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::StateUpdate(state_msg) => {
                let default = props.default.as_deref().unwrap_or("").to_string();
                let changes = self.state.update_hook(&props.input_props, state_msg, default, false);
                if changes {
                    let (value, _valid) = self.state.get_field_data();
                    self.selection.select(Key::from(value.as_str().unwrap_or("")));
                }
                changes
            }
            Msg::LoadResult(res) => {
                match res {
                    Ok(data) => {
                        self.load_error = None;
                        props.store.set_data(data);
                        self.state.validate();
                    }
                    Err(err) => {
                        props.store.clear();
                        let default = props.default.as_deref().unwrap_or("").to_string();
                        self.state.set_value(default);
                        self.load_error = Some(err.to_string());
                    }
                }
                true
            }
            Msg::DataChange => {
                let (value, _valid) = self.state.get_field_data();
                let value = value.as_str().unwrap_or("");
                if self.load_error.is_none() {
                    if value.is_empty() {

                        let mut default = props.default.clone();

                        if default.is_none() && props.autoselect {

                            if let Some((_pos, node)) = props.store.filtered_data().next() {
                                default = Some(AttrValue::from(node.key().to_string()));
                            }
                        }

                        if let Some(default) = default {
                            self.state.set_value(default.to_string().clone());
                            self.state.set_default(default.to_string().clone());
                            return true; // set_value already validates
                        }
                    }
                }
                true
            }
            Msg::Select(value) => {
                if props.input_props.disabled { return true; }
                self.state.set_value(value);
                true
            }
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, old_props: &Self::Properties) -> bool {
        let props = ctx.props();

        if props.input_props.name.is_some() {
            self.state.update_field_options(&props.input_props);
        }

        let mut reload = false;

        if props.store != old_props.store {
            self._store_observer = props.store.add_listener(ctx.link().callback(|_| {
                Msg::DataChange
            }));
            reload = true;
        }

        if props.loader != old_props.loader {
            reload = true;
        }

        if reload { self.load(ctx); }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let (value, valid) = self.state.get_field_data();
        let value = value.as_str().unwrap_or("").to_owned();

        let picker = {
            let picker = props.picker.clone();
            let store = props.store.clone();
            let selection = self.selection.clone();

            let load_error = self.load_error.clone();

            move |on_select: &Callback<Key>| {

                if let Some(load_error) = &load_error {
                    return error_message(&format!("Error: {}", load_error), "pwt-p-2");
                }

                if store.is_empty() {
                    return html!{
                        <div class="pwt-p-2">{"List does not contain any items."}</div>
                    };
                }

                let render_picker_args = SelectorRenderArgs {
                    store: store.clone(),
                    selection: selection.clone(),
                    on_select: on_select.clone(), // fixme
                };
                picker.apply(&render_picker_args)
            }
        };

        let tip = match &valid {
            Err(msg) => Some(html!{msg}),
            Ok(_) => None,
        };

        Dropdown::new(picker)
            .with_std_props(&props.std_props)
            .with_input_props(&props.input_props)
            .editable(props.editable)
            .class(if valid.is_ok() { "is-valid" } else { "is-invalid" })
            .on_change(ctx.link().callback(|key: String| Msg::Select(key)))
            .value(value)
            .tip(tip)
            .into()
    }

}

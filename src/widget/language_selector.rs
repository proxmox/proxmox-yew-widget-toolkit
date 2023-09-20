use std::rc::Rc;

use crate::state::{Selection, Store};
use yew::html::IntoEventCallback;
use yew::virtual_dom::{Key, VComp, VNode};

use crate::prelude::*;
use crate::state::{get_available_languages, Language, LanguageInfo};
use crate::widget::data_table::{DataTable, DataTableColumn, DataTableHeader};
use crate::widget::{Dropdown, GridPicker};

use pwt_macros::builder;


/// Language Selector
///
/// Combobox like selector to choose from a list of available languages.
///
/// Please initialize the list of available languages on application startup:
///
/// ```
/// use pwt::prelude::*;
/// use pwt::state::{set_available_languages, LanguageInfo};
/// # fn init() {
/// set_available_languages(vec![
///     LanguageInfo::new("de", "Deutsch", gettext_noop("German")),
///     LanguageInfo::new("en", "English", gettext_noop("English")),
/// ]);
/// # }
/// ```
///
/// If you do not specilfy an `on_change` callback, the selected language is directly
/// stored using the global [Language] state, so that the [CatalogLoader] automatically
/// loads the new catalog and redraw the whole page (you loose the page state).
#[derive(Clone, PartialEq, Properties)]
#[builder]
pub struct LanguageSelector {
    /// On change callback.
    #[builder_cb(IntoEventCallback, into_event_callback, String)]
    on_change: Option<Callback<String>>,
}

impl LanguageSelector {
    pub fn new() -> Self {
        yew::props!(Self {})
    }
}

#[doc(hidden)]
pub struct ProxmoxLanguageSelector {
    store: Store<LanguageInfo>,
    selection: Selection,
    lang: String,
}

pub enum Msg {
    Select(String),
}

impl Component for ProxmoxLanguageSelector {
    type Message = Msg;
    type Properties = LanguageSelector;

    fn create(_ctx: &Context<Self>) -> Self {
        let store = Store::new();
        let languages = get_available_languages();

        let mut lang = Language::load();
        if lang.is_empty() {
            if languages.iter().find(|info| info.lang == "en").is_some() {
                lang = "en".into();
            } else if let Some(first) = languages.first().map(|info| info.lang.clone()) {
                lang = first;
            }
        }

        store.set_data(languages);
        let selection = Selection::new();

        selection.select(Key::from(lang.clone()));

        Self { store, selection, lang }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::Select(lang) => {
                self.lang = lang.clone();
                if let Some(on_change) = &props.on_change {
                    on_change.emit(lang);
                } else {
                    Language::store(lang);
                }
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {

        let picker = {
            let store = self.store.clone();
            let columns = COLUMNS.with(Rc::clone);
            let selection = self.selection.clone();

            move |on_select: &Callback<Key>| {
                let table = DataTable::new(columns.clone(), store.clone());

                GridPicker::new(table)
                    .selection(selection.clone())
                    .show_filter(false)
                    .on_select(on_select.clone())
                    .into()
            }
        };

        let store = self.store.clone();

        Dropdown::new(picker)
            .value(self.lang.clone())
            .on_change(ctx.link().callback(Msg::Select))
            .render_value(move |id: &AttrValue| {
                let key = Key::from(id.to_string());
                if let Some(info) = store.read().lookup_record(&key) {
                    html! {&info.text}
                } else {
                    html! {id}
                }
            })
            .into()
    }
}

impl Into<VNode> for LanguageSelector {
    fn into(self) -> VNode {
        let comp = VComp::new::<ProxmoxLanguageSelector>(Rc::new(self), None);
        VNode::from(comp)
    }
}

thread_local! {
    static COLUMNS: Rc<Vec<DataTableHeader<LanguageInfo>>> = Rc::new(vec![
        DataTableColumn::new(tr!("Language"))
            .width("200px")
            .show_menu(false)
            .render(|info: &LanguageInfo| {
                html!{&info.text}
            })
            .sorter(|a: &LanguageInfo, b: &LanguageInfo| {
                a.text.cmp(&b.text)
            })
            .sort_order(true)
            .into(),
        DataTableColumn::new(tr!("Translated"))
            .width("200px")
            .show_menu(false)
            .render(|info: &LanguageInfo| {
                html!{&info.translated_text}
            })
            .sorter(|a: &LanguageInfo, b: &LanguageInfo| {
                a.translated_text.cmp(&b.translated_text)
            })
           .into(),
    ]);
}

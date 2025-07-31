use std::rc::Rc;

use gettext::Catalog;
use yew::html::IntoPropValue;
use yew::virtual_dom::{VComp, VNode};

use crate::props::{IntoOptionalTextRenderFn, TextRenderFn};
use crate::state::{Language, LanguageObserver};
use crate::{impl_to_html, prelude::*};

use pwt_macros::builder;

/// Catalog loader (load I18N translation catalogs)
///
/// This component uses the the language from the [Language] state, and
/// automatically reloads the catalog on changes.
///
/// It is also possible to directly specify the language using the 'lang' property.
#[derive(Properties, Clone, PartialEq)]
#[builder]
pub struct CatalogLoader {
    body: VNode,

    /// Language (ISO 639-1 language code).
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub lang: Option<AttrValue>,

    /// Convert ISO 639-1 language code to server side URL
    ///
    /// Default callback is:
    /// ```
    /// # use yew::Callback;
    /// # fn test () -> Callback<String, String> {
    ///  Callback::from(|lang: String| format!("catalog-{}.mo", lang))
    /// # }
    #[builder_cb(IntoOptionalTextRenderFn, into_optional_text_render_fn, String)]
    #[prop_or_default]
    pub url_builder: Option<TextRenderFn<String>>,

    /// Default language (skip catalog loading for this language)
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or(AttrValue::Static("en"))]
    pub default_lang: AttrValue,
}

impl CatalogLoader {
    pub fn new(body: impl Into<VNode>) -> Self {
        yew::props!(Self { body: body.into() })
    }

    fn lang_to_url(&self, lang: impl Into<String>) -> String {
        let lang = lang.into();
        if let Some(url_builder) = &self.url_builder {
            url_builder.apply(&lang)
        } else {
            format!("catalog-{}.mo", lang)
        }
    }
}

pub enum Msg {
    ChangeLanguage(String),
    LoadFinished(String),
    LoadDone,
}

#[derive(Clone, PartialEq)]
enum LoadState {
    Idle,
    Loading,
    LoadFinished(String),
}

#[doc(hidden)]
pub struct PwtCatalogLoader {
    _observer: LanguageObserver,
    lang: String,
    last_url: String,
    state: LoadState,
}

impl Component for PwtCatalogLoader {
    type Message = Msg;
    type Properties = CatalogLoader;

    fn create(ctx: &yew::Context<Self>) -> Self {
        let props = ctx.props();

        let _observer = LanguageObserver::new(ctx.link().callback(Msg::ChangeLanguage));

        let lang = props
            .lang
            .clone()
            .map(|l| l.to_string())
            .unwrap_or(Language::load());

        Self {
            lang,
            last_url: String::new(),
            state: LoadState::Idle,
            _observer,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::LoadDone => {
                self.state = LoadState::Idle;
                true
            }
            Msg::LoadFinished(url) => {
                self.state = LoadState::LoadFinished(url);
                true
            }
            Msg::ChangeLanguage(lang) => {
                self.lang = props.lang.clone().map(|l| l.to_string()).unwrap_or(lang);
                true
            }
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, old_props: &Self::Properties) -> bool {
        let props = ctx.props();
        if props.lang != old_props.lang {
            self.lang = props
                .lang
                .clone()
                .map(|l| l.to_string())
                .unwrap_or(Language::load());
        }
        true
    }

    fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
        let props = ctx.props();

        if matches!(self.state, LoadState::LoadFinished(_)) {
            html! {}
        } else {
            html! {props.body.clone()}
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, _first_render: bool) {
        let props = ctx.props();
        match &self.state {
            LoadState::Idle => {
                if !self.lang.is_empty() {
                    let url = props.lang_to_url(&self.lang);
                    if self.last_url != url {
                        self.state = LoadState::Loading;
                        let link = ctx.link().clone();

                        if self.lang == props.default_lang {
                            crate::init_i18n(Catalog::empty());
                            link.send_message(Msg::LoadFinished(url));
                        } else {
                            crate::init_i18n_from_url(&url, move |url| {
                                link.send_message(Msg::LoadFinished(url));
                            });
                        }
                    }
                } else {
                    crate::init_i18n(Catalog::empty());
                }
            }
            LoadState::Loading => { /* wait until loaded */ }
            LoadState::LoadFinished(loaded_url) => {
                self.last_url = loaded_url.to_owned();
                ctx.link().send_message(Msg::LoadDone);
            }
        }
    }
}

impl From<CatalogLoader> for VNode {
    fn from(val: CatalogLoader) -> Self {
        let comp = VComp::new::<PwtCatalogLoader>(Rc::new(val), None);
        VNode::from(comp)
    }
}

impl_to_html!(CatalogLoader);

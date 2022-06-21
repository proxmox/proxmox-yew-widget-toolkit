use std::rc::Rc;

use web_sys::{Element, HtmlElement};

use yew::prelude::*;
use yew::virtual_dom::{Key, VComp, VNode};

#[derive(Properties, Clone, PartialEq)]
pub struct Modal {
    #[prop_or_default]
    node_ref: NodeRef,
    pub key: Option<Key>,

    pub onclose: Option<Callback<()>>,

    #[prop_or_default]
    pub children: Vec<VNode>,
}

impl Modal {

    pub fn new() -> Self {
        yew::props!(Self {})
    }

    /// Builder style method to set the yew `node_ref`
    pub fn node_ref(mut self, node_ref: ::yew::html::NodeRef) -> Self {
        self.node_ref = node_ref;
        self
    }

    /// Builder style method to set the yew `key` property
    pub fn key(mut self, key: impl Into<Key>) -> Self {
        self.key = Some(key.into());
        self
    }

    pub fn with_child(mut self, child: impl Into<Html>) -> Self {
        self.children.push(child.into());
        self
    }

    pub fn onclose(mut self, cb: impl Into<Option<Callback<()>>>) -> Self {
        self.onclose = cb.into();
        self
    }

    pub fn html(self) -> VNode {
        self.into()
    }
}

pub struct PwtModal {
    inner_host: Option<Element>,
}

impl Component for PwtModal {
    type Message = ();
    type Properties = Modal;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            inner_host: None,
        }
    }

    fn update(&mut self, _: &Context<Self>, _: Self::Message) -> bool {
        true
    }
    
    fn view(&self, ctx: &Context<Self>) -> Html {
        let contents = match &self.inner_host {
            Some(inner_host) => create_portal(
                html!{
                    <div style="height:100vh;width:100vw;position:absolute;overflow:hidden;">
                        <div style="position:absolute;top:50%;left:50%;transform: translate(-50%,-50%);">
                          {ctx.props().children.clone()}
                        </div>
                    </div>
                },
                inner_host.clone(),
            ),
            None => html! { <></> }
        };

        contents
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            let window = web_sys::window().unwrap();
            let document = window.document().unwrap();
            
            let modal_host = lookup_modal_host(&document);
 
            let portal = document
                .create_element("div")
                .expect("can create modal portal");
            
            modal_host.append_child(&portal)
                .expect("can attach modal portal");
            
            self.inner_host = Some(portal);
            ctx.link().send_message(()); // redraw
        }
    }
}

fn lookup_modal_host(document: &web_sys::Document) -> Element {
    let body: HtmlElement = document.body().unwrap();
            
    log::info!("BODY {:?}", body);

    match body.children().named_item("__pwt_modal_host") {
        Some(modal_host) => modal_host,
        None => {
            let modal_host = document
                .create_element("div")
                .expect("can create modal host wrapper");
        
            modal_host.set_id("__pwt_modal_host");
            modal_host.set_attribute("style", "position:absolute;top:0;left:0;").unwrap();
                
            body
                .append_child(&modal_host)
                .expect("can attach modal host");
            modal_host
        }
    }
}

impl Into<VNode> for Modal {
    fn into(self) -> VNode {
        let node_ref = self.node_ref.clone();
        let key = self.key.clone();
        let comp = VComp::new::<PwtModal>(Rc::new(self), node_ref, key);
        VNode::from(comp)
    }
}

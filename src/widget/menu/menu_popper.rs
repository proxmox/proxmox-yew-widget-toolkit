use wasm_bindgen::JsValue;
use serde_json::json;

use yew::prelude::*;

pub struct MenuPopper {
    content_ref: NodeRef,
    submenu_ref: NodeRef,
    popper: Option<JsValue>,
    prefer_bottom: bool,
}

impl MenuPopper {

    pub fn new(content_ref: NodeRef, submenu_ref: NodeRef, prefer_bottom: bool) -> Self {
        Self {
            popper: None,
            content_ref,
            submenu_ref,
            prefer_bottom,
        }
    }

    pub fn update(&mut self) {
        if self.popper.is_none() {
            let opts = json!({
                "placement": if self.prefer_bottom { "bottom-start" } else { "right-start"},
                "strategy": "fixed",
                "modifiers": [
                    {
                        "name": "preventOverflow",
                        "options": {
                            "mainAxis": true, // true by default
                            "altAxis": false, // false by default
                        },
                    },
                    {
                        "name": "flip",
                        "options": {
                            "fallbackPlacements": [if self.prefer_bottom { "right-start" } else { "bottom-start" }],
                        },
                    },
                ],
            });

            let opts = crate::to_js_value(&opts).unwrap();

            if let Some(content_node) = self.content_ref.get() {
                if let Some(submenu_node) = self.submenu_ref.get() {
                    self.popper = Some(crate::create_popper(content_node, submenu_node, &opts));
                }
            }
        }

        if let Some(popper) = &self.popper {
            crate::update_popper(popper);
        }
    }
}

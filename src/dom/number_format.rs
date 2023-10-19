use serde::Deserialize;
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq)]
pub struct LocaleInfo {
    decimal: String,
    group: String,
}

impl LocaleInfo {
    pub fn new() -> Self {
        let nf = js_sys::Intl::NumberFormat::new(&js_sys::Array::new(), &js_sys::Object::new());

        let info = nf.format_to_parts(11111.22);

        let parts: Vec<NumberPartInfo> = serde_wasm_bindgen::from_value(info.into()).unwrap();

        let decimal = parts.iter().find(|i| i.ty == "decimal").map(|i| i.value.clone());
        let group = parts.iter().find(|i| i.ty == "group").map(|i| i.value.clone());

        if let (Some(decimal), Some(group)) = (decimal, group) {
            Self { decimal, group }
        } else {
            log::error!("LocaleInfo: unable to detect locale info - using defaults.");
            Self { decimal: ".".into(), group: ",". into() }
        }
    }

    pub fn format_float(&self, value: f64) -> String {
        let mut text = value.to_string();
        if self.decimal != "." {
            text = text.replace(".", &self.decimal);
        }
        log::info!("format_float {} -> {}", value, text);
        text
    }

    pub fn parse_float(&self, text: &str) -> f64 {
        let text = text.replace(&self.decimal, "{D}");
        let text = text.replace(&self.group, "{G}");

        if text.contains(['.', ',']) {
            //log::info!("parse_float1 {}", text);
            return f64::NAN;
        }

        // f64::from_str will fail if it finds a group separator!
        // This is good, because group separators just add more confusion...
        let text = text.replace("{G}", ",");
        // f64::from_str uses '.' as decimal separator
        let text = text.replace("{D}", ".");

        let number = f64::from_str(&text).unwrap_or(f64::NAN);

        // log::info!("parse_float2 {} -> {}", text, number);

        number
    }
}


// result from js_sys::Intl::NumberFormat::format_to_parts
#[derive(Deserialize, Debug)]
struct NumberPartInfo {
    #[serde(rename = "type")]
    ty: String,
    value: String,
}

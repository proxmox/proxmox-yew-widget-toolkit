use std::str::FromStr;

use serde::Deserialize;

use crate::tr;

/// Locale related setting like deciaml and thousands separator.
#[derive(Clone, Debug, PartialEq)]
pub struct LocaleInfo {
    /// Decimal separator.
    pub decimal: String,
    /// Digit group separator.
    pub group: String,
}

fn test_browser_locale() -> Option<LocaleInfo> {
    let nf = js_sys::Intl::NumberFormat::new(&js_sys::Array::new(), &js_sys::Object::new());

    let info = nf.format_to_parts(11111.22);

    let parts: Vec<NumberPartInfo> = serde_wasm_bindgen::from_value(info.into()).unwrap();

    let decimal = parts.iter().find(|i| i.ty == "decimal").map(|i| i.value.clone());
    let group = parts.iter().find(|i| i.ty == "group").map(|i| i.value.clone());

    if let (Some(decimal), Some(group)) = (decimal, group) {
        Some(LocaleInfo { decimal, group })
    } else {
        None
    }
}

thread_local!{
    static BROWSER_LOCALE: LocaleInfo = {
        if let Some(info) = test_browser_locale() {
            info
        } else {
            log::error!("LocaleInfo: unable to detect locale info - using defaults.");
            LocaleInfo::default()
        }
    }
}

fn get_browser_locale_info() -> LocaleInfo {
    BROWSER_LOCALE.with(|info| info.clone())
}

/// Returns a string with a language-sensitive representation of this number.
///
/// Uses current browser locale setting.
pub fn format_float(value: f64) -> String {
    BROWSER_LOCALE.with(|info| info.format_float(value))
}

/// Parse a language-sensitive representation of f64.
///
/// Uses current browser locale setting.
pub fn parse_float(text: &str) -> Result<f64, String> {
    BROWSER_LOCALE.with(|info| info.parse_float(text))
}

impl Default for LocaleInfo {
    fn default() -> Self {
        Self { decimal: ".".into(), group: ",". into() }
    }
}

impl LocaleInfo {
    /// Return current browser locale settings.
    pub fn new() -> Self { get_browser_locale_info() }

    /// Rust f64 float format, but replaces decimal point from browser locale settings.
    pub fn format_float(&self, value: f64) -> String {
        let mut text = value.to_string();
        if self.decimal != "." {
            text = text.replace(".", &self.decimal);
        }
        // log::info!("format_float {} -> {}", value, text);
        text
    }

    /// Parse Rust f64 float format, but uses decimal point from browser locale settings.
    ///
    /// Note: Values 'inf' | 'infinity' | 'nan' are not allowed.
    pub fn parse_float(&self, text: &str) -> Result<f64, String> {
        let text = text.replace(&self.decimal, "{D}");
        let text = text.replace(&self.group, "{G}");

        if text.contains(['.', ',']) {
            return Err(tr!("invalid float literal (wrong decimal separator)"));
        }

        // f64::from_str will fail if it finds a group separator!
        // This is good, because group separators just add more confusion...
        let text = text.replace("{G}", ",");
        // f64::from_str uses '.' as decimal separator
        let text = text.replace("{D}", ".");

        let number = match f64::from_str(&text) {
            Ok(number) => number,
            Err(_) => return Err(tr!("invalid float literal")),
        };

        if !number.is_finite() { // do not allow "inf", "nan", ...
            return Err(tr!("invalid float literal"));
        }

        Ok(number)
    }
}


// result from js_sys::Intl::NumberFormat::format_to_parts
#[derive(Deserialize, Debug)]
struct NumberPartInfo {
    #[serde(rename = "type")]
    ty: String,
    value: String,
}

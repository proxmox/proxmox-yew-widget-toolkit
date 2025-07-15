use indexmap::IndexMap;
use yew::{html::IntoPropValue, AttrValue};

/// Holds the CSS styles to set on elements
#[derive(Clone, Default, Debug, PartialEq)]
pub struct CssStyles {
    styles: IndexMap<AttrValue, AttrValue>,
}

impl CssStyles {
    /// Method to set style attributes
    ///
    /// Note: Value 'None' removes the attribute.
    /// Note: In debug mode, panics on invalid characters (';' and ':')
    pub fn set_style(
        &mut self,
        key: impl Into<AttrValue>,
        value: impl IntoPropValue<Option<AttrValue>>,
    ) {
        let key = key.into();
        #[cfg(debug_assertions)]
        if key.contains([';', ':']) {
            panic!("invalid character in style key: '{key}'");
        }
        if let Some(value) = value.into_prop_value() {
            #[cfg(debug_assertions)]
            if value.contains([';', ':']) {
                panic!("invalid character in style value '{value}' for '{key}'");
            }
            self.styles
                .insert(AttrValue::from(key), AttrValue::from(value));
        } else {
            self.styles.swap_remove(&AttrValue::from(key));
        }
    }

    /// Method to compile the finished style attribute to use
    ///
    /// Optionally takes an additional [yew::AttrValue] to append
    pub fn compile_style_attribute(&self, additional_style: Option<AttrValue>) -> AttrValue {
        let mut style = String::new();

        for (key, value) in self.styles.iter() {
            style += &format!("{key}: {value};");
        }

        if let Some(additional_style) = additional_style {
            style += &additional_style;
        }

        AttrValue::from(style)
    }

    /// Return the number of key-value pairs in the map.
    pub fn len(&self) -> usize {
        self.styles.len()
    }

    /// Returns true if the map contains no elements.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

// not completely spec compliant, since we ignore 'at-rules', but there are
// no valid ones currently, so this should not be an issue
// https://w3c.github.io/csswg-drafts/css-style-attr/
impl<T: AsRef<str>> From<T> for CssStyles {
    fn from(value: T) -> Self {
        let mut this: CssStyles = Default::default();
        for rule in value.as_ref().split(';') {
            if let Some((key, val)) = rule.split_once(':') {
                this.set_style(key.to_owned(), val.to_owned());
            }
        }
        this
    }
}

/// Trait which provides mutable access to the style property.
pub trait AsCssStylesMut {
    fn as_css_styles_mut(&mut self) -> &mut CssStyles;
}

impl AsCssStylesMut for CssStyles {
    fn as_css_styles_mut(&mut self) -> &mut CssStyles {
        self
    }
}

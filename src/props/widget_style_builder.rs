use std::fmt::Display;

use yew::{html::IntoPropValue, AttrValue};

use crate::props::{AsCssStylesMut, CssStyles};

/// CSS length in pixel, em or percentage.
#[derive(Copy, Clone, PartialEq)]
pub enum CssLength {
    Px(f64),
    Em(f64),
    Fraction(f32),
    None,
}

impl Default for CssLength {
    fn default() -> Self {
        CssLength::Px(0.0)
    }
}

impl Display for CssLength {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            CssLength::Px(v) => write!(f, "{v}px"),
            CssLength::Em(v) => write!(f, "{v}em"),
            CssLength::Fraction(v) => write!(f, "{}%", v * 100.0),
            CssLength::None => Ok(()),
        }
    }
}

impl From<f32> for CssLength {
    fn from(v: f32) -> CssLength {
        CssLength::Px(v as f64)
    }
}

impl From<f64> for CssLength {
    fn from(v: f64) -> CssLength {
        CssLength::Px(v)
    }
}

impl From<usize> for CssLength {
    fn from(v: usize) -> CssLength {
        CssLength::Px(v as f64)
    }
}

impl From<i32> for CssLength {
    fn from(v: i32) -> CssLength {
        CssLength::Px(v as f64)
    }
}

impl From<CssLength> for AttrValue {
    fn from(v: CssLength) -> Self {
        v.to_string().into()
    }
}

impl IntoPropValue<Option<AttrValue>> for CssLength {
    fn into_prop_value(self) -> Option<AttrValue> {
        match self {
            CssLength::None => None,
            other => Some(other.into()),
        }
    }
}

// macro to generate the trait functions

macro_rules! generate_style_trait_fn {
    ($func:ident, $builder:ident, $name:literal) => {
        /// Builder style method to set the $name of the element style.
        ///
        /// Note: Value [CssLength::None] removes it.
        fn $builder(mut self, value: impl Into<CssLength>) -> Self {
            self.$func(value);
            self
        }

        /// Sets the $name of the element style.
        ///
        /// Note: Value [CssLength::None] removes it.
        fn $func(&mut self, value: impl Into<CssLength>) {
            self.as_css_styles_mut().set_style($name, value.into());
        }
    };
}

pub trait WidgetStyleBuilder: AsCssStylesMut + Sized {
    /// Builder style method to override all styles for the element with the given ones
    fn styles(mut self, styles: CssStyles) -> Self {
        self.set_styles(styles);
        self
    }

    /// Overrides all styles for the element with the given ones
    fn set_styles(&mut self, styles: CssStyles) {
        *self.as_css_styles_mut() = styles;
    }

    /// Builder style method to set additional css styles via the 'style' attribute
    ///
    /// Note: Value 'None' removes the style.
    fn style(
        mut self,
        key: impl Into<AttrValue>,
        value: impl IntoPropValue<Option<AttrValue>>,
    ) -> Self {
        self.set_style(key, value);
        self
    }

    /// Method to set additional css styles via the 'style' attribute
    ///
    /// Note: Value 'None' removes the style.
    fn set_style(
        &mut self,
        key: impl Into<AttrValue>,
        value: impl IntoPropValue<Option<AttrValue>>,
    ) {
        self.as_css_styles_mut().set_style(key, value)
    }

    generate_style_trait_fn!(set_width, width, "width");
    generate_style_trait_fn!(set_min_width, min_width, "min-width");
    generate_style_trait_fn!(set_max_width, max_width, "max-width");
    generate_style_trait_fn!(set_height, height, "height");
    generate_style_trait_fn!(set_min_height, min_height, "min-height");
    generate_style_trait_fn!(set_max_height, max_height, "max-height");
}

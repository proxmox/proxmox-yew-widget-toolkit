use yew::{html::IntoPropValue, AttrValue};

/// CSS length in spacer units, pixel, em or percentage.
#[derive(Copy, Clone, PartialEq)]
pub enum PwtSpace {
    Pwt(usize),
    Px(f64),
    Em(f64),
    Fraction(f32),
    None,
}

impl std::fmt::Display for PwtSpace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            PwtSpace::Pwt(v) => write!(f, "calc({v} * var(--pwt-spacer-base-width))px"),
            PwtSpace::Px(v) => write!(f, "{v}px"),
            PwtSpace::Em(v) => write!(f, "{v}em"),
            PwtSpace::Fraction(v) => write!(f, "{}%", v * 100.0),
            PwtSpace::None => Ok(()),
        }
    }
}

impl From<usize> for PwtSpace {
    fn from(v: usize) -> PwtSpace {
        PwtSpace::Pwt(v)
    }
}

impl From<f32> for PwtSpace {
    fn from(v: f32) -> PwtSpace {
        PwtSpace::Px(v as f64)
    }
}

impl From<f64> for PwtSpace {
    fn from(v: f64) -> PwtSpace {
        PwtSpace::Px(v)
    }
}

impl From<PwtSpace> for AttrValue {
    fn from(v: PwtSpace) -> Self {
        v.to_string().into()
    }
}

impl IntoPropValue<Option<AttrValue>> for PwtSpace {
    fn into_prop_value(self) -> Option<AttrValue> {
        match self {
            PwtSpace::None => None,
            other => Some(other.into()),
        }
    }
}

// macro to generate the trait functions

macro_rules! generate_padding_trait_fn {
    ($func:ident, $builder:ident, $name:literal, $class:literal) => {
        /// Builder style method to set $name CSS property.
        fn $builder(mut self, padding: impl Into<PwtSpace>) -> Self {
            self.$func(padding);
            self
        }

        /// Sets the $name CSS property for an element.
        fn $func(&mut self, space: impl Into<PwtSpace>) {
            match space.into() {
                PwtSpace::None => {}
                PwtSpace::Pwt(factor) if factor <= 4 => {
                    self.as_classes_mut().push(format!($class, factor));
                }
                space => self.as_css_styles_mut().set_style($name, space.to_string()),
            }
        }
    };
}

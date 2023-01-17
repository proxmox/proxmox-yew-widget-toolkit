/// Rust type to specify CSS margins.
///
/// Convienent way to describe CSS margins. This struct implement
/// `From` for sevaral types, and maps that to CSS classes:
///
/// - `usize`: CSS class `pwt-m-{usize}`
/// - `(usize, usize)`: sets CSS class `pwt-my-{usize}` and `pwt-my-{usize}`
/// - `(usize, usize, usize, usize)`: sets CSS class `pwt-mt-{usize}`, `pwt-me-{usize}`, `pwt-mb-{usize}` and `pwt-ms-{usize}`
///
/// Please note that all values are optional, so can also specify `None` if
/// you dont want to set a class.
///
/// Our currect CSS template sepcifies those classes for values 0, 1, 2
/// and 3. The real size is specified inside the CSS and defaults to
/// 0.5em, 1em, 1.5em and 2em.

/// The [WidgetBuilder](super::WidgetBuilder) trait provides methods
/// to set margins for our standard widgets, .i.e:
///
///```
/// # use pwt::widget::Column;
/// # use crate::pwt::props::WidgetBuilder;
/// Column::new()
///     .margin((2, None)) // sets class `pwt-my-2`
///
/// # ;
///```


#[derive(PartialEq, Debug, Clone)]
pub enum Margin {
    Single(Option<usize>),
    Tuple(Option<usize>, Option<usize>),
    Quad(Option<usize>, Option<usize>, Option<usize>, Option<usize>),
}

impl Margin {
    // Note: Using html style tag is maybe simpler ...
    pub fn to_class(&self) -> Option<String> {
        match self {
            Margin::Single(None) => None,
            Margin::Single(Some(m)) => Some(format!("pwt-m-{}", m)),
            Margin::Tuple(None, None) => None,
            Margin::Tuple(None, Some(mx)) => Some(format!("pwt-mx-{}", mx)),
            Margin::Tuple(Some(my), None) => Some(format!("pwt-my-{}", my)),
            Margin::Tuple(Some(my), Some(mx)) =>
                Some(format!("pwt-mx-{} pwt-my-{}", mx, my)),
            Margin::Quad(mt, mr, mb, ml) => {
                let mut c = Vec::new();
                if let Some(mt) = *mt { c.push(format!("pwt-mt-{}", mt)); }
                if let Some(mr) = *mr { c.push(format!("pwt-me-{}", mr)); }
                if let Some(mb) = *mb { c.push(format!("pwt-mb-{}", mb)); }
                if let Some(ml) = *ml { c.push(format!("pwt-ms-{}", ml)); }
                if c.is_empty() {
                    None
                } else {
                    Some(c.join(" "))
                }
            }
        }
    }
}

impl Default for Margin {
    fn default() -> Self {
        Margin::Single(None)
    }
}

pub trait IntoOptionalMarginSize {
    fn into_optional_margin_size(self) -> Option<usize>;
}

impl IntoOptionalMarginSize for usize {
    fn into_optional_margin_size(self) -> Option<usize> {
        Some(self)
    }
}

impl IntoOptionalMarginSize for Option<usize> {
    fn into_optional_margin_size(self) -> Option<usize> {
        self
    }
}

impl<I: IntoOptionalMarginSize> From<I> for Margin {
    fn from(v: I) -> Self {
        Margin::Single(v.into_optional_margin_size())
    }
}

impl<I1, I2> From<(I1, I2)> for Margin
where
    I1: IntoOptionalMarginSize,
    I2: IntoOptionalMarginSize,
{
    fn from(v: (I1, I2)) -> Self {
        Margin::Tuple(
            v.0.into_optional_margin_size(),
            v.1.into_optional_margin_size(),
        )
    }
}

impl<I1, I2, I3, I4> From<(I1, I2, I3, I4)> for Margin
where
    I1: IntoOptionalMarginSize,
    I2: IntoOptionalMarginSize,
    I3: IntoOptionalMarginSize,
    I4: IntoOptionalMarginSize,
{
    fn from(v: (I1, I2, I3, I4)) -> Self {
        Margin::Quad(
            v.0.into_optional_margin_size(),
            v.1.into_optional_margin_size(),
            v.2.into_optional_margin_size(),
            v.3.into_optional_margin_size(),
        )
    }
}

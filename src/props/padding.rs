/// Rust type to specify CSS paddings.
///
/// Convienent way to describe CSS paddings. This struct implement
/// `From` for sevaral types, and maps that to CSS classes:
///
/// - `usize`: CSS class `pwt-p-{usize}`
/// - `(usize, usize)`: sets CSS class `pwt-py-{usize}` and `pwt-py-{usize}`
/// - `(usize, usize, usize, usize)`: sets CSS class `pwt-pt-{usize}`, `pwt-pe-{usize}`, `pwt-pb-{usize}` and `pwt-ps-{usize}`
///
/// Please note that all values are optional, so can also specify `None` if
/// you dont want to set a class.
///
/// Our currect CSS template sepcifies those classes for values 0, 1, 2
/// and 3. The real size is specified inside the CSS and defaults to
/// 0.5em, 1em, 1.5em and 2em.

/// The [WidgetBuilder](super::WidgetBuilder) trait provides methods
/// to set paddings for our standard widgets, .i.e:
///
///```
/// # use pwt::widget::Column;
/// # use crate::pwt::props::WidgetBuilder;
/// Column::new()
///     .padding((2, None)) // sets class `pwt-py-2`
///
/// # ;
///```

#[derive(PartialEq, Debug, Clone)]
pub enum Padding {
    Single(Option<usize>),
    Tuple(Option<usize>, Option<usize>),
    Quad(Option<usize>, Option<usize>, Option<usize>, Option<usize>),
}

impl Padding {
    // Note: Using html style tag is maybe simpler ...
    pub fn to_class(&self) -> Option<String> {
        match self {
            Padding::Single(None) => None,
            Padding::Single(Some(p)) => Some(format!("pwt-p-{}", p)),
            Padding::Tuple(None, None) => None,
            Padding::Tuple(None, Some(px)) => Some(format!("pwt-px-{}", px)),
            Padding::Tuple(Some(py), None) => Some(format!("pwt-py-{}", py)),
            Padding::Tuple(Some(py), Some(px)) =>
                Some(format!("pwt-px-{} pwt-py-{}", px, py)),
            Padding::Quad(pt, pr, pb, pl) => {
                let mut c = Vec::new();
                if let Some(pt) = *pt  { c.push(format!("pwt-pt-{}", pt)); }
                if let Some(pr) = *pr { c.push(format!("pwt-pe-{}", pr)); }
                if let Some(pb) = *pb { c.push(format!("pwt-pb-{}", pb)); }
                if let Some(pl) = *pl { c.push(format!("pwt-ps-{}", pl)); }
                if c.is_empty() {
                    None
                } else {
                    Some(c.join(" "))
                }
            }
        }
    }
}

impl Default for Padding {
    fn default() -> Self {
        Padding::Single(None)
    }
}

pub trait IntoOptionalPaddingSize {
    fn into_optional_padding_size(self) -> Option<usize>;
}

impl IntoOptionalPaddingSize for usize {
    fn into_optional_padding_size(self) -> Option<usize> {
        Some(self)
    }
}

impl IntoOptionalPaddingSize for Option<usize> {
    fn into_optional_padding_size(self) -> Option<usize> {
        self
    }
}

impl<I: IntoOptionalPaddingSize> From<I> for Padding {
    fn from(v: I) -> Self {
        Padding::Single(v.into_optional_padding_size())
    }
}

impl<I1, I2> From<(I1, I2)> for Padding
where
    I1: IntoOptionalPaddingSize,
    I2: IntoOptionalPaddingSize,
{
    fn from(v: (I1, I2)) -> Self {
        Padding::Tuple(
            v.0.into_optional_padding_size(),
            v.1.into_optional_padding_size(),
        )
    }
}

impl<I1, I2, I3, I4> From<(I1, I2, I3, I4)> for Padding
where
    I1: IntoOptionalPaddingSize,
    I2: IntoOptionalPaddingSize,
    I3: IntoOptionalPaddingSize,
    I4: IntoOptionalPaddingSize,
{
    fn from(v: (I1, I2, I3, I4)) -> Self {
        Padding::Quad(
            v.0.into_optional_padding_size(),
            v.1.into_optional_padding_size(),
            v.2.into_optional_padding_size(),
            v.3.into_optional_padding_size(),
        )
    }
}

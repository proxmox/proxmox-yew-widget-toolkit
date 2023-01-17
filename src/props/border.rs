
/// Rust type to specify CSS borders.
///
/// Convienent way to describe CSS borders. This struct implement
/// `From` for sevaral types, and maps that to CSS classes:
///
/// - `true`: CSS class `pwt-border`
/// - `(true, false)`: CSS class `pwt-border-top pwt-border-bottom`
/// - `(false, true)`: CSS class `pwt-border-left pwt-border-right`
/// - `(true, true)`: CSS class `pwt-border`
/// - `(bool, bool, bool, bool)`: sets CSS class (`pwt-border-top`, `pwt-border-right`, `pwt-border-bottom`, `pwt-border-left`) if the corresponding value is true.
///
/// The [WidgetBuilder](super::WidgetBuilder) trait provides methods
/// to set borders for our standard widgets, .i.e:
///
///```
/// # use pwt::widget::Column;
/// # use crate::pwt::props::WidgetBuilder;
/// Column::new()
///     .border((true, false)) // border on top and bottom
///
/// # ;
///```
#[derive(PartialEq, Debug, Clone)]
pub enum Border {
    Single(bool),
    Tuple(bool, bool),
    Quad(bool, bool, bool, bool),
}

impl Border {
    // Note: Using html style tag would be so much easier ...
    pub fn to_class(&self) -> Option<String> {
        match self {
            Border::Single(false) => None,
            Border::Single(true) => Some("pwt-border".into()),
            Border::Tuple(false, false) => None,
            Border::Tuple(false, true) => Some("pwt-border-left pwt-border-right".into()),
            Border::Tuple(true, false) => Some("pwt-border-top pwt-border-bottom".into()),
            Border::Tuple(true, true) => Some("pwt-border".into()),
            Border::Quad(bt, br, bb, bl) => {
                let mut c = Vec::new();
                if *bt { c.push("pwt-border-top"); }
                if *br { c.push("pwt-border-right"); }
                if *bb { c.push("pwt-border-bottom"); }
                if *bl { c.push("pwt-border-left"); }
                if c.is_empty() {
                    None
                } else {
                    Some(c.join(" "))
                }
            }
        }
    }
}

impl Default for Border {
    fn default() -> Self {
        Border::Single(false)
    }
}

impl From<bool> for Border {
    fn from(b: bool) -> Self {
        Border::Single(b)
    }
}

impl From<(bool, bool)> for Border {
    fn from(b: (bool, bool)) -> Self {
        Border::Tuple(b.0, b.1)
    }
}

impl From<(bool, bool, bool, bool)> for Border {
    fn from(b: (bool, bool, bool, bool)) -> Self {
        Border::Quad(b.0, b.1, b.2, b.3)
    }
}

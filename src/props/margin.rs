
#[derive(PartialEq, Debug, Clone)]
pub enum Margin {
    Single(usize),
    Tuple(usize, usize),
    Quad(usize, usize, usize, usize),
}

impl Margin {
    // Note: Using html style tag would be so much easier ...
    pub fn to_class(&self) -> Option<String> {
        match self {
            Margin::Single(0) => None,
            Margin::Single(m) => Some(format!("pwt-m-{}", m)),
            Margin::Tuple(0, 0) => None,
            Margin::Tuple(0, mx) => Some(format!("pwt-mx-{}", mx)),
            Margin::Tuple(my, 0) => Some(format!("pwt-my-{}", my)),
            Margin::Tuple(my, mx) =>
                Some(format!("pwt-mx-{} pwt-my-{}", mx, my)),
            Margin::Quad(mt, mr, mb, ml) => {
                let mut c = Vec::new();
                if *mt > 0 { c.push(format!("pwt-mt-{}", mt)); }
                if *mr > 0 { c.push(format!("pwt-me-{}", mr)); }
                if *mb > 0 { c.push(format!("pwt-mb-{}", mb)); }
                if *ml > 0 { c.push(format!("pwt-ms-{}", ml)); }
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
        Margin::Single(0)
    }
}

impl From<usize> for Margin {
    fn from(p: usize) -> Self {
        Margin::Single(p)
    }
}
impl From<(usize, usize)> for Margin {
    fn from(p: (usize, usize)) -> Self {
        Margin::Tuple(p.0, p.1)
    }
}
impl From<(usize, usize, usize, usize)> for Margin {
    fn from(p: (usize, usize, usize, usize)) -> Self {
        Margin::Quad(p.0, p.1, p.2, p.3)
    }
}

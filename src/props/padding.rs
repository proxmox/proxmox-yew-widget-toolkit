
#[derive(PartialEq, Debug, Clone)]
pub enum Padding {
    Single(usize),
    Tuple(usize, usize),
    Quad(usize, usize, usize, usize),
}

impl Padding {
    // Note: Using html style tag would be so much easier ...
    pub fn to_class(&self) -> Option<String> {
        match self {
            Padding::Single(0) => None,
            Padding::Single(p) => Some(format!("pwt-p-{}", p)),
            Padding::Tuple(0, 0) => None,
            Padding::Tuple(0, px) => Some(format!("pwt-px-{}", px)),
            Padding::Tuple(py, 0) => Some(format!("pwt-py-{}", py)),
            Padding::Tuple(py, px) =>
                Some(format!("pwt-px-{} pwt-py-{}", px, py)),
            Padding::Quad(pt, pr, pb, pl) => {
                let mut c = Vec::new();
                if *pt > 0 { c.push(format!("pwt-pt-{}", pt)); }
                if *pr > 0 { c.push(format!("pwt-pe-{}", pr)); }
                if *pb > 0 { c.push(format!("pwt-pb-{}", pb)); }
                if *pl > 0 { c.push(format!("pwt-ps-{}", pl)); }
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
        Padding::Single(0)
    }
}

impl From<usize> for Padding {
    fn from(p: usize) -> Self {
        Padding::Single(p)
    }
}
impl From<(usize, usize)> for Padding {
    fn from(p: (usize, usize)) -> Self {
        Padding::Tuple(p.0, p.1)
    }
}
impl From<(usize, usize, usize, usize)> for Padding {
    fn from(p: (usize, usize, usize, usize)) -> Self {
        Padding::Quad(p.0, p.1, p.2, p.3)
    }
}

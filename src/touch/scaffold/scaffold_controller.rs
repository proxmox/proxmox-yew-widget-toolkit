
use crate::state::SharedState;

#[derive(Copy, Clone)]
pub(crate) struct ScaffoldState {
    pub(crate) show_drawer: bool,
    pub(crate) show_end_drawer: bool,
}

/// Scaffold controller.
#[derive(Clone, PartialEq)]
pub struct ScaffoldController {
    pub(crate) state: SharedState<ScaffoldState>,
}

impl ScaffoldController {
    /// Crteate a new instance.
    pub fn new() -> Self {
        Self {
            state: SharedState::new(ScaffoldState {
                show_drawer: false,
                show_end_drawer: false,
             }),
        }
    }

    /// Show/hide the drawer.
    pub fn show_drawer(&self, show: bool) {
        let mut state = self.state.write();
        state.show_drawer = show;
    }

    /// Toggle drawer between show and hide.
    pub fn toggle_drawer(&self) {
        let mut state = self.state.write();
        state.show_drawer = !state.show_drawer;
    }

    /// Show/hide the end drawer.
    pub fn show_end_drawer(&self, show: bool) {
        let mut state = self.state.write();
        state.show_end_drawer = show;
    }

    /// Toggle end drawer between show and hide.
    pub fn toggle_end_drawer(&self) {
        let mut state = self.state.write();
        state.show_end_drawer = !state.show_end_drawer;
    }
}

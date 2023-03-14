//! ### Widgets for Touch devices
//!

mod gesture_detector;
pub use gesture_detector::{
    GestureDetector, GestureDragEvent, GestureSwipeEvent, PwtGestureDetector,
};

mod fab;
pub use fab::{Fab, PwtFab};

mod fab_menu;
pub use fab_menu::{FabMenu, FabMenuDirection, PwtFabMenu};

mod page_view;
pub use page_view::{PageView, PwtPageView};

mod slidable;
pub use slidable::{
    PwtSlidable, PwtSlidableAction, Slidable, SlidableAction, SlidableActionMouseEvent,
    SlidableController,
};

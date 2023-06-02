//! ### Widgets for Touch devices
//!

mod gesture_detector;
pub use gesture_detector::{
    GestureDetector, GestureDragEvent, GestureSwipeEvent, PwtGestureDetector,
};

mod fab;
pub use fab::{Fab, PwtFab};

mod fab_menu;
pub use fab_menu::{FabMenu, FabMenuAlign, FabMenuDirection, PwtFabMenu};

mod navigation_bar;
pub use navigation_bar::NavigationBar;

mod navigation_rail;
pub use navigation_rail::NavigationRail;

mod page_view;
pub use page_view::{PageView, PwtPageView};

mod page_stack;
pub use page_stack::{PageAnimationStyle, PageStack, PwtPageStack};

mod slidable;
pub use slidable::{
    PwtSlidable, PwtSlidableAction, Slidable, SlidableAction, SlidableActionMouseEvent,
    SlidableController,
};

//! ### Widgets for Touch devices
//!

mod application_bar;
pub use application_bar::{ApplicationBar, PwtApplicationBar};

mod gesture_detector;
pub use gesture_detector::{
    GestureDetector, GestureDragEvent, GestureSwipeEvent, PwtGestureDetector,
};

mod fab;
pub use fab::{Fab, PwtFab};

mod fab_menu;
pub use fab_menu::{FabMenu, FabMenuAlign, FabMenuDirection, PwtFabMenu};

mod material_app;
pub use material_app::PageController;
pub use material_app::{MaterialApp, MaterialAppRouteContext, PwtMaterialApp};

mod navigation_bar;
pub use navigation_bar::NavigationBar;

mod navigation_rail;
pub use navigation_rail::NavigationRail;

mod page_view;
pub use page_view::{PageView, PwtPageView};

mod page_stack;
pub use page_stack::{PageAnimationStyle, PageStack, PwtPageStack};

mod side_dialog;
pub use side_dialog::{PwtSideDialog, SideDialog, SideDialogController, SideDialogLocation};

mod scaffold;
pub use scaffold::{PwtScaffold, Scaffold};

mod slidable;
pub use slidable::{
    PwtSlidable, PwtSlidableAction, Slidable, SlidableAction, SlidableActionMouseEvent,
    SlidableController,
};

mod snack_bar;
pub use snack_bar::SnackBar;

mod snack_bar_manager;
pub use snack_bar_manager::{PwtSnackBarManager, SnackBarController, SnackBarManager};

mod snack_bar_context_extension;
pub use snack_bar_context_extension::SnackBarContextExt;

/// # Prelude for mobile apps, including common scope extension.
///
/// ```
/// use pwt::touch::prelude::*;
/// ```
///
/// This also re-exports the pwt standard prelude: `use pwt::prelude::*;`
///
pub mod prelude {
    #[doc(hidden)]
    pub use crate::prelude::*;

    pub use crate::touch::SnackBarContextExt;
}

//! Tools for aligning elements to each other
use js_sys::Error;
use web_sys::{window, HtmlElement};

use crate::widget::{
    dom::{element_direction_rtl, IntoHtmlElement},
    SizeObserver,
};

/// Defines a point on a rectangle
///
/// The points are defined on a rectangle with left-to-right direction as follows:
/// ```asciiart
/// TopStart ------- Top -------- TopEnd
/// Start           Center           End
/// BottomStart --- Bottom --- BottomEnd
/// ```
///
/// In case the direction is right-to-left, all `Start` and `End` points are
/// switched.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Point {
    TopStart,
    Top,
    TopEnd,
    Start,
    Center,
    End,
    BottomStart,
    BottomEnd,
    Bottom,
}

impl Point {
    fn map_rtl(&mut self, is_rtl: bool) {
        if is_rtl {
            *self = match *self {
                Point::TopStart => Point::TopEnd,
                Point::TopEnd => Point::TopStart,
                Point::Start => Point::End,
                Point::End => Point::Start,
                Point::BottomStart => Point::BottomEnd,
                Point::BottomEnd => Point::BottomStart,
                other => other,
            };
        }
    }
}

/// Defines a direction in which the element is allowed to grow and move
#[derive(Clone, Copy, Debug)]
pub enum GrowDirection {
    None,
    TopBottom,
    StartEnd,
}

#[derive(Clone)]
struct Placement {
    base: Point,
    element: Point,
    direction: GrowDirection,
}

/// Alignment options for aligning to elements to each other
///
/// It takes a [`Point`] for the element you want to align and the base element,
/// (which you want to align to) and positions it so that these to points are at the same place.
///
/// You can define a [`GrowDirection`]. This specifies the axis where the element
/// can move (and potentially scroll).
///
/// If you give more than one placement (with [`Self::add_fallback_placement`]), the element uses the
/// next placement when the first one is bigger than the viewport on the perpendicular axis to
/// the [`GrowDirection`] (if this is `None`, any overlfow triggers the fallback). When no
/// fallback fits, the original placement is used.
///
/// With [`Self::set_offset`], you can specify an offset for the element, relative to the [`Point`] on
/// the base element.
///
/// There is also a [`Self::set_viewport_padding`], where you can limit the element further into the
/// viewport.
///
/// Sometimes it is useful to set the minimum width to that of the base element, you can do that
/// with setting [`Self::set_align_width`].
///
/// There is a builder style method for each of the options.
#[derive(Clone)]
pub struct AlignOptions {
    placements: Vec<Placement>,
    offset: (f64, f64),
    viewport_padding: f64,
    align_width: bool,
}

impl Default for AlignOptions {
    fn default() -> Self {
        Self {
            placements: vec![Placement {
                base: Point::BottomStart,
                element: Point::TopStart,
                direction: GrowDirection::None,
            }],
            offset: (0.0, 0.0),
            viewport_padding: 0.0,
            align_width: false,
        }
    }
}

impl AlignOptions {
    /// Defines how the elements are aligned. The target element will be aligned such that
    /// its [`Point`] is in the same place as the [`Point`] of the base element
    pub fn new(base: Point, element: Point, direction: GrowDirection) -> Self {
        Self {
            placements: vec![Placement {
                base,
                element,
                direction,
            }],
            offset: (0.0, 0.0),
            viewport_padding: 0.0,
            align_width: false,
        }
    }

    /// Builder style An offset that will be added to the position
    pub fn offset(mut self, offset_x: f64, offset_y: f64) -> Self {
        self.offset = (offset_x, offset_y);
        self
    }

    /// Set an offset that will be added to the position relative to the base point.
    pub fn set_offset(&mut self, x: f64, y: f64) {
        self.offset = (x, y);
    }

    /// Builder style method to add a fallback placement
    pub fn with_fallback_placement(
        mut self,
        base: Point,
        target: Point,
        grow: GrowDirection,
    ) -> Self {
        self.add_fallback_placement(base, target, grow);
        self
    }

    /// Adds a fallback placement. If such a placement exists, they will be used in case the
    /// element overflows the viewport on the axis that is perpendicular to the grow direction.
    /// If all fallbacks are exhausted and none fits, the original placement will be used.
    ///
    /// In case the element overflows only in the grow direction, 'overflow: auto' will be set on
    /// it to enable scrolling. This is useful when used with menus.
    pub fn add_fallback_placement(
        &mut self,
        base: Point,
        element: Point,
        direction: GrowDirection,
    ) {
        self.placements.push(Placement {
            base,
            element,
            direction,
        });
    }

    /// Builder style method to set the viewport padding.
    pub fn viewport_padding(mut self, padding: f64) -> Self {
        self.set_viewport_padding(padding);
        self
    }

    /// Sets the viewport padding such that there is at least this much
    /// space to the border of the viewport.
    pub fn set_viewport_padding(&mut self, padding: f64) {
        self.viewport_padding = padding;
    }

    fn map_rtl(&mut self, is_rtl: bool) {
        for placement in self.placements.iter_mut() {
            placement.base.map_rtl(is_rtl);
            placement.element.map_rtl(is_rtl);
        }
    }

    /// Builder style method to set `align width`
    pub fn align_width(mut self, align_width: bool) -> Self {
        self.set_align_width(align_width);
        self
    }

    /// Sets if the width of the node should be aligned with the base. This is
    /// useful for drop down windows
    pub fn set_align_width(&mut self, align_width: bool) {
        self.align_width = align_width;
    }
}

#[derive(Debug, Clone)]
struct Rect {
    x: f64,
    y: f64,
    x_end: f64,
    y_end: f64,
}

impl Rect {
    fn width(&self) -> f64 {
        self.x_end - self.x
    }

    fn height(&self) -> f64 {
        self.y_end - self.y
    }

    fn shift(&mut self, x: f64, y: f64) {
        self.x += x;
        self.x_end += x;
        self.y += y;
        self.y_end += y;
    }

    fn get_position(&self, point: Point) -> (f64, f64) {
        let x = match &point {
            Point::TopStart | Point::Start | Point::BottomStart => self.x,
            Point::Top | Point::Center | Point::Bottom => (self.x + self.x_end) / 2.0,
            Point::TopEnd | Point::End | Point::BottomEnd => self.x_end,
        };
        let y = match &point {
            Point::TopStart | Point::Top | Point::TopEnd => self.y,
            Point::Start | Point::Center | Point::End => (self.y + self.y_end) / 2.0,
            Point::BottomStart | Point::Bottom | Point::BottomEnd => self.y_end,
        };
        (x, y)
    }
}

fn get_offset(element: &HtmlElement, point: Point, mut base: (f64, f64)) -> (f64, f64) {
    let rect = element.get_bounding_client_rect();
    let width = rect.width();
    let height = rect.height();

    match &point {
        Point::Top | Point::Center | Point::Bottom => base.0 += width / 2.0,
        Point::TopEnd | Point::End | Point::BottomEnd => base.0 += width,
        _ => {}
    }

    match point {
        Point::Start | Point::Center | Point::End => base.1 += height / 2.0,
        Point::BottomStart | Point::Bottom | Point::BottomEnd => base.1 += height,
        _ => {}
    }

    base
}

fn get_position(
    base: &HtmlElement,
    element: &HtmlElement,
    placement: &Placement,
    offset: (f64, f64),
) -> (f64, f64) {
    let el_rect = base.get_bounding_client_rect();
    let base_pos = get_offset(base, placement.base, (el_rect.x(), el_rect.y()));
    let node_offset = get_offset(element, placement.element, (0.0, 0.0));

    let mut x = base_pos.0 - node_offset.0;
    let mut y = base_pos.1 - node_offset.1;

    if node_offset.0 > 0.0 {
        x -= offset.0;
    } else {
        x += offset.0;
    }

    if node_offset.1 > 0.0 {
        y -= offset.1;
    } else {
        y += offset.1;
    }

    (x, y)
}

// Trys to return the rect where the element should be, moving along the axis of the grow direction
// if possible. If the element is too wide/tall, it still could be taller than the constraint
fn try_fit_rect(
    base: &HtmlElement,
    element: &HtmlElement,
    constraint: &Rect,
    placement: &Placement,
    offset: (f64, f64),
    has_fallback: bool,
) -> Rect {
    let (x, y) = get_position(base, element, placement, offset);
    let (x_end, y_end) = get_offset(element, Point::BottomEnd, (x, y));
    let mut rect = Rect { x, y, x_end, y_end };

    let shift_y = |rect: &mut Rect| {
        if rect.y_end > constraint.y_end {
            rect.shift(0.0, constraint.y_end - rect.y_end);
        }
        if rect.y < constraint.y {
            rect.shift(0.0, constraint.y - rect.y);
        }
    };

    let shift_x = |rect: &mut Rect| {
        if rect.x_end > constraint.x_end {
            rect.shift(constraint.x_end - rect.x_end, 0.0);
        }
        if rect.x < constraint.x {
            rect.shift(constraint.x - rect.x, 0.0);
        }
    };
    // try to move inside viewport along the grow direction if there is a fallback
    if has_fallback {
        match placement.direction {
            GrowDirection::None => {}
            GrowDirection::TopBottom => {
                shift_y(&mut rect);
            }
            GrowDirection::StartEnd => {
                shift_x(&mut rect);
            }
        }
    } else {
        shift_y(&mut rect);
        shift_x(&mut rect);
    }

    rect
}

// check if inner fits into outer, depending on the grow direction
fn fits(inner: &Rect, outer: &Rect, direction: &GrowDirection) -> bool {
    let top = inner.y < outer.y;
    let bottom = inner.y_end > outer.y_end;
    let start = inner.x < outer.x;
    let end = inner.x_end > outer.x_end;

    match direction {
        GrowDirection::None => !(top || bottom || start || end),
        GrowDirection::TopBottom => !(start || end),
        GrowDirection::StartEnd => !(top || bottom),
    }
}

// for 'position: fixed' elements, gets the containing block, either the root, or the next parent
// up with a transform property that is not 'none'
fn get_containing_block(element: &HtmlElement) -> Option<HtmlElement> {
    if element.node_name().to_lowercase() == "dialog" {
        return None;
    }
    let mut current = element.parent_node();

    while let Some(node) = current {
        let node_name = node.node_name().to_lowercase();
        if node_name == "html" || node_name == "body" || node_name == "#document" {
            break;
        }
        if let Some(html) = node.into_html_element() {
            if let Ok(Some(style)) = window().unwrap().get_computed_style(&html) {
                match style.get_property_value("transform") {
                    Ok(transform) if transform != "none" => return Some(html),
                    _ => {}
                }
            }
            current = html.parent_node();
        } else {
            break;
        }
    }
    None
}

// use transform to not affect layouting the inner content of the aligned element
fn set_position_style(
    style: &web_sys::CssStyleDeclaration,
    _element: HtmlElement,
    pos: (f64, f64),
) -> Result<(), Error> {
    style.set_property(
        "transform",
        &format!("translate({}px, {}px)", pos.0.round(), pos.1.round()),
    )?;
    Ok(())
}

/// Aligns `element` to `base`.
///
/// The possible options are described in [`AlignOptions`]. Note that if nested elements need
/// aligning (for examples tooltips inside dialogs), make sure to allow visible overflow on the
/// aligned elements, otherwise there may be issues with scrollbars
pub fn align_to<B, N>(base: B, element: N, options: Option<AlignOptions>) -> Result<(), Error>
where
    B: IntoHtmlElement,
    N: IntoHtmlElement,
{
    let base = base
        .into_html_element()
        .ok_or_else(|| js_sys::Error::new("base is not an HtmlElement"))?;
    let element = element
        .into_html_element()
        .ok_or_else(|| js_sys::Error::new("element is not an HtmlElement"))?;

    let mut options = options.unwrap_or_default();

    // if the base element is rtl, we have to reverse start & end positions
    options.map_rtl(element_direction_rtl(base.clone()).unwrap_or(false));

    let window = web_sys::window().unwrap();
    let window_rect = Rect {
        x: options.viewport_padding,
        y: options.viewport_padding,
        x_end: window.inner_width().unwrap().as_f64().unwrap() - options.viewport_padding,
        y_end: window.inner_height().unwrap().as_f64().unwrap() - options.viewport_padding,
    };

    // set some of the style now, so that it renders with/without possible scroll bar, which influences
    // the size of the rect
    let style = element.style();
    style.set_property("position", "fixed")?;
    style.set_property("inset", "0px auto auto 0px")?;
    if element.scroll_height() as f64 > window_rect.height()
        || element.scroll_width() as f64 > window_rect.width()
    {
        style.set_property("overflow", "auto")?;
    } else {
        style.remove_property("overflow")?;
    }
    let padding = 2.0 * options.viewport_padding;
    style.set_property("max-height", &format!("calc(100vh - {padding}px)"))?;
    style.set_property("max-width", &format!("calc(100vw - {padding}px)"))?;

    if options.align_width {
        let width = base.get_bounding_client_rect().width();
        style.set_property("min-width", &format!("{}px", width))?;
    }

    let num_placements = options.placements.len();
    // try first placement
    let mut rect = try_fit_rect(
        &base,
        &element,
        &window_rect,
        &options.placements[0],
        options.offset,
        num_placements > 1,
    );

    // try fallback placements if the first one does not fit
    if options.placements.len() > 1 && !fits(&rect, &window_rect, &options.placements[0].direction)
    {
        for (idx, placement) in options.placements.iter().skip(1).enumerate() {
            let new_rect = try_fit_rect(
                &base,
                &element,
                &window_rect,
                placement,
                options.offset,
                idx < num_placements,
            );

            if fits(&new_rect, &window_rect, &placement.direction) {
                rect = new_rect;
                break;
            }
        }
    }

    // since the node has 'position: fixed', we must correct the position for nodes which have a
    // transformed parent, see https://developer.mozilla.org/en-US/docs/Web/CSS/position
    if let Some(offset_parent) = get_containing_block(&element) {
        let offset_rect = offset_parent.get_bounding_client_rect();
        rect.shift(
            0.0 - offset_rect.left() + offset_parent.client_left() as f64,
            0.0 - offset_rect.top() + offset_parent.client_top() as f64,
        );
    }

    set_position_style(&style, element, (rect.x, rect.y))?;

    Ok(())
}

/// Aligns the `point` of `element` to the given x/y coordinates.
///
/// The possible options are described in [`AlignOptions`].
pub fn align_to_xy<N>(element: N, coordinates: (f64, f64), point: Point) -> Result<(), Error>
where
    N: IntoHtmlElement,
{
    let element = element
        .into_html_element()
        .ok_or_else(|| js_sys::Error::new("element is not an HtmlElement"))?;

    let style = element.style();
    style.set_property("position", "fixed")?;
    style.set_property("inset", "0px auto auto 0px")?;

    let offset = get_offset(&element, point, (0.0, 0.0));

    set_position_style(
        &style,
        element,
        (coordinates.0 - offset.0, coordinates.1 - offset.1),
    )?;

    Ok(())
}

/// Aligns the given `target` point of `element` to the `base` Point of the window
pub fn align_to_viewport<N: IntoHtmlElement>(
    element: N,
    base: Point,
    target: Point,
) -> Result<(), Error> {
    let element = element
        .into_html_element()
        .ok_or_else(|| js_sys::Error::new("element is not an HtmlElement"))?;

    let style = element.style();
    style.set_property("position", "fixed")?;
    style.set_property("inset", "0px auto auto 0px")?;

    let offset = get_offset(&element, target, (0.0, 0.0));
    let window = window().unwrap();
    let viewport_rect = Rect {
        x: 0.0,
        y: 0.0,
        x_end: window.inner_width().unwrap().as_f64().unwrap(),
        y_end: window.inner_height().unwrap().as_f64().unwrap(),
    };

    let vp_coordinates = viewport_rect.get_position(base);

    set_position_style(
        &style,
        element,
        (vp_coordinates.0 - offset.0, vp_coordinates.1 - offset.1),
    )?;

    Ok(())
}

/// Uses [`align_to`] and a [`SizeObserver`] to automatically adjust the position of floating
/// elements when they change size. This is useful for elements where the initial size is not
/// known (e.g. a [`crate::widget::data_table::DataTable`] with virtual scrolling).
pub struct AutoFloatingPlacement {
    base: HtmlElement,
    element: HtmlElement,
    options: AlignOptions,
    _size_observer: SizeObserver,
}

impl AutoFloatingPlacement {
    /// Sets up the [`SizeObserver`] on `element` and updates the intial alignment.
    pub fn new<B, N>(base: B, element: N, options: AlignOptions) -> Result<Self, Error>
    where
        B: IntoHtmlElement + Clone + 'static,
        N: IntoHtmlElement + Clone + 'static,
    {
        let observer_base = base.clone();
        let observer_element = element.clone();
        let observer_opts = options.clone();

        let base = base
            .into_html_element()
            .ok_or_else(|| js_sys::Error::new("base is not an HtmlElement"))?;
        let element = element
            .into_html_element()
            .ok_or_else(|| js_sys::Error::new("element is not an HtmlElement"))?;

        let size_observer = SizeObserver::new(element.as_ref(), move |(_, _)| {
            if let Err(err) = align_to(
                observer_base.clone(),
                observer_element.clone(),
                Some(observer_opts.clone()),
            ) {
                log::error!("could not align element: {}", err.to_string());
            }
        });
        let this = Self {
            base,
            element,
            options,
            _size_observer: size_observer,
        };

        this.update()?;

        Ok(this)
    }

    /// Updates the placement manually
    pub fn update(&self) -> Result<(), Error> {
        align_to(
            self.base.clone(),
            self.element.clone(),
            Some(self.options.clone()),
        )
    }
}

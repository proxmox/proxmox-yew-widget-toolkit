use crate::widget::charts::map::Coordinates;

// ratio per zooming step
const ZOOMING_RATIO: f64 = 1.1;

/// Represents a zooming state
pub struct ZoomInfo {
    pan: (f64, f64),
    user_scale: f64,
    width: f64,
    height: f64,
    max_zoom_level: f64,
}

/// The zoom action a user can take.
pub enum ZoomAction {
    /// Zooms in by a fixed ratio
    In,
    /// Zooms out by a fixed ratio
    Out,
    /// Resets the zoom level to 1.0
    Reset,
    /// Tries to zoom to this level
    Scale(f64),
}

impl ZoomInfo {
    /// Creates a new ZoomInfo instance.
    pub fn new(width: f64, height: f64, max_zoom_level: f64) -> Self {
        Self {
            pan: (0.0, 0.0),
            user_scale: 1.0,
            width,
            height,
            max_zoom_level,
        }
    }

    /// Updates the zoom to or from the given center point.
    /// Returns true if the scale changed, false otherwise.
    pub fn update_zoom(&mut self, change: ZoomAction, center_x: f64, center_y: f64) -> bool {
        let point_x = (center_x - self.pan.0) / self.user_scale;
        let point_y = (center_y - self.pan.1) / self.user_scale;

        let old_scale = self.user_scale;
        self.update_scale(change);
        if old_scale == self.user_scale {
            return false;
        }

        self.update_pan(
            center_x - point_x * self.user_scale,
            center_y - point_y * self.user_scale,
        );

        true
    }

    fn update_pan(&mut self, x: f64, y: f64) {
        self.pan = (
            x.max(self.width - self.width * self.user_scale).min(0.0),
            y.max(self.height - self.height * self.user_scale).min(0.0),
        );
    }

    /// Pans the view around by the given diff
    pub fn move_pan(&mut self, diff_x: f64, diff_y: f64) {
        self.update_pan(self.pan.0 + diff_x, self.pan.1 + diff_y);
    }

    fn update_scale(&mut self, change: ZoomAction) {
        self.user_scale = match change {
            ZoomAction::In => self.user_scale * ZOOMING_RATIO,
            ZoomAction::Out => self.user_scale / ZOOMING_RATIO,
            ZoomAction::Reset => 1.0,
            ZoomAction::Scale(scale) => scale,
        }
        .clamp(1.0, self.max_zoom_level);
    }

    /// Returns the transform string that pans and scales appropriately.
    pub fn get_transform(&self) -> String {
        format!(
            "translate({:.3}px, {:.3}px) scale({:.3})",
            self.pan.0, self.pan.1, self.user_scale
        )
    }

    /// Maps a coordinate of the original size, to the current zoomed/panned view.
    pub fn map_point(&self, coordinates: Coordinates) -> Coordinates {
        let x = coordinates.x * self.user_scale + self.pan.0;
        let y = coordinates.y * self.user_scale + self.pan.1;
        Coordinates { x, y }
    }

    fn zoomed_size(&self) -> (f64, f64) {
        (self.width / self.user_scale, self.height / self.user_scale)
    }

    /// Centers the view at the given point
    pub fn center_point(&mut self, coordinates: Coordinates) {
        let (width, height) = self.zoomed_size();
        self.update_pan(
            -(coordinates.x - width / 2.0) * self.user_scale,
            -(coordinates.y - height / 2.0) * self.user_scale,
        );
    }

    fn zoom_to_point(&mut self, coordinates: Coordinates, width: f64) {
        self.update_scale(ZoomAction::Scale(self.width / width));
        self.center_point(coordinates);
    }

    /// Changes the scale and pan so that all points of the list are included including the padding
    pub fn zoom_to_points(&mut self, points: impl IntoIterator<Item = Coordinates>, padding: f64) {
        let mut points = points.into_iter().peekable();

        if points.peek().is_none() {
            return;
        }

        let mut x_min = f64::INFINITY;
        let mut x_max = f64::NEG_INFINITY;
        let mut y_min = f64::INFINITY;
        let mut y_max = f64::NEG_INFINITY;

        for Coordinates { x, y } in points {
            if x > x_max {
                x_max = x;
            }
            if y > y_max {
                y_max = y;
            }
            if x < x_min {
                x_min = x;
            }
            if y < y_min {
                y_min = y;
            }
        }

        let width = x_max - x_min + 2.0 * padding;
        let height = y_max - y_min + 2.0 * padding;

        let mid_point = Coordinates {
            x: (x_min + x_max) / 2.0,
            y: (y_min + y_max) / 2.0,
        };

        self.zoom_to_point(mid_point, width.max(height * self.width / self.height));
    }

    /// tests if the given point is outside the current view
    pub fn is_out_of_bounds(&self, coordinates: Coordinates) -> bool {
        !(0.0..=self.width).contains(&coordinates.x)
            || !(0.0..=self.height).contains(&coordinates.y)
    }

    /// Returns the current zoom level
    pub fn get_zoom_level(&self) -> f64 {
        self.user_scale
    }
}

#[cfg(test)]
mod test {
    use super::ZoomInfo;
    use crate::widget::charts::{map::zoom_info::ZoomAction, Coordinates};

    #[test]
    fn test_zooming() {
        let mut zoom = ZoomInfo::new(200.0, 200.0, 100.0);
        let coordinate = Coordinates { x: 100.0, y: 100.0 };
        assert_eq!(zoom.map_point(coordinate), coordinate);
        zoom.update_zoom(ZoomAction::In, 50.0, 50.0);
        assert_eq!(
            zoom.map_point(coordinate),
            Coordinates { x: 105.0, y: 105.0 }
        );

        assert_eq!(zoom.get_zoom_level(), 1.1);
        zoom.update_zoom(ZoomAction::Scale(10.0), 0.0, 0.0);

        assert_eq!(zoom.get_zoom_level(), 10.0);

        zoom.zoom_to_point(Coordinates { x: 50.0, y: 50.0 }, 100.0);

        assert_eq!(zoom.get_zoom_level(), 2.0);
    }
}

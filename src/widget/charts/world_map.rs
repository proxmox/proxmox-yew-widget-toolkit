use std::marker::PhantomData;
use std::rc::Rc;

use geojson::{GeoJson, Geometry};
use yew::prelude::*;

use crate::prelude::*;
use crate::widget::canvas::Path;
use crate::widget::charts::{Coordinates, Map, MapPoint, MapPointData};
use pwt_macros::{builder, widget};

// the constant size of the svg viewport. also use to project lon/lat to svg coordinates
const WIDTH: f64 = 3600.0; // use 10 units per longitude
// Aspect ratio of the equirectangular projection; it equals 2*cos(standard parallel), the
// latitude drawn at true proportions. 1.65 keeps that near 34 degrees, so latitudes poleward
// of it (most of Europe) stretch east-west and those toward the equator compress. Lower the
// ratio to move the true latitude poleward.
const WIDTH_RATIO: f64 = 1.65;
const HEIGHT: f64 = WIDTH / WIDTH_RATIO;

/// Represents a Location in the world using the geographic coordinate system.
#[derive(Clone, Copy, PartialEq)]
pub struct Location {
    pub longitude: f64,
    pub latitude: f64,
}

impl Location {
    /// Create a new location from longitude and latitude
    pub fn new(longitude: f64, latitude: f64) -> Self {
        Self {
            longitude,
            latitude,
        }
    }
}

impl From<Location> for Coordinates {
    fn from(value: Location) -> Self {
        project(value)
    }
}

/// Holds a location and arbitrary data that implements [MapPointData]
///
/// Can be converted into a [MapPoint] (its location will be projected to
/// a coordinate system useful for [WorldMap])
#[derive(Clone, PartialEq)]
pub struct WorldPoint<T: MapPointData> {
    pub location: Location,
    pub data: T,
}

impl<T: MapPointData> From<WorldPoint<T>> for MapPoint<T> {
    fn from(value: WorldPoint<T>) -> Self {
        MapPoint {
            coordinates: value.location.into(),
            data: value.data,
        }
    }
}

/// A world map using GeoJSON data to draw SVG lines and polygons.
#[widget(pwt=crate,comp=WorldMapComp<T>, @element)]
#[builder]
#[derive(Properties, PartialEq, Clone)]
pub struct WorldMap<T: MapPointData + 'static> {
    #[prop_or_default]
    /// A list of points to highlight on the map.
    points: Vec<MapPoint<T>>,

    #[prop_or(30.0)]
    #[builder]
    /// The maximum zoom level that is allowed. Forwarded to the inner [Map].
    max_zoom_level: f64,

    #[prop_or(8.0)]
    #[builder]
    /// The radius for info points. Forwarded to the inner [Map].
    info_point_radius: f64,

    map_data: Rc<GeoJson>,
}

impl<T: MapPointData> WorldMap<T> {
    /// Creates a new WorldMap, takes the necessary GeoJson as Rc to not copy data unnecessarily
    /// around
    pub fn new(map_data: Rc<GeoJson>) -> Self {
        yew::props!(Self { map_data })
    }

    /// Set the points of the map. Converts the List into a list of [MapPoint].
    pub fn set_points(&mut self, points: impl Into<Vec<WorldPoint<T>>>) {
        self.points = points
            .into()
            .into_iter()
            .map(|point| point.into())
            .collect();
    }

    /// Builder style method to set the points of the map. Converts the List into a list of [MapPoint].
    pub fn points(mut self, points: impl Into<Vec<WorldPoint<T>>>) -> Self {
        self.set_points(points);
        self
    }
}

pub struct WorldMapComp<T: MapPointData> {
    path: String,
    _phantom_data: PhantomData<T>,
}

fn calculate_path(geojson: &GeoJson) -> String {
    let mut paths = Vec::new();
    match geojson {
        GeoJson::Geometry(geometry) => {
            paths.append(&mut parse_geometry(geometry));
        }
        GeoJson::Feature(feature) => {
            if let Some(geometry) = &feature.geometry {
                let mut new_paths = parse_geometry(geometry);
                paths.append(&mut new_paths);
            }
        }
        GeoJson::FeatureCollection(feature_collection) => {
            for f in feature_collection {
                if let Some(geometry) = &f.geometry {
                    let mut new_paths = parse_geometry(geometry);
                    paths.append(&mut new_paths);
                }
            }
        }
    }
    paths.join(" ")
}

impl<T: MapPointData + 'static> yew::Component for WorldMapComp<T> {
    type Message = ();
    type Properties = WorldMap<T>;

    fn create(ctx: &Context<Self>) -> Self {
        let path = calculate_path(&ctx.props().map_data);
        Self {
            path,
            _phantom_data: PhantomData::<T>,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        Map::new(
            Path::new()
                .d(self.path.clone())
                .style("vector-effect", "non-scaling-stroke")
                .style("stroke-width", "0.2px")
                .style("fill", "var(--pwt-color-neutral)")
                .style("stroke", "var(--pwt-color-on-neutral)"),
        )
        .with_std_props(&props.std_props)
        .listeners(&props.listeners)
        .style("background-color", "var(--pwt-color-surface)")
        .width(WIDTH)
        .height(HEIGHT)
        .max_zoom_level(props.max_zoom_level)
        .info_point_radius(props.info_point_radius)
        .points(props.points.clone())
        .into()
    }

    fn changed(&mut self, ctx: &Context<Self>, old_props: &Self::Properties) -> bool {
        let props = ctx.props();
        if props.map_data != old_props.map_data {
            self.path = calculate_path(&props.map_data);
        }
        true
    }
}

// equirectangular projection, but with a separate scale per axis (longitude uses WIDTH/360,
// latitude uses HEIGHT/180) so the result looks closer to common maps instead of a flat 2:1 ratio
fn project(location: Location) -> Coordinates {
    Coordinates {
        x: (location.longitude + 180.0) * (WIDTH / 360.0),
        y: (90.0 - location.latitude) * (HEIGHT / 180.0),
    }
}

fn ring_to_path(coordinates: &[Vec<f64>]) -> String {
    let mut path = line_to_path(coordinates);
    path.push('Z');
    path
}

fn line_to_path(coordinates: &[Vec<f64>]) -> String {
    let mut path = String::new();
    let mut prefix = "M";
    for list in coordinates {
        if list.len() < 2 {
            continue;
        }
        let Coordinates { x, y } = project(Location::new(list[0], list[1]));
        path.push_str(&format!("{prefix}{:.2},{:.2}", x, y));
        prefix = "L";
    }
    path
}

fn parse_geometry(geometry: &Geometry) -> Vec<String> {
    let mut paths = Vec::new();

    match &geometry.value {
        geojson::Value::Polygon(polygon) => {
            paths.append(&mut polygon.iter().map(|ring| ring_to_path(ring)).collect());
        }
        geojson::Value::MultiPolygon(items) => {
            for poly in items {
                paths.append(&mut poly.iter().map(|ring| ring_to_path(ring)).collect())
            }
        }
        geojson::Value::LineString(line) => paths.push(line_to_path(line)),
        geojson::Value::MultiLineString(line) => {
            paths.append(&mut line.iter().map(|ring| line_to_path(ring)).collect())
        }
        geojson::Value::GeometryCollection(items) => {
            for geom in items {
                paths.append(&mut parse_geometry(geom));
            }
        }
        geojson::Value::Point(_) => {}
        geojson::Value::MultiPoint(_) => {}
    }
    paths
}

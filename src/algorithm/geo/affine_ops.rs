use crate::algorithm::broadcasting::BroadcastableVec;
use crate::array::*;
use arrow2::types::Offset;
use geo::{AffineTransform, MapCoords};

/// Apply an [`AffineTransform`] like [`scale`](AffineTransform::scale),
/// [`skew`](AffineTransform::skew), or [`rotate`](AffineTransform::rotate) to a
/// [`Geometry`](crate::geometry::Geometry).
///
/// Multiple transformations can be composed in order to be efficiently applied in a single
/// operation. See [`AffineTransform`] for more on how to build up a transformation.
///
/// If you are not composing operations, traits that leverage this same machinery exist which might
/// be more readable. See: [`Scale`](crate::algorithm::Scale),
/// [`Translate`](crate::algorithm::Translate), [`Rotate`](crate::algorithm::Rotate),
/// and [`Skew`](crate::algorithm::Skew).
///
/// # Examples
/// ## Build up transforms by beginning with a constructor, then chaining mutation operations
/// ```
/// use geo::{AffineOps, AffineTransform};
/// use geo::{line_string, BoundingRect, Point, LineString};
/// use approx::assert_relative_eq;
///
/// let ls: LineString = line_string![
///     (x: 0.0f64, y: 0.0f64),
///     (x: 0.0f64, y: 10.0f64),
/// ];
/// let center = ls.bounding_rect().unwrap().center();
///
/// let transform = AffineTransform::skew(40.0, 40.0, center).rotated(45.0, center);
///
/// let skewed_rotated = ls.affine_transform(&transform);
///
/// assert_relative_eq!(skewed_rotated, line_string![
///     (x: 0.5688687f64, y: 4.4311312),
///     (x: -0.5688687, y: 5.5688687)
/// ], max_relative = 1.0);
/// ```
pub trait AffineOps {
    /// Apply `transform` immutably, outputting a new geometry.
    #[must_use]
    fn affine_transform(&self, transform: BroadcastableVec<AffineTransform>) -> Self;

    // TODO: add COW API for affine_transform_mut
    //
    // /// Apply `transform` to mutate `self`.
    // fn affine_transform_mut(&mut self, transform: &AffineTransform<T>);
}

// Note: this can't (easily) be parameterized in the macro because PointArray is not generic over O
impl AffineOps for PointArray {
    fn affine_transform(&self, transform: BroadcastableVec<AffineTransform>) -> Self {
        let output_geoms: Vec<Option<geo::Point>> = self
            .iter_geo()
            .zip(transform.into_iter())
            .map(|(maybe_g, transform)| {
                maybe_g.map(|geom| geom.map_coords(|coord| transform.apply(coord)))
            })
            .collect();

        output_geoms.into()
    }
}

/// Implementation that iterates over geo objects
macro_rules! iter_geo_impl {
    ($type:ty, $geo_type:ty) => {
        impl<O: Offset> AffineOps for $type {
            fn affine_transform(&self, transform: BroadcastableVec<AffineTransform>) -> Self {
                let output_geoms: Vec<Option<$geo_type>> = self
                    .iter_geo()
                    .zip(transform.into_iter())
                    .map(|(maybe_g, transform)| {
                        maybe_g.map(|geom| geom.map_coords(|coord| transform.apply(coord)))
                    })
                    .collect();

                output_geoms.into()
            }
        }
    };
}

iter_geo_impl!(LineStringArray<O>, geo::LineString);
iter_geo_impl!(PolygonArray<O>, geo::Polygon);
iter_geo_impl!(MultiLineStringArray<O>, geo::MultiLineString);
iter_geo_impl!(MultiPolygonArray<O>, geo::MultiPolygon);
iter_geo_impl!(WKBArray<O>, geo::Geometry);

impl<O: Offset> AffineOps for MultiPointArray<O> {
    fn affine_transform(&self, transform: BroadcastableVec<AffineTransform>) -> Self {
        let output_geoms: Vec<Option<geo::MultiPoint>> = self
            .iter_geo()
            .zip(transform.into_iter())
            .map(|(maybe_g, transform)| {
                maybe_g.map(|geom| geom.map_coords(|coord| transform.apply(coord)))
            })
            .collect();

        output_geoms.into()
    }
}

impl<O: Offset> AffineOps for GeometryArray<O> {
    crate::geometry_array_delegate_impl! {
        fn affine_transform(&self, transform: BroadcastableVec<AffineTransform>) -> Self;
    }
}
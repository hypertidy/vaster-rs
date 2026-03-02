//! # vaster
//!
//! Raster grid logic, without any pesky data.
//!
//! Raster grids are defined by **dimension** (ncol, nrow) and **extent**
//! (xmin, xmax, ymin, ymax). Everything else — resolution, cell centres,
//! geotransforms, cell indexing — derives from those six numbers.
//!
//! This crate provides that derivation with **zero dependencies**.
//!
//! ## Conventions
//!
//! - **Extent** is `[xmin, xmax, ymin, ymax]` — same as the R `vaster` package
//! - **Dimension** is `[ncol, nrow]`
//! - **Cell indices** are 0-based (the R package uses 1-based)
//! - **Geotransform** follows GDAL convention:
//!   `[origin_x, pixel_width, row_rotation, origin_y, col_rotation, pixel_height]`
//! - Cell (0, 0) is the **top-left** pixel, traversing right then down
//!
//! ## GDAL geotransform
//!
//! The 6-element geotransform maps pixel coordinates to geographic coordinates:
//!
//! ```text
//! x_geo = gt[0] + pixel * gt[1] + line * gt[2]
//! y_geo = gt[3] + pixel * gt[4] + line * gt[5]
//! ```
//!
//! For north-up images (no rotation): `gt[2] == 0`, `gt[4] == 0`, `gt[5] < 0`.
//!
//! Reference: <https://gdal.org/tutorials/geotransforms_tut.html>
//!
//! ## Quick start
//!
//! ```
//! use vaster::*;
//!
//! let extent = [0.0, 360.0, -90.0, 90.0];
//! let dim = [360, 180];
//!
//! // Build a geotransform
//! let gt = extent_dim_to_gt(&extent, &dim);
//! assert_eq!(gt[1], 1.0); // 1 degree per pixel
//!
//! // Cell centre of top-left pixel
//! let (x, y) = xy_from_cell(&dim, &extent, 0);
//! assert!((x - 0.5).abs() < 1e-10);
//! assert!((y - 89.5).abs() < 1e-10);
//!
//! // Which cell contains a point?
//! let cell = cell_from_xy(&dim, &extent, 10.3, 45.7);
//! assert_eq!(cell, Some(10 + 44 * 360));
//! ```
//!
//! ## Relationship to the R package
//!
//! This crate is the Rust equivalent of [hypertidy/vaster](https://github.com/hypertidy/vaster),
//! an R package for raster grid logic. The API follows the same conventions
//! with one difference: cell indices are **0-based** in Rust (1-based in R).

mod cell;
mod crop;
mod geotransform;
mod world;

pub use cell::*;
pub use crop::*;
pub use geotransform::*;
pub use world::*;

/// A GDAL-style affine geotransform.
///
/// `[origin_x, pixel_width, row_rotation, origin_y, col_rotation, pixel_height]`
///
/// For north-up images: index 2 and 4 are zero, index 5 is negative.
pub type GeoTransform = [f64; 6];

/// Raster extent: `[xmin, xmax, ymin, ymax]`.
pub type Extent = [f64; 4];

/// Raster dimensions: `[ncol, nrow]`.
pub type Dimension = [usize; 2];

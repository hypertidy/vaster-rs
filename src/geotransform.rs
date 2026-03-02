//! Geotransform construction, inversion, and coordinate mapping.
//!
//! The geotransform is the bridge between pixel space and geographic space.
//! These functions operate on raw `[f64; 6]` arrays following the GDAL convention.

use crate::{Dimension, Extent, GeoTransform};

/// Build a geotransform from extent and dimensions.
///
/// # Arguments
/// * `extent` — `[xmin, xmax, ymin, ymax]`
/// * `dim` — `[ncol, nrow]`
///
/// # Returns
/// A 6-element geotransform: `[xmin, pixel_width, 0, ymax, 0, -pixel_height]`
///
/// # Example
/// ```
/// use vaster::extent_dim_to_gt;
///
/// let gt = extent_dim_to_gt(&[0.0, 360.0, -90.0, 90.0], &[360, 180]);
/// assert_eq!(gt[0], 0.0);     // origin x = xmin
/// assert_eq!(gt[1], 1.0);     // 1 degree per pixel
/// assert_eq!(gt[3], 90.0);    // origin y = ymax
/// assert_eq!(gt[5], -1.0);    // negative (north-up)
/// ```
pub fn extent_dim_to_gt(extent: &Extent, dim: &Dimension) -> GeoTransform {
    let pixel_width = (extent[1] - extent[0]) / dim[0] as f64;
    let pixel_height = -(extent[3] - extent[2]) / dim[1] as f64;
    [extent[0], pixel_width, 0.0, extent[3], 0.0, pixel_height]
}

/// Extract extent from a geotransform and dimensions.
///
/// # Example
/// ```
/// use vaster::{extent_dim_to_gt, gt_to_extent};
///
/// let ext = [100.0, 200.0, -50.0, 50.0];
/// let dim = [1000, 500];
/// let gt = extent_dim_to_gt(&ext, &dim);
/// let ext2 = gt_to_extent(&gt, &dim);
/// assert!((ext[0] - ext2[0]).abs() < 1e-10);
/// assert!((ext[3] - ext2[3]).abs() < 1e-10);
/// ```
pub fn gt_to_extent(gt: &GeoTransform, dim: &Dimension) -> Extent {
    let xmin = gt[0];
    let xmax = gt[0] + dim[0] as f64 * gt[1];
    let ymax = gt[3];
    let ymin = gt[3] + dim[1] as f64 * gt[5];
    [xmin, xmax, ymin, ymax]
}

/// Invert a geotransform.
///
/// For non-rotated grids (`gt[2] == 0 && gt[4] == 0`), this is a simple
/// reciprocal. For rotated grids, a full 2×2 matrix inversion is used.
///
/// Returns `None` if the transform is singular (zero determinant).
///
/// # GDAL equivalent
/// `GDALInvGeoTransform()` in `gdaltransformer.cpp`
///
/// # Example
/// ```
/// use vaster::{extent_dim_to_gt, inv_geotransform, x_from_col, col_from_x};
///
/// let gt = extent_dim_to_gt(&[100.0, 200.0, 0.0, 50.0], &[100, 50]);
/// let inv = inv_geotransform(&gt).unwrap();
///
/// // Roundtrip: pixel → geo → pixel
/// let x = x_from_col(&gt, 42.0);
/// let col_back = inv[0] + x * inv[1];
/// assert!((col_back - 42.5).abs() < 1e-10); // 42.5 = centre of pixel 42
/// ```
pub fn inv_geotransform(gt: &GeoTransform) -> Option<GeoTransform> {
    let det = gt[1] * gt[5] - gt[2] * gt[4];
    if det.abs() < 1e-15 {
        return None;
    }
    let inv_det = 1.0 / det;
    Some([
        (gt[2] * gt[3] - gt[0] * gt[5]) * inv_det,
        gt[5] * inv_det,
        -gt[2] * inv_det,
        (gt[0] * gt[4] - gt[1] * gt[3]) * inv_det,
        -gt[4] * inv_det,
        gt[1] * inv_det,
    ])
}

/// Geographic x-coordinate of a pixel column centre.
///
/// # Arguments
/// * `gt` — geotransform
/// * `col` — 0-based column index (can be fractional)
///
/// # Example
/// ```
/// use vaster::{extent_dim_to_gt, x_from_col};
///
/// let gt = extent_dim_to_gt(&[0.0, 10.0, 0.0, 5.0], &[10, 5]);
/// assert!((x_from_col(&gt, 0.0) - 0.5).abs() < 1e-10);
/// assert!((x_from_col(&gt, 9.0) - 9.5).abs() < 1e-10);
/// ```
pub fn x_from_col(gt: &GeoTransform, col: f64) -> f64 {
    gt[0] + (col + 0.5) * gt[1]
}

/// Geographic y-coordinate of a pixel row centre.
///
/// # Arguments
/// * `gt` — geotransform
/// * `row` — 0-based row index (can be fractional)
///
/// # Example
/// ```
/// use vaster::{extent_dim_to_gt, y_from_row};
///
/// let gt = extent_dim_to_gt(&[0.0, 10.0, 0.0, 5.0], &[10, 5]);
/// assert!((y_from_row(&gt, 0.0) - 4.5).abs() < 1e-10);  // top row
/// assert!((y_from_row(&gt, 4.0) - 0.5).abs() < 1e-10);  // bottom row
/// ```
pub fn y_from_row(gt: &GeoTransform, row: f64) -> f64 {
    gt[3] + (row + 0.5) * gt[5]
}

/// Geographic (x, y) from pixel (col, row), using the full affine transform.
///
/// Handles rotated grids. For non-rotated grids, equivalent to
/// `(x_from_col(gt, col), y_from_row(gt, row))`.
///
/// # Example
/// ```
/// use vaster::{extent_dim_to_gt, xy_from_col_row};
///
/// let gt = extent_dim_to_gt(&[0.0, 10.0, 0.0, 5.0], &[10, 5]);
/// let (x, y) = xy_from_col_row(&gt, 0.0, 0.0);
/// assert!((x - 0.5).abs() < 1e-10);
/// assert!((y - 4.5).abs() < 1e-10);
/// ```
pub fn xy_from_col_row(gt: &GeoTransform, col: f64, row: f64) -> (f64, f64) {
    let pixel = col + 0.5;
    let line = row + 0.5;
    (
        gt[0] + pixel * gt[1] + line * gt[2],
        gt[3] + pixel * gt[4] + line * gt[5],
    )
}

/// Fractional column index from geographic x-coordinate.
///
/// The returned value is the fractional position of the coordinate within the grid.
/// Floor the result to get the 0-based integer column index.
///
/// For non-rotated grids only. Use [`col_row_from_xy`] for rotated grids.
///
/// # Example
/// ```
/// use vaster::{extent_dim_to_gt, col_from_x, x_from_col};
///
/// let gt = extent_dim_to_gt(&[0.0, 10.0, 0.0, 5.0], &[10, 5]);
/// let x = x_from_col(&gt, 3.0);
/// assert!((col_from_x(&gt, x) - 3.0).abs() < 1e-10);
/// ```
pub fn col_from_x(gt: &GeoTransform, x: f64) -> f64 {
    (x - gt[0]) / gt[1] - 0.5
}

/// Fractional row index from geographic y-coordinate.
///
/// The returned value is the fractional position of the coordinate within the grid.
/// Floor the result to get the 0-based integer row index.
///
/// For non-rotated grids only. Use [`col_row_from_xy`] for rotated grids.
///
/// # Example
/// ```
/// use vaster::{extent_dim_to_gt, row_from_y, y_from_row};
///
/// let gt = extent_dim_to_gt(&[0.0, 10.0, 0.0, 5.0], &[10, 5]);
/// let y = y_from_row(&gt, 2.0);
/// assert!((row_from_y(&gt, y) - 2.0).abs() < 1e-10);
/// ```
pub fn row_from_y(gt: &GeoTransform, y: f64) -> f64 {
    (y - gt[3]) / gt[5] - 0.5
}

/// Fractional (col, row) from geographic (x, y), using the full affine inverse.
///
/// Handles rotated grids. Returns `None` if the geotransform is singular.
///
/// # Example
/// ```
/// use vaster::{extent_dim_to_gt, xy_from_col_row, col_row_from_xy};
///
/// let gt = extent_dim_to_gt(&[100.0, 200.0, -50.0, 50.0], &[100, 100]);
/// let (x, y) = xy_from_col_row(&gt, 42.0, 17.0);
/// let (col, row) = col_row_from_xy(&gt, x, y).unwrap();
/// assert!((col - 42.0).abs() < 1e-10);
/// assert!((row - 17.0).abs() < 1e-10);
/// ```
pub fn col_row_from_xy(gt: &GeoTransform, x: f64, y: f64) -> Option<(f64, f64)> {
    let inv = inv_geotransform(gt)?;
    let pixel = inv[0] + x * inv[1] + y * inv[2];
    let line = inv[3] + x * inv[4] + y * inv[5];
    Some((pixel - 0.5, line - 0.5))
}

/// Resolution (pixel width) in the x direction.
///
/// # Example
/// ```
/// use vaster::x_res;
/// assert!((x_res(&[10, 5], &[0.0, 100.0, 0.0, 50.0]) - 10.0).abs() < 1e-10);
/// ```
pub fn x_res(dim: &Dimension, extent: &Extent) -> f64 {
    (extent[1] - extent[0]) / dim[0] as f64
}

/// Resolution (pixel height) in the y direction.
///
/// Always returned as a positive value.
///
/// # Example
/// ```
/// use vaster::y_res;
/// assert!((y_res(&[10, 5], &[0.0, 100.0, 0.0, 50.0]) - 10.0).abs() < 1e-10);
/// ```
pub fn y_res(dim: &Dimension, extent: &Extent) -> f64 {
    (extent[3] - extent[2]) / dim[1] as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_col() {
        let gt = [100.0, 10.0, 0.0, 200.0, 0.0, -10.0];
        let x = x_from_col(&gt, 5.0);
        let c = col_from_x(&gt, x);
        assert!((c - 5.0).abs() < 1e-10);
    }

    #[test]
    fn roundtrip_row() {
        let gt = [100.0, 10.0, 0.0, 200.0, 0.0, -10.0];
        let y = y_from_row(&gt, 3.0);
        let r = row_from_y(&gt, y);
        assert!((r - 3.0).abs() < 1e-10);
    }

    #[test]
    fn roundtrip_extent() {
        let ext = [100.0, 200.0, -50.0, 50.0];
        let dim = [1000, 500];
        let gt = extent_dim_to_gt(&ext, &dim);
        let ext2 = gt_to_extent(&gt, &dim);
        for i in 0..4 {
            assert!((ext[i] - ext2[i]).abs() < 1e-10);
        }
    }

    #[test]
    fn inv_geotransform_roundtrip() {
        let gt = [100.0, 10.0, 0.0, 500.0, 0.0, -10.0];
        let inv = inv_geotransform(&gt).unwrap();
        // Pixel (0,0) centre is at geo (105, 495)
        let x = 105.0;
        let y = 495.0;
        let col = inv[0] + x * inv[1] + y * inv[2];
        let row = inv[3] + x * inv[4] + y * inv[5];
        assert!((col - 0.5).abs() < 1e-10);
        assert!((row - 0.5).abs() < 1e-10);
    }

    #[test]
    fn inv_geotransform_full_affine() {
        // Rotated grid
        let gt = [100.0, 10.0, 2.0, 500.0, -1.0, -10.0];
        let inv = inv_geotransform(&gt).unwrap();
        // Roundtrip: apply gt then inv
        let pixel = 3.5;
        let line = 7.5;
        let x = gt[0] + pixel * gt[1] + line * gt[2];
        let y = gt[3] + pixel * gt[4] + line * gt[5];
        let pixel2 = inv[0] + x * inv[1] + y * inv[2];
        let line2 = inv[3] + x * inv[4] + y * inv[5];
        assert!((pixel - pixel2).abs() < 1e-10);
        assert!((line - line2).abs() < 1e-10);
    }

    #[test]
    fn inv_geotransform_singular() {
        // Zero pixel width → singular
        let gt = [100.0, 0.0, 0.0, 500.0, 0.0, -10.0];
        assert!(inv_geotransform(&gt).is_none());
    }

    #[test]
    fn col_row_from_xy_roundtrip() {
        let gt = extent_dim_to_gt(&[100.0, 200.0, -50.0, 50.0], &[100, 100]);
        for col in [0.0, 25.0, 50.0, 99.0] {
            for row in [0.0, 25.0, 50.0, 99.0] {
                let (x, y) = xy_from_col_row(&gt, col, row);
                let (c2, r2) = col_row_from_xy(&gt, x, y).unwrap();
                assert!((col - c2).abs() < 1e-10, "col {col} → {c2}");
                assert!((row - r2).abs() < 1e-10, "row {row} → {r2}");
            }
        }
    }
}

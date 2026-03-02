//! Grid alignment and cropping.
//!
//! The key operation is "snap to grid": given an arbitrary bounding box,
//! find the smallest sub-grid of a parent grid that fully contains it.
//! This is the Rust equivalent of R vaster's `vcrop()`.

use crate::{Dimension, Extent};

/// Snap an extent to align with a parent grid.
///
/// Given a target extent and a parent grid (defined by dimension and extent),
/// returns the aligned extent and the corresponding sub-grid dimensions.
/// The aligned extent always "snaps out" — it is at least as large as the
/// target, and its edges fall exactly on cell boundaries of the parent grid.
///
/// # Arguments
/// * `target` — the extent to align `[xmin, xmax, ymin, ymax]`
/// * `parent_dim` — dimensions of the parent grid `[ncol, nrow]`
/// * `parent_extent` — extent of the parent grid
///
/// # Returns
/// `(aligned_extent, aligned_dimension)` — the sub-grid that covers `target`
///
/// # Example
/// ```
/// use vaster::vcrop;
///
/// // Parent grid: 360×180 covering the world
/// let parent_dim = [360, 180];
/// let parent_extent = [-180.0, 180.0, -90.0, 90.0];
///
/// // Target: a rough bounding box
/// let target = [10.3, 20.7, -5.2, 8.9];
///
/// let (ext, dim) = vcrop(&target, &parent_dim, &parent_extent);
///
/// // Aligned extent snaps out to grid boundaries
/// assert!((ext[0] - 10.0).abs() < 1e-10);   // snapped left
/// assert!((ext[1] - 21.0).abs() < 1e-10);   // snapped right
/// assert!((ext[2] - (-6.0)).abs() < 1e-10);  // snapped down
/// assert!((ext[3] - 9.0).abs() < 1e-10);    // snapped up
///
/// assert_eq!(dim, [11, 15]);
/// ```
pub fn vcrop(
    target: &Extent,
    parent_dim: &Dimension,
    parent_extent: &Extent,
) -> (Extent, Dimension) {
    let x_res = (parent_extent[1] - parent_extent[0]) / parent_dim[0] as f64;
    let y_res = (parent_extent[3] - parent_extent[2]) / parent_dim[1] as f64;

    // Compute fractional column/row indices of the target corners
    // in the parent grid, then snap outward.
    let col_min = ((target[0] - parent_extent[0]) / x_res).floor() as i64;
    let col_max = ((target[1] - parent_extent[0]) / x_res).ceil() as i64;
    let row_min = ((target[2] - parent_extent[2]) / y_res).floor() as i64;
    let row_max = ((target[3] - parent_extent[2]) / y_res).ceil() as i64;

    // Clamp to parent grid bounds
    let col_min = col_min.max(0) as usize;
    let col_max = (col_max as usize).min(parent_dim[0]);
    let row_min = row_min.max(0) as usize;
    let row_max = (row_max as usize).min(parent_dim[1]);

    let aligned_extent = [
        parent_extent[0] + col_min as f64 * x_res,
        parent_extent[0] + col_max as f64 * x_res,
        parent_extent[2] + row_min as f64 * y_res,
        parent_extent[2] + row_max as f64 * y_res,
    ];

    let aligned_dim = [col_max - col_min, row_max - row_min];

    (aligned_extent, aligned_dim)
}

/// Compute the column and row offset of an aligned sub-grid within its parent.
///
/// Returns `(col_offset, row_offset)` — the 0-based position of the sub-grid's
/// top-left cell within the parent grid.
///
/// # Example
/// ```
/// use vaster::{vcrop, crop_offset};
///
/// let parent_dim = [360, 180];
/// let parent_extent = [-180.0, 180.0, -90.0, 90.0];
/// let target = [10.3, 20.7, -5.2, 8.9];
/// let (ext, _dim) = vcrop(&target, &parent_dim, &parent_extent);
///
/// let (col_off, row_off) = crop_offset(&ext, &parent_dim, &parent_extent);
/// assert_eq!(col_off, 190);  // 10 degrees east of -180 → column 190
/// assert_eq!(row_off, 81);   // rows count from top (90°N), 90 - 9 = 81
/// ```
pub fn crop_offset(
    sub_extent: &Extent,
    parent_dim: &Dimension,
    parent_extent: &Extent,
) -> (usize, usize) {
    let x_res = (parent_extent[1] - parent_extent[0]) / parent_dim[0] as f64;
    let y_res = (parent_extent[3] - parent_extent[2]) / parent_dim[1] as f64;

    let col_off = ((sub_extent[0] - parent_extent[0]) / x_res).round() as usize;
    // Row offset from the top (ymax end)
    let row_off = ((parent_extent[3] - sub_extent[3]) / y_res).round() as usize;

    (col_off, row_off)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vcrop_exact_alignment() {
        // Target already aligned to grid
        let parent_dim = [10, 10];
        let parent_extent = [0.0, 10.0, 0.0, 10.0];
        let target = [2.0, 5.0, 3.0, 7.0];
        let (ext, dim) = vcrop(&target, &parent_dim, &parent_extent);
        assert!((ext[0] - 2.0).abs() < 1e-10);
        assert!((ext[1] - 5.0).abs() < 1e-10);
        assert!((ext[2] - 3.0).abs() < 1e-10);
        assert!((ext[3] - 7.0).abs() < 1e-10);
        assert_eq!(dim, [3, 4]);
    }

    #[test]
    fn vcrop_snaps_outward() {
        let parent_dim = [10, 10];
        let parent_extent = [0.0, 10.0, 0.0, 10.0];
        let target = [2.3, 4.7, 3.1, 6.9];
        let (ext, dim) = vcrop(&target, &parent_dim, &parent_extent);
        // Should snap out
        assert!((ext[0] - 2.0).abs() < 1e-10);
        assert!((ext[1] - 5.0).abs() < 1e-10);
        assert!((ext[2] - 3.0).abs() < 1e-10);
        assert!((ext[3] - 7.0).abs() < 1e-10);
        assert_eq!(dim, [3, 4]);
    }

    #[test]
    fn vcrop_clamped_to_parent() {
        let parent_dim = [10, 10];
        let parent_extent = [0.0, 10.0, 0.0, 10.0];
        let target = [-5.0, 15.0, -5.0, 15.0];
        let (ext, dim) = vcrop(&target, &parent_dim, &parent_extent);
        assert!((ext[0] - 0.0).abs() < 1e-10);
        assert!((ext[1] - 10.0).abs() < 1e-10);
        assert!((ext[2] - 0.0).abs() < 1e-10);
        assert!((ext[3] - 10.0).abs() < 1e-10);
        assert_eq!(dim, [10, 10]);
    }

    #[test]
    fn crop_offset_basic() {
        let parent_dim = [360, 180];
        let parent_extent = [-180.0, 180.0, -90.0, 90.0];
        let sub_extent = [10.0, 21.0, -6.0, 9.0];
        let (col_off, row_off) = crop_offset(&sub_extent, &parent_dim, &parent_extent);
        assert_eq!(col_off, 190);
        assert_eq!(row_off, 81);
    }
}

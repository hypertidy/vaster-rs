//! Cell index arithmetic.
//!
//! Cells are numbered 0-based, starting from the top-left corner,
//! traversing left-to-right then top-to-bottom (row-major order).
//!
//! ```text
//!  0  1  2  3  4
//!  5  6  7  8  9
//! 10 11 12 13 14
//! ```
//!
//! This matches GDAL's pixel ordering and the R `vaster` package
//! (except R uses 1-based indexing).

use crate::{geotransform, Dimension, Extent};

/// Total number of cells.
///
/// # Example
/// ```
/// use vaster::ncell;
/// assert_eq!(ncell(&[10, 5]), 50);
/// ```
pub fn ncell(dim: &Dimension) -> usize {
    dim[0] * dim[1]
}

/// Cell index from 0-based (row, col).
///
/// Returns `None` if row or col is out of bounds.
///
/// # Example
/// ```
/// use vaster::cell_from_row_col;
/// assert_eq!(cell_from_row_col(&[5, 3], 0, 0), Some(0));
/// assert_eq!(cell_from_row_col(&[5, 3], 1, 2), Some(7));
/// assert_eq!(cell_from_row_col(&[5, 3], 3, 0), None); // row out of bounds
/// ```
pub fn cell_from_row_col(dim: &Dimension, row: usize, col: usize) -> Option<usize> {
    if col >= dim[0] || row >= dim[1] {
        return None;
    }
    Some(row * dim[0] + col)
}

/// Row index (0-based) from cell index.
///
/// # Panics
/// Panics if `dim[0]` is zero.
///
/// # Example
/// ```
/// use vaster::row_from_cell;
/// assert_eq!(row_from_cell(&[5, 3], 7), 1);
/// assert_eq!(row_from_cell(&[5, 3], 0), 0);
/// ```
pub fn row_from_cell(dim: &Dimension, cell: usize) -> usize {
    cell / dim[0]
}

/// Column index (0-based) from cell index.
///
/// # Panics
/// Panics if `dim[0]` is zero.
///
/// # Example
/// ```
/// use vaster::col_from_cell;
/// assert_eq!(col_from_cell(&[5, 3], 7), 2);
/// assert_eq!(col_from_cell(&[5, 3], 0), 0);
/// ```
pub fn col_from_cell(dim: &Dimension, cell: usize) -> usize {
    cell % dim[0]
}

/// (row, col) from cell index (both 0-based).
///
/// # Example
/// ```
/// use vaster::row_col_from_cell;
/// assert_eq!(row_col_from_cell(&[5, 3], 7), (1, 2));
/// ```
pub fn row_col_from_cell(dim: &Dimension, cell: usize) -> (usize, usize) {
    (row_from_cell(dim, cell), col_from_cell(dim, cell))
}

/// Geographic (x, y) of a cell centre, from extent and dimension.
///
/// This is the primary cell-to-coordinate function when you don't need
/// a pre-built geotransform.
///
/// # Example
/// ```
/// use vaster::xy_from_cell;
///
/// let dim = [4, 3];
/// let extent = [0.0, 4.0, 0.0, 3.0];
///
/// // Top-left cell
/// let (x, y) = xy_from_cell(&dim, &extent, 0);
/// assert!((x - 0.5).abs() < 1e-10);
/// assert!((y - 2.5).abs() < 1e-10);
///
/// // Bottom-right cell
/// let (x, y) = xy_from_cell(&dim, &extent, 11);
/// assert!((x - 3.5).abs() < 1e-10);
/// assert!((y - 0.5).abs() < 1e-10);
/// ```
pub fn xy_from_cell(dim: &Dimension, extent: &Extent, cell: usize) -> (f64, f64) {
    let gt = geotransform::extent_dim_to_gt(extent, dim);
    let col = col_from_cell(dim, cell);
    let row = row_from_cell(dim, cell);
    geotransform::xy_from_col_row(&gt, col as f64, row as f64)
}

/// Cell index from geographic (x, y) coordinates.
///
/// Returns `None` if the point falls outside the grid extent.
///
/// # Example
/// ```
/// use vaster::cell_from_xy;
///
/// let dim = [4, 3];
/// let extent = [0.0, 4.0, 0.0, 3.0];
///
/// assert_eq!(cell_from_xy(&dim, &extent, 0.5, 2.5), Some(0));  // top-left
/// assert_eq!(cell_from_xy(&dim, &extent, 3.5, 0.5), Some(11)); // bottom-right
/// assert_eq!(cell_from_xy(&dim, &extent, -1.0, 1.5), None);    // outside
/// ```
pub fn cell_from_xy(dim: &Dimension, extent: &Extent, x: f64, y: f64) -> Option<usize> {
    // All four boundary edges are included, matching R vaster / terra.
    if x < extent[0] || x > extent[1] || y < extent[2] || y > extent[3] {
        return None;
    }

    let x_res = (extent[1] - extent[0]) / dim[0] as f64;
    let y_res = (extent[3] - extent[2]) / dim[1] as f64;

    let col = ((x - extent[0]) / x_res) as usize;
    let row = ((extent[3] - y) / y_res) as usize;

    // Clamp to valid range (handles points exactly on xmax or ymin boundary)
    let col = col.min(dim[0] - 1);
    let row = row.min(dim[1] - 1);

    Some(row * dim[0] + col)
}

/// X-coordinates of cell centres for each column.
///
/// Returns a vector of length `dim[0]`.
///
/// # Example
/// ```
/// use vaster::x_centre;
///
/// let xs = x_centre(&[4, 2], &[0.0, 4.0, 0.0, 2.0]);
/// assert_eq!(xs.len(), 4);
/// assert!((xs[0] - 0.5).abs() < 1e-10);
/// assert!((xs[3] - 3.5).abs() < 1e-10);
/// ```
pub fn x_centre(dim: &Dimension, extent: &Extent) -> Vec<f64> {
    let gt = geotransform::extent_dim_to_gt(extent, dim);
    (0..dim[0])
        .map(|col| geotransform::x_from_col(&gt, col as f64))
        .collect()
}

/// Y-coordinates of cell centres for each row.
///
/// Returns a vector of length `dim[1]`, from top (ymax side) to bottom (ymin side).
///
/// # Example
/// ```
/// use vaster::y_centre;
///
/// let ys = y_centre(&[4, 3], &[0.0, 4.0, 0.0, 3.0]);
/// assert_eq!(ys.len(), 3);
/// assert!((ys[0] - 2.5).abs() < 1e-10);  // top row
/// assert!((ys[2] - 0.5).abs() < 1e-10);  // bottom row
/// ```
pub fn y_centre(dim: &Dimension, extent: &Extent) -> Vec<f64> {
    let gt = geotransform::extent_dim_to_gt(extent, dim);
    (0..dim[1])
        .map(|row| geotransform::y_from_row(&gt, row as f64))
        .collect()
}

/// X-coordinates of cell edges (corners) for each column boundary.
///
/// Returns a vector of length `dim[0] + 1` (one more than the number of columns).
///
/// # Example
/// ```
/// use vaster::x_corner;
///
/// let xs = x_corner(&[4, 2], &[0.0, 4.0, 0.0, 2.0]);
/// assert_eq!(xs.len(), 5);
/// assert!((xs[0] - 0.0).abs() < 1e-10);
/// assert!((xs[4] - 4.0).abs() < 1e-10);
/// ```
pub fn x_corner(dim: &Dimension, extent: &Extent) -> Vec<f64> {
    let res = geotransform::x_res(dim, extent);
    (0..=dim[0]).map(|i| extent[0] + i as f64 * res).collect()
}

/// Y-coordinates of cell edges (corners) for each row boundary.
///
/// Returns a vector of length `dim[1] + 1`, from top (ymax) to bottom (ymin).
///
/// # Example
/// ```
/// use vaster::y_corner;
///
/// let ys = y_corner(&[4, 3], &[0.0, 4.0, 0.0, 3.0]);
/// assert_eq!(ys.len(), 4);
/// assert!((ys[0] - 3.0).abs() < 1e-10);  // ymax
/// assert!((ys[3] - 0.0).abs() < 1e-10);  // ymin
/// ```
pub fn y_corner(dim: &Dimension, extent: &Extent) -> Vec<f64> {
    let res = geotransform::y_res(dim, extent);
    (0..=dim[1]).map(|i| extent[3] - i as f64 * res).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cell_row_col_roundtrip() {
        let dim = [10, 8];
        for cell in 0..ncell(&dim) {
            let (row, col) = row_col_from_cell(&dim, cell);
            assert_eq!(cell_from_row_col(&dim, row, col), Some(cell));
        }
    }

    #[test]
    fn xy_cell_roundtrip() {
        let dim = [20, 15];
        let extent = [-180.0, 180.0, -90.0, 90.0];
        for cell in 0..ncell(&dim) {
            let (x, y) = xy_from_cell(&dim, &extent, cell);
            let cell2 = cell_from_xy(&dim, &extent, x, y);
            assert_eq!(cell2, Some(cell), "failed for cell {cell}");
        }
    }

    #[test]
    fn cell_from_xy_outside() {
        let dim = [10, 10];
        let extent = [0.0, 10.0, 0.0, 10.0];
        // Outside
        assert_eq!(cell_from_xy(&dim, &extent, -0.1, 5.0), None);
        assert_eq!(cell_from_xy(&dim, &extent, 10.1, 5.0), None);
        assert_eq!(cell_from_xy(&dim, &extent, 5.0, -0.1), None);
        assert_eq!(cell_from_xy(&dim, &extent, 5.0, 10.1), None);
        // All four boundaries are included (matches R vaster / terra)
        assert!(cell_from_xy(&dim, &extent, 0.0, 5.0).is_some()); // xmin
        assert!(cell_from_xy(&dim, &extent, 10.0, 5.0).is_some()); // xmax
        assert!(cell_from_xy(&dim, &extent, 5.0, 0.0).is_some()); // ymin
        assert!(cell_from_xy(&dim, &extent, 5.0, 10.0).is_some()); // ymax
    }

    #[test]
    fn corners_and_centres_consistent() {
        let dim = [5, 3];
        let extent = [0.0, 10.0, 0.0, 6.0];
        let xc = x_corner(&dim, &extent);
        let xm = x_centre(&dim, &extent);
        // Each centre should be midway between its corners
        for i in 0..dim[0] {
            let mid = (xc[i] + xc[i + 1]) / 2.0;
            assert!((xm[i] - mid).abs() < 1e-10);
        }
        let yc = y_corner(&dim, &extent);
        let ym = y_centre(&dim, &extent);
        for i in 0..dim[1] {
            let mid = (yc[i] + yc[i + 1]) / 2.0;
            assert!((ym[i] - mid).abs() < 1e-10);
        }
    }
}

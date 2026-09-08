//! Cross-validation tests against the R `vaster` package.
//!
//! These test values were computed in R and hardcoded here to ensure
//! the Rust implementation matches the R package exactly.
//!
//! R vaster uses 1-based cell indices; we adjust to 0-based here.

use vaster::*;

// ---------------------------------------------------------------------------
// Cell indexing (vs R vaster)
// ---------------------------------------------------------------------------

/// R: xy_from_cell(c(40, 36), c(-180, 180, -90, 90), 1)
///  → (-175.5, 87.5)
/// (R cell 1 = Rust cell 0)
#[test]
fn r_xy_from_cell_world_grid() {
    let dim = [40, 36];
    let extent = [-180.0, 180.0, -90.0, 90.0];
    let (x, y) = xy_from_cell(&dim, &extent, 0);
    assert!((x - (-175.5)).abs() < 1e-10);
    assert!((y - 87.5).abs() < 1e-10);
}

/// R: xy_from_cell(c(40, 36), c(-180, 180, -90, 90), 1440)
///  → (175.5, -87.5)
/// (R cell 1440 = Rust cell 1439 = last cell)
#[test]
fn r_xy_from_cell_last() {
    let dim = [40, 36];
    let extent = [-180.0, 180.0, -90.0, 90.0];
    let (x, y) = xy_from_cell(&dim, &extent, 1439);
    assert!((x - 175.5).abs() < 1e-10);
    assert!((y - (-87.5)).abs() < 1e-10);
}

/// R: cell_from_xy(c(40, 36), c(-180, 180, -90, 90), cbind(0, 0))
///  → 741 (1-based) = 740 (0-based)
/// col = ((0 - (-180)) / 9) as usize = 20
/// row = ((90 - 0) / 5) as usize = 18
/// cell = 18 * 40 + 20 = 740
#[test]
fn r_cell_from_xy_origin() {
    let dim = [40, 36];
    let extent = [-180.0, 180.0, -90.0, 90.0];
    let cell = cell_from_xy(&dim, &extent, 0.0, 0.0);
    assert_eq!(cell, Some(740));
}

// ---------------------------------------------------------------------------
// Geotransform (vs GDAL convention)
// ---------------------------------------------------------------------------

/// A 1-degree global grid should have pixel width = 1, pixel height = -1,
/// origin at (-180, 90).
#[test]
fn global_1deg_geotransform() {
    let gt = extent_dim_to_gt(&[-180.0, 180.0, -90.0, 90.0], &[360, 180]);
    assert!((gt[0] - (-180.0)).abs() < 1e-10);
    assert!((gt[1] - 1.0).abs() < 1e-10);
    assert!((gt[2] - 0.0).abs() < 1e-10);
    assert!((gt[3] - 90.0).abs() < 1e-10);
    assert!((gt[4] - 0.0).abs() < 1e-10);
    assert!((gt[5] - (-1.0)).abs() < 1e-10);
}

/// Sentinel-2 10m band: 10980×10980 pixels, 109800m extent
/// (UTM zone, origin at zone corner)
#[test]
fn sentinel2_geotransform() {
    // Typical T55GCM tile
    let extent = [499980.0, 609780.0, -4900020.0, -4790220.0];
    let dim = [10980, 10980];
    let gt = extent_dim_to_gt(&extent, &dim);
    assert!((gt[1] - 10.0).abs() < 1e-10); // 10m pixels
    assert!((gt[5] - (-10.0)).abs() < 1e-10); // north-up
    assert!((gt[0] - 499980.0).abs() < 1e-10);
    assert!((gt[3] - (-4790220.0)).abs() < 1e-10); // ymax
}

// ---------------------------------------------------------------------------
// Geotransform inversion
// ---------------------------------------------------------------------------

/// Inversion should roundtrip through geo coordinates.
#[test]
fn inv_geotransform_roundtrip_through_coords() {
    let gt = extent_dim_to_gt(
        &[499980.0, 609780.0, -4900020.0, -4790220.0],
        &[10980, 10980],
    );
    let inv = inv_geotransform(&gt).unwrap();

    // Pixel (5434, 2646) centre
    let x = x_from_col(&gt, 5434.0);
    let y = y_from_row(&gt, 2646.0);

    // Inverse should recover the pixel coordinates (as pixel centres: col + 0.5)
    let col_back = inv[0] + x * inv[1] + y * inv[2];
    let row_back = inv[3] + x * inv[4] + y * inv[5];
    assert!((col_back - 5434.5).abs() < 1e-10);
    assert!((row_back - 2646.5).abs() < 1e-10);
}

// ---------------------------------------------------------------------------
// vcrop (vs R vaster::vcrop)
// ---------------------------------------------------------------------------

/// R: vcrop(c(10.3, 20.7, -5.2, 8.9), c(360, 180), extent = c(-180, 180, -90, 90))
///  → $extent = [10, 21, -6, 9], $dimension = [11, 15]
#[test]
fn r_vcrop_world_grid() {
    let parent_dim = [360, 180];
    let parent_extent = [-180.0, 180.0, -90.0, 90.0];
    let target = [10.3, 20.7, -5.2, 8.9];

    let (ext, dim) = vcrop(&target, &parent_dim, &parent_extent);
    assert!((ext[0] - 10.0).abs() < 1e-10);
    assert!((ext[1] - 21.0).abs() < 1e-10);
    assert!((ext[2] - (-6.0)).abs() < 1e-10);
    assert!((ext[3] - 9.0).abs() < 1e-10);
    assert_eq!(dim, [11, 15]);
}

// ---------------------------------------------------------------------------
// Resolution
// ---------------------------------------------------------------------------

#[test]
fn resolution_global() {
    let dim = [360, 180];
    let extent = [-180.0, 180.0, -90.0, 90.0];
    assert!((x_res(&dim, &extent) - 1.0).abs() < 1e-10);
    assert!((y_res(&dim, &extent) - 1.0).abs() < 1e-10);
}

#[test]
fn resolution_sentinel2() {
    let dim = [10980, 10980];
    let extent = [499980.0, 609780.0, -4900020.0, -4790220.0];
    assert!((x_res(&dim, &extent) - 10.0).abs() < 1e-10);
    assert!((y_res(&dim, &extent) - 10.0).abs() < 1e-10);
}

// ---------------------------------------------------------------------------
// World file
// ---------------------------------------------------------------------------

#[test]
fn world_file_global_1deg() {
    let gt = extent_dim_to_gt(&[-180.0, 180.0, -90.0, 90.0], &[360, 180]);
    let wf = geotransform_to_world(&gt);
    // x_res, y_skew, x_skew, y_res, x_centre, y_centre
    assert!((wf[0] - 1.0).abs() < 1e-10); // x_res
    assert!((wf[1] - 0.0).abs() < 1e-10); // no skew
    assert!((wf[2] - 0.0).abs() < 1e-10); // no skew
    assert!((wf[3] - (-1.0)).abs() < 1e-10); // y_res (negative)
    assert!((wf[4] - (-179.5)).abs() < 1e-10); // centre of top-left pixel
    assert!((wf[5] - 89.5).abs() < 1e-10);
}

// ---------------------------------------------------------------------------
// Edge cases
// ---------------------------------------------------------------------------

#[test]
fn single_cell_grid() {
    let dim = [1, 1];
    let extent = [0.0, 1.0, 0.0, 1.0];
    let (x, y) = xy_from_cell(&dim, &extent, 0);
    assert!((x - 0.5).abs() < 1e-10);
    assert!((y - 0.5).abs() < 1e-10);
    assert_eq!(ncell(&dim), 1);
}

#[test]
fn cell_from_xy_on_boundary() {
    // R: cell_from_xy(c(10, 10), c(0, 10, 0, 10), cbind(1.0, 9.5)) → 2 (1-based)
    // x=1.0 is on the boundary between col 0 and col 1.
    // col = ((1.0 - 0.0) / 1.0) as usize = 1 → Rust cell 1
    let dim = [10, 10];
    let extent = [0.0, 10.0, 0.0, 10.0];
    assert_eq!(cell_from_xy(&dim, &extent, 1.0, 9.5), Some(1));
    assert_eq!(cell_from_xy(&dim, &extent, 1.5, 9.5), Some(1));

    // R boundary values from R vaster:
    // cell_from_xy(c(10,10), c(0,10,0,10), cbind(0.0, 5.0))  → 51 (R) = 50
    // cell_from_xy(c(10,10), c(0,10,0,10), cbind(10.0, 5.0)) → 60 (R) = 59
    // cell_from_xy(c(10,10), c(0,10,0,10), cbind(5.0, 0.0))  → 96 (R) = 95
    // cell_from_xy(c(10,10), c(0,10,0,10), cbind(5.0, 10.0)) → 6  (R) = 5
    assert_eq!(cell_from_xy(&dim, &extent, 0.0, 5.0), Some(50));
    assert_eq!(cell_from_xy(&dim, &extent, 10.0, 5.0), Some(59));
    assert_eq!(cell_from_xy(&dim, &extent, 5.0, 0.0), Some(95));
    assert_eq!(cell_from_xy(&dim, &extent, 5.0, 10.0), Some(5));
}

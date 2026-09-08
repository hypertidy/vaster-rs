//! ESRI world file format interop.
//!
//! World files (`.tfw`, `.jgw`, `.pgw`, etc.) use a 6-parameter format
//! that differs from the GDAL geotransform in two ways:
//!
//! 1. The parameter order is different
//! 2. The origin is the **centre** of the top-left pixel (not the corner)
//!
//! ```text
//! World file:          GDAL geotransform:
//! line 1: x_res        gt[0]: x_origin (top-left corner)
//! line 2: y_skew        gt[1]: x_res
//! line 3: x_skew        gt[2]: row rotation (x_skew)
//! line 4: y_res         gt[3]: y_origin (top-left corner)
//! line 5: x_centre      gt[4]: col rotation (y_skew)
//! line 6: y_centre      gt[5]: y_res
//! ```

use crate::GeoTransform;

/// World file parameters: `[x_res, y_skew, x_skew, y_res, x_centre, y_centre]`
pub type WorldFile = [f64; 6];

/// Convert a world file to a GDAL geotransform.
///
/// The world file origin is the centre of the top-left pixel.
/// The geotransform origin is the top-left corner of the top-left pixel.
///
/// # Example
/// ```
/// use vaster::world_to_geotransform;
///
/// // 1-degree pixels, no rotation, top-left pixel centred at (0.5, 89.5)
/// let wf = [1.0, 0.0, 0.0, -1.0, 0.5, 89.5];
/// let gt = world_to_geotransform(&wf);
/// assert!((gt[0] - 0.0).abs() < 1e-10);   // corner = centre - res/2
/// assert!((gt[1] - 1.0).abs() < 1e-10);
/// assert!((gt[3] - 90.0).abs() < 1e-10);  // corner = centre - yres/2 (yres is negative)
/// assert!((gt[5] - (-1.0)).abs() < 1e-10);
/// ```
pub fn world_to_geotransform(wf: &WorldFile) -> GeoTransform {
    [
        wf[4] - wf[0] / 2.0 - wf[2] / 2.0, // x_origin = x_centre - x_res/2 - x_skew/2
        wf[0],                             // x_res
        wf[2],                             // x_skew (row rotation)
        wf[5] - wf[3] / 2.0 - wf[1] / 2.0, // y_origin = y_centre - y_res/2 - y_skew/2
        wf[1],                             // y_skew (col rotation)
        wf[3],                             // y_res
    ]
}

/// Convert a GDAL geotransform to world file parameters.
///
/// # Example
/// ```
/// use vaster::{geotransform_to_world, world_to_geotransform};
///
/// let gt = [0.0, 1.0, 0.0, 90.0, 0.0, -1.0];
/// let wf = geotransform_to_world(&gt);
/// let gt2 = world_to_geotransform(&wf);
/// for i in 0..6 {
///     assert!((gt[i] - gt2[i]).abs() < 1e-10);
/// }
/// ```
pub fn geotransform_to_world(gt: &GeoTransform) -> WorldFile {
    [
        gt[1],                             // x_res
        gt[4],                             // y_skew
        gt[2],                             // x_skew
        gt[5],                             // y_res
        gt[0] + gt[1] / 2.0 + gt[2] / 2.0, // x_centre = x_origin + x_res/2 + x_skew/2
        gt[3] + gt[5] / 2.0 + gt[4] / 2.0, // y_centre = y_origin + y_res/2 + y_skew/2
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_no_rotation() {
        let gt = [100.0, 10.0, 0.0, 500.0, 0.0, -10.0];
        let wf = geotransform_to_world(&gt);
        let gt2 = world_to_geotransform(&wf);
        for i in 0..6 {
            assert!((gt[i] - gt2[i]).abs() < 1e-10, "index {i}");
        }
    }

    #[test]
    fn roundtrip_with_rotation() {
        let gt = [100.0, 10.0, 2.0, 500.0, -1.0, -10.0];
        let wf = geotransform_to_world(&gt);
        let gt2 = world_to_geotransform(&wf);
        for i in 0..6 {
            assert!((gt[i] - gt2[i]).abs() < 1e-10, "index {i}");
        }
    }

    #[test]
    fn world_file_origin_is_pixel_centre() {
        // Non-rotated: world file x,y should be centre of top-left pixel
        let gt = [0.0, 10.0, 0.0, 100.0, 0.0, -10.0];
        let wf = geotransform_to_world(&gt);
        assert!((wf[4] - 5.0).abs() < 1e-10); // x_centre = 0 + 10/2
        assert!((wf[5] - 95.0).abs() < 1e-10); // y_centre = 100 + (-10)/2
    }
}

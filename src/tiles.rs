//! Parametric tile schemes: a quadtree of square tiles pinned to an origin.
//!
//! A [`TileScheme`] is four numbers and a CRS string:
//!
//! ```text
//!   crs      an opaque identifier ("EPSG:3857", a proj string, WKT); never
//!            interpreted here, only carried
//!   origin   [x, y] of the top-left corner of the single zoom-0 tile
//!   size0    side length of that tile in CRS units
//!   tile     tile size in pixels (square)
//! ```
//!
//! Tile `(z, x, y)` is `size0 / 2^z` on a side with its top-left corner at
//! `(origin.x + x * size, origin.y - y * size)`; y increases downward, as in
//! every web tile scheme and in GDAL's geotransform convention. Indices are
//! signed: a scheme anchored at a place of interest extends in every
//! direction, and tiles west or north of the origin have negative indices.
//!
//! Web Mercator (`EPSG:3857`, 256 px, origin at the top-left of the world)
//! is one instance. A scheme anchored at the projected centre of a study
//! area in a local equal-area projection is another, and needs no more
//! description than this struct. The scheme is the key for caches of
//! warped tiles and the thing a map client and a batch pipeline must agree
//! on; keeping it to four numbers is the point.
//!
//! [`TileScheme::tile_matrix_set_json`] writes the equivalent OGC
//! TileMatrixSet 2.0 document for consumers that speak that vocabulary.

use crate::{Extent, GeoTransform};

/// Half the Web Mercator world width in metres (EPSG:3857).
pub const WEBMERC_HALF: f64 = 20037508.342789244;

/// A quadtree of square tiles pinned to an origin. See the module docs.
#[derive(Debug, Clone, PartialEq)]
pub struct TileScheme {
    /// Coordinate reference system, carried but not interpreted.
    pub crs: String,
    /// Top-left corner of the zoom-0 tile, `[x, y]` in CRS units.
    pub origin: [f64; 2],
    /// Side length of the zoom-0 tile in CRS units.
    pub size0: f64,
    /// Tile size in pixels (square).
    pub tile: usize,
}

impl TileScheme {
    /// A scheme from its four parameters.
    pub fn new(crs: impl Into<String>, origin: [f64; 2], size0: f64, tile: usize) -> Self {
        assert!(size0 > 0.0, "size0 must be positive");
        assert!(tile > 0, "tile size must be positive");
        Self {
            crs: crs.into(),
            origin,
            size0,
            tile,
        }
    }

    /// A scheme from the zoom-0 pixel resolution rather than tile size.
    pub fn from_resolution(
        crs: impl Into<String>,
        origin: [f64; 2],
        res0: f64,
        tile: usize,
    ) -> Self {
        Self::new(crs, origin, res0 * tile as f64, tile)
    }

    /// The standard Web Mercator scheme (`EPSG:3857`, 256 px tiles).
    pub fn web_mercator() -> Self {
        Self::new(
            "EPSG:3857",
            [-WEBMERC_HALF, WEBMERC_HALF],
            2.0 * WEBMERC_HALF,
            256,
        )
    }

    /// Side length of a tile at zoom `z`, in CRS units.
    #[inline]
    pub fn tile_size(&self, z: u32) -> f64 {
        self.size0 / (1u64 << z) as f64
    }

    /// Pixel resolution at zoom `z`, in CRS units per pixel.
    #[inline]
    pub fn resolution(&self, z: u32) -> f64 {
        self.tile_size(z) / self.tile as f64
    }

    /// GDAL geotransform of tile `(z, x, y)`.
    pub fn geotransform(&self, z: u32, x: i64, y: i64) -> GeoTransform {
        let t = self.tile_size(z);
        let r = t / self.tile as f64;
        [
            self.origin[0] + x as f64 * t,
            r,
            0.0,
            self.origin[1] - y as f64 * t,
            0.0,
            -r,
        ]
    }

    /// Extent `[xmin, xmax, ymin, ymax]` of tile `(z, x, y)`.
    pub fn extent(&self, z: u32, x: i64, y: i64) -> Extent {
        let t = self.tile_size(z);
        let x0 = self.origin[0] + x as f64 * t;
        let y1 = self.origin[1] - y as f64 * t;
        [x0, x0 + t, y1 - t, y1]
    }

    /// Indices `(x, y)` of the tile at zoom `z` containing the point.
    /// Points on a shared edge belong to the tile to the right / below,
    /// matching pixel-index truncation.
    pub fn tile_at(&self, z: u32, px: f64, py: f64) -> (i64, i64) {
        let t = self.tile_size(z);
        (
            ((px - self.origin[0]) / t).floor() as i64,
            ((self.origin[1] - py) / t).floor() as i64,
        )
    }

    /// Inclusive index ranges `[xmin, xmax, ymin, ymax]` of the tiles at
    /// zoom `z` that intersect `extent`. An extent edge exactly on a tile
    /// boundary does not pull in the neighbouring tile.
    pub fn tiles_in_extent(&self, z: u32, extent: &Extent) -> [i64; 4] {
        let eps = self.resolution(z) * 1e-6;
        let (x0, y0) = self.tile_at(z, extent[0] + eps, extent[3] - eps);
        let (x1, y1) = self.tile_at(z, extent[1] - eps, extent[2] + eps);
        [x0, x1, y0, y1]
    }

    /// The tile one zoom level up that contains `(z, x, y)`.
    pub fn parent(&self, z: u32, x: i64, y: i64) -> Option<(u32, i64, i64)> {
        if z == 0 {
            return None;
        }
        Some((z - 1, x.div_euclid(2), y.div_euclid(2)))
    }

    /// The four tiles at `z + 1` that make up `(z, x, y)`, row-major.
    pub fn children(&self, z: u32, x: i64, y: i64) -> [(u32, i64, i64); 4] {
        let (z1, x1, y1) = (z + 1, x * 2, y * 2);
        [
            (z1, x1, y1),
            (z1, x1 + 1, y1),
            (z1, x1, y1 + 1),
            (z1, x1 + 1, y1 + 1),
        ]
    }

    /// Zoom level whose resolution is closest (in log2) to `res`.
    pub fn zoom_for_resolution(&self, res: f64, zmax: u32) -> u32 {
        let z = (self.resolution(0) / res).log2().round();
        z.clamp(0.0, zmax as f64) as u32
    }

    /// `"z/x/y"`, the conventional key.
    pub fn key(z: u32, x: i64, y: i64) -> String {
        format!("{z}/{x}/{y}")
    }

    /// A short, stable identifier for the scheme itself: the four parameters
    /// joined, suitable as the `scheme` part of a cache key. Not a hash;
    /// two schemes with the same parameters give the same string.
    pub fn id(&self) -> String {
        format!(
            "{}|{:.6}|{:.6}|{:.6}|{}",
            self.crs, self.origin[0], self.origin[1], self.size0, self.tile
        )
    }

    /// OGC TileMatrixSet 2.0 JSON for zoom levels `0..=zmax`, bounded to
    /// `extent` if given (otherwise each matrix is one tile wide at zoom 0
    /// and doubles per level, i.e. the region south-east of the origin).
    ///
    /// `metres_per_unit` converts CRS units to metres for the scale
    /// denominator (1.0 for metric CRSs; about 111319.49 for degrees), using
    /// the OGC 0.28 mm standard pixel.
    pub fn tile_matrix_set_json(
        &self,
        id: &str,
        zmax: u32,
        extent: Option<&Extent>,
        metres_per_unit: f64,
    ) -> String {
        let mut out = String::new();
        out.push_str("{\n");
        out.push_str(&format!("  \"id\": \"{}\",\n", esc(id)));
        out.push_str(&format!("  \"crs\": \"{}\",\n", esc(&self.crs)));
        out.push_str("  \"orderedAxes\": [\"X\", \"Y\"],\n");
        if let Some(e) = extent {
            out.push_str(&format!(
                "  \"boundingBox\": {{\"lowerLeft\": [{}, {}], \"upperRight\": [{}, {}]}},\n",
                e[0], e[2], e[1], e[3]
            ));
        }
        out.push_str("  \"tileMatrices\": [\n");
        for z in 0..=zmax {
            let res = self.resolution(z);
            let (mw, mh) = match extent {
                Some(e) => {
                    let r = self.tiles_in_extent(z, e);
                    ((r[1] - r[0] + 1).max(1), (r[3] - r[2] + 1).max(1))
                }
                None => (1i64 << z, 1i64 << z),
            };
            let corner = match extent {
                Some(e) => {
                    let r = self.tiles_in_extent(z, e);
                    let ex = self.extent(z, r[0], r[2]);
                    [ex[0], ex[3]]
                }
                None => self.origin,
            };
            out.push_str(&format!(
                "    {{\"id\": \"{z}\", \"scaleDenominator\": {}, \"cellSize\": {}, \"pointOfOrigin\": [{}, {}], \"tileWidth\": {}, \"tileHeight\": {}, \"matrixWidth\": {}, \"matrixHeight\": {}}}{}\n",
                res * metres_per_unit / 0.00028, res, corner[0], corner[1], self.tile, self.tile, mw, mh,
                if z < zmax { "," } else { "" }
            ));
        }
        out.push_str("  ]\n}\n");
        out
    }
}

fn esc(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64, tol: f64) -> bool {
        (a - b).abs() <= tol
    }

    #[test]
    fn web_mercator_matches_known_tiles() {
        let s = TileScheme::web_mercator();
        // Zoom 0 is the world.
        let e = s.extent(0, 0, 0);
        assert!(close(e[0], -WEBMERC_HALF, 1e-6) && close(e[1], WEBMERC_HALF, 1e-6));
        // Zoom 1, tile (1,1) is the south-east quadrant.
        let e = s.extent(1, 1, 1);
        assert!(
            close(e[0], 0.0, 1e-6) && close(e[3], 0.0, 1e-6) && close(e[1], WEBMERC_HALF, 1e-6)
        );
        // Standard resolution table: z0 156543.03 m/px, z10 152.87 m/px.
        assert!(close(s.resolution(0), 156543.03392804097, 1e-6));
        assert!(close(s.resolution(10), 152.8740565703525, 1e-6));
        // Hobart at z10 is tile x=931, y=647.
        let (x, y) = s.tile_at(10, 16400221.9, -5294119.4);
        assert_eq!((x, y), (931, 647));
    }

    #[test]
    fn geotransform_agrees_with_extent() {
        let s = TileScheme::new(
            "+proj=laea +lat_0=-42 +lon_0=147",
            [26_700.0, -98_000.0],
            524_288.0,
            256,
        );
        for &(z, x, y) in &[(0u32, 0i64, 0i64), (3, -2, 5), (7, 100, -33)] {
            let gt = s.geotransform(z, x, y);
            let e = s.extent(z, x, y);
            assert!(close(gt[0], e[0], 1e-9) && close(gt[3], e[3], 1e-9));
            assert!(close(gt[0] + gt[1] * 256.0, e[1], 1e-9));
            assert!(close(gt[3] + gt[5] * 256.0, e[2], 1e-9));
            assert_eq!(gt[2], 0.0);
            assert_eq!(gt[4], 0.0);
        }
    }

    #[test]
    fn anchored_scheme_has_negative_indices_and_quadtree_structure() {
        let s = TileScheme::new("EPSG:3031", [0.0, 0.0], 4_000_000.0, 512);
        // A point north-west of the anchor.
        let (x, y) = s.tile_at(2, -10.0, 10.0);
        assert_eq!((x, y), (-1, -1));
        // Parent/children round trip, including across zero.
        for &(z, x, y) in &[(2u32, -1i64, -1i64), (4, 7, -3), (5, 0, 0)] {
            let kids = s.children(z, x, y);
            for &(cz, cx, cy) in &kids {
                assert_eq!(s.parent(cz, cx, cy), Some((z, x, y)));
            }
            // Children tile the parent exactly.
            let pe = s.extent(z, x, y);
            let k0 = s.extent(kids[0].0, kids[0].1, kids[0].2);
            let k3 = s.extent(kids[3].0, kids[3].1, kids[3].2);
            assert!(close(k0[0], pe[0], 1e-9) && close(k0[3], pe[3], 1e-9));
            assert!(close(k3[1], pe[1], 1e-9) && close(k3[2], pe[2], 1e-9));
        }
        assert_eq!(s.parent(0, 0, 0), None);
    }

    #[test]
    fn tiles_in_extent_respects_boundaries() {
        let s = TileScheme::new("EPSG:3857", [0.0, 0.0], 1000.0, 10);
        // Exactly one zoom-2 tile (250 units) starting at the origin.
        assert_eq!(
            s.tiles_in_extent(2, &[0.0, 250.0, -250.0, 0.0]),
            [0, 0, 0, 0]
        );
        // A hair over the edge pulls in the neighbour.
        assert_eq!(
            s.tiles_in_extent(2, &[0.0, 250.001, -250.0, 0.0]),
            [0, 1, 0, 0]
        );
        // Straddling the origin: negative indices.
        assert_eq!(
            s.tiles_in_extent(2, &[-100.0, 100.0, -100.0, 100.0]),
            [-1, 0, -1, 0]
        );
    }

    #[test]
    fn zoom_for_resolution() {
        let s = TileScheme::web_mercator();
        assert_eq!(s.zoom_for_resolution(152.87, 19), 10);
        assert_eq!(s.zoom_for_resolution(200.0, 19), 10);
        assert_eq!(s.zoom_for_resolution(1e9, 19), 0);
        assert_eq!(s.zoom_for_resolution(0.001, 19), 19);
    }

    #[test]
    fn tms_json_is_wellformed_and_matches_scale() {
        let s = TileScheme::web_mercator();
        let j = s.tile_matrix_set_json("WebMercatorQuad", 2, None, 1.0);
        assert!(j.contains("\"crs\": \"EPSG:3857\""));
        // z0 scale denominator for Web Mercator is 559082264.03.
        assert!(j.contains("\"scaleDenominator\": 559082264.02871"), "{j}");
        assert!(j.contains("\"matrixWidth\": 4"));
        // Bounded variant: Tasmania-ish box at z8 spans a few tiles, not 256.
        let e = [16_200_000.0, 16_600_000.0, -5_400_000.0, -5_000_000.0];
        let j = s.tile_matrix_set_json("tas", 8, Some(&e), 1.0);
        assert!(j.contains("\"boundingBox\""));
        assert!(j.contains("\"id\": \"8\", \"scaleDenominator\""));
        assert!(!j.contains("\"matrixWidth\": 256"));
    }

    #[test]
    fn id_is_stable() {
        let a = TileScheme::new("EPSG:3031", [0.0, 0.0], 4_000_000.0, 512);
        let b = TileScheme::new("EPSG:3031", [0.0, 0.0], 4_000_000.0, 512);
        assert_eq!(a.id(), b.id());
        assert_ne!(a.id(), TileScheme::web_mercator().id());
    }
}

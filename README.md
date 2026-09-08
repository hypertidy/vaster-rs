# vaster

[![CI](https://github.com/hypertidy/vaster-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/hypertidy/vaster-rs/actions/workflows/ci.yml)
[![docs.rs](https://docs.rs/vaster/badge.svg)](https://docs.rs/vaster)
[![crates.io](https://img.shields.io/crates/v/vaster.svg)](https://crates.io/crates/vaster)

**Guide:** <https://hypertidy.github.io/vaster-rs/> | **API:** <https://docs.rs/vaster>

Raster grid logic, without any pesky data.

Raster grids are defined by **dimension** (ncol, nrow) and **extent** (xmin, xmax, ymin, ymax). Everything else — resolution, cell centres, geotransforms, cell indexing — derives from those six numbers. 

## Quick start

```rust
use vaster::*;

let extent = [-180.0, 180.0, -90.0, 90.0];
let dim = [360, 180];

// Build a  geotransform
let gt = extent_dim_to_gt(&extent, &dim);
assert_eq!(gt[1], 1.0); // 1 degree per pixel

// Cell centre of the top-left pixel
let (x, y) = xy_from_cell(&dim, &extent, 0);
assert!((x - (-179.5)).abs() < 1e-10);
assert!((y - 89.5).abs() < 1e-10);

// Which cell contains a point?
let cell = cell_from_xy(&dim, &extent, 145.5, -43.5);
assert!(cell.is_some());

// Snap a bounding box to grid alignment
let target = [10.3, 20.7, -5.2, 8.9];
let (aligned_ext, aligned_dim) = vcrop(&target, &dim, &extent);
// aligned_ext snaps outward to cell boundaries
```

## What's in the box

| Module | Functions | Purpose |
|---|---|---|
| `geotransform` | `extent_dim_to_gt`, `gt_to_extent`, `inv_geotransform`, `x_from_col`, `y_from_row`, `col_from_x`, `row_from_y`, `xy_from_col_row`, `col_row_from_xy`, `x_res`, `y_res` | GDAL geotransform construction, inversion, coordinate ↔ pixel mapping |
| `cell` | `ncell`, `cell_from_row_col`, `row_from_cell`, `col_from_cell`, `row_col_from_cell`, `xy_from_cell`, `cell_from_xy`, `x_centre`, `y_centre`, `x_corner`, `y_corner` | Cell index arithmetic, centre/corner coordinates |
| `crop` | `vcrop`, `crop_offset` | Snap-to-grid alignment |
| `world` | `world_to_geotransform`, `geotransform_to_world` | ESRI world file interop |
| `tiles` | `TileScheme` (`web_mercator`, `geotransform`, `extent`, `tile_at`, `tiles_in_extent`, `parent`, `children`, `zoom_for_resolution`, `tile_matrix_set_json`) | Parametric quadtree tile schemes, OGC TileMatrixSet export |

## Tile schemes

A `TileScheme` is a quadtree of square tiles pinned to an origin: a CRS
string (carried, never interpreted), the top-left corner of the zoom-0
tile, its side length in CRS units, and the tile size in pixels. Web
Mercator is one instance; a scheme anchored at the centre of a study area
in a local projection is another, and needs no more description than this.

```rust
use vaster::TileScheme;

// Standard EPSG:3857, 256 px tiles.
let wm = TileScheme::web_mercator();
let (x, y) = wm.tile_at(10, 16400221.9, -5294119.4); // Hobart
assert_eq!((x, y), (931, 647));

// A polar stereographic scheme anchored at the pole, 512 px tiles,
// 4000 km on a side at zoom 0. Indices are signed: tiles north or west
// of the origin are negative.
let s = TileScheme::new("EPSG:3031", [0.0, 0.0], 4_000_000.0, 512);
let gt = s.geotransform(3, -2, 1);           // GDAL geotransform of tile z3 x-2 y1
let range = s.tiles_in_extent(5, &[-1.5e6, 1.5e6, -1.5e6, 1.5e6]); // [xmin, xmax, ymin, ymax] indices
assert_eq!(gt[1], 4_000_000.0 / 8.0 / 512.0);
assert_eq!(range, [-12, 11, -12, 11]);

// OGC TileMatrixSet 2.0 JSON for zooms 0..=8, for consumers that speak that vocabulary.
let tms = s.tile_matrix_set_json("Antarctic4000", 8, None, 1.0);
```

## Conventions

- **Extent** is `[xmin, xmax, ymin, ymax]`
- **Dimension** is `[ncol, nrow]`
- **Cell indices** are **0-based**, traversing left-to-right then top-to-bottom
- **Geotransform** follows GDAL convention: `[origin_x, pixel_width, row_rotation, origin_y, col_rotation, pixel_height]`

## What's NOT in this crate

- No CRS / projection handling
- No I/O, no file formats, no deps
- No image data, no pixel buffers


## Relationship to the R package

This is the Rust equivalent of [hypertidy/vaster](https://github.com/hypertidy/vaster), an R package for raster grid logic. The API follows the same conventions with one difference: cell indices are **0-based** in Rust (1-based in R).

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

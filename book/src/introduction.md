# vaster

Raster grid logic, without any pesky data.

A raster grid is six numbers: a **dimension** (`ncol`, `nrow`) and an
**extent** (`xmin`, `xmax`, `ymin`, `ymax`). Resolution, cell centres,
cell corners, the GDAL geotransform, which cell a point falls in, how a
bounding box snaps to the grid: all of it derives from those six numbers
and none of it needs a pixel buffer, a file, or a coordinate reference
system.

`vaster` is that derivation, in Rust, with zero dependencies. It is the
Rust counterpart of the R package [vaster](https://github.com/hypertidy/vaster)
and follows the same conventions, so a grid described in one can be
handed to the other as six numbers.

```toml
[dependencies]
vaster = "0.2"
```

## Ten lines

```rust
# extern crate vaster;
use vaster::*;

// A 1-degree global grid: 360 columns, 180 rows.
let dim = [360, 180];
let extent = [-180.0, 180.0, -90.0, 90.0];

// Its GDAL geotransform.
let gt = extent_dim_to_gt(&extent, &dim);
assert_eq!(gt, [-180.0, 1.0, 0.0, 90.0, 0.0, -1.0]);

// The cell containing Hobart (147.3 E, 42.9 S) and that cell's centre.
let cell = cell_from_xy(&dim, &extent, 147.3, -42.9).unwrap();
assert_eq!(cell, 47847);
assert_eq!(xy_from_cell(&dim, &extent, cell), (147.5, -42.5));

// The sub-grid of that grid that covers Tasmania.
let (tas_extent, tas_dim) = vcrop(&[143.8, 148.5, -43.7, -39.5], &dim, &extent);
assert_eq!(tas_extent, [143.0, 149.0, -44.0, -39.0]);
assert_eq!(tas_dim, [6, 5]);
```

## What is here

| Module | Purpose |
|---|---|
| `geotransform` | Build, invert and apply GDAL geotransforms; pixel to coordinate and back |
| `cell` | Cell index arithmetic; centre and corner coordinates |
| `crop` | Snap a bounding box to a parent grid; offsets of a sub-grid |
| `world` | ESRI world file (`.tfw`) interop |
| `tiles` | Parametric quadtree tile schemes; OGC TileMatrixSet export |

Everything is re-exported at the crate root, so `use vaster::*` gets all
of it.

## What is deliberately not here

- **No CRS.** A `TileScheme` carries a CRS string but never interprets
  it. Projection is somebody else's job (PROJ, via whatever binding you
  already use).
- **No I/O.** Nothing reads or writes a file. The geotransform you get
  from GDAL, rasterio, or a `.tfw` is the input; what you do with the
  answer is up to you.
- **No pixels.** There is no image type. This crate tells you *which*
  pixels and *where* they are, not what is in them.

That boundary is the point. The functions here are the arithmetic that
every raster library reimplements, usually slightly differently at the
edges. Having them in one place with the edge cases pinned down by tests
means a warp pipeline, a tile cache and a map client can agree on a grid
by exchanging six numbers.

## Reading this book

The [Conventions](conventions.md) page is the one to read first and the
one to link to; it states the orderings, the origin, and which edges are
inclusive. The guide pages after it each cover one module with worked
examples on real grids. Every Rust block in this book is compiled and run
against the crate by `mdbook test` in CI, so the examples are correct for
the version they document.

The API reference is on [docs.rs](https://docs.rs/vaster); a copy built
alongside this book is at [/api](api/vaster/index.html).

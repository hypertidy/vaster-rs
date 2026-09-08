# From R vaster

This crate is the Rust counterpart of the R package
[vaster](https://github.com/hypertidy/vaster). The conventions are the
same: extent is `c(xmin, xmax, ymin, ymax)`, dimension is `c(ncol, nrow)`,
cells run left-to-right then top-to-bottom, functions are named
`<get>_from_<have>` with the grid arguments first.

There is one difference, and it is the one that bites:

> **Cells are 1-based in R and 0-based in Rust.** R cell `n` is Rust
> cell `n - 1`. Column and row indices likewise.

Everything below is stated in each language's own convention.

## Function table

| R vaster | Rust vaster | Notes |
|---|---|---|
| `n_cell(dimension)` | `ncell(&dim)` | |
| `n_col`, `n_row` | `dim[0]`, `dim[1]` | Plain indexing |
| `x_res(dimension, extent)` | `x_res(&dim, &extent)` | |
| `y_res(dimension, extent)` | `y_res(&dim, &extent)` | Both always positive |
| `x_min`, `x_max`, `y_min`, `y_max` | `extent[0..4]` | Plain indexing |
| `cell_from_row_col(dimension, row, col)` | `cell_from_row_col(&dim, row, col)` | R 1-based; Rust 0-based, `Option` |
| `row_from_cell(dimension, cell)` | `row_from_cell(&dim, cell)` | |
| `col_from_cell(dimension, cell)` | `col_from_cell(&dim, cell)` | |
| `rowcol_from_cell(dimension, cell)` | `row_col_from_cell(&dim, cell)` | Returns `(row, col)` |
| `xy_from_cell(dimension, extent, cell)` | `xy_from_cell(&dim, &extent, cell)` | Cell centre |
| `x_from_cell`, `y_from_cell` | `xy_from_cell(...).0`, `.1` | |
| `cell_from_xy(dimension, extent, xy)` | `cell_from_xy(&dim, &extent, x, y)` | R vectorised over rows of `xy`; Rust one point, `Option` |
| `x_centre(dimension, extent)` | `x_centre(&dim, &extent)` | `Vec<f64>` |
| `y_centre(dimension, extent)` | `y_centre(&dim, &extent)` | Top to bottom in both |
| `x_corner`, `y_corner` | `x_corner`, `y_corner` | Length `n + 1` |
| `extent_dim_to_gt(extent, dimension)` | `extent_dim_to_gt(&extent, &dim)` | Same argument order |
| `gt_dim_to_extent(gt, dimension)` | `gt_to_extent(&gt, &dim)` | |
| `x_from_col(gt, col)`, `y_from_row(gt, row)` | same | Centre-based |
| `col_from_x(gt, x)`, `row_from_y(gt, y)` | same | Fractional, centre-based |
| `world_to_geotransform`, `geotransform_to_world` | same | |
| `vcrop(x, dimension, extent)` | `vcrop(&target, &dim, &extent)` | Returns `(extent, dim)` |
| `snap_extent`, `align_extent` | `vcrop(...).0` | |
| `intersect_extent`, `buffer_extent` | not provided | Trivial array arithmetic |
| `cell_from_extent`, `extent_from_cell` | not provided | Compose `vcrop` + `cell_from_row_col` |
| `vaster_long`, `vaster_listxyz` | not provided | Data-frame shapes; see below |
| `plot_extent`, `draw_extent` | not provided | Graphics |
| `raster_sfio`, `rasterio0`, `rasterio_idx` | not provided | GDAL RasterIO window helpers; use `crop_offset` |
| `gdal_te`, `gdal_ts`, `ts_te` | not provided | String formatting of extent and dim for the GDAL CLI |
| `fit_dims` | not provided | |
| (none) | `TileScheme` | New in 0.2.0; not in R vaster |

## The same call in both

R:

```r
library(vaster)
dimension <- c(360, 180)
extent <- c(-180, 180, -90, 90)

cell_from_xy(dimension, extent, cbind(147.3, -42.9))
#> [1] 47848
xy_from_cell(dimension, extent, 47848)
#>          x     y
#> [1,] 147.5 -42.5
vcrop(c(143.8, 148.5, -43.7, -39.5), dimension, extent)
#> $extent
#> [1] 143 149 -44 -39
#> $dimension
#> [1] 6 5
```

Rust:

```rust
# extern crate vaster;
# use vaster::*;
let dim = [360, 180];
let extent = [-180.0, 180.0, -90.0, 90.0];

assert_eq!(cell_from_xy(&dim, &extent, 147.3, -42.9), Some(47847)); // 47848 - 1
assert_eq!(xy_from_cell(&dim, &extent, 47847), (147.5, -42.5));
assert_eq!(
    vcrop(&[143.8, 148.5, -43.7, -39.5], &dim, &extent),
    ([143.0, 149.0, -44.0, -39.0], [6, 5])
);
```

## Vectorisation

R functions take vectors of cells or matrices of points and return
vectors. The Rust functions take one cell or one point. Map over a
slice:

```rust
# extern crate vaster;
# use vaster::*;
let dim = [360, 180];
let extent = [-180.0, 180.0, -90.0, 90.0];
let cells = [0usize, 359, 47847, 64799];

let xy: Vec<(f64, f64)> = cells.iter().map(|&c| xy_from_cell(&dim, &extent, c)).collect();
assert_eq!(xy[2], (147.5, -42.5));
```

`vaster_long` (a data frame of `cell, x, y`) is the same map with the
cell index kept.

## Cross-validation

The crate's `tests/against_r.rs` holds values computed with the R
package and asserts the Rust functions reproduce them, adjusting for the
index base. If you find a case where the two disagree, that is a bug in
one of them; please report it with the six numbers and the call.

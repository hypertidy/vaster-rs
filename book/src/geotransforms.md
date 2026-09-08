# Geotransforms

A geotransform is the six-number affine map from pixel space to map
space that GDAL, rasterio and every GeoTIFF carry. The crate builds one
from an extent and dimension, recovers the extent from one, inverts it,
and applies it in both directions.

## From extent and dimension

```rust
# extern crate vaster;
# use vaster::*;
// A Sentinel-2 10 m tile in UTM zone 55 south: 10980 x 10980 pixels,
// 109.8 km on a side.
let extent = [499_980.0, 609_780.0, 5_190_220.0, 5_300_020.0];
let dim = [10980, 10980];

let gt = extent_dim_to_gt(&extent, &dim);
assert_eq!(gt, [499_980.0, 10.0, 0.0, 5_300_020.0, 0.0, -10.0]);
```

Note `gt[3]` is `ymax`, the top of the image, and `gt[5]` is negative.
`extent_dim_to_gt` always produces a north-up transform; rotated grids
come from elsewhere (a GeoTIFF with a model transformation, say) and
are consumed by the functions below, but never constructed here.

## Back to an extent

`gt_to_extent` needs the dimension too, because a geotransform alone
only pins the top-left corner and the pixel size:

```rust
# extern crate vaster;
# use vaster::*;
let gt = [499_980.0, 10.0, 0.0, 5_300_020.0, 0.0, -10.0];
let extent = gt_to_extent(&gt, &[10980, 10980]);
assert_eq!(extent, [499_980.0, 609_780.0, 5_190_220.0, 5_300_020.0]);
```

## Pixel to coordinate

`x_from_col` and `y_from_row` return the **centre** of the requested
column or row. They accept fractional indices, so `x_from_col(&gt, -0.5)`
is the left edge of the grid and `x_from_col(&gt, 0.0)` is the centre of
the first column.

```rust
# extern crate vaster;
# use vaster::*;
let gt = extent_dim_to_gt(&[-180.0, 180.0, -90.0, 90.0], &[360, 180]);

assert_eq!(x_from_col(&gt, 0.0), -179.5);   // centre of column 0
assert_eq!(x_from_col(&gt, -0.5), -180.0);  // left edge
assert_eq!(y_from_row(&gt, 0.0), 89.5);     // centre of row 0
assert_eq!(y_from_row(&gt, 179.0), -89.5);  // centre of the last row

// Both at once, including for rotated transforms.
assert_eq!(xy_from_col_row(&gt, 327.0, 132.0), (147.5, -42.5));
```

This differs from the raw GDAL formula, where `pixel = 0` is the corner.
The choice matches the R package and most raster-analysis code, where
"the coordinate of column 3" means the centre. If you need the GDAL
corner convention, offset by `-0.5`.

## Coordinate to pixel

`col_from_x` and `row_from_y` are the exact inverses of the above: they
return a *fractional, centre-based* index, so a point at the centre of
column 3 gives `3.0` and a point at the left edge of column 3 gives
`2.5`.

To turn that into an integer cell index, add a half and floor. Flooring
the raw value is wrong for the left half of every cell.

```rust
# extern crate vaster;
# use vaster::*;
let gt = extent_dim_to_gt(&[0.0, 10.0, 0.0, 5.0], &[10, 5]);

// A point in the left half of column 3.
let c = col_from_x(&gt, 3.2);
assert!((c - 2.7).abs() < 1e-12);
assert_eq!((c + 0.5).floor() as i64, 3);  // correct
assert_eq!(c.floor() as i64, 2);          // wrong

// Round trip through the centre.
let x = x_from_col(&gt, 3.0);
assert!((col_from_x(&gt, x) - 3.0).abs() < 1e-12);
```

For a point you already have as `(x, y)` and a grid you have as extent
and dimension, `cell_from_xy` on the [Cells](cells.md) page does all of
this, with bounds checking, in one call.

## Rotated grids

`col_from_x` and `row_from_y` assume `gt[2] == gt[4] == 0`. For a
rotated or sheared transform use the full inverse:

```rust
# extern crate vaster;
# use vaster::*;
// A grid rotated a few degrees: non-zero gt[2] and gt[4].
let gt = [100.0, 10.0, 2.0, 500.0, -1.0, -10.0];

let (x, y) = xy_from_col_row(&gt, 42.0, 17.0);
let (col, row) = col_row_from_xy(&gt, x, y).unwrap();
assert!((col - 42.0).abs() < 1e-9);
assert!((row - 17.0).abs() < 1e-9);

// The inverse transform itself, in the same six-number layout,
// maps map-space to pixel-space (corner convention, as in GDAL).
let inv = inv_geotransform(&gt).unwrap();
let pixel = inv[0] + x * inv[1] + y * inv[2];
assert!((pixel - 42.5).abs() < 1e-9);

// A degenerate transform has no inverse.
assert_eq!(inv_geotransform(&[0.0, 0.0, 0.0, 0.0, 0.0, 0.0]), None);
```

`inv_geotransform` is `GDALInvGeoTransform`, including its singularity
threshold, so results agree with GDAL to floating-point precision.

## World files

An ESRI world file (`.tfw`, `.jgw`, `.pgw`) holds the same six numbers
in a different order and with the origin at the **centre** of the
top-left pixel rather than its corner. `world_to_geotransform` and
`geotransform_to_world` convert exactly, rotation included:

```rust
# extern crate vaster;
# use vaster::*;
// Lines of a .tfw for the 1-degree world grid:
//   1.0      x res
//   0.0      y skew
//   0.0      x skew
//   -1.0     y res
//   -179.5   x of the centre of the top-left pixel
//   89.5     y of the centre of the top-left pixel
let wf = [1.0, 0.0, 0.0, -1.0, -179.5, 89.5];
let gt = world_to_geotransform(&wf);
assert_eq!(gt, [-180.0, 1.0, 0.0, 90.0, 0.0, -1.0]);
assert_eq!(geotransform_to_world(&gt), wf);
```

## What GDAL calls these

| vaster | GDAL |
|---|---|
| `extent_dim_to_gt` | what `gdal_translate -a_ullr` computes |
| `gt_to_extent` | `GDALGetGeoTransform` + `GDALGetRasterXSize/YSize` |
| `inv_geotransform` | `GDALInvGeoTransform` |
| `xy_from_col_row` (minus 0.5) | `GDALApplyGeoTransform` |
| `world_to_geotransform` | what the `WORLDFILE` sidecar reader does |

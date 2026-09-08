# Conventions

This page is the contract. Everything else in the crate follows from it.

## The six numbers

```text
dimension   [ncol, nrow]               usize
extent      [xmin, xmax, ymin, ymax]   f64
```

Extent is a closed box in map units. Dimension is the number of cells in
each direction. Resolution is derived, never stored:

```rust
# extern crate vaster;
# use vaster::*;
let dim = [720, 720];
let extent = [-9_000_000.0, 9_000_000.0, -9_000_000.0, 9_000_000.0]; // EASE-Grid 2.0 South, 25 km
assert_eq!(x_res(&dim, &extent), 25_000.0);
assert_eq!(y_res(&dim, &extent), 25_000.0); // always positive
```

The order `[xmin, xmax, ymin, ymax]` is the R `raster`/`terra`/`vaster`
order, not the `[xmin, ymin, xmax, ymax]` of GDAL `-te`, GeoJSON bbox
and most of the Python world. When you cross that boundary, reorder.

## Cells

Cells are numbered from **0**, starting at the **top-left**, running
left-to-right along a row and then down to the next row (row-major):

```text
        col 0  col 1  col 2  col 3  col 4
row 0     0      1      2      3      4
row 1     5      6      7      8      9
row 2    10     11     12     13     14
```

so `cell = row * ncol + col`. This is GDAL's pixel/line order. The R
package numbers the same cells from 1.

```rust
# extern crate vaster;
# use vaster::*;
let dim = [5, 3];
assert_eq!(cell_from_row_col(&dim, 1, 2), Some(7));
assert_eq!(row_col_from_cell(&dim, 7), (1, 2));
assert_eq!(cell_from_row_col(&dim, 3, 0), None); // row 3 does not exist
assert_eq!(ncell(&dim), 15);
```

## Geotransform

The GDAL six-parameter affine transform, in GDAL's order:

```text
gt[0]  x coordinate of the top-left corner of the top-left pixel
gt[1]  pixel width (x resolution)
gt[2]  row rotation (0 for north-up)
gt[3]  y coordinate of the top-left corner of the top-left pixel
gt[4]  column rotation (0 for north-up)
gt[5]  pixel height, NEGATIVE for north-up
```

It maps pixel-space `(pixel, line)` to map-space `(x, y)`:

```text
x = gt[0] + pixel * gt[1] + line * gt[2]
y = gt[3] + pixel * gt[4] + line * gt[5]
```

with `(0, 0)` at the outer corner of the top-left pixel and `(0.5, 0.5)`
at its centre. The crate's `x_from_col` and `y_from_row` return the
**centre** of the requested column or row, so `x_from_col(&gt, 0.0)` is
`gt[0] + gt[1] / 2`, not `gt[0]`. See [Geotransforms](geotransforms.md).

## Which edges are inclusive

This is where raster libraries quietly disagree, so the rule here is
stated and tested.

**Point-in-cell (`cell_from_xy`)**: all four edges of the extent are
inclusive. A point on the shared boundary between two cells belongs to
the cell to the right or below (index truncation); a point exactly on
`xmax` or `ymin` belongs to the last column or row rather than falling
outside. This matches R `vaster` and `terra`.

```rust
# extern crate vaster;
# use vaster::*;
let dim = [10, 10];
let extent = [0.0, 10.0, 0.0, 10.0];
assert_eq!(cell_from_xy(&dim, &extent, 1.0, 9.5), Some(1));   // boundary goes right
assert_eq!(cell_from_xy(&dim, &extent, 10.0, 5.0), Some(59)); // xmax is inside
assert_eq!(cell_from_xy(&dim, &extent, 5.0, 0.0), Some(95));  // ymin is inside
assert_eq!(cell_from_xy(&dim, &extent, 10.1, 5.0), None);
```

**Snapping (`vcrop`)**: the aligned extent is the smallest set of whole
cells that contains the target. A target edge already on a cell boundary
stays there; it does not pull in the neighbour.

**Tiles (`tiles_in_extent`)**: same rule. An extent edge exactly on a
tile boundary does not include the tile on the other side of it.

## Naming

Function names read as `<what you get>_from_<what you have>`:
`xy_from_cell`, `cell_from_xy`, `col_from_x`, `row_from_y`. Arguments
that describe a grid come first, in the order `dim, extent` or `gt`, and
the thing being converted comes last. This is the R package's naming,
kept so the two APIs line up in a table (see [From R vaster](from-r.md)).

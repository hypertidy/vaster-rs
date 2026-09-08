# Cells and coordinates

Everything on this page takes `(dim, extent)`, builds the geotransform
internally, and never asks you to hold one.

## Which cell is a point in

```rust
# extern crate vaster;
# use vaster::*;
// GHRSST MUR: a 0.01-degree global grid, 36000 x 17999 cells.
let dim = [36000, 17999];
let extent = [-180.0, 180.0, -89.995, 89.995];

// Macquarie Island, 158.937 E 54.624 S.
let cell = cell_from_xy(&dim, &extent, 158.937, -54.624).unwrap();
let (row, col) = row_col_from_cell(&dim, cell);
assert_eq!((row, col), (14461, 33893));

// Anything outside the extent is None, not a clamped guess.
assert_eq!(cell_from_xy(&dim, &extent, 200.0, 0.0), None);
```

The edge rules (all four edges inclusive; a shared boundary belongs to
the cell to the right or below) are on the [Conventions](conventions.md)
page.

## Where a cell is

`xy_from_cell` returns the cell **centre**:

```rust
# extern crate vaster;
# use vaster::*;
let dim = [360, 180];
let extent = [-180.0, 180.0, -90.0, 90.0];

assert_eq!(xy_from_cell(&dim, &extent, 0), (-179.5, 89.5));      // top-left
assert_eq!(xy_from_cell(&dim, &extent, 64799), (179.5, -89.5));  // bottom-right

// cell -> xy -> cell is the identity.
for cell in [0, 359, 360, 47847, 64799] {
    let (x, y) = xy_from_cell(&dim, &extent, cell);
    assert_eq!(cell_from_xy(&dim, &extent, x, y), Some(cell));
}
```

## Axis vectors

For plotting, for building a NetCDF coordinate variable, or for
comparing two grids, the centre and corner coordinates along each axis:

```rust
# extern crate vaster;
# use vaster::*;
let dim = [4, 3];
let extent = [0.0, 4.0, 0.0, 3.0];

assert_eq!(x_centre(&dim, &extent), vec![0.5, 1.5, 2.5, 3.5]);
assert_eq!(y_centre(&dim, &extent), vec![2.5, 1.5, 0.5]);   // top to bottom
assert_eq!(x_corner(&dim, &extent), vec![0.0, 1.0, 2.0, 3.0, 4.0]);
assert_eq!(y_corner(&dim, &extent), vec![3.0, 2.0, 1.0, 0.0]);
```

Centre vectors have `ncol` and `nrow` elements; corner vectors have one
more. `y_*` run from the top of the grid down, in row order, which is
the order the cells are numbered in and the order the pixels sit in a
GeoTIFF. A NetCDF file whose latitude increases with index is the same
grid with its rows flipped, and that flip is your job, not the crate's.

## Index arithmetic without coordinates

When you only need to move around inside the grid:

```rust
# extern crate vaster;
# use vaster::*;
let dim = [5, 3];
assert_eq!(ncell(&dim), 15);
assert_eq!(row_from_cell(&dim, 7), 1);
assert_eq!(col_from_cell(&dim, 7), 2);
assert_eq!(cell_from_row_col(&dim, 2, 4), Some(14));
assert_eq!(cell_from_row_col(&dim, 2, 5), None);
```

`row_from_cell` and `col_from_cell` do not bounds-check the cell
against `nrow` (they cannot, cheaply, without recomputing `ncell`), so a
cell index past the end gives a row past the end. `cell_from_row_col`
does check, and that is the one to use when the input is untrusted.

## A note on `usize`

Dimensions and cell indices are `usize`. On 64-bit targets that is
plenty; a 0.01-degree global grid has 648 million cells and a 10 m
global grid would have about 5 x 10^12, both well inside range. On a
32-bit target (wasm32, say) `ncell` overflows past 4.29 billion cells,
which is a 65 000-square grid. If that matters for you, check
`dim[0].checked_mul(dim[1])` before trusting the index.

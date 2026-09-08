# Aligning and cropping

The recurring job in any raster pipeline: you have a grid, you have a
box of interest, and you need the sub-grid of the first that covers the
second, with edges on the parent's cell boundaries so that reading it is
a plain window and not a resample.

## `vcrop`

```rust
# extern crate vaster;
# use vaster::*;
// Parent: NSIDC sea-ice polar stereographic south, 25 km (EPSG:3976).
let parent_dim = [316, 332];
let parent_extent = [-3_950_000.0, 3_950_000.0, -3_950_000.0, 4_350_000.0];

// Target: a box around the Ross Sea, not on any cell boundary.
let target = [-500_000.0, 700_000.0, -2_600_000.0, -1_300_000.0];

let (ext, dim) = vcrop(&target, &parent_dim, &parent_extent);

// Snapped outward to whole cells of the parent.
assert_eq!(ext, [-500_000.0, 700_000.0, -2_600_000.0, -1_300_000.0]);
assert_eq!(dim, [48, 52]);
```

That target happened to land on cell edges (25 km multiples from an
origin at a 25 km multiple), so it came back unchanged. One that does
not:

```rust
# extern crate vaster;
# use vaster::*;
let parent_dim = [360, 180];
let parent_extent = [-180.0, 180.0, -90.0, 90.0];

let target = [10.3, 20.7, -5.2, 8.9];
let (ext, dim) = vcrop(&target, &parent_dim, &parent_extent);
assert_eq!(ext, [10.0, 21.0, -6.0, 9.0]);
assert_eq!(dim, [11, 15]);
```

Two rules:

- **Snap out, never in.** The result contains the target. A target edge
  already on a boundary stays; it is not pushed a cell further.
- **Clamp to the parent.** A target that overhangs the parent is cut to
  the parent's extent. A target wholly outside gives a zero-sized
  dimension in that direction; check for it if the target is untrusted.

```rust
# extern crate vaster;
# use vaster::*;
let (ext, dim) = vcrop(&[-200.0, 200.0, -100.0, 100.0], &[360, 180], &[-180.0, 180.0, -90.0, 90.0]);
assert_eq!(ext, [-180.0, 180.0, -90.0, 90.0]);
assert_eq!(dim, [360, 180]);
```

## `crop_offset`

Once you have the aligned extent, `crop_offset` gives the column and row
of its top-left cell in the parent, which is exactly the `xoff, yoff` of
a GDAL `-srcwin` or a rasterio `Window`:

```rust
# extern crate vaster;
# use vaster::*;
let parent_dim = [360, 180];
let parent_extent = [-180.0, 180.0, -90.0, 90.0];
let (ext, dim) = vcrop(&[10.3, 20.7, -5.2, 8.9], &parent_dim, &parent_extent);

let (xoff, yoff) = crop_offset(&ext, &parent_dim, &parent_extent);
assert_eq!((xoff, yoff), (190, 81));

// gdal_translate -srcwin 190 81 11 15 in.tif out.tif
let srcwin = [xoff, yoff, dim[0], dim[1]];
assert_eq!(srcwin, [190, 81, 11, 15]);
```

Rows count from the top, so `yoff` is measured down from `ymax`.

## The pattern

`vcrop` then `crop_offset` is the whole of "read the part of this file
that covers that box":

1. Get the file's extent and dimension (from its geotransform and size,
   via `gt_to_extent`).
2. `vcrop` the box of interest against it.
3. `crop_offset` for the window origin; the aligned dimension is the
   window size.
4. Read that window. The aligned extent and dimension are the new grid's
   six numbers; `extent_dim_to_gt` gives the geotransform to write back
   out.

No resampling happened, because every edge is on a parent cell boundary.
When the box came from another grid at a different resolution or
projection, that is the moment to warp, and the aligned grid from step 3
is the target grid to warp *to*.

## What it is in R

`vcrop(target, dimension, extent)` is R `vaster::vcrop()` with the same
argument order and the same snap-out rule; `crop_offset` corresponds to
subtracting 1 from what the R package reports as the 1-based window
origin.

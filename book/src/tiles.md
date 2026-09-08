# Tile schemes

A tile scheme is a quadtree of square tiles pinned to an origin. Web
Mercator's `z/x/y` is the one everybody knows, but there is nothing
special about it: any projection, any origin, any zoom-0 size makes a
scheme, and a scheme is the thing a map client, a warp pipeline and a
tile cache must agree on.

`TileScheme` describes one with four numbers and a string:

```text
crs      an identifier ("EPSG:3857", a PROJ string, WKT); carried, never interpreted
origin   [x, y] of the top-left corner of the single zoom-0 tile
size0    side length of that tile in CRS units
tile     tile size in pixels (square)
```

Tile `(z, x, y)` is `size0 / 2^z` on a side, with its top-left corner at
`(origin.x + x * size, origin.y - y * size)`. `y` increases downward, as
in every web tile scheme and in the GDAL geotransform. Indices are
**signed**: a scheme anchored at a place of interest extends in every
direction, and tiles west or north of the origin have negative indices.

## Web Mercator

```rust
# extern crate vaster;
# use vaster::*;
let wm = TileScheme::web_mercator();
assert_eq!(wm.crs, "EPSG:3857");
assert_eq!(wm.tile, 256);

// Zoom 0 is one tile covering the world.
assert_eq!(wm.extent(0, 0, 0), [-WEBMERC_HALF, WEBMERC_HALF, -WEBMERC_HALF, WEBMERC_HALF]);

// The familiar resolution table.
assert!((wm.resolution(0) - 156_543.033_928_04).abs() < 1e-6);
assert!((wm.resolution(10) - 152.874_056_57).abs() < 1e-6);

// Hobart in EPSG:3857 is at about (16400222, -5294119); at z10 that is tile 931/647.
assert_eq!(wm.tile_at(10, 16_400_221.9, -5_294_119.4), (931, 647));
assert_eq!(TileScheme::key(10, 931, 647), "10/931/647");
```

## A scheme of your own

The reason the struct exists. Anchor a scheme at the pole in Antarctic
polar stereographic, 4000 km on a side at zoom 0, 512-pixel tiles:

```rust
# extern crate vaster;
# use vaster::*;
let s = TileScheme::new("EPSG:3031", [0.0, 0.0], 4_000_000.0, 512);

// Resolution halves per zoom: 7812.5 m/px at z0, 61.04 m/px at z7.
assert_eq!(s.resolution(0), 7_812.5);
assert_eq!(s.resolution(7), 61.03515625);

// The origin is the pole, so the four zoom-1 tiles are the four quadrants
// and two of them have a negative index.
assert_eq!(s.tile_at(1, -10.0, 10.0), (-1, -1));   // north-west of the pole
assert_eq!(s.tile_at(1, 10.0, -10.0), (0, 0));     // south-east

// Every tile has a GDAL geotransform: the target grid for a warp.
let gt = s.geotransform(3, -2, 1);
assert_eq!(gt, [-1_000_000.0, 976.5625, 0.0, -500_000.0, 0.0, -976.5625]);

// And the same information as an extent.
assert_eq!(s.extent(3, -2, 1), [-1_000_000.0, -500_000.0, -1_000_000.0, -500_000.0]);
```

Or, if you think in resolutions rather than tile sizes, give the zoom-0
pixel size and let the tile size follow:

```rust
# extern crate vaster;
# use vaster::*;
let a = TileScheme::from_resolution("EPSG:3031", [0.0, 0.0], 7_812.5, 512);
let b = TileScheme::new("EPSG:3031", [0.0, 0.0], 4_000_000.0, 512);
assert_eq!(a, b);
```

## Which tiles cover a box

`tiles_in_extent` returns inclusive index ranges `[xmin, xmax, ymin,
ymax]` at a zoom. An extent edge exactly on a tile boundary does not
pull in the neighbouring tile.

```rust
# extern crate vaster;
# use vaster::*;
let s = TileScheme::new("EPSG:3031", [0.0, 0.0], 4_000_000.0, 512);

// A 3000 km box centred on the pole, at zoom 5 (125 km tiles).
let r = s.tiles_in_extent(5, &[-1.5e6, 1.5e6, -1.5e6, 1.5e6]);
assert_eq!(r, [-12, 11, -12, 11]);

// Enumerate them.
let mut n = 0;
for y in r[2]..=r[3] {
    for x in r[0]..=r[1] {
        let _key = TileScheme::key(5, x, y);
        n += 1;
    }
}
assert_eq!(n, 24 * 24);
```

## Moving through the tree

```rust
# extern crate vaster;
# use vaster::*;
let s = TileScheme::web_mercator();

assert_eq!(s.parent(10, 931, 647), Some((9, 465, 323)));
assert_eq!(s.parent(0, 0, 0), None);

let kids = s.children(9, 465, 323);
assert_eq!(kids, [(10, 930, 646), (10, 931, 646), (10, 930, 647), (10, 931, 647)]);

// parent/children hold across zero for anchored schemes.
let a = TileScheme::new("EPSG:3031", [0.0, 0.0], 4_000_000.0, 512);
assert_eq!(a.parent(3, -1, -1), Some((2, -1, -1)));
assert_eq!(a.parent(3, -2, -2), Some((2, -1, -1)));
```

`parent` uses Euclidean division, so `-1 / 2` is `-1`, not `0`, and the
four children of `(2, -1, -1)` are `(3, -2..=-1, -2..=-1)`.

## Picking a zoom

Given the resolution you want (the source's native resolution, say, or
the screen's), the zoom whose resolution is nearest in log2:

```rust
# extern crate vaster;
# use vaster::*;
let wm = TileScheme::web_mercator();
assert_eq!(wm.zoom_for_resolution(152.87, 19), 10);   // exact
assert_eq!(wm.zoom_for_resolution(200.0, 19), 10);    // nearer z10 than z9
assert_eq!(wm.zoom_for_resolution(1e9, 19), 0);       // clamped
assert_eq!(wm.zoom_for_resolution(0.001, 19), 19);    // clamped

// 250 m MODIS data into the polar scheme above: z5 (244 m/px).
let s = TileScheme::new("EPSG:3031", [0.0, 0.0], 4_000_000.0, 512);
assert_eq!(s.zoom_for_resolution(250.0, 12), 5);
```

## As a cache key

`id()` joins the parameters into a string that is the same for two
schemes with the same parameters. It is not a hash; floats are written
to six decimal places, which is intentional (a cache key should not be
sensitive to the last bits of an origin).

```rust
# extern crate vaster;
# use vaster::*;
let s = TileScheme::new("EPSG:3031", [0.0, 0.0], 4_000_000.0, 512);
assert_eq!(s.id(), "EPSG:3031|0.000000|0.000000|4000000.000000|512");
```

Cache path: `<scheme id>/<z>/<x>/<y>`.

## OGC TileMatrixSet

For consumers that speak the OGC vocabulary (a tile server, a STAC
extension, QGIS), `tile_matrix_set_json` writes a TileMatrixSet 2.0
document for zooms `0..=zmax`:

```rust
# extern crate vaster;
# use vaster::*;
let s = TileScheme::new("EPSG:3031", [0.0, 0.0], 4_000_000.0, 512);

// Bounded to a region, so each matrix is only as wide as it needs to be.
let region = [-3.0e6, 3.0e6, -3.0e6, 3.0e6];
let json = s.tile_matrix_set_json("AntarcticPolar4000", 8, Some(&region), 1.0);

assert!(json.contains("\"crs\": \"EPSG:3031\""));
assert!(json.contains("\"boundingBox\""));
assert!(json.contains("\"id\": \"8\""));
```

The last argument is metres per CRS unit, used only for the OGC
`scaleDenominator` (0.28 mm standard pixel): `1.0` for a metric CRS,
about `111319.49` for a degree-based one. Pass `None` for the extent to
get the unbounded form, where zoom `z` is `2^z` tiles wide starting at
the origin.

The JSON is written by hand rather than through a serializer, which is
how the crate stays dependency-free; the document shape is fixed and
small.

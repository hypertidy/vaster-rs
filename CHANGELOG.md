# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
Before 1.0, a breaking change bumps the minor version.

## [Unreleased]

## [0.2.0] - 2026-09-08

### Added

- `TileScheme`: parametric quadtree tile schemes described by a CRS string,
  an origin, the zoom-0 tile size and the tile size in pixels. Derives tile
  geotransforms and extents, point containment (`tile_at`), extent coverage
  (`tiles_in_extent`), `parent`/`children` navigation with signed indices,
  `zoom_for_resolution`, a stable `id()` for cache keys, and an OGC
  TileMatrixSet 2.0 JSON document (`tile_matrix_set_json`).
- `TileScheme::web_mercator()` and the `WEBMERC_HALF` constant.
- `row_col_from_cell` returning `(row, col)` in one call.

### Fixed

- `cell_from_xy` now includes all four extent edges, matching R vaster and
  terra: points exactly on `xmax` or `ymin` map to the last column or row
  rather than falling outside the grid.

### Changed

- Declared a minimum supported Rust version of 1.70.
- CI workflow moved to `.github/workflows/` so it actually runs.

## [0.1.0] - 2025

### Added

- Initial release: geotransform construction and inversion, cell index
  arithmetic, cell centre and corner coordinates, `vcrop` grid alignment,
  ESRI world file interop. Zero dependencies.

[Unreleased]: https://github.com/hypertidy/vaster-rs/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/hypertidy/vaster-rs/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/hypertidy/vaster-rs/releases/tag/v0.1.0

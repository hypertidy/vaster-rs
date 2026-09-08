# Contributing

Thanks for looking. The crate is small on purpose and these notes are
short for the same reason.

## Scope

- Grid arithmetic only: geotransforms, cells, extents, alignment, tile
  schemes. No CRS interpretation, no I/O, no pixel buffers.
- Zero runtime dependencies. Dev-dependencies are fine. An optional
  feature that pulls a dependency (a `serde` derive, say) is acceptable
  only if it is off by default.
- Semantics follow the R `vaster` package and GDAL. Where the two
  differ, GDAL wins and the difference is documented.

## Code conventions

- Out-of-range input returns `Option`. Constructors may panic on
  parameters that make no sense (`TileScheme::new` with a zero tile
  size). A single function does not do both.
- Extent, dimension and geotransform are plain arrays. Keep them so
  unless a real bug motivates a newtype.
- Every function whose result depends on which edge is inclusive says
  so in its doc comment.
- Every `pub fn` has a doctest with a numeric assertion. Prefer real
  grids (a 1-degree world grid, a Sentinel-2 tile, a polar 25 km grid)
  over toy ones; the numbers are the documentation.
- ASCII only in source, docs and the book. The crate is consumed as
  text by tools that are not always UTF-8 clean.
- `cargo fmt` and `cargo clippy -- -D warnings` are clean.

## Tests

- Unit tests live beside the code in `src/`. Behaviour checked against
  the R package lives in `tests/against_r.rs`, with the R call that
  produced each expected value in a comment.
- The book under `book/` is also a test: `mdbook test book` compiles and
  runs every Rust block. If you change an API, the book will tell you
  which page to update.

To run everything CI runs:

```sh
cargo fmt --check
cargo clippy -- -D warnings
cargo test
cargo doc --no-deps
cargo build --target-dir target/book && mdbook test book -L target/book/debug/deps
```

## Workflow

- `main` is protected. Changes land by pull request with green CI, even
  from the maintainer.
- Commit subjects are imperative and under 72 characters. Add a body
  when the reason is not obvious from the diff.
- A user-visible change adds a line under `Unreleased` in
  `CHANGELOG.md` in the same pull request.

## Versioning

Semantic versioning; before 1.0 a breaking change bumps the minor
version and everything else bumps the patch. The minimum supported Rust
version is declared in `Cargo.toml` and checked in CI; raising it is a
minor bump and a changelog entry.

## Releasing

1. On a branch: bump `version` in `Cargo.toml`, rename `Unreleased` in
   the changelog to the version and date, update README examples if the
   API moved. Open a PR, merge.
2. `cargo publish --dry-run` and `cargo package --list` to check what
   ships.
3. `git tag -a vX.Y.Z -m "vaster X.Y.Z" && git push --tags`.
4. `cargo publish`, then a GitHub release from the tag with the
   changelog section as the body.

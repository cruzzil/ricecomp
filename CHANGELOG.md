# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.5.0] - 2026-07-11

### Fixed

- `RCDecoder::decode`, `decode_short`, and `decode_byte`: return
  `DecodeError::EndOfBuffer` instead of panicking (or, for a run of zero bytes,
  spinning forever) when the compressed stream is truncated or the caller
  requests more pixels than were encoded. The decoders only checked the input
  boundary once per block, so streaming reads could index past the end of the
  slice. Every streaming read now goes through a bounds-checked helper, matching
  the graceful end-of-stream error returned by the original CFITSIO
  `fits_rdecomp`.

### Changed

- The `arbitrary` support now lives entirely in the fuzz crate. The
  fuzzing-only `DataInt`, `DataShort`, and `DataByte` wrapper types (and their
  `#[derive(arbitrary::Arbitrary)]` impls) were moved out of the library and
  into the individual fuzz targets. As a result the `arbitrary` optional
  dependency and the `arbitrary` feature have been removed, and `ricecomp` now
  builds with **no external runtime dependencies**.
- The fuzz targets were updated to the current encoder/decoder API introduced in
  0.3.0 and 0.4.0.

## [0.4.1] - 2025-05-04

### Added

- `DecodeError::NotProperlyAllocated`, returned by the decoders when the input
  buffer is shorter than the mandatory 4-byte header, instead of panicking on an
  out-of-range index.

## [0.4.0] - 2025-05-03

### Changed

- **Breaking:** `RCDecoder::decode`, `decode_short`, and `decode_byte` now write
  the decoded pixels into a caller-supplied `&mut [_]` output buffer and return
  `Result<(), DecodeError>`, rather than allocating and returning the buffer.
  This lets callers reuse allocations across calls.

## [0.3.0] - 2025-04-28

### Added

- 16-bit (`i16`) and 8-bit (`i8`) encode/decode variants: `encode_short` /
  `decode_short` and `encode_byte` / `decode_byte`, alongside the existing 32-bit
  (`i32`) `encode` / `decode`.
- Fuzz targets covering the int, short, and byte round trips.

### Changed

- **Breaking:** `RCEncoder` is now generic over its output sink,
  `RCEncoder<W: Write>`. `RCEncoder::new` takes the writer to emit the compressed
  stream into, and `encode*` write directly to it instead of returning an owned
  buffer.

## [0.2.0] - 2024-09-22

### Changed

- **Breaking:** Reorganised the flat, CFITSIO-style free functions
  (`fits_rcomp` / `fits_rdecomp`) into `RCEncoder` and `RCDecoder` types split
  across the `write` and `read` modules.
- Introduced the `EncodeError` and `DecodeError` types (`EndOfBuffer`,
  `ZeroSizeInput`) so failures are reported through `Result` instead of magic
  return codes.

## [0.1.2] - 2022-03-09

### Changed

- Updated the README and reorganised the bundled licenses into a `licenses/`
  directory.

## [0.1.1] - 2022-03-05

### Added

- Repository and package metadata for publishing to crates.io.

## [0.1.0] - 2022-03-05

### Added

- Initial release: a Rust port of the Rice compression routines
  (`fits_rcomp` / `fits_rdecomp`) from CFITSIO's `ricecomp.c`, originally written
  by Richard White at STScI.

[![Crates.io](https://img.shields.io/crates/v/ricecomp.svg)](https://crates.io/crates/ricecomp)
[![Actions Status](https://github.com/cruzzil/ricecomp/workflows/CI/badge.svg)](https://github.com/cruzzil/ricecomp/actions)
[![Documentation](https://docs.rs/ricecomp/badge.svg)](https://docs.rs/ricecomp/)
[![codecov](https://codecov.io/gh/cruzzil/ricecomp/branch/master/graph/badge.svg?token=YZEX06JT7K)](https://codecov.io/gh/cruzzil/ricecomp)
[![Dependency status](https://deps.rs/repo/github/cruzzil/ricecomp/status.svg)](https://deps.rs/repo/github/cruzzil/ricecomp)

# ricecomp

A Rust implementation of the **Rice compression** algorithm used by
[CFITSIO](https://heasarc.gsfc.nasa.gov/fitsio/) for the tile-compression of FITS
images. It is a transpiled and refactored port of `ricecomp.c`, originally
written by Richard White at STScI and contributed to CFITSIO in July 1999.

Rice coding is a fast, lossless entropy coder for integer data. It works best on
images whose adjacent pixels are strongly correlated (astronomical frames,
detector readouts, and similar), where the pixel-to-pixel differences are small
and cluster near zero. For a full walk-through of how the coder works, see
[ALGORITHM.md](ALGORITHM.md).

## Features

- Lossless round-trip compression and decompression of `i32`, `i16`, and `i8`
  pixel arrays.
- Streaming encoder generic over any [`std::io::Write`] sink.
- Decoder writes into a caller-supplied output buffer so allocations can be
  reused.
- Graceful, `Result`-based error handling — a truncated or malformed stream
  returns an error instead of panicking.
- No external runtime dependencies.
- Bit-compatible with CFITSIO's `fits_rcomp` / `fits_rdecomp`.

## Installation

```toml
[dependencies]
ricecomp = "0.5"
```

## Usage

The API is split into two modules:

- `write::RCEncoder` — compresses a slice of pixels into a byte sink.
- `read::RCDecoder` — decompresses a byte slice back into pixels.

Both take a **block size** (`nblock`), the number of pixels that share a single
Rice parameter. CFITSIO uses `32` for images; larger blocks adapt more slowly to
local statistics, smaller blocks spend more overhead on per-block headers.

The number of pixels (`nx`) and the block size are **not** stored in the stream,
so the decoder must be given the same values that were used to encode.

### Round trip

```rust
use ricecomp::read::RCDecoder;
use ricecomp::write::RCEncoder;

let pixels: [i32; 8] = [100, 101, 103, 102, 104, 108, 110, 109];
let block_size = 32;

// Encode into any std::io::Write sink (here a Vec<u8>).
let mut compressed: Vec<u8> = Vec::new();
let mut encoder = RCEncoder::new(&mut compressed);
let written = encoder
    .encode(&pixels, pixels.len(), block_size)
    .expect("encoding failed");
println!("compressed {} pixels into {} bytes", pixels.len(), written);

// Decode into a caller-supplied buffer.
let decoder = RCDecoder::new();
let mut output = vec![0u32; pixels.len()];
decoder
    .decode(&compressed, pixels.len(), block_size, &mut output)
    .expect("decoding failed");

let recovered: Vec<i32> = output.iter().map(|&x| x as i32).collect();
assert_eq!(recovered, pixels);
```

`RCEncoder::new` accepts anything implementing `std::io::Write`, so you can
compress straight into a `Vec<u8>`, a file, or any other writer. The decoder
writes into the `output` slice you pass in, letting you reuse allocations across
calls.

### 16-bit and 8-bit data

The same pattern applies to narrower integers via the dedicated variants:

| Element type | Encode          | Decode          | Decode output |
| ------------ | --------------- | --------------- | ------------- |
| `i32`        | `encode`        | `decode`        | `&mut [u32]`  |
| `i16`        | `encode_short`  | `decode_short`  | `&mut [u16]`  |
| `i8`         | `encode_byte`   | `decode_byte`   | `&mut [u8]`   |

Each width uses Rice parameters tuned to its dynamic range, so the streams are
not interchangeable — decode with the same variant you encoded with.

## Error handling

- `write::EncodeError` — `ZeroSizeInput` (empty input or zero block size) and
  `EndOfBuffer` (the writer could not accept more bytes).
- `read::DecodeError` — `ZeroSizeInput`, `NotProperlyAllocated` (input shorter
  than the mandatory 4-byte header), and `EndOfBuffer` (the compressed stream
  ended before the requested number of pixels was produced).

## References

- Algorithm description, section 6.1 of the FITS tile-compression convention:
  <https://fits.gsfc.nasa.gov/registry/tilecompression/tilecompression2.3.pdf>
- Comparison of compression methods for FITS:
  <https://arxiv.org/pdf/0903.2140.pdf>
- Original C source: `ricecomp.c` in CFITSIO, by Richard White (STScI), 1999.

## License

Licensed under the MIT license. The bundled CFITSIO license, which covers the
original C implementation this port derives from, is included under
[`licenses/`](licenses/).

# The Rice Compression Algorithm

This document explains how the Rice coder implemented in this crate works. It
describes the same algorithm as CFITSIO's `ricecomp.c`; the implementation here
is bit-compatible with it. For the formal specification see section 6.1 of the
[FITS tile-compression convention](https://fits.gsfc.nasa.gov/registry/tilecompression/tilecompression2.3.pdf).

## The idea in one paragraph

Rice coding is an entropy coder for integers that are *small on average*. Images
rarely satisfy that directly, but the **difference** between neighbouring pixels
usually does: in a smooth region consecutive pixels are nearly equal, so their
differences cluster tightly around zero. The coder therefore (1) takes
differences of adjacent pixels, (2) folds the signed differences into
non-negative integers, and (3) encodes each value as a short *unary* prefix plus
a fixed number of *raw* low bits. The number of raw bits is chosen per block of
pixels to match the local data, which is what makes the scheme adaptive.

The whole pipeline is lossless: every step is exactly invertible.

## Step 1 — Difference and zig-zag mapping

Processing is sequential. `lastpix` starts as the first pixel; for every
subsequent pixel the coder computes

```text
pdiff = pixel[i] - lastpix        (wrapping / modular subtraction)
lastpix = pixel[i]
```

The subtraction is allowed to overflow — it wraps — because the decoder performs
the exact inverse wrapping addition, so the original value is always recovered.

`pdiff` is signed, but Rice coding wants non-negative inputs. A **zig-zag**
mapping interleaves positive and negative values onto the non-negative integers
so that small-magnitude differences (of either sign) map to small codes:

```text
diff = if pdiff < 0 { !(pdiff << 1) }   // -1 -> 1, -2 -> 3, ...
       else         {   pdiff << 1  }   //  0 -> 0,  1 -> 2, ...
```

(`!` is bitwise complement.) So `0, -1, 1, -2, 2, …` map to `0, 1, 2, 3, 4, …`.

## Step 2 — Choosing the split parameter FS per block

Pixels are processed in blocks of `nblock` (the caller's block size; CFITSIO uses
32 for images). The final block may be shorter.

For each block the coder sums the mapped differences and derives **FS**, the
number of low-order bits that will be stored raw for every value in the block.
FS is essentially `floor(log2(mean/2))`, computed as:

```text
mean  = (pixelsum - nblock/2 - 1) / nblock     // clamped at 0
psum  = floor(mean) >> 1
FS    = number of bits needed to represent psum // 0 if psum == 0
```

Picking FS from the block mean balances the two halves of each code: the raw low
bits capture the typical magnitude, while the unary prefix stays short because
`value >> FS` is small for typical values.

## Step 3 — Encoding a block

FS is written into the stream in a small fixed-width header field of `fsbits`
bits, and the header value is offset so that two values can act as escapes:

| Header value (`fsbits` wide) | Meaning                                             |
| ---------------------------- | --------------------------------------------------- |
| `0`                          | Low-entropy escape: the whole block is zero         |
| `FS + 1`                     | Normal case: split each value at FS bits            |
| `FSMAX + 1`                  | High-entropy escape: store values raw in `BBITS`    |

### Normal case

The header `FS + 1` is written, then each mapped value `v` is encoded as:

- **Unary prefix:** `top = v >> FS` is written as `top` zero bits followed by a
  terminating `1` bit.
- **Raw remainder:** the low `FS` bits of `v` are written verbatim.

So a value is `(v >> FS)` zeros, a `1`, then `FS` literal bits — short when `v`
is close to the block's typical magnitude.

### Low-entropy escape

If FS is 0 *and* every mapped difference in the block is 0 (a perfectly flat run
of pixels), the block collapses to a single `0` header and nothing else.

### High-entropy escape

If FS reaches `FSMAX`, the differences are too large for Rice coding to pay off —
the unary prefixes would explode. Instead the coder writes the header `FSMAX + 1`
and stores each mapped value directly in `BBITS` bits, side-stepping Rice coding
for that block. This caps the worst case at roughly the raw bit width.

## Step 4 — The stream header

The very first pixel cannot be differenced against anything, so it is written
**raw as a 32-bit value** at the start of the stream (the first four bytes). The
decoder reads it back to seed `lastpix` before decoding the first block. This is
why `DecodeError::NotProperlyAllocated` is returned for any input shorter than
four bytes.

Note that the number of pixels (`nx`) and the block size (`nblock`) are **not**
stored in the stream. The caller must supply the same values to the decoder that
were used by the encoder.

## Width-specific parameters

`fsbits`, `FSMAX`, and `BBITS` depend on the integer width being compressed. They
are chosen so the header field is just wide enough to hold every legal FS value
plus the two escapes, and so `BBITS` matches the element size:

| Element | `fsbits` | `FSMAX` | `BBITS` (raw width) |
| ------- | -------- | ------- | ------------------- |
| `i32`   | 5        | 25      | 32                  |
| `i16`   | 4        | 14      | 16                  |
| `i8`    | 3        | 6       | 8                   |

Because these parameters differ, a stream produced by `encode` cannot be read by
`decode_short`/`decode_byte` (and vice versa); always decode with the variant
that matches the encoder.

## Decoding

Decoding mirrors encoding exactly:

1. Read the first 32-bit pixel to seed `lastpix`.
2. For each block, read the `fsbits`-wide header:
   - `0` → the block is all zeros (all differences are 0).
   - `FSMAX + 1` → read each value raw from `BBITS` bits.
   - otherwise → `FS = header - 1`; for each value, count zero bits up to the
     terminating `1` to recover the unary prefix `top`, read `FS` raw low bits,
     and reassemble `v = (top << FS) | low`.
3. Undo the zig-zag mapping to recover the signed difference, then add it
   (with wrapping) to `lastpix` to recover the pixel.

Every read from the compressed buffer is bounds-checked, so a truncated stream
yields `DecodeError::EndOfBuffer` rather than a panic or an infinite loop.

## Why it is fast

Rice coding needs no probability tables, no arithmetic, and no divisions in the
inner loop — only shifts, additions, and bit packing. Compression and
decompression are close to a single linear pass over the data, which is what
makes it attractive for the large image tiles that FITS files carry.

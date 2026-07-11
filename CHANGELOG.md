# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.5.0]

### Fixed

- `RCDecoder::decode`, `decode_short`, and `decode_byte`: return
  `DecodeError::EndOfBuffer` instead of panicking (or, for a run of zero bytes,
  spinning forever) when the compressed stream is truncated or the caller
  requests more pixels than were encoded. The decoders only checked the input
  boundary once per block, so streaming reads could index past the end of the
  slice. Every streaming read now goes through a bounds-checked helper, matching
  the graceful end-of-stream error returned by the original CFITSIO
  `fits_rdecomp`.

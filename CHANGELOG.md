# Changelog

## v0.11.8

- Adds the built-in function `le()`, which reverses the bytes of an integer,
essentially performing little-endian encoding. It's important that the
argument be sized with a multiple of 8 bits. For example: `le(0x1234)`
or ```le(65000`16)```.
- Makes it so assembly won't stop at the first resolve error, which allows
more errors to be caught in a single execution.
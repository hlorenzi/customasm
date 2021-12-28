# Changelog

## v0.11.13

- Adds string encoding conversion functions: `utf8()`, `ascii()`, `utf16be()`,
`utf16le()`, `utf32be()`, and `utf32le()`. For example: `utf16be("abc")` will
give you `0x0061_0062_0063`. The default encoding is UTF-8. The ASCII encoding
will convert unavailable codepoints to `0x00`.

## v0.11.9

- Makes it so the assembler will select the rule with the fewest amount of
output bits in the case of multiple matches.
- Adds the `mesen-mlb` symbol output format, for use with the Mesen NES emulator.

## v0.11.8

- Adds the built-in function `le()`, which reverses the bytes of an integer,
essentially performing little-endian encoding. It's important that the
argument be sized with a multiple of 8 bits. For example: `le(0x1234)`
or ```le(65000`16)```.
- Makes it so assembly won't stop at the first resolve error, which allows
more errors to be caught in a single execution.
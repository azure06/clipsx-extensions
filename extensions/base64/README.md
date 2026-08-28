# Base64

Encodes captured text as plain Base64 and bounded binary representations as
MIME-preserving data URLs. PNG, JPEG, WebP, GIF, AVIF, BMP, and icon results
open as native expiring image previews after Decode; the package recognizes
those formats from bounded binary signatures when raw Base64 has no MIME
declaration. Other binary results show their media type and size. Decoding
restores UTF-8 text, explicit data-URL media types, or bounded generic binary
bytes without reading copied file paths.

The package owns Base64 recognition, its metadata card, and its codec. Detection
records only syntax-derived metadata such as decoded size and declared MIME
type; decoded content is revealed only after the user runs Decode. The Actions
menu shows exactly one relevant action and previews the result before changing
the clipboard or history. Encoding is limited to 7 MiB so its Base64 output
stays below the host's 10 MiB transform-result ceiling.

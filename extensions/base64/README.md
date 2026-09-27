# Base64

Encodes captured text as plain Base64 and bounded binary representations as
MIME-preserving data URLs. PNG, JPEG, WebP, GIF, AVIF, BMP, and icon results
open as host-rendered, durable clip-owned result tabs after Decode; the package recognizes
those formats from bounded binary signatures when raw Base64 has no MIME
declaration. Other binary results show their media type and size. Decoding
restores UTF-8 text, explicit data-URL media types, or bounded generic binary
bytes without reading copied file paths.

The package owns Base64 recognition, its compact key/value summary, and its codec. Detection
records only syntax-derived metadata such as decoded size and declared MIME
type; decoded content is revealed only after the user runs Decode. Automatic
recognition requires a data URL, canonical Base64 signals, printable UTF-8
output, or a known image signature, so ordinary alphabet-only text is not
hijacked. Decode remains available manually for every strictly decodable plain
text value. Encoding is limited to 7 MiB so its Base64 output stays below the
host's output budget.

The v3.2 offline availability check offers Decode for strictly decodable text
and Encode for other supported text or binary input. The setup selector supplies
`operation`; there is no duplicate operation field. Conversion runs locally and
returns a named `converted` output with Result and Compare views. No model,
network request, or automatic clipboard write is involved.

Build and validate the local archive with:

```powershell
npm run package -- base64 dist/base64-2.0.0-v3.2.clipsx
```

# JWT Inspector

Safely decodes JWT headers and payload claims into a compact ClipsX view. This extension never presents decoded tokens as verified and performs no network requests.

- Strict three-segment JWT detection
- Header and payload claim views
- Copy individual claims or extract either JSON section
- Human-readable `exp`, `iat`, and `nbf` timestamps

Signature verification is intentionally out of scope.

Tools offers JWT payload and JWT header setups for valid tokens. Extraction creates a saved, clip-owned JSON result with Result and Compare views.

# Rewrite

Rewrite selected or newly captured plain text with ClipsX's configured local generation model. Automatic rules are configured by the host and results remain attached to the source clip until the user deletes or promotes them.

## Local review

Built-in setups and saved setups use one durable operation. The guest requests a
host model step, then returns the named `rewritten` output with Result and Compare
views. Truncated generation fails visibly. The package never writes the clipboard.

The `on-copy` activation considers accepted plain-text captures only after an
exact app rule is enabled and consented. Configure Local Text Generation first.
Manual and automatic results remain attached to their source clip across restart.

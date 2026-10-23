# Rewrite

Rewrite selected or newly captured plain text with ClipsX's configured generation
model. The selected provider may run locally or be hosted. Results remain attached
to the source clip until the user deletes or promotes them; Rewrite never copies
or pastes automatically.

## Configuration

Choose an extension-generation model in Intelligence. Built-in and saved setups
use the same durable operation, with Result and Compare views. Truncated generation
fails visibly. Manual Rewrite runs the selected setup directly.

For automatic captures, open Rewrite's automation settings. Add multiple ordered
rules for an application or All copied clips, choose a setup, and optionally ask a
yes/no question under “Only run when…”. Conditions use General decisions. Configure
that model before enabling conditional rules; a local model keeps this processing
on the device, while a hosted provider receives the text.

ClipsX tries application-specific rules first, then All copied clips as a fallback.
Only the first match runs; an unconditional rule always matches. A Yes probability
of at least 50% matches. Model failures stop the attempt and offer retry. Sample
testing checks saved rules without saving the sample or generating a rewrite.

The host owns conditions, consent, private/secret exclusions, capture snapshots,
and restart recovery. No guest contract change is required. Only the selected
setup generates output, originals remain unchanged, and attached results survive
restart.

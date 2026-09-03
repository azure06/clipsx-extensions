#[allow(clippy::too_many_arguments)]
mod bindings {
    use super::Base64;
    wit_bindgen::generate!({ path: "../../sdk/wit", world: "extension" });
    export!(Base64);
}
use bindings::clipsx::extension::types::*;
struct Base64;

struct DecodedBase64 {
    bytes: Vec<u8>,
    mime_type: Option<String>,
}

const MAX_ENCODE_INPUT_BYTES: usize = 7 * 1024 * 1024;

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Base64FacetPayload {
    schema_version: u8,
    decoded_bytes: usize,
    encoded_chars: usize,
    mime_type: Option<String>,
    encoding: String,
    data_url: bool,
}

struct Base64Analysis {
    decoded_bytes: usize,
    encoded_chars: usize,
    mime_type: Option<String>,
    encoding: &'static str,
    data_url: bool,
}

impl bindings::Guest for Base64 {
    fn detect(id: String, input: Representation) -> Result<Vec<Facet>, GuestError> {
        if id != "detect-base64" {
            return Ok(vec![]);
        }
        let Some(raw) = text(&input).map(str::trim) else {
            return Ok(vec![]);
        };
        let Some(analysis) = analyze_input(raw) else {
            return Ok(vec![]);
        };
        if raw.len() < 7 || analysis.decoded_bytes == 0 {
            return Ok(vec![]);
        }
        Ok(vec![Facet {
            id: "base64".into(),
            payload_json: serde_json::to_string(&Base64FacetPayload {
                schema_version: 1,
                decoded_bytes: analysis.decoded_bytes,
                encoded_chars: analysis.encoded_chars,
                mime_type: analysis.mime_type,
                encoding: analysis.encoding.into(),
                data_url: analysis.data_url,
            })
            .map_err(|_| failed("could not serialize Base64 metadata"))?,
        }])
    }
    fn render_detail(
        id: String,
        input: Representation,
        facet: Option<Facet>,
    ) -> Result<RenderModel, GuestError> {
        if id != "base64-summary" {
            return Err(unsupported("unknown Base64 renderer"));
        }
        let raw = text(&input)
            .map(str::trim)
            .ok_or_else(|| invalid("Base64 text is required"))?;
        let facet = facet.ok_or_else(|| invalid("Base64 metadata is missing"))?;
        let payload: Base64FacetPayload = serde_json::from_str(&facet.payload_json)
            .map_err(|_| invalid("Base64 metadata is invalid"))?;
        let decoded = decode_input(raw).ok_or_else(|| invalid("Base64 metadata is invalid"))?;
        Ok(RenderModel::KeyValue(vec![
            KeyValueEntry {
                key: "Encoded".into(),
                value: preview_value(raw),
            },
            KeyValueEntry {
                key: "Decoded".into(),
                value: decoded_preview(&decoded),
            },
            KeyValueEntry {
                key: "Details".into(),
                value: format!(
                    "{} · {} chars · {} decoded · {}{}",
                    if payload.data_url {
                        "Data URL"
                    } else {
                        "Base64"
                    },
                    payload.encoded_chars,
                    human_bytes(payload.decoded_bytes),
                    payload.encoding,
                    payload
                        .mime_type
                        .as_deref()
                        .map(|mime| format!(" · {mime}"))
                        .unwrap_or_default()
                ),
            },
        ]))
    }
    fn render_compact(
        id: String,
        _: Representation,
        _: Option<Facet>,
    ) -> Result<CompactModel, GuestError> {
        if id != "base64-summary" {
            return Err(unsupported("unknown Base64 renderer"));
        }
        Ok(CompactModel {
            leading: LeadingVisual::HostIcon("binary".into()),
            title: None,
            subtitle: None,
            badge: Some("Base64".into()),
            accessibility_label: "Base64 encoded data".into(),
        })
    }
    fn transform(
        id: String,
        input: Representation,
        parameters: String,
    ) -> Result<Vec<OutputRepresentation>, GuestError> {
        if id != "base64-codec" {
            return Err(unsupported("unknown transformer"));
        }
        let p: serde_json::Value =
            serde_json::from_str(&parameters).map_err(|_| invalid("invalid parameters"))?;
        match p.get("operation").and_then(|v| v.as_str()) {
            Some("encode") => {
                let bytes = bytes(&input).ok_or_else(|| invalid("file lists cannot be encoded"))?;
                if bytes.len() > MAX_ENCODE_INPUT_BYTES {
                    return Err(invalid("Base64 encoding is limited to 7 MiB"));
                }
                let encoded = encode(bytes);
                let value = match input.content {
                    Content::Binary(_) => format!(
                        "data:{};base64,{encoded}",
                        safe_mime(input.mime_type.as_deref()).unwrap_or("application/octet-stream")
                    ),
                    Content::Text(_) => encoded,
                    Content::Files(_) => return Err(invalid("file lists cannot be encoded")),
                };
                Ok(vec![text_output(value)])
            }
            Some("decode") => {
                let raw = text(&input).ok_or_else(|| invalid("Base64 text is required"))?;
                let decoded = decode_input(raw.trim()).ok_or_else(|| invalid("invalid Base64"))?;
                if decoded.mime_type.as_deref().is_none_or(is_text_mime) {
                    if let Ok(value) = String::from_utf8(decoded.bytes.clone()) {
                        return Ok(vec![OutputRepresentation {
                            format_key: format!(
                                "mime:{}",
                                decoded.mime_type.as_deref().unwrap_or("text/plain")
                            ),
                            mime_type: decoded.mime_type.unwrap_or_else(|| "text/plain".into()),
                            content: OutputContent::Text(value),
                        }]);
                    }
                }
                let mime_type = decoded
                    .mime_type
                    .or_else(|| sniff_raster_mime(&decoded.bytes).map(str::to_string))
                    .unwrap_or_else(|| "application/octet-stream".into());
                Ok(vec![OutputRepresentation {
                    format_key: format!("mime:{mime_type}"),
                    mime_type,
                    content: OutputContent::Binary(decoded.bytes),
                }])
            }
            _ => Err(invalid("operation must be encode or decode")),
        }
    }
    fn run_action(
        _: String,
        _: Representation,
        _: Option<Facet>,
        _: String,
    ) -> Result<ActionResult, GuestError> {
        Err(unsupported("actions use transformer presets"))
    }
    fn action_state(
        id: String,
        input: Representation,
        facet: Option<Facet>,
        _: String,
    ) -> Result<ActionState, GuestError> {
        let recognized = facet
            .as_ref()
            .is_some_and(|facet| facet.id == "infiniti.base64.base64");
        let decodable = recognized || text(&input).is_some_and(is_decodable);
        let encodable = bytes(&input).is_some();
        match id.as_str() {
            "decode-base64" => Ok(if decodable {
                ActionState::Enabled
            } else {
                ActionState::Hidden
            }),
            "encode-base64" => Ok(if recognized || !encodable {
                ActionState::Hidden
            } else if bytes(&input).is_some_and(|value| value.len() > MAX_ENCODE_INPUT_BYTES) {
                ActionState::Disabled("Base64 encoding is limited to 7 MiB".into())
            } else {
                ActionState::Enabled
            }),
            _ => Ok(ActionState::Enabled),
        }
    }
}
fn text(i: &Representation) -> Option<&str> {
    if let Content::Text(v) = &i.content {
        Some(v)
    } else {
        None
    }
}
fn bytes(i: &Representation) -> Option<&[u8]> {
    match &i.content {
        Content::Text(value) => Some(value.as_bytes()),
        Content::Binary(value) => Some(value),
        Content::Files(_) => None,
    }
}

fn text_output(value: String) -> OutputRepresentation {
    OutputRepresentation {
        format_key: "mime:text/plain".into(),
        mime_type: "text/plain".into(),
        content: OutputContent::Text(value),
    }
}

fn safe_mime(value: Option<&str>) -> Option<&str> {
    value.filter(|value| {
        let Some((kind, subtype)) = value.split_once('/') else {
            return false;
        };
        !kind.is_empty()
            && !subtype.is_empty()
            && value.len() <= 127
            && value.bytes().all(|byte| {
                byte.is_ascii_alphanumeric()
                    || matches!(
                        byte,
                        b'!' | b'#' | b'$' | b'&' | b'+' | b'-' | b'.' | b'^' | b'_' | b'/'
                    )
            })
    })
}

fn is_text_mime(value: &str) -> bool {
    value.starts_with("text/")
        || matches!(
            value,
            "application/json" | "application/xml" | "image/svg+xml"
        )
}

fn sniff_raster_mime(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(&[137, 80, 78, 71, 13, 10, 26, 10]) {
        Some("image/png")
    } else if bytes.starts_with(&[0xff, 0xd8, 0xff]) {
        Some("image/jpeg")
    } else if bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        Some("image/webp")
    } else if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        Some("image/gif")
    } else if bytes.len() >= 12
        && &bytes[4..8] == b"ftyp"
        && (&bytes[8..12] == b"avif" || &bytes[8..12] == b"avis")
    {
        Some("image/avif")
    } else if bytes.starts_with(b"BM") {
        Some("image/bmp")
    } else if bytes.starts_with(&[0, 0, 1, 0]) {
        Some("image/x-icon")
    } else {
        None
    }
}
const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
fn encode(input: &[u8]) -> String {
    let mut out = String::with_capacity(input.len().div_ceil(3) * 4);
    for chunk in input.chunks(3) {
        let n = ((chunk[0] as u32) << 16)
            | ((chunk.get(1).copied().unwrap_or(0) as u32) << 8)
            | chunk.get(2).copied().unwrap_or(0) as u32;
        out.push(ALPHABET[((n >> 18) & 63) as usize] as char);
        out.push(ALPHABET[((n >> 12) & 63) as usize] as char);
        out.push(if chunk.len() > 1 {
            ALPHABET[((n >> 6) & 63) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            ALPHABET[(n & 63) as usize] as char
        } else {
            '='
        });
    }
    out
}

fn analyze_input(value: &str) -> Option<Base64Analysis> {
    let value = value.trim();
    let (payload, mime_type, data_url) = if let Some(data_url) = value.strip_prefix("data:") {
        let (metadata, payload) = data_url.split_once(',')?;
        let mut metadata = metadata.split(';');
        let declared_mime = metadata.next().unwrap_or_default();
        if !metadata.any(|parameter| parameter.eq_ignore_ascii_case("base64")) {
            return None;
        }
        let mime_type = if declared_mime.is_empty() {
            Some("text/plain".into())
        } else {
            Some(safe_mime(Some(declared_mime))?.into())
        };
        (payload, mime_type, true)
    } else {
        (value, None, false)
    };
    let (decoded_bytes, encoded_chars, encoding) = analyze_payload(payload)?;
    let has_explicit_signal = data_url
        || payload
            .bytes()
            .any(|byte| matches!(byte, b'=' | b'+' | b'/' | b'-' | b'_'))
        || (payload.bytes().any(|byte| matches!(byte, b'\r' | b'\n')) && encoded_chars >= 32);
    if !has_explicit_signal {
        let decoded = decode(payload)?;
        if !is_printable_utf8(&decoded) && sniff_raster_mime(&decoded).is_none() {
            return None;
        }
    }
    Some(Base64Analysis {
        decoded_bytes,
        encoded_chars,
        mime_type,
        encoding,
        data_url,
    })
}

fn is_decodable(value: &str) -> bool {
    let value = value.trim();
    let payload = if let Some(data_url) = value.strip_prefix("data:") {
        let Some((metadata, payload)) = data_url.split_once(',') else {
            return false;
        };
        if !metadata
            .split(';')
            .skip(1)
            .any(|parameter| parameter.eq_ignore_ascii_case("base64"))
        {
            return false;
        }
        payload
    } else {
        value
    };
    analyze_payload(payload).is_some()
}

fn is_printable_utf8(value: &[u8]) -> bool {
    std::str::from_utf8(value).is_ok_and(|text| {
        !text.is_empty()
            && text.chars().all(|character| {
                !character.is_control() || matches!(character, '\n' | '\r' | '\t')
            })
    })
}

fn analyze_payload(value: &str) -> Option<(usize, usize, &'static str)> {
    let mut non_padding = 0usize;
    let mut padding = 0usize;
    let mut last_value = 0u8;
    let mut saw_padding = false;
    let mut standard = false;
    let mut url_safe = false;
    for byte in value.bytes() {
        if byte.is_ascii_whitespace() {
            continue;
        }
        if byte == b'=' {
            saw_padding = true;
            padding += 1;
            if padding > 2 {
                return None;
            }
            continue;
        }
        if saw_padding {
            return None;
        }
        last_value = match byte {
            b'A'..=b'Z' => byte - b'A',
            b'a'..=b'z' => byte - b'a' + 26,
            b'0'..=b'9' => byte - b'0' + 52,
            b'+' => {
                standard = true;
                62
            }
            b'/' => {
                standard = true;
                63
            }
            b'-' => {
                url_safe = true;
                62
            }
            b'_' => {
                url_safe = true;
                63
            }
            _ => return None,
        };
        non_padding += 1;
    }
    if non_padding == 0 || standard && url_safe {
        return None;
    }
    let remainder = non_padding % 4;
    if remainder == 1
        || (padding > 0 && !(non_padding + padding).is_multiple_of(4)
            || padding == 1 && remainder != 3
            || padding == 2 && remainder != 2)
        || remainder == 2 && last_value & 0x0f != 0
        || remainder == 3 && last_value & 0x03 != 0
    {
        return None;
    }
    Some((
        non_padding * 6 / 8,
        non_padding + padding,
        if url_safe { "URL-safe" } else { "Standard" },
    ))
}

const PREVIEW_LIMIT: usize = 460;

fn preview_value(value: &str) -> String {
    let mut chars = value.chars();
    let preview = chars.by_ref().take(PREVIEW_LIMIT).collect::<String>();
    if chars.next().is_none() {
        preview
    } else {
        format!("{preview}\n… preview truncated")
    }
}

fn decoded_preview(decoded: &DecodedBase64) -> String {
    if let Ok(value) = std::str::from_utf8(&decoded.bytes) {
        return preview_value(value);
    }
    format!(
        "Binary payload\n{} · {}\nUse Decode Base64 for the rendered preview.",
        decoded.mime_type.as_deref().unwrap_or("undetected type"),
        human_bytes(decoded.bytes.len())
    )
}

fn human_bytes(value: usize) -> String {
    if value >= 1024 * 1024 {
        format!("{:.2} MiB", value as f64 / (1024.0 * 1024.0))
    } else if value >= 1024 {
        format!("{:.1} KiB", value as f64 / 1024.0)
    } else {
        format!("{value} bytes")
    }
}

fn decode_input(value: &str) -> Option<DecodedBase64> {
    let value = value.trim();
    if let Some(data_url) = value.strip_prefix("data:") {
        let (metadata, payload) = data_url.split_once(',')?;
        let mut metadata = metadata.split(';');
        let declared_mime = metadata.next().unwrap_or_default();
        if !metadata.any(|parameter| parameter.eq_ignore_ascii_case("base64")) {
            return None;
        }
        let mime_type = if declared_mime.is_empty() {
            "text/plain"
        } else {
            safe_mime(Some(declared_mime))?
        };
        return Some(DecodedBase64 {
            bytes: decode(payload)?,
            mime_type: Some(mime_type.into()),
        });
    }
    Some(DecodedBase64 {
        bytes: decode(value)?,
        mime_type: None,
    })
}

fn decode(value: &str) -> Option<Vec<u8>> {
    let mut compact = value
        .bytes()
        .filter(|b| !b.is_ascii_whitespace())
        .map(|byte| match byte {
            b'-' => b'+',
            b'_' => b'/',
            byte => byte,
        })
        .collect::<Vec<_>>();
    if compact.is_empty() || compact.len() % 4 == 1 {
        return None;
    }
    if compact.contains(&b'=') && compact.len() % 4 != 0 {
        return None;
    }
    while compact.len() % 4 != 0 {
        compact.push(b'=');
    }
    let mut out = Vec::new();
    for (ci, c) in compact.chunks(4).enumerate() {
        let last = ci + 1 == compact.len() / 4;
        if (!last && c.contains(&b'=')) || c[0] == b'=' || c[1] == b'=' {
            return None;
        }
        let v = |b| match b {
            b'A'..=b'Z' => Some(b - b'A'),
            b'a'..=b'z' => Some(b - b'a' + 26),
            b'0'..=b'9' => Some(b - b'0' + 52),
            b'+' => Some(62),
            b'/' => Some(63),
            b'=' => Some(0),
            _ => None,
        };
        let n = ((v(c[0])? as u32) << 18)
            | ((v(c[1])? as u32) << 12)
            | ((v(c[2])? as u32) << 6)
            | v(c[3])? as u32;
        out.push((n >> 16) as u8);
        if c[2] != b'=' {
            out.push((n >> 8) as u8)
        }
        if c[3] != b'=' {
            out.push(n as u8)
        }
    }
    let canonical = encode(&out);
    let canonical = canonical.trim_end_matches('=');
    let supplied = std::str::from_utf8(&compact).ok()?.trim_end_matches('=');
    (canonical == supplied).then_some(out)
}
fn err(code: GuestErrorCode, m: &str) -> GuestError {
    GuestError {
        code,
        message: m.into(),
    }
}
fn invalid(m: &str) -> GuestError {
    err(GuestErrorCode::InvalidInput, m)
}
fn unsupported(m: &str) -> GuestError {
    err(GuestErrorCode::Unsupported, m)
}
fn failed(m: &str) -> GuestError {
    err(GuestErrorCode::Failed, m)
}
#[cfg(test)]
mod tests {
    use super::*;
    use bindings::Guest;
    #[test]
    fn round_trip_utf8() {
        let s = "ClipsX 日本語";
        assert_eq!(
            String::from_utf8(decode(&encode(s.as_bytes())).unwrap()).unwrap(),
            s
        )
    }
    #[test]
    fn strict_padding() {
        assert!(decode("SGVsbG8=").is_some());
        assert_eq!(decode("SGVsbG8"), Some(b"Hello".to_vec()));
        assert_eq!(decode("SGVsbG8tXw"), Some(b"Hello-_".to_vec()));
        assert_eq!(decode("-_8"), Some(vec![251, 255]));
        assert!(decode("ordinary prose").is_none());
        assert!(decode("=AAA").is_none());
        assert!(decode("Zh==").is_none())
    }
    #[test]
    fn detector_heuristic_rejects_plain_alphanumeric_text() {
        let raw = "testtest";
        let bytes = decode(raw).unwrap();
        assert!(String::from_utf8(bytes).is_err());
        assert!(!raw.bytes().any(|byte| matches!(byte, b'=' | b'+' | b'/')));
        assert!(Base64::detect("detect-base64".into(), text_input(raw))
            .unwrap()
            .is_empty());
    }

    #[test]
    fn detector_accepts_unpadded_printable_text_and_known_binary() {
        assert_eq!(
            Base64::detect("detect-base64".into(), text_input("SGVsbG8"))
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            Base64::detect("detect-base64".into(), text_input("iVBORw0KGgo"))
                .unwrap()
                .len(),
            1
        );
    }

    fn text_input(value: &str) -> Representation {
        Representation {
            format_key: "test:text/plain".into(),
            mime_type: Some("text/plain".into()),
            storage_kind: "text".into(),
            content: Content::Text(value.into()),
        }
    }

    fn binary_input(value: &[u8], mime_type: &str) -> Representation {
        Representation {
            format_key: format!("mime:{mime_type}"),
            mime_type: Some(mime_type.into()),
            storage_kind: "binary_asset".into(),
            content: Content::Binary(value.into()),
        }
    }

    #[test]
    fn detects_and_round_trips_a_mime_preserving_image_data_url() {
        let png_header = [137, 80, 78, 71, 13, 10, 26, 10];
        let encoded = Base64::transform(
            "base64-codec".into(),
            binary_input(&png_header, "image/png"),
            serde_json::json!({ "operation": "encode" }).to_string(),
        )
        .unwrap();
        let OutputContent::Text(data_url) = &encoded[0].content else {
            panic!("binary encoding must produce text");
        };
        assert_eq!(data_url, "data:image/png;base64,iVBORw0KGgo=");
        let mut facets = Base64::detect("detect-base64".into(), text_input(data_url)).unwrap();
        assert_eq!(facets.len(), 1);
        assert!(!facets[0].payload_json.contains("preview"));
        assert!(!facets[0].payload_json.contains("utf8"));
        let detail = Base64::render_detail(
            "base64-summary".into(),
            text_input(data_url),
            Some(facets.remove(0)),
        )
        .unwrap();
        assert!(matches!(
            detail,
            RenderModel::KeyValue(entries)
                if entries.len() == 3
                    && entries[0].key == "Encoded"
                    && entries[0].value == *data_url
                    && entries[1].key == "Decoded"
                    && entries[2].value.contains("Data URL")
        ));

        let decoded = Base64::transform(
            "base64-codec".into(),
            text_input(data_url),
            serde_json::json!({ "operation": "decode" }).to_string(),
        )
        .unwrap();
        assert_eq!(decoded[0].mime_type, "image/png");
        assert!(matches!(
            &decoded[0].content,
            OutputContent::Binary(value) if value == &png_header
        ));

        let raw = encode(&png_header);
        let decoded = Base64::transform(
            "base64-codec".into(),
            text_input(&raw),
            serde_json::json!({ "operation": "decode" }).to_string(),
        )
        .unwrap();
        assert_eq!(decoded[0].mime_type, "image/png");
    }

    #[test]
    fn recognizes_supported_raster_signatures_only_after_explicit_decode() {
        assert_eq!(sniff_raster_mime(b"\xff\xd8\xffrest"), Some("image/jpeg"));
        assert_eq!(
            sniff_raster_mime(b"RIFF\x00\x00\x00\x00WEBP"),
            Some("image/webp")
        );
        assert_eq!(sniff_raster_mime(b"GIF89a"), Some("image/gif"));
        assert_eq!(
            sniff_raster_mime(b"\x00\x00\x00\x00ftypavif"),
            Some("image/avif")
        );
        assert_eq!(sniff_raster_mime(b"BMrest"), Some("image/bmp"));
        assert_eq!(sniff_raster_mime(&[0, 0, 1, 0]), Some("image/x-icon"));
        assert_eq!(sniff_raster_mime(b"<svg></svg>"), None);
    }

    #[test]
    fn binary_assets_offer_encode() {
        assert!(matches!(
            Base64::action_state(
                "encode-base64".into(),
                binary_input(&[1, 2, 3], "application/pdf"),
                None,
                "{}".into()
            )
            .unwrap(),
            ActionState::Enabled
        ));
    }

    #[test]
    fn oversized_binary_assets_explain_why_encoding_is_disabled() {
        let input = binary_input(&vec![0; MAX_ENCODE_INPUT_BYTES + 1], "image/png");
        assert!(matches!(
            Base64::action_state("encode-base64".into(), input, None, "{}".into()).unwrap(),
            ActionState::Disabled(reason) if reason.contains("7 MiB")
        ));
    }

    #[test]
    fn action_state_offers_exactly_one_of_encode_or_decode() {
        assert!(matches!(
            Base64::action_state(
                "decode-base64".into(),
                text_input("SGVsbG8"),
                None,
                "{}".into()
            )
            .unwrap(),
            ActionState::Enabled
        ));
        assert!(matches!(
            Base64::action_state(
                "encode-base64".into(),
                text_input("SGVsbG8="),
                Some(Facet {
                    id: "infiniti.base64.base64".into(),
                    payload_json: "{}".into(),
                }),
                "{}".into()
            )
            .unwrap(),
            ActionState::Hidden
        ));

        assert!(matches!(
            Base64::action_state(
                "decode-base64".into(),
                text_input("ordinary prose"),
                None,
                "{}".into()
            )
            .unwrap(),
            ActionState::Hidden
        ));
        assert!(matches!(
            Base64::action_state(
                "encode-base64".into(),
                text_input("ordinary prose"),
                None,
                "{}".into()
            )
            .unwrap(),
            ActionState::Enabled
        ));
    }


    #[test]
    fn ambiguous_but_valid_base64_remains_manually_decodable() {
        assert!(Base64::detect("detect-base64".into(), text_input("testtest"))
            .unwrap()
            .is_empty());
        assert!(matches!(
            Base64::action_state("decode-base64".into(), text_input("testtest"), None, "{}".into())
                .unwrap(),
            ActionState::Enabled
        ));
    }
}

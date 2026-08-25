mod bindings {
    use super::Base64;
    wit_bindgen::generate!({ path: "../../sdk/wit", world: "extension" });
    export!(Base64);
}
use bindings::clipsx::extension::types::*;
struct Base64;
impl bindings::Guest for Base64 {
    fn detect(id: String, input: Representation) -> Result<Vec<Facet>, GuestError> {
        if id != "detect-base64" {
            return Ok(vec![]);
        }
        let Some(raw) = text(&input).map(str::trim) else {
            return Ok(vec![]);
        };
        let Some(bytes) = decode(raw) else {
            return Ok(vec![]);
        };
        if raw.len() < 8 || bytes.is_empty() {
            return Ok(vec![]);
        }
        let utf8 = String::from_utf8(bytes.clone()).ok();
        let has_base64_signal = raw.bytes().any(|byte| matches!(byte, b'=' | b'+' | b'/'));
        let readable_utf8 = utf8.as_deref().is_some_and(|value| {
            let total = value.chars().count();
            total > 0
                && value
                    .chars()
                    .filter(|ch| !ch.is_control() || matches!(ch, '\n' | '\r' | '\t'))
                    .count()
                    * 100
                    / total
                    >= 85
        });
        if !has_base64_signal && !readable_utf8 {
            return Ok(vec![]);
        }
        Ok(vec![Facet{id:"base64".into(),payload_json:serde_json::json!({"schemaVersion":1,"decodedBytes":bytes.len(),"utf8":utf8.is_some(),"preview":utf8.as_deref().map(|v|v.chars().take(160).collect::<String>())}).to_string()}])
    }
    fn render_detail(
        _: String,
        _: Representation,
        _: Option<Facet>,
    ) -> Result<RenderModel, GuestError> {
        Err(unsupported("Base64 uses its custom view"))
    }
    fn render_compact(
        id: String,
        input: Representation,
        _: Option<Facet>,
    ) -> Result<CompactModel, GuestError> {
        if id != "base64-workbench" {
            return Err(unsupported("unknown renderer"));
        }
        let bytes = decode(text(&input).unwrap_or_default().trim())
            .ok_or_else(|| invalid("invalid Base64"))?;
        let utf8 = String::from_utf8(bytes.clone()).ok();
        Ok(CompactModel {
            leading: LeadingVisual::Monogram("64".into()),
            title: Some(
                utf8.as_deref()
                    .and_then(|v| v.lines().next())
                    .filter(|v| !v.is_empty())
                    .unwrap_or("Base64 data")
                    .chars()
                    .take(80)
                    .collect(),
            ),
            subtitle: Some(format!("{} decoded bytes", bytes.len())),
            badge: Some(if utf8.is_some() { "UTF-8" } else { "Binary" }.into()),
            accessibility_label: format!("Base64 value with {} decoded bytes", bytes.len()),
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
        let raw = text(&input).ok_or_else(|| invalid("text input required"))?;
        let p: serde_json::Value =
            serde_json::from_str(&parameters).map_err(|_| invalid("invalid parameters"))?;
        let value = match p.get("operation").and_then(|v| v.as_str()) {
            Some("encode") => encode(raw.as_bytes()),
            Some("decode") => {
                String::from_utf8(decode(raw.trim()).ok_or_else(|| invalid("invalid Base64"))?)
                    .map_err(|_| invalid("decoded value is binary, not UTF-8"))?
            }
            _ => return Err(invalid("operation must be encode or decode")),
        };
        Ok(vec![OutputRepresentation {
            format_key: "mime:text/plain".into(),
            mime_type: "text/plain".into(),
            content: OutputContent::Text(value),
        }])
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
        _: Option<Facet>,
        _: String,
    ) -> Result<ActionState, GuestError> {
        if id != "decode-base64" {
            return Ok(ActionState::Enabled);
        }
        let available = text(&input)
            .map(str::trim)
            .and_then(decode)
            .and_then(|bytes| String::from_utf8(bytes).ok())
            .is_some();
        Ok(if available {
            ActionState::Enabled
        } else {
            ActionState::Disabled("Input is not UTF-8 Base64".into())
        })
    }
}
fn text(i: &Representation) -> Option<&str> {
    if let Content::Text(v) = &i.content {
        Some(v)
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
fn decode(value: &str) -> Option<Vec<u8>> {
    let compact = value
        .bytes()
        .filter(|b| !b.is_ascii_whitespace())
        .collect::<Vec<_>>();
    if compact.is_empty() || compact.len() % 4 != 0 {
        return None;
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
    Some(out)
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
#[cfg(test)]
mod tests {
    use super::*;
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
        assert!(decode("ordinary prose").is_none());
        assert!(decode("=AAA").is_none())
    }
    #[test]
    fn detector_heuristic_rejects_plain_alphanumeric_text() {
        let raw = "testtest";
        let bytes = decode(raw).unwrap();
        assert!(String::from_utf8(bytes).is_err());
        assert!(!raw.bytes().any(|byte| matches!(byte, b'=' | b'+' | b'/')));
    }
}

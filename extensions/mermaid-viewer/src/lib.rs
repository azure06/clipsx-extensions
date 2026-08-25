mod bindings {
    use super::MermaidViewer;
    wit_bindgen::generate!({ path: "../../sdk/wit", world: "extension" });
    export!(MermaidViewer);
}

use bindings::clipsx::extension::types::{
    ActionResult, ActionState, CompactModel, Content, Facet, GuestError, GuestErrorCode,
    OutputRepresentation, RenderModel, Representation,
};

struct MermaidViewer;

impl bindings::Guest for MermaidViewer {
    fn detect(contribution_id: String, input: Representation) -> Result<Vec<Facet>, GuestError> {
        if contribution_id != "detect-mermaid" {
            return Ok(Vec::new());
        }
        let Content::Text(text) = input.content else {
            return Ok(Vec::new());
        };
        let kind = if diagram_declaration(&text).is_some() {
            Some("mermaid")
        } else if contains_mermaid_fence(&text) {
            Some("markdown-mermaid")
        } else {
            None
        };
        Ok(kind
            .map(|kind| Facet {
                id: kind.into(),
                payload_json: serde_json::json!({ "schemaVersion": 1, "kind": kind }).to_string(),
            })
            .into_iter()
            .collect())
    }

    fn render_detail(
        _: String,
        _: Representation,
        _: Option<Facet>,
    ) -> Result<RenderModel, GuestError> {
        Err(unsupported(
            "Mermaid detail is provided by isolated package UI",
        ))
    }
    fn render_compact(
        _: String,
        _: Representation,
        _: Option<Facet>,
    ) -> Result<CompactModel, GuestError> {
        Err(unsupported("Mermaid uses the host compact summary"))
    }
    fn transform(
        _: String,
        _: Representation,
        _: String,
    ) -> Result<Vec<OutputRepresentation>, GuestError> {
        Err(unsupported("Mermaid Viewer has no transformer"))
    }
    fn run_action(
        _: String,
        _: Representation,
        _: Option<Facet>,
        _: String,
    ) -> Result<ActionResult, GuestError> {
        Err(unsupported("Mermaid Viewer has no action"))
    }
    fn action_state(
        _: String,
        _: Representation,
        _: Option<Facet>,
        _: String,
    ) -> Result<ActionState, GuestError> {
        Ok(ActionState::Hidden)
    }
}

fn contains_mermaid_fence(source: &str) -> bool {
    let mut fence: Option<(char, usize, bool)> = None;
    for line in source.lines() {
        let trimmed = line.trim_start();
        let marker = trimmed.chars().next();
        let Some(marker @ ('`' | '~')) = marker else {
            continue;
        };
        let count = trimmed.chars().take_while(|value| *value == marker).count();
        if count < 3 {
            continue;
        }
        match fence {
            None => {
                let language = trimmed[count..]
                    .trim()
                    .split_whitespace()
                    .next()
                    .unwrap_or_default();
                fence = Some((marker, count, language.eq_ignore_ascii_case("mermaid")));
            }
            Some((open_marker, open_count, is_mermaid))
                if marker == open_marker && count >= open_count =>
            {
                if is_mermaid {
                    return true;
                }
                fence = None;
            }
            _ => {}
        }
    }
    false
}

fn diagram_declaration(source: &str) -> Option<&str> {
    let mut lines = source.trim_start_matches('\u{feff}').lines().peekable();
    let first = lines.find(|line| !line.trim().is_empty())?.trim();
    let mut candidate = first;

    if candidate == "---" {
        for line in lines.by_ref() {
            if line.trim() == "---" {
                break;
            }
        }
        candidate = lines
            .find(|line| {
                let line = line.trim();
                !line.is_empty() && !line.starts_with("%%")
            })?
            .trim();
    } else if candidate.starts_with("%%") {
        candidate = lines
            .find(|line| {
                let line = line.trim();
                !line.is_empty() && !line.starts_with("%%")
            })?
            .trim();
    }

    const STARTERS: &[&str] = &[
        "flowchart",
        "graph",
        "sequenceDiagram",
        "classDiagram",
        "stateDiagram-v2",
        "stateDiagram",
        "erDiagram",
        "journey",
        "gantt",
        "pie",
        "quadrantChart",
        "requirementDiagram",
        "gitGraph",
        "C4Context",
        "C4Container",
        "C4Component",
        "C4Dynamic",
        "C4Deployment",
        "mindmap",
        "timeline",
        "zenuml",
        "sankey-beta",
        "xychart-beta",
        "block-beta",
        "packet-beta",
        "kanban",
        "architecture-beta",
        "radar-beta",
        "treemap-beta",
    ];
    STARTERS.iter().copied().find(|starter| {
        candidate == *starter
            || candidate
                .strip_prefix(*starter)
                .is_some_and(|rest| rest.chars().next().is_some_and(char::is_whitespace))
    })
}

fn unsupported(message: &str) -> GuestError {
    GuestError {
        code: GuestErrorCode::Unsupported,
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_pie_and_declarations_after_metadata() {
        assert_eq!(
            diagram_declaration("pie title NETFLIX\n  \"Looking\" : 90"),
            Some("pie")
        );
        assert_eq!(
            diagram_declaration("%%{init: { 'theme': 'neutral' }}%%\nflowchart LR\nA-->B"),
            Some("flowchart")
        );
        assert_eq!(
            diagram_declaration("---\ntitle: Example\n---\nsequenceDiagram\nA->>B: Hello"),
            Some("sequenceDiagram")
        );
    }

    #[test]
    fn rejects_ordinary_text() {
        assert_eq!(diagram_declaration("This is ordinary prose."), None);
        assert_eq!(diagram_declaration("graphical results"), None);
    }

    #[test]
    fn detects_mermaid_fences_inside_markdown() {
        assert!(contains_mermaid_fence(
            "# Architecture\n\n```mermaid\nflowchart LR\nA-->B\n```"
        ));
        assert!(!contains_mermaid_fence(
            "# Example\n\n```rust\nfn main() {}\n```"
        ));
    }
}

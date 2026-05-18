use flavor_core::{FactPayload, PendingFact, Span};

pub fn name_fact(
    key: &'static str,
    name: impl Into<String>,
    kind: &'static str,
    issue_kind: &'static str,
    span: Span,
    line: usize,
) -> PendingFact {
    PendingFact::new(
        key,
        FactPayload::new()
            .text("name", name)
            .text("kind", kind)
            .text("issue_kind", issue_kind),
    )
    .span(span)
    .line(line)
}

pub fn line_count_fact(key: &'static str, lines: usize, span: Span, line: usize) -> PendingFact {
    PendingFact::new(key, FactPayload::new().usize("lines", lines))
        .span(span)
        .line(line)
}

pub fn descriptor_block_fact(
    key: &'static str,
    content: impl Into<String>,
    start_offset: usize,
    start_line: usize,
) -> PendingFact {
    PendingFact::new(
        key,
        FactPayload::new()
            .text("content", content)
            .usize("start_offset", start_offset)
            .usize("start_line", start_line),
    )
    .line(start_line)
}

pub fn embedded_script_fact(
    key: &'static str,
    content: impl Into<String>,
    lang: impl Into<String>,
    tsx: bool,
    start_offset: usize,
    start_line: usize,
) -> PendingFact {
    PendingFact::new(
        key,
        FactPayload::new()
            .text("content", content)
            .text("lang", lang)
            .bool("tsx", tsx)
            .usize("start_offset", start_offset)
            .usize("start_line", start_line),
    )
    .line(start_line)
}

pub fn named_span_fact(
    key: &'static str,
    name: impl Into<String>,
    span: Span,
    line: usize,
) -> PendingFact {
    PendingFact::new(key, FactPayload::new().text("name", name))
        .span(span)
        .line(line)
}

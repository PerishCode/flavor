use flavor_core::{SourceText, SyntaxBuilder, SyntaxNode};
use tree_sitter::Node;

use crate::syntax_kind::RustSyntaxKind;

include!(concat!(env!("OUT_DIR"), "/rust_tree_sitter_nodes.rs"));
include!(concat!(env!("OUT_DIR"), "/rust_tree_sitter_tokens.rs"));
include!(concat!(env!("OUT_DIR"), "/rust_gap_kind.rs"));

pub fn build(root: Node<'_>, source: &SourceText) -> SyntaxNode {
    let mut builder = SyntaxBuilder::new();
    builder.start_schema_node(RustSyntaxKind::SourceFile);
    build_children_in(
        &mut builder,
        root,
        source.as_str(),
        0,
        source.as_str().len(),
    );
    builder.finish_node();
    builder.finish()
}

pub fn build_error(source: &SourceText) -> SyntaxNode {
    let mut builder = SyntaxBuilder::new();
    builder.start_schema_node(RustSyntaxKind::SourceFile);
    if !source.as_str().is_empty() {
        builder.schema_token(RustSyntaxKind::RawText, source.as_str());
    }
    builder.finish_node();
    builder.finish()
}

fn build_node(builder: &mut SyntaxBuilder, node: Node<'_>, kind: RustSyntaxKind, source: &str) {
    builder.start_schema_node(kind);
    build_children_in(builder, node, source, node.start_byte(), node.end_byte());
    builder.finish_node();
}

fn build_children_in(
    builder: &mut SyntaxBuilder,
    node: Node<'_>,
    source: &str,
    start: usize,
    end: usize,
) {
    let mut cursor = node.walk();
    let mut position = start;
    for child in node.named_children(&mut cursor) {
        push_gap(builder, source, position, child.start_byte());
        build_child(builder, child, source);
        position = child.end_byte();
    }
    push_gap(builder, source, position, end);
}

fn build_child(builder: &mut SyntaxBuilder, node: Node<'_>, source: &str) {
    if let Some(kind) = node_kind(node.kind()) {
        build_node(builder, node, kind, source);
    } else if let Some(kind) = token_kind_for_node(node.kind()) {
        push_token(builder, source, node.start_byte(), node.end_byte(), kind);
    } else if node.named_child_count() == 0 {
        push_token(
            builder,
            source,
            node.start_byte(),
            node.end_byte(),
            RustSyntaxKind::RawText,
        );
    } else {
        build_children_in(builder, node, source, node.start_byte(), node.end_byte());
    }
}

fn push_gap(builder: &mut SyntaxBuilder, source: &str, start: usize, end: usize) {
    if start >= end {
        return;
    }
    let text = &source[start..end];
    let kind = gap_kind(text);
    builder.schema_token(kind, text);
}

fn push_token(
    builder: &mut SyntaxBuilder,
    source: &str,
    start: usize,
    end: usize,
    kind: RustSyntaxKind,
) {
    if start < end {
        builder.schema_token(kind, &source[start..end]);
    }
}

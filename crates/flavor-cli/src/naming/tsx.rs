use std::{collections::BTreeSet, path::Path};

use swc_common::{sync::Lrc, SourceMap, Span};
use swc_ecma_ast::{
    ImportDecl, ImportSpecifier, JSXElementName, JSXMemberExpr, JSXObject, JSXOpeningElement,
    Module,
};
use swc_ecma_visit::{Visit, VisitWith};

use crate::{
    config::{GuardConfig, NodeKind, RuleSettings},
    model::{issue, Issue},
    rules::{
        PAYLOAD_ALLOWED_INTRINSICS, PAYLOAD_PRIMITIVE_SOURCES, TSX_NO_INTRINSICS,
        TSX_REQUIRES_PRIMITIVE,
    },
};

pub(crate) fn check_tsx_rules(
    config: &GuardConfig,
    relative: &Path,
    path: &str,
    module: &Module,
    cm: &Lrc<SourceMap>,
    line_offset: usize,
    issues: &mut Vec<Issue>,
) {
    let intrinsic_rule = config.rule(relative, NodeKind::File, TSX_NO_INTRINSICS);
    let primitive_rule = config.rule(relative, NodeKind::File, TSX_REQUIRES_PRIMITIVE);
    if !intrinsic_rule.enabled && !primitive_rule.enabled {
        return;
    }

    let mut visitor = TsxBoundaryVisitor {
        path,
        issues,
        cm: cm.clone(),
        line_offset,
        intrinsic_rule: &intrinsic_rule,
        primitive_rule: &primitive_rule,
        allowed_intrinsics: intrinsic_rule
            .string_list(PAYLOAD_ALLOWED_INTRINSICS)
            .unwrap_or_default()
            .into_iter()
            .collect(),
        primitive_sources: primitive_rule
            .string_list(PAYLOAD_PRIMITIVE_SOURCES)
            .unwrap_or_default()
            .into_iter()
            .collect(),
        named_primitives: BTreeSet::new(),
        namespace_primitives: BTreeSet::new(),
        saw_jsx: false,
        used_primitive: false,
    };
    module.visit_with(&mut visitor);
    visitor.finish();
}

struct TsxBoundaryVisitor<'a> {
    path: &'a str,
    issues: &'a mut Vec<Issue>,
    cm: Lrc<SourceMap>,
    line_offset: usize,
    intrinsic_rule: &'a RuleSettings,
    primitive_rule: &'a RuleSettings,
    allowed_intrinsics: BTreeSet<String>,
    primitive_sources: BTreeSet<String>,
    named_primitives: BTreeSet<String>,
    namespace_primitives: BTreeSet<String>,
    saw_jsx: bool,
    used_primitive: bool,
}

impl Visit for TsxBoundaryVisitor<'_> {
    fn visit_import_decl(&mut self, node: &ImportDecl) {
        if node.type_only {
            return;
        }
        let source = node.src.value.to_string_lossy().to_string();
        if !self.primitive_sources.contains(&source) {
            return;
        }
        for specifier in &node.specifiers {
            match specifier {
                ImportSpecifier::Named(named) if !named.is_type_only => {
                    self.named_primitives.insert(named.local.sym.to_string());
                }
                ImportSpecifier::Default(default) => {
                    self.named_primitives.insert(default.local.sym.to_string());
                }
                ImportSpecifier::Namespace(namespace) => {
                    self.namespace_primitives
                        .insert(namespace.local.sym.to_string());
                }
                ImportSpecifier::Named(_) => {}
            }
        }
    }

    fn visit_jsx_opening_element(&mut self, node: &JSXOpeningElement) {
        self.saw_jsx = true;
        if self.is_primitive_usage(&node.name) {
            self.used_primitive = true;
        }
        if self.intrinsic_rule.enabled {
            self.check_intrinsic(node);
        }
        node.visit_children_with(self);
    }
}

impl TsxBoundaryVisitor<'_> {
    fn finish(&mut self) {
        if !self.primitive_rule.enabled || !self.saw_jsx || self.used_primitive {
            return;
        }
        let sources = if self.primitive_sources.is_empty() {
            "configured primitive sources".to_string()
        } else {
            self.primitive_sources
                .iter()
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        };
        self.issues.push(issue(
            self.primitive_rule.severity,
            self.primitive_rule.id,
            self.path,
            None,
            format!("component JSX does not compose a primitive from {sources}"),
        ));
    }

    fn check_intrinsic(&mut self, node: &JSXOpeningElement) {
        let Some(name) = intrinsic_name(&node.name) else {
            return;
        };
        if self.allowed_intrinsics.contains(&name) {
            return;
        }
        self.issues.push(issue(
            self.intrinsic_rule.severity,
            self.intrinsic_rule.id,
            self.path,
            Some(self.line_for(node.span)),
            format!("JSX intrinsic element `<{name}>` is not allowed in this boundary"),
        ));
    }

    fn is_primitive_usage(&self, name: &JSXElementName) -> bool {
        match name {
            JSXElementName::Ident(ident) => self.named_primitives.contains(ident.sym.as_ref()),
            JSXElementName::JSXMemberExpr(member) => {
                root_jsx_object(member).is_some_and(|root| self.namespace_primitives.contains(root))
            }
            JSXElementName::JSXNamespacedName(_) => false,
        }
    }

    fn line_for(&self, span: Span) -> usize {
        self.cm.lookup_char_pos(span.lo()).line + self.line_offset
    }
}

fn intrinsic_name(name: &JSXElementName) -> Option<String> {
    match name {
        JSXElementName::Ident(ident) => {
            let name = ident.sym.as_ref();
            name.chars()
                .next()
                .is_some_and(|ch| ch.is_ascii_lowercase())
                .then(|| name.to_string())
        }
        JSXElementName::JSXNamespacedName(namespaced) => {
            Some(format!("{}:{}", namespaced.ns.sym, namespaced.name.sym))
        }
        JSXElementName::JSXMemberExpr(_) => None,
    }
}

fn root_jsx_object(member: &JSXMemberExpr) -> Option<&str> {
    match &member.obj {
        JSXObject::Ident(ident) => Some(ident.sym.as_ref()),
        JSXObject::JSXMemberExpr(member) => root_jsx_object(member),
    }
}

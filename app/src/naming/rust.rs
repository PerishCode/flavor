use std::path::Path;

use syn::{
    spanned::Spanned,
    visit::{self, Visit},
    ExprMatch, ImplItemFn, ItemFn, PatIdent, TraitItemFn,
};

use crate::{
    config::{GuardConfig, NodeKind, RuleSettings},
    model::{issue, Issue},
    naming::check_name,
    rules::{DISPATCH_BRANCH_TOO_LONG, NAMING_TOO_MANY_WORDS, PAYLOAD_MAX_BRANCH_LINES},
};

pub(crate) fn check_rust_names(
    config: &GuardConfig,
    relative: &Path,
    path: &str,
    source: &str,
    issues: &mut Vec<Issue>,
    parse_rule: &RuleSettings,
) {
    match syn::parse_file(source) {
        Ok(file) => {
            let rule = config.rule(relative, NodeKind::File, NAMING_TOO_MANY_WORDS);
            let dispatch_rule = config.rule(relative, NodeKind::File, DISPATCH_BRANCH_TOO_LONG);
            let mut visitor = RustNameVisitor {
                path,
                issues,
                rule: &rule,
                dispatch_rule: &dispatch_rule,
            };
            visitor.visit_file(&file);
        }
        Err(error) => {
            if parse_rule.enabled {
                issues.push(issue(
                    parse_rule.severity,
                    parse_rule.id,
                    path,
                    Some(error.span().start().line),
                    format!("failed to parse Rust source: {error}"),
                ));
            }
        }
    }
}

struct RustNameVisitor<'a> {
    path: &'a str,
    issues: &'a mut Vec<Issue>,
    rule: &'a RuleSettings,
    dispatch_rule: &'a RuleSettings,
}

impl<'ast> Visit<'ast> for RustNameVisitor<'_> {
    fn visit_expr_match(&mut self, node: &'ast ExprMatch) {
        if self.dispatch_rule.enabled {
            let max_lines = self
                .dispatch_rule
                .usize(PAYLOAD_MAX_BRANCH_LINES)
                .unwrap_or(24);
            for arm in &node.arms {
                let span = arm.body.span();
                let start = span.start().line;
                let end = span.end().line;
                let lines = end.saturating_sub(start) + 1;
                if lines > max_lines {
                    self.issues.push(issue(
                        self.dispatch_rule.severity,
                        self.dispatch_rule.id,
                        self.path,
                        Some(start),
                        format!("match arm body spans {lines} lines; max is {max_lines}"),
                    ));
                }
            }
        }

        visit::visit_expr_match(self, node);
    }

    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        check_name(
            self.issues,
            self.rule,
            self.path,
            node.sig.ident.span().start().line,
            "function",
            &node.sig.ident.to_string(),
        );
        visit::visit_item_fn(self, node);
    }

    fn visit_impl_item_fn(&mut self, node: &'ast ImplItemFn) {
        check_name(
            self.issues,
            self.rule,
            self.path,
            node.sig.ident.span().start().line,
            "method",
            &node.sig.ident.to_string(),
        );
        visit::visit_impl_item_fn(self, node);
    }

    fn visit_trait_item_fn(&mut self, node: &'ast TraitItemFn) {
        check_name(
            self.issues,
            self.rule,
            self.path,
            node.sig.ident.span().start().line,
            "method",
            &node.sig.ident.to_string(),
        );
        visit::visit_trait_item_fn(self, node);
    }

    fn visit_pat_ident(&mut self, node: &'ast PatIdent) {
        let name = node.ident.to_string();
        if name != "self" {
            check_name(
                self.issues,
                self.rule,
                self.path,
                node.ident.span().start().line,
                "binding",
                &name,
            );
        }
        visit::visit_pat_ident(self, node);
    }
}

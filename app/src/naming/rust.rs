use std::path::Path;

use syn::{
    visit::{self, Visit},
    ImplItemFn, ItemFn, PatIdent, TraitItemFn,
};

use crate::{
    config::{GuardConfig, NodeKind, RuleSettings},
    model::{issue, Issue},
    naming::check_name,
    rules::NAMING_TOO_MANY_WORDS,
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
            let mut visitor = RustNameVisitor {
                path,
                issues,
                rule: &rule,
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
}

impl<'ast> Visit<'ast> for RustNameVisitor<'_> {
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

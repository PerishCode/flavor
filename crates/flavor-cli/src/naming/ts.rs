use std::path::Path;

use flavor_compiler_core::{Diagnostic, SourceText};
use flavor_compiler_vue::{sfc::VueSfcBlock, VueCompilerConfig};
use swc_common::{sync::Lrc, FileName, SourceMap, Span};
use swc_ecma_ast::{
    ClassMethod, FnDecl, FnExpr, Ident, MethodProp, Param, Pat, PropName, SwitchCase, VarDeclarator,
};
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax, TsSyntax};
use swc_ecma_visit::{Visit, VisitWith};

use crate::{
    config::{GuardConfig, NodeKind, RuleSettings},
    model::{issue, Issue},
    naming::check_name,
    rules::{
        DISPATCH_BRANCH_TOO_LONG, NAMING_TOO_MANY_WORDS, PAYLOAD_MAX_BRANCH_LINES, VUE_PARSE_ERROR,
    },
};

pub(crate) fn check_ts_names(
    config: &GuardConfig,
    relative: &Path,
    path: &str,
    source: &str,
    issues: &mut Vec<Issue>,
    parse_rule: &RuleSettings,
) {
    let extension = Path::new(path).extension().and_then(|value| value.to_str());
    let scripts = if extension == Some("vue") {
        let vue_parse_rule = config.rule(relative, NodeKind::File, VUE_PARSE_ERROR);
        vue_script_blocks(path, source, issues, &vue_parse_rule)
    } else {
        vec![TsScriptBlock {
            content: source.to_string(),
            start_line: 0,
            tsx: extension == Some("tsx"),
        }]
    };
    check_ts_script_blocks(config, relative, path, scripts, issues, parse_rule);
}

pub(crate) fn check_ts_script_blocks(
    config: &GuardConfig,
    relative: &Path,
    path: &str,
    scripts: Vec<TsScriptBlock>,
    issues: &mut Vec<Issue>,
    parse_rule: &RuleSettings,
) {
    let name_rule = config.rule(relative, NodeKind::File, NAMING_TOO_MANY_WORDS);
    let dispatch_rule = config.rule(relative, NodeKind::File, DISPATCH_BRANCH_TOO_LONG);
    for script in scripts {
        check_script(path, script, issues, &name_rule, &dispatch_rule, parse_rule);
    }
}

pub(crate) struct TsScriptBlock {
    pub(crate) content: String,
    pub(crate) start_line: usize,
    pub(crate) tsx: bool,
}

fn check_script(
    path: &str,
    script: TsScriptBlock,
    issues: &mut Vec<Issue>,
    name_rule: &RuleSettings,
    dispatch_rule: &RuleSettings,
    parse_rule: &RuleSettings,
) {
    let cm: Lrc<SourceMap> = Default::default();
    let fm = cm.new_source_file(FileName::Custom(path.to_string()).into(), script.content);
    let lexer = Lexer::new(
        Syntax::Typescript(TsSyntax {
            tsx: script.tsx,
            decorators: true,
            ..Default::default()
        }),
        Default::default(),
        StringInput::from(&*fm),
        None,
    );
    let mut parser = Parser::new_from(lexer);
    let module = match parser.parse_module() {
        Ok(module) => module,
        Err(error) => {
            push_parse_issue(issues, parse_rule, path, format!("{error:?}"));
            return;
        }
    };

    for error in parser.take_errors() {
        push_parse_issue(issues, parse_rule, path, format!("{error:?}"));
    }

    let mut visitor = TsNameVisitor {
        path,
        issues,
        cm,
        line_offset: script.start_line,
        rule: name_rule,
        dispatch_rule,
    };
    module.visit_with(&mut visitor);
}

fn push_parse_issue(issues: &mut Vec<Issue>, rule: &RuleSettings, path: &str, error: String) {
    if !rule.enabled {
        return;
    }

    issues.push(issue(
        rule.severity,
        rule.id,
        path,
        None,
        format!("failed to parse TypeScript source: {error}"),
    ));
}

fn vue_script_blocks(
    path: &str,
    source: &str,
    issues: &mut Vec<Issue>,
    parse_rule: &RuleSettings,
) -> Vec<TsScriptBlock> {
    let source_text = SourceText::new(path, source);
    let line_index = source_text.line_index();
    let output = flavor_compiler_vue::run(source_text, VueCompilerConfig::default());
    for diagnostic in output.diagnostics {
        let line = diagnostic
            .span
            .map(|span| line_index.position(span.start).line as usize);
        push_vue_parse_issue(issues, parse_rule, path, diagnostic, line);
    }

    let mut scripts = Vec::new();
    if let Some(block) = output.descriptor.script {
        push_vue_script_block(&mut scripts, block);
    }
    if let Some(block) = output.descriptor.script_setup {
        push_vue_script_block(&mut scripts, block);
    }
    scripts
}

fn push_vue_script_block(scripts: &mut Vec<TsScriptBlock>, block: VueSfcBlock) {
    if block.content.trim().is_empty() {
        return;
    }
    let Some(tsx) = vue_script_tsx(&block) else {
        return;
    };
    scripts.push(TsScriptBlock {
        content: block.content,
        start_line: block.start_line,
        tsx,
    });
}

fn vue_script_tsx(block: &VueSfcBlock) -> Option<bool> {
    let lang = block
        .attrs
        .get("lang")
        .and_then(|value| value.as_deref())
        .map(|value| value.to_ascii_lowercase());
    match lang.as_deref() {
        None | Some("js" | "ts") => Some(false),
        Some("jsx" | "tsx") => Some(true),
        _ => None,
    }
}

fn push_vue_parse_issue(
    issues: &mut Vec<Issue>,
    rule: &RuleSettings,
    path: &str,
    diagnostic: Diagnostic,
    line: Option<usize>,
) {
    if !rule.enabled {
        return;
    }

    issues.push(issue(
        rule.severity,
        rule.id,
        path,
        line,
        format!("failed to parse Vue SFC source: {}", diagnostic.message),
    ));
}

struct TsNameVisitor<'a> {
    path: &'a str,
    issues: &'a mut Vec<Issue>,
    cm: Lrc<SourceMap>,
    line_offset: usize,
    rule: &'a RuleSettings,
    dispatch_rule: &'a RuleSettings,
}

impl Visit for TsNameVisitor<'_> {
    fn visit_switch_case(&mut self, node: &SwitchCase) {
        if self.dispatch_rule.enabled {
            let max_lines = self
                .dispatch_rule
                .usize(PAYLOAD_MAX_BRANCH_LINES)
                .unwrap_or(24);
            let start = self.line_for(node.span);
            let end = self.cm.lookup_char_pos(node.span.hi()).line + self.line_offset;
            let lines = end.saturating_sub(start) + 1;
            if lines > max_lines {
                self.issues.push(issue(
                    self.dispatch_rule.severity,
                    self.dispatch_rule.id,
                    self.path,
                    Some(start),
                    format!("switch case spans {lines} lines; max is {max_lines}"),
                ));
            }
        }

        node.visit_children_with(self);
    }

    fn visit_fn_decl(&mut self, node: &FnDecl) {
        self.check_ident("function", &node.ident);
        node.visit_children_with(self);
    }

    fn visit_fn_expr(&mut self, node: &FnExpr) {
        if let Some(ident) = &node.ident {
            self.check_ident("function", ident);
        }
        node.visit_children_with(self);
    }

    fn visit_var_declarator(&mut self, node: &VarDeclarator) {
        self.check_pat("binding", &node.name);
        node.visit_children_with(self);
    }

    fn visit_param(&mut self, node: &Param) {
        self.check_pat("parameter", &node.pat);
        node.visit_children_with(self);
    }

    fn visit_class_method(&mut self, node: &ClassMethod) {
        self.check_prop_name("method", &node.key);
        node.visit_children_with(self);
    }

    fn visit_method_prop(&mut self, node: &MethodProp) {
        self.check_prop_name("method", &node.key);
        node.visit_children_with(self);
    }
}

impl TsNameVisitor<'_> {
    fn check_pat(&mut self, kind: &str, pat: &Pat) {
        match pat {
            Pat::Ident(binding) => self.check_ident(kind, &binding.id),
            Pat::Array(array) => {
                for elem in array.elems.iter().flatten() {
                    self.check_pat(kind, elem);
                }
            }
            Pat::Rest(rest) => self.check_pat(kind, &rest.arg),
            Pat::Object(object) => {
                for prop in &object.props {
                    prop.visit_with(self);
                }
            }
            Pat::Assign(assign) => self.check_pat(kind, &assign.left),
            Pat::Invalid(_) | Pat::Expr(_) => {}
        }
    }

    fn check_ident(&mut self, kind: &str, ident: &Ident) {
        check_name(
            self.issues,
            self.rule,
            self.path,
            self.line_for(ident.span),
            kind,
            ident.sym.as_ref(),
        );
    }

    fn check_prop_name(&mut self, kind: &str, prop: &PropName) {
        if let PropName::Ident(ident) = prop {
            check_name(
                self.issues,
                self.rule,
                self.path,
                self.line_for(ident.span),
                kind,
                ident.sym.as_ref(),
            );
        }
    }

    fn line_for(&self, span: Span) -> usize {
        self.cm.lookup_char_pos(span.lo()).line + self.line_offset
    }
}

use crate::{
    source::{G4Source, RawAstSchema, RawAstSymbol, RawAstSymbolKind},
    GrammarError, GrammarMetadata,
};

impl RawAstSchema {
    pub fn render_rust_enum(
        &self,
        enum_name: &str,
        raw_kind_path: &str,
    ) -> Result<String, Vec<GrammarError>> {
        self.render_rust_enum_fallback(enum_name, raw_kind_path, None)
    }

    pub fn render_rust_enum_fallback(
        &self,
        enum_name: &str,
        raw_kind_path: &str,
        fallback_variant: Option<&str>,
    ) -> Result<String, Vec<GrammarError>> {
        let errors = self.render_errors(enum_name, raw_kind_path, fallback_variant);
        if !errors.is_empty() {
            return Err(errors);
        }

        let mut output = String::new();
        self.render_enum(enum_name, &mut output);
        self.render_from(enum_name, raw_kind_path, &mut output);
        self.render_schema_trait(enum_name, raw_kind_path, &mut output);
        self.render_categories(enum_name, raw_kind_path, &mut output);
        if let Some(fallback) = fallback_variant {
            self.render_fallback(enum_name, raw_kind_path, fallback, &mut output);
        }
        Ok(output)
    }

    pub fn render_rust_node_adapter(
        &self,
        enum_name: &str,
        function_name: &str,
        metadata: &GrammarMetadata,
        backend: &str,
    ) -> Result<String, Vec<GrammarError>> {
        let mut errors = adapter_errors(enum_name, function_name, backend);
        let mut mappings = Vec::new();
        let prefix = format!("{backend}:");
        if let Some(nodes) = metadata.section("nodes") {
            for entry in &nodes.entries {
                let Some(binding) = entry.value.strip_prefix(&prefix) else {
                    continue;
                };
                if let Some(variant) = self.adapter_variant(
                    &entry.key,
                    RawAstSymbolKind::Node,
                    entry.line,
                    &mut errors,
                ) {
                    mappings.push((binding.to_string(), variant));
                }
            }
        }
        render_option_adapter(enum_name, function_name, mappings, errors)
    }

    pub fn render_rust_token_adapter(
        &self,
        enum_name: &str,
        function_name: &str,
        sources: &[G4Source],
        backend: &str,
    ) -> Result<String, Vec<GrammarError>> {
        let mut errors = adapter_errors(enum_name, function_name, backend);
        let mut mappings = Vec::new();
        for rule in sources.iter().flat_map(|source| &source.lexer_tokens) {
            for binding in rule
                .bindings
                .iter()
                .filter(|binding| binding.backend == backend)
            {
                if let Some(variant) = self.adapter_variant(
                    &rule.name,
                    RawAstSymbolKind::Token,
                    rule.line,
                    &mut errors,
                ) {
                    mappings.push((binding.name.clone(), variant));
                }
            }
        }
        render_option_adapter(enum_name, function_name, mappings, errors)
    }

    pub fn render_rust_gap_adapter(
        &self,
        enum_name: &str,
        function_name: &str,
        sources: &[G4Source],
        whitespace_token: &str,
        fallback_token: &str,
    ) -> Result<String, Vec<GrammarError>> {
        let mut errors = adapter_errors(enum_name, function_name, "literal");
        let whitespace =
            self.adapter_variant(whitespace_token, RawAstSymbolKind::Token, 1, &mut errors);
        let fallback =
            self.adapter_variant(fallback_token, RawAstSymbolKind::Token, 1, &mut errors);
        let mut mappings = Vec::new();
        for rule in sources.iter().flat_map(|source| &source.lexer_tokens) {
            let Some(literal) = simple_literal(&rule.body) else {
                continue;
            };
            if let Some(variant) =
                self.adapter_variant(&rule.name, RawAstSymbolKind::Token, rule.line, &mut errors)
            {
                mappings.push((literal, variant));
            }
        }
        let (Some(whitespace), Some(fallback)) = (whitespace, fallback) else {
            return Err(errors);
        };
        render_gap_adapter(
            enum_name,
            function_name,
            whitespace,
            fallback,
            mappings,
            errors,
        )
    }

    fn adapter_variant(
        &self,
        name: &str,
        kind: RawAstSymbolKind,
        line: usize,
        errors: &mut Vec<GrammarError>,
    ) -> Option<String> {
        let Some(symbol) = self.symbol(name) else {
            errors.push(GrammarError {
                line,
                message: format!("raw AST schema is missing symbol `{name}`"),
            });
            return None;
        };
        if symbol.kind != kind {
            errors.push(GrammarError {
                line,
                message: format!("raw AST symbol `{name}` has wrong category"),
            });
            return None;
        }
        Some(symbol.variant.clone())
    }

    fn render_errors(
        &self,
        enum_name: &str,
        raw_kind_path: &str,
        fallback_variant: Option<&str>,
    ) -> Vec<GrammarError> {
        let mut errors = Vec::new();
        if !valid_rust_type_ident(enum_name) {
            errors.push(GrammarError {
                line: 1,
                message: format!("invalid Rust enum name `{enum_name}`"),
            });
        }
        if raw_kind_path.trim().is_empty() {
            errors.push(GrammarError {
                line: 1,
                message: "missing Rust raw kind path".to_string(),
            });
        }
        if let Some(fallback) = fallback_variant {
            if self.symbols.iter().all(|symbol| symbol.variant != fallback) {
                errors.push(GrammarError {
                    line: 1,
                    message: format!("missing fallback Rust variant `{fallback}`"),
                });
            }
        }
        errors
    }

    fn render_enum(&self, enum_name: &str, output: &mut String) {
        output.push_str("#[repr(u16)]\n");
        output.push_str("#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]\n");
        output.push_str(&format!("pub enum {enum_name} {{\n"));
        for symbol in &self.symbols {
            output.push_str(&format!("    {} = {},\n", symbol.variant, symbol.raw_kind));
        }
        output.push_str("}\n\n");
    }

    fn render_from(&self, enum_name: &str, raw_kind_path: &str, output: &mut String) {
        output.push_str(&format!("impl From<{enum_name}> for {raw_kind_path} {{\n"));
        output.push_str(&format!(
            "    fn from(kind: {enum_name}) -> Self {{\n        Self(kind as u16)\n    }}\n"
        ));
        output.push_str("}\n");
    }

    fn render_categories(&self, enum_name: &str, raw_kind_path: &str, output: &mut String) {
        output.push_str(&format!("\nimpl {enum_name} {{\n"));
        output.push_str("    pub fn is_node(self) -> bool {\n");
        output.push_str("        Self::raw_is_node(self.into())\n");
        output.push_str("    }\n\n");
        output.push_str("    pub fn is_token(self) -> bool {\n");
        output.push_str("        Self::raw_is_token(self.into())\n");
        output.push_str("    }\n\n");
        output.push_str(&format!(
            "    pub fn raw_is_node(kind: {raw_kind_path}) -> bool {{\n"
        ));
        output.push_str(&format!(
            "        {}\n",
            rust_raw_match(&self.symbols, RawAstSymbolKind::Node)
        ));
        output.push_str("    }\n\n");
        output.push_str(&format!(
            "    pub fn raw_is_token(kind: {raw_kind_path}) -> bool {{\n"
        ));
        output.push_str(&format!(
            "        {}\n",
            rust_raw_match(&self.symbols, RawAstSymbolKind::Token)
        ));
        output.push_str("    }\n");
        output.push_str("}\n");
    }

    fn render_schema_trait(&self, enum_name: &str, raw_kind_path: &str, output: &mut String) {
        output.push_str(&format!(
            "\nimpl flavor_core::SyntaxKindSchema for {enum_name} {{\n"
        ));
        output.push_str(&format!(
            "    fn raw_is_node(kind: {raw_kind_path}) -> bool {{\n"
        ));
        output.push_str("        Self::raw_is_node(kind)\n");
        output.push_str("    }\n\n");
        output.push_str(&format!(
            "    fn raw_is_token(kind: {raw_kind_path}) -> bool {{\n"
        ));
        output.push_str("        Self::raw_is_token(kind)\n");
        output.push_str("    }\n");
        output.push_str("}\n");
    }

    fn render_fallback(
        &self,
        enum_name: &str,
        raw_kind_path: &str,
        fallback: &str,
        output: &mut String,
    ) {
        output.push_str(&format!("\nimpl {enum_name} {{\n"));
        output.push_str(&format!(
            "    pub fn from_raw(kind: {raw_kind_path}) -> Self {{\n        match kind.0 {{\n"
        ));
        for symbol in &self.symbols {
            output.push_str(&format!(
                "            {} => Self::{},\n",
                symbol.raw_kind, symbol.variant
            ));
        }
        output.push_str(&format!("            _ => Self::{fallback},\n"));
        output.push_str("        }\n    }\n}\n");
    }
}

fn rust_raw_match(symbols: &[RawAstSymbol], kind: RawAstSymbolKind) -> String {
    let Some((first, last)) = raw_range(symbols, kind) else {
        return "false".to_string();
    };
    if first == last {
        format!("kind.0 == {first}")
    } else {
        format!("matches!(kind.0, {first}..={last})")
    }
}

fn adapter_errors(enum_name: &str, function_name: &str, backend: &str) -> Vec<GrammarError> {
    let mut errors = Vec::new();
    if !valid_rust_type_ident(enum_name) {
        errors.push(GrammarError {
            line: 1,
            message: format!("invalid Rust enum name `{enum_name}`"),
        });
    }
    if !valid_rust_value_ident(function_name) {
        errors.push(GrammarError {
            line: 1,
            message: format!("invalid Rust function name `{function_name}`"),
        });
    }
    if backend.trim().is_empty() {
        errors.push(GrammarError {
            line: 1,
            message: "missing adapter backend".to_string(),
        });
    }
    errors
}

fn render_option_adapter(
    enum_name: &str,
    function_name: &str,
    mut mappings: Vec<(String, String)>,
    errors: Vec<GrammarError>,
) -> Result<String, Vec<GrammarError>> {
    if !errors.is_empty() {
        return Err(errors);
    }
    mappings.sort();
    let mut output = String::new();
    output.push_str(&format!(
        "fn {function_name}(kind: &str) -> Option<{enum_name}> {{\n"
    ));
    output.push_str("    match kind {\n");
    for (binding, variant) in mappings {
        output.push_str(&format!(
            "        {binding:?} => Some({enum_name}::{variant}),\n"
        ));
    }
    output.push_str("        _ => None,\n");
    output.push_str("    }\n");
    output.push_str("}\n");
    Ok(output)
}

fn render_gap_adapter(
    enum_name: &str,
    function_name: &str,
    whitespace: String,
    fallback: String,
    mut mappings: Vec<(String, String)>,
    errors: Vec<GrammarError>,
) -> Result<String, Vec<GrammarError>> {
    if !errors.is_empty() {
        return Err(errors);
    }
    mappings.sort();
    let mut output = String::new();
    output.push_str(&format!(
        "fn {function_name}(text: &str) -> {enum_name} {{\n"
    ));
    output.push_str("    if text.chars().all(char::is_whitespace) {\n");
    output.push_str(&format!("        return {enum_name}::{whitespace};\n"));
    output.push_str("    }\n");
    output.push_str("    match text.trim() {\n");
    for (literal, variant) in mappings {
        output.push_str(&format!("        {literal:?} => {enum_name}::{variant},\n"));
    }
    output.push_str(&format!("        _ => {enum_name}::{fallback},\n"));
    output.push_str("    }\n");
    output.push_str("}\n");
    Ok(output)
}

fn simple_literal(body: &str) -> Option<String> {
    let body = body.trim();
    let rest = body.strip_prefix('\'')?;
    let (literal, rest) = rest.split_once('\'')?;
    if rest.trim().is_empty() {
        Some(literal.to_string())
    } else {
        None
    }
}

fn raw_range(symbols: &[RawAstSymbol], kind: RawAstSymbolKind) -> Option<(u16, u16)> {
    let mut raw_kinds = symbols
        .iter()
        .filter(|symbol| symbol.kind == kind)
        .map(|symbol| symbol.raw_kind);
    let first = raw_kinds.next()?;
    let last = raw_kinds.fold(first, |_, raw_kind| raw_kind);
    Some((first, last))
}

fn valid_rust_type_ident(value: &str) -> bool {
    let mut chars = value.chars();
    chars
        .next()
        .is_some_and(|ch| ch.is_ascii_alphabetic() || ch == '_')
        && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

fn valid_rust_value_ident(value: &str) -> bool {
    let mut chars = value.chars();
    chars
        .next()
        .is_some_and(|ch| ch.is_ascii_lowercase() || ch == '_')
        && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

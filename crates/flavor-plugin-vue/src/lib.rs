mod facts;
pub mod model;
pub mod plugin;
mod sfc;
pub mod state;
mod style;
mod template;
mod visit;

use flavor_core::{Diagnostic, SourceText, Span};

pub use model::{
    TemplateAst, VueAnalysisOutput, VueFacts, VueSfcBlock, VueSfcDescriptor, VueSfcError,
    VueTemplateDirectiveClass, VueTemplateDirectiveFact, VueTemplateElementFact,
    VueTemplateExpressionFact,
};
pub use state::{TemplateConfig, VuePluginConfig, VuePluginState};

#[derive(Debug, Clone)]
pub struct VuePluginAnalyzer {
    state: VuePluginState,
}

impl VuePluginAnalyzer {
    pub fn new(config: VuePluginConfig) -> Self {
        Self {
            state: VuePluginState::new(config),
        }
    }

    pub fn run(&mut self, source: SourceText) -> VueAnalysisOutput {
        let descriptor = sfc::parse(&source, self.state.config());
        let mut diagnostics: Vec<Diagnostic> = descriptor
            .errors
            .iter()
            .map(|error| {
                Diagnostic::error_code(
                    span_for_line(&source, error.line),
                    "vue/parse/error",
                    error.message.clone(),
                )
            })
            .collect();
        let template = if self.state.config().template.ast {
            descriptor.template.as_ref().map(|block| {
                let ast = template::parse_template(&block.content);
                diagnostics.extend(
                    ast.diagnostics()
                        .iter()
                        .map(|diagnostic| offset_diagnostic(diagnostic, block.start_offset)),
                );
                if self.state.config().template.expressions {
                    diagnostics.extend(
                        template::validate_expressions(&ast)
                            .into_iter()
                            .map(|diagnostic| offset_diagnostic(&diagnostic, block.start_offset)),
                    );
                }
                ast
            })
        } else {
            None
        };
        let facts = facts::collect(&descriptor, template.as_ref());
        VueAnalysisOutput {
            source,
            descriptor,
            template,
            facts,
            diagnostics,
        }
    }
}

pub fn run(source: SourceText, config: VuePluginConfig) -> VueAnalysisOutput {
    VuePluginAnalyzer::new(config).run(source)
}

fn span_for_line(source: &SourceText, line: usize) -> Option<Span> {
    let line = u32::try_from(line).ok()?;
    let offset = source.line_index().line_start(line)?;
    Some(Span::new(offset, offset))
}

fn offset_diagnostic(diagnostic: &Diagnostic, offset: usize) -> Diagnostic {
    let offset = u32::try_from(offset).unwrap_or(u32::MAX);
    Diagnostic {
        severity: diagnostic.severity,
        code: diagnostic.code.clone(),
        span: diagnostic.span.map(|span| {
            Span::new(
                span.start.saturating_add(offset),
                span.end.saturating_add(offset),
            )
        }),
        message: diagnostic.message.clone(),
    }
}

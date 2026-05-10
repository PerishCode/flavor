pub mod descriptor;
pub mod facts;
pub mod markup;
pub mod state;

use flavor_compiler_core::{Diagnostic, LineIndex, SourceText, Span};

pub use descriptor::{SvelteBlock, SvelteDescriptor, SvelteMarkup};
pub use facts::SvelteFacts;
pub use markup::{SvelteMarkupAst, SvelteMarkupKind};
pub use state::{SvelteCompilerConfig, SvelteCompilerState};

#[derive(Debug, Clone)]
pub struct SvelteCompileOutput {
    pub source: SourceText,
    pub descriptor: SvelteDescriptor,
    pub markup: Option<SvelteMarkupAst>,
    pub facts: SvelteFacts,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone)]
pub struct SvelteCompiler {
    state: SvelteCompilerState,
}

impl SvelteCompiler {
    pub fn new(config: SvelteCompilerConfig) -> Self {
        Self {
            state: SvelteCompilerState::new(config),
        }
    }

    pub fn run(&mut self, source: SourceText) -> SvelteCompileOutput {
        let descriptor = descriptor::parse(&source, self.state.config());
        let mut diagnostics: Vec<Diagnostic> = descriptor
            .errors
            .iter()
            .map(|error| {
                Diagnostic::error(span_for_line(&source, error.line), error.message.clone())
            })
            .collect();
        let markup = if self.state.config().markup {
            let ast = markup::parse_markup(&descriptor.markup.content);
            diagnostics.extend(ast.diagnostics().iter().map(|diagnostic| {
                offset_markup_diagnostic(diagnostic, &source, &descriptor.markup.content)
            }));
            if self.state.config().expressions {
                diagnostics.extend(markup::validate_expressions(&ast).iter().map(|diagnostic| {
                    offset_markup_diagnostic(diagnostic, &source, &descriptor.markup.content)
                }));
            }
            Some(ast)
        } else {
            None
        };
        let facts = facts::collect(&descriptor, markup.as_ref());
        SvelteCompileOutput {
            source,
            descriptor,
            markup,
            facts,
            diagnostics,
        }
    }
}

pub fn run(source: SourceText, config: SvelteCompilerConfig) -> SvelteCompileOutput {
    SvelteCompiler::new(config).run(source)
}

fn span_for_line(source: &SourceText, line: usize) -> Option<Span> {
    let line = u32::try_from(line).ok()?;
    let offset = source.line_index().line_start(line)?;
    Some(Span::new(offset, offset))
}

fn offset_markup_diagnostic(
    diagnostic: &Diagnostic,
    source: &SourceText,
    markup_source: &str,
) -> Diagnostic {
    let markup_index = LineIndex::new(markup_source);
    let source_index = source.line_index();
    Diagnostic {
        severity: diagnostic.severity,
        span: diagnostic.span.map(|span| {
            Span::new(
                map_markup_offset(span.start, &markup_index, &source_index),
                map_markup_offset(span.end, &markup_index, &source_index),
            )
        }),
        message: diagnostic.message.clone(),
    }
}

fn map_markup_offset(offset: u32, markup_index: &LineIndex, source_index: &LineIndex) -> u32 {
    let position = markup_index.position(offset);
    source_index
        .line_start(position.line)
        .map(|line_start| line_start.saturating_add(position.column.saturating_sub(1)))
        .unwrap_or(offset)
}

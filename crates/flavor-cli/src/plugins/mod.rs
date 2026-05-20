mod filesystem;
mod g4;
pub(crate) mod helper;
mod language;
mod product;

use std::{
    collections::{BTreeSet, VecDeque},
    path::Path,
};

pub(crate) use product::ProductSet;

use crate::{
    config::{GuardConfig, SourceKind},
    model::Issue,
};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum ScopeKind {
    FilePath,
    SourceFile,
    SourceDirectory,
    DirectoryChildren,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum SourceScope {
    Any,
    Kind(SourceKind),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct ScopeDecl {
    pub(crate) kind: ScopeKind,
    pub(crate) source: SourceScope,
}

impl ScopeDecl {
    pub(crate) const fn file_path() -> Self {
        Self {
            kind: ScopeKind::FilePath,
            source: SourceScope::Any,
        }
    }

    pub(crate) const fn source_file(kind: SourceKind) -> Self {
        Self {
            kind: ScopeKind::SourceFile,
            source: SourceScope::Kind(kind),
        }
    }

    pub(crate) const fn any_source_file() -> Self {
        Self {
            kind: ScopeKind::SourceFile,
            source: SourceScope::Any,
        }
    }

    pub(crate) const fn source_directory() -> Self {
        Self {
            kind: ScopeKind::SourceDirectory,
            source: SourceScope::Any,
        }
    }

    pub(crate) const fn directory_children() -> Self {
        Self {
            kind: ScopeKind::DirectoryChildren,
            source: SourceScope::Any,
        }
    }

    fn matches(self, scope: Scope<'_>) -> bool {
        if self.kind != scope.kind {
            return false;
        }
        match self.source {
            SourceScope::Any => true,
            SourceScope::Kind(expected) => scope.source_kind() == Some(expected),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct GrammarUse {
    pub(crate) scope: ScopeKind,
    pub(crate) grammar_id: &'static str,
    pub(crate) entrypoint: &'static str,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct FactUse {
    pub(crate) grammar_id: &'static str,
    pub(crate) key: &'static str,
    pub(crate) contains: &'static [&'static str],
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct PluginManifest {
    pub(crate) id: &'static str,
    pub(crate) scopes: &'static [ScopeDecl],
    pub(crate) grammars: &'static [GrammarUse],
    pub(crate) facts: &'static [FactUse],
    pub(crate) rules: &'static [&'static str],
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Scope<'a> {
    kind: ScopeKind,
    data: ScopeData<'a>,
}

impl<'a> Scope<'a> {
    pub(crate) fn file_path(relative: &'a Path) -> Self {
        Self {
            kind: ScopeKind::FilePath,
            data: ScopeData::FilePath { relative },
        }
    }

    pub(crate) fn source_directory(relative: &'a Path) -> Self {
        Self {
            kind: ScopeKind::SourceDirectory,
            data: ScopeData::SourceDirectory { relative },
        }
    }

    pub(crate) fn directory_children(
        relative: &'a Path,
        children: &'a BTreeSet<String>,
        source_child_count: usize,
    ) -> Self {
        Self {
            kind: ScopeKind::DirectoryChildren,
            data: ScopeData::DirectoryChildren {
                relative,
                children,
                source_child_count,
            },
        }
    }

    pub(crate) fn source_file(
        relative: &'a Path,
        path: &'a str,
        source: &'a str,
        kind: SourceKind,
    ) -> Self {
        Self {
            kind: ScopeKind::SourceFile,
            data: ScopeData::SourceFile {
                relative,
                path,
                source,
                kind,
            },
        }
    }

    pub(crate) fn source_kind(self) -> Option<SourceKind> {
        match self.data {
            ScopeData::SourceFile { kind, .. } => Some(kind),
            _ => None,
        }
    }

    pub(crate) fn file_path_data(self) -> Option<FilePathScope<'a>> {
        match self.data {
            ScopeData::FilePath { relative } => Some(FilePathScope { relative }),
            _ => None,
        }
    }

    pub(crate) fn source_directory_data(self) -> Option<SourceDirectoryScope<'a>> {
        match self.data {
            ScopeData::SourceDirectory { relative } => Some(SourceDirectoryScope { relative }),
            _ => None,
        }
    }

    pub(crate) fn directory_children_data(self) -> Option<DirectoryChildrenScope<'a>> {
        match self.data {
            ScopeData::DirectoryChildren {
                relative,
                children,
                source_child_count,
            } => Some(DirectoryChildrenScope {
                relative,
                children,
                source_child_count,
            }),
            _ => None,
        }
    }

    pub(crate) fn source_file_data(self) -> Option<SourceFileScope<'a>> {
        match self.data {
            ScopeData::SourceFile {
                relative,
                path,
                source,
                kind,
            } => Some(SourceFileScope {
                relative,
                path,
                source,
                kind,
            }),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum ScopeData<'a> {
    FilePath {
        relative: &'a Path,
    },
    SourceDirectory {
        relative: &'a Path,
    },
    DirectoryChildren {
        relative: &'a Path,
        children: &'a BTreeSet<String>,
        source_child_count: usize,
    },
    SourceFile {
        relative: &'a Path,
        path: &'a str,
        source: &'a str,
        kind: SourceKind,
    },
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct FilePathScope<'a> {
    pub(crate) relative: &'a Path,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct SourceDirectoryScope<'a> {
    pub(crate) relative: &'a Path,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct DirectoryChildrenScope<'a> {
    pub(crate) relative: &'a Path,
    pub(crate) children: &'a BTreeSet<String>,
    pub(crate) source_child_count: usize,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct SourceFileScope<'a> {
    pub(crate) relative: &'a Path,
    pub(crate) path: &'a str,
    pub(crate) source: &'a str,
    pub(crate) kind: SourceKind,
}

pub(crate) struct AnalysisContext<'a> {
    pub(crate) config: &'a GuardConfig,
    pub(crate) scope: Scope<'a>,
    pub(crate) products: ProductSet,
}

#[derive(Debug, Default)]
pub(crate) struct PluginOutput<'a> {
    pub(crate) issues: Vec<Issue>,
    pub(crate) child_scopes: Vec<Scope<'a>>,
    pub(crate) failure_surfaces: Vec<FailureSurfaceSignal>,
}

impl<'a> PluginOutput<'a> {
    pub(crate) fn issues(issues: Vec<Issue>) -> Self {
        Self {
            issues,
            child_scopes: Vec::new(),
            failure_surfaces: Vec::new(),
        }
    }

    pub(crate) fn with_failure_surfaces(
        issues: Vec<Issue>,
        failure_surfaces: Vec<FailureSurfaceSignal>,
    ) -> Self {
        Self {
            issues,
            child_scopes: Vec::new(),
            failure_surfaces,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct FailureSurfaceSignal {
    pub(crate) path: String,
    pub(crate) raw_count: usize,
    pub(crate) structured_count: usize,
    pub(crate) examples: Vec<String>,
}

type PluginAnalyzer = for<'a> fn(&AnalysisContext<'a>) -> PluginOutput<'a>;

struct FirstPartyPlugin {
    manifest: PluginManifest,
    analyze: PluginAnalyzer,
}

static PLUGINS: &[FirstPartyPlugin] = &[
    FirstPartyPlugin {
        manifest: filesystem::MANIFEST,
        analyze: filesystem::analyze,
    },
    FirstPartyPlugin {
        manifest: g4::MANIFEST,
        analyze: g4::analyze,
    },
    FirstPartyPlugin {
        manifest: language::RUST_MANIFEST,
        analyze: language::analyze_rust_source,
    },
    FirstPartyPlugin {
        manifest: language::TYPESCRIPT_MANIFEST,
        analyze: language::analyze_typescript_source,
    },
    FirstPartyPlugin {
        manifest: language::VUE_MANIFEST,
        analyze: language::analyze_vue_source,
    },
    FirstPartyPlugin {
        manifest: language::SVELTE_MANIFEST,
        analyze: language::analyze_svelte_source,
    },
];

#[derive(Clone, Copy)]
pub(crate) struct PluginHost {
    plugins: &'static [FirstPartyPlugin],
}

impl PluginHost {
    pub(crate) fn bundled() -> Self {
        Self { plugins: PLUGINS }
    }

    pub(crate) fn manifests(&self) -> Vec<PluginManifest> {
        self.plugins.iter().map(|plugin| plugin.manifest).collect()
    }

    pub(crate) fn run_scope<'a>(
        &self,
        config: &'a GuardConfig,
        initial_scope: Scope<'a>,
        issues: &mut Vec<Issue>,
    ) {
        let mut failure_surfaces = Vec::new();
        self.run_scope_with_signals(config, initial_scope, issues, &mut failure_surfaces);
    }

    pub(crate) fn run_scope_with_signals<'a>(
        &self,
        config: &'a GuardConfig,
        initial_scope: Scope<'a>,
        issues: &mut Vec<Issue>,
        failure_surfaces: &mut Vec<FailureSurfaceSignal>,
    ) {
        let mut queue = VecDeque::from([initial_scope]);
        while let Some(scope) = queue.pop_front() {
            for plugin in self.plugins_for(scope) {
                let context = AnalysisContext {
                    config,
                    scope,
                    products: ProductSet::new(config, plugin.manifest, scope),
                };
                let output = (plugin.analyze)(&context);
                issues.extend(output.issues);
                failure_surfaces.extend(output.failure_surfaces);
                queue.extend(output.child_scopes);
            }
        }
    }

    fn plugins_for<'a>(
        &self,
        scope: Scope<'a>,
    ) -> impl Iterator<Item = &'static FirstPartyPlugin> + 'a {
        let plugins: &'static [FirstPartyPlugin] = self.plugins;
        plugins.iter().filter(move |plugin| {
            plugin
                .manifest
                .scopes
                .iter()
                .any(|declaration| declaration.matches(scope))
        })
    }
}

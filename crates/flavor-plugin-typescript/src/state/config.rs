#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TsPluginConfig {
    pub source_mode: SourceMode,
    pub jsx: JsxConfig,
    pub decorators: DecoratorsConfig,
    pub failure_surface: TsFailureSurfaceConfig,
}

impl Default for TsPluginConfig {
    fn default() -> Self {
        Self {
            source_mode: SourceMode::TypeScript,
            jsx: JsxConfig::default(),
            decorators: DecoratorsConfig::default(),
            failure_surface: TsFailureSurfaceConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TsFailureSurfaceConfig {
    pub structured_guards: Vec<String>,
    pub structured_factories: Vec<String>,
    pub raw_reject_callees: Vec<String>,
}

impl Default for TsFailureSurfaceConfig {
    fn default() -> Self {
        Self {
            structured_guards: Vec::new(),
            structured_factories: Vec::new(),
            raw_reject_callees: ["Promise.reject", "reject"]
                .into_iter()
                .map(str::to_string)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SourceMode {
    JavaScript,
    Jsx,
    TypeScript,
    Tsx,
    Declaration,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct JsxConfig {
    pub enabled: bool,
}

impl Default for JsxConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct DecoratorsConfig {
    pub standard: bool,
    pub legacy: bool,
}

impl Default for DecoratorsConfig {
    fn default() -> Self {
        Self {
            standard: true,
            legacy: true,
        }
    }
}

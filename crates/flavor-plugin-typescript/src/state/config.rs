#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TsPluginConfig {
    pub source_mode: SourceMode,
    pub jsx: JsxConfig,
    pub decorators: DecoratorsConfig,
}

impl Default for TsPluginConfig {
    fn default() -> Self {
        Self {
            source_mode: SourceMode::TypeScript,
            jsx: JsxConfig::default(),
            decorators: DecoratorsConfig::default(),
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

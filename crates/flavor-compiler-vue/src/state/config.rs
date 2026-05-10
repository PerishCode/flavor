#[derive(Debug, Clone, Eq, PartialEq)]
pub struct VueCompilerConfig {
    pub template: TemplateConfig,
    pub style_facts: bool,
    pub script_facts: bool,
}

impl Default for VueCompilerConfig {
    fn default() -> Self {
        Self {
            template: TemplateConfig::default(),
            style_facts: true,
            script_facts: true,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct TemplateConfig {
    pub ast: bool,
    pub expressions: bool,
}

impl Default for TemplateConfig {
    fn default() -> Self {
        Self {
            ast: true,
            expressions: true,
        }
    }
}

use std::path::{Path, PathBuf};

use globset::{GlobBuilder, GlobMatcher};

#[derive(Debug, Clone)]
pub(crate) struct PathPattern {
    matcher: GlobMatcher,
}

impl PathPattern {
    pub(crate) fn new(pattern: &str) -> Result<Self, String> {
        let normalized = pattern.replace('\\', "/");
        let matcher = GlobBuilder::new(&normalized)
            .literal_separator(true)
            .build()
            .map_err(|error| format!("invalid glob pattern '{pattern}': {error}"))?
            .compile_matcher();
        Ok(Self { matcher })
    }

    pub(crate) fn matches(&self, path: &Path) -> bool {
        let path = path_string(path);
        self.matcher.is_match(&path)
    }
}

pub(crate) fn relative_path(root: &Path, path: &Path) -> Result<PathBuf, String> {
    path.strip_prefix(root)
        .map(Path::to_path_buf)
        .map_err(|error| format!("failed to strip root from {}: {error}", path.display()))
}

pub(crate) fn path_string(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

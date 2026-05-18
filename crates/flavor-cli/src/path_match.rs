use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct PathPattern {
    segments: Vec<Segment>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum Segment {
    AnyMany,
    AnyOne,
    Glob(String),
    Literal(String),
}

impl PathPattern {
    pub(crate) fn new(pattern: &str) -> Self {
        let segments = pattern
            .split('/')
            .filter(|segment| !segment.is_empty())
            .map(|segment| match segment {
                "**" => Segment::AnyMany,
                "*" => Segment::AnyOne,
                other if other.contains('*') => Segment::Glob(other.to_string()),
                other => Segment::Literal(other.to_string()),
            })
            .collect();

        Self { segments }
    }

    pub(crate) fn matches(&self, path: &Path) -> bool {
        let path_segments = path_segments(path);
        matches_segments(&self.segments, &path_segments)
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

fn path_segments(path: &Path) -> Vec<String> {
    path.components()
        .filter_map(|component| component.as_os_str().to_str())
        .filter(|segment| !segment.is_empty())
        .map(str::to_string)
        .collect()
}

fn matches_segments(pattern: &[Segment], path: &[String]) -> bool {
    match (pattern.split_first(), path.split_first()) {
        (None, None) => true,
        (None, Some(_)) => false,
        (Some((Segment::AnyMany, rest)), _) => {
            matches_segments(rest, path)
                || path
                    .split_first()
                    .is_some_and(|(_, path_rest)| matches_segments(pattern, path_rest))
        }
        (Some((Segment::AnyOne, rest)), Some((_, path_rest))) => matches_segments(rest, path_rest),
        (Some((Segment::Literal(expected), rest)), Some((actual, path_rest))) => {
            expected == actual && matches_segments(rest, path_rest)
        }
        (Some((Segment::Glob(expected), rest)), Some((actual, path_rest))) => {
            matches_glob(expected, actual) && matches_segments(rest, path_rest)
        }
        (Some(_), None) => false,
    }
}

fn matches_glob(pattern: &str, value: &str) -> bool {
    let pattern: Vec<char> = pattern.chars().collect();
    let value: Vec<char> = value.chars().collect();
    let mut pattern_index = 0;
    let mut value_index = 0;
    let mut star_index = None;
    let mut star_value_index = 0;

    while value_index < value.len() {
        if pattern
            .get(pattern_index)
            .is_some_and(|expected| *expected == value[value_index])
        {
            pattern_index += 1;
            value_index += 1;
        } else if pattern.get(pattern_index) == Some(&'*') {
            star_index = Some(pattern_index);
            star_value_index = value_index;
            pattern_index += 1;
        } else if let Some(star) = star_index {
            pattern_index = star + 1;
            star_value_index += 1;
            value_index = star_value_index;
        } else {
            return false;
        }
    }

    pattern[pattern_index..].iter().all(|ch| *ch == '*')
}

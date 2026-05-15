use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    config::GuardConfig,
    rules::{
        FS_CHILDREN_SHAPE, FS_FORBIDDEN_EXTENSION, FS_NAME_SHAPE, FS_TOO_MANY_CHILDREN,
        TSX_NO_INTRINSICS, TSX_REQUIRES_PRIMITIVE,
    },
    scan::run_scan,
};

#[test]
fn preference_expands_atomic_rules() {
    let root = test_root("expands");
    let src = root.join("apps/chat/src");
    fs::create_dir_all(src.join("components")).unwrap();
    fs::create_dir_all(src.join("views")).unwrap();
    fs::create_dir_all(src.join("lib/operation")).unwrap();
    fs::create_dir_all(src.join("styles")).unwrap();
    fs::write(src.join("app.tsx"), "export const app = 1;\n").unwrap();
    fs::write(src.join("main.tsx"), "export const main = 1;\n").unwrap();
    fs::write(src.join("style.css"), ".root { display: grid; }\n").unwrap();
    fs::write(
        src.join("components/panel.tsx"),
        "export function Panel() { return <div />; }\n",
    )
    .unwrap();
    fs::write(
        src.join("lib/operation/timelineView.ts"),
        "export const timeline = 1;\n",
    )
    .unwrap();

    let config = renderer_preference_config(&root);
    let issues = issues(&config);

    assert!(issues.iter().any(|issue| issue.rule == FS_CHILDREN_SHAPE
        && issue.message.contains("forbidden direct children")));
    assert!(issues
        .iter()
        .any(|issue| issue.rule == FS_FORBIDDEN_EXTENSION));
    assert!(issues
        .iter()
        .any(|issue| issue.rule == FS_NAME_SHAPE && issue.path.ends_with("components/panel.tsx")));
    assert!(issues.iter().any(|issue| issue.rule == FS_NAME_SHAPE
        && issue.path.ends_with("lib/operation/timelineView.ts")));
    assert!(issues.iter().any(|issue| issue.rule == TSX_NO_INTRINSICS));
    assert!(issues
        .iter()
        .any(|issue| issue.rule == TSX_REQUIRES_PRIMITIVE));
    assert!(!issues
        .iter()
        .any(|issue| issue.rule == FS_TOO_MANY_CHILDREN));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn explicit_overrides_win_preferences() {
    let root = test_root("override");
    write_renderer_skeleton(&root);
    fs::write(
        root.join("apps/chat/src/components/Panel.tsx"),
        "export function Panel() { return <div />; }\n",
    )
    .unwrap();

    let config = config_from(
        &root,
        r#"{
            "scan": { "include": ["apps/*/src/**"] },
            "preferences": [
                {
                    "name": "frontend/renderer-boundary",
                    "match": "apps/*/src",
                    "primitiveSources": ["@mini-stim/components"]
                }
            ],
            "overrides": [
                {
                    "match": "apps/chat/src/components/*.tsx",
                    "kind": "file",
                    "rules": {
                        "tsx/jsx/no-intrinsic-elements": {
                            "enabled": false,
                            "reason": "legacy component awaiting primitive extraction"
                        },
                        "tsx/component/requires-primitive-composition": {
                            "enabled": false,
                            "reason": "legacy component awaiting primitive extraction"
                        }
                    }
                }
            ]
        }"#,
    );
    let issues = issues(&config);

    assert!(!issues.iter().any(|issue| issue.rule == TSX_NO_INTRINSICS));
    assert!(!issues
        .iter()
        .any(|issue| issue.rule == TSX_REQUIRES_PRIMITIVE));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn accepts_primitive_composition() {
    let root = test_root("primitive");
    write_renderer_skeleton(&root);
    fs::write(
        root.join("apps/chat/src/components/Panel.tsx"),
        r#"import { Stack as Surface } from "@mini-stim/components";

export function Panel() {
  return <Surface />;
}
"#,
    )
    .unwrap();
    fs::write(
        root.join("apps/chat/src/components/Shell.tsx"),
        r#"import * as Primitive from "@mini-stim/components";

export function Shell() {
  return <Primitive.Stack />;
}
"#,
    )
    .unwrap();

    let config = renderer_preference_config(&root);
    let issues = issues(&config);

    assert!(!issues.iter().any(|issue| issue.rule == TSX_NO_INTRINSICS));
    assert!(!issues
        .iter()
        .any(|issue| issue.rule == TSX_REQUIRES_PRIMITIVE));

    let _ = fs::remove_dir_all(root);
}

fn test_root(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "flavor-renderer-boundary-{name}-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    root
}

fn renderer_preference_config(root: &Path) -> GuardConfig {
    config_from(
        root,
        r#"{
            "scan": { "include": ["apps/*/src/**"] },
            "preferences": [
                {
                    "name": "frontend/renderer-boundary",
                    "match": "apps/*/src",
                    "primitiveSources": ["@mini-stim/components"]
                }
            ]
        }"#,
    )
}

fn write_renderer_skeleton(root: &Path) {
    let src = root.join("apps/chat/src");
    fs::create_dir_all(src.join("components")).unwrap();
    fs::create_dir_all(src.join("views")).unwrap();
    fs::create_dir_all(src.join("lib")).unwrap();
    fs::write(src.join("app.tsx"), "export const app = 1;\n").unwrap();
    fs::write(src.join("main.tsx"), "export const main = 1;\n").unwrap();
}

fn config_from(root: &Path, source: &str) -> GuardConfig {
    let path = root.join("flavor.json");
    fs::write(&path, source).unwrap();
    GuardConfig::from_file(root.to_path_buf(), &path).unwrap()
}

fn issues(config: &GuardConfig) -> Vec<crate::model::Issue> {
    run_scan(config).unwrap().issues
}

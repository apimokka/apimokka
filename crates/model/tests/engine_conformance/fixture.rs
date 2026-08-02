//! On-disk workspace fixtures for the engine-conformance suite.
//!
//! Mirrors `apimock-config`'s own internal test fixture
//! (`workspace/tests/common.rs::make_workspace`), since this suite exists
//! to compare behavior against that crate and must exercise TOML shapes it
//! is already known to accept, not shapes we merely believe are valid.

use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// A minimal workspace: one rule set with two rules, an empty fallback
/// directory, and default listener settings. Returns the tempdir guard
/// (the directory is removed when it drops) and the absolute path to the
/// root `apimock.toml`.
pub fn minimal_workspace() -> (TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("create temp dir for engine-conformance fixture");

    let rule_set_toml = concat!(
        "[[rules]]\n",
        "when.request.url_path = \"/api/users\"\n",
        "respond = { text = \"ok\" }\n",
        "\n",
        "[[rules]]\n",
        "when.request.url_path = \"/api/health\"\n",
        "respond = { status = 204 }\n",
    );
    let rule_set_path = dir.path().join("apimock-rule-set.toml");
    fs::write(&rule_set_path, rule_set_toml).expect("write fixture rule set");

    let fallback_dir = dir.path().join("fallback");
    fs::create_dir_all(&fallback_dir).expect("create fixture fallback dir");

    let root_toml = format!(
        "[listener]\n\
         ip_address = \"127.0.0.1\"\n\
         port = 3001\n\
         \n\
         [service]\n\
         rule_sets = [\"{}\"]\n\
         fallback_respond_dir = \"{}\"\n",
        rule_set_path.file_name().unwrap().to_string_lossy(),
        fallback_dir.file_name().unwrap().to_string_lossy(),
    );
    let root_path = dir.path().join("apimock.toml");
    fs::write(&root_path, root_toml).expect("write fixture root config");

    (dir, root_path)
}

/// A workspace whose first rule already has one header and one body
/// condition, for update/remove/preserve-clear-replace scenarios that need
/// a pre-existing condition to target. Byte-equivalent to `apimock-config`'s
/// own `workspace/tests/common.rs::make_workspace_with_headers_and_body`.
pub fn workspace_with_headers_and_body() -> (TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("create temp dir for engine-conformance fixture");
    let fallback_dir = dir.path().join("fallback");
    fs::create_dir_all(&fallback_dir).expect("create fixture fallback dir");

    let rule_set_toml = concat!(
        "[[rules]]\n",
        "when.request.url_path = \"/api/protected\"\n",
        "when.request.headers.x-api-key = { value = \"shh\" }\n",
        "when.request.body.json.\"action\" = { op = \"equal\", value = \"go\" }\n",
        "respond = { text = \"ok\" }\n",
    );
    let rule_set_path = dir.path().join("apimock-rule-set.toml");
    fs::write(&rule_set_path, rule_set_toml).expect("write fixture rule set");

    let root_toml = format!(
        "[listener]\n\
         ip_address = \"127.0.0.1\"\n\
         port = 3001\n\
         \n\
         [service]\n\
         rule_sets = [\"{}\"]\n\
         fallback_respond_dir = \"{}\"\n",
        rule_set_path.file_name().unwrap().to_string_lossy(),
        fallback_dir.file_name().unwrap().to_string_lossy(),
    );
    let root_path = dir.path().join("apimock.toml");
    fs::write(&root_path, root_toml).expect("write fixture root config");

    (dir, root_path)
}

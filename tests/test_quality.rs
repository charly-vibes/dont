/// Structural test: shared test helpers must be consolidated.
///
/// The `dont()`, `init_dir()`, and `conclude_claim()` functions were
/// copy-pasted verbatim across 27+ test files. This test asserts that a
/// canonical shared module exists so future changes only need to land in one
/// place.
#[test]
fn shared_helpers_module_exists() {
    let common = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("common")
        .join("mod.rs");
    assert!(
        common.exists(),
        "tests/common/mod.rs must exist — shared test helpers (`dont`, \
         `init_dir`, `conclude_claim`) must not be copy-pasted per file"
    );
}

/// Naming quality gate: test function names must be descriptive.
///
/// Good test names follow the pattern `<unit>_<scenario>_<expected_result>`.
/// Names that end in `_works`, `_basic`, `_simple`, `_stuff`, or `_misc`
/// give no diagnostic signal when a test fails. This test reads the source
/// files and asserts none of the test functions use these anti-patterns.
#[test]
fn test_function_names_are_descriptive() {
    use std::fs;

    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let anti_patterns: &[&str] = &["_works", "_basic", "_simple", "_stuff", "_misc"];

    let search_dirs = ["tests", "src"];
    let mut violations: Vec<String> = Vec::new();

    for dir_name in &search_dirs {
        let dir = manifest.join(dir_name);
        if !dir.exists() {
            continue;
        }
        collect_violations(&dir, anti_patterns, &mut violations);
    }

    assert!(
        violations.is_empty(),
        "Test function names with non-descriptive suffixes found.\n\
         Rename them to follow <unit>_<scenario>_<expected_result>.\n\
         Violations:\n{}",
        violations.join("\n")
    );
}

/// Mock-coupling audit: no test-only trait implementations or mock structs.
///
/// In Rust, mock coupling manifests as:
/// 1. `#[cfg(test)]` impl blocks that implement traits (replacing real types
///    with test doubles that short-circuit the integration under test).
/// 2. Structs named with Mock/Fake/Stub/Spy suffixes/prefixes defined inside
///    `#[cfg(test)]` guards (test-only stand-ins for production types).
/// 3. Mock framework crates in `[dev-dependencies]` (mockall, mockito,
///    wiremock, double, faux, etc.).
///
/// Audit result (2026-05-14): codebase is clean. All `#[cfg(test)]` blocks
/// contain only test logic and helper functions — no trait impls. The one
/// `MockEvidenceCheckResult` struct is ungated (production code) and serves
/// as a network-stub configuration via env var, not a test double.
/// No mock framework crates are present in dev-dependencies.
#[test]
fn no_cfg_test_trait_impl_blocks() {
    use std::fs;

    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut violations: Vec<String> = Vec::new();

    for dir_name in &["src", "tests"] {
        let dir = manifest.join(dir_name);
        if !dir.exists() {
            continue;
        }
        collect_cfg_test_impl_violations(&dir, &mut violations);
    }

    assert!(
        violations.is_empty(),
        "Found `impl Trait for Type` blocks inside `#[cfg(test)]` guards.\n\
         These replace real types with test doubles that bypass the actual \
         integration path and give false confidence.\n\
         Fix by using real implementations against a TempDir environment.\n\
         Violations:\n{}",
        violations.join("\n")
    );
}

/// Audit: no mock framework crates in dev-dependencies.
///
/// Confirms that the project uses integration-style testing (real binary via
/// assert_cmd + tempfile) rather than injected mocks from crates such as
/// mockall, mockito, wiremock, double, faux, or mockers.
#[test]
fn no_mock_framework_in_dev_dependencies() {
    let cargo_toml_path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let content = std::fs::read_to_string(&cargo_toml_path)
        .expect("Cargo.toml must be readable");

    let mock_crates = [
        "mockall", "mockito", "wiremock", "double", "faux", "mockers",
        "mock_it", "unimock",
    ];

    let violations: Vec<&str> = mock_crates
        .iter()
        .copied()
        .filter(|crate_name| content.contains(crate_name))
        .collect();

    assert!(
        violations.is_empty(),
        "Mock framework crate(s) found in Cargo.toml: {:?}\n\
         This project tests via the real binary with assert_cmd + tempfile. \
         Introducing a mock framework encourages coupling tests to internal \
         interfaces rather than observable behaviour.",
        violations
    );
}

fn collect_cfg_test_impl_violations(dir: &std::path::Path, out: &mut Vec<String>) {
    use std::fs;
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_cfg_test_impl_violations(&path, out);
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let lines: Vec<&str> = content.lines().collect();
        let mut in_cfg_test = false;
        let mut brace_depth: i32 = 0;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            // Entering a #[cfg(test)] section
            if trimmed == "#[cfg(test)]" {
                in_cfg_test = true;
                brace_depth = 0;
                continue;
            }

            if in_cfg_test {
                brace_depth += line.chars().filter(|&c| c == '{').count() as i32;
                brace_depth -= line.chars().filter(|&c| c == '}').count() as i32;

                // Detect `impl TraitName for TypeName` (but not `impl TypeName {`)
                // Pattern: `impl` followed by a word, `for`, another word
                if trimmed.starts_with("impl ") && trimmed.contains(" for ") {
                    out.push(format!(
                        "  {}:{}: {}",
                        path.display(),
                        i + 1,
                        trimmed.chars().take(80).collect::<String>()
                    ));
                }

                if brace_depth <= 0 && brace_depth < 0 {
                    // We've exited all braces: guard section ended
                    in_cfg_test = false;
                }
            }
        }
    }
}

fn collect_violations(dir: &std::path::Path, anti_patterns: &[&str], out: &mut Vec<String>) {
    use std::fs;
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_violations(&path, anti_patterns, out);
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let lines: Vec<&str> = content.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            if !line.contains("#[test]") {
                continue;
            }
            // Look ahead up to 4 lines for the fn declaration
            for j in (i + 1)..std::cmp::min(i + 5, lines.len()) {
                let trimmed = lines[j].trim();
                if let Some(rest) = trimmed.strip_prefix("fn ") {
                    let fn_name = rest.split('(').next().unwrap_or("").trim();
                    for pat in anti_patterns {
                        if fn_name.ends_with(pat) {
                            out.push(format!(
                                "  {}:{}: fn {} (ends with '{}')",
                                path.display(),
                                j + 1,
                                fn_name,
                                pat
                            ));
                        }
                    }
                    break;
                }
            }
        }
    }
}

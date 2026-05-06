use assert_cmd::Command;
use serde_json::Value;
use tempfile::TempDir;

fn dont() -> Command {
    Command::cargo_bin("dont").unwrap()
}

fn init_dir(dir: &TempDir) {
    dont()
        .arg("init")
        .arg("--json")
        .env("DONT_DIR", dir.path())
        .assert()
        .success();
}

#[test]
fn define_creates_unverified_term() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let output = dont()
        .args([
            "define",
            "WB:P001",
            "--doc",
            "Process by which X becomes Y",
            "--json",
        ])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["envelope_kind"], "term");
    assert!(v["data"]["id"].as_str().unwrap().starts_with("term:"));
    assert_eq!(v["data"]["entity_kind"], "term");
    assert_eq!(v["data"]["curie"], "WB:P001");
    assert_eq!(v["data"]["definition"], "Process by which X becomes Y");
    assert_eq!(v["data"]["status"], "unverified");
    assert!(v["meta"]["tx"].is_number());
}

#[test]
fn define_missing_curie_returns_structured_refusal() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let output = dont()
        .args(["define", "--doc", "A definition", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["envelope_kind"], "error");
    assert_eq!(v["data"]["code"], "curie-required");
    assert!(!v["data"]["remediation"].as_array().unwrap().is_empty());
}

#[test]
fn define_missing_doc_returns_structured_refusal() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let output = dont()
        .args(["define", "WB:P001", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "doc-required");
    assert!(!v["data"]["remediation"].as_array().unwrap().is_empty());
}

#[test]
fn defined_term_can_be_shown_after_reopen() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let output = dont()
        .args(["define", "WB:P001", "--doc", "Process by which X", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&output).unwrap();
    let id = v["data"]["id"].as_str().unwrap();

    let output = dont()
        .args(["show", id, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["envelope_kind"], "term");
    assert_eq!(v["data"]["curie"], "WB:P001");
}

#[test]
fn define_with_empty_label_is_refused() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let output = dont()
        .args(["define", "WB:P001", "--doc", "A valid definition", "--label", "", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "term-label-empty");
    assert!(!v["data"]["remediation"].as_array().unwrap().is_empty());
}

#[test]
fn define_with_label_missing_article_is_refused() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let output = dont()
        .args(["define", "WB:P001", "--doc", "A valid definition", "--label", "Ricci tensor", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "term-shape-indefinite");
    assert!(!v["data"]["remediation"].as_array().unwrap().is_empty());
}

#[test]
fn define_with_punctuated_label_is_refused() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let output = dont()
        .args(["define", "WB:P001", "--doc", "A valid definition", "--label", "a Ricci tensor.", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "term-shape-punctuated");
    assert!(!v["data"]["remediation"].as_array().unwrap().is_empty());
}

#[test]
fn define_with_compound_label_without_vars_is_refused() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let output = dont()
        .args(["define", "WB:P001", "--doc", "A valid definition", "--label", "a pair of integers", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "term-compound-undeclared");
    assert!(!v["data"]["remediation"].as_array().unwrap().is_empty());
}

#[test]
fn define_with_sentence_shaped_label_is_refused() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let output = dont()
        .args(["define", "WB:P001", "--doc", "A valid definition", "--label", "a tensor that is Ricci", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "term-label-sentence");
    assert!(!v["data"]["remediation"].as_array().unwrap().is_empty());
}

#[test]
fn define_with_well_formed_label_succeeds() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let output = dont()
        .args(["define", "WB:P001", "--doc", "a Ricci tensor on a smooth manifold", "--label", "a Ricci tensor", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["label"], "a Ricci tensor");
}

#[test]
fn define_with_an_label_succeeds() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let output = dont()
        .args(["define", "WB:P001", "--doc", "an amino acid found in dairy products", "--label", "an amino acid found in dairy", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["label"], "an amino acid found in dairy");
}

#[test]
fn define_without_label_emits_doc_shape_warning() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let output = dont()
        .args(["define", "WB:P001", "--doc", "Ricci tensor. Extended explanation of what it is.", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], true);
    let warnings = v["warnings"].as_array().unwrap();
    assert!(
        warnings.iter().any(|w| w["rule_name"] == "term-doc-shape-indefinite"),
        "expected term-doc-shape-indefinite warning, got: {:?}", warnings
    );
}

#[test]
fn define_with_where_clause_label_passes_sentence_check() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let output = dont()
        .args([
            "define", "WB:P001",
            "--doc", "a pair (x, y) where x is an integer and y is a real number",
            "--label", "a pair (x, y) where x is an integer and y is a real number",
            "--json",
        ])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], true);
}

#[test]
fn define_with_arity_mismatch_compound_label_is_refused() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let output = dont()
        .args(["define", "WB:P001", "--doc", "A valid definition", "--label", "a triple (x, y)", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "term-compound-undeclared");
}

#[test]
fn define_without_label_compound_doc_emits_no_doc_shape_warning() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let output = dont()
        .args(["define", "WB:P001", "--doc", "a pair (x, y) where x and y are integers", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["ok"], true);
    let warnings = v["warnings"].as_array().unwrap();
    assert!(
        !warnings.iter().any(|w| w["rule_name"].as_str().unwrap_or("").starts_with("term-doc-shape-")),
        "expected no doc-shape warnings for compound doc, got: {:?}", warnings
    );
}

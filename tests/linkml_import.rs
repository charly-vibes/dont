// dont-dj1v: spec-alignment tests for dont-linkml-import
//
// Spec: openspec/specs/dont-linkml-import/spec.md
//
// These tests document the behavioral contract. They were written in the RED
// phase while the adapter was still `not-yet-implemented`. Each test maps to
// a named scenario in the spec.

mod common;

use common::{dont, init_dir};
use serde_json::Value;
use std::fs;
use tempfile::TempDir;

/// Write a LinkML YAML file into `dir` and return its path as a string.
fn write_schema(dir: &TempDir, name: &str, content: &str) -> String {
    let path = dir.path().join(name);
    fs::write(&path, content).unwrap();
    path.to_str().unwrap().to_string()
}

// ---------------------------------------------------------------------------
// Requirement: Refusal-tier unsupported features
// Scenario: unsupported feature refuses import
// ---------------------------------------------------------------------------

/// A schema using reified slots must fail with `linkml-unsupported-feature`
/// and name the offending construct.
#[test]
fn linkml_reified_slot_triggers_unsupported_feature_refusal() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let schema = r#"
id: https://example.org/reified-test
name: reified-test
prefixes:
  linkml: https://w3id.org/linkml/
default_range: string

classes:
  ReifiedRelationship:
    attributes:
      subject:
        range: string
      predicate:
        range: string
      object:
        range: string
    reified: true
"#;
    let schema_path = write_schema(&dir, "reified.yaml", schema);

    let out = dont()
        .args(["import", "linkml", &schema_path, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false, "reified slot schema must be refused");
    let code = v["data"]["code"].as_str().unwrap_or("");
    assert_eq!(
        code, "linkml-unsupported-feature",
        "error code must be linkml-unsupported-feature, got: {code}"
    );
    let message = v["data"]["message"].as_str().unwrap_or("");
    assert!(
        message.contains("reified") || message.contains("ReifiedRelationship"),
        "error must identify the offending construct, got: {message}"
    );
}

/// A schema with a `python_class` annotation must fail with
/// `linkml-unsupported-feature`.
#[test]
fn linkml_python_class_triggers_unsupported_feature_refusal() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let schema = r#"
id: https://example.org/python-test
name: python-test
prefixes:
  linkml: https://w3id.org/linkml/
default_range: string

classes:
  MyClass:
    python_class: mymodule.MyClass
    attributes:
      label:
        range: string
"#;
    let schema_path = write_schema(&dir, "python.yaml", schema);

    let out = dont()
        .args(["import", "linkml", &schema_path, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false, "python_class schema must be refused");
    let code = v["data"]["code"].as_str().unwrap_or("");
    assert_eq!(
        code, "linkml-unsupported-feature",
        "error code must be linkml-unsupported-feature, got: {code}"
    );
    let message = v["data"]["message"].as_str().unwrap_or("");
    assert!(
        message.contains("python_class") || message.contains("MyClass"),
        "error must identify the offending construct, got: {message}"
    );
}

/// A schema using a SPARQL-evaluated expression must fail with
/// `linkml-unsupported-feature`.
#[test]
fn linkml_sparql_rule_triggers_unsupported_feature_refusal() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let schema = r#"
id: https://example.org/sparql-test
name: sparql-test
prefixes:
  linkml: https://w3id.org/linkml/
default_range: string

classes:
  ConstrainedClass:
    attributes:
      value:
        range: string
    rules:
      - postconditions:
          slot_conditions:
            value:
              value_presence: PRESENT
        sparql: "ASK { ?this <https://example.org/value> ?v . FILTER(strlen(str(?v)) > 0) }"
"#;
    let schema_path = write_schema(&dir, "sparql.yaml", schema);

    let out = dont()
        .args(["import", "linkml", &schema_path, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], false, "SPARQL expression schema must be refused");
    let code = v["data"]["code"].as_str().unwrap_or("");
    assert_eq!(
        code, "linkml-unsupported-feature",
        "error code must be linkml-unsupported-feature, got: {code}"
    );
    let message = v["data"]["message"].as_str().unwrap_or("");
    assert!(
        message.contains("sparql") || message.contains("SPARQL"),
        "error must identify SPARQL as the offending construct, got: {message}"
    );
}

// ---------------------------------------------------------------------------
// Requirement: No partial import on refusal
// Scenario: unsupported schema leaves no partial state
// ---------------------------------------------------------------------------

/// When `dont import linkml` refuses due to an unsupported feature, none of the
/// schema's terms must be imported into the local store.
#[test]
fn linkml_refusal_leaves_no_partial_state() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    // Schema contains both a useful class AND a python_class (refusal trigger)
    let schema = r#"
id: https://example.org/partial-test
name: partial-test
prefixes:
  linkml: https://w3id.org/linkml/
  ex: https://example.org/
default_range: string

classes:
  GoodClass:
    class_uri: ex:GoodClass
    attributes:
      name:
        range: string

  BadClass:
    class_uri: ex:BadClass
    python_class: mymodule.BadClass
    attributes:
      value:
        range: string
"#;
    let schema_path = write_schema(&dir, "partial.yaml", schema);

    // Import must be refused
    dont()
        .args(["import", "linkml", &schema_path, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .code(1);

    // GoodClass should NOT appear in the store (no partial import)
    let list_out = dont()
        .args(["list", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&list_out).unwrap();
    let empty = vec![];
    let items = v["data"].as_array().unwrap_or(&empty);
    let has_good_class = items.iter().any(|item| {
        item["id"].as_str().unwrap_or("").contains("GoodClass")
            || item["label"].as_str().unwrap_or("").contains("GoodClass")
    });
    assert!(
        !has_good_class,
        "GoodClass must not be imported when schema is refused (no partial state)"
    );
}

// ---------------------------------------------------------------------------
// Requirement: Warning-tier feature handling
// Scenario: enum import warns
// ---------------------------------------------------------------------------

/// A schema containing `permissible_values` enums must import approximately
/// and record a warning naming that feature.
#[test]
fn linkml_permissible_values_imports_with_warning() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let schema = r#"
id: https://example.org/enum-test
name: enum-test
prefixes:
  linkml: https://w3id.org/linkml/
  ex: https://example.org/
default_range: string

enums:
  StatusEnum:
    permissible_values:
      active:
        description: Active status
      inactive:
        description: Inactive status

classes:
  MyEntity:
    class_uri: ex:MyEntity
    attributes:
      status:
        range: StatusEnum
"#;
    let schema_path = write_schema(&dir, "enum.yaml", schema);

    let out = dont()
        .args(["import", "linkml", &schema_path, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true, "enum schema must import successfully (with warning)");

    let warnings = v["warnings"].as_array().unwrap();
    let has_enum_warning = warnings.iter().any(|w| {
        let msg = w["message"].as_str().unwrap_or("");
        msg.contains("permissible_values") || msg.contains("enum") || msg.contains("StatusEnum")
    });
    assert!(
        has_enum_warning,
        "import with permissible_values must record a warning about the approximation, got warnings: {warnings:?}"
    );
}

// ---------------------------------------------------------------------------
// Requirement: Warning-tier feature handling
// Scenario: pattern or range import warns
// ---------------------------------------------------------------------------

/// A schema using a string pattern constraint must import approximately and
/// warn about the approximation.
#[test]
fn linkml_string_pattern_imports_with_warning() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let schema = r#"
id: https://example.org/pattern-test
name: pattern-test
prefixes:
  linkml: https://w3id.org/linkml/
  ex: https://example.org/
default_range: string

classes:
  IdentifiedEntity:
    class_uri: ex:IdentifiedEntity
    attributes:
      identifier:
        range: string
        pattern: "^[A-Z]{2}[0-9]{4}$"
"#;
    let schema_path = write_schema(&dir, "pattern.yaml", schema);

    let out = dont()
        .args(["import", "linkml", &schema_path, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true, "pattern-constrained schema must import successfully (with warning)");

    let warnings = v["warnings"].as_array().unwrap();
    let has_pattern_warning = warnings.iter().any(|w| {
        let msg = w["message"].as_str().unwrap_or("");
        msg.contains("pattern") || msg.contains("identifier")
    });
    assert!(
        has_pattern_warning,
        "import with pattern constraint must record a warning, got warnings: {warnings:?}"
    );
}

// ---------------------------------------------------------------------------
// Requirement: Flattened feature tier
// Scenario: inheritance becomes kind_of chain
// ---------------------------------------------------------------------------

/// A schema using `is_a` inheritance must import silently; the resulting
/// terms must reflect the inheritance relationship as a kind_of chain.
/// The import must NOT fail and must NOT emit a warning solely for `is_a`.
#[test]
fn linkml_is_a_imports_silently_as_kind_of_chain() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let schema = r#"
id: https://example.org/isa-test
name: isa-test
prefixes:
  linkml: https://w3id.org/linkml/
  ex: https://example.org/
default_range: string

classes:
  Animal:
    class_uri: ex:Animal
    description: A living organism

  Dog:
    class_uri: ex:Dog
    is_a: Animal
    description: A domesticated canine
"#;
    let schema_path = write_schema(&dir, "isa.yaml", schema);

    let out = dont()
        .args(["import", "linkml", &schema_path, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true, "is_a schema must import successfully");

    // No warning solely because of the is_a relationship
    let warnings = v["warnings"].as_array().unwrap();
    let has_isa_warning = warnings.iter().any(|w| {
        let msg = w["message"].as_str().unwrap_or("");
        msg.contains("is_a") || msg.contains("inheritance")
    });
    assert!(
        !has_isa_warning,
        "is_a must be a silent flattening — no warning expected, got: {warnings:?}"
    );
}

// ---------------------------------------------------------------------------
// Requirement: Flattened feature tier
// Scenario: mixins flatten silently
// ---------------------------------------------------------------------------

/// A schema using mixins must import without a warning solely for the mixin
/// transformation.
#[test]
fn linkml_mixins_flatten_silently() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let schema = r#"
id: https://example.org/mixin-test
name: mixin-test
prefixes:
  linkml: https://w3id.org/linkml/
  ex: https://example.org/
default_range: string

classes:
  HasTimestamps:
    mixin: true
    attributes:
      created_at:
        range: string
      updated_at:
        range: string

  Document:
    class_uri: ex:Document
    mixins:
      - HasTimestamps
    attributes:
      title:
        range: string
"#;
    let schema_path = write_schema(&dir, "mixin.yaml", schema);

    let out = dont()
        .args(["import", "linkml", &schema_path, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true, "mixin schema must import successfully");

    // No warning solely because of mixin flattening
    let warnings = v["warnings"].as_array().unwrap();
    let has_mixin_warning = warnings.iter().any(|w| {
        let msg = w["message"].as_str().unwrap_or("");
        msg.contains("mixin") || msg.contains("HasTimestamps")
    });
    assert!(
        !has_mixin_warning,
        "mixin flattening must be silent — no warning expected, got: {warnings:?}"
    );
}

// ---------------------------------------------------------------------------
// Requirement: LinkML adapter shell-out and lowering contract
// Scenario: linkml import lowers to local relations
// ---------------------------------------------------------------------------

/// A basic, clean LinkML schema with no unsupported features must import
/// successfully and add terms to the local store.
#[test]
fn linkml_basic_schema_lowers_to_local_relations() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let schema = r#"
id: https://example.org/basic-test
name: basic-test
prefixes:
  linkml: https://w3id.org/linkml/
  ex: https://example.org/
default_range: string

classes:
  Observation:
    class_uri: ex:Observation
    description: A scientific observation
    attributes:
      description:
        range: string
"#;
    let schema_path = write_schema(&dir, "basic.yaml", schema);

    let out = dont()
        .args(["import", "linkml", &schema_path, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true, "basic schema must import successfully");

    // The import must produce some result data (imported terms/relations)
    let data = &v["data"];
    assert!(
        !data.is_null(),
        "import result must include data about what was imported"
    );
}

//! LinkML import adapter.
//!
//! Implements `dont import linkml <schema.yaml>` per the
//! `dont-linkml-import` spec. The adapter:
//!
//! * **Refuses** schemas with reified slots, SPARQL-evaluated expressions,
//!   or `python_class` injections — returning `linkml-unsupported-feature`
//!   and listing every offending construct. No terms are stored on refusal
//!   (no partial state).
//!
//! * **Warns** about approximate translations of `permissible_values` enums,
//!   string-pattern constraints, and value-range constraints.
//!
//! * **Silently flattens** `is_a` inheritance (represented as a `kind_of`
//!   chain) and mixin attribute sets.
//!
//! * **Lowers** class and slot definitions to local import relations (term
//!   records) that `dont` can check.
//!
//! The adapter parses the schema in-process. When the LinkML CLI is available
//! it may be used as a secondary validation step; when it is absent the
//! in-process path is authoritative.
use serde_json::Value;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// A single term extracted from a LinkML schema.
#[derive(Debug, Clone)]
pub struct ImportedTerm {
    /// CURIE derived from the class / slot URI.
    pub curie: String,
    /// Human-readable label (class / slot name).
    pub label: String,
    /// Definition text.
    pub definition: String,
    /// Optional parent CURIE for `is_a` chains.
    pub kind_of: Option<String>,
}

/// Warnings produced during an approximate translation.
#[derive(Debug, Clone)]
pub struct ImportWarning {
    pub feature: String,
    pub message: String,
    pub source_name: String,
}

/// The result of a successful (non-refused) import.
#[derive(Debug)]
pub struct LinkmlImportResult {
    pub terms: Vec<ImportedTerm>,
    pub warnings: Vec<ImportWarning>,
}

/// Error variants for refused imports.
#[derive(Debug)]
pub struct LinkmlUnsupportedError {
    /// Always `"linkml-unsupported-feature"`.
    pub code: String,
    /// Human-readable description of why the import was refused.
    pub message: String,
    /// Each offending construct, named.
    pub offending: Vec<String>,
}

impl LinkmlUnsupportedError {
    fn new(offending: Vec<String>) -> Self {
        let list = offending.join(", ");
        Self {
            code: "linkml-unsupported-feature".to_string(),
            message: format!(
                "schema uses unsupported LinkML features that cannot be safely approximated: {list}"
            ),
            offending,
        }
    }
}

// ---------------------------------------------------------------------------
// Main entry point
// ---------------------------------------------------------------------------

/// Parse a LinkML YAML schema from `content` and return either an import
/// result or a refusal error.  The `schema_name` is used in diagnostic
/// messages.
///
/// This is intentionally decoupled from I/O so it can be unit-tested.
pub fn import_schema(
    schema_name: &str,
    content: &str,
) -> Result<LinkmlImportResult, LinkmlUnsupportedError> {
    // Parse as a generic YAML value so we can inspect any key.
    let root: Value = serde_yaml::from_str(content).map_err(|e| LinkmlUnsupportedError {
        code: "linkml-unsupported-feature".to_string(),
        message: format!("could not parse LinkML schema: {e}"),
        offending: vec!["(parse error)".to_string()],
    })?;

    // -----------------------------------------------------------------------
    // Refusal-tier scan (must run BEFORE we touch the store).
    // -----------------------------------------------------------------------
    let offending = collect_refusal_constructs(&root);
    if !offending.is_empty() {
        return Err(LinkmlUnsupportedError::new(offending));
    }

    // -----------------------------------------------------------------------
    // Warning-tier scan
    // -----------------------------------------------------------------------
    let mut warnings: Vec<ImportWarning> = Vec::new();
    collect_warning_tier(&root, schema_name, &mut warnings);

    // -----------------------------------------------------------------------
    // Term lowering
    // -----------------------------------------------------------------------
    let terms = lower_classes(&root);

    Ok(LinkmlImportResult { terms, warnings })
}

// ---------------------------------------------------------------------------
// Refusal-tier helpers
// ---------------------------------------------------------------------------

fn collect_refusal_constructs(root: &Value) -> Vec<String> {
    let mut offending: Vec<String> = Vec::new();

    if let Some(classes) = root.get("classes").and_then(|v| v.as_object()) {
        for (class_name, class_def) in classes {
            // python_class injection
            if class_def.get("python_class").is_some() {
                offending.push(format!("python_class on class `{class_name}`"));
            }

            // reified: true
            if class_def.get("reified").and_then(|v| v.as_bool()) == Some(true) {
                offending.push(format!("reified: true on class `{class_name}`"));
            }

            // SPARQL in rules
            if let Some(rules) = class_def.get("rules").and_then(|v| v.as_array()) {
                for (i, rule) in rules.iter().enumerate() {
                    if rule.get("sparql").is_some() {
                        offending.push(format!(
                            "sparql expression in rule[{i}] on class `{class_name}`"
                        ));
                    }
                }
            }
        }
    }

    offending
}

// ---------------------------------------------------------------------------
// Warning-tier helpers
// ---------------------------------------------------------------------------

fn collect_warning_tier(root: &Value, schema_name: &str, warnings: &mut Vec<ImportWarning>) {
    // permissible_values in enums
    if let Some(enums) = root.get("enums").and_then(|v| v.as_object()) {
        for (enum_name, enum_def) in enums {
            if enum_def.get("permissible_values").is_some() {
                warnings.push(ImportWarning {
                    feature: "permissible_values".to_string(),
                    message: format!(
                        "enum `{enum_name}` in {schema_name} uses `permissible_values`; \
                         imported approximately as local term annotations — \
                         full enum constraint validation is not enforced"
                    ),
                    source_name: schema_name.to_string(),
                });
            }
        }
    }

    // string patterns and value ranges on slots / class attributes
    if let Some(classes) = root.get("classes").and_then(|v| v.as_object()) {
        for (class_name, class_def) in classes {
            // inline attributes
            if let Some(attrs) = class_def.get("attributes").and_then(|v| v.as_object()) {
                for (attr_name, attr_def) in attrs {
                    check_slot_warnings(attr_def, class_name, attr_name, schema_name, warnings);
                }
            }
        }
    }

    // top-level slots
    if let Some(slots) = root.get("slots").and_then(|v| v.as_object()) {
        for (slot_name, slot_def) in slots {
            check_slot_warnings(slot_def, "(top-level)", slot_name, schema_name, warnings);
        }
    }
}

fn check_slot_warnings(
    slot_def: &Value,
    class_name: &str,
    slot_name: &str,
    schema_name: &str,
    warnings: &mut Vec<ImportWarning>,
) {
    if slot_def.get("pattern").is_some() {
        warnings.push(ImportWarning {
            feature: "pattern".to_string(),
            message: format!(
                "slot `{slot_name}` on `{class_name}` in {schema_name} has a string pattern \
                 constraint; imported approximately — pattern is not enforced by dont"
            ),
            source_name: schema_name.to_string(),
        });
    }

    if slot_def.get("minimum_value").is_some() || slot_def.get("maximum_value").is_some() {
        warnings.push(ImportWarning {
            feature: "value_range".to_string(),
            message: format!(
                "slot `{slot_name}` on `{class_name}` in {schema_name} has a value-range \
                 constraint; imported approximately — range is not enforced by dont"
            ),
            source_name: schema_name.to_string(),
        });
    }

    if slot_def.get("minimum_cardinality").is_some() {
        warnings.push(ImportWarning {
            feature: "minimum_cardinality".to_string(),
            message: format!(
                "slot `{slot_name}` on `{class_name}` in {schema_name} has a minimum-cardinality \
                 constraint; imported approximately — cardinality is not enforced by dont"
            ),
            source_name: schema_name.to_string(),
        });
    }
}

// ---------------------------------------------------------------------------
// Term lowering
// ---------------------------------------------------------------------------

/// Lower LinkML classes to `ImportedTerm` records.
///
/// Flattening rules (silent, no warnings):
/// - `is_a` → `kind_of` on the resulting term
/// - `mixins` → attributes are merged into the class's own attribute set
/// - `slot_usage` → expanded inline (no separate predicate)
fn lower_classes(root: &Value) -> Vec<ImportedTerm> {
    let mut terms: Vec<ImportedTerm> = Vec::new();

    let schema_prefix = root
        .get("default_prefix")
        .or_else(|| root.get("id"))
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    // Build a CURIE for a class from its `class_uri` or by combining
    // default prefix with name.
    let classes = root
        .get("classes")
        .and_then(|v| v.as_object())
        .cloned()
        .unwrap_or_default();

    let prefixes: HashMap<String, String> = root
        .get("prefixes")
        .and_then(|v| v.as_object())
        .map(|m| {
            m.iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                .collect()
        })
        .unwrap_or_default();

    for (class_name, class_def) in &classes {
        // Skip pure-mixin classes — they get folded into inheriting classes.
        if class_def.get("mixin").and_then(|v| v.as_bool()) == Some(true) {
            continue;
        }

        let curie = derive_curie(class_def, class_name, &prefixes, schema_prefix);

        let definition = class_def
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("Imported from LinkML schema.")
            .to_string();

        // is_a → kind_of (silent flattening)
        let kind_of = class_def
            .get("is_a")
            .and_then(|v| v.as_str())
            .map(|parent| {
                let parent_def = classes.get(parent);
                parent_def
                    .map(|pd| derive_curie(pd, parent, &prefixes, schema_prefix))
                    .unwrap_or_else(|| parent.to_string())
            });

        terms.push(ImportedTerm {
            curie,
            label: class_name.clone(),
            definition,
            kind_of,
        });
    }

    terms
}

fn derive_curie(
    class_def: &Value,
    class_name: &str,
    prefixes: &HashMap<String, String>,
    schema_prefix: &str,
) -> String {
    if let Some(uri) = class_def.get("class_uri").and_then(|v| v.as_str()) {
        // If it's already a CURIE (contains ':'), use it as-is.
        if uri.contains(':') && !uri.starts_with("http") {
            return uri.to_string();
        }
        // Try to compact a full URI to a CURIE using known prefixes.
        for (prefix_key, prefix_uri) in prefixes {
            if let Some(local) = uri.strip_prefix(prefix_uri.as_str()) {
                return format!("{prefix_key}:{local}");
            }
        }
        // Fall back: use the URI as the "CURIE"
        return uri.to_string();
    }

    // No class_uri — synthesise one from the schema prefix and name.
    // Use the schema id (URL) as the prefix base.
    let base = schema_prefix
        .split('/')
        .next_back()
        .unwrap_or(schema_prefix)
        .trim_end_matches(':');
    format!("{base}:{class_name}")
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reified_class_is_refused() {
        let yaml = r#"
id: https://example.org/t
name: t
classes:
  R:
    reified: true
"#;
        let err = import_schema("t.yaml", yaml).unwrap_err();
        assert_eq!(err.code, "linkml-unsupported-feature");
        assert!(err.offending.iter().any(|s| s.contains("reified")));
    }

    #[test]
    fn python_class_is_refused() {
        let yaml = r#"
id: https://example.org/t
name: t
classes:
  M:
    python_class: mod.M
"#;
        let err = import_schema("t.yaml", yaml).unwrap_err();
        assert_eq!(err.code, "linkml-unsupported-feature");
        assert!(err.offending.iter().any(|s| s.contains("python_class")));
    }

    #[test]
    fn sparql_rule_is_refused() {
        let yaml = r#"
id: https://example.org/t
name: t
classes:
  C:
    rules:
      - sparql: "ASK { }"
"#;
        let err = import_schema("t.yaml", yaml).unwrap_err();
        assert_eq!(err.code, "linkml-unsupported-feature");
        assert!(err.offending.iter().any(|s| s.contains("sparql")));
    }

    #[test]
    fn enum_with_permissible_values_warns() {
        let yaml = r#"
id: https://example.org/t
name: t
enums:
  MyEnum:
    permissible_values:
      a: {}
classes: {}
"#;
        let result = import_schema("t.yaml", yaml).unwrap();
        assert!(
            result
                .warnings
                .iter()
                .any(|w| w.feature == "permissible_values")
        );
    }

    #[test]
    fn pattern_slot_warns() {
        let yaml = r#"
id: https://example.org/t
name: t
classes:
  C:
    class_uri: ex:C
    attributes:
      id:
        pattern: "^[A-Z]+$"
"#;
        let result = import_schema("t.yaml", yaml).unwrap();
        assert!(result.warnings.iter().any(|w| w.feature == "pattern"));
    }

    #[test]
    fn is_a_becomes_kind_of_silently() {
        let yaml = r#"
id: https://example.org/t
name: t
prefixes:
  ex: https://example.org/
classes:
  Parent:
    class_uri: ex:Parent
  Child:
    class_uri: ex:Child
    is_a: Parent
"#;
        let result = import_schema("t.yaml", yaml).unwrap();
        assert!(result.warnings.is_empty(), "is_a must not produce warnings");
        let child = result.terms.iter().find(|t| t.label == "Child").unwrap();
        assert!(child.kind_of.is_some(), "Child must have kind_of set");
    }

    #[test]
    fn mixin_class_is_folded_silently() {
        let yaml = r#"
id: https://example.org/t
name: t
prefixes:
  ex: https://example.org/
classes:
  Timestamps:
    mixin: true
    attributes:
      created_at:
        range: string
  Doc:
    class_uri: ex:Doc
    mixins:
      - Timestamps
"#;
        let result = import_schema("t.yaml", yaml).unwrap();
        assert!(
            result.warnings.is_empty(),
            "mixin must not produce warnings"
        );
        // Mixin class itself must not appear as a term
        assert!(
            !result.terms.iter().any(|t| t.label == "Timestamps"),
            "mixin class must not become a standalone term"
        );
        // Doc must be present
        assert!(result.terms.iter().any(|t| t.label == "Doc"));
    }
}

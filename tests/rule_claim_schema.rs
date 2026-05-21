// Tests for dont-rule-claim-schema spec alignment.
//
// Spec: openspec/specs/dont-rule-claim-schema/spec.md
// Ticket: dont-uka4
//
// Coverage:
//   Req: Slot marker format and template
//     - template discoverable via `dont help --howto rule-claims` (scenario: template is discoverable via help)
//     - `.dont/AGENTS.md` contains canonical template (scenario: template is in agent-facing docs)
//     - claim authored from full canonical template passes rule-claim-structure (scenario: claim authored from template passes structural validation)

mod common;

use common::dont;
use std::fs;

// ── Requirement: Slot marker format and template ──────────────────────────────

/// Spec scenario: "template is discoverable via help"
/// WHEN the caller runs `dont help --howto rule-claims`
/// THEN the output includes the canonical template and per-slot guidance.
#[test]
fn help_howto_rule_claims_exits_successfully() {
    dont()
        .args(["help", "--howto", "rule-claims"])
        .assert()
        .success();
}

/// Spec scenario: "template is discoverable via help" — content check
/// WHEN the caller runs `dont help --howto rule-claims`
/// THEN the output includes the mandatory slot markers [TRIGGER] and [CONFIG]/[MODE].
#[test]
fn help_howto_rule_claims_includes_mandatory_slot_markers() {
    let output = dont()
        .args(["help", "--howto", "rule-claims"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8_lossy(&output);
    assert!(
        s.contains("[TRIGGER]"),
        "rule-claims how-to should include [TRIGGER] slot marker: {s}"
    );
    assert!(
        s.contains("[CONFIG]") || s.contains("[MODE]"),
        "rule-claims how-to should include [CONFIG] or [MODE] slot marker: {s}"
    );
}

/// Spec scenario: "template is discoverable via help" — optional slots listed
/// WHEN the caller runs `dont help --howto rule-claims`
/// THEN the output includes per-slot guidance covering optional slots.
#[test]
fn help_howto_rule_claims_includes_optional_slot_guidance() {
    let output = dont()
        .args(["help", "--howto", "rule-claims"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8_lossy(&output);
    assert!(
        s.contains("[INVOCATION]") || s.contains("INVOCATION"),
        "rule-claims how-to should mention INVOCATION slot: {s}"
    );
    assert!(
        s.contains("[GUARD]") || s.contains("GUARD"),
        "rule-claims how-to should mention GUARD slot: {s}"
    );
}

/// Spec scenario: "template is discoverable via help" — listed in topics
/// WHEN the caller runs `dont help --topics`
/// THEN the output lists the rule-claims how-to as an available topic.
#[test]
fn help_topics_lists_rule_claims_howto() {
    let output = dont()
        .args(["help", "--topics"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8_lossy(&output);
    assert!(
        s.contains("rule-claims") || s.contains("rule-claim"),
        "help --topics should list the rule-claims how-to: {s}"
    );
}

/// Spec scenario: "template is in agent-facing docs"
/// WHEN an agent reads `.dont/AGENTS.md` in the dont project itself
/// THEN the rule claim authoring section includes the canonical template with [TRIGGER] and [CONFIG]/[MODE].
#[test]
fn dont_project_agents_md_includes_rule_claim_template() {
    // The static .dont/AGENTS.md in the dont project itself is what agents read.
    // It should contain the rule claim authoring section and canonical template.
    let agents_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(".dont/AGENTS.md");
    let agents = fs::read_to_string(&agents_path)
        .unwrap_or_else(|e| panic!("could not read .dont/AGENTS.md: {e}"));
    assert!(
        agents.contains("Rule Claim")
            || agents.contains("rule claim")
            || agents.contains("rule-claim"),
        ".dont/AGENTS.md should include a rule claim authoring section: {agents}"
    );
    assert!(
        agents.contains("[TRIGGER]"),
        ".dont/AGENTS.md rule claim section should include [TRIGGER] slot marker: {agents}"
    );
    assert!(
        agents.contains("[CONFIG]") || agents.contains("[MODE]"),
        ".dont/AGENTS.md rule claim section should include [CONFIG] or [MODE] slot marker: {agents}"
    );
    assert!(
        agents.contains("[INVOCATION]"),
        ".dont/AGENTS.md rule claim section should include [INVOCATION] optional slot: {agents}"
    );
}

/// Spec scenario: "claim authored from template passes structural validation"
/// WHEN an author creates a rule claim using the canonical template with both mandatory slots filled
/// AND the claim includes the rule-claim-type term:uuid in its depends_on
/// THEN rule-claim-structure does not flag the claim.
///
/// This is an integration check via the rule module — see unit tests in
/// src/rules/rule_claim_structure.rs for the full coverage. This test
/// verifies the canonical template strings from the spec pass the rule.
#[test]
fn canonical_template_claim_passes_rule_claim_structure() {
    use dont::config::RuleClaimStructureConfig;
    use dont::rules::rule_claim_structure;
    use dont::store::Store;
    use tempfile::TempDir;

    let dir = TempDir::new().unwrap();
    let store = Store::open_dont_dir(dir.path()).unwrap();
    let tag_term = store.append_term("ns:rule-claim-type", "", None).unwrap();

    // Canonical template from the spec (mandatory slots filled, optional omitted as spec allows)
    let canonical_claim = concat!(
        "[INVOCATION] ungrounded runs as: background lint\n",
        "[CONFIG]     Enabled by default: yes\n",
        "[MODE]       In permissive mode: warn\n",
        "[TRIGGER]    Fires when: a claim has no evidence entry\n",
        "[GUARD]      Silently skips: locked claims\n",
        "[EVAL]       Evaluation model: stateless demand\n",
        "[BOUNDARY]   Does not handle: claims with external URIs; defers to unresolved-terms",
    );

    store
        .append_claim(canonical_claim, std::slice::from_ref(&tag_term.id), None)
        .unwrap();

    let config = RuleClaimStructureConfig {
        enabled: true,
        tag_term_id: Some(tag_term.id.clone()),
    };
    let matches = rule_claim_structure::check(&store, &config).unwrap();
    assert!(
        matches.is_empty(),
        "canonical template claim should pass rule-claim-structure: {matches:?}"
    );
}

mod common;

use common::{dont, init_dir};
use serde_json::Value;
use tempfile::TempDir;

// --- Unicode in claim content strings ---

#[test]
fn conclude_japanese_content_succeeds_and_roundtrips() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let statement = "Claim with 日本語 content";
    let out = dont()
        .args(["conclude", statement, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["statement"], statement);
}

#[test]
fn conclude_accented_french_content_succeeds() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let statement = "Hélas, être ou ne pas être";
    let out = dont()
        .args(["conclude", statement, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["statement"], statement);
}

#[test]
fn conclude_emoji_content_succeeds() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let statement = "🔬 Scientific claim";
    let out = dont()
        .args(["conclude", statement, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["statement"], statement);
}

// --- Unicode in term label / doc strings ---

#[test]
fn define_unicode_label_and_doc_succeeds() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    // Label must begin with "a"/"an" per SK11 §2.1.1(i); use "a ラベル" so
    // the indefinite-article check passes while the noun head is Japanese.
    let out = dont()
        .args([
            "define",
            "NS:001",
            "--label",
            "a ラベル",
            "--doc",
            "a ラベル — 説明文",
            "--json",
        ])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["curie"], "NS:001");
    assert_eq!(v["data"]["label"], "a ラベル");
    assert_eq!(v["data"]["definition"], "a ラベル — 説明文");
}

// --- Unicode in evidence excerpts ---

#[test]
fn conclude_arabic_content_succeeds_and_roundtrips() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    // Arabic right-to-left text as evidence excerpt content in a claim
    let statement = "البيانات العلمية دقيقة";
    let out = dont()
        .args(["conclude", statement, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["statement"], statement);
}

#[test]
fn conclude_cjk_content_returns_exact_string_in_output() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    // Mixed CJK (Chinese + Japanese + Korean)
    let statement = "科学的な主張 — 과학적 주장";
    let out = dont()
        .args(["conclude", statement, "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(
        v["data"]["statement"], statement,
        "CJK statement must roundtrip without mojibake"
    );
}

//! Spec-alignment test: the full 5×6 state transition matrix.
//!
//! Every (input_status, transition_command) pair is verified against the
//! dont-status-lifecycle spec as a single parameterized matrix, forming a
//! formal specification document rendered as executable code.
//!
//! Individual transition tests (validity + error messages) live in
//! tests/model.rs. This file tests the matrix as a whole: every entry,
//! completeness, and error-code consistency.

use dont::model::{Status, TransitionError, flag, ignore, lock, reopen, trust, undoubt};

type R = Result<Status, TransitionError>;

use Status::*;

/// Build the full 5x6 transition table.
///
/// Source order: [Unverified, Verified, Doubted, Ignored, Locked]
/// Each row has 5 entries. Valid transitions use Ok(target); refused
/// transitions use the actual Err from calling the function.
#[allow(clippy::type_complexity)]
fn data() -> Vec<(&'static str, fn(Status) -> R, Vec<R>)> {
    vec![
        (
            "trust",
            trust as fn(Status) -> R,
            vec![
                Ok(Doubted),
                Ok(Doubted),
                trust(Doubted),
                trust(Ignored),
                trust(Locked),
            ],
        ),
        (
            "flag",
            flag as fn(Status) -> R,
            vec![
                Ok(Verified),
                flag(Verified),
                Ok(Verified),
                flag(Ignored),
                flag(Locked),
            ],
        ),
        (
            "undoubt",
            undoubt as fn(Status) -> R,
            vec![
                undoubt(Unverified),
                undoubt(Verified),
                Ok(Unverified),
                undoubt(Ignored),
                undoubt(Locked),
            ],
        ),
        (
            "ignore",
            ignore as fn(Status) -> R,
            vec![
                Ok(Ignored),
                Ok(Ignored),
                Ok(Ignored),
                ignore(Ignored),
                ignore(Locked),
            ],
        ),
        (
            "reopen",
            reopen as fn(Status) -> R,
            vec![
                reopen(Unverified),
                reopen(Verified),
                reopen(Doubted),
                Ok(Unverified),
                reopen(Locked),
            ],
        ),
        (
            "lock",
            lock as fn(Status) -> R,
            vec![
                lock(Unverified),
                Ok(Locked),
                lock(Doubted),
                lock(Ignored),
                lock(Locked),
            ],
        ),
    ]
}

const SOURCES: &[Status] = &[Unverified, Verified, Doubted, Ignored, Locked];

/// Every entry in the 5x6 transition matrix matches the spec.
#[test]
fn full_transition_matrix_matches_spec() {
    let mut fails = Vec::new();

    for (cmd_name, cmd_fn, row) in data() {
        for (i, source) in SOURCES.iter().enumerate() {
            let expected = &row[i];
            let actual = (cmd_fn)(*source);

            let msg = match (&actual, expected) {
                (Ok(a), Ok(e)) if a != e => {
                    format!(
                        "{}({:?}) expected Ok({:?}), got Ok({:?})",
                        cmd_name, source, e, a
                    )
                }
                (Ok(a), Err(_)) => {
                    format!("{}({:?}) expected Err, got Ok({:?})", cmd_name, source, a)
                }
                (Err(e), Ok(exp)) => {
                    format!(
                        "{}({:?}) expected Ok({:?}), got Err({})",
                        cmd_name, source, exp, e.code
                    )
                }
                (Err(e), Err(_)) if e.code != "invalid-transition" => {
                    format!(
                        "{}({:?}) expected code invalid-transition, got {}",
                        cmd_name, source, e.code
                    )
                }
                _ => continue,
            };

            fails.push(msg);
        }
    }

    assert!(
        fails.is_empty(),
        "Transition matrix spec mismatch ({} failure(s)):\n  {}",
        fails.len(),
        fails.join("\n  ")
    );
}

/// The matrix has exactly 6 x 5 = 30 documented transitions.
#[test]
fn matrix_has_30_entries() {
    let rows = data();
    assert_eq!(rows.len(), 6, "6 commands");
    for (name, _, row) in &rows {
        assert_eq!(row.len(), 5, "{} should have 5 source-status entries", name);
    }
}

/// Exactly 10 valid transition paths in the matrix.
#[test]
fn matrix_has_exactly_10_valid_entries() {
    let n: usize = data()
        .iter()
        .flat_map(|(_, _, row)| row.iter())
        .filter(|e| e.is_ok())
        .count();
    assert_eq!(n, 10, "spec requires exactly 10 valid transition paths");
}

/// Locked and Ignored reject all normal transitions (closure states per spec).
/// reopen is excluded because it is the explicit escape hatch for Ignored.
#[test]
fn locked_and_ignored_reject_all_normal_transitions() {
    #[allow(clippy::type_complexity)]
    let normal_cmds: Vec<(&str, fn(Status) -> R)> = data()
        .into_iter()
        .filter(|(name, _, _)| *name != "reopen")
        .map(|(name, f, _)| (name, f))
        .collect();

    for (name, cmd_fn) in &normal_cmds {
        for source in &[Locked, Ignored] {
            assert!(
                (cmd_fn)(*source).is_err(),
                "{}({:?}) should be refused — closure state",
                name,
                source
            );
        }
    }
}

/// All refused transitions use the canonical error code.
#[test]
fn refused_transitions_use_canonical_code() {
    for source in SOURCES {
        for (name, cmd_fn, _) in data() {
            if let Err(e) = (cmd_fn)(*source) {
                assert_eq!(
                    e.code, "invalid-transition",
                    "{}({:?}) should use invalid-transition, got {}",
                    name, source, e.code
                );
            }
        }
    }
}

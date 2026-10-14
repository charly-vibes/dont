#!/usr/bin/env python3
"""Tests for scripts/pretender-gate (gh #29 / dont-3rzr).

The gate contract:
- fails (exit 1) only on NEW violations vs the committed baseline
  (.pretender/last-check.json) or on files pretender could not parse
  (Error-severity diagnostics);
- never mutates the baseline during a gate run — a failed run must not
  absorb its own violations (the snapshot would otherwise advance past
  the gate on the next run);
- exits 1 with a clear message when no baseline exists;
- `--record` deliberately rewrites the baseline from the current scan.

Run: python3 scripts/test_pretender_gate.py
"""

import pathlib
import shutil
import subprocess
import sys
import tempfile

REPO = pathlib.Path(__file__).resolve().parent.parent
GATE = REPO / "scripts" / "pretender-gate"

TOML = """\
[pretender]
mode = "tiered"
advisory_until = "2099-12-31"
languages = ["rust"]
exclude = ["target/**"]

[roles.test]
paths = ["tests/**"]
"""

CLEAN_SRC = """\
pub fn add(a: u32, b: u32) -> u32 {
    a + b
}
"""

# 60-line function trips app.function_lines_max (default 40).
BAD_SRC = "\n".join(
    ["pub fn giant(x: u32) -> u32 {"] + [f"    let v{i} = x + {i};" for i in range(58)]
    + ["    v0", "}"]
)

# Broken Rust triggers an Error-severity parse diagnostic.
PARSE_ERR_SRC = "pub fn broken( {\n"


def run_gate(workdir, *args):
    return subprocess.run(
        [str(GATE), *args], cwd=workdir, capture_output=True, text=True, timeout=120
    )


def make_project(tmp, files):
    root = pathlib.Path(tmp)
    (root / "src").mkdir()
    (root / "pretender.toml").write_text(TOML)
    for name, content in files.items():
        path = root / "src" / name
        path.write_text(content)
    return root


def baseline(root):
    return root / ".pretender" / "history" / "last-findings.json"


def case_record_then_pass():
    with tempfile.TemporaryDirectory() as tmp:
        root = make_project(tmp, {"ok.rs": CLEAN_SRC})
        r = run_gate(root, "--record")
        assert r.returncode == 0, f"--record failed: {r.stderr}"
        assert baseline(root).exists(), "--record did not write a baseline"
        before = baseline(root).read_bytes()
        r = run_gate(root)
        assert r.returncode == 0, f"clean tree should pass: {r.stdout} {r.stderr}"
        assert baseline(root).read_bytes() == before, "gate run mutated the baseline"
    print("ok: --record + clean tree passes")


def case_new_violation_gates_and_does_not_absorb():
    with tempfile.TemporaryDirectory() as tmp:
        root = make_project(tmp, {"ok.rs": CLEAN_SRC})
        assert run_gate(root, "--record").returncode == 0
        before = baseline(root).read_bytes()

        (root / "src" / "bad.rs").write_text(BAD_SRC)
        r = run_gate(root)
        assert r.returncode == 1, f"new violation must fail the gate: {r.stdout}"
        assert "bad.rs" in r.stdout, f"output must name the violating file: {r.stdout}"
        assert baseline(root).read_bytes() == before, "failed run absorbed its violation"

        # Second run: still fails — proof the first failure did not advance
        # the baseline (the snapshot is persisted before the exit decision).
        r = run_gate(root)
        assert r.returncode == 1, "violation disappeared after failed run — baseline was absorbed"

        (root / "src" / "bad.rs").unlink()
        assert run_gate(root).returncode == 0, "resolution should restore a green gate"
    print("ok: new violation gates repeatedly, resolution passes")


def case_missing_baseline_fails():
    with tempfile.TemporaryDirectory() as tmp:
        root = make_project(tmp, {"ok.rs": CLEAN_SRC})
        r = run_gate(root)
        assert r.returncode == 1, "missing baseline must fail closed"
        combined = r.stdout + r.stderr
        assert "baseline" in combined.lower(), f"unhelpful error: {combined}"
    print("ok: missing baseline fails closed with guidance")


def case_parse_error_gates():
    with tempfile.TemporaryDirectory() as tmp:
        root = make_project(tmp, {"ok.rs": CLEAN_SRC})
        assert run_gate(root, "--record").returncode == 0
        before = baseline(root).read_bytes()

        (root / "src" / "broken.rs").write_text(PARSE_ERR_SRC)
        r = run_gate(root)
        assert r.returncode == 1, "unparsable file must fail the gate, not slip through"
        assert "broken.rs" in r.stdout, f"output must name the skipped file: {r.stdout}"
        assert baseline(root).read_bytes() == before, "failed run mutated the baseline"
    print("ok: parse errors gate")


def main():
    assert GATE.exists(), f"gate script missing: {GATE}"
    assert shutil.which("pretender"), "pretender must be on PATH"
    failures = []
    for case in (
        case_record_then_pass,
        case_new_violation_gates_and_does_not_absorb,
        case_missing_baseline_fails,
        case_parse_error_gates,
    ):
        try:
            case()
        except AssertionError as e:
            failures.append(f"{case.__name__}: {e}")
    if failures:
        print("\nFAILURES:")
        for f in failures:
            print(f"  {f}")
        sys.exit(1)
    print("\nall pretender-gate tests passed")


if __name__ == "__main__":
    main()

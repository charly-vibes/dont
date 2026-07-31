//! Managed block injector for the `<!-- DONT:START/END -->` block.
//!
//! Core injector types sourced from `genesis::managed_block`; dont-specific
//! content and helper functions remain here.

use std::fs;
use std::io;
use std::path::Path;

use crate::fs_util::write_restricted;

// Re-export genesis types for callers that need them.
pub use genesis::managed_block::{BlockDef, BlockInjector, BlockRegistry, InjectResult};

/// The DONT block start marker.
pub const START_MARKER: &str = "<!-- DONT:START -->";
/// The DONT block end marker.
pub const END_MARKER: &str = "<!-- DONT:END -->";

/// Render the DONT managed block around the given body.
///
/// Delegates to `genesis::aix::agents_block` for consistent managed block
/// generation across the genesis suite.
pub fn render_root_block(body: &str) -> String {
    genesis::aix::agents_block("DONT", body)
}

/// Normalize text for comparison (strip trailing whitespace, unify line endings).
pub fn normalize_for_compare(text: &str) -> String {
    let unified = text.replace("\r\n", "\n").replace('\r', "\n");
    let lines: Vec<String> = unified
        .split('\n')
        .map(|line| line.trim_end().to_string())
        .collect();
    lines.join("\n").trim_end().to_string()
}

/// Find the bounds of the DONT managed region in text.
fn managed_region_bounds(text: &str) -> Option<(usize, usize)> {
    let start = text.find(START_MARKER)?;
    let end_marker_start = text[start..].find(END_MARKER)? + start;
    let end = end_marker_start + END_MARKER.len();
    Some((start, end))
}

/// Build a [`BlockInjector`] pre-configured with the DONT block.
fn dont_injector() -> BlockInjector {
    let mut reg = BlockRegistry::new();
    reg.register(BlockDef::new("DONT"));
    BlockInjector::new(reg)
}

/// Read the DONT managed block from a file.
///
/// Returns `None` if the file doesn't exist or has no DONT block.
pub fn read_root_block(path: &Path) -> io::Result<Option<String>> {
    let injector = dont_injector();
    Ok(injector.read_block(path, "DONT"))
}

/// Check if a file's DONT block matches the expected content.
pub fn root_block_matches(path: &Path, expected_block: &str) -> io::Result<bool> {
    let Some(actual) = read_root_block(path)? else {
        return Ok(false);
    };
    Ok(normalize_for_compare(&actual) == normalize_for_compare(expected_block))
}

/// Replace or prepend the DONT managed block in a file.
///
/// `expected_block` should be the fully rendered block (including markers),
/// as produced by [`render_root_block`].
pub fn replace_or_prepend_root_block(path: &Path, expected_block: &str) -> io::Result<()> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            return fs::write(path, format!("{expected_block}\n"));
        }
        Err(err) => return Err(err),
    };

    let updated = if let Some((start, end)) = managed_region_bounds(&text) {
        let mut updated = String::with_capacity(text.len() - (end - start) + expected_block.len());
        updated.push_str(&text[..start]);
        updated.push_str(expected_block);
        updated.push_str(&text[end..]);
        updated
    } else if text.is_empty() {
        format!("{expected_block}\n")
    } else {
        format!("{expected_block}\n\n{text}")
    };

    fs::write(path, updated)
}

/// Check if a file's full content matches the expected string.
pub fn file_matches(path: &Path, expected: &str) -> io::Result<bool> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(err) => return Err(err),
    };
    Ok(normalize_for_compare(&text) == normalize_for_compare(expected))
}

/// Write canonical content to a file using restricted write.
pub fn write_canonical(path: &Path, expected: &str) -> io::Result<()> {
    write_restricted(path, expected.as_bytes())
}

use std::fs;
use std::io;
use std::path::Path;

pub const START_MARKER: &str = "<!-- DONT:START -->";
pub const END_MARKER: &str = "<!-- DONT:END -->";

pub fn render_root_block(body: &str) -> String {
    format!("{START_MARKER}\n{body}\n{END_MARKER}")
}

pub fn normalize_for_compare(text: &str) -> String {
    let unified = text.replace("\r\n", "\n").replace('\r', "\n");
    let lines: Vec<String> = unified
        .split('\n')
        .map(|line| line.trim_end().to_string())
        .collect();
    lines.join("\n").trim_end().to_string()
}

fn managed_region_bounds(text: &str) -> Option<(usize, usize)> {
    let start = text.find(START_MARKER)?;
    let end_marker_start = text[start..].find(END_MARKER)? + start;
    let end = end_marker_start + END_MARKER.len();
    Some((start, end))
}

pub fn read_root_block(path: &Path) -> io::Result<Option<String>> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(err) => return Err(err),
    };
    Ok(managed_region_bounds(&text).map(|(start, end)| text[start..end].to_string()))
}

pub fn root_block_matches(path: &Path, expected_block: &str) -> io::Result<bool> {
    let Some(actual) = read_root_block(path)? else {
        return Ok(false);
    };
    Ok(normalize_for_compare(&actual) == normalize_for_compare(expected_block))
}

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

pub fn file_matches(path: &Path, expected: &str) -> io::Result<bool> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(err) => return Err(err),
    };
    Ok(normalize_for_compare(&text) == normalize_for_compare(expected))
}

pub fn write_canonical(path: &Path, expected: &str) -> io::Result<()> {
    fs::write(path, expected)
}

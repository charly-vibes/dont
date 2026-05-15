use std::fs::OpenOptions;
use std::path::Path;

#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;

/// Write `content` to `path` with 0o600 permissions on Unix, creating or truncating the file.
pub fn write_restricted(path: &Path, content: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    #[cfg(unix)]
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .mode(0o600)
        .open(path)?;
    #[cfg(not(unix))]
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)?;
    file.write_all(content)
}

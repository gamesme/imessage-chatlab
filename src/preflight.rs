//! Fail-fast database readability checks. Run before `Config::new` so that
//! permission errors and wrong-format files don't waste cache-build time.

use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

use crate::error::RuntimeError;

/// Minimum bytes to read to verify SQLite magic.
const SQLITE_MAGIC: &[u8; 16] = b"SQLite format 3\0";

/// Check that `path` exists, is readable, and starts with the SQLite magic
/// header. Returns descriptive errors otherwise.
pub fn check_db_readable(path: &Path) -> Result<(), RuntimeError> {
    let mut f = File::open(path).map_err(|e| match e.kind() {
        io::ErrorKind::PermissionDenied => RuntimeError::InvalidOptions(format!(
            "Cannot read {}: Permission denied.\n\
             On macOS, grant Full Disk Access to your terminal:\n  \
             System Settings → Privacy & Security → Full Disk Access",
            path.display()
        )),
        io::ErrorKind::NotFound => RuntimeError::InvalidOptions(format!(
            "Database file not found: {}",
            path.display()
        )),
        _ => RuntimeError::InvalidOptions(format!(
            "Cannot open {}: {}",
            path.display(),
            e
        )),
    })?;
    let mut buf = [0u8; 16];
    f.read_exact(&mut buf).map_err(|e| {
        RuntimeError::InvalidOptions(format!("Cannot read header of {}: {}", path.display(), e))
    })?;
    if &buf != SQLITE_MAGIC {
        return Err(RuntimeError::InvalidOptions(format!(
            "Not a SQLite database: {}",
            path.display()
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::path::PathBuf;

    fn fixture_db() -> PathBuf {
        PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/test_data/db/test.db"))
    }

    #[test]
    fn check_db_readable_accepts_real_sqlite() {
        check_db_readable(&fixture_db()).expect("fixture DB should pass");
    }

    #[test]
    fn check_db_readable_rejects_missing_file() {
        let result = check_db_readable(Path::new("/tmp/nonexistent-imchatlab-test.db"));
        let err = result.expect_err("missing file should error");
        let msg = format!("{err}");
        assert!(msg.contains("not found") || msg.contains("Database file"));
    }

    #[test]
    fn check_db_readable_rejects_non_sqlite() {
        let path = std::env::temp_dir().join("imchatlab-non-sqlite-test.bin");
        let mut f = File::create(&path).unwrap();
        f.write_all(b"this is not sqlite          ").unwrap();
        let result = check_db_readable(&path);
        let _ = std::fs::remove_file(&path);
        let err = result.expect_err("non-sqlite file should error");
        assert!(format!("{err}").contains("Not a SQLite database"));
    }
}

//! Internal logging primitives. Provides `info!` (suppressible) and a global
//! quiet flag set once at startup.
//!
//! Use `info!` for status/progress lines and plain `eprintln!` for errors.

use std::sync::OnceLock;

static QUIET: OnceLock<bool> = OnceLock::new();

/// Called once from `main.rs` after CLI parsing. Subsequent calls are ignored
/// (a OnceLock allows only one successful set).
pub fn set_quiet(quiet: bool) {
    let _ = QUIET.set(quiet);
}

pub fn is_quiet() -> bool {
    *QUIET.get().unwrap_or(&false)
}

/// Print to stderr if not in quiet mode. Same formatting as `eprintln!`.
#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        if !$crate::logging::is_quiet() {
            eprintln!($($arg)*);
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_quiet_is_false() {
        // Don't call set_quiet so OnceLock stays uninitialized for this test.
        // Test fixture isolation is tricky with global state; this test must run
        // before any other test in the file or be run in its own process.
        // Using a separate `not_set_quiet` test only.
        assert!(!is_quiet());
    }
}

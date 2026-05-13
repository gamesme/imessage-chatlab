//! End-to-end check: --dry-run must not write any files.

use std::path::PathBuf;
use std::process::Command;

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn dry_run_does_not_write_files() {
    let target = std::env::temp_dir().join("imchatlab-dry-run-it");
    let _ = std::fs::remove_dir_all(&target);

    // Force a debug build of the binary before testing.
    let status = Command::new("cargo")
        .args(["build"])
        .current_dir(project_root())
        .status()
        .expect("cargo build");
    assert!(status.success(), "cargo build failed");

    let exe = project_root().join("target/debug/imessage-chatlab");
    let db = project_root().join("test_data/db/test.db");

    let output = Command::new(&exe)
        .args([
            "--dry-run",
            "-p",
            db.to_str().unwrap(),
            "-o",
            target.to_str().unwrap(),
            "--no-timestamp",
        ])
        .output()
        .expect("run imessage-chatlab --dry-run");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "expected exit 0, got {:?}\nstderr:\n{stderr}",
        output.status
    );

    assert!(
        !target.exists(),
        "--dry-run should not have created {}",
        target.display()
    );

    assert!(
        stderr.contains("dry run") || stderr.contains("Chats:") || stderr.contains("Messages:"),
        "expected preview output, got:\n{stderr}"
    );
}

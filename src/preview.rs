//! Builds and prints the `ExportPreview` — a structured summary of what an
//! export *would* do, without writing any files.
//!
//! Used by:
//!   - `--dry-run` (skips writing entirely after building the preview)
//!   - Wizard Screen 7 (summary + confirm gate before kicking off the export)

use std::path::PathBuf;

use imessage_database::tables::messages::Message;

use crate::error::RuntimeError;
use crate::runtime::Config;

#[derive(Debug, Clone)]
pub struct ExportPreview {
    pub source_path: PathBuf,
    /// Display name of the source platform (e.g. "macOS", "iOS").
    pub source_platform: String,
    pub chat_count: usize,
    pub selected_chat_count: usize,
    pub message_count: u64,
    pub selected_message_count: u64,
    /// (start, end) Unix-epoch seconds. None if no messages selected.
    pub date_range: Option<(i64, i64)>,
    pub attachment_total_bytes: u64,
    pub output_path: PathBuf,
}

/// Build a preview from the fully initialized Config (post-`resolve_filtered_handles`).
pub fn build(config: &Config) -> Result<ExportPreview, RuntimeError> {
    let db = config.data_source.db();

    let chat_count = config.chatrooms.len();
    let selected_chat_count = config
        .options
        .query_context
        .selected_chat_ids
        .as_ref()
        .map(|s| s.len())
        .unwrap_or(chat_count);

    // get_count returns i64; cast to u64 (clamping negatives to 0)
    let message_count = Message::get_count(db, &Default::default())
        .map(|n| n.max(0) as u64)
        .map_err(RuntimeError::DatabaseError)?;
    let selected_message_count = Message::get_count(db, &config.options.query_context)
        .map(|n| n.max(0) as u64)
        .map_err(RuntimeError::DatabaseError)?;

    let date_range: Option<(i64, i64)> = {
        let mut stmt = db
            .prepare("SELECT MIN(date), MAX(date) FROM message")
            .map_err(|e| RuntimeError::InvalidOptions(format!("preview min/max prep: {e}")))?;
        stmt.query_row([], |r| {
            Ok((r.get::<_, Option<i64>>(0)?, r.get::<_, Option<i64>>(1)?))
        })
        .map(|(lo, hi)| match (lo, hi) {
            (Some(a), Some(b)) => Some((normalize(a, config.offset), normalize(b, config.offset))),
            _ => None,
        })
        .map_err(|e| RuntimeError::InvalidOptions(format!("preview range: {e}")))?
    };

    let attachment_total_bytes: u64 = {
        let mut stmt = db
            .prepare("SELECT IFNULL(SUM(total_bytes), 0) FROM attachment")
            .map_err(|e| RuntimeError::InvalidOptions(format!("preview attach prep: {e}")))?;
        stmt.query_row([], |r| r.get::<_, i64>(0))
            .map(|n| n.max(0) as u64)
            .map_err(|e| RuntimeError::InvalidOptions(format!("preview attach: {e}")))?
    };

    Ok(ExportPreview {
        source_path: config.options.db_path.clone(),
        source_platform: config.options.platform.to_string(),
        chat_count,
        selected_chat_count,
        message_count,
        selected_message_count,
        date_range,
        attachment_total_bytes,
        output_path: config.options.export_path.clone(),
    })
}

fn normalize(date: i64, offset: i64) -> i64 {
    const TIMESTAMP_FACTOR: i64 = 1_000_000_000;
    let secs = if date >= 1_000_000_000_000 {
        date / TIMESTAMP_FACTOR
    } else {
        date
    };
    secs + offset
}

/// Print a human-readable preview to stderr.
pub fn print(p: &ExportPreview) {
    eprintln!();
    eprintln!("  Source:       {} ({})", p.source_path.display(), p.source_platform);
    eprintln!(
        "  Chats:        {} of {} selected",
        p.selected_chat_count, p.chat_count
    );
    eprintln!(
        "  Messages:     {} of {} (after filters)",
        format_int(p.selected_message_count),
        format_int(p.message_count)
    );
    if let Some((start, end)) = p.date_range {
        eprintln!(
            "  Date range:   {} to {}",
            format_unix_date(start),
            format_unix_date(end)
        );
    } else {
        eprintln!("  Date range:   (no messages)");
    }
    eprintln!(
        "  Attachments:  ~{} (estimated)",
        format_bytes(p.attachment_total_bytes)
    );
    eprintln!("  Output:       {}", p.output_path.display());
    eprintln!();
}

fn format_int(n: u64) -> String {
    // Inserts thousands separators: 12841 → "12,841"
    let s = n.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    out.chars().rev().collect()
}

fn format_bytes(n: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    if n >= GB {
        format!("{:.1} GB", n as f64 / GB as f64)
    } else if n >= MB {
        format!("{:.1} MB", n as f64 / MB as f64)
    } else if n >= KB {
        format!("{:.1} KB", n as f64 / KB as f64)
    } else {
        format!("{n} B")
    }
}

fn format_unix_date(secs: i64) -> String {
    chrono::DateTime::<chrono::Utc>::from_timestamp(secs, 0)
        .map(|dt| dt.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "?".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::options::{ExportType, Options};

    fn make_app() -> Config {
        Config::fake_app(Options::fake_options(ExportType::Json))
    }

    #[test]
    fn build_does_not_error_on_fake_app() {
        let app = make_app();
        // fake_app has empty chatrooms; build() queries the real DB attached to
        // data_source. Just verify build doesn't error.
        let p = build(&app).unwrap();
        // chat_count comes from config.chatrooms which is empty in fake_app.
        assert_eq!(p.chat_count, 0);
    }

    #[test]
    fn build_message_count_is_non_zero_for_fixture() {
        let app = make_app();
        let p = build(&app).unwrap();
        // The fixture DB at test_data/db/test.db has real messages.
        assert!(p.message_count > 0);
    }

    #[test]
    fn format_bytes_thresholds() {
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(2048), "2.0 KB");
        assert_eq!(format_bytes(5 * 1024 * 1024), "5.0 MB");
        assert_eq!(format_bytes(3u64 * 1024 * 1024 * 1024), "3.0 GB");
    }

    #[test]
    fn format_int_thousands_separator() {
        assert_eq!(format_int(123), "123");
        assert_eq!(format_int(1234), "1,234");
        assert_eq!(format_int(12_345_678), "12,345,678");
    }
}

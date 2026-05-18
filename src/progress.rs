/*!
 Defines the export progress bar.
*/

use std::{
    cell::{Cell, RefCell},
    io::{self, Write},
    time::Instant,
};

const BAR_WIDTH: usize = 20;
const BAR_FILL: char = '#';
const BAR_ARROW: char = '>';
const BAR_EMPTY: char = ' ';
const MAX_CHAT_NAME_CHARS: usize = 40;

const HUMAN_COUNT_THRESHOLDS: [(u64, &str); 5] = [
    (1_000_000_000_000, "T"), // trillion
    (1_000_000_000, "B"),     // billion
    (1_000_000, "M"),         // million
    (1_000, "k"),             // thousand
    (0, ""),                  // no suffix
];

/// Format a number with comma separators
fn format_with_commas(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::with_capacity(s.len() + s.len() / 3);
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            result.push(',');
        }
        result.push(c);
    }
    result
}

/// Format a rate as a human-readable string with appropriate suffix
fn format_human_rate(rate: f64) -> String {
    let rate_u64 = rate as u64;
    for &(threshold, suffix) in &HUMAN_COUNT_THRESHOLDS {
        if rate_u64 >= threshold && threshold > 0 {
            let scaled = rate / threshold as f64;
            return format!("{scaled:.1}{suffix}");
        }
    }
    format!("{rate:.1}")
}

/// Truncate a chat name to at most `max_chars` characters, appending '…' when shortened.
/// Operates on Unicode scalar values so multi-byte names (e.g. CJK, emoji) are not split mid-codepoint.
fn truncate_chat_name(name: &str, max_chars: usize) -> String {
    if name.chars().count() <= max_chars {
        return name.to_string();
    }
    let keep = max_chars.saturating_sub(1);
    let head: String = name.chars().take(keep).collect();
    format!("{head}…")
}

/// Bespoke progress bar for iMessage exports.
pub struct ExportProgress {
    length: Cell<u64>,
    position: Cell<u64>,
    start_time: Cell<Option<Instant>>,
    message: RefCell<Option<String>>,
    current_chat: RefCell<Option<String>>,
}

impl ExportProgress {
    /// Creates a new hidden progress bar
    pub fn new() -> Self {
        Self {
            length: Cell::new(0),
            position: Cell::new(0),
            start_time: Cell::new(None),
            message: RefCell::new(None),
            current_chat: RefCell::new(None),
        }
    }

    /// Starts the progress bar with the specified total length
    pub fn start(&self, length: i64) {
        self.length.set(length.try_into().unwrap_or(0));
        self.position.set(0);
        self.start_time.set(Some(Instant::now()));
        self.draw();
    }

    /// Sets the position of the progress bar
    pub fn set_position(&self, pos: u64) {
        self.position.set(pos);
        self.draw();
    }

    /// Sets the chat name shown alongside the bar. Pass `None` to clear.
    /// Does not trigger a redraw on its own — the next `set_position`/`finish` call shows the update.
    pub fn set_current_chat(&self, name: Option<String>) {
        *self.current_chat.borrow_mut() = name;
    }

    /// Finishes the progress bar
    pub fn finish(&self) {
        self.position.set(self.length.get());
        self.draw();
        eprintln!();
    }

    /// Render the progress bar to stderr
    fn draw(&self) {
        let elapsed = self
            .start_time
            .get()
            .map(|t| t.elapsed())
            .unwrap_or_default();
        let elapsed_secs = elapsed.as_secs();

        let length = self.length.get();
        let position = self.position.get();

        // Build the bar: [##########>         ]
        let fraction = if length > 0 {
            position as f64 / length as f64
        } else {
            0.0
        };
        let filled = (fraction * BAR_WIDTH as f64) as usize;
        let mut bar = String::with_capacity(BAR_WIDTH);
        for i in 0..BAR_WIDTH {
            if i < filled {
                bar.push(BAR_FILL);
            } else if i == filled && filled < BAR_WIDTH {
                bar.push(BAR_ARROW);
            } else {
                bar.push(BAR_EMPTY);
            }
        }

        let pos_str = format_with_commas(position);
        let len_str = format_with_commas(length);

        // Rate/ETA or busy message
        let message = self.message.borrow();
        let rate_eta = if let Some(ref msg) = *message {
            format!("(ETA: N/A) {msg}")
        } else {
            let elapsed_f64 = elapsed.as_secs_f64();
            let rate = if elapsed_f64 > 0.0 {
                position as f64 / elapsed_f64
            } else {
                0.0
            };
            let eta = if rate > 0.0 {
                let remaining = length.saturating_sub(position) as f64 / rate;
                format!("{remaining:.0}s")
            } else {
                "N/A".to_string()
            };
            format!("({}/s, ETA: {eta})", format_human_rate(rate))
        };

        // Current chat name suffix
        let chat_suffix = {
            let cc = self.current_chat.borrow();
            match cc.as_deref() {
                Some(name) if !name.is_empty() => {
                    format!(" → {}", truncate_chat_name(name, MAX_CHAT_NAME_CHARS))
                }
                _ => String::new(),
            }
        };

        let line =
            format!("\r  [{elapsed_secs}s] [\x1b[36m{bar}\x1b[0m] {pos_str}/{len_str} {rate_eta}{chat_suffix}");

        let mut stderr = io::stderr().lock();
        // \x1b[K erases from cursor to end of line, clearing any leftover characters
        let _ = write!(stderr, "{line}\x1b[K");
        let _ = stderr.flush();
    }
}

impl Default for ExportProgress {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_with_commas() {
        assert_eq!(format_with_commas(0), "0");
        assert_eq!(format_with_commas(999), "999");
        assert_eq!(format_with_commas(1_000), "1,000");
        assert_eq!(format_with_commas(1_000_000), "1,000,000");
        assert_eq!(format_with_commas(234_399), "234,399");
    }

    #[test]
    fn test_format_human_rate() {
        assert_eq!(format_human_rate(500.0), "500.0");
        assert_eq!(format_human_rate(1_500.0), "1.5k");
        assert_eq!(format_human_rate(89_209.7), "89.2k");
        assert_eq!(format_human_rate(1_500_000.0), "1.5M");
        assert_eq!(format_human_rate(2_500_000_000.0), "2.5B");
        assert_eq!(format_human_rate(1_200_000_000_000.0), "1.2T");
    }

    #[test]
    fn truncate_chat_name_returns_name_when_within_limit() {
        assert_eq!(truncate_chat_name("Family", 40), "Family");
    }

    #[test]
    fn truncate_chat_name_returns_name_when_exactly_at_limit() {
        let name = "a".repeat(40);
        assert_eq!(truncate_chat_name(&name, 40), name);
    }

    #[test]
    fn truncate_chat_name_shortens_with_ellipsis_when_over_limit() {
        let name = "a".repeat(45);
        let truncated = truncate_chat_name(&name, 40);
        assert_eq!(truncated.chars().count(), 40);
        assert!(truncated.ends_with('…'));
        assert!(truncated.starts_with(&"a".repeat(39)));
    }

    #[test]
    fn truncate_chat_name_handles_multibyte_chars_without_panic() {
        // 30 Chinese chars (each 3 bytes in UTF-8) — must truncate by char count, not byte count
        let name: String = "群组".chars().cycle().take(60).collect();
        let truncated = truncate_chat_name(&name, 40);
        assert_eq!(truncated.chars().count(), 40);
        assert!(truncated.ends_with('…'));
    }

    #[test]
    fn set_current_chat_updates_field() {
        let pb = ExportProgress::new();
        pb.set_current_chat(Some("Family Group Chat".to_string()));
        assert_eq!(
            pb.current_chat.borrow().as_deref(),
            Some("Family Group Chat")
        );
        pb.set_current_chat(None);
        assert!(pb.current_chat.borrow().is_none());
    }
}

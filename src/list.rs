//! `imessage-chatlab list` subcommand implementation.

use jzon::JsonValue;

use crate::error::RuntimeError;
use crate::query::{ChatKind, ChatSummary, chat_summaries, relative_time};
use crate::runtime::Config;

pub fn run(config: &Config, json: bool) -> Result<(), RuntimeError> {
    let summaries = chat_summaries(config)?;
    if json {
        println!("{}", to_json(&summaries));
    } else {
        print_table(&summaries);
    }
    Ok(())
}

fn print_table(summaries: &[ChatSummary]) {
    let max_name_len = summaries
        .iter()
        .map(|s| display_width(&s.name).min(40))
        .max()
        .unwrap_or(20);

    println!(
        "{:>6}  {:<name_w$}  {:>10}  {:<14}  TYPE",
        "ROWID",
        "NAME",
        "MESSAGES",
        "LAST ACTIVE",
        name_w = max_name_len
    );
    println!(
        "{}",
        "-".repeat(6 + 2 + max_name_len + 2 + 10 + 2 + 14 + 2 + 7)
    );

    for s in summaries {
        let kind = match s.kind {
            ChatKind::Private => "private",
            ChatKind::Group => "group",
        };
        let active = s
            .last_active
            .map(relative_time)
            .unwrap_or_else(|| "\u{2014}".to_string());
        let trimmed_name = truncate(&s.name, max_name_len);
        println!(
            "{:>6}  {:<name_w$}  {:>10}  {:<14}  {}",
            s.rowid,
            trimmed_name,
            commafy(s.message_count),
            active,
            kind,
            name_w = max_name_len
        );
    }
}

fn truncate(s: &str, max: usize) -> String {
    if display_width(s) <= max {
        s.to_string()
    } else {
        // Reserve 2 display columns for the ellipsis (U+2026 is a wide char).
        let budget = max.saturating_sub(2);
        let mut out = String::new();
        let mut w = 0;
        for ch in s.chars() {
            let cw = if ch.is_ascii() { 1 } else { 2 };
            if w + cw > budget {
                break;
            }
            out.push(ch);
            w += cw;
        }
        out.push('\u{2026}');
        out
    }
}

fn display_width(s: &str) -> usize {
    s.chars().map(|c| if c.is_ascii() { 1 } else { 2 }).sum()
}

fn commafy(n: i64) -> String {
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

fn to_json(summaries: &[ChatSummary]) -> String {
    let mut arr = JsonValue::new_array();
    for s in summaries {
        let mut obj = JsonValue::new_object();
        obj.insert("rowid", s.rowid).unwrap();
        obj.insert("name", s.name.clone()).unwrap();
        obj.insert("message_count", s.message_count).unwrap();
        obj.insert(
            "last_active",
            s.last_active
                .map(|t| {
                    chrono::DateTime::<chrono::Utc>::from_timestamp(t, 0)
                        .map(|dt| dt.to_rfc3339())
                        .unwrap_or_default()
                })
                .unwrap_or_default(),
        )
        .unwrap();
        obj.insert(
            "type",
            match s.kind {
                ChatKind::Private => "private",
                ChatKind::Group => "group",
            },
        )
        .unwrap();
        arr.push(obj).unwrap();
    }
    arr.pretty(2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_json_empty_produces_empty_array() {
        let json = to_json(&[]);
        assert!(json.contains("[]") || json == "[]");
    }

    #[test]
    fn to_json_one_item_includes_all_fields() {
        let item = ChatSummary {
            rowid: 1,
            name: "Alice".to_string(),
            message_count: 100,
            last_active: Some(1_700_000_000),
            kind: ChatKind::Private,
        };
        let json = to_json(&[item]);
        assert!(json.contains("\"rowid\""));
        assert!(json.contains("\"name\""));
        assert!(json.contains("\"message_count\""));
        assert!(json.contains("\"last_active\""));
        assert!(json.contains("\"type\""));
        assert!(json.contains("\"Alice\""));
        assert!(json.contains("private"));
    }

    #[test]
    fn commafy_handles_zero() {
        assert_eq!(commafy(0), "0");
    }

    #[test]
    fn commafy_thousands() {
        assert_eq!(commafy(1_234_567), "1,234,567");
    }

    #[test]
    fn truncate_short_string_unchanged() {
        assert_eq!(truncate("abc", 10), "abc");
    }

    #[test]
    fn truncate_long_string_gets_ellipsis() {
        let result = truncate("abcdefghijklmnopqrstuvwxyz", 10);
        assert!(result.ends_with('\u{2026}'));
        assert!(display_width(&result) <= 10);
    }
}

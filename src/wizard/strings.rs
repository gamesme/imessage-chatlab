//! Bilingual string tables for the wizard. Other wizard modules reference
//! fields on a `Strings` struct so the prompts can be translated wholesale.
//!
//! When adding a new prompt, add the field to `Strings`, then update both
//! `EN` and `ZH`. The compile-time invariant (same struct) prevents drift.

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    En,
    Zh,
}

pub struct Strings {
    // Screen 1
    pub source_use_detected: &'static str,
    pub source_choice_yes: &'static str,
    pub source_choice_other_path: &'static str,
    pub source_choice_ios: &'static str,
    pub source_ios_path: &'static str,
    pub source_ios_password: &'static str,

    // Screen 2
    pub filter_summary: &'static str, // "Found {chats} chats with {msgs} messages total."
    pub filter_mode_question: &'static str,
    pub filter_mode_all: &'static str,
    pub filter_mode_pick: &'static str,
    pub filter_mode_date: &'static str,
    pub filter_mode_people: &'static str,

    // Screen 3a (multi-select)
    pub pick_chats_prompt: &'static str,
    pub pick_chats_empty_error: &'static str,

    // Screen 3b (date)
    pub date_start_prompt: &'static str,
    pub date_end_prompt: &'static str,

    // Screen 3c (people)
    pub people_prompt: &'static str,

    // Screen 4 (attachments)
    pub attach_question: &'static str,
    pub attach_clone: &'static str,
    pub attach_basic: &'static str,
    pub attach_full: &'static str,
    pub attach_disabled: &'static str,

    // Screen 5 (avatars)
    pub avatars_question: &'static str,
    pub avatars_yes: &'static str,
    pub avatars_no: &'static str,

    // Screen 6 (output)
    pub output_prompt: &'static str,

    // Screen 7 (summary)
    pub summary_proceed: &'static str,
    pub summary_label_source: &'static str,
    pub summary_label_chats: &'static str,
    pub summary_label_attach: &'static str,
    pub summary_label_avatars: &'static str,
    pub summary_label_output: &'static str,
    pub summary_avatars_embedded: &'static str,
    pub summary_avatars_not_embedded: &'static str,
}

pub const EN: Strings = Strings {
    source_use_detected: "Use this database?",
    source_choice_yes: "Yes",
    source_choice_other_path: "No, point me to a different path",
    source_choice_ios: "iOS backup (encrypted or not)",
    source_ios_path: "Path to iOS backup directory",
    source_ios_password: "Backup password (leave empty if unencrypted)",
    filter_summary: "Found {chats} chats with {msgs} messages total.",
    filter_mode_question: "What do you want to back up?",
    filter_mode_all: "Everything",
    filter_mode_pick: "Pick specific conversations",
    filter_mode_date: "Only messages within a date range",
    filter_mode_people: "Only conversations with specific people",
    pick_chats_prompt: "Pick conversations to export (Space toggles, Enter confirms, type to filter)",
    pick_chats_empty_error: "Select at least one conversation",
    date_start_prompt: "Start date (YYYY-MM-DD)",
    date_end_prompt: "End date (YYYY-MM-DD, exclusive)",
    people_prompt: "Enter participant names, numbers, or emails (comma-separated)",
    attach_question: "How should attachments be handled?",
    attach_clone: "Copy all as-is (clone) — recommended for backup",
    attach_basic: "Copy + convert HEIC images to JPEG (basic)",
    attach_full: "Copy + convert images, audio, and video (full)",
    attach_disabled: "Don't copy attachments (smaller export)",
    avatars_question: "Embed contact and group avatars into the JSON?",
    avatars_yes: "Yes (recommended — keeps everything in one file)",
    avatars_no: "No (smaller JSON, no faces)",
    output_prompt: "Where should the export go?",
    summary_proceed: "Proceed?",
    summary_label_source: "Source",
    summary_label_chats: "Chats",
    summary_label_attach: "Attachments",
    summary_label_avatars: "Avatars",
    summary_label_output: "Output",
    summary_avatars_embedded: "embedded",
    summary_avatars_not_embedded: "not embedded",
};

pub const ZH: Strings = Strings {
    source_use_detected: "使用这个数据库?",
    source_choice_yes: "好",
    source_choice_other_path: "不,我用另一个路径",
    source_choice_ios: "iOS 备份(加密与否都行)",
    source_ios_path: "iOS 备份目录路径",
    source_ios_password: "备份密码(未加密的话留空)",
    filter_summary: "找到 {chats} 个会话,共 {msgs} 条消息。",
    filter_mode_question: "要备份哪些?",
    filter_mode_all: "全部",
    filter_mode_pick: "挑几个会话",
    filter_mode_date: "按日期范围",
    filter_mode_people: "按参与者",
    pick_chats_prompt: "选择会话(空格切换,回车确认,可直接输入过滤)",
    pick_chats_empty_error: "至少选一个会话",
    date_start_prompt: "起始日期 (YYYY-MM-DD)",
    date_end_prompt: "结束日期 (YYYY-MM-DD,不含)",
    people_prompt: "输入姓名 / 号码 / 邮箱(逗号分隔)",
    attach_question: "附件怎么处理?",
    attach_clone: "原样复制(clone)—— 备份推荐",
    attach_basic: "复制并把 HEIC 图转 JPEG(basic)",
    attach_full: "复制并转换图 / 音 / 视频(full)",
    attach_disabled: "不复制附件(导出更小)",
    avatars_question: "把联系人和群头像内嵌进 JSON?",
    avatars_yes: "是(推荐 —— 所有内容在一个文件里)",
    avatars_no: "否(JSON 更小,但没有头像)",
    output_prompt: "导出到哪个目录?",
    summary_proceed: "确认开始?",
    summary_label_source: "数据源",
    summary_label_chats: "会话",
    summary_label_attach: "附件",
    summary_label_avatars: "头像",
    summary_label_output: "输出",
    summary_avatars_embedded: "已内嵌",
    summary_avatars_not_embedded: "未内嵌",
};

/// Pick a language based on explicit override → env LANG → default English.
pub fn select(explicit: Option<&str>) -> &'static Strings {
    if let Some(name) = explicit {
        return match name {
            "zh" => &ZH,
            _ => &EN,
        };
    }
    if let Ok(lang) = std::env::var("LANG")
        && lang.to_lowercase().starts_with("zh")
    {
        return &ZH;
    }
    &EN
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_zh_returns_zh() {
        // Use a known label that differs between languages.
        assert_eq!(select(Some("zh")).source_choice_yes, "好");
    }

    #[test]
    fn explicit_en_returns_en() {
        assert_eq!(select(Some("en")).source_choice_yes, "Yes");
    }

    #[test]
    fn explicit_unknown_falls_back_to_en() {
        assert_eq!(select(Some("xx")).source_choice_yes, "Yes");
    }

    #[test]
    fn no_explicit_defaults_returns_some_table() {
        // We can't reliably mutate $LANG without test interference. Just verify
        // that select(None) returns *some* valid table.
        let s = select(None);
        assert!(!s.source_choice_yes.is_empty());
    }
}

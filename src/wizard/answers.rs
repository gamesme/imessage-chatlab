//! Pure mapping from "what the wizard collected" to a populated `Options`.
//!
//! Kept separate from `flow.rs` (which calls `inquire` and is hard to test)
//! so the data → Options translation is unit-testable.

use std::path::PathBuf;

use imessage_database::util::{platform::Platform, query_context::QueryContext};

use crate::compatibility::attachment_manager::{AttachmentManager, AttachmentManagerMode};
use crate::error::RuntimeError;
use crate::options::{ExportType, Options};

#[derive(Debug, PartialEq, Eq)]
pub struct WizardAnswers {
    pub db_path: PathBuf,
    pub platform: Platform,
    pub cleartext_password: Option<String>,
    pub filter: FilterChoice,
    pub copy_method: AttachmentManagerMode,
    pub embed_avatars: bool,
    pub export_path: PathBuf,
}

#[derive(Debug, PartialEq, Eq)]
pub enum FilterChoice {
    All,
    PickedChatIds(Vec<i32>),
    DateRange { start: String, end: String },
    Participants(String),
}

pub fn to_options(answers: WizardAnswers) -> Result<Options, RuntimeError> {
    let mut query_context = QueryContext::default();
    let mut conversation_filter: Option<String> = None;

    match &answers.filter {
        FilterChoice::All => {}
        FilterChoice::PickedChatIds(ids) => {
            query_context.set_selected_chat_ids(ids.iter().copied().collect());
        }
        FilterChoice::DateRange { start, end } => {
            if !start.is_empty() {
                query_context
                    .set_start(start)
                    .map_err(|e| RuntimeError::InvalidOptions(format!("{e}")))?;
            }
            if !end.is_empty() {
                query_context
                    .set_end(end)
                    .map_err(|e| RuntimeError::InvalidOptions(format!("{e}")))?;
            }
        }
        FilterChoice::Participants(s) => {
            conversation_filter = Some(s.clone());
        }
    }

    Ok(Options {
        db_path: answers.db_path,
        attachment_root: None,
        attachment_manager: AttachmentManager::from(answers.copy_method),
        export_type: Some(ExportType::Json),
        export_path: answers.export_path,
        query_context,
        custom_name: None,
        use_caller_id: false,
        platform: answers.platform,
        ignore_disk_space: false,
        conversation_filter,
        cleartext_password: answers.cleartext_password,
        contacts_path: None,
        embed_avatars: answers.embed_avatars,
        quiet: false,
        no_timestamp: false,
        dry_run: false,
        incremental: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_answers() -> WizardAnswers {
        WizardAnswers {
            db_path: PathBuf::from("/tmp/test.db"),
            platform: Platform::macOS,
            cleartext_password: None,
            filter: FilterChoice::All,
            copy_method: AttachmentManagerMode::Clone,
            embed_avatars: true,
            export_path: PathBuf::from("/tmp/out"),
        }
    }

    #[test]
    fn to_options_with_all_filter_sets_no_filter() {
        let opts = to_options(base_answers()).unwrap();
        assert!(opts.conversation_filter.is_none());
        assert!(opts.query_context.selected_chat_ids.is_none());
    }

    #[test]
    fn to_options_with_picked_chat_ids_sets_query_context() {
        let mut a = base_answers();
        a.filter = FilterChoice::PickedChatIds(vec![1, 5, 12]);
        let opts = to_options(a).unwrap();
        let ids = opts.query_context.selected_chat_ids.unwrap();
        assert!(ids.contains(&1));
        assert!(ids.contains(&5));
        assert!(ids.contains(&12));
    }

    #[test]
    fn to_options_with_date_range_sets_query_context() {
        let mut a = base_answers();
        a.filter = FilterChoice::DateRange {
            start: "2026-01-01".to_string(),
            end: "2026-12-31".to_string(),
        };
        let opts = to_options(a).unwrap();
        assert!(opts.query_context.has_filters());
    }

    #[test]
    fn to_options_with_participants_sets_conversation_filter() {
        let mut a = base_answers();
        a.filter = FilterChoice::Participants("Alice,Bob".to_string());
        let opts = to_options(a).unwrap();
        assert_eq!(opts.conversation_filter.as_deref(), Some("Alice,Bob"));
    }

    #[test]
    fn to_options_propagates_attachment_mode() {
        let mut a = base_answers();
        a.copy_method = AttachmentManagerMode::Full;
        let opts = to_options(a).unwrap();
        // AttachmentManagerMode has a Display impl; assert via the mode field.
        assert!(format!("{}", opts.attachment_manager.mode).contains("full"));
    }

    #[test]
    fn to_options_with_bad_date_returns_error() {
        let mut a = base_answers();
        a.filter = FilterChoice::DateRange {
            start: "not-a-date".to_string(),
            end: "".to_string(),
        };
        assert!(to_options(a).is_err());
    }
}

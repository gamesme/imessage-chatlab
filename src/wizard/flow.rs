//! Sequential prompts for Screens 1–7. The shape mirrors §5 of the spec.
//!
//! All UI strings come from `crate::wizard::strings`. The mapping from
//! collected answers to `Options` lives in `crate::wizard::answers`.

use std::path::{Path, PathBuf};

use inquire::error::InquireError;
use inquire::{Confirm, MultiSelect, Password, Select, Text};

use imessage_database::util::{dirs::default_db_path, platform::Platform};

use crate::compatibility::attachment_manager::AttachmentManagerMode;
use crate::error::RuntimeError;
use crate::options::{DEFAULT_OUTPUT_DIR, resolve_export_path};
use crate::query::{ChatKind, ChatSummary, chat_summaries, relative_time};
use crate::runtime::Config;
use crate::wizard::answers::{FilterChoice, WizardAnswers};
use crate::wizard::strings::Strings;

pub fn collect(strings: &Strings) -> Result<WizardAnswers, RuntimeError> {
    let (db_path, platform, password) = screen1_source(strings)?;
    let preview_config = build_preview_config(&db_path, &platform, password.as_deref())?;
    let filter = screen2_filter(strings, &preview_config)?;
    let copy_method = screen4_attachments(strings)?;
    let embed_avatars = screen5_avatars(strings)?;
    let export_path = screen6_output(strings)?;
    screen7_confirm(
        strings,
        &preview_config,
        &filter,
        &copy_method,
        embed_avatars,
        &export_path,
    )?;
    Ok(WizardAnswers {
        db_path,
        platform,
        cleartext_password: password,
        filter,
        copy_method,
        embed_avatars,
        export_path,
    })
}

fn screen1_source(s: &Strings) -> Result<(PathBuf, Platform, Option<String>), RuntimeError> {
    let detected = default_db_path();
    let choices = vec![
        s.source_choice_yes,
        s.source_choice_other_path,
        s.source_choice_ios,
    ];
    let selected = Select::new(s.source_use_detected, choices)
        .prompt()
        .map_err(map_inquire_err)?;

    if selected == s.source_choice_yes {
        return Ok((detected, Platform::macOS, None));
    }
    if selected == s.source_choice_other_path {
        let path_str = Text::new(s.source_use_detected)
            .with_default(detected.to_str().unwrap_or(""))
            .prompt()
            .map_err(map_inquire_err)?;
        return Ok((PathBuf::from(path_str), Platform::macOS, None));
    }
    // iOS backup
    let path_str = Text::new(s.source_ios_path)
        .prompt()
        .map_err(map_inquire_err)?;
    let pw = Password::new(s.source_ios_password)
        .without_confirmation()
        .prompt_skippable()
        .map_err(map_inquire_err)?;
    let pw_opt = pw.filter(|p| !p.is_empty());
    Ok((PathBuf::from(path_str), Platform::iOS, pw_opt))
}

fn build_preview_config(
    db_path: &Path,
    platform: &Platform,
    cleartext_password: Option<&str>,
) -> Result<Config, RuntimeError> {
    use crate::compatibility::attachment_manager::AttachmentManager;
    use crate::options::{ExportType, Options};
    use imessage_database::util::query_context::QueryContext;

    // Platform has no Clone — reconstruct from the reference.
    let platform_owned = match platform {
        Platform::macOS => Platform::macOS,
        Platform::iOS => Platform::iOS,
    };
    let opts = Options {
        db_path: db_path.to_path_buf(),
        attachment_root: None,
        attachment_manager: AttachmentManager::default(),
        export_type: Some(ExportType::Json),
        export_path: std::env::temp_dir(), // not used during wizard
        query_context: QueryContext::default(),
        custom_name: None,
        use_caller_id: false,
        platform: platform_owned,
        ignore_disk_space: true,
        conversation_filter: None,
        cleartext_password: cleartext_password.map(String::from),
        contacts_path: None,
        embed_avatars: true,
        quiet: true, // suppress cache progress during the wizard
        no_timestamp: false,
        dry_run: false,
        incremental: false,
    };
    crate::logging::set_quiet(true);
    crate::preflight::check_db_readable(&opts.get_db_path())?;
    Config::new(opts)
}

fn screen2_filter(s: &Strings, config: &Config) -> Result<FilterChoice, RuntimeError> {
    let total_chats = config.chatrooms.len();
    let p = crate::preview::build(config)?;
    eprintln!(
        "{}",
        s.filter_summary
            .replace("{chats}", &total_chats.to_string())
            .replace("{msgs}", &p.message_count.to_string())
    );

    let choices = vec![
        s.filter_mode_all,
        s.filter_mode_pick,
        s.filter_mode_date,
        s.filter_mode_people,
    ];
    let chosen = Select::new(s.filter_mode_question, choices)
        .prompt()
        .map_err(map_inquire_err)?;

    if chosen == s.filter_mode_all {
        Ok(FilterChoice::All)
    } else if chosen == s.filter_mode_pick {
        screen3a_pick(s, config)
    } else if chosen == s.filter_mode_date {
        screen3b_date(s)
    } else {
        screen3c_people(s)
    }
}

fn screen3a_pick(s: &Strings, config: &Config) -> Result<FilterChoice, RuntimeError> {
    let summaries = chat_summaries(config)?;
    let options: Vec<String> = summaries.iter().map(format_chat_line).collect();
    let selected = MultiSelect::new(s.pick_chats_prompt, options)
        .with_help_message(s.pick_chats_empty_error)
        .prompt()
        .map_err(map_inquire_err)?;
    let ids: Vec<i32> = selected
        .iter()
        .filter_map(|line| line.split_whitespace().next().and_then(|s| s.parse().ok()))
        .collect();
    if ids.is_empty() {
        return Err(RuntimeError::InvalidOptions(
            s.pick_chats_empty_error.to_string(),
        ));
    }
    Ok(FilterChoice::PickedChatIds(ids))
}

fn format_chat_line(c: &ChatSummary) -> String {
    let kind = match c.kind {
        ChatKind::Private => "private",
        ChatKind::Group => "group",
    };
    let active = c
        .last_active
        .map(relative_time)
        .unwrap_or_else(|| "—".to_string());
    format!(
        "{:>6}  {:<30}  {:>10}  {:<14}  {}",
        c.rowid,
        truncate(&c.name, 30),
        c.message_count,
        active,
        kind
    )
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let mut out: String = s.chars().take(max.saturating_sub(1)).collect();
        out.push('…');
        out
    }
}

fn screen3b_date(s: &Strings) -> Result<FilterChoice, RuntimeError> {
    let start = Text::new(s.date_start_prompt)
        .prompt()
        .map_err(map_inquire_err)?;
    let end = Text::new(s.date_end_prompt)
        .prompt()
        .map_err(map_inquire_err)?;
    Ok(FilterChoice::DateRange { start, end })
}

fn screen3c_people(s: &Strings) -> Result<FilterChoice, RuntimeError> {
    let people = Text::new(s.people_prompt)
        .prompt()
        .map_err(map_inquire_err)?;
    Ok(FilterChoice::Participants(people))
}

fn screen4_attachments(s: &Strings) -> Result<AttachmentManagerMode, RuntimeError> {
    let labels: Vec<(&str, AttachmentManagerMode)> = vec![
        (s.attach_clone, AttachmentManagerMode::Clone),
        (s.attach_basic, AttachmentManagerMode::Basic),
        (s.attach_full, AttachmentManagerMode::Full),
        (s.attach_disabled, AttachmentManagerMode::Disabled),
    ];
    let display: Vec<&str> = labels.iter().map(|(l, _)| *l).collect();
    let chosen = Select::new(s.attach_question, display)
        .prompt()
        .map_err(map_inquire_err)?;
    let mode = labels
        .into_iter()
        .find(|(l, _)| *l == chosen)
        .map(|(_, m)| m)
        .unwrap_or(AttachmentManagerMode::Disabled);
    Ok(mode)
}

fn screen5_avatars(s: &Strings) -> Result<bool, RuntimeError> {
    let chosen = Select::new(s.avatars_question, vec![s.avatars_yes, s.avatars_no])
        .prompt()
        .map_err(map_inquire_err)?;
    Ok(chosen == s.avatars_yes)
}

fn screen6_output(s: &Strings) -> Result<PathBuf, RuntimeError> {
    use imessage_database::util::dirs::home;
    let default = format!("{}/{DEFAULT_OUTPUT_DIR}", home());
    let entered = Text::new(s.output_prompt)
        .with_default(&default)
        .prompt()
        .map_err(map_inquire_err)?;
    Ok(resolve_export_path(Some(&PathBuf::from(entered)), false))
}

fn screen7_confirm(
    s: &Strings,
    config: &Config,
    filter: &FilterChoice,
    copy_method: &AttachmentManagerMode,
    embed_avatars: bool,
    export_path: &Path,
) -> Result<(), RuntimeError> {
    eprintln!();
    eprintln!(
        "  {}:  {}",
        s.summary_label_source,
        config.options.db_path.display()
    );
    let chat_part = match filter {
        FilterChoice::All => "all".to_string(),
        FilterChoice::PickedChatIds(ids) => format!("{} selected", ids.len()),
        FilterChoice::DateRange { .. } => "all chats, filtered by date".to_string(),
        FilterChoice::Participants(p) => format!("filtered by participants: {p}"),
    };
    eprintln!("  {}:        {}", s.summary_label_chats, chat_part);
    eprintln!(
        "  {}:    {}",
        s.summary_label_attach,
        format!("{copy_method}").to_lowercase()
    );
    eprintln!(
        "  {}:      {}",
        s.summary_label_avatars,
        if embed_avatars {
            s.summary_avatars_embedded
        } else {
            s.summary_avatars_not_embedded
        }
    );
    eprintln!(
        "  {}:       {}",
        s.summary_label_output,
        export_path.display()
    );
    eprintln!();

    let ok = Confirm::new(s.summary_proceed)
        .with_default(true)
        .prompt()
        .map_err(map_inquire_err)?;
    if !ok {
        return Err(RuntimeError::WizardCancelled);
    }
    Ok(())
}

fn map_inquire_err(e: InquireError) -> RuntimeError {
    match e {
        InquireError::OperationInterrupted | InquireError::OperationCanceled => {
            RuntimeError::WizardCancelled
        }
        other => RuntimeError::InvalidOptions(format!("{other}")),
    }
}

/*!
 CLI options for `imessage-chatlab`.
*/

use std::path::PathBuf;

use clap::{Arg, ArgAction, ArgMatches, Command, crate_version};

use imessage_database::{
    tables::{attachment::DEFAULT_MESSAGES_ROOT, table::DEFAULT_PATH_IOS},
    util::{
        dirs::{default_db_path, home},
        platform::Platform,
        query_context::QueryContext,
    },
};

use crate::{
    compatibility::attachment_manager::{AttachmentManager, AttachmentManagerMode},
    error::RuntimeError,
};

// MARK: Constants
/// Default export directory name (under `$HOME`).
pub const DEFAULT_OUTPUT_DIR: &str = "imessage_chatlab_export";

// CLI Arg Names
pub const OPTION_DB_PATH: &str = "db-path";
pub const OPTION_ATTACHMENT_ROOT: &str = "attachment-root";
pub const OPTION_ATTACHMENT_MANAGER: &str = "copy-method";
pub const OPTION_EXPORT_PATH: &str = "export-path";
pub const OPTION_START_DATE: &str = "start-date";
pub const OPTION_END_DATE: &str = "end-date";
pub const OPTION_CUSTOM_NAME: &str = "custom-name";
pub const OPTION_PLATFORM: &str = "platform";
pub const OPTION_BYPASS_FREE_SPACE_CHECK: &str = "ignore-disk-warning";
pub const OPTION_USE_CALLER_ID: &str = "use-caller-id";
pub const OPTION_CONVERSATION_FILTER: &str = "conversation-filter";
pub const OPTION_CLEARTEXT_PASSWORD: &str = "cleartext-password";
pub const OPTION_CUSTOM_CONTACTS_DB_PATH: &str = "contacts-path";
pub const OPTION_EMBED_AVATARS: &str = "embed-avatars";
pub const OPTION_QUIET: &str = "quiet";
pub const OPTION_NO_TIMESTAMP: &str = "no-timestamp";
pub const OPTION_DRY_RUN: &str = "dry-run";

// Other CLI Text
pub const SUPPORTED_PLATFORMS: &str = "macOS, iOS";
pub const SUPPORTED_ATTACHMENT_MANAGER_MODES: &str = "clone, basic, full, disabled";
pub const ABOUT: &str = concat!(
    "Export iMessage data to the ChatLab v0.0.2 standard JSON format.\n",
    "One JSON file per conversation, with optional attachment copying\n",
    "and inlined avatars as base64 Data URLs."
);

// MARK: ExportType
/// Output format. This crate only emits ChatLab JSON; the enum is kept so
/// downstream code (`runtime.rs`, `exporter.rs`) doesn't need to special-case
/// the single-variant world.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportType {
    Json,
}

impl ExportType {
    pub fn extension(&self) -> &'static str {
        match self {
            ExportType::Json => ".json",
        }
    }
}

impl std::fmt::Display for ExportType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExportType::Json => write!(f, "json"),
        }
    }
}

// MARK: Options
#[derive(Debug, PartialEq, Eq)]
pub struct Options {
    pub db_path: PathBuf,
    pub attachment_root: Option<String>,
    pub attachment_manager: AttachmentManager,
    pub export_type: Option<ExportType>,
    pub export_path: PathBuf,
    pub query_context: QueryContext,
    pub custom_name: Option<String>,
    pub use_caller_id: bool,
    pub platform: Platform,
    pub ignore_disk_space: bool,
    pub conversation_filter: Option<String>,
    pub cleartext_password: Option<String>,
    pub contacts_path: Option<PathBuf>,
    pub embed_avatars: bool,
    pub quiet: bool,
    pub no_timestamp: bool,
    pub dry_run: bool,
}

// MARK: Validation
impl Options {
    pub fn from_args(args: &ArgMatches) -> Result<Self, RuntimeError> {
        let user_path: Option<&String> = args.get_one(OPTION_DB_PATH);
        let attachment_root: Option<&String> = args.get_one(OPTION_ATTACHMENT_ROOT);
        let attachment_manager_type: Option<&String> = args.get_one(OPTION_ATTACHMENT_MANAGER);
        let user_export_path: Option<&String> = args.get_one(OPTION_EXPORT_PATH);
        let start_date: Option<&String> = args.get_one(OPTION_START_DATE);
        let end_date: Option<&String> = args.get_one(OPTION_END_DATE);
        let custom_name: Option<&String> = args.get_one(OPTION_CUSTOM_NAME);
        let use_caller_id = args.get_flag(OPTION_USE_CALLER_ID);
        let platform_type: Option<&String> = args.get_one(OPTION_PLATFORM);
        let ignore_disk_space = args.get_flag(OPTION_BYPASS_FREE_SPACE_CHECK);
        let conversation_filter: Option<&String> = args.get_one(OPTION_CONVERSATION_FILTER);
        let cleartext_password: Option<&String> = args.get_one(OPTION_CLEARTEXT_PASSWORD);
        let contacts_path: Option<&String> = args.get_one(OPTION_CUSTOM_CONTACTS_DB_PATH);
        let no_timestamp = args.get_flag(OPTION_NO_TIMESTAMP);
        let dry_run = args.get_flag(OPTION_DRY_RUN);

        // Only one format; always set so `runtime.rs` always takes the export branch.
        let export_type: Option<ExportType> = Some(ExportType::Json);

        // Prevent custom_name vs. use_caller_id collision
        if custom_name.is_some() && use_caller_id {
            return Err(RuntimeError::InvalidOptions(format!(
                "--{OPTION_CUSTOM_NAME} is enabled; --{OPTION_USE_CALLER_ID} is disallowed"
            )));
        }

        // Build query context
        let mut query_context = QueryContext::default();
        if let Some(start) = start_date
            && let Err(why) = query_context.set_start(start)
        {
            return Err(RuntimeError::InvalidOptions(format!("{why}")));
        }
        if let Some(end) = end_date
            && let Err(why) = query_context.set_end(end)
        {
            return Err(RuntimeError::InvalidOptions(format!("{why}")));
        }

        let db_path = match user_path {
            Some(path) => PathBuf::from(path),
            None => default_db_path(),
        };

        let platform = match platform_type {
            Some(platform_str) => Platform::from_cli(platform_str).ok_or(
                RuntimeError::InvalidOptions(format!(
                    "{platform_str} is not a valid platform! Must be one of <{SUPPORTED_PLATFORMS}>"
                )),
            )?,
            None => Platform::determine(&db_path)?,
        };

        if cleartext_password.is_some() && !matches!(platform, Platform::iOS) {
            return Err(RuntimeError::InvalidOptions(format!(
                "--{OPTION_CLEARTEXT_PASSWORD} is enabled; it can only be used with iOS backups."
            )));
        }

        if let Some(path) = attachment_root {
            let custom_attachment_path = PathBuf::from(path);
            if !custom_attachment_path.exists() {
                return Err(RuntimeError::InvalidOptions(format!(
                    "Supplied --{OPTION_ATTACHMENT_ROOT} `{path}` does not exist!"
                )));
            }
        }

        if attachment_root.is_some() && platform == Platform::iOS {
            eprintln!(
                "Option --{OPTION_ATTACHMENT_ROOT} is enabled, but the platform is {}, so the root will have no effect!",
                Platform::iOS
            );
        }

        if let Some(path) = contacts_path {
            let custom_contacts_path = PathBuf::from(path);
            if !custom_contacts_path.exists() {
                return Err(RuntimeError::InvalidOptions(format!(
                    "Supplied --{OPTION_CUSTOM_CONTACTS_DB_PATH} `{path}` does not exist!"
                )));
            }
        }

        if contacts_path.is_some() && platform == Platform::iOS {
            eprintln!(
                "Option --{OPTION_CUSTOM_CONTACTS_DB_PATH} is enabled, but the platform is {}, so the path will have no effect!",
                Platform::iOS
            );
        }

        let attachment_manager_mode = match attachment_manager_type {
            Some(manager) => AttachmentManagerMode::from_cli(manager).ok_or(
                RuntimeError::InvalidOptions(format!(
                    "{manager} is not a valid attachment manager mode! Must be one of <{SUPPORTED_ATTACHMENT_MANAGER_MODES}>"
                )),
            )?,
            None => AttachmentManagerMode::default(),
        };

        let user_export_buf = user_export_path.map(PathBuf::from);
        let export_path = resolve_export_path(user_export_buf.as_ref(), no_timestamp);
        if no_timestamp {
            // Only enforce the legacy "no existing JSON" check when timestamping
            // is suppressed. With timestamps on, collisions are vanishingly rare.
            validate_path_no_existing_json(&export_path, &export_type)?;
        }

        let embed_avatars = args
            .get_one::<bool>(OPTION_EMBED_AVATARS)
            .copied()
            .unwrap_or(true);

        let quiet = args.get_flag(OPTION_QUIET);

        Ok(Options {
            db_path,
            attachment_root: attachment_root.cloned(),
            attachment_manager: AttachmentManager::from(attachment_manager_mode),
            export_type,
            export_path,
            query_context,
            custom_name: custom_name.cloned(),
            use_caller_id,
            platform,
            ignore_disk_space,
            conversation_filter: conversation_filter.cloned(),
            cleartext_password: cleartext_password.cloned(),
            contacts_path: contacts_path.cloned().map(PathBuf::from),
            embed_avatars,
            quiet,
            no_timestamp,
            dry_run,
        })
    }

    /// Resolve the actual on-disk database path, taking platform into account.
    pub fn get_db_path(&self) -> PathBuf {
        match self.platform {
            Platform::iOS => self.db_path.join(DEFAULT_PATH_IOS),
            Platform::macOS => self.db_path.clone(),
        }
    }
}

/// Used only when `--no-timestamp` is set. Refuses to overwrite a directory
/// that already contains JSON files of the same export type.
fn validate_path_no_existing_json(
    export_path: &std::path::Path,
    export_type: &Option<ExportType>,
) -> Result<(), RuntimeError> {
    let Some(export_type) = export_type else {
        return Ok(());
    };
    if !export_path.exists() {
        return Ok(());
    }
    let extension = export_type.to_string();
    match export_path.read_dir() {
        Ok(files) => {
            for file in files.flatten() {
                if file
                    .path()
                    .extension()
                    .is_some_and(|s| s.to_str().unwrap_or("") == extension)
                {
                    return Err(RuntimeError::InvalidOptions(format!(
                        "Export path {} already contains \"{export_type}\" files; \
                         remove them or omit --no-timestamp",
                        export_path.display()
                    )));
                }
            }
            Ok(())
        }
        Err(why) => Err(RuntimeError::InvalidOptions(format!(
            "Cannot read export path {}: {why}",
            export_path.display()
        ))),
    }
}

/// Returns the current UTC time formatted as ISO-8601 with filesystem-safe
/// separators: `YYYY-MM-DDTHH-MM-SSZ` (colons replaced by hyphens).
pub(crate) fn timestamp_for_path() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let dt = chrono::DateTime::<chrono::Utc>::from_timestamp(secs, 0)
        .unwrap_or_default();
    dt.format("%Y-%m-%dT%H-%M-%SZ").to_string()
}

/// Combine the user-provided base path (or default) with the timestamp policy.
///
/// `--no-timestamp` returns `base` unchanged. Otherwise appends a timestamped
/// subdirectory.
pub(crate) fn resolve_export_path(
    user_export_path: Option<&PathBuf>,
    no_timestamp: bool,
) -> PathBuf {
    let base = match user_export_path {
        Some(p) => p.clone(),
        None => PathBuf::from(format!("{}/{DEFAULT_OUTPUT_DIR}", home())),
    };
    if no_timestamp {
        base
    } else {
        base.join(timestamp_for_path())
    }
}

// MARK: CLI
/// Build the clap parser. `imessage-chatlab` only emits JSON, so there is no
/// `--format` flag; everything below is scoped to that single format.
pub fn cli() -> Command {
    Command::new("imessage-chatlab")
        .version(crate_version!())
        .about(ABOUT)
        .arg(
            Arg::new(OPTION_ATTACHMENT_MANAGER)
                .short('c')
                .long(OPTION_ATTACHMENT_MANAGER)
                .help(format!("Specify an optional method to use when copying message attachments\n`clone` will copy all files without converting anything\n`basic` will copy all files and convert HEIC images to JPEG\n`full` will copy all files and convert HEIC files to JPEG, CAF to MP4, and MOV to MP4\nIf omitted, the default is `{}`\nImageMagick is required to convert images on non-macOS platforms\nffmpeg is required to convert audio on non-macOS platforms and video on all platforms\n", AttachmentManagerMode::default()))
                .display_order(1)
                .value_name(SUPPORTED_ATTACHMENT_MANAGER_MODES),
        )
        .arg(
            Arg::new(OPTION_DB_PATH)
                .short('p')
                .long(OPTION_DB_PATH)
                .help(format!("Specify an optional custom path for the iMessage database location\nFor macOS, specify a path to a `chat.db` file\nFor iOS, specify a path to the root of a device backup directory\nIf the iOS backup is encrypted, --{OPTION_CLEARTEXT_PASSWORD} must be passed\nIf omitted, the default directory is {}\n", default_db_path().display()))
                .display_order(2)
                .value_name("path/to/source"),
        )
        .arg(
            Arg::new(OPTION_ATTACHMENT_ROOT)
                .short('r')
                .long(OPTION_ATTACHMENT_ROOT)
                .help(format!("Specify an optional custom path to look for attachment data in\nOnly use this if attachments are stored separately from the database's default location\nThe provided path should be absolute\nThis option affects both the `Attachments` and `StickerCache` directories\nAlso works with jailbroken iOS sms.db databases (use `--platform macOS`)\nHas no effect on iOS backups\nThe default location is {}\n", DEFAULT_MESSAGES_ROOT.replacen('~', &home(), 1)))
                .display_order(3)
                .value_name("path/to/messages/root"),
        )
        .arg(
            Arg::new(OPTION_PLATFORM)
                .short('a')
                .long(OPTION_PLATFORM)
                .help("Specify the platform the database was created on\nIf omitted, the platform type is determined automatically\n")
                .display_order(4)
                .value_name(SUPPORTED_PLATFORMS),
        )
        .arg(
            Arg::new(OPTION_EXPORT_PATH)
                .short('o')
                .long(OPTION_EXPORT_PATH)
                .help(format!("Specify an optional custom directory for outputting exported data\nIf omitted, the default directory is {}/{DEFAULT_OUTPUT_DIR}\n", home()))
                .display_order(5)
                .value_name("path/to/save/files"),
        )
        .arg(
            Arg::new(OPTION_START_DATE)
                .short('s')
                .long(OPTION_START_DATE)
                .help("The start date filter\nOnly messages sent on or after this date will be included\n")
                .display_order(6)
                .value_name("YYYY-MM-DD"),
        )
        .arg(
            Arg::new(OPTION_END_DATE)
                .short('e')
                .long(OPTION_END_DATE)
                .help("The end date filter\nOnly messages sent before this date will be included\n")
                .display_order(7)
                .value_name("YYYY-MM-DD"),
        )
        .arg(
            Arg::new(OPTION_CUSTOM_NAME)
                .short('m')
                .long(OPTION_CUSTOM_NAME)
                .help(format!("Specify an optional custom name for the database owner's messages in exports\nConflicts with --{OPTION_USE_CALLER_ID}\n"))
                .display_order(8),
        )
        .arg(
            Arg::new(OPTION_USE_CALLER_ID)
                .short('i')
                .long(OPTION_USE_CALLER_ID)
                .help(format!("Use the database owner's caller ID in exports instead of \"Me\"\nConflicts with --{OPTION_CUSTOM_NAME}\n"))
                .action(ArgAction::SetTrue)
                .display_order(9),
        )
        .arg(
            Arg::new(OPTION_BYPASS_FREE_SPACE_CHECK)
                .short('b')
                .long(OPTION_BYPASS_FREE_SPACE_CHECK)
                .help("Bypass the disk space check when exporting data\nBy default, exports will not run if there is not enough free disk space\n")
                .action(ArgAction::SetTrue)
                .display_order(10),
        )
        .arg(
            Arg::new(OPTION_CONVERSATION_FILTER)
                .short('t')
                .long(OPTION_CONVERSATION_FILTER)
                .help("Filter exported conversations by contact names, numbers, or emails\nTo provide multiple filter criteria, use a comma-separated string\nAll conversations with the specified participants are exported, including group conversations\nExample: `-t steve@apple.com,5558675309`\n")
                .display_order(11)
                .value_name("filter"),
        )
        .arg(
            Arg::new(OPTION_CLEARTEXT_PASSWORD)
                .short('x')
                .long(OPTION_CLEARTEXT_PASSWORD)
                .help("Optional password for encrypted iOS backups\nThis is only used when the source is an encrypted iOS backup directory\n")
                .display_order(12)
                .value_name("password"),
        )
        .arg(
            Arg::new(OPTION_CUSTOM_CONTACTS_DB_PATH)
                .short('n')
                .long(OPTION_CUSTOM_CONTACTS_DB_PATH)
                .help("Optional custom path for a macOS or iOS contacts database file\nThis should be resolved automatically, but can be manually provided\nHandles from the messages table will be mapped to names in the provided database\nGenerally, one of `AddressBook-v22.abcddb` or `AddressBook.sqlitedb`\n")
                .display_order(13)
                .value_name("path"),
        )
        .arg(
            Arg::new(OPTION_EMBED_AVATARS)
                .long(OPTION_EMBED_AVATARS)
                .help("Embed contact and group avatars as base64 Data URLs in exports\nDefault: true\n")
                .required(false)
                .value_parser(clap::value_parser!(bool))
                .action(ArgAction::Set)
                .display_order(14),
        )
        .arg(
            Arg::new(OPTION_QUIET)
                .short('q')
                .long(OPTION_QUIET)
                .help("Suppress informational output (cache progress, status lines)\nErrors are still printed\n")
                .action(ArgAction::SetTrue)
                .display_order(15),
        )
        .arg(
            Arg::new(OPTION_NO_TIMESTAMP)
                .long(OPTION_NO_TIMESTAMP)
                .help("Don't append a timestamp subdirectory to the output path\nUseful for scripted overwriting in a known location\n")
                .action(ArgAction::SetTrue)
                .display_order(16),
        )
        .arg(
            Arg::new(OPTION_DRY_RUN)
                .long(OPTION_DRY_RUN)
                .help("Show what would be exported (counts, size, output path) and exit\nDoes not write any files\n")
                .action(ArgAction::SetTrue)
                .display_order(17),
        )
}

#[cfg(test)]
impl Options {
    /// Build an `Options` instance for tests that don't actually need to read a
    /// real database. Several callers under `src/` use this — keep the signature
    /// matching the upstream form.
    pub fn fake_options(export_type: ExportType) -> Options {
        Options {
            db_path: PathBuf::from(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/test_data/db/test.db"
            )),
            attachment_root: None,
            attachment_manager: AttachmentManager::default(),
            export_type: Some(export_type),
            export_path: PathBuf::from("/tmp"),
            query_context: QueryContext::default(),
            custom_name: None,
            use_caller_id: false,
            platform: Platform::macOS,
            ignore_disk_space: false,
            conversation_filter: None,
            cleartext_password: None,
            contacts_path: None,
            embed_avatars: true,
            quiet: false,
            no_timestamp: false,
            dry_run: false,
        }
    }
}

#[cfg(test)]
mod timestamp_path_tests {
    use super::*;
    use regex::Regex;
    use std::path::PathBuf;

    #[test]
    fn timestamp_format_is_iso8601_fs_safe() {
        let stamp = timestamp_for_path();
        // Format: YYYY-MM-DDTHH-MM-SSZ — no colons, ends with Z.
        let re = Regex::new(r"^\d{4}-\d{2}-\d{2}T\d{2}-\d{2}-\d{2}Z$").unwrap();
        assert!(re.is_match(&stamp), "got {stamp}");
    }

    #[test]
    fn resolve_export_path_appends_timestamp_by_default() {
        let base = PathBuf::from("/tmp/imchatlab-test-default");
        let resolved = resolve_export_path(Some(&base), false);
        assert!(resolved.starts_with(&base));
        assert_ne!(resolved, base, "timestamp suffix should differ from base");
    }

    #[test]
    fn resolve_export_path_skips_when_no_timestamp() {
        let base = PathBuf::from("/tmp/imchatlab-test-no-ts");
        let resolved = resolve_export_path(Some(&base), true);
        assert_eq!(resolved, base);
    }
}

#[cfg(test)]
mod quiet_flag_tests {
    use super::*;

    #[test]
    fn quiet_flag_short_form_sets_field() {
        let cmd = cli();
        let matches = cmd.get_matches_from(["imessage-chatlab", "-q", "-o", "/tmp/test_export_quiet_short"]);
        let opts = Options::from_args(&matches).unwrap();
        assert!(opts.quiet);
    }

    #[test]
    fn quiet_flag_long_form_sets_field() {
        let cmd = cli();
        let matches = cmd.get_matches_from(["imessage-chatlab", "--quiet", "-o", "/tmp/test_export_quiet_long"]);
        let opts = Options::from_args(&matches).unwrap();
        assert!(opts.quiet);
    }

    #[test]
    fn quiet_flag_default_is_false() {
        let cmd = cli();
        let matches = cmd.get_matches_from(["imessage-chatlab", "-o", "/tmp/test_export_quiet_default"]);
        let opts = Options::from_args(&matches).unwrap();
        assert!(!opts.quiet);
    }
}

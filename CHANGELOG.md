# Changelog

## 0.2.1 — 2026-05-18

### Added
- Progress bar now shows the current chat name (`→ <name>`) so you can see which
  conversation is being processed. Long names are truncated to 40 characters.

## 0.2.0 — 2026-05-14

### Added
- Interactive wizard (`imessage-chatlab` with no args + TTY)
- `list` subcommand with `--json` output
- `--dry-run` flag for preview without writing
- `--no-timestamp` flag to opt out of timestamped subdirectories
- `-q` / `--quiet` flag for cron-friendly output suppression
- `--lang zh|en` flag (and `$LANG` auto-detection) for Chinese wizard prompts
- `-t` accepts `@rowid:N` syntax for ID-based filter (use with `list` output)
- `export` subcommand alias for forward compatibility
- Fail-fast database readability check (catches Full Disk Access errors fast)

### Changed
- Default output paths now include a timestamped subdirectory (see README).
  Use `--no-timestamp` to opt out.
- Informational status lines now go through the suppressible `info!` macro;
  warnings and errors still go directly to stderr.

## 0.1.0 — Unreleased

Initial release. Single output format: ChatLab v0.0.2 standard JSON.

### Features
- One JSON file per conversation, written to the export directory
- Attachment copying via the existing imessage-exporter pipeline
  (`--copy-method clone|basic|full|disabled`)
- HEIC→JPEG / CAF→MP4 / MOV→MP4 conversion through `sips`/`imagemagick`/`ffmpeg`
- Avatar inlining as base64 Data URLs for both members and group photo
  (`--embed-avatars`, default on)
- AddressBook contact resolution on macOS and iOS

### Known limitations
See README.md.

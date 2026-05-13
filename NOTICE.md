# Attributions

This project is built on and adapts code from the [`imessage-exporter`](https://github.com/ReagentX/imessage-exporter)
binary by Christopher Sardegna (@ReagentX), licensed under GPL-3.0-or-later.

## Library dependency

- [`imessage-database`](https://crates.io/crates/imessage-database) — the core
  iMessage SQLite parser. This project links against the published version on
  crates.io; no source is copied from it.

## Files containing code adapted from `imessage-exporter` (the binary)

- `src/contacts.rs` — AddressBook reader; extended in this project with image
  blob reads and a `get_avatar` accessor
- `src/runtime.rs` — runtime config and helpers, trimmed to JSON-only needs
- `src/progress.rs` — progress bar wrapper
- `src/data_source.rs` — database handle + contact index plumbing
- `src/error.rs` — error type definitions
- `src/compatibility/attachment_manager.rs` — attachment copy/transcode driver
- `src/compatibility/backup.rs` — encrypted iOS backup decryption (via crabapple)
- `src/compatibility/models.rs` — converter type definitions
- `src/compatibility/converters/*.rs` — image/audio/video/sticker converters
  wrapping `sips`, `imagemagick`, and `ffmpeg`

## Original work in this project

- `src/exporter.rs` — the ChatLab JSON exporter (originally written by
  @gamesme; lives in the source repo as `exporters/json.rs`)
- `src/avatar.rs` — MIME sniffing and base64 Data URL conversion
- `src/main.rs` — CLI entry point
- `src/options.rs` — CLI argument definitions

## Test fixtures

- `test_data/db/test.db` is copied verbatim from the [`imessage-database`](https://github.com/ReagentX/imessage-exporter/tree/develop/imessage-database/test_data/db)
  test corpus by Christopher Sardegna (@ReagentX), licensed under GPL-3.0-or-later.
  It is excluded from published crate artifacts via `Cargo.toml`'s `exclude` list.

## Format specification

The output JSON format follows the [ChatLab](https://chatlab.fun) v0.0.2
standard. The ChatLab project itself is not affiliated with this tool.

# imessage-chatlab

> English | [简体中文](./README.zh-CN.md)

Export your iMessage data to the [ChatLab](https://chatlab.fun) v0.0.2
standard JSON format. One JSON file per conversation, with optional
attachment copying and inlined avatars as base64 Data URLs.

> Built on [`imessage-database`](https://crates.io/crates/imessage-database)
> by [@ReagentX](https://github.com/ReagentX). See `NOTICE.md` for full
> attribution.

## Installation

```bash
cargo install imessage-chatlab
```

Or from source:

```bash
cargo install --git https://github.com/gamesme/imessage-chatlab
```

## Usage

Export the local iMessage database to ChatLab JSON, copying all attachments
into the export folder and embedding avatars as Data URLs:

```bash
imessage-chatlab -c clone -o ~/imessage_chatlab_export
```

Lighter export with no attachment copy and no avatar inlining:

```bash
imessage-chatlab --embed-avatars=false -o ~/imessage_chatlab_export
```

### Options

```text
-c, --copy-method <clone|basic|full|disabled>
        How to handle media attachments
        `disabled` (default): do not copy attachments
        `clone`: copy attachments as-is
        `basic`: copy + convert HEIC images to JPEG
        `full`: also convert CAF/MOV audio/video to MP4
-p, --db-path <path>
        Optional custom iMessage database path
        macOS: a chat.db file
        iOS: the root of an iOS backup directory
-r, --attachment-root <path>
        Optional custom path to look for attachment data in
-a, --platform <macOS|iOS>
        Source platform; auto-detected if omitted
-o, --export-path <path>
        Output directory (default: ~/imessage_chatlab_export)
-s, --start-date <YYYY-MM-DD>
        Earliest message date to include
-e, --end-date <YYYY-MM-DD>
        Latest message date to include (exclusive)
-m, --custom-name <name>
        Custom name for the database owner in exports
-i, --use-caller-id
        Use the owner's caller ID in exports instead of "Me"
-t, --conversation-filter <filter>
        Filter conversations by participant (names, numbers, emails)
-x, --cleartext-password <password>
        Password for encrypted iOS backups
-n, --contacts-path <path>
        Optional custom AddressBook/Contacts database path
    --embed-avatars <true|false>
        Embed contact and group avatars as base64 Data URLs (default: true)
-h, --help
    --version
```

## Interactive wizard

Run `imessage-chatlab` with no arguments inside a terminal and you'll be
walked through a 7-step interactive setup:

1. Confirm the database source (auto-detected)
2. Choose what to back up (everything / pick / date range / specific people)
3. (If "pick") multi-select conversations with fuzzy search
4. Choose how to handle attachments
5. Choose whether to embed avatars
6. Choose the output directory
7. Confirm

Press `Ctrl+C` at any prompt to cancel; no files are written.

Pass `--lang zh` for Chinese prompts (auto-detected from `$LANG`).

The wizard is **skipped** when:

- Any CLI flag is passed (e.g. `imessage-chatlab -c clone`)
- stdin or stdout is not a TTY (CI, pipes, redirects)

So `cron` jobs continue to work as in v0.1.

## List conversations

Print all chats without exporting:

```bash
imessage-chatlab list
# ROWID  NAME              MESSAGES   LAST ACTIVE     TYPE
# 1      Alice Wang        12,841     2 days ago      private
# 2      Family Group       8,201     yesterday       group
# ...
```

JSON output for scripts:

```bash
imessage-chatlab list --json | jq '.[] | select(.message_count > 1000)'
```

The `ROWID` column is stable per database snapshot. Feed it back into `-t`:

```bash
imessage-chatlab -t '@rowid:1,@rowid:5'
```

### Output paths

By default, exports go to a timestamped subdirectory:

```
~/imessage_chatlab_export/2026-05-14T22-50-37Z/
```

so repeated runs don't collide. Pass `--no-timestamp` to use the literal
path you provided.

## Output format

See [ChatLab's standard format spec](https://chatlab.fun/cn/standard/chatlab-format.html)
for the canonical wire format. Message `type` codes follow the ChatLab enum
(0 text, 1 image, 2 voice, 3 video, 4 file, 5 sticker, 7 link, 23 call, 80
system, 81 recall, 99 other). Media messages use labeled placeholders in
`content`:

| Scenario | `content` |
|---|---|
| Image, copy enabled | `[Image] attachments/12/8421.jpeg` |
| Image with caption | `[Image] attachments/12/8421.jpeg — look at this` |
| Voice with transcription | `[Voice] msg.caf — Transcription: hello` |

## Known limitations

- Some modern macOS AddressBook schemas store contact photos in
  `ZABCDLIKENESS` rather than `ZABCDIMAGE`, or as external blobs under
  `.AddressBook-v22_SUPPORT/_EXTERNAL_DATA/`. Avatars are silently empty in
  those cases.
- iOS `ABMultiValue.property` numbers for phone/email are best-effort.
- Group avatar bytes are read via plain `fs::read`, which does **not** route
  through the encrypted iOS backup decryption path; `meta.groupAvatar` is
  silently omitted on encrypted backups.
- Shared-location start/stop events currently fall through to `type: 0`,
  `content: null`.
- When attachment copy is requested but the source file can't be read,
  decrypted, or copied, the JSON still references the bare filename without
  an in-band failure signal.
- **Group chat `members` only includes participants who sent at least one
  message** (plus the exporter as `ownerId`). Silent members who never
  sent a message in the exported range will not appear in the `members`
  array. This is a structural limitation of the current message-driven
  member collection approach.

## Roadmap / TODO

Short-term (no spec changes needed):

- [ ] **Config file support** — save common options to
  `~/.config/imessage-chatlab/config.toml`
- [ ] **Contact index cache** — cache parsed contacts to
  `~/.cache/imessage-chatlab/contacts.json` to skip rebuild on every run
- [ ] **Deprecate or opt-in `orphaned.json`** — most users never need it
- [ ] **`meta.groupId`** — expose the iMessage chat identifier for group chats
- [ ] **`TYPE_LOCATION = 8`** — detect shared-location messages instead of
  falling through to `TYPE_OTHER(99)`
- [ ] **Progress bar message** — show current conversation name while exporting

Blocked on ChatLab spec extension:

- [ ] **Message edit history** — iMessage stores edits; ChatLab v0.0.2 has no
  `edits` array to represent them
- [ ] **Reaction / tapback detail** — currently textified as `TYPE_OTHER(99)`;
  structured reactions need a `reactions` field in the spec

## License

GPL-3.0-or-later. See `LICENSE` and `NOTICE.md`.

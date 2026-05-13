# Changelog

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

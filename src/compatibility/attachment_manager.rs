/*!
 Defines routines for how attachments should be handled.
*/

use std::{
    fmt::Display,
    fs::remove_file,
    path::{Path, PathBuf},
};

use imessage_database::tables::{
    attachment::{Attachment, MediaType},
    messages::Message,
};

use crate::{
    compatibility::{
        backup::decrypt_file,
        converters::{
            audio::audio_copy_convert,
            common::{copy_raw, update_file_metadata},
            image::image_copy_convert,
            sticker::sticker_copy_convert,
            video::video_copy_convert,
        },
        models::{AudioConverter, Converter, HardwareEncoder, ImageConverter, VideoConverter},
    },
    runtime::Config,
};

// MARK: Manager
#[derive(Debug, PartialEq, Eq, Default)]
pub struct AttachmentManager {
    pub mode: AttachmentManagerMode,
    pub image_converter: Option<ImageConverter>,
    pub audio_converter: Option<AudioConverter>,
    pub video_converter: Option<VideoConverter>,
    hardware_encoder: Option<HardwareEncoder>,
}

impl AttachmentManager {
    pub fn from(mode: AttachmentManagerMode) -> Self {
        AttachmentManager {
            mode,
            image_converter: ImageConverter::determine(),
            audio_converter: AudioConverter::determine(),
            video_converter: VideoConverter::determine(),
            hardware_encoder: HardwareEncoder::detect(),
        }
    }
}

impl AttachmentManager {
    // MARK: Files
    /// Handle an attachment, copying and converting if requested
    ///
    /// If copied, update attachment's `copied_path` and `mime_type`
    pub fn handle_attachment<'a>(
        &'a self,
        message: &Message,
        attachment: &'a mut Attachment,
        config: &Config,
    ) -> Option<()> {
        if !matches!(self.mode, AttachmentManagerMode::Disabled) {
            // Resolve the path to the attachment
            let attachment_path = attachment.resolved_attachment_path(
                &config.options.platform,
                &config.options.db_path,
                config.options.attachment_root.as_deref(),
            )?;

            let mut is_temp = false;
            let mut from = PathBuf::from(&attachment_path);

            // Handle encrypted files from iOS backups
            if let Some(backup) = &config.data_source.backup {
                // We shouldn't get here without an encrypted backup, but just in case, validate it
                if backup.is_encrypted() {
                    match decrypt_file(backup, &from) {
                        Ok(decrypted_path) => {
                            // If the decrypted file is different from the original, use the decrypted one
                            from = decrypted_path;
                            // The decrypted file is temporary, so we need to remove it later
                            is_temp = true;
                        }
                        Err(why) => {
                            eprintln!("Unable to decrypt {}: {why}", from.display());
                            return None;
                        }
                    }
                }
            }

            // Ensure the file exists at the specified location
            if !from.exists() {
                eprintln!("Attachment not found at specified path: {}", from.display());
                return None;
            }

            // Create a path to copy the file to
            let mut to = config.attachment_path();

            // Add the subdirectory
            let sub_dir = config.conversation_attachment_path(message.chat_id);
            to.push(sub_dir);

            // Add a stable filename
            to.push(attachment.rowid.to_string());

            // Set the new file's extension to the original one, if provided
            if !from.is_dir() && attachment.extension().is_some() {
                to.set_extension(attachment.extension()?);
            }

            // If the same file was referenced more than once, i.e. in a reply or response that we render twice, escape early
            if to.exists() {
                attachment.copied_path = Some(to);
                return Some(());
            }

            // If we convert the attachment, we need to update the media type
            let mut new_media_type: Option<MediaType> = None;

            let mime_type = attachment.mime_type();
            match mime_type {
                MediaType::Image(_) => match self.mode {
                    AttachmentManagerMode::Basic | AttachmentManagerMode::Full => {
                        match &self.image_converter {
                            Some(converter) => {
                                if attachment.is_sticker {
                                    new_media_type = sticker_copy_convert(
                                        &from,
                                        &mut to,
                                        converter,
                                        self.video_converter.as_ref(),
                                        &mime_type,
                                    );
                                } else {
                                    new_media_type =
                                        image_copy_convert(&from, &mut to, converter, &mime_type);
                                }
                            }
                            None => copy_raw(&from, &to),
                        }
                    }
                    AttachmentManagerMode::Clone => copy_raw(&from, &to),
                    AttachmentManagerMode::Disabled => unreachable!(),
                },
                MediaType::Video(_) => match self.mode {
                    AttachmentManagerMode::Full => match &self.video_converter {
                        Some(converter) => {
                            new_media_type = video_copy_convert(
                                &from,
                                &mut to,
                                converter,
                                &self.hardware_encoder,
                                &mime_type,
                            );
                        }
                        None => copy_raw(&from, &to),
                    },
                    AttachmentManagerMode::Clone | AttachmentManagerMode::Basic => {
                        copy_raw(&from, &to);
                    }
                    AttachmentManagerMode::Disabled => unreachable!(),
                },
                MediaType::Audio(_) => match self.mode {
                    AttachmentManagerMode::Full => match &self.audio_converter {
                        Some(converter) => {
                            new_media_type =
                                audio_copy_convert(&from, &mut to, converter, &mime_type);
                        }
                        None => copy_raw(&from, &to),
                    },
                    AttachmentManagerMode::Clone | AttachmentManagerMode::Basic => {
                        copy_raw(&from, &to);
                    }
                    AttachmentManagerMode::Disabled => unreachable!(),
                },
                _ => copy_raw(&from, &to),
            }

            // Update file metadata
            if is_temp {
                // If the file was decrypted, we need to update the metadata from the original file
                update_file_metadata(Path::new(&attachment_path), &to, message, config);
            } else {
                // If the file was copied, we need to update the metadata from the source file
                update_file_metadata(&from, &to, message, config);
            }
            attachment.copied_path = Some(to);
            if let Some(media_type) = new_media_type {
                attachment.mime_type = Some(media_type.as_mime_type());
            }

            // Remove the temporary file used for decryption, if it exists
            if is_temp && let Err(why) = remove_file(&from) {
                eprintln!("Unable to remove encrypted file {}: {why}", from.display());
            }
        }

        Some(())
    }
}

// MARK: Mode
/// Represents different ways the app can interact with attachment data
#[derive(Debug, PartialEq, Eq, Default)]
pub enum AttachmentManagerMode {
    /// Do not copy attachments
    #[default]
    Disabled,
    /// Copy and convert image attachments to more compatible formats using a [`Converter`]
    Basic,
    /// Copy attachments without converting; preserves quality but may not display correctly in all browsers
    Clone,
    /// Copy and convert all attachments to more compatible formats using a [`Converter`]
    Full,
}

impl AttachmentManagerMode {
    /// Create an instance of the enum given user input
    pub fn from_cli(copy_state: &str) -> Option<Self> {
        match copy_state.to_lowercase().as_str() {
            "disabled" => Some(Self::Disabled),
            "basic" => Some(Self::Basic),
            "clone" => Some(Self::Clone),
            "full" => Some(Self::Full),
            _ => None,
        }
    }
}

impl Display for AttachmentManagerMode {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AttachmentManagerMode::Disabled => write!(fmt, "disabled"),
            AttachmentManagerMode::Basic => write!(fmt, "basic"),
            AttachmentManagerMode::Clone => write!(fmt, "clone"),
            AttachmentManagerMode::Full => write!(fmt, "full"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::compatibility::attachment_manager::AttachmentManagerMode;

    #[test]
    fn test_attachment_manager_mode() {
        assert_eq!(
            AttachmentManagerMode::from_cli("disabled"),
            Some(AttachmentManagerMode::Disabled)
        );
        assert_eq!(
            AttachmentManagerMode::from_cli("basic"),
            Some(AttachmentManagerMode::Basic)
        );
        assert_eq!(
            AttachmentManagerMode::from_cli("clone"),
            Some(AttachmentManagerMode::Clone)
        );
        assert_eq!(
            AttachmentManagerMode::from_cli("full"),
            Some(AttachmentManagerMode::Full)
        );
        assert_eq!(AttachmentManagerMode::from_cli("invalid"), None);
    }
}

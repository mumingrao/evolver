#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MediaKind {
    Audio,
    Image,
    Video,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct MediaAttachment {
    pub file_name: String,

    pub data: Vec<u8>,

    // `audio/ogg`, `image/jpeg` etc
    pub mime_type: Option<String>,
}

impl MediaAttachment {
    pub fn kind(&self) -> MediaKind {
        if let Some(ref mime) = self.mime_type {
            let lower = mime.to_ascii_lowercase();
            if lower.starts_with("audio/") {
                return MediaKind::Audio;
            }
            if lower.starts_with("image/") {
                return MediaKind::Image;
            }
            if lower.starts_with("video/") {
                return MediaKind::Video;
            }
        }

        let ext = self
            .file_name
            .rsplit_once('.')
            .map(|(_, e)| e.to_ascii_lowercase())
            .unwrap_or_default();

        match ext.as_str() {
            "flac" | "mp3" | "mpeg" | "mpga" | "m4a" | "ogg" | "oga" | "opus" | "wav" | "webm" => {
                MediaKind::Audio
            }
            "png" | "jpg" | "jpeg" | "gif" | "bmp" | "webp" | "heic" | "tiff" | "svg" => {
                MediaKind::Image
            }
            "mp4" | "mkv" | "avi" | "mov" | "wmv" | "flv" => MediaKind::Video,
            _ => MediaKind::Unknown,
        }
    }
}

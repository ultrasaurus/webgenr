//-- Path utlity functions
use std::{borrow::Cow, path::Path};

// return mimetype given an extension
pub fn get_mimetype(ext: &str) -> Cow<'static, str> {
    info!("get_mimetype for: {}", ext);
    Cow::from(match ext {
        "mp3" => "audio/mpeg",
        "mp4" => "video/mp4",
        "m4a" => "audio/mp4",
        "wav" => "audio/wav",
        "ogg" => "audio/ogg",
        "jpg" => "image/jpeg",
        "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "webp" => "image/webp",
        "pdf" => "application/pdf",
        "zip" => "application/zip",
        "gz" => "application/gzip",
        "tar" => "application/x-tar",
        "txt" => "text/plain",
        "md" => "text/markdown",
        "html" => "text/html",
        "css" => "text/css",
        "js" => "text/javascript",
        "json" => "application/json",
        "xml" => "application/xml",
        "yaml" => "text/yaml",
        "yml" => "text/yaml",
        _ => "application/octet-stream",
    })
}

pub trait PathExt {
    // given a path, ensure that all parent directories of that path exist
    // and create any that don't exist
    fn create_all_parent_dir(&self) -> std::io::Result<()>;
    fn get_ext(&self) -> Option<Cow<'static, str>>;
    fn mimetype(&self) -> Cow<'static, str>;
    fn is_markdown(&self) -> bool;
}

impl PathExt for Path {
    fn create_all_parent_dir(&self) -> std::io::Result<()> {
        let dir = self.parent().unwrap();
        if !dir.exists() {
            std::fs::create_dir_all(dir)?;
        }
        Ok(())
    }

    fn get_ext(&self) -> Option<Cow<'static, str>> {
        if let Some(ext_osstr) = self.extension() {
            Some(Cow::Owned(ext_osstr.to_string_lossy().to_lowercase()))
        } else {
            None
        }
    }
    fn mimetype(&self) -> Cow<'static, str> {
        let ext = self.get_ext().unwrap_or(Cow::Borrowed(""));
        get_mimetype(&ext)
    }
    fn is_markdown(&self) -> bool {
        if let Some(ext) = self.extension() {
            if ext == "md" || ext == "markdown" {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    // importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_get_ext_png() {
        let result = Path::new("foo.png").get_ext().unwrap();
        assert_eq!(result, "png".to_string());
    }
    #[test]
    fn test_get_ext_empty() {
        let result = Path::new("").get_ext();
        assert_eq!(result, None);
    }
    #[test]
    fn test_get_mimetype_png() {
        let result = get_mimetype("png");
        assert_eq!(result, "image/png".to_string());
    }
    #[test]
    fn test_get_mimetype_empty() {
        let result = get_mimetype("");
        assert_eq!(result, "application/octet-stream".to_string());
    }
}

use pulldown_cmark::CowStr;
use std::path::Path;

pub fn is_audio_file(url: &CowStr) -> bool {
    let audio_format = ["mp3", "mp4", "m4a", "wav", "ogg"];
    let path = Path::new(url.as_ref());
    if let Some(ext_osstr) = path.extension() {
        let extension = ext_osstr.to_string_lossy().to_lowercase();
        if audio_format.contains(&extension.as_str()) {
            return true;
        }
    }
    false
}

// return the extension of an url as a string
pub fn get_ext(url: &CowStr) -> String {
    let path = Path::new(url.as_ref());
    if let Some(ext_osstr) = path.extension() {
        ext_osstr.to_string_lossy().to_lowercase()
    } else {
        String::new()
    }
}

// return mimetype given an extension
pub fn get_mimetype(ext: &str) -> String {
    match ext {
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
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    // importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_get_ext_png() {
        let result = get_ext(&CowStr::Borrowed("foo.png"));
        assert_eq!(result, "png".to_string());
    }
    #[test]
    fn test_get_ext_empty() {
        let result = get_ext(&CowStr::Borrowed(""));
        assert_eq!(result, "".to_string());
    }
    #[test]
    fn test_get_mimetype_png() {
        let result = get_mimetype(&CowStr::Borrowed("png"));
        assert_eq!(result, "image/png".to_string());
    }
    #[test]
    fn test_get_mimetype_empty() {
        let result = get_mimetype(&CowStr::Borrowed(""));
        assert_eq!(result, "application/octet-stream".to_string());
    }
}

use pulldown_cmark::CowStr;
use std::path::Path;

pub fn is_audio_file(url: &CowStr) -> bool {
    let audio_format = ["mp3", "mp4", "m4a", "wav", "ogg"];
    let path = Path::new(url.as_ref());
    if let Some(ext_osstr) = path.extension() {
        let extension = ext_osstr.to_string_lossy().to_lowercase();
        if audio_format.contains(&extension.as_str()) {
            return true
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
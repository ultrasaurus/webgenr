mod dir_entry;
pub use self::dir_entry::DirEntryExt;
mod path;
pub use self::path::get_mimetype;
pub use self::path::PathExt;

use pulldown_cmark::CowStr;
use std::{borrow::Cow, path::Path};

// return the extension of an url as a lowercase string
// or empty string, if there is no extension
pub fn get_ext<T: AsRef<str>>(url: T) -> Cow<'static, str> {
    let path = Path::new(url.as_ref());
    path.get_ext().unwrap_or(Cow::Borrowed(""))
}

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

#[cfg(test)]
mod tests {
    // importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_get_ext_png() {
        let result = get_ext("foo.png");
        assert_eq!(result, "png".to_string());
    }
    #[test]
    fn test_get_ext_empty() {
        let result = get_ext("");
        assert_eq!(result, "".to_string());
    }
}

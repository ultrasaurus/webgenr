//-- Path utlity functions -----------------------------------------------
// extensons to Path struct and related helper functions

use mime::Mime;
use std::{borrow::Cow, path::Path};

pub trait PathExt {
    // given a path, ensure that all parent directories of that path exist
    // and create any that don't exist
    fn create_all_parent_dir(&self) -> std::io::Result<()>;
    fn get_ext(&self) -> Option<Cow<'static, str>>;
    fn mimetype(&self) -> Option<Mime>;
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
    fn mimetype(&self) -> Option<Mime> {
        mime_guess::from_path(self).first()
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
    fn test_imetype_png() {
        let result = Path::new("foo.png").mimetype();
        assert_eq!(result, Some(mime::IMAGE_PNG));
    }
    #[test]
    fn test_get_mimetype_empty() {
        let result = Path::new("foo").mimetype();
        assert_eq!(result, None);
    }
}

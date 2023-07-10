//-- Path utlity functions
use std::{borrow::Cow, path::Path};

// return the extension of an url as a lowercase string
// or empty string, if there is no extension
pub fn get_ext<T: AsRef<str>>(url: T) -> Cow<'static, str> {
    let path = Path::new(url.as_ref());
    path.get_ext()
}

pub trait PathExt {
    // given a path, ensure that all parent directories of that path exist
    // and create any that don't exist
    fn create_all_parent_dir(&self) -> std::io::Result<()>;
    fn get_ext(&self) -> Cow<'static, str>;
}

impl PathExt for Path {
    fn create_all_parent_dir(&self) -> std::io::Result<()> {
        let dir = self.parent().unwrap();
        if !dir.exists() {
            std::fs::create_dir_all(dir)?;
        }
        Ok(())
    }

    fn get_ext(&self) -> Cow<'static, str> {
        if let Some(ext_osstr) = self.extension() {
            Cow::Owned(ext_osstr.to_string_lossy().to_lowercase())
        } else {
            Cow::Borrowed("")
        }
    }
}

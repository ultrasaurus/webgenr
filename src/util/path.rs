//-- Path utlity functions
use std::path::Path;

pub trait PathExt {
    // given a path, ensure that all parent directories of that path exist
    // and create any that don't exist
    fn create_all_parent_dir(&self) -> std::io::Result<()>;
}

impl PathExt for Path {
    fn create_all_parent_dir(&self) -> std::io::Result<()> {
        let dir = self.parent().unwrap();
        if !dir.exists() {
            std::fs::create_dir_all(dir)?;
        }
        Ok(())
    }
}

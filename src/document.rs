use std::path::PathBuf;

pub struct Document {
    pub source_path: PathBuf,
}

impl Document {
    pub fn new(source_path: PathBuf) -> Self {
        Document { source_path }
    }
}

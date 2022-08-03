use crate::document::Document;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

pub struct Web {
    pub in_path: PathBuf,
    pub out_path: PathBuf,
    doc_list: Vec<Document>,
}

impl Web {
    // return true if the DirEntry represents a hidden file or directory
    fn is_hidden(entry: &DirEntry) -> bool {
        entry
            .file_name()
            .to_str()
            .map(|s| s.starts_with("."))
            .unwrap_or(false)
    }

    fn new_doc_list<P: AsRef<Path>>(path_ref: P) -> io::Result<Vec<Document>> {
        let mut vec: Vec<Document> = Vec::new();
        let root = path_ref.as_ref().to_path_buf();

        let walker = WalkDir::new(root).follow_links(true).into_iter();
        for entry_result in walker.filter_entry(|e| !Web::is_hidden(e)) {
            let entry = entry_result?;
            let path = entry.path();
            if fs::metadata(path)?.is_file() {
                vec.push(Document::new(path));
            }
        }
        Ok(vec)
    }

    pub fn new<P: AsRef<Path>>(in_path: P, out_path: P) -> io::Result<Self> {
        Ok(Web {
            in_path: in_path.as_ref().to_path_buf(),
            out_path: out_path.as_ref().to_path_buf(),
            doc_list: Web::new_doc_list(in_path)?,
        })
    }

    pub fn gen(&mut self) -> std::io::Result<()> {
        info!("generating html for {} files", self.doc_list.len());
        for doc in &self.doc_list {
            let rel_path = doc
                .source_path
                .strip_prefix(&self.in_path)
                .expect("strip prefix match");
            let outpath = self.out_path.join(rel_path);
            let dir = outpath.parent().unwrap();
            if !dir.exists() {
                fs::create_dir_all(dir)?;
            }
            if doc.is_markdown() {
                let out_file = fs::OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open(outpath.with_extension("html"))?;
                info!("outfile created: {:?}", out_file);
                let writer = io::BufWriter::new(out_file);
                doc.write_html(writer);
                println!("convert-> {}", doc.source_path.display())
            } else {
                println!("  copy-> {}", doc.source_path.display());
                fs::copy(&doc.source_path, outpath)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_web() {
        let web = Web::new("markdown", "_website").expect("new web");
        assert_eq!(web.in_path, Path::new("markdown"));
        assert_eq!(web.out_path, Path::new("_website"));
    }
}

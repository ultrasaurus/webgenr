use crate::document::Document;
use anyhow::Result;
use handlebars::Handlebars;
use serde_json::json;
use std::fs;
use std::io;
use std::io::Write;
use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

pub struct Web<'a> {
    pub in_path: PathBuf,
    pub out_path: PathBuf,
    doc_list: Vec<Document>,
    template_registry: Handlebars<'a>,
}

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
    for entry_result in walker.filter_entry(|e| !is_hidden(e)) {
        let entry = entry_result?;
        let path = entry.path();
        if std::fs::metadata(path)?.is_file() {
            vec.push(Document::new(path));
        }
    }
    Ok(vec)
}

impl Web<'_> {
    pub fn new<P: AsRef<Path>>(in_path: P, out_path: P) -> Result<Self> {
        let mut handlebars = Handlebars::new();
        handlebars.register_template_file("default", "templates/default.html")?;
        handlebars.register_escape_fn(handlebars::no_escape);
        Ok(Web {
            in_path: in_path.as_ref().to_path_buf(),
            out_path: out_path.as_ref().to_path_buf(),
            doc_list: new_doc_list(in_path)?,
            template_registry: handlebars,
        })
    }

    fn outpath(&self, doc: &Document) -> std::io::Result<PathBuf> {
        let rel_path = doc
            .source_path
            .strip_prefix(&self.in_path)
            .expect("strip prefix match");
        Ok(self.out_path.join(rel_path))
    }

    fn create_dir_all(&self, path: &Path) -> std::io::Result<()> {
        let dir = path.parent().unwrap();
        if !dir.exists() {
            fs::create_dir_all(dir)?;
        }
        Ok(())
    }

    pub fn gen(&mut self) -> Result<()> {
        info!("generating html for {} files", self.doc_list.len());
        for doc in &self.doc_list {
            let outpath = self.outpath(doc)?;
            self.create_dir_all(&outpath)?;
            if doc.is_markdown() {
                let out_file = fs::OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open(outpath.with_extension("html"))?;

                // TODO: BUG? now self.outpath is incorrect
                // move extension logic into that function?
                info!(
                    "convert-> {}\t{}",
                    doc.source_path.display(),
                    outpath.with_extension("html").display()
                );

                let mut writer = io::BufWriter::new(out_file);

                let mut html = Vec::new();

                doc.write_html(&mut html)?;

                let html_string = String::from_utf8(html)?;

                let s = self
                    .template_registry
                    .render("default", &json!({ "body": html_string }))?;

                writer.write_all(s.as_bytes())?;
            } else {
                info!(
                    "copy-> {}\t{}",
                    doc.source_path.display(),
                    &outpath.display()
                );
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

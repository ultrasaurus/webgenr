use crate::document::Document;
use anyhow::{Context, Result};
use handlebars::Handlebars;
use rust_embed::RustEmbed;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

pub struct Web<'a> {
    pub in_path: PathBuf,
    pub out_path: PathBuf,
    doc_list: Vec<Document>,
    pub template_registry: Handlebars<'a>,
}

#[derive(RustEmbed)]
#[folder = "templates/"]
struct Asset;

// return true if the DirEntry represents a hidden file or directory
fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

fn new_doc_list<P: AsRef<Path>>(path_ref: P) -> anyhow::Result<Vec<Document>> {
    let mut vec: Vec<Document> = Vec::new();
    let root = path_ref.as_ref().to_path_buf();

    let walker = WalkDir::new(root).follow_links(true).into_iter();
    for entry_result in walker.filter_entry(|e| !is_hidden(e)) {
        let entry = entry_result?;
        let path = entry.path();
        if fs::metadata(path)?.is_file() {
            let doc = Document::new(path)
                .with_context(|| format!("Failed to read {}", path.display()))?;
            vec.push(doc);
        }
    }
    Ok(vec)
}

impl Web<'_> {
    pub fn new<P: AsRef<Path>>(in_path: P, out_path: P, templatedir_path: P) -> Result<Self> {
        fs::create_dir_all(&in_path)?;
        fs::create_dir_all(&templatedir_path)?;
        let default_template_path = Path::new("").join(&templatedir_path).join("default.hbs");
        if default_template_path.exists() != true {
            // not sure why I need to use this syntax, instead of Asset.get
            if let Some(default_template) = Asset::get("default.hbs") {
                let mut file = std::fs::OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open(&default_template_path)
                    .unwrap();
                file.write_all(default_template.data.as_ref())?;
            }
        }
        let mut handlebars = Handlebars::new();
        handlebars.register_templates_directory(".hbs", templatedir_path)?;
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
            std::fs::create_dir_all(dir)?;
        }
        Ok(())
    }

    pub fn gen(&mut self) -> Result<usize> {
        if self.doc_list.len() == 0 {
            println!(
                "\nplease add markdown files (.md extension) to source directory: {}\n",
                self.in_path.display()
            );
        }
        info!("generating html for {} files", self.doc_list.len());
        for doc in &self.doc_list {
            let outpath = self.outpath(doc)?;
            self.create_dir_all(&outpath)?;
            doc.webgen(&self)?;
        }
        Ok(self.doc_list.len())
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

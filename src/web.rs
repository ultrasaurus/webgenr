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

    fn make_book_internal(&self, author: &str, title: &str) -> anyhow::Result<()> {
        use epub_builder::EpubBuilder;
        use epub_builder::EpubContent;
        use epub_builder::ReferenceType;
        use epub_builder::ZipLibrary;
        use std::fs::File;
        use anyhow::anyhow;
        
        let writer = std::fs::File::create("book.epub")?;
        let zip_lib = ZipLibrary::new().map_err(|err| anyhow!("initializing zip {:#?}", err))?;
        let mut epub = EpubBuilder::new(zip_lib)
            .map_err(|err| anyhow!("initializing epub {:#?}", err))?;

        epub.add_author(author);
        epub.set_title(title);
        let mut chapter_number = 1;
        for doc in &self.doc_list {
            if Some(std::ffi::OsStr::new("cover")) == doc.source_path.file_stem(){
                let default_extension = "png";
                let extension = match doc.source_path.file_stem() {
                    Some(os_str) => {
                        match os_str.to_str() {
                            Some(str) => str,
                            None => {
                                println!("can't convert file extension {:?} to str", os_str);
                                default_extension
                            },
                        }
                    },
                    None => {
                        println!("no file extension for cover image, assuming png");
                        default_extension
                    },
                };
               epub.add_cover_image(&doc.source_path, 
                        File::open(&doc.source_path)?, 
                        format!("image/{}", extension))
                        .map_err(|err| anyhow!("adding cover image {:#?}", err))?;
            } else {
                
                let default_zip_path = format!("chapter{}.xhtml", chapter_number);
                let chapter_title = format!("Chapter {}", chapter_number);  // TODO: get from YAML front matter
                let zip_path = match doc.source_path.file_stem() {
                    Some(os_str) => format!("{}.xhtml", os_str.to_string_lossy()),
                    None => default_zip_path,
                };
                println!("adding {}\tas {},\ttitle: {}", doc.source_path.display(), zip_path, chapter_title);
                epub.add_content(
                    EpubContent::new(zip_path, File::open(&doc.source_path)?)
                        .title(chapter_title)
                        .reftype(ReferenceType::Text),
                )
                .map_err(|err| anyhow!("adding content to epub {:#?}", err))?;
                chapter_number = chapter_number +1;



            //   epub.add_content(
            // EpubContent::new(&doc.source_path, File::open(doc.source_path)?)
            //     .title("First computer program")
            //     .reftype(ReferenceType::Text))
            //     .map_err(|err| anyhow!("adding content to epub {:#?}", err))?;
            
            }
        }
        epub.generate(writer)
        .map_err(|err| anyhow!("generating epub {:#?}", err))?;

        Ok(())
    }

    pub fn gen_book(&mut self) -> Result<usize> {
        if self.doc_list.len() == 0 {
            println!(
                "\nplease add files to source directory: {}\n",
                self.in_path.display()
            );
        }
        info!("generating ePub for {} files", self.doc_list.len());
        
        match self.make_book_internal("Author Name", "My Book") {
            Err(e) => anyhow::bail!("Problem creating ebook: {:#?}", e),
            Ok(_) => Ok(self.doc_list.len())
        }
    }

    pub fn gen_website(&mut self) -> Result<usize> {
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

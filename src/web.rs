use crate::document::Document;
use crate::util::*;
use anyhow::Context;
use epub_builder::{EpubBuilder, ZipLibrary};
use handlebars::Handlebars;
use rust_embed::RustEmbed;
use std::ffi::OsStr;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct Web<'a> {
    pub in_path: PathBuf,
    pub out_path: PathBuf,
    pub template_dir_path: PathBuf,
    doc_list: Vec<Document>,
    pub template_registry: Handlebars<'a>,
}

#[derive(RustEmbed)]
#[folder = "templates/"]
#[exclude = ".*"] // ignore hidden files
struct Asset;

// this is a weird plance for this function
// TODO: consider refactoring once book/website feel done
fn new_doc_list<P: AsRef<Path>>(path_ref: P) -> anyhow::Result<Vec<Document>> {
    let mut vec: Vec<Document> = Vec::new();
    let root = path_ref.as_ref().to_path_buf();

    let walker = WalkDir::new(root).follow_links(true).into_iter();
    for entry_result in walker.filter_entry(|e| !e.is_hidden()) {
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
    // copy embedded templates into given directory path
    fn inflate_default_templates<P: AsRef<Path>>(templatedir_path: P) -> anyhow::Result<()> {
        info!("inflating default templates");
        for relative_path_str in Asset::iter() {
            info!("  {}", relative_path_str);
            let relative_path = PathBuf::from(relative_path_str.to_string());
            let new_template_path = Path::new("").join(&templatedir_path).join(&relative_path);
            new_template_path.create_all_parent_dir()?;
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .open(&new_template_path)
                .unwrap();
            file.write_all(&Asset::get(&relative_path_str).unwrap().data.as_ref())?;
        }
        Ok(())
    }

    // copy files recursively from `source_dir` to `dest_dir`
    // omitting files with extension `omit_ext` (.gitignore syntax from globwalk crate)
    fn copy_files<P: AsRef<Path>>(
        source_dir: P,
        dest_dir: P,
        omit_ext: &str,
    ) -> anyhow::Result<()> {
        info!("copyfiles, omitting {}", omit_ext);
        info!(" std::env::current_dir: {:?}", std::env::current_dir());
        let walker = WalkDir::new(&source_dir).follow_links(true).into_iter();
        for entry_result in walker
            .filter_entry(|e| !e.is_hidden() && e.path().extension() != Some(OsStr::new(omit_ext)))
        {
            if let Ok(dir_entry) = entry_result {
                let rel_path = dir_entry
                    .path()
                    .strip_prefix(&source_dir)
                    .expect("strip prefix match");

                let dest_path = PathBuf::from(dest_dir.as_ref()).join(rel_path);
                if dir_entry.path().is_dir() {
                    info!("  dir: {:?}", dest_path);
                    fs::create_dir_all(&dest_path)?;
                } else {
                    // copy file
                    info!("  file: {:?}", dir_entry.path());

                    match fs::copy(dir_entry.path(), &dest_path) {
                        Ok(bytes) => info!(
                            "copy {} bytes-> {}\t{}",
                            bytes,
                            dir_entry.path().display(),
                            &dest_path.display()
                        ),
                        Err(e) => anyhow::bail!(
                            "error: {}, failed to copy from: {} to {}",
                            e,
                            dir_entry.path().display(),
                            &dest_path.display()
                        ),
                    }
                }
            }
        }
        Ok(())
    }

    fn path_not_found<P: AsRef<Path>>(path: P) -> anyhow::Result<bool> {
        if let Err(err) = fs::metadata(&path) {
            match err.kind() {
                std::io::ErrorKind::NotFound => return Ok(true),
                _ => {
                    error!("Error finding templates directory"); // TODO: remove 'templates' or take param?
                    return Err(err.into());
                }
            }
        }
        Ok(false) // path was found
    }

    // creates required folders (but does not delete any old files)
    pub fn new<P: AsRef<Path>>(
        in_path: P,
        out_path: P,
        templatedir_path: P,
    ) -> anyhow::Result<Self> {
        fs::create_dir_all(&in_path)?;
        // create templates directory and fill with default templates if needed
        if Self::path_not_found(&templatedir_path)? {
            fs::create_dir_all(&templatedir_path)?;
            Self::inflate_default_templates(&templatedir_path)?;
        }

        let mut handlebars = Handlebars::new();
        handlebars.register_templates_directory(".hbs", &templatedir_path)?;
        handlebars.register_escape_fn(handlebars::no_escape);
        Ok(Web {
            in_path: in_path.as_ref().to_path_buf(),
            out_path: out_path.as_ref().to_path_buf(),
            template_dir_path: templatedir_path.as_ref().to_path_buf(),
            doc_list: new_doc_list(in_path)?,
            template_registry: handlebars,
        })
    }

    // given a `source_path` return corresponding output path
    fn outpath(&self, doc: &Document) -> std::io::Result<PathBuf> {
        let rel_path = doc
            .source_path
            .strip_prefix(&self.in_path)
            .expect("strip prefix match");
        Ok(self.out_path.join(rel_path))
    }

    fn add_template_stylesheet_files(
        &self,
        mut epub: EpubBuilder<ZipLibrary>,
    ) -> anyhow::Result<EpubBuilder<ZipLibrary>> {
        info!(
            "add_template_stylesheet_files from {}",
            self.template_dir_path.display()
        );
        info!("  std::env::current_dir: {:?}", std::env::current_dir());
        let walker = WalkDir::new(&self.template_dir_path)
            .follow_links(true)
            .into_iter();

        for entry_result in walker
            .filter_entry(|e| !e.is_hidden() && e.path().extension() != Some(OsStr::new("hbs")))
        {
            let dir_entry = entry_result?;
            if dir_entry.file_type().is_file() {
                info!("  dir_entry: {:?}", dir_entry.path().display());

                let rel_path = dir_entry
                    .path()
                    .strip_prefix(&self.template_dir_path)
                    .expect("strip prefix match");

                let mimetype = rel_path.mimetype();
                info!("  rel_path: {}, mimetype: {}", rel_path.display(), mimetype);
                let result =
                    epub.add_resource(rel_path, fs::File::open(dir_entry.path())?, mimetype);
                // TODO: figure out why "?" doesn't work at end of statement above
                if result.is_err() {
                    anyhow::bail!(
                        "failed to add resource to epub: {}",
                        dir_entry.path().display()
                    )
                }
            }
        }
        info!("done");
        Ok(epub)
    }

    fn make_book_internal(&self, author: &str, title: &str) -> anyhow::Result<()> {
        use anyhow::anyhow;
        use epub_builder::EpubContent;
        use epub_builder::ReferenceType;
        use std::fs::File;

        let epub_filename = "book.epub"; // TODO: use outpath or add book output path?
        let writer = std::fs::File::create(epub_filename)?;
        let zip_lib = ZipLibrary::new().map_err(|err| anyhow!("initializing zip {:#?}", err))?;
        let mut epub =
            EpubBuilder::new(zip_lib).map_err(|err| anyhow!("initializing epub {:#?}", err))?;

        epub = self
            .add_template_stylesheet_files(epub)
            .map_err(|err| anyhow!("adding epub stylesheets {:#?}", err))?;

        epub.add_author(author);
        epub.set_title(title);
        let mut chapter_number = 1;

        for doc in &self.doc_list {
            let file_stem = doc.file_stem()?;

            match file_stem {
                "cover" | "_cover" => {
                    println!("cover: {}", doc.source_path.display());
                    let default_extension = "png";
                    let extension = match doc.source_path.file_stem() {
                        Some(os_str) => match os_str.to_str() {
                            Some(str) => str,
                            None => {
                                println!("can't convert file extension {:?} to str", os_str);
                                default_extension
                            }
                        },
                        None => {
                            println!("no file extension for cover image, assuming png");
                            default_extension
                        }
                    };
                    epub.add_cover_image(
                        &doc.source_path,
                        File::open(&doc.source_path)?,
                        format!("image/{}", extension),
                    )
                    .map_err(|err| anyhow!("adding cover image {:#?}", err))?;
                }
                "title" | "_title" => {
                    println!("title page: {}", doc.source_path.display());
                    let file_name = doc.source_path.file_name().unwrap().to_string_lossy();
                    if doc.is_markdown() {
                        println!(
                            "converting {}\tto {} for title page",
                            doc.source_path.display(),
                            file_name
                        );
                        // TODO: refactor webgen to create a fn that returns impl Read something
                        let s: String = doc.gen_html(&self)?;
                        epub.add_content(
                            EpubContent::new(file_name, s.as_bytes())
                                .title("Title Page")
                                .reftype(ReferenceType::TitlePage),
                        )
                        .map_err(|err| anyhow!("adding title page to epub {:#?}", err))?;
                    } else {
                        epub.add_content(
                            EpubContent::new(file_name, File::open(&doc.source_path)?)
                                .title("Title Page")
                                .reftype(ReferenceType::TitlePage),
                        )
                        .map_err(|err| anyhow!("adding title page to epub {:#?}", err))?;
                    }
                }
                _ => {
                    let chapter_title = format!("Chapter {}", chapter_number); // TODO: get from YAML front matter
                    let zip_path = format!("{}.xhtml", file_stem);
                    if doc.is_markdown() {
                        println!(
                            "converting {}\tto {},\ttitle: {}",
                            doc.source_path.display(),
                            zip_path,
                            chapter_title
                        );

                        // TODO: refactor webgen to create a fn that returns impl Read something
                        let s: String = doc.gen_html(&self)?;
                        epub.add_content(
                            EpubContent::new(zip_path, s.as_bytes())
                                .title(chapter_title)
                                .reftype(ReferenceType::Text),
                        )
                        .map_err(|err| anyhow!("adding content to epub {:#?}", err))?;
                    } else {
                        println!(
                            "adding {}\tas {},\ttitle: {}",
                            doc.source_path.display(),
                            zip_path,
                            chapter_title
                        );
                        epub.add_content(
                            EpubContent::new(zip_path, File::open(&doc.source_path)?)
                                .title(chapter_title)
                                .reftype(ReferenceType::Text),
                        )
                        .map_err(|err| anyhow!("adding content to epub {:#?}", err))?;
                    };

                    chapter_number = chapter_number + 1;
                }
            } // match file_stem
        }
        epub.generate(writer)
            .map_err(|err| anyhow!("generating epub {:#?}", err))?;

        info!("book created: {}", epub_filename);
        Ok(())
    }

    // if folder exists, delete it & all contents and create new
    fn clean_folder<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
        if Path::new(path.as_ref()).exists() {
            fs::remove_dir_all(&path)?;
        }
        fs::create_dir_all(&path)?;
        Ok(())
    }

    fn source_directory_has_files(&self) -> anyhow::Result<usize> {
        let num_files = self.doc_list.len();
        if num_files == 0 {
            anyhow::bail!(
                "please add files to source directory: {}",
                self.in_path.display()
            )
        }
        Ok(num_files)
    }
    pub fn gen_book(&mut self) -> anyhow::Result<usize> {
        self.source_directory_has_files()?;
        info!("generating ePub for {} files", self.doc_list.len());

        match self.make_book_internal("Author Name", "My Book") {
            Err(e) => anyhow::bail!("Problem creating ebook: {}", e),
            Ok(_) => Ok(self.doc_list.len()),
        }
    }

    fn gen_website_clean_and_setup_outpath(&self) -> anyhow::Result<()> {
        Self::clean_folder(&self.out_path)?;
        Self::copy_files(&self.template_dir_path, &self.out_path, "hbs")?;
        Ok(())
    }

    pub fn gen_website(&mut self) -> anyhow::Result<usize> {
        self.source_directory_has_files()?;
        self.gen_website_clean_and_setup_outpath()?;
        info!("generating html for {} files", self.doc_list.len());
        for doc in &self.doc_list {
            let outpath = self.outpath(doc)?;
            outpath.create_all_parent_dir()?;
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
        let web = Web::new("markdown", "_website", "templates").expect("new web");
        assert_eq!(web.in_path, Path::new("markdown"));
        assert_eq!(web.out_path, Path::new("_website"));
        assert_eq!(web.template_dir_path, Path::new("templates"));
    }
}

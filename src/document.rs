use crate::Web;
use pulldown_cmark::{html, Event, Options, Parser as MarkdownParser, Tag};
use serde_json;
use serde_yaml;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use anyhow::bail;

pub struct FrontMatter {
    vars: std::collections::HashMap<String, String>,
}

impl FrontMatter {
    // currently only supports string data types, like
    // ---
    // title: "My Website"
    // ---
    // input: if it has yaml front matter, the yaml is removeed
    // returns: yaml front matter as struct or None
    pub fn split_yml_from_string<'a>(input: &mut String) -> anyhow::Result<Option<FrontMatter>> {
        let mut yaml: bool = false;
        let mut yaml_separator: &str = "";
        let mut yaml_text: Option<String> = None;
        for sep in vec!["---\n", "---\r\n"] {
            if input.starts_with(sep) {
                yaml = true;
                yaml_separator = sep;
            }
        }

        if yaml {
            let mut split = input.split(yaml_separator);
            let _empty_string = split.next(); // split will give us empty string after the newline
            if let Some(yaml_str) = split.next() {
                yaml_text = Some(yaml_str.to_string());
            }
            if let Some(yaml_string) = yaml_text {
                input.drain(..yaml_string.len() + yaml_separator.len() * 2);
                // println!("========== yaml ===========");
                // println!("{}", &yaml_string);
                // println!("========== text ===========");
                // println!("{}", &input);
                // println!("===========================");
                return Ok(Some(FrontMatter {
                    vars: serde_yaml::from_str(&yaml_string)?, // TODO: support more complex data types?
                }));
            }
        }
        Ok(None)
    }
}

pub enum DocumentInfo {
    Markdown {
        front_matter: Option<FrontMatter>,
        text: String,
    },
    Other,
}
pub struct Document {
    pub source_path: PathBuf,
    pub info: DocumentInfo,
}

impl Document {
    pub fn new<P: AsRef<Path>>(source_path: P) -> anyhow::Result<Self> {
        let info = if Self::is_markdown_path(&source_path) {
            let mut f = fs::File::open(&source_path)?;
            let mut markdown = String::new();
            f.read_to_string(&mut markdown)?;
            let front_matter = FrontMatter::split_yml_from_string(&mut markdown)?;
            DocumentInfo::Markdown {
                front_matter,
                text: markdown,
            }
        } else {
            DocumentInfo::Other
        };
        Ok(Document {
            source_path: source_path.as_ref().to_path_buf(),
            info,
        })
    }
    // The only reason this would fail is if at some point we create a
    // Document with path from user input.
    // Right now that would only happen for developer error, so this
    // is simply wrapper function for rust std api providing nice error messages
    pub fn file_stem(&self) -> anyhow::Result<&str> {
        match self.source_path.file_stem() {
            Some(os_stem) =>
                match os_stem.to_str() {
                    Some(stem) => Ok(stem),
                    None => bail!("Document: could not parse path, perhaps UTF8 conversion error for {}", self.source_path.display())
                },
            None => bail!("Document: unexpected empty file name for: {}", self.source_path.display()),
        }
    }

    fn outpath(&self, root: &PathBuf, out_dir: &PathBuf) -> std::io::Result<PathBuf> {
        let rel_path = self
            .source_path
            .strip_prefix(&root)
            .expect("strip prefix match");
        Ok(out_dir.join(rel_path))
    }

    pub fn webgen(&self, context: &Web) -> anyhow::Result<()> {
        let outpath = self.outpath(&context.in_path, &context.out_path)?;
        match &self.info {
            DocumentInfo::Other => {
                // copy file
                info!(
                    "copy-> {}\t{}",
                    self.source_path.display(),
                    &outpath.display()
                );
                std::fs::copy(&self.source_path, outpath)?;
            }
            DocumentInfo::Markdown { front_matter, text } => {
                let out_file = fs::OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open(outpath.with_extension("html"))?;
                info!(
                    "convert-> {}\t{}",
                    self.source_path.display(),
                    outpath.with_extension("html").display()
                );
                let mut writer = std::io::BufWriter::new(out_file);
                let mut html = Vec::new();
                Self::write_html(&mut html, &text)?;
                let html_string = String::from_utf8(html)?;

                let mut template_vars = match front_matter {
                    Some(front_matter) => front_matter.vars.clone(),
                    None => Default::default(),
                };
                if let Some(_) = template_vars.insert("body".into(), html_string) {
                    println!("warning: yaml var 'body' will be ignored");
                }

                let s = context
                    .template_registry
                    .render("default", &serde_json::json!(template_vars))?;

                writer.write_all(s.as_bytes())?;
            }
        }
        Ok(())
    }
    pub fn is_markdown(&self) -> bool {
        match self.info {
            DocumentInfo::Markdown { .. } => true,
            _ => false,
        }
    }

    pub fn is_markdown_path<P: AsRef<Path>>(path: P) -> bool {
        if let Some(ext) = path.as_ref().to_path_buf().extension() {
            if ext == "md" || ext == "markdown" {
                return true;
            }
        }
        false
    }

    fn write_html<W: Write>(out_writer: W, text: &String) -> anyhow::Result<()> {
        // Set up options and parser.
        let mut options = Options::empty();
        // Strikethroughs are not part of the CommonMark standard
        // so must be enabled explicitly (TODO: maybe configure?)
        options.insert(Options::ENABLE_STRIKETHROUGH);
        let parser = MarkdownParser::new_ext(&text, options).map(|event| {
            // transform links from .md to .html
            match event {
                Event::Start(Tag::Link(link_type, url, title)) => {
                    let md_suffix = ".md";
                    if url.ends_with(md_suffix) {
                        let new_url = format!("{}.html", url.trim_end_matches(md_suffix));
                        Event::Start(Tag::Link(link_type, new_url.into(), title))
                    } else {
                        Event::Start(Tag::Link(link_type, url, title))
                    }
                }
                _ => event,
            }
        });

        html::write_html(out_writer, parser)?;
        Ok(())
    }
}

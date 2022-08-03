use pulldown_cmark::{html, Options, Parser as MarkdownParser};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

pub struct Document {
    pub source_path: PathBuf,
}

impl Document {
    pub fn new<P: AsRef<Path>>(source_path: P) -> Self {
        Document {
            source_path: source_path.as_ref().to_path_buf(),
        }
    }
    pub fn is_markdown(&self) -> bool {
        if let Some(ext) = self.source_path.extension() {
            if ext == "md" || ext == "markdown" {
                return true;
            }
        }
        false
    }

    pub fn write_html<W: Write>(&self, out_writer: W) -> std::io::Result<()> {
        let mut f = fs::File::open(&self.source_path).expect("file not found");
        let mut markdown_input = String::new();
        f.read_to_string(&mut markdown_input)
            .expect("error reading file");
        //info!("input: \n{}", markdown_input);

        // Set up options and parser.
        let mut options = Options::empty();
        // Strikethroughs are not part of the CommonMark standard
        // so must be enabled explicitly (TODO: maybe configure?)
        options.insert(Options::ENABLE_STRIKETHROUGH);
        let parser = MarkdownParser::new_ext(&markdown_input, options);

        html::write_html(out_writer, parser).expect("unable to write converted html");
        Ok(())
    }
}

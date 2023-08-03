use crate::util::*;
use crate::Web;
use anyhow::bail;
use pulldown_cmark::{Event, Parser as MarkdownParser, Tag};
use serde_json;
use serde_yaml;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

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
        let source_pathbuf = source_path.as_ref().to_path_buf();
        let info = if source_pathbuf.is_markdown() {
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
            source_path: source_pathbuf,
            info,
        })
    }
    // The only reason this would fail is if at some point we create a
    // Document with path from user input.
    // Right now that would only happen for developer error, so this
    // is simply wrapper function for rust std api providing nice error messages
    pub fn file_stem(&self) -> anyhow::Result<&str> {
        match self.source_path.file_stem() {
            Some(os_stem) => match os_stem.to_str() {
                Some(stem) => Ok(stem),
                None => bail!(
                    "Document: could not parse path, perhaps UTF8 conversion error for {}",
                    self.source_path.display()
                ),
            },
            None => bail!(
                "Document: unexpected empty file name for: {}",
                self.source_path.display()
            ),
        }
    }

    fn outpath(&self, root: &PathBuf, out_dir: &PathBuf) -> std::io::Result<PathBuf> {
        let rel_path = self
            .source_path
            .strip_prefix(&root)
            .expect("strip prefix match");
        Ok(out_dir.join(rel_path))
    }

    pub fn gen_html(&self, context: &Web) -> anyhow::Result<String> {
        match &self.info {
            DocumentInfo::Markdown { front_matter, text } => {
                // generate html
                let mut html = Vec::new();
                Self::write_html(&mut html, &text)?;
                let html_string = String::from_utf8(html)?;

                // insert into handlebars template
                let mut template_vars = match front_matter {
                    Some(front_matter) => front_matter.vars.clone(),
                    None => Default::default(),
                };
                if let Some(_) = template_vars.insert("body".into(), html_string) {
                    println!("warning: yaml var 'body' will be ignored");
                }

                Ok(context
                    .template_registry
                    .render("default", &serde_json::json!(template_vars))?)
            }
            _ => {
                // unimplemented, TODO: return appropriate error
                bail!("unimplemented!")
            }
        }
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
            DocumentInfo::Markdown {
                front_matter: _,
                text: _,
            } => {
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

                // // generate html
                // let mut html = Vec::new();
                // Self::write_html(&mut html, &text)?;
                // let html_string = String::from_utf8(html)?;

                // // insert into handlebars template
                // let mut template_vars = match front_matter {
                //     Some(front_matter) => front_matter.vars.clone(),
                //     None => Default::default(),
                // };
                // if let Some(_) = template_vars.insert("body".into(), html_string) {
                //     println!("warning: yaml var 'body' will be ignored");
                // }

                // let s = context
                //     .template_registry
                //     .render("default", &serde_json::json!(template_vars))?;
                let s = self.gen_html(context)?;
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

    // private utility function
    fn write_html<W: Write>(out_writer: W, markdown: &String) -> anyhow::Result<()> {
        // Set up pulldown_cmark options and parser.
        let mut options = pulldown_cmark::Options::empty();
        // Strikethroughs are not part of the CommonMark standard
        // so must be enabled explicitly (TODO: maybe configure?)
        options.insert(pulldown_cmark::Options::ENABLE_STRIKETHROUGH);

        let mut parser = MarkdownParser::new_ext(&markdown, options);

        let mut new_event_list: Vec<Event> = Vec::new();
        while let Some(event) = parser.next() {
            let next_event = match event {
                Event::Start(Tag::Link(link_type, url, title)) => {
                    match mime_guess::from_path(url.to_string()).first() {
                        None => {
                            // no extension or no matching mime for extension
                            // just return the link unmodified
                            Event::Start(Tag::Link(link_type, url, title))
                        }
                        Some(mimetype) => {
                            match (mimetype.type_(), mimetype.subtype()) {
                                (mime::TEXT, subtype) if subtype == "markdown" => {
                                    // TODO: remove any extension, not just .md
                                    // this will fail for .markdown
                                    let new_url = format!("{}.html", url.trim_end_matches(".md"));
                                    Event::Start(Tag::Link(link_type, new_url.into(), title))
                                }
                                (mime::AUDIO, _) => {
                                    let link_text = if let Some(next_event) = parser.next() {
                                        if let Event::Text(text) = next_event {
                                            parser.next(); // skip past Event::End
                                            text
                                        } else {
                                            // no text event, just Event::End
                                            "#".into()
                                        }
                                    } else {
                                        "".into()
                                    };
                                    let link_tag= format!("<a href=\"{}\" title=\"{}\" class=\"audio\"><span class=\"fa-solid fa-play\">{}</span></a>",
                                        &url, &title, &link_text);
                                    let audio_tag= format!("<audio controls><source src=\"{}\" type=\"{}\">Your browser does not support the audio element. {}</audio>",
                                        url, mimetype, &link_tag);
                                    Event::Html(audio_tag.into())
                                }
                                (_, _) => Event::Start(Tag::Link(link_type, url, title)),
                            }
                        }
                    }
                }
                _ => event,
            };
            new_event_list.push(next_event);
        }

        pulldown_cmark::html::write_html(out_writer, new_event_list.into_iter())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HELLO_MD: &str = "hello world...";
    const HELLO_HTML: &str = "<p>hello world...</p>\n";

    const EMPTY_BUF: &[u8] = b"";

    #[test]
    fn test_write_html_empty() {
        let markdown = "".to_string();
        let mut output = Vec::new();
        Document::write_html(&mut output, &markdown).unwrap();
        assert_eq!(&output, EMPTY_BUF);
    }

    #[test]
    fn test_write_html_simple_string() {
        let markdown: String = HELLO_MD.to_string();
        let mut output = Vec::new();
        Document::write_html(&mut output, &markdown).unwrap();
        let output_str = std::str::from_utf8(&output).unwrap();
        assert_eq!(output_str, HELLO_HTML);
    }

    struct TestData<'a> {
        md: &'a str,
        html: &'a str,
    }

    fn verify_write_html_with_test_data(test_data: Vec<TestData>) {
        test_data.iter().for_each(|test| {
            let markdown: String = test.md.to_string();
            let mut output = Vec::new();
            Document::write_html(&mut output, &markdown).unwrap();
            let output_str = std::str::from_utf8(&output).unwrap();
            assert_eq!(output_str, test.html);
        });
    }

    #[test]
    // test of standard CommonMark formatting
    fn test_write_html_cmark_basics() {
        let test_data = vec![
            TestData {
                // basic text
                md: "hello",
                html: "<p>hello</p>\n",
            },
            TestData {
                // unordered list
                md: "* one\n* two",
                html: "<ul>\n<li>one</li>\n<li>two</li>\n</ul>\n",
            },
            TestData {
                // simple link
                md: "link: [thing](https://example.com/thing)",
                html: "<p>link: <a href=\"https://example.com/thing\">thing</a></p>\n",
            },
            TestData {
                // link with mis-matched quote in title
                md: r#"link: ["thing](https://example.com/thing)"#,
                html: "<p>link: <a href=\"https://example.com/thing\">&quot;thing</a></p>\n",
            },
        ];
        verify_write_html_with_test_data(test_data);
    }

    #[test]
    // test of converting markdwon links to .html
    fn test_write_html_link_to_markdown() {
        let test_data = vec![TestData {
            // .md link conversion to .html
            md: "link: [thing](https://example.com/thing.md)",
            html: "<p>link: <a href=\"https://example.com/thing.html\">thing</a></p>\n",
        }];
        verify_write_html_with_test_data(test_data);
    }
}

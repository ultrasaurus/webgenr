use clap::{AppSettings, Parser};
use pulldown_cmark::{html, Options, Parser as MarkdownParser};
use std::fs;
use std::io::{BufWriter, Read};
use std::path::Path;
use walkdir::{DirEntry, WalkDir};

extern crate pretty_env_logger;

#[macro_use]
extern crate log;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(global_setting(AppSettings::DeriveDisplayOrder))]
struct Cli {
    /// directory path for markdown source files
    #[clap(short, long, value_parser, default_value = "markdown")]
    inpath: String,

    /// destination path for html
    #[clap(short, long, value_parser, default_value = ".site")]
    outpath: String,
}

// struct Config {
//     outpath: String,
// }

// impl Config {
//     pub fn new() -> Self {
//         Config {
//             outpath: DEFAULT_OUTPATH,
//         }
//     }
// }

fn clean_folder(path: &str) -> std::io::Result<()> {
    // delete if exists and create new
    if Path::new(path).exists() {
        fs::remove_dir_all(path)?;
    }
    fs::create_dir_all(path)?;
    Ok(())
}

// fn copy_static_files(from_path: &str, to_path: &str) {
//     // ignore hidden files and directories (e.g. .dist, .gitignore)
//     // copy media files
// }

fn is_markdown_filename(s: &str) -> bool {
    s.ends_with(".md") || s.ends_with(".markdown")
}

fn is_markdown_entry(entry: &DirEntry) -> bool {
    match entry.file_name().to_str() {
        Some(s) => is_markdown_filename(s),
        None => false,
    }
}
// return true if the file is a source file for markdown to html conversion
fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

fn process_files(inpath: &str, outpath: &str) -> std::io::Result<()> {
    let walker = WalkDir::new(inpath).follow_links(true).into_iter();
    for entry in walker.filter_entry(|e| !is_hidden(e)) {
        let e = entry?;
        if is_markdown_entry(&e) {
            println!("{}", e.path().display());
        }
    }

    // iterate over all files
    let path = "markdown/index.md";
    let mut f = fs::File::open(path).expect("file not found");

    let mut markdown_input = String::new();
    f.read_to_string(&mut markdown_input)
        .expect("error reading file");

    info!("input: \n{}", markdown_input);
    // Set up options and parser.
    let mut options = Options::empty();
    // Strikethroughs are not part of the CommonMark standard
    // so must be enabled explicitly (TODO: maybe configure?)
    options.insert(Options::ENABLE_STRIKETHROUGH);
    let parser = MarkdownParser::new_ext(&markdown_input, options);

    // Write to a file
    let out_file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(".site/test.html")
        .expect("could not open output file");

    let writer = BufWriter::new(out_file);
    html::write_html(writer, parser).expect("unable to write to file");
    info!("HTML file written!");
    Ok(())
}

fn main() {
    pretty_env_logger::init();
    let cli = Cli::parse();
    clean_folder(&cli.outpath).expect("could not setup output directory");
    process_files(&cli.inpath, &&cli.outpath).expect("process files")
}

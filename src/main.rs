use clap::{AppSettings, Parser};
use pulldown_cmark::{html, Options, Parser as MarkdownParser};
use std::fs;
use std::io::{BufWriter, Read, Write};
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

// if path references a markdown file, return filename stem
// index.md -> Some(index)
// thing.png -> None
fn markdown_filename(path: &Path) -> Option<&str> {
    if let Some(ext) = path.extension() {
        if ext == "md" || ext == "markdown" {
            if let Some(stem) = path.file_stem() {
                return stem.to_str();
            }
        }
    }
    None
}

// return true if the file is a source file for markdown to html conversion
fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

fn convert_one_file<W: Write>(source_path: &Path, out_writer: W) -> std::io::Result<()> {
    Ok(())
}

fn process_files(inpath: &str, outpath_str: &str) -> std::io::Result<()> {
    let outpath = Path::new(outpath_str);
    let walker = WalkDir::new(inpath).follow_links(true).into_iter();
    for entry_result in walker.filter_entry(|e| !is_hidden(e)) {
        let entry = entry_result?;
        let path = entry.path();
        if let Some(filename) = markdown_filename(&path) {
            println!("{}", path.display());
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

            // let out_filename = format!("{}.html", filename);
            let out_filepath = outpath.join(format!("{}.html", filename));
            // TODO: match directory structure
            // Write to a file
            let out_file = fs::OpenOptions::new()
                .write(true)
                .create(true)
                .open(out_filepath)
                .expect("could not open output file");

            let writer = BufWriter::new(out_file);
            html::write_html(writer, parser).expect("unable to write to file");
            info!("HTML file written!");
        }
    }

    // iterate over all files
    // let path = "markdown/index.md";
    // let mut f = fs::File::open(path).expect("file not found");

    // let mut markdown_input = String::new();
    // f.read_to_string(&mut markdown_input)
    //     .expect("error reading file");

    // info!("input: \n{}", markdown_input);

    Ok(())
}

fn main() {
    pretty_env_logger::init();
    let cli = Cli::parse();
    clean_folder(&cli.outpath).expect("could not setup output directory");
    process_files(&cli.inpath, &&cli.outpath).expect("process files")
}

use pulldown_cmark::{html, Options, Parser};
use std::fs;
use std::io::{BufWriter, Read};

const DEFAULT_OUTPATH: &str = ".site";

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

fn dist_folder_setup(path: &str) -> std::io::Result<()> {
    // delete if exists and create new -- TODO: if exists
    std::fs::remove_dir_all(path)?;
    fs::create_dir_all(path)?;
    Ok(())
}

// fn copy_static_files(from_path: &str, to_path: &str) {
//     // ignore hidden files and directories (e.g. .dist, .gitignore)
//     // copy media files
// }

fn main() {
    dist_folder_setup(DEFAULT_OUTPATH).expect("could not setup output directory");
    let path = "index.md";
    let mut f = fs::File::open(path).expect("file not found");
    // let mut markdown_input = BufReader::new(f);

    let mut markdown_input = String::new();
    f.read_to_string(&mut markdown_input)
        .expect("error reading file");

    println!("input: {}", markdown_input);
    // Set up options and parser.
    let mut options = Options::empty();
    // Example: Strikethroughs are not part of the CommonMark standard
    // so must be enabled explicitly (TODO: maybe configure?)
    options.insert(Options::ENABLE_STRIKETHROUGH);
    let parser = Parser::new_ext(&markdown_input, options);

    // Write to a file
    let out_file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(".site/test.html")
        .expect("could not open output file");
    let writer = BufWriter::new(out_file);
    html::write_html(writer, parser).expect("unable to write to file");
    println!("HTML file written!");
}

use anyhow::Result;
use clap::{AppSettings, Parser};
use std::fs;
use std::path::Path;
use webgenr::Web;

extern crate pretty_env_logger;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(global_setting(AppSettings::DeriveDisplayOrder))]
struct Cli {
    /// directory path for markdown source files
    #[clap(short, long, value_parser, default_value = "markdown")]
    inpath: String,

    /// destination path for html
    #[clap(short, long, value_parser, default_value = "_website")]
    outpath: String,

    /// directory path for template files
    #[clap(short, long, value_parser, default_value = "templates")]
    templatedir: String,

    #[clap(long, short, action)]
    book: bool,
}

fn clean_folder(path: &str) -> std::io::Result<()> {
    // delete if exists and create new
    if Path::new(path).exists() {
        fs::remove_dir_all(path)?;
    }
    fs::create_dir_all(path)?;
    Ok(())
}

fn process_files(cli: Cli) -> Result<()> {
    println!("processing source files from:\t{}", &cli.inpath);
    let mut web = Web::new(&cli.inpath, &cli.outpath, &cli.templatedir)?;
    if cli.book {
        println!("book!");
    } else {
        clean_folder(&cli.outpath).expect("could not setup output directory");
        let count = web.gen_website()?;
        if count > 0 {
            println!("success! see output files:\t{}", &cli.outpath);
        }
    }
    Ok(())
}

fn main() {
    pretty_env_logger::init();
    let cli = Cli::parse();

    if let Err(e) = process_files(cli) {
        println!("Erorr processing files: {:#?}", e);
    }

}

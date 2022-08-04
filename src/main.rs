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
}

fn clean_folder(path: &str) -> std::io::Result<()> {
    // delete if exists and create new
    if Path::new(path).exists() {
        fs::remove_dir_all(path)?;
    }
    fs::create_dir_all(path)?;
    Ok(())
}

fn process_files(cli: Cli) -> std::io::Result<()> {
    let mut web = Web::new(&cli.inpath, &cli.outpath)?;
    web.gen()?;
    Ok(())
}

fn main() {
    pretty_env_logger::init();
    let cli = Cli::parse();
    clean_folder(&cli.outpath).expect("could not setup output directory");
    match process_files(cli) {
        Ok(_) => println!("success!"),
        Err(e) => println!("Erorr processing files: {:?}", e),
    }
}

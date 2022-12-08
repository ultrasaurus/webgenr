# Simple Static Site Generator (written in Rust)

work-in-progress, experimental

## Install & Run locally

clone this repo, then from within the repo's top-level directory:
```
cargo install --path .
cd ~ && mkdir test && cd test   

webgenr
```

Directories will be created, as needed. 

## Templates

All files ending in `.hbs` in templates directory will use their relative name as template name. For example, the file `templates/some/path/file.hbs` will be registered as `some/path/file`.

Hidden files and tempfile (starts with #) will be ignored.

https://docs.rs/handlebars/latest/handlebars/struct.Handlebars.html#method.register_templates_directory

# Development

run with verbose output
```
RUST_LOG=info cargo run
```

see command-line options:
```
RUST_LOG=info cargo run -- --help
```

## TODOs
- config file (for options avail on command-line)






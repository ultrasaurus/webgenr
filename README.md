# Not ready for anyone to look at

work-in-progress, experimental, might just be for my learning

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

## TBD User Stories

1. I can install webgenr globally and run in any directory

Current behavior (starting inside webgenr directory):
```
cargo install --path .
cd .. && mkdir test && cd test

webgenr
processing source files:	markdown
Erorr processing files: Template "default": No such file or directory (os error 2)
```

workaround (starting from above state):
```
cp -r ../webgenr/templates .
mkdir markdown
```

## Templates

Hidden files and tempfile (starts with #) will be ignored. All registered will use their relative name as template name. For example, when dir_path is templates/ and tpl_extension is .hbs, the file templates/some/path/file.hbs will be registered as some/path/file.

https://docs.rs/handlebars/latest/handlebars/struct.Handlebars.html#method.register_templates_directory







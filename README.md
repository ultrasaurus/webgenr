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

see command-line options:
```
RUST_LOG=info cargo run -- --help
```

see `examples` folder for more usage examples

## TBD
Eventually will move to github issues, just keeping roadmap-y things here
until there are multiple maintainers or additional users.

### Misc TODO
- config file (for options avail on command-line), prolly will wait till
  there's a bit more clarity on options needed. Or if someone wants it,
  they can propose a format with config options, file an issue and link it here.



### TODO - tech debt
-  currently using https://github.com/ultrasaurus/epub-builder.
   At some point, need to figure out if we need to:
    1. hard fork the library we're using "own" the epub making code
    2. find an alternated library, or
    3. if @lisa-henry is going to actively maintain https://github.com/lise-henry/epub-builder
    since need PRs:
        - https://github.com/lise-henry/epub-builder/pull/37
        - https://github.com/lise-henry/epub-builder/pull/34


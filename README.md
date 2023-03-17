# Simple Static Site Generator (written in Rust)

work-in-progress, experimental

demo: https://ultrasaurus.github.io/webgenr/

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
- how to deal with book cover image and title page? I think I like option 2
  - current: now there are special filenames, but feels like wrong approach.
    I've found reserving "special" names can lead to hard to figure out bugs
    for users
        - cover or _cover
        - title or _title
  - option 1: a folder for front-matter or paratext
    (if ebooks have other special pages / annotations)
  - option 2: --cover name --title name, with no argument uses default name
    otherwise book lacks cover image or title page
- option to create book and/or web
  - current: --book creates epub, without it website is generated
  - future: something like --format=web,epub,pdf or something semantically
    equivalent (look into common way for multiple options like that?)
- config file (for options avail on command-line), prolly will wait till
  there's a bit more clarity on options needed. Or if someone wants it,
  they can propose a format with config options, file an issue and link it here.

### TODO - if requested
- file extensons: particular file extensions are hard-coded; however,
  there are common variants not currently supported
  - code writes HTML files with `.html` extension. Alternate `.htm` could
    be future config option
  - code identifies markdown files with `.md` extension. Would be easy to
    also look for `.markdown`

### TODO - tech debt
- need to write some automated tests
- currently using https://github.com/ultrasaurus/epub-builder.
  At some point, need to figure out if we need to:
    1. hard fork the library we're using "own" the epub making code
    2. find an alternated library, or
    3. if @lisa-henry is going to actively maintain https://github.com/lise-henry/epub-builder
    since need PRs:
        - https://github.com/lise-henry/epub-builder/pull/37
        - https://github.com/lise-henry/epub-builder/pull/34


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


## User Stories

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
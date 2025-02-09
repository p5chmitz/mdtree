# mdtree
A simple `tree`-like table-of-contents generator for Markdown documents.

![](assets/mdtree.png)

<hr>

## About
I write a lot of Markdown and I often need high-level views of document contents. This program provides hierarchical views of Markdown headers within a document using [box drawing](https://en.wikipedia.org/wiki/Box_Drawing) components for visual flair. Its like `tree`, but for Markdown headers. The program works on files and directories, recursively looking for Markdown docs from the specified path. If a document skips heading levels, for example if you put an H4 under an H2, the program prints an empty node as `[]` for visual reference.

My website is written with [Astro frontmatter](https://docs.astro.build/en/guides/markdown-content/) so the program will display `title` values if they exist for the document. If no `title` values exist the program prints the name of the document from the file path.

## Install 
Installation is dirt simple if you have the Rust (and more specifically `cargo` and `rustc`) installed. You can install the program by running `cargo install --git https://github.com/p5chmitz/mdtree`. See the [cargo-install](https://doc.rust-lang.org/cargo/commands/cargo-install.html) docs for more details about building and installing binaries from source code.

## Use
You can use the `-h` / `--help` option to see a list of arguments with descriptions. Since there are only two right now its easy to just list them here:
- `-p` / `--path`: Relative path to a file or directory from the current working directory
- `-l` / `--level`: Exclude headings at and above the specified level; E.g. `-l 1` skips H1s, `-l 2` skips H1s and H2s


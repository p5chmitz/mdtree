use clap::Parser;
use std::path::PathBuf; // Imports lib.rs

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Specify the level for the table of contents
    #[arg(
        short,
        long,
        default_value_t = 0,
        help = "Exclude headings at and above the specified level;\n-l 1 skips H1s, -l 2 skips H1s and H2s."
    )]
    level: usize,

    /// Relative path to MD doc or dir
    #[arg(short, long, default_value = ".")]
    path: PathBuf,
}

fn main() {
    let args = Args::parse();
    mdtree::navigator(args.level, &args.path);
}

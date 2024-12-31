
use clap::Parser;
use std::path::PathBuf;

mod process;
mod params;

#[derive(Parser, Debug)]
struct Args {
    #[clap(long, short('n'))]
    dry_run: bool,

    #[clap(long, default_value="256")]
    compress_threshold: u64,

    #[clap(long)]
    sort: Option<params::Sort>,

    files: Vec<PathBuf>,
}



fn main() {
    let args = Args::parse();
    let params = params::ProcessParams {
        dry_run: args.dry_run,
        compress_threshold: args.compress_threshold,
        sort: args.sort,
    };

    for file in args.files {
        if let Err(e) = process::process_file(&file, &params) {
            eprintln!("{}: {}", file.display(), e);
            std::process::exit(1);
        }
    }
}


use clap::Parser;
use std::path::PathBuf;

mod process;

#[derive(Parser, Debug)]
struct Args {
    #[clap(long, default_value="256")]
    compress_threshold: u64,

    files: Vec<PathBuf>,
}

fn main() {
    let args = Args::parse();
    let params = process::ProcessParams {
        compress_threshold: args.compress_threshold
    };

    for file in args.files {
        if let Err(e) = process::process_file(&file, &params) {
            eprintln!("{}: {}", file.display(), e);
            std::process::exit(1);
        }
    }
}

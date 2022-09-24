
use std::fs;

mod args;
use args::*;

mod fsevents;

fn main() {
    // get args
    let args = ClapArgs::parse();
    println!("args: {:?}", args);
    if !validate_args(&args) {
        return;
    }

    // find all archives in fseventsd directory
    let archive_files = fsevents::find_archives(&args.input_path);
    println!("found {} archives", archive_files.len());

    // parse fsevents and save
    archive_files.iter().for_each(|f| {
        _ = fsevents::parse_archive(f);
    });
}

fn validate_args(args: &ClapArgs) -> bool {

    // check input file existence
    if let Err(err) = fs::read_dir(&args.input_path) {
        println!("invalid input path: {}", err);
        return false;
    }

    // check output path dir existence
    if let Err(err) = fs::remove_file(&args.output_path) {
        if err.kind() != std::io::ErrorKind::NotFound {
            println!("failed to remove legacy output: {}", err);
            return false;
        }
    }

    return true;
}
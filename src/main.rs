
mod args;
use args::*;

mod fsevents;

fn main() {
    // get args
    let args = ArgParse::parse();
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

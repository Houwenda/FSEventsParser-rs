
mod args;
use args::*;

mod fsevents;

fn main() {
    // get args
    let args = ArgParse::parse();
    if !validate_args(&args) {
        return;
    }

    // find all archives in fseventsd directory
    let archive_files = fsevents::find_archives(&args.input_path);
    if archive_files.len() == 0 {
        println!("no valid archive found in input directory, existing");
    }
    println!("found {} archives in {}", archive_files.len(), args.input_path);

    // parse fsevents and save
    archive_files.iter().for_each(|f| {
        if let Some(archive) = fsevents::parse_archive(f) {
            println!("---------- {} ----------", archive.filename);
            println!("page count: {}", archive.pages.len());
            archive.pages.iter().for_each(|p| {
                println!("entry count: {}", p.entries.len());
            });
        }
    });
}

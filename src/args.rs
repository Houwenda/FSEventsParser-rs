pub use clap::Parser;
use std::fs;

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about=None)]
pub struct ArgParse {
    #[clap(short, long, value_parser,
        default_value = "/System/Volumes/Data/.fseventsd")]
    pub input_path: String,

    #[clap(short, long, value_parser,
        default_value = "./output.json")]
    pub output_path: String,

    #[clap(short, long, value_enum,
        default_value_t = ArgsOutputFormat::Json)]
    pub format: ArgsOutputFormat,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum ArgsOutputFormat {
    Json,
    Csv,
    Sqlite,
}

pub fn validate_args(args: &ArgParse) -> bool {

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
pub use clap::Parser;

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about=None)]
pub struct ClapArgs {
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
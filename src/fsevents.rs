use std::{fs, io::Read, fmt};

use regex::Regex;
use flate2::read::GzDecoder;

pub fn find_archives(dir: &str) -> Vec<String> {
    let fname_re = Regex::new("^[0-9a-f]{16}$").unwrap();

    if let Ok(dir_result) = fs::read_dir(dir) {
        return dir_result.into_iter()
            .filter_map(|s| { // file name & type
                let ss = s.ok()?;
                match fname_re.is_match(ss.file_name().to_str()?) && 
                    ss.metadata().ok()?.is_file() {
                    true => Some(String::from(ss.path().to_str()?)),
                    false => None
                }
            })
            .collect::<Vec::<String>>();
    }

    vec![] // failed to read dir
}

pub fn parse_archive(file_path: &str) -> Option<Archive> {
    println!("parsing file: {}", file_path);

    // parse from memory
    let parse_result = Archive::new(file_path);

    match parse_result {
        Ok(result) => {
            println!("succeed to parse: {:?}", result);
            Some(result)
        },
        Err(e) => {
            println!("failed to parse: {:?}", e);
            None
        }
    }
}

#[derive(Debug)]
pub struct Archive {
    pub pages: Vec<Page>,

    pub filename: String, 
    pub mtime: std::time::SystemTime, 
    pub ctime: std::time::SystemTime, 
}

impl Archive {
    pub fn new(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // timestamp
        let metadata = fs::metadata(path)?;
        let filename = match std::path::Path::new(path).file_name() {
            Some(s) => match s.to_str() {
                Some(s) => String::from(s),
                None => String::from(""),
            },
            None => String::from(""),
        };

        /*
         * pages
         */
        // uncompress
        let mut buf = Vec::new();
        let fd = fs::File::open(path)?;
        let mut decoder = GzDecoder::new(fd);
        let out_size = decoder.read_to_end(&mut buf)?;
        println!("uncompressed size: {}", out_size);

        // parse all pages
        let mut offset: usize = 0;
        while offset < out_size {
            match Page::new(&buf[offset..]) {
                Ok((page, consumed)) => {
                    println!("succeed to parse page: {:?}", page);
                    offset += consumed;
                }, 
                Err(e) => {
                    println!("encountered error when parsing page,
                         move to next archive: {:?}", e);
                    break;
                }, 
            }
        }

        Ok(Archive {
            pages: vec![], 
            filename, 
            mtime: metadata.modified()?,
            ctime: metadata.created()?, 
        })
    }

    pub fn is_timestamp_credible(&self) -> bool {
        self.mtime != self.ctime
    }
} // impl Archive

#[derive(Debug)]
pub struct PageHeader {

}

#[derive(Debug)]
pub struct Page {
    pub header: PageHeader, 
    pub entries: Vec<Entry>, 
}

impl Page {
    // usize consumed
    pub fn new(mem: &[u8]) -> Result<(Self, usize), Box<dyn std::error::Error>> {
        // TODO: parse from memory
        // find page magic
        // parse length from header
        // parse entries by length

        // let offset = mem.windows(4).position(|window| window == b"2SLD");
        // if offset == None {
        //     return Err(Box::new(ParseError));
        // }
        // let offset = offset?;

        Ok((Page{
            header: PageHeader {}, 
            entries: vec![], 
        }, 0)) // mem len actually consumed
    }
} // impl Page

#[derive(Debug)]
pub struct Entry {

}

#[derive(Debug, Clone)]
pub struct ParseError;
impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "failed to parse")
    }
}
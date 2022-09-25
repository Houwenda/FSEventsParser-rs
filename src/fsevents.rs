use std::{fs, fmt};
use std::io::Read;
use regex::Regex;

use flate2::read::MultiGzDecoder;

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
        // timestamp & filename
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
        let mut decoder = MultiGzDecoder::new(fd);
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
} // impl Archive


#[derive(Debug)]
pub struct Page {
    pub header: PageHeader, 
    pub entries: Vec<Entry>, 
}

impl Page {
    // usize consumed
    pub fn new(mem: &[u8]) -> Result<(Self, usize), Box<dyn std::error::Error>> {

        // find page magic
        let offset = mem.windows(4).position(|window| window == b"2SLD");
        let mut offset = offset.unwrap_or(usize::MAX);
        if offset > mem.len() {
            return Err(Box::new(ParseError::NoPageFound));
        }

        // parse header
        let header = PageHeader::new(&mem[offset..])?;
        if matches!(header.version, Version::V1) {
            return Err(Box::new(ParseError::UnsupportedVersion));
        }

        // parse entries by length
        println!("parsing entries in page, size: {}", header.stream_size);
        offset += PageHeader::len(); // skip header
        let mut entries = vec![];
        while offset < header.stream_size as usize {
            // TODO
            break;
            
        }

        Ok((Page{
            header, 
            entries, 
        }, offset)) // mem len actually consumed
    }
} // impl Page


#[derive(Debug)]
pub struct PageHeader {
    version: Version,
    stream_size: u32,
}
#[derive(Debug)]
pub enum Version { Unkown, V1, V2, }
impl PageHeader {
    pub fn new(mem: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        // validate len
        if mem.len() < Self::len()+1 {
            return Err(Box::new(ParseError::InvalidHeader));
        }

        // parse version
        let mut version = Version::Unkown;
        if mem.starts_with(b"1SLD") {
            version = Version::V1;
        } else if mem.starts_with(b"2SLD") {
            version = Version::V2
        }
        if matches!(version, Version::Unkown) {
            return Err(Box::new(ParseError::InvalidHeader));
        }

        // parse len
        let len = u32::from_le_bytes(mem[8..12].try_into()?);
        if len as usize > mem.len() {
            return Err(Box::new(ParseError::InvalidHeader));
        }

        Ok(PageHeader { 
            version, 
            stream_size: len, 
        })
    }

    pub fn len() -> usize {
        12
    }
} // impl PageHeader

#[derive(Debug)]
pub struct Entry {

}

#[derive(Debug)]
pub enum ParseError {
    NoPageFound,
    InvalidHeader, 
    UnsupportedVersion, 
    IoError(std::io::Error), 
}
impl std::error::Error for ParseError {

}
impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::NoPageFound => {
                write!(f, "no page found")
            }, 
            ParseError::InvalidHeader => {
                write!(f, "invalid header")
            }
            ParseError::UnsupportedVersion => {
                write!(f, "page version not supported")
            }
            ParseError::IoError(e) => {
                write!(f, "{}", e)
            }
        }
    }
}
impl From<std::io::Error> for ParseError {
    fn from(err: std::io::Error) -> Self {
        ParseError::IoError(err)
    }
}
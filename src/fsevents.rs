use std::{fs, fmt};
use std::io::Read;
use regex::Regex;

use flate2::read::MultiGzDecoder;
use bitflags::bitflags;

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
    // parse from compressed file
    let parse_result = Archive::new(file_path);
    match parse_result {
        Ok(archive) => {
            if archive.pages.len() == 0 {
                println!("archive contains no pages");
                return None;
            }

            // println!("parse archive {} succeeded, page count: {}", 
            //     archive.filename, archive.pages.len());
            Some(archive)
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
        decoder.read_to_end(&mut buf)?;
        // println!("uncompressed size: {} {}", filename, buf.len());

        // parse all pages
        let mut pages = vec![];
        let mut offset: usize = 0;
        while offset < buf.len() {
            match Page::new(&buf[offset..]) {
                Ok((page, consumed)) => {
                    offset += consumed;
                    // println!("parse page succeeded: {:?}, entry count: {}, page consumed: {}, stream left: {}", 
                    //     page.header, page.entries.len(), consumed, buf.len() - offset);
                    pages.push(page);
                }, 
                Err(e) => {
                    println!("encountered error when parsing page,
                         move to next archive: {:?}", e);
                    break;
                }, 
            }
        }

        Ok(Archive {
            pages,  
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
        // println!("parsing entries in page, size: {}", header.stream_size);
        offset += PageHeader::len(); // skip header
        let mut entries = vec![];
        while offset < header.stream_size as usize && 
              offset < mem.len()-1 {
            if let Some(path_len) = mem[offset..]
                                                .iter()
                                                .position(|&r| r == 0) {
                /*
                 * | full path | end with 0x00
                 * | event id | 8 bytes
                 * | event flags | 4 bytes
                 * | node id | 8 bytes
                 */
                                                
                // path can be empty? offset == end_offset
                let end_offset = offset + path_len;
                if end_offset + 20 >= mem.len() { // other attributes
                    println!("invalid record for path, stop parsing page: {:?}", &mem[offset..end_offset+1]);
                    break;
                }

                let full_path = String::from_utf8_lossy(&mem[offset..end_offset+1]).into_owned();
                offset = end_offset + 1; // skip 0x00
                // println!("found path: {}", full_path);

                // event id
                let event_id = u64::from_le_bytes(mem[offset..offset+8].try_into()?);
                offset += 8;
                // println!("event id: {}", event_id);

                // flags
                let flags = u32::from_le_bytes(mem[offset..offset+4].try_into()?);
                offset += 4;
                // println!("event flags: {}", flags);

                // skip node id
                offset += 8;

                // new entry generated
                entries.push(Entry { 
                    full_path, 
                    event_id, 
                    flags: EventFlag::from_bits_truncate(flags), 
                });

            } else { // no 0x00 any more
                offset = mem.len();
                break;
            }
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
    pub full_path: String, 
    pub event_id: u64,
    pub flags: EventFlag, 
}

bitflags! {
    pub struct EventFlag : u32 {
        const FSE_NONE = 0x00000000;

        const FSE_CREATE_FILE = 0x00000001;  
        const FSE_DELETE = 0x00000002; 
        const FSE_STAT_CHANGED = 0x00000004; 
        const FSE_RENAME = 0x00000008; 
        const FSE_CONTENT_MODIFIED = 0x00000010; 
        const FSE_EXCHANGE = 0x00000020; 
        const FSE_FINDER_INFO_CHANGED = 0x00000040; 
        const FSE_CREATE_DIR = 0x00000080;
        const FSE_CHOWN = 0x00000100;
        const FSE_XATTR_MODIFIED = 0x00000200;
        const FSE_XATTR_REMOVED = 0x00000400;
        const FSE_DOCID_CREATED = 0x00000800;
        const FSE_DOCID_CHANGED = 0x00001000;
        const FSE_UNMOUNT_PENDING = 0x00002000;
        const FSE_CLONE = 0x00004000;
        const FSE_MODE_CLONE = 0x00010000;
        const FSE_TRUNCATED_PATH = 0x00020000;
        const FSE_REMOTE_DIR_EVENT = 0x00040000;
        const FSE_MODE_LAST_HLINK = 0x00080000;
        const FSE_MODE_HLINK = 0x00100000;
        
        const FSE_IS_SYMLINK = 0x00400000;
        const FSE_IS_FILE = 0x00800000;
        const FSE_IS_DIR = 0x01000000;
        const FSE_MOUNT = 0x02000000;
        const FSE_UNMOUNT = 0x04000000;

        const FSE_END_TRANSACTION = 0x20000000;
    }
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
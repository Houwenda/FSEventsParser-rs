
use crate::fsevents::Archive;
pub trait Registry {
    fn export_archive(&mut self, archive: &Archive) -> bool;
}

pub mod json_registry {

use std::fs;
use std::io::Write;
use std::time::UNIX_EPOCH;

use serde::{Serialize,};
use serde_json;

use crate::fsevents::Archive;
use crate::registry::Registry;

pub struct JsonRegistry {
    pub written_count: usize, 

    fd: fs::File, 
}

#[derive(Serialize)]
struct JsonRecord {
    path: String, // record path
    id: u64, // record id
    flags: String, // flag description

    create_ts: u64,
    modiy_ts: u64, 
    source: String, // source archive file name
}

impl JsonRegistry {
    pub fn new(path: &str) -> Result<JsonRegistry, std::io::Error> {
        Ok(JsonRegistry {
            written_count: 0, 
            fd: fs::File::create(path)?,
        })
    }
} // impl JsonResgistry

impl Registry for JsonRegistry {
   
    fn export_archive(&mut self, archive: &Archive) -> bool {
        for page in archive.pages.iter() {
            for entry in page.entries.iter() {

                let json_record = JsonRecord {
                    path: String::from(&entry.full_path), 
                    id: entry.event_id,
                    flags: format!("{:?}", entry.flags),

                    create_ts: archive.ctime.duration_since(UNIX_EPOCH)
                                .unwrap_or_default().as_secs(),
                    modiy_ts: archive.mtime.duration_since(UNIX_EPOCH)
                                .unwrap_or_default().as_secs(),
                    source: String::from(&archive.filename),
                };
                
                if let Ok(j) = serde_json::to_string(&json_record) {
                    if let Err(e) = self.fd.write_all(&j.as_bytes()) {
                        println!("failed to write json record: {}", e);
                        break;
                    }
                    _ = self.fd.write(b"\n");
                }

            }
        }
        false
    }
} // impl Registry for JsonRegistry

} // mod json_registry


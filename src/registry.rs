
use crate::fsevents::Archive;
pub trait Registry {
    fn export_archive(&mut self, archive: &Archive) -> bool;
}

pub mod json {

use std::fs;
use std::io::Write;
use std::time::UNIX_EPOCH;

use serde::Serialize;
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
    pub fn new(path: &str) -> Result<Box<JsonRegistry>, std::io::Error> {
        Ok(Box::new(JsonRegistry {
            written_count: 0, 
            fd: fs::File::create(path)?,
        }))
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
        
        true
    }
} // impl Registry for JsonRegistry

} // mod json_registry


pub mod csv {

use std::fs;
use std::time::UNIX_EPOCH;

use csv;

use crate::fsevents::Archive;
use crate::registry::Registry;

pub struct CsvRegistry {
    pub written_count: usize, 

    writer: csv::Writer<fs::File>, 
}

impl CsvRegistry {
    pub fn new(path: &str) -> Result<Box<CsvRegistry>, std::io::Error> {
        Ok(Box::new(CsvRegistry {
            written_count: 0, 
            writer: csv::Writer::from_path(path)?,
        }))
    }
} // impl JsonResgistry

impl Registry for CsvRegistry {
   
    fn export_archive(&mut self, archive: &Archive) -> bool {
        for page in archive.pages.iter() {
            for entry in page.entries.iter() {

                let csv_record = (
                    &entry.full_path,
                    &entry.event_id,
                    format!("{:?}", entry.flags),
                    archive.ctime.duration_since(UNIX_EPOCH)
                        .unwrap_or_default().as_secs(),
                    archive.mtime.duration_since(UNIX_EPOCH)
                        .unwrap_or_default().as_secs(),
                    &archive.filename,
                );

                if let Err(e) = self.writer.serialize(csv_record) {
                    println!("failed to serialize record to csv: {}", e);
                    continue;
                }
                if let Err(e) = self.writer.flush() {
                    println!("failed to write record to file: {}", e);
                    break;
                }

            }
        }
        
        true
    }


} // impl Registry for CsvRegistry

} // mod csv


pub mod sqlite {

use std::time::UNIX_EPOCH;

use rusqlite;

use crate::fsevents::Archive;
use crate::registry::Registry;

pub struct SqliteRegistry {
    pub written_count: usize, 

    conn: rusqlite::Connection, 
}

impl SqliteRegistry {
    pub fn new(path: &str) -> Result<Box<SqliteRegistry>, rusqlite::Error> {
        let conn = rusqlite::Connection::open(path)?;
        conn.execute(
            "CREATE TABLE record (
                path TEXT, 
                id TEXT NOT NULL, 
                flags TEXT, 
                create_ts INTEGER,
                modify_ts INTEGER,
                source TEXT NOT NULL
            )", 
            (), 
        )?;

        Ok(Box::new(SqliteRegistry {
            written_count: 0, 
            conn,
        }))
    }
} // impl SqliteRegistry

impl Registry for SqliteRegistry {

    fn export_archive(&mut self, archive: &Archive) -> bool {
        // start transaction
        let txn = match self.conn.transaction() {
            Ok(t) => t, 
            Err(e) => {
                println!("failed to create transaction: {}", e);
                return false;
            }
        };

        for page in archive.pages.iter() {
            for entry in page.entries.iter() {

                if let Err(e) = txn.execute(
                    "INSERT INTO record (
                        path, id, flags, create_ts, modify_ts, source) 
                        VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    (&entry.full_path, 
                    &entry.event_id.to_string(), 
                    format!("{:?}", entry.flags), 
                    archive.ctime.duration_since(UNIX_EPOCH)
                        .unwrap_or_default().as_secs(),
                    archive.mtime.duration_since(UNIX_EPOCH)
                        .unwrap_or_default().as_secs(),
                    &archive.filename, 
                    ),
                ) {
                    println!("failed to insert record: {}", e);
                    continue;
                }

            }
        }
        
        // end transaction
        if let Err(e) = txn.commit() {
            println!("failed to commit transaction: {}", e);
            return false;
        }

        true
    }

} // impl Registry for SqliteRegistry


} // mod sqlite
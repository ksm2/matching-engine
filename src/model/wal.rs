use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{bail, Result};

use super::Order;

#[derive(Debug)]
pub struct WriteAheadLog {
    file: BufWriter<File>,
}

impl WriteAheadLog {
    pub fn new(dir: &String) -> Result<Self, Box<std::io::Error>> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        let path = Path::new(dir).join(timestamp.as_millis().to_string() + ".wal");
        let file = OpenOptions::new().append(true).create(true).open(&path)?;
        let file = BufWriter::new(file);

        Ok(WriteAheadLog { file })
    }

    pub fn append_order(&mut self, order: &Order) -> Result<(), anyhow::Error> {
        // Serialization
        let entry = match serde_json::to_string(order) {
            Ok(e) => e,
            Err(_) => bail!("Failed to parse"),
        };

        // Write on file
        writeln!(self.file, "{}", entry)?;
        self.file.flush()?;

        Ok(())
    }
}

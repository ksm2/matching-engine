use std::fs::{File, OpenOptions, read_dir};
use std::io::{BufWriter, Write, BufReader, BufRead};
use std::path::{PathBuf};

use anyhow::{bail, Result};

use super::Order;

#[derive(Debug)]
pub struct WriteAheadLog {
    file: BufWriter<File>,
    path: PathBuf
}

impl WriteAheadLog {
    pub fn new(dir: &str) -> Result<Self> {
        let path_dir: PathBuf  = dir.into();
        let path_file = path_dir.join("write_ahead_log.wal");
        let file = OpenOptions::new().append(true).create(true).open(&path_file)?;
        let file = BufWriter::new(file);

        Ok(WriteAheadLog { file, path: path_dir})
    }

    pub fn append_order(&mut self, order: &Order) -> Result<()> {
        // Serialization
        let entry : String= match serde_json::to_string(order) {
            Ok(e) => e,
            Err(_) => bail!("Failed to parse"),
        };

        // Write on file
        writeln!(self.file, "{}", entry)?;
        self.file.flush()?;

        Ok(())
    }

    fn get_files_path(&mut self ) -> Vec<PathBuf> {
        let mut files = Vec::new();
        for file in read_dir(self.path.as_path()).unwrap() {
            let path = file.unwrap().path();
            files.push(path);
        }

        files.sort();
        files
    }


    pub fn read_file(&mut self) -> anyhow::Result<Vec<Order>> {
        let files = self.get_files_path();
        if files.is_empty() { return Ok(Vec::new()); }

        let head = &files[0];
        let file = OpenOptions::new().read(true).open(head).expect("Error reading the file");
        let file = BufReader::new(file);

        let mut orders = Vec::new();

        for line in file.lines() {
            let json = line?;
            let order: Order = serde_json::from_str(&json)?;
            orders.push(order);
        }

        Ok(orders)
    }
}

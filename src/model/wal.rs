use std::fs::{create_dir_all, read_dir, File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

use anyhow::Result;
use log::warn;

use super::Order;

#[derive(Debug)]
pub struct WriteAheadLog {
    file: BufWriter<File>,
    path: PathBuf,
}

impl WriteAheadLog {
    pub fn new(path_dir: &Path) -> Result<Self> {
        let path_file = path_dir.join("write_ahead_log.wal");
        create_dir_all(path_dir)?;
        let file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(&path_file)?;
        let file = BufWriter::new(file);

        Ok(WriteAheadLog {
            file,
            path: path_dir.into(),
        })
    }

    pub fn append_order(&mut self, order: &Order) -> Result<()> {
        // Serialization
        let entry = serde_json::to_string(order)?;

        // Write on file
        writeln!(self.file, "{}", entry)?;
        self.file.flush()?;

        Ok(())
    }

    pub fn read_orders(&mut self) -> Vec<Order> {
        let files = self.get_files_path();
        if files.is_empty() {
            return Vec::new();
        }

        files
            .into_iter()
            .filter_map(|file| Self::read_log_file(&file).ok())
            .flatten()
            .collect()
    }

    fn get_files_path(&mut self) -> Vec<PathBuf> {
        let mut files: Vec<_> = read_dir(&self.path)
            .into_iter()
            .flatten()
            .filter_map(|file| {
                file.map_err(|err| warn!("Failed to read WAL: {}", err))
                    .ok()
            })
            .map(|file| file.path())
            .collect();

        files.sort();
        files
    }

    fn read_log_file(path: &Path) -> Result<Vec<Order>> {
        let file = OpenOptions::new().read(true).open(path)?;
        let file = BufReader::new(file);

        Ok(file
            .lines()
            .enumerate()
            .filter_map(|(index, line)| {
                line.map_err(anyhow::Error::from)
                    .and_then(|json| Self::parse_order(&json))
                    .map_err(|err| {
                        warn!(
                            "Failed to read line {} from WAL {}: {}",
                            index + 1,
                            path.to_string_lossy(),
                            err
                        )
                    })
                    .ok()
            })
            .collect())
    }

    fn parse_order(json: &str) -> Result<Order> {
        let order = serde_json::from_str(json)?;
        Ok(order)
    }
}

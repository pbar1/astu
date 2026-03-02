use astu::db::DuckDb;
use camino::Utf8PathBuf;

use crate::args::OutputFormat;

#[derive(Debug, Clone)]
pub struct Runtime {
    data_dir: Utf8PathBuf,
    db: DuckDb,
    output: OutputFormat,
}

impl Runtime {
    pub fn new(data_dir: Utf8PathBuf, db: DuckDb, output: OutputFormat) -> Self {
        Self {
            data_dir,
            db,
            output,
        }
    }

    pub fn data_dir(&self) -> &Utf8PathBuf {
        &self.data_dir
    }

    pub fn db(&self) -> &DuckDb {
        &self.db
    }

    pub const fn output(&self) -> OutputFormat {
        self.output
    }
}

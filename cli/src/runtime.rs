use astu::db::DuckDb;
use camino::Utf8PathBuf;

#[derive(Debug, Clone)]
pub struct Runtime {
    data_dir: Utf8PathBuf,
    db: DuckDb,
}

impl Runtime {
    pub fn new(data_dir: Utf8PathBuf, db: DuckDb) -> Self {
        Self { data_dir, db }
    }

    pub fn data_dir(&self) -> &Utf8PathBuf {
        &self.data_dir
    }

    pub fn db(&self) -> &DuckDb {
        &self.db
    }
}

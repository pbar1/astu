//! Wrapper for getting default data storage paths.
//!
//! Follows these rules:
//!
//! - XDG base directory (all platforms if environment variable is set)
//! - Platform-specific well known directory (see [`dirs`])
//! - Home directory
//! - Current directory

use std::env;
use std::path::PathBuf;

const XDG_CONFIG_HOME: &str = "XDG_CONFIG_HOME";
const XDG_DATA_HOME: &str = "XDG_DATA_HOME";

/// Gets the config directory. Creates it if it does not exist.
///
/// # Panics
///
/// If none of the rules succeeds.
pub fn config_dir(name: &str) -> PathBuf {
    let dir = if let Ok(path) = env::var(XDG_CONFIG_HOME) {
        PathBuf::from(path).join(name)
    } else if let Some(path) = dirs::config_dir() {
        path.join(name)
    } else if let Some(path) = dirs::home_dir() {
        path.join(format!(".{name}"))
    } else if let Ok(path) = env::current_dir() {
        path.join(format!(".{name}"))
    } else {
        panic!("all sources for config dir failed");
    };
    std::fs::create_dir_all(&dir).expect("unable to ensure config dir exists");
    dir
}

/// Gets the data directory. Creates it if it does not exist.
///
/// # Panics
///
/// If none of the rules succeeds.
pub fn data_dir(name: &str) -> PathBuf {
    let dir = if let Ok(path) = env::var(XDG_DATA_HOME) {
        PathBuf::from(path).join(name)
    } else if let Some(path) = dirs::data_dir() {
        path.join(name)
    } else if let Some(path) = dirs::home_dir() {
        path.join(format!(".{name}"))
    } else if let Ok(path) = env::current_dir() {
        path.join(format!(".{name}"))
    } else {
        panic!("all sources for data dir failed");
    };
    std::fs::create_dir_all(&dir).expect("unable to ensure data dir exists");
    dir
}

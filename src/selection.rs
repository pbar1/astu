pub mod fzf;
pub mod skim;

use anyhow::Result;

pub trait Selector {
    fn filter(&self, source: Vec<u8>, preview: Option<&str>) -> Result<String>;
}

/// Dynamically chose a [`Selector`] implementation based on the user's system.
/// Specifically, prefer [`FzfSelector`](self::fzf::FzfSelector) if `fzf` exists
/// on `PATH`, otherwise fallback to [`SkimSelector`](self::skim::SkimSelector).
pub fn auto() -> Box<dyn Selector> {
    if which::which("fzf").is_ok() {
        Box::<self::fzf::FzfSelector>::default()
    } else {
        Box::<self::skim::SkimSelector>::default()
    }
}

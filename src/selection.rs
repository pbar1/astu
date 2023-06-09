pub(crate) mod fzf;
pub(crate) mod skim;

use anyhow::Result;

pub(crate) trait Selector {
    fn filter(&self, source: Vec<u8>, preview: Option<&str>) -> Result<String>;
}

/// Dynamically chose a [`Selector`] implementation based on the user's system.
/// Specifically, prefer [`FzfSelector`](self::fzf::FzfSelector) if `fzf` exists
/// on `PATH`, otherwise fallback to [`SkimSelector`](self::skim::SkimSelector).
pub(crate) fn auto() -> Box<dyn Selector> {
    if which::which("fzf").is_ok() {
        return Box::new(self::fzf::FzfSelector::default());
    }

    Box::new(self::skim::SkimSelector::default())
}

pub(crate) mod fzf;
pub(crate) mod skim;

use anyhow::Result;

use self::fzf::FzfSelector;

pub(crate) trait Selector {
    fn filter(&self, source: Vec<u8>, preview: Option<&str>) -> Result<String>;
}

/// Dynamically chose a `Selector` implementation based on the user's system.
/// Specifically, prefer `FzfSelector` if `fzf` exists on `PATH`, otherwise
/// fallback to `SkimSelector`.
pub(crate) fn auto() -> Box<dyn Selector> {
    // FIXME: Implement actual checking and fallback
    Box::new(FzfSelector::default())
}

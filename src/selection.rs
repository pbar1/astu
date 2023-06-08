use std::io::Cursor;

use anyhow::Context;
use anyhow::Result;
use skim::prelude::*;

pub fn preview_select(
    source: impl AsRef<[u8]> + Send + 'static,
    preview: Option<&str>,
) -> Result<String> {
    let options = SkimOptionsBuilder::default().preview(preview).build()?;

    let item_reader = SkimItemReader::default();

    let items = item_reader.of_bufread(Cursor::new(source));

    let output = Skim::run_with(&options, Some(items)).context("Failed to run Skim selection")?;

    let sel = output
        .selected_items
        .first()
        .context("Selection was empty")?
        .output()
        .to_string();

    Ok(sel)
}

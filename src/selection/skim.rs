use std::io::Cursor;

use anyhow::Context;
use anyhow::Result;
use skim::prelude::Skim;
use skim::prelude::SkimItemReader;
use skim::prelude::SkimOptionsBuilder;

use super::Selector;

#[derive(Default)]
pub(crate) struct SkimSelector {
    item_reader: SkimItemReader,
}

impl Selector for SkimSelector {
    fn filter(&self, source: Vec<u8>, preview: Option<&str>) -> Result<String> {
        let options = SkimOptionsBuilder::default().preview(preview).build()?;

        let items = self.item_reader.of_bufread(Cursor::new(source));

        let output =
            Skim::run_with(&options, Some(items)).context("Failed to run Skim selection")?;

        let sel = output
            .selected_items
            .first()
            .context("Selection was empty")?
            .output()
            .to_string();

        Ok(sel)
    }
}

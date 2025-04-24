use std::str::FromStr;

use fluent_uri::Iri;

fn main() -> anyhow::Result<()> {
    let iri = Iri::from_str("ip://127.0.0.1:4-90")?;

    dbg!(iri);

    Ok(())
}

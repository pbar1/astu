use anyhow::Result;
use anyhow::{anyhow, bail};

pub fn require_confirm(confirm: Option<usize>, target_count: usize) -> Result<()> {
    if let Some(confirm) = confirm {
        if confirm != target_count {
            bail!("--confirm expected {target_count}, got {confirm}");
        }
        return Ok(());
    }

    if !std::io::IsTerminal::is_terminal(&std::io::stdin()) {
        bail!("--confirm={target_count} is required in non-interactive mode");
    }

    eprintln!("Plan affects {target_count} targets.");
    eprint!("Enter target count to confirm: ");
    let mut answer = String::new();
    std::io::stdin().read_line(&mut answer)?;
    let parsed = answer
        .trim()
        .parse::<usize>()
        .map_err(|_| anyhow!("invalid confirmation input"))?;
    if parsed != target_count {
        bail!("confirmation failed: expected {target_count}, got {parsed}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::require_confirm;

    #[test]
    fn confirm_count_must_match() {
        let err = require_confirm(Some(1), 2).expect_err("must fail");
        assert!(err.to_string().contains("expected 2"));
    }

    #[test]
    fn confirm_count_match_passes() {
        require_confirm(Some(2), 2).expect("must pass");
    }
}

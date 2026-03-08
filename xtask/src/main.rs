use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command as ProcessCommand;

use clap::CommandFactory;
use clap::Parser;
use clap::Subcommand;
use clap_complete::Shell;
use clap_complete::generate_to;
use eyre::WrapErr;
use eyre::ensure;
use eyre::eyre;

fn main() -> eyre::Result<()> {
    match Xtask::parse().command {
        XtaskCommand::Man { out_dir } => gen_man_pages(out_dir),
        XtaskCommand::Completions { out_dir, shell } => gen_completions(&out_dir, shell),
        XtaskCommand::Book { view } => build_book(view),
    }
}

#[derive(Debug, Parser)]
#[command(name = "xtask")]
struct Xtask {
    #[command(subcommand)]
    command: XtaskCommand,
}

#[derive(Debug, Subcommand)]
enum XtaskCommand {
    /// Generate clap man pages for the astu CLI
    Man {
        /// Destination directory for generated man pages
        #[arg(long, default_value = "target/man/man1")]
        out_dir: PathBuf,
    },
    /// Generate shell completions for the astu CLI
    Completions {
        /// Destination directory for generated completion files
        #[arg(long, default_value = "target/completions")]
        out_dir: PathBuf,
        /// Shells to generate. If omitted, all supported shells are generated.
        #[arg(long, value_enum)]
        shell: Vec<Shell>,
    },
    /// Build or serve the project mdBook
    Book {
        /// Serve the book and open it in a browser instead of building once
        #[arg(long)]
        view: bool,
    },
}

fn gen_man_pages(out_dir: PathBuf) -> eyre::Result<()> {
    let out_dir = normalize_man_output_dir(resolve_workspace_path(out_dir));
    let man_root = man_output_root(&out_dir);

    clean_dir(&man_root, "man dir")?;
    fs::create_dir_all(&out_dir)
        .wrap_err_with(|| format!("failed to create output directory {}", out_dir.display()))?;

    let mut root = astu_cli::Cli::command();
    root = root.name("astu");

    write_command_and_subcommands(&root, "astu", &out_dir)
}

fn gen_completions(out_dir: &Path, shells: Vec<Shell>) -> eyre::Result<()> {
    let out_dir = resolve_workspace_path(out_dir.to_path_buf());
    clean_dir(&out_dir, "completion dir")?;
    fs::create_dir_all(&out_dir)
        .wrap_err_with(|| format!("failed to create output directory {}", out_dir.display()))?;

    let shells = if shells.is_empty() {
        vec![
            Shell::Bash,
            Shell::Elvish,
            Shell::Fish,
            Shell::PowerShell,
            Shell::Zsh,
        ]
    } else {
        shells
    };

    for shell in shells {
        let shell_name = format!("{shell:?}");
        let mut command = astu_cli::Cli::command();
        command = command.name("astu");
        generate_to(shell, &mut command, "astu", &out_dir).wrap_err_with(|| {
            format!(
                "failed to generate completion for {} in {}",
                shell_name,
                out_dir.display()
            )
        })?;
    }

    Ok(())
}

fn build_book(view: bool) -> eyre::Result<()> {
    let workspace_root = workspace_root()?;
    let mut command = ProcessCommand::new("mdbook");
    let command_str = if view {
        command.args(["serve", "--open", "book"]);
        "mdbook serve --open book"
    } else {
        command.args(["build", "book"]);
        "mdbook build book"
    };
    command.current_dir(&workspace_root);

    let status = command
        .status()
        .wrap_err_with(|| format!("failed to run `{command_str}`"))?;
    ensure!(status.success(), "`{command_str}` exited with {status}");
    Ok(())
}

fn resolve_workspace_path(path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        return path;
    }

    workspace_root().map_or_else(|_| path.clone(), |root| root.join(&path))
}

fn workspace_root() -> eyre::Result<PathBuf> {
    let xtask_manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    xtask_manifest_dir
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| eyre!("failed to determine workspace root from xtask manifest path"))
}

fn normalize_man_output_dir(out_dir: PathBuf) -> PathBuf {
    let dir_name = out_dir
        .file_name()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or_default();

    if dir_name == "man" {
        return out_dir.join("man1");
    }

    out_dir
}

fn man_output_root(out_dir: &Path) -> PathBuf {
    let is_man1_dir = out_dir
        .file_name()
        .and_then(std::ffi::OsStr::to_str)
        .is_some_and(|name| name == "man1");
    if is_man1_dir {
        return out_dir
            .parent()
            .map_or_else(|| out_dir.to_path_buf(), Path::to_path_buf);
    }

    out_dir.to_path_buf()
}

fn clean_dir(path: &Path, label: &str) -> eyre::Result<()> {
    match fs::remove_dir_all(path) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(err) => {
            Err(err).wrap_err_with(|| format!("failed to remove {label} {}", path.display()))
        }
    }
}

fn write_command_and_subcommands(
    command: &clap::Command,
    name: &str,
    out_dir: &Path,
) -> eyre::Result<()> {
    let mut page_command = command.clone();
    let page_name: &'static str = Box::leak(name.to_owned().into_boxed_str());
    page_command = page_command
        .name(page_name)
        .display_name(name.to_owned())
        .bin_name(name.to_owned());

    let mut rendered = Vec::new();
    clap_mangen::Man::new(page_command)
        .render(&mut rendered)
        .wrap_err_with(|| format!("failed rendering man page for '{name}'"))?;

    let output_file = out_dir.join(format!("{name}.1"));
    fs::write(&output_file, rendered).wrap_err_with(|| {
        format!(
            "failed writing man page '{}' to {}",
            name,
            output_file.display()
        )
    })?;

    for subcommand in command.get_subcommands() {
        let sub_name = format!("{name}-{}", subcommand.get_name());
        write_command_and_subcommands(subcommand, &sub_name, out_dir)?;
    }

    Ok(())
}

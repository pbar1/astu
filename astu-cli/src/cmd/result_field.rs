use clap::ValueEnum;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ResultField {
    Stdout,
    Stderr,
    Exitcode,
    Error,
}

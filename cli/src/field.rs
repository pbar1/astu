use astu::db::DbField;
use clap::ValueEnum;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ResultFieldArg {
    Stdout,
    Stderr,
    Exitcode,
    Error,
}

impl ResultFieldArg {
    pub const fn into_db(self) -> DbField {
        match self {
            Self::Stdout => DbField::Stdout,
            Self::Stderr => DbField::Stderr,
            Self::Exitcode => DbField::Exitcode,
            Self::Error => DbField::Error,
        }
    }

    pub const fn freq_title(self) -> &'static str {
        match self {
            Self::Error => "error-freq",
            _ => self.output_title(),
        }
    }

    pub const fn output_title(self) -> &'static str {
        match self {
            Self::Stdout => "stdout",
            Self::Stderr => "stderr",
            Self::Exitcode => "exitcode",
            Self::Error => "error",
        }
    }
}

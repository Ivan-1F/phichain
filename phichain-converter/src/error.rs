use rust_i18n::t;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConvertError {
    NoSuchFile(PathBuf),
    ExpectedFile(PathBuf),
    UnableToInferFormat,
    Io(#[from] std::io::Error),
    Json(#[from] serde_json::Error),
    OfficialInput(#[from] phichain_format::official::OfficialInputError),
    OfficialOutput(#[from] phichain_format::official::OfficialOutputError),
    RpeInput(#[from] phichain_format::rpe::RpeInputError),
}

impl std::fmt::Display for ConvertError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConvertError::NoSuchFile(path) => {
                write!(f, "{}", t!("cli.error.no_such_file", path = path.display()))
            }
            ConvertError::ExpectedFile(path) => {
                write!(
                    f,
                    "{}",
                    t!("cli.error.expected_file", path = path.display())
                )
            }
            ConvertError::UnableToInferFormat => {
                write!(f, "{}", t!("cli.error.unable_to_infer_format"))
            }
            ConvertError::Io(e) => write!(f, "{e}"),
            ConvertError::Json(e) => write!(f, "{e}"),
            ConvertError::OfficialInput(e) => write!(f, "{e}"),
            ConvertError::OfficialOutput(e) => write!(f, "{e}"),
            ConvertError::RpeInput(e) => write!(f, "{e}"),
        }
    }
}

/// Extract value from `Result<T, Infallible>`.
pub fn unwrap_infallible<T>(result: Result<T, std::convert::Infallible>) -> T {
    match result {
        Ok(v) => v,
        Err(e) => match e {},
    }
}

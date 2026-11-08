use thiserror::Error;

#[derive(Debug, Error)]
pub enum OfficialInputError {
    #[error("expected at least one line")]
    NoLine,
    #[error("unsupported formatVersion, expected 1 or 3, got {0}")]
    UnsupportedFormatVersion(u32),
}

#[derive(Debug, Error)]
pub enum OfficialOutputError {
    #[error("BPM list is empty")]
    EmptyBpmList,
}

use phichain_chart::event::LineEventKind;
use phichain_compiler::helpers::EventSequenceError;
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
    #[error("event sequence error in line '{line_name}' for event kind {event_kind:?}: {source}")]
    EventSequenceError {
        line_name: String,
        event_kind: LineEventKind,
        source: EventSequenceError,
    },
}

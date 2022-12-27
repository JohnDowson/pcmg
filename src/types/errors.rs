extern crate thiserror;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum InstrumentLoadingError {
    #[error("No libraries could be loaded")]
    NoInstruments,
    #[error("No libraries could be loaded")]
    InvalidNoteString,
}

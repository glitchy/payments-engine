use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("AccountError: {:?}", .0)]
    AccountError(&'static str),
    #[error("CSV error: {:?}", .0)]
    Csv(#[from] csv::Error),
    #[error("IoError: {:?}", .0)]
    Io(#[from] std::io::Error),
    #[error("TransactionError: {:?}", .0)]
    TransactionError(&'static str),
}

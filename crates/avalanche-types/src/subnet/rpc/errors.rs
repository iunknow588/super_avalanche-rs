//! Custom database errors and helpers.
use std::io;

use tonic::Status;

/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/database#ErrClosed>
#[derive(Copy, Clone, Debug)]
pub enum Error {
    DatabaseClosed = 1, // 0 is reserved for grpc unspecified.
    NotFound,
    HeightIndexedVMNotImplemented,
    IndexIncomplete,
    StateSyncableVMNotImplemented,
}

impl Error {
    /// Returns the string representation of the error.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match *self {
            Self::DatabaseClosed => "database closed",
            Self::NotFound => "not found",
            Self::HeightIndexedVMNotImplemented => {
                "vm does not implement HeightIndexedChainVM interface"
            }
            Self::IndexIncomplete => "query failed because height index is incomplete",
            Self::StateSyncableVMNotImplemented => {
                "vm does not implement StateSyncableVM interface"
            }
        }
    }

    /// Converts the error to an integer code.
    #[must_use]
    pub const fn to_i32(&self) -> i32 {
        match self {
            Self::DatabaseClosed => 1,
            Self::NotFound => 2,
            Self::HeightIndexedVMNotImplemented => 3,
            Self::IndexIncomplete => 4,
            Self::StateSyncableVMNotImplemented => 5,
        }
    }

    /// Returns coresponding `io::Error`.
    #[must_use]
    pub fn to_err(&self) -> io::Error {
        match *self {
            Self::DatabaseClosed => {
                io::Error::new(io::ErrorKind::Other, Self::DatabaseClosed.as_str())
            }
            Self::NotFound => io::Error::new(io::ErrorKind::NotFound, Self::NotFound.as_str()),
            Self::HeightIndexedVMNotImplemented => io::Error::new(
                io::ErrorKind::Other,
                Self::HeightIndexedVMNotImplemented.as_str(),
            ),
            Self::IndexIncomplete => {
                io::Error::new(io::ErrorKind::Other, Self::IndexIncomplete.as_str())
            }
            Self::StateSyncableVMNotImplemented => io::Error::new(
                io::ErrorKind::Other,
                Self::StateSyncableVMNotImplemented.as_str(),
            ),
        }
    }
}

/// Converts an integer error code to a Result.
///
/// # Errors
///
/// Returns an error if the error code corresponds to a known error type.
///
/// # Panics
///
/// Panics if the error code is not recognized.
pub fn from_i32(err: i32) -> io::Result<()> {
    match err {
        0 => Ok(()),
        1 => Err(Error::DatabaseClosed.to_err()),
        2 => Err(Error::NotFound.to_err()),
        3 => Err(Error::HeightIndexedVMNotImplemented.to_err()),
        4 => Err(Error::IndexIncomplete.to_err()),
        5 => Err(Error::StateSyncableVMNotImplemented.to_err()),
        _ => panic!("invalid error type"),
    }
}

/// Accepts an error and returns a corruption error if the original error is not "database closed"
/// or "not found".
#[must_use]
pub fn is_corruptible(error: &io::Error) -> bool {
    match error {
        e if e.kind() == io::ErrorKind::NotFound => false,
        e if e.to_string() == Error::DatabaseClosed.as_str() => false,
        _ => true,
    }
}

/// Returns true if the `io::Error` is `ErrorKind::NotFound` and contains a string "not found".
#[must_use]
pub fn is_not_found(error: &io::Error) -> bool {
    error.kind() == io::ErrorKind::NotFound && error.to_string() == Error::NotFound.as_str()
}

/// Returns an `io::Error` with `ErrorKind::Other` from a string.
#[must_use]
pub fn from_string(message: String) -> io::Error {
    io::Error::new(io::ErrorKind::Other, message)
}

/// Returns a common database error from a tonic Status.
#[must_use]
pub fn from_status(status: &Status) -> io::Error {
    match status.message() {
        m if m.contains("database closed") => Error::DatabaseClosed.to_err(),
        m if m.contains("not found") => Error::NotFound.to_err(),
        _ => io::Error::new(io::ErrorKind::Other, status.message()),
    }
}

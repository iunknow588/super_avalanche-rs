//! Custom error types used in avalanche-types.
use std::{cell::RefCell, fmt, rc::Rc};

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

/// Backing errors for all consensus operations.
#[derive(Clone, Debug, Error)]
pub enum Error {
    #[error("failed API (message: {message:?}, retryable: {retryable:?})")]
    API { message: String, retryable: bool },
    #[error("failed for other reasons (message: {message:?}, retryable: {retryable:?})")]
    Other { message: String, retryable: bool },
}

impl Error {
    #[inline]
    #[must_use]
    pub fn message(&self) -> String {
        match self {
            Error::API { message, .. } | Error::Other { message, .. } => message.clone(),
        }
    }

    #[inline]
    #[must_use]
    pub fn retryable(&self) -> bool {
        match self {
            Error::API { retryable, .. } | Error::Other { retryable, .. } => *retryable,
        }
    }
}

#[derive(Debug)]
pub struct Errors {
    d: Rc<RefCell<Vec<Error>>>,
}

impl Errors {
    pub fn new() -> Self {
        Self {
            d: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn errored(&self) -> bool {
        !self.d.borrow().is_empty()
    }

    pub fn add(&self, e: Error) {
        self.d.borrow_mut().push(e);
    }
}

impl Default for Errors {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Errors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut errs: Vec<String> = Vec::new();
        for e in self.d.borrow().iter() {
            errs.push(e.message());
        }
        write!(f, "{}", errs.join(", "))
    }
}

//! Custom error types used in avalanche-types.
use std::{cell::RefCell, fmt, num::TryFromIntError, rc::Rc};

pub type Result<T> = std::result::Result<T, Error>;

/// Backing errors for all consensus operations.
#[derive(Clone, Debug)]
pub enum Error {
    /// `GetUtxosResult` 结果为 None
    UnexpectedNoneGetUtxosResult,
    /// Utxos from `GetUtxosResult` 结果为 None
    UnexpectedNoneUtxosFromGetUtxosResult,
    /// 通用 None 错误
    UnexpectedNone(String),
    /// 整数转换错误
    IntConversion(String),
    /// API 错误
    API { message: String, retryable: bool },
    /// 其他错误
    Other { message: String, retryable: bool },
}

impl Error {
    #[inline]
    #[must_use]
    pub fn message(&self) -> String {
        match self {
            Self::API { message, .. } | Self::Other { message, .. } => message.clone(),
            Self::UnexpectedNoneGetUtxosResult => "GetUtxosResult is None".to_string(),
            Self::UnexpectedNoneUtxosFromGetUtxosResult => {
                "Utxos from GetUtxosResult is None".to_string()
            }
            Self::UnexpectedNone(msg) => format!("Unexpected None: {msg}"),
            Self::IntConversion(msg) => format!("Integer conversion error: {msg}"),
        }
    }

    #[inline]
    #[must_use]
    pub const fn retryable(&self) -> bool {
        match self {
            Self::API { retryable, .. } | Self::Other { retryable, .. } => *retryable,
            _ => false,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message())
    }
}

#[derive(Debug)]
pub struct AvalancheErrors {
    /// Collection of errors
    d: Rc<RefCell<Vec<Error>>>,
}

impl AvalancheErrors {
    #[must_use]
    pub fn new() -> Self {
        Self {
            d: Rc::new(RefCell::new(Vec::new())),
        }
    }

    #[must_use]
    pub fn errored(&self) -> bool {
        !self.d.borrow().is_empty()
    }

    pub fn add(&self, e: Error) {
        self.d.borrow_mut().push(e);
    }
}

impl Default for AvalancheErrors {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for AvalancheErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut errs: Vec<String> = Vec::new();
        for e in self.d.borrow().iter() {
            errs.push(e.message());
        }
        write!(f, "{}", errs.join(", "))
    }
}

impl From<TryFromIntError> for Error {
    fn from(err: TryFromIntError) -> Self {
        Self::IntConversion(err.to_string())
    }
}

/// gRPC 工具模块。
pub mod grpc;

use std::{
    io::{Error, Result},
    net::{SocketAddr, UdpSocket},
};

/// Returns a localhost address with next available port.
///
/// # Panics
/// 本函数在绑定端口或获取本地地址失败时会 panic。
///
/// # 返回值
/// 返回一个可用的本地 `SocketAddr`。
#[must_use]
pub fn new_socket_addr() -> SocketAddr {
    UdpSocket::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
}

/// Persists first \[`io::Error`\] collected.
#[derive(Debug)]
pub struct Errors {
    /// Stores the collected error if any.
    err: Option<Error>,
}

impl Default for Errors {
    fn default() -> Self {
        Self::new()
    }
}

impl Errors {
    /// Creates a new collector instance.
    #[must_use]
    pub const fn new() -> Self {
        Self { err: None }
    }

    /// Persists the error if no error currently exists.
    pub fn add(&mut self, error: &Error) {
        if self.err.is_none() {
            self.err = Some(Error::new(error.kind(), error.to_string()));
        }
    }

    /// Returns an `io::Error` if collected.
    /// Returns error if collected.
    ///
    /// # Errors
    /// Returns an error if one has been collected.
    pub fn err(&self) -> Result<()> {
        if let Some(e) = &self.err {
            return Err(Error::new(e.kind(), e.to_string()));
        }
        Ok(())
    }

    /// Returns true an error has been collected.
    /// Returns true if an error has been collected.
    #[must_use]
    pub const fn is_some(&self) -> bool {
        self.err.is_some()
    }
}

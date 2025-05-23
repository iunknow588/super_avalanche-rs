//! Common JSON-RPC types.
// Copied from <https://github.com/gakonst/ethers-rs/blob/master/ethers-providers/src/transports/common.rs>.
// Remove once is <https://github.com/gakonst/ethers-rs/issues/1997> resolved.
use std::fmt;

use ethers_core::types::U256;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

#[derive(Serialize, Deserialize, Debug, Clone, Error)]
/// A JSON-RPC 2.0 error
pub struct JsonRpcError {
    /// The error code
    pub code: i64,
    /// The error message
    pub message: String,
    /// Additional data
    pub data: Option<Value>,
}

impl fmt::Display for JsonRpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "(code: {}, message: {}, data: {:?})",
            self.code, self.message, self.data
        )
    }
}

/// 检查类型是否为零大小类型（Zero-Sized Type）。
const fn is_zst<T>(_t: &T) -> bool {
    std::mem::size_of::<T>() == 0
}

/// A JSON-RPC request
#[derive(Serialize, Deserialize, Debug)]
pub struct Request<'a, T> {
    /// 请求ID
    pub id: u64,
    /// JSON-RPC 版本
    pub jsonrpc: &'a str,
    /// 方法名称
    pub method: &'a str,
    /// 参数
    #[serde(skip_serializing_if = "is_zst")]
    pub params: T,
}

/// A JSON-RPC Notifcation
#[derive(Serialize, Deserialize, Debug)]
pub struct Notification<R> {
    /// JSON-RPC 版本
    jsonrpc: String,
    /// 方法名称
    method: String,
    /// 订阅参数
    pub params: Subscription<R>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Subscription<R> {
    pub subscription: U256,
    pub result: R,
}

impl<'a, T> Request<'a, T> {
    /// Creates a new JSON RPC request
    #[must_use]
    pub const fn new(id: u64, method: &'a str, params: T) -> Self {
        Self {
            id,
            jsonrpc: "2.0",
            method,
            params,
        }
    }
}

/// JSON-RPC 响应
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Response<T> {
    /// 请求ID
    pub(crate) id: u64,
    /// JSON-RPC 版本
    jsonrpc: String,
    /// 响应数据
    #[serde(flatten)]
    pub data: ResponseData<T>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum ResponseData<R> {
    Error { error: JsonRpcError },
    Success { result: R },
}

impl<R> ResponseData<R> {
    /// Consume response and return value
    ///
    /// # Errors
    ///
    /// Returns an error if the response contains an error.
    pub fn into_result(self) -> Result<R, JsonRpcError> {
        match self {
            Self::Success { result } => Ok(result),
            Self::Error { error } => Err(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deser_response() {
        let response: Response<u64> =
            serde_json::from_str(r#"{"jsonrpc": "2.0", "result": 19, "id": 1}"#).unwrap();
        assert_eq!(response.id, 1);
        assert_eq!(response.data.into_result().unwrap(), 19);
    }

    #[test]
    fn ser_request() {
        let request: Request<()> = Request::new(300, "method_name", ());
        assert_eq!(
            &serde_json::to_string(&request).unwrap(),
            r#"{"id":300,"jsonrpc":"2.0","method":"method_name"}"#
        );

        let request: Request<u32> = Request::new(300, "method_name", 1);
        assert_eq!(
            &serde_json::to_string(&request).unwrap(),
            r#"{"id":300,"jsonrpc":"2.0","method":"method_name","params":1}"#
        );
    }
}

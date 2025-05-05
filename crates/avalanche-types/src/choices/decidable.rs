//! The `Decidable` trait.
use crate::errors::Result;
use crate::{choices::status::Status, ids::Id};

/// Represents an element that can be decided.
/// `Decidable` objects are transactions, blocks, or vertices.
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/snow/choices#Decidable>.
pub trait Decidable {
    /// Returns the `Id` of this block's parent.
    /// Returns the ID of this block's parent.
    fn id(&self) -> Id;

    /// Returns the current status.
    fn status(&self) -> Status;

    /// Accepts this element.
    /// TODO: use <https://docs.rs/tokio-context/latest/tokio_context>?
    ///
    /// # Errors
    ///
    /// 当接受操作失败时返回错误。
    fn accept(&mut self) -> Result<()>;
    /// Rejects this element.
    /// TODO: use <https://docs.rs/tokio-context/latest/tokio_context>?
    ///
    /// # Errors
    ///
    /// 当拒绝操作失败时返回错误。
    fn reject(&mut self) -> Result<()>;
}

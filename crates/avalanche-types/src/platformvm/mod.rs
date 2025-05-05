//! Avalanche platformvm utilities.
pub mod txs;

use crate::ids;

/// 返回平台链的链ID。
///
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/utils/constants#pkg-variables>
#[must_use]
pub const fn chain_id() -> ids::Id {
    ids::Id::empty()
}

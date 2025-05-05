//! Avalanche Rust SDK: Types and building blocks to assist with creating a custom `subnet` VM.
//!
//! Example VM's created with SDK:
//! * Simple Rust VM: [TimestampVM](https://github.com/ava-labs/timestampvm-rs)
//! * Complex Rust VM: [SpacesVM](https://github.com/ava-labs/spacesvm-rs)

pub mod config;
pub mod rpc;

use std::io::{self, Error, ErrorKind};

use crate::ids;

/// ref. <https://docs.avax.network/build/avalanchego-apis/platform/#platformgetblockchains>
///
/// Convert a given Vm name to an encoded Vm Id.
///
/// # Errors
/// 如果解析 VM 名称失败，返回 `io::Error`。
/// 将虚拟机名称转换为ID
///
/// # Arguments
/// * `s` - 虚拟机名称
///
/// # Errors
/// 当名称转换失败时返回错误
pub fn vm_name_to_id(s: impl AsRef<[u8]>) -> io::Result<ids::Id> {
    let d = s.as_ref();
    if d.len() > ids::LEN {
        return Err(Error::new(
            ErrorKind::Other,
            format!("non-hashed name must be <= 32 bytes, found {}", d.len()),
        ));
    }
    Ok(ids::Id::from_slice(d))
}

/// `RUST_LOG=debug` cargo test --package avalanche-types --lib -- `subnet::test_vm_name_to_id` --exact --show-output
#[test]
fn test_vm_name_to_id() {
    let id = vm_name_to_id("timestampvm").unwrap();
    println!("{id}");
    assert_eq!(
        id.to_string(),
        "tGas3T58KzdjcJ2iKSyiYsWiqYctRXaPTqBCA11BqEkNg8kPc"
    );
}

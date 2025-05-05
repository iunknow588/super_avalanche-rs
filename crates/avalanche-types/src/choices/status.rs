//! `Status` enum that represents the possible statuses of an consensus operation.
use crate::{errors, packer::Packer};
use bytes::Bytes;
use serde::{Deserialize, Serialize};

/// Defines possible status values.
///
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/snow/choices#Status>
#[derive(
    Deserialize,
    Serialize,
    std::clone::Clone,
    std::cmp::Eq,
    std::cmp::Ord,
    std::cmp::PartialEq,
    std::cmp::PartialOrd,
    std::fmt::Debug,
    std::hash::Hash,
)]
pub enum Status {
    /// The operation is known but has not been decided yet.
    Processing,

    /// The operation is already rejected and will never be accepted.
    Rejected,

    /// The operation has been accepted.
    Accepted,

    /// The status is unknown.
    Unknown(String),
}

impl Default for Status {
    fn default() -> Self {
        Self::Unknown("default".to_owned())
    }
}

impl std::convert::From<&str> for Status {
    fn from(s: &str) -> Self {
        match s {
            "Processing" => Self::Processing,
            "Rejected" => Self::Rejected,
            "Accepted" => Self::Accepted,
            other => Self::Unknown(other.to_owned()),
        }
    }
}

impl std::str::FromStr for Status {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Self::from(s))
    }
}

/// ref. <https://doc.rust-lang.org/std/string/trait.ToString.html>
/// ref. <https://doc.rust-lang.org/std/fmt/trait.Display.html>
/// Use `Self.to_string()` to directly invoke this.
impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Status {
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Processing => "Processing",
            Self::Rejected => "Rejected",
            Self::Accepted => "Accepted",
            Self::Unknown(s) => s.as_ref(),
        }
    }

    /// Returns all the `&str` values of the enum members.
    #[must_use]
    pub const fn values() -> &'static [&'static str] {
        &["Processing", "Rejected", "Accepted"]
    }

    /// Returns "true" if the status has been decided.
    #[must_use]
    pub const fn decided(&self) -> bool {
        matches!(self, Self::Rejected | Self::Accepted)
    }

    /// Returns "true" if the status has been set.
    #[must_use]
    pub const fn fetched(&self) -> bool {
        match self {
            Self::Processing => true,
            _ => self.decided(),
        }
    }

    /// Returns the bytes representation of this status.
    ///
    /// # Errors
    ///
    /// 当序列化失败时返回错误。
    pub fn bytes(&self) -> errors::Result<Bytes> {
        let iota = match self {
            Self::Processing => 1_u32,
            Self::Rejected => 2_u32,
            Self::Accepted => 3_u32,
            Self::Unknown(_) => 0_u32,
        };

        let packer = Packer::new(4, 4);
        packer.pack_u32(iota)?;
        Ok(packer.take_bytes())
    }

    /// Returns the u32 primitive representation of this status.
    #[must_use]
    pub const fn to_u32(&self) -> u32 {
        match self {
            Self::Processing => 1,
            Self::Rejected => 2,
            Self::Accepted => 3,
            Self::Unknown(_) => 0,
        }
    }

    /// Returns the i32 primitive representation of this status.
    #[must_use]
    pub const fn to_i32(&self) -> i32 {
        match self {
            Self::Processing => 1,
            Self::Rejected => 2,
            Self::Accepted => 3,
            Self::Unknown(_) => 0,
        }
    }

    /// Returns native endian value from a slice if u8s.
    ///
    /// # Panics
    ///
    /// 当 `bytes` 长度大于 4 时会 panic。
    #[must_use]
    pub fn u32_from_slice(bytes: &[u8]) -> u32 {
        assert!(bytes.len() <= 4);
        let d: [u8; 4] = bytes.try_into().unwrap();
        u32::from_ne_bytes(d)
    }
}

impl AsRef<str> for Status {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

/// `RUST_LOG=debug` cargo test --package avalanche-types --lib --
/// `choices::status::test_bytes` --exact --show-output
#[test]
fn test_bytes() {
    let sb = Status::Processing.bytes().unwrap().to_vec();
    assert!(cmp_manager::eq_vectors(&sb, &[0x00, 0x00, 0x00, 0x01]));

    let sb = Status::Rejected.bytes().unwrap().to_vec();
    assert!(cmp_manager::eq_vectors(&sb, &[0x00, 0x00, 0x00, 0x02]));

    let sb = Status::Accepted.bytes().unwrap().to_vec();
    assert!(cmp_manager::eq_vectors(&sb, &[0x00, 0x00, 0x00, 0x03]));

    let sb = Status::Unknown("()".to_string()).bytes().unwrap().to_vec();
    assert!(cmp_manager::eq_vectors(&sb, &[0x00, 0x00, 0x00, 0x00]));
}

/// `RUST_LOG=debug` cargo test --package avalanche-types --lib --
/// `choices::status::test_to_u32` --exact --show-output
#[test]
fn test_to_u32() {
    assert_eq!(Status::Unknown("hello".to_string()).to_u32(), 0);
    assert_eq!(Status::Processing.to_u32(), 1);
    assert_eq!(Status::Rejected.to_u32(), 2);
    assert_eq!(Status::Accepted.to_u32(), 3);
}

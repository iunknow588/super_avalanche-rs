//! Node short ID used in `AvalancheGo`.
use std::{
    cmp::Ordering,
    fmt,
    hash::{Hash, Hasher},
    io::{self, Error, ErrorKind},
    str::FromStr,
};

use crate::{formatting, hash, key::secp256k1};
use lazy_static::lazy_static;
use serde::{self, Deserialize, Deserializer, Serialize, Serializer};
use zerocopy::{AsBytes, FromBytes, FromZeroes, Unaligned};

pub const LEN: usize = 20;

lazy_static! {
    static ref EMPTY: Vec<u8> = vec![0; LEN];
}

/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/ids#ShortID>
/// ref. <https://docs.rs/zerocopy/latest/zerocopy/trait.AsBytes.html#safety>
#[derive(Debug, Clone, Eq, AsBytes, FromZeroes, FromBytes, Unaligned)]
#[repr(transparent)]
pub struct Id([u8; LEN]);

impl Default for Id {
    fn default() -> Self {
        Self::empty()
    }
}

impl Id {
    #[must_use]
    pub const fn empty() -> Self {
        Self([0; LEN])
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        (*self) == Self::empty()
    }

    /// 从字节切片创建一个短ID。
    ///
    /// # Panics
    ///
    /// 如果切片长度大于 `LEN`，则会 panic。
    #[must_use]
    pub fn from_slice(d: &[u8]) -> Self {
        assert!(d.len() <= LEN);
        let mut d: Vec<u8> = Vec::from(d);
        if d.len() < LEN {
            d.resize(LEN, 0);
        }
        let d: [u8; LEN] = d.try_into().unwrap();
        Self(d)
    }

    /// "hashing.PubkeyBytesToAddress"
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/utils/hashing#PubkeyBytesToAddress>
    ///
    /// # Errors
    ///
    /// 如果哈希计算失败，则返回错误。
    pub fn from_public_key_bytes<S>(pub_key_bytes: S) -> io::Result<Self>
    where
        S: AsRef<[u8]>,
    {
        let hashed = hash::sha256_ripemd160(pub_key_bytes)?;

        // "ids.Id.String"
        // ref. https://pkg.go.dev/github.com/ava-labs/avalanchego/ids#Id.String
        let encoded = formatting::encode_cb58_with_checksum_string(&hashed);
        Self::from_str(&encoded)
    }
}

impl AsRef<[u8]> for Id {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

/// ref. <https://doc.rust-lang.org/std/string/trait.ToString.html>
/// ref. <https://doc.rust-lang.org/std/fmt/trait.Display.html>
/// Use `Self.to_string()` to directly invoke this.
impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = formatting::encode_cb58_with_checksum_string(&self.0);
        write!(f, "{s}")
    }
}

/// ref. <https://doc.rust-lang.org/std/str/trait.FromStr.html>
impl FromStr for Id {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // trim in case it's parsed from list
        let decoded = formatting::decode_cb58_with_checksum(s.trim()).map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("failed decode_cb58_with_checksum '{e}'"),
            )
        })?;
        Ok(Self::from_slice(&decoded))
    }
}

/// Custom serializer.
/// ref. <https://serde.rs/impl-serialize.html>
impl Serialize for Id {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

/// Custom deserializer.
/// ref. <https://serde.rs/impl-deserialize.html>
impl<'de> Deserialize<'de> for Id {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        let ss: Vec<&str> = s.split('-').collect();
        if ss.len() == 1 {
            return Self::from_str(&s).map_err(serde::de::Error::custom);
        }

        let addr = ss[1];
        let (_, short_bytes) = secp256k1::address::avax_address_to_short_bytes("", addr)
            .map_err(serde::de::Error::custom)?;
        Ok(Self::from_slice(&short_bytes))
    }
}

/// 从字符串反序列化为 `Id`。
///
/// # Errors
///
/// 如果反序列化失败，则返回错误。
fn fmt_id<'de, D>(deserializer: D) -> Result<Id, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Id::from_str(&s).map_err(serde::de::Error::custom)
}

/// Custom deserializer.
/// ref. <https://serde.rs/impl-deserialize.html>
///
/// # Errors
///
/// 如果反序列化失败，则返回错误。
pub fn deserialize_id<'de, D>(deserializer: D) -> Result<Option<Id>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct Wrapper(#[serde(deserialize_with = "fmt_id")] Id);
    let v = Option::deserialize(deserializer)?;
    Ok(v.map(|Wrapper(a)| a))
}

/// Custom deserializer.
/// Use [`serde(deserialize_with = "ids::must_deserialize_id")`] to serde without
/// derive. ref. <https://serde.rs/impl-deserialize.html>
///
/// # Errors
///
/// 如果反序列化失败或结果为空，则返回错误。
pub fn must_deserialize_id<'de, D>(deserializer: D) -> Result<Id, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct Wrapper(#[serde(deserialize_with = "fmt_id")] Id);
    let v = Option::deserialize(deserializer)?;
    v.map(|Wrapper(a)| a).map_or_else(
        || {
            Err(serde::de::Error::custom(
                "empty short::Id from deserialization",
            ))
        },
        Ok,
    )
}

/// Custom deserializer.
/// ref. <https://serde.rs/impl-deserialize.html>
///
/// # Errors
///
/// 如果反序列化失败，则返回错误。
pub fn deserialize_ids<'de, D>(deserializer: D) -> Result<Option<Vec<Id>>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct Wrapper(#[serde(deserialize_with = "fmt_ids")] Vec<Id>);
    let v = Option::deserialize(deserializer)?;
    Ok(v.map(|Wrapper(a)| a))
}

/// Custom deserializer.
/// Use [`serde(deserialize_with = "short::must_deserialize_ids")`] to serde with
/// derive. ref. <https://serde.rs/impl-deserialize.html>
///
/// # Errors
///
/// 如果反序列化失败或结果为空，则返回错误。
pub fn must_deserialize_ids<'de, D>(deserializer: D) -> Result<Vec<Id>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct Wrapper(#[serde(deserialize_with = "fmt_ids")] Vec<Id>);
    let v = Option::deserialize(deserializer)?;
    v.map(|Wrapper(a)| a).map_or_else(
        || Err(serde::de::Error::custom("empty Ids from deserialization")),
        Ok,
    )
}

/// 从字符串数组反序列化为 `Vec<Id>`。
///
/// # Errors
///
/// 如果反序列化失败，则返回错误。
fn fmt_ids<'de, D>(deserializer: D) -> Result<Vec<Id>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    type Strings = Vec<String>;
    let ss = Strings::deserialize(deserializer)?;
    match ss
        .iter()
        .map(|x| x.parse::<Id>())
        .collect::<Result<Vec<Id>, Error>>()
    {
        Ok(x) => Ok(x),
        Err(e) => Err(serde::de::Error::custom(format!(
            "failed to deserialize Ids {e}"
        ))),
    }
}

/// `RUST_LOG=debug` cargo test --package avalanche-types --lib --
/// `ids::short::test_serialize` --exact --show-output
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
struct Data {
    /// The short ID.
    id: Id,
    /// Optional short ID.
    id2: Option<Id>,
    /// List of short IDs.
    ids: Vec<Id>,
}

#[test]
fn test_serialize() {
    let id = Id::from_slice(&[
        0x3d, 0x0a, 0xd1, 0x2b, 0x8e, 0xe8, 0x92, 0x8e, 0xdf, 0x24, 0x8c, 0xa9, 0x1c, 0xa5, 0x56,
        0x00, 0xfb, 0x38, 0x3f, 0x07,
    ]);
    assert_eq!(id.to_string(), "6ZmBHXTqjknJoZtXbnJ6x7af863rXDTwx");

    let d = Data {
        id: id.clone(),
        id2: Some(id.clone()),
        ids: vec![id.clone(), id.clone(), id.clone(), id.clone(), id],
    };

    let yaml_encoded = serde_yaml::to_string(&d).unwrap();
    assert!(yaml_encoded.contains("6ZmBHXTqjknJoZtXbnJ6x7af863rXDTwx"));
    let yaml_decoded = serde_yaml::from_str(&yaml_encoded).unwrap();
    assert_eq!(d, yaml_decoded);

    let json_encoded = serde_json::to_string(&d).unwrap();
    assert!(json_encoded.contains("6ZmBHXTqjknJoZtXbnJ6x7af863rXDTwx"));
    let json_decoded = serde_json::from_str(&json_encoded).unwrap();
    assert_eq!(d, json_decoded);

    let pk = crate::key::secp256k1::private_key::Key::generate().unwrap();
    let pubkey = pk.to_public_key();
    let short_addr = pubkey.to_short_id().unwrap();
    let p_addr = pubkey.to_hrp_address(1, "P").unwrap();

    let d1: Data = serde_json::from_str(&format!(
        "{{\"id\": \"{short_addr}\", \"ids\": [\"{p_addr}\", \"{p_addr}\"]}}"
    ))
    .unwrap();
    let d2 = Data {
        id: short_addr.clone(),
        id2: None,
        ids: vec![short_addr.clone(), short_addr],
    };
    assert_eq!(d1, d2);

    let id = Id::from_str("6Y3kysjF9jnHnYkdS9yGAuoHyae2eNmeV").unwrap();
    println!("{id}");

    let d: Data = serde_json::from_str(
        "{\"id\": \"6Y3kysjF9jnHnYkdS9yGAuoHyae2eNmeV\", \"ids\": \
         [\"6Y3kysjF9jnHnYkdS9yGAuoHyae2eNmeV\", \"6Y3kysjF9jnHnYkdS9yGAuoHyae2eNmeV\"]}",
    )
    .unwrap();
    println!("{d:?}");
}

/// `RUST_LOG=debug` cargo test --package avalanche-types --lib --
/// `ids::short::test_id` --exact --show-output
#[test]
fn test_id() {
    let id = Id::from_slice(&[
        0x3d, 0x0a, 0xd1, 0x2b, 0x8e, 0xe8, 0x92, 0x8e, 0xdf, 0x24, 0x8c, 0xa9, 0x1c, 0xa5, 0x56,
        0x00, 0xfb, 0x38, 0x3f, 0x07,
    ]);
    assert_eq!(id.to_string(), "6ZmBHXTqjknJoZtXbnJ6x7af863rXDTwx");
    let id_from_str = Id::from_str("6ZmBHXTqjknJoZtXbnJ6x7af863rXDTwx").unwrap();
    assert_eq!(id, id_from_str);
}

impl Ord for Id {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&(other.0))
    }
}

impl PartialOrd for Id {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Id {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

/// ref. <https://rust-lang.github.io/rust-clippy/master/index.html#derive_hash_xor_eq>
impl Hash for Id {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

#[derive(Debug, Eq, Clone)]
pub struct Ids(Vec<Id>);

impl Ids {
    #[must_use]
    pub fn new(ids: &[Id]) -> Self {
        Self(Vec::from(ids))
    }
}

impl From<Vec<Id>> for Ids {
    fn from(ids: Vec<Id>) -> Self {
        Self::new(&ids)
    }
}

impl Ord for Ids {
    fn cmp(&self, other: &Self) -> Ordering {
        // packer encodes the array length first
        // so if the lengths differ, the ordering is decided
        let l1 = self.0.len();
        let l2 = other.0.len();
        l1.cmp(&l2) // returns when lengths are not Equal
            .then_with(
                || self.0.cmp(&other.0), // if lengths are Equal, compare the ids
            )
    }
}

impl PartialOrd for Ids {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Ids {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

/// Tests for sorting and comparing short IDs.
#[cfg(test)]
mod sort_tests {
    use super::*;

    /// Helper function to create a short ID from a byte value.
    fn create_id(first_byte: u8) -> Id {
        Id::from_slice(&[first_byte])
    }

    /// Helper function to create a set of short IDs.
    fn create_id_set(first_bytes: &[u8]) -> Ids {
        let ids = first_bytes
            .iter()
            .map(|&b| create_id(b))
            .collect::<Vec<_>>();
        Ids(ids)
    }

    /// Test equality of short IDs with different byte lengths but same value.
    #[test]
    fn test_id_equality_different_lengths() {
        let id1 = Id::from_slice(&[0x01, 0x00, 0x00, 0x00]);
        let id2 = Id::from_slice(&[0x01, 0x00, 0x00, 0x00, 0x00]);
        assert_eq!(id1, id2);
    }

    /// Test comparison of short IDs with different values.
    #[test]
    fn test_id_comparison() {
        let id1 = Id::from_slice(&[0x01, 0x00, 0x00, 0x00, 0x00]);
        let id2 = Id::from_slice(&[0x02]);

        assert!(id1 < id2);

        let id1 = Id::from_slice(&[0x02, 0x00, 0x00, 0x00, 0x00]);
        let id2 = Id::from_slice(&[0x01, 0x00, 0x00, 0x00, 0x00]);

        assert!(id1 > id2);
    }

    /// Test equality of ID sets with same elements.
    #[test]
    fn test_id_set_equality() {
        let id_set1 = create_id_set(&[1, 2, 3]);
        let id_set2 = create_id_set(&[1, 2, 3]);

        assert_eq!(id_set1, id_set2);
    }

    /// Test comparison of ID sets with different lengths.
    #[test]
    fn test_id_set_length_comparison() {
        // Set with 3 elements vs set with 4 elements
        let id_set1 = create_id_set(&[5, 6, 7]);
        let id_set2 = create_id_set(&[1, 2, 3, 4]);

        assert!(id_set1 < id_set2);
    }

    /// Test comparison of ID sets where longer set has larger elements.
    #[test]
    fn test_id_set_mixed_comparison() {
        // Set with 4 elements vs set with 3 elements with larger values
        let id_set1 = create_id_set(&[1, 2, 3, 4]);
        let id_set2 = create_id_set(&[9, 9, 9]);

        assert!(id_set1 > id_set2);
    }

    /// Test comparison of ID sets with same length but different elements.
    #[test]
    fn test_id_set_element_comparison() {
        // Same length (3), but different elements
        let id_set1 = create_id_set(&[1, 2, 3]);
        let id_set2 = create_id_set(&[1, 2, 5]);

        assert!(id_set1 < id_set2);
    }

    /// Test sorting of short IDs.
    #[test]
    fn test_id_sorting() {
        let mut ids = vec![create_id(3), create_id(2), create_id(1)];
        ids.sort();

        let expected = vec![create_id(1), create_id(2), create_id(3)];
        assert_eq!(ids, expected);
    }
}

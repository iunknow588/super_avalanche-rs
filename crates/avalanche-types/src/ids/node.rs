//! Node ID utilities.
use std::{
    cmp::Ordering,
    collections::HashSet,
    fmt,
    hash::{Hash, Hasher},
    io::{self, Error, ErrorKind},
    path::Path,
    str::FromStr,
};

use lazy_static::lazy_static;
use serde::{self, de::Visitor, Deserialize, Deserializer, Serialize, Serializer};
use zerocopy::{AsBytes, FromBytes, FromZeroes, Unaligned};

use crate::{formatting, hash, ids::short};

pub const LEN: usize = 20;
pub const ENCODE_PREFIX: &str = "NodeID-";

lazy_static! {
    static ref EMPTY: Vec<u8> = vec![0; LEN];
}

/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/ids#ShortID>
/// ref. <https://docs.rs/zerocopy/latest/zerocopy/trait.AsBytes.html#safety>
#[derive(Debug, Copy, Clone, Eq, AsBytes, FromZeroes, FromBytes, Unaligned)]
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

    #[must_use]
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    /// 从字节切片创建一个节点ID。
    ///
    /// # Panics
    ///
    /// 如果切片长度不等于 `LEN`，则会 panic。
    #[must_use]
    pub fn from_slice(d: &[u8]) -> Self {
        assert_eq!(d.len(), LEN);
        let d: [u8; LEN] = d.try_into().unwrap();
        Self(d)
    }

    /// Loads a node ID from the PEM-encoded X509 certificate.
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/node#Node.Initialize>
    ///
    /// # Errors
    ///
    /// 如果无法加载或解析证书文件，则返回错误。
    pub fn from_cert_pem_file(cert_file_path: &str) -> io::Result<Self> {
        log::info!("loading node ID from certificate {cert_file_path}");
        let pub_key_der = cert_manager::x509::load_pem_cert_to_der(cert_file_path)?;

        // "ids.ToShortID(hashing.PubkeyBytesToAddress(StakingTLSCert.Leaf.Raw))"
        // ref. https://pkg.go.dev/github.com/ava-labs/avalanchego/node#Node.Initialize
        Self::from_cert_der_bytes(pub_key_der)
    }

    /// Encodes the DER-encoded certificate bytes to a node ID.
    /// It applies "sha256" and "ripemd160" on "Certificate.Leaf.Raw".
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/utils/hashing#PubkeyBytesToAddress>
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/ids#ToShortID>
    ///
    /// # Errors
    ///
    /// 如果无法对证书字节进行哈希处理，则返回错误。
    pub fn from_cert_der_bytes<S>(cert_bytes: S) -> io::Result<Self>
    where
        S: AsRef<[u8]>,
    {
        let short_address = hash::sha256_ripemd160(cert_bytes)?;
        let node_id = Self::from_slice(&short_address);
        Ok(node_id)
    }

    /// Loads the existing staking certificates if exists,
    /// and returns the loaded or generated node Id.
    /// Returns "true" if generated.
    ///
    /// # Errors
    ///
    /// 如果无法加载或生成证书，则返回错误。
    pub fn load_or_generate_pem(key_path: &str, cert_path: &str) -> io::Result<(Self, bool)> {
        let tls_key_exists = Path::new(&key_path).exists();
        log::info!("staking TLS key {key_path} exists? {tls_key_exists}");

        let tls_cert_exists = Path::new(&cert_path).exists();
        log::info!("staking TLS cert {cert_path} exists? {tls_cert_exists}");

        let generated = if !tls_key_exists || !tls_cert_exists {
            log::info!(
                "generating staking TLS certs (key exists {tls_key_exists}, cert exists {tls_cert_exists})"
            );
            cert_manager::x509::generate_and_write_pem(None, key_path, cert_path)?;
            true
        } else {
            log::info!(
                "loading existing staking TLS certificates from '{key_path}' and '{cert_path}'"
            );
            false
        };

        let node_id = Self::from_cert_pem_file(cert_path)?;
        Ok((node_id, generated))
    }

    #[must_use]
    pub fn short_id(&self) -> short::Id {
        short::Id::from_slice(&self.0)
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
        let mut node_id = String::from(ENCODE_PREFIX);
        let short_id = formatting::encode_cb58_with_checksum_string(&self.0);
        node_id.push_str(&short_id);
        write!(f, "{node_id}")
    }
}

/// ref. <https://doc.rust-lang.org/std/str/trait.FromStr.html>
impl FromStr for Id {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // trim in case it's parsed from list
        let processed = s.trim().trim_start_matches(ENCODE_PREFIX);
        let decoded = formatting::decode_cb58_with_checksum(processed).map_err(|e| {
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
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct IdVisitor;

        impl Visitor<'_> for IdVisitor {
            type Value = Id;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a base-58 encoded ID-string with checksum")
            }

            fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Id::from_str(v).map_err(E::custom)
            }
        }

        deserializer.deserialize_any(IdVisitor)
    }
}

/// `RUST_LOG=debug` cargo test --package avalanche-types --lib --
/// `ids::node::test_custom_de_serializer` --exact --show-output
#[test]
fn test_custom_de_serializer() {
    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
    struct Data {
        node_id: Id,
    }

    let d = Data {
        node_id: Id::from_str("NodeID-6ZmBHXTqjknJoZtXbnJ6x7af863rXDTwx").unwrap(),
    };

    let yaml_encoded = serde_yaml::to_string(&d).unwrap();
    println!("yaml_encoded:\n{yaml_encoded}");
    let yaml_decoded = serde_yaml::from_str(&yaml_encoded).unwrap();
    assert_eq!(d, yaml_decoded);

    let json_encoded = serde_json::to_string(&d).unwrap();
    println!("json_encoded:\n{json_encoded}");
    let json_decoded = serde_json::from_str(&json_encoded).unwrap();
    assert_eq!(d, json_decoded);

    let json_decoded_2: Data =
        serde_json::from_str(r#"{ "node_id":"NodeID-6ZmBHXTqjknJoZtXbnJ6x7af863rXDTwx" }"#)
            .unwrap();
    assert_eq!(d, json_decoded_2);

    let json_encoded_3 = serde_json::json!(
        {
            "node_id": "NodeID-6ZmBHXTqjknJoZtXbnJ6x7af863rXDTwx"
        }
    );
    let json_decoded_3: Data = serde_json::from_value(json_encoded_3).unwrap();
    assert_eq!(d, json_decoded_3);
}

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
                "empty node::Id from deserialization",
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
/// Use [`serde(deserialize_with = "``short::must_deserialize_ids``")`] to serde with
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
/// `ids::node::test_serialize` --exact --show-output
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
struct Data {
    /// The node ID.
    id: Id,
    /// Optional node ID.
    id2: Option<Id>,
    /// List of node IDs.
    ids: Vec<Id>,
}

#[test]
fn test_serialize() {
    let id = Id::from_slice(&<Vec<u8>>::from([
        0x3d, 0x0a, 0xd1, 0x2b, 0x8e, 0xe8, 0x92, 0x8e, 0xdf, 0x24, //
        0x8c, 0xa9, 0x1c, 0xa5, 0x56, 0x00, 0xfb, 0x38, 0x3f, 0x07, //
    ]));
    assert_eq!(id.to_string(), "NodeID-6ZmBHXTqjknJoZtXbnJ6x7af863rXDTwx");

    let d = Data {
        id,
        id2: Some(id),
        ids: vec![id, id, id, id, id],
    };

    let yaml_encoded = serde_yaml::to_string(&d).unwrap();
    assert!(yaml_encoded.contains("NodeID-6ZmBHXTqjknJoZtXbnJ6x7af863rXDTwx"));
    let yaml_decoded = serde_yaml::from_str(&yaml_encoded).unwrap();
    assert_eq!(d, yaml_decoded);

    let json_encoded = serde_json::to_string(&d).unwrap();
    assert!(json_encoded.contains("NodeID-6ZmBHXTqjknJoZtXbnJ6x7af863rXDTwx"));
    let json_decoded = serde_json::from_str(&json_encoded).unwrap();
    assert_eq!(d, json_decoded);
}

/// Set is a set of `NodeIds`
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/ids#NewNodeIDSet>
pub type Set = HashSet<Id>;

/// Return a new `NodeIdSet` with initial capacity [`size`].
///
/// More or less than [`size`] elements can be added to this set.
/// Using `NewNodeIDSet()` rather than ids.NodeIDSet{} is just an optimization
/// that can be used if you know how many elements will be put in this set.
#[must_use]
pub fn new_set(size: usize) -> Set {
    let set: HashSet<Id> = HashSet::with_capacity(size);
    set
}

/// Tests for node ID functionality with certificate files.
#[cfg(test)]
mod cert_file_tests {
    use super::*;
    use std::str::FromStr;

    /// Initialize logger for tests.
    fn init_logger() {
        // No-op function for test setup
        // Logger initialization moved to individual test functions if needed
    }

    /// Verify a node ID from bytes matches expected string representation.
    fn verify_node_id_from_bytes(bytes: &[u8], expected_id: &str) {
        let node_id = Id::from_slice(&Vec::from(bytes));
        assert_eq!(format!("{node_id}"), expected_id);
        assert_eq!(node_id.to_string(), expected_id);

        // Test with and without prefix
        let short_id = expected_id.trim_start_matches("NodeID-");
        assert_eq!(node_id.short_id().to_string(), short_id);
        assert_eq!(node_id, Id::from_str(short_id).unwrap());
        assert_eq!(node_id, Id::from_str(expected_id).unwrap());
    }

    /// Verify a node ID from certificate file matches expected string representation.
    fn verify_node_id_from_cert(cert_path: &str, expected_id: &str) {
        let node_id = Id::from_cert_pem_file(cert_path).unwrap();
        assert_eq!(format!("{node_id}"), expected_id);
        assert_eq!(node_id.to_string(), expected_id);

        // Test with and without prefix
        let short_id = expected_id.trim_start_matches("NodeID-");
        assert_eq!(node_id, Id::from_str(short_id).unwrap());
        assert_eq!(node_id, Id::from_str(expected_id).unwrap());
    }

    /// Test basic node ID creation from bytes.
    #[test]
    fn test_node_id_from_bytes() {
        init_logger();

        verify_node_id_from_bytes(
            &[
                0x3d, 0x0a, 0xd1, 0x2b, 0x8e, 0xe8, 0x92, 0x8e, 0xdf, 0x24, 0x8c, 0xa9, 0x1c, 0xa5,
                0x56, 0x00, 0xfb, 0x38, 0x3f, 0x07,
            ],
            "NodeID-6ZmBHXTqjknJoZtXbnJ6x7af863rXDTwx",
        );
    }

    /// Test node ID creation from staker1 certificate.
    #[test]
    fn test_staker1_cert() {
        init_logger();

        // copied from "avalanchego/staking/local/staking1.key,crt"
        // verified by "avalanchego-compatibility/node-id" for compatibility with Go
        verify_node_id_from_cert(
            "./artifacts/staker1.insecure.crt",
            "NodeID-7Xhw2mDxuDS44j42TCB6U5579esbSt3Lg",
        );
    }

    /// Test node ID creation from staker2 certificate.
    #[test]
    fn test_staker2_cert() {
        init_logger();

        verify_node_id_from_cert(
            "./artifacts/staker2.insecure.crt",
            "NodeID-MFrZFVCXPv5iCn6M9K6XduxGTYp891xXZ",
        );
    }

    /// Test node ID creation from staker3 certificate.
    #[test]
    fn test_staker3_cert() {
        init_logger();

        verify_node_id_from_cert(
            "./artifacts/staker3.insecure.crt",
            "NodeID-NFBbbJ4qCmNaCzeW7sxErhvWqvEQMnYcN",
        );
    }

    /// Test node ID creation from staker4 certificate.
    #[test]
    fn test_staker4_cert() {
        init_logger();

        verify_node_id_from_cert(
            "./artifacts/staker4.insecure.crt",
            "NodeID-GWPcbFJZFfZreETSoWjPimr846mXEKCtu",
        );
    }

    /// Test node ID creation from staker5 certificate.
    #[test]
    fn test_staker5_cert() {
        init_logger();

        verify_node_id_from_cert(
            "./artifacts/staker5.insecure.crt",
            "NodeID-P7oB2McjBGgW2NXXWVYjV8JEDFoW9xDE5",
        );
    }

    /// Test node ID creation from test certificate.
    #[test]
    fn test_test_cert() {
        init_logger();

        verify_node_id_from_cert(
            "./artifacts/test.insecure.crt",
            "NodeID-29HTAG5cfN2fw79A67Jd5zY9drcT51EBG",
        );
    }
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

/// Tests for sorting and comparing node IDs.
#[cfg(test)]
mod sort_tests {
    use super::*;

    /// Helper function to create a node ID from a byte value.
    fn create_id(first_byte: u8) -> Id {
        let mut bytes = vec![0u8; 20];
        bytes[0] = first_byte;
        Id::from_slice(&bytes)
    }

    /// Helper function to create a set of node IDs.
    fn create_id_set(first_bytes: &[u8]) -> Ids {
        let ids = first_bytes
            .iter()
            .map(|&b| create_id(b))
            .collect::<Vec<_>>();
        Ids(ids)
    }

    /// Test equality of node IDs.
    #[test]
    fn test_id_equality() {
        let id1 = create_id(1);
        let id2 = create_id(1);
        assert_eq!(id1, id2);
    }

    /// Test comparison of node IDs.
    #[test]
    fn test_id_comparison() {
        let id1 = create_id(1);
        let id2 = create_id(2);

        assert!(id1 < id2);
        assert!(id2 > id1);
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

    /// Test comparison of ID sets with same length but different elements.
    #[test]
    fn test_id_set_element_comparison() {
        // Same length (3), but different elements
        let id_set1 = create_id_set(&[1, 2, 3]);
        let id_set2 = create_id_set(&[1, 2, 5]);

        assert!(id_set1 < id_set2);
    }

    /// Test comparison of ID sets where longer set has smaller elements.
    #[test]
    fn test_id_set_mixed_comparison() {
        // Set with 4 elements vs set with 3 elements with larger values
        let id_set1 = create_id_set(&[1, 2, 3, 4]);
        let id_set2 = create_id_set(&[9, 9, 9]);

        assert!(id_set1 > id_set2);
    }

    /// Test sorting of node IDs.
    #[test]
    fn test_id_sorting() {
        let mut ids = vec![create_id(3), create_id(2), create_id(1)];
        ids.sort();

        let expected = vec![create_id(1), create_id(2), create_id(3)];
        assert_eq!(ids, expected);
    }
}

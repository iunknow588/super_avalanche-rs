//! Implements the Avalanche ID type (32-byte).
//!
//! ```
//! use avalanche_types::ids;
//!
//! assert_eq!(
//!     format!("{}", ids::Id::default()),
//!     "11111111111111111111111111111111LpoYY"
//! );
//! ```

pub mod bag;
pub mod bits;
pub mod node;
pub mod short;

use std::{
    cmp::Ordering,
    collections::HashSet,
    fmt,
    hash::{Hash, Hasher},
    str::FromStr,
};

use crate::{
    errors::{Error, Result},
    formatting, hash, packer,
};
use lazy_static::lazy_static;
use serde::{self, de::Visitor, Deserialize, Deserializer, Serialize, Serializer};
use zerocopy::{AsBytes, FromBytes, FromZeroes, Unaligned};

pub const LEN: usize = 32;

lazy_static! {
    static ref EMPTY: Vec<u8> = vec![0; LEN];
}

/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/ids#ID>
/// ref. <https://docs.rs/zerocopy/latest/zerocopy/trait.AsBytes.html#safety>
#[derive(Debug, Clone, Copy, Eq, AsBytes, FromZeroes, FromBytes, Unaligned)]
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

    /// SHA256-hashes the given byte slice into an Id.
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/ids#ToID>
    #[must_use]
    pub fn sha256(d: impl AsRef<[u8]>) -> Self {
        Self::from_slice(&hash::sha256(d))
    }

    /// If the passed array is shorter than the LEN,
    /// it fills in with zero.
    ///
    /// # Panics
    ///
    /// Panics if the input slice is longer than LEN or if the conversion to fixed-size array fails.
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

    /// ref. "`ids.ID.Prefix(output_index)`"
    ///
    /// # Errors
    ///
    /// Returns an error if packing the data fails.
    pub fn prefix(&self, prefixes: &[u64]) -> Result<Self> {
        let n = prefixes.len() + packer::U64_LEN + 32;
        let packer = packer::Packer::new(n, n);
        for pfx in prefixes {
            packer.pack_u64(*pfx)?;
        }
        packer.pack_bytes(&self.0)?;

        let b = packer.take_bytes();
        let d = hash::sha256(&b);
        Ok(Self::from_slice(&d))
    }

    /// Returns the bit value at the i-th index of the byte array (0 or 1).
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/ids#ID.Bit>
    ///
    /// # Panics
    ///
    /// Panics if the bit value is not 0 or 1, which should never happen.
    #[must_use]
    pub fn bit(&self, i: usize) -> bits::Bit {
        let byte_index = i / 8;
        let bit_index = i % 8;

        let mut b = self.0[byte_index];

        // b = [7, 6, 5, 4, 3, 2, 1, 0]

        b >>= bit_index;

        // b = [0, ..., bit_index + 1, bit_index]
        // 1 = [0, 0, 0, 0, 0, 0, 0, 1]

        b &= 1;

        // b = [0, 0, 0, 0, 0, 0, 0, bit_index]

        // must be either 0 or 1
        bits::Bit::try_from(b as usize).expect("bit value must be 0 or 1")
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
    type Err = std::io::Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        // trim in case it's parsed from list
        let decoded = formatting::decode_cb58_with_checksum(s.trim()).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("failed decode_cb58_with_checksum '{e}'"),
            )
        })?;
        Ok(Self::from_slice(&decoded))
    }
}

impl TryFrom<std::borrow::Cow<'static, str>> for Id {
    type Error = std::io::Error;

    fn try_from(v: std::borrow::Cow<'static, str>) -> std::result::Result<Self, Self::Error> {
        Self::from_str(v.as_ref())
    }
}

/// Custom serializer.
/// ref. <https://serde.rs/impl-serialize.html>
impl Serialize for Id {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
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
/// `ids::test_custom_de_serializer` --exact --show-output
#[test]
fn test_custom_de_serializer() {
    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
    struct Data {
        id: Id,
    }

    let d = Data {
        id: Id::from_str("g25v3qDyAaHfR7kBev8tLUHouSgN5BJuZjy1BYS1oiHd2vres").unwrap(),
    };

    let yaml_encoded = serde_yaml::to_string(&d).unwrap();
    println!("yaml_encoded:\n{yaml_encoded}");
    let yaml_decoded = serde_yaml::from_str(&yaml_encoded).unwrap();
    assert_eq!(d, yaml_decoded);

    let json_encoded = serde_json::to_string(&d).unwrap();
    println!("json_encoded:\n{json_encoded}");
    let json_decoded = serde_json::from_str(&json_encoded).unwrap();
    assert_eq!(d, json_decoded);

    let json_decoded_2: Data = serde_json::from_str(
        "

{
    \"id\":\"g25v3qDyAaHfR7kBev8tLUHouSgN5BJuZjy1BYS1oiHd2vres\"
}

",
    )
    .unwrap();
    assert_eq!(d, json_decoded_2);

    let json_encoded_3 = serde_json::json!(
        {
            "id": "g25v3qDyAaHfR7kBev8tLUHouSgN5BJuZjy1BYS1oiHd2vres"
        }
    );
    let json_decoded_3: Data = serde_json::from_value(json_encoded_3).unwrap();
    assert_eq!(d, json_decoded_3);
}

fn fmt_id<'de, D>(deserializer: D) -> std::result::Result<Id, D::Error>
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
/// Returns an error if deserialization fails.
pub fn deserialize_id<'de, D>(deserializer: D) -> std::result::Result<Option<Id>, D::Error>
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
/// Returns an error if deserialization fails or if the ID is empty.
pub fn must_deserialize_id<'de, D>(deserializer: D) -> std::result::Result<Id, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct Wrapper(#[serde(deserialize_with = "fmt_id")] Id);
    let v = Option::deserialize(deserializer)?;
    v.map(|Wrapper(a)| a).map_or_else(
        || Err(serde::de::Error::custom("empty Id from deserialization")),
        Ok,
    )
}

/// Custom deserializer.
/// ref. <https://serde.rs/impl-deserialize.html>
///
/// # Errors
///
/// Returns an error if deserialization fails.
pub fn deserialize_ids<'de, D>(deserializer: D) -> std::result::Result<Option<Vec<Id>>, D::Error>
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
/// Returns an error if deserialization fails or if the IDs are empty.
pub fn must_deserialize_ids<'de, D>(deserializer: D) -> std::result::Result<Vec<Id>, D::Error>
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

/// Formats IDs from a string.
///
/// # Errors
///
/// Returns an error if deserialization fails.
fn fmt_ids<'de, D>(deserializer: D) -> std::result::Result<Vec<Id>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    type Strings = Vec<String>;
    let ss = Strings::deserialize(deserializer)?;
    match ss
        .iter()
        .map(|x| x.parse::<Id>())
        .collect::<std::result::Result<Vec<Id>, std::io::Error>>()
    {
        Ok(x) => Ok(x),
        Err(e) => Err(serde::de::Error::custom(format!(
            "failed to deserialize Ids {e}"
        ))),
    }
}

/// `RUST_LOG=debug` cargo test --package avalanche-types --lib --
/// `ids::test_serialize` --exact --show-output
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
struct Data {
    /// The ID.
    id: Id,
    /// Optional ID.
    id2: Option<Id>,
    /// List of IDs.
    ids: Vec<Id>,
}

#[test]
fn test_serialize() {
    let id = Id::from_slice(&<Vec<u8>>::from([
        0x3d, 0x0a, 0xd1, 0x2b, 0x8e, 0xe8, 0x92, 0x8e, 0xdf, 0x24, //
        0x8c, 0xa9, 0x1c, 0xa5, 0x56, 0x00, 0xfb, 0x38, 0x3f, 0x07, //
        0xc3, 0x2b, 0xff, 0x1d, 0x6d, 0xec, 0x47, 0x2b, 0x25, 0xcf, //
        0x59, 0xa7,
    ]));
    assert_eq!(
        id.to_string(),
        "TtF4d2QWbk5vzQGTEPrN48x6vwgAoAmKQ9cbp79inpQmcRKES"
    );

    let d = Data {
        id,
        id2: Some(id),
        ids: vec![id, id, id, id, id],
    };

    let yaml_encoded = serde_yaml::to_string(&d).unwrap();
    assert!(yaml_encoded.contains("TtF4d2QWbk5vzQGTEPrN48x6vwgAoAmKQ9cbp79inpQmcRKES"));
    let yaml_decoded = serde_yaml::from_str(&yaml_encoded).unwrap();
    assert_eq!(d, yaml_decoded);

    let json_encoded = serde_json::to_string(&d).unwrap();
    assert!(json_encoded.contains("TtF4d2QWbk5vzQGTEPrN48x6vwgAoAmKQ9cbp79inpQmcRKES"));
    let json_decoded = serde_json::from_str(&json_encoded).unwrap();
    assert_eq!(d, json_decoded);
}

/// Set is a set of Ids.
/// <https://pkg.go.dev/github.com/ava-labs/avalanchego/ids#Set>
pub type Set = HashSet<Id>;

/// Return a new set with initial capacity `size`.
/// More or less than `size` elements can be added to this set.
#[must_use]
pub fn new_set(size: usize) -> Set {
    let set: HashSet<Id> = HashSet::with_capacity(size);
    set
}

/// `RUST_LOG=debug` cargo test --package avalanche-types --lib -- `ids::test_id`
/// --exact --show-output ref. "avalanchego/ids.TestIDMarshalJSON"
#[test]
fn test_id() {
    let id = Id::from_slice(&<Vec<u8>>::from([
        0x3d, 0x0a, 0xd1, 0x2b, 0x8e, 0xe8, 0x92, 0x8e, 0xdf, 0x24, //
        0x8c, 0xa9, 0x1c, 0xa5, 0x56, 0x00, 0xfb, 0x38, 0x3f, 0x07, //
        0xc3, 0x2b, 0xff, 0x1d, 0x6d, 0xec, 0x47, 0x2b, 0x25, 0xcf, //
        0x59, 0xa7,
    ]));
    assert_eq!(
        id.to_string(),
        "TtF4d2QWbk5vzQGTEPrN48x6vwgAoAmKQ9cbp79inpQmcRKES"
    );
    assert_eq!(
        id.to_vec(),
        <Vec<u8>>::from([
            0x3d, 0x0a, 0xd1, 0x2b, 0x8e, 0xe8, 0x92, 0x8e, 0xdf, 0x24, //
            0x8c, 0xa9, 0x1c, 0xa5, 0x56, 0x00, 0xfb, 0x38, 0x3f, 0x07, //
            0xc3, 0x2b, 0xff, 0x1d, 0x6d, 0xec, 0x47, 0x2b, 0x25, 0xcf, //
            0x59, 0xa7,
        ])
    );

    let id_from_str = Id::from_str("TtF4d2QWbk5vzQGTEPrN48x6vwgAoAmKQ9cbp79inpQmcRKES")
        .expect("Id::from_str failed");
    assert_eq!(id, id_from_str);

    let id = Id::from_slice(&<Vec<u8>>::from([
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        0x00, 0x00,
    ]));
    assert_eq!(id.to_string(), "11111111111111111111111111111111LpoYY");
    let id_from_str =
        Id::from_str("11111111111111111111111111111111LpoYY").expect("Id::from_str failed");
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

/// Tests for sorting and comparing IDs.
#[cfg(test)]
mod sort_tests {
    use super::*;

    /// Helper function to create an ID from a byte value.
    fn create_id(first_byte: u8) -> Id {
        Id::from_slice(&[first_byte])
    }

    /// Helper function to create a set of IDs.
    fn create_id_set(first_bytes: &[u8]) -> Ids {
        let ids = first_bytes
            .iter()
            .map(|&b| create_id(b))
            .collect::<Vec<_>>();
        Ids(ids)
    }

    /// Test basic comparison of IDs.
    #[test]
    fn test_basic_comparison() {
        let id1 = create_id(1);
        let id2 = create_id(2);
        let id3 = create_id(3);

        assert!(id1 < id2);
        assert!(id2 < id3);
        assert!(id1 < id3);
    }

    /// Test equality of IDs with different byte lengths but same value.
    #[test]
    fn test_id_equality_different_lengths() {
        let id1 = Id::from_slice(&[0x01, 0x00, 0x00, 0x00]);
        let id2 = Id::from_slice(&[0x01, 0x00, 0x00, 0x00, 0x00]);

        assert_eq!(id1, id2);
    }

    /// Test comparison of IDs with different values.
    #[test]
    fn test_id_comparison() {
        // Test less than
        let id1 = Id::from_slice(&[0x01, 0x00, 0x00, 0x00, 0x00]);
        let id2 = Id::from_slice(&[0x02]);
        assert!(id1 < id2);

        // Test greater than
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

    /// Test sorting of ID sets.
    #[test]
    fn test_id_set_sorting() {
        let mut id_set = Ids(vec![create_id(3), create_id(2), create_id(1)]);
        id_set.0.sort();

        let expected = Ids(vec![create_id(1), create_id(2), create_id(3)]);
        assert_eq!(id_set, expected);
    }

    /// Test sorting of ID vectors.
    #[test]
    fn test_id_vector_sorting() {
        let mut ids = vec![create_id(3), create_id(2), create_id(1)];
        ids.sort();

        let expected = vec![create_id(1), create_id(2), create_id(3)];
        assert_eq!(ids, expected);
    }
}

/// Generates VM ID based on the name.
/// Encodes a VM name to an ID.
///
/// # Errors
///
/// Returns an error if the VM name is invalid.
pub fn encode_vm_name_to_id(name: &str) -> Result<Id> {
    let n = name.len();
    if n > LEN {
        return Err(Error::Other {
            message: format!("can't id {n} bytes (>{LEN})"),
            retryable: false,
        });
    }

    let input = name.as_bytes().to_vec();
    Ok(Id::from_slice(&input))
}

/// `RUST_LOG=debug` cargo test --package avalanche-types --lib -- `ids::test_vm_id`
/// --exact --show-output
#[test]
fn test_vm_id() {
    use log::info;
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let subnet_evm_id = encode_vm_name_to_id("subnetevm").expect("failed to generate id from str");
    assert_eq!(
        format!("{subnet_evm_id}"),
        "srEXiWaHuhNyGwPUi444Tu47ZEDwxTWrbQiuD7FmgSAQ6X7Dy"
    );

    let contents = random_manager::secure_string(30);
    let v = encode_vm_name_to_id(&contents).expect("failed to generate id from str");
    info!("vm_id_from_str: {v}");
}

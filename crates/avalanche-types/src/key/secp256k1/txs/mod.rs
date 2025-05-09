//! secp256k1 credential types.
pub mod transfer;

use std::cmp::Ordering;
use std::io;

use crate::{
    codec::{self, serde::hex_0x_bytes::Hex0xBytes},
    ids::short,
};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

/// Secp256k1 signature credential, containing multiple signatures (each with a length of 65 bytes).
///
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/avm/fxs#FxCredential>
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/components/verify#Verifiable>
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/secp256k1fx#Credential>
#[serde_as]
#[derive(Debug, Serialize, Deserialize, Eq, Clone, Default)]
pub struct Credential {
    /// Signatures, each must be length of 65.
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/utils/crypto#SECP256K1RSigLen>
    #[serde_as(as = "Vec<Hex0xBytes>")]
    pub signatures: Vec<Vec<u8>>,
}

impl Credential {
    #[must_use]
    pub const fn new(sigs: Vec<Vec<u8>>) -> Self {
        Self { signatures: sigs }
    }

    #[must_use]
    pub fn type_name() -> String {
        "secp256k1fx.Credential".to_string()
    }

    /// Returns the type ID for this credential.
    ///
    /// # Panics
    ///
    /// Panics if the type name is not found in the codec types map.
    #[must_use]
    pub fn type_id() -> u32 {
        u32::try_from(*(codec::X_TYPES.get(&Self::type_name()).unwrap())).unwrap()
    }

    /// # Errors
    ///
    /// Returns error if JSON encoding fails
    pub fn encode_json(&self) -> io::Result<String> {
        serde_json::to_string(self).map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))
    }
}

impl Ord for Credential {
    fn cmp(&self, other: &Self) -> Ordering {
        Signatures::new(&self.signatures).cmp(&Signatures::new(&other.signatures))
    }
}

impl PartialOrd for Credential {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Credential {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

/// `RUST_LOG=debug` cargo test --package avalanche-types --lib -- `key::secp256k1::txs::test_credential_custom_de_serializer` --exact --show-output
#[test]
fn test_credential_custom_de_serializer() {
    let d = Credential {
        signatures: vec![vec![123]],
    };

    let yaml_encoded = serde_yaml::to_string(&d).unwrap();
    println!("yaml_encoded:\n{yaml_encoded}");
    let yaml_decoded = serde_yaml::from_str(&yaml_encoded).unwrap();
    assert_eq!(d, yaml_decoded);

    let json_encoded = serde_json::to_string(&d).unwrap();
    println!("json_encoded:\n{json_encoded}");
    let json_decoded = serde_json::from_str(&json_encoded).unwrap();
    assert_eq!(d, json_decoded);

    let json_decoded_2: Credential = serde_json::from_str(
        "

{
    \"signatures\":[\"0x7b\"]
}

",
    )
    .unwrap();
    assert_eq!(d, json_decoded_2);
}

#[derive(Eq)]
pub struct Signatures(Vec<Vec<u8>>);

impl Signatures {
    #[must_use]
    pub fn new(sigs: &[Vec<u8>]) -> Self {
        Self(Vec::from(sigs))
    }
}

impl Ord for Signatures {
    fn cmp(&self, other: &Self) -> Ordering {
        // packer encodes the array length first
        // so if the lengths differ, the ordering is decided
        let l1 = self.0.len();
        let l2 = other.0.len();
        l1.cmp(&l2) // returns when lengths are not Equal
            .then_with(
                || self.0.cmp(&other.0), // if lengths are Equal, compare the signatures
            )
    }
}

impl PartialOrd for Signatures {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Signatures {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

/// NOTE: all signatures are fixed length
/// `RUST_LOG=debug` cargo test --package avalanche-types --lib -- `key::secp256k1::txs::test_sort_credentials` --exact --show-output
#[test]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn test_sort_credentials() {
    let mut credentials: Vec<Credential> = Vec::new();
    for i in (0..10).rev() {
        credentials.push(Credential {
            signatures: vec![
                vec![i as u8, 1, 2, 3],
                vec![i as u8, 2, 2, 3],
                vec![i as u8, 4, 2, 3],
            ],
        });
        credentials.push(Credential {
            signatures: vec![
                vec![i as u8, 1, 2, 3],
                vec![i as u8, 2, 2, 3],
                vec![i as u8, 3, 2, 3],
            ],
        });
        credentials.push(Credential {
            signatures: vec![vec![i as u8, 1, 2, 3], vec![i as u8, 2, 2, 3]],
        });
        credentials.push(Credential {
            signatures: vec![vec![i as u8, 2, 2, 3]],
        });
        credentials.push(Credential {
            signatures: vec![vec![i as u8, 1, 2, 3]],
        });
    }
    assert!(!cmp_manager::is_sorted_and_unique(&credentials));
    credentials.sort();

    let mut sorted_credentials: Vec<Credential> = Vec::new();
    for i in 0..10 {
        sorted_credentials.push(Credential {
            signatures: vec![vec![i as u8, 1, 2, 3]],
        });
        sorted_credentials.push(Credential {
            signatures: vec![vec![i as u8, 2, 2, 3]],
        });
    }
    for i in 0..10 {
        sorted_credentials.push(Credential {
            signatures: vec![vec![i as u8, 1, 2, 3], vec![i as u8, 2, 2, 3]],
        });
    }
    for i in 0..10 {
        sorted_credentials.push(Credential {
            signatures: vec![
                vec![i as u8, 1, 2, 3],
                vec![i as u8, 2, 2, 3],
                vec![i as u8, 3, 2, 3],
            ],
        });
        sorted_credentials.push(Credential {
            signatures: vec![
                vec![i as u8, 1, 2, 3],
                vec![i as u8, 2, 2, 3],
                vec![i as u8, 4, 2, 3],
            ],
        });
    }
    assert!(cmp_manager::is_sorted_and_unique(&sorted_credentials));
    assert_eq!(credentials, sorted_credentials);
}

#[derive(Eq)]
pub struct SigIndices(Vec<u32>);

impl SigIndices {
    #[must_use]
    pub fn new(ids: &[u32]) -> Self {
        Self(Vec::from(ids))
    }
}

impl Ord for SigIndices {
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

impl PartialOrd for SigIndices {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for SigIndices {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

/// Secp256k1 output owner information, containing locktime, threshold, and address list.
///
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/platformvm/fx#Owner>
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/secp256k1fx#OutputOwners>
#[derive(Debug, Serialize, Deserialize, Eq, Clone, Default)]
pub struct OutputOwners {
    pub locktime: u64,
    pub threshold: u32,
    pub addresses: Vec<short::Id>,
}

impl OutputOwners {
    #[must_use]
    pub fn new(locktime: u64, threshold: u32, addrs: &[short::Id]) -> Self {
        Self {
            locktime,
            threshold,
            addresses: Vec::from(addrs),
        }
    }

    #[must_use]
    pub fn type_name() -> String {
        "secp256k1fx.OutputOwners".to_string()
    }

    /// Returns the type ID for this output owner.
    ///
    /// # Panics
    ///
    /// Panics if the type name is not found in the codec types map.
    #[must_use]
    pub fn type_id() -> u32 {
        u32::try_from(*(codec::P_TYPES.get(&Self::type_name()).unwrap())).unwrap()
    }
}

impl Ord for OutputOwners {
    fn cmp(&self, other: &Self) -> Ordering {
        self.locktime
            .cmp(&(other.locktime)) // returns when "locktime"s are not Equal
            .then_with(
                || self.threshold.cmp(&other.threshold), // if "locktime"s are Equal, compare "threshold"
            )
            .then_with(
                || short::Ids::new(&self.addresses).cmp(&short::Ids::new(&other.addresses)), // if "locktime"s and "threshold"s are Equal, compare "addrs"
            )
    }
}

impl PartialOrd for OutputOwners {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for OutputOwners {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

/// `RUST_LOG=debug` cargo test --package avalanche-types --lib -- `key::secp256k1::txs::test_sort_output_owners` --exact --show-output
#[test]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn test_sort_output_owners() {
    let mut owners: Vec<OutputOwners> = Vec::new();
    for i in (0..10).rev() {
        owners.push(OutputOwners {
            locktime: i as u64,
            threshold: u32::try_from(i).unwrap(),
            addresses: vec![
                short::Id::from_slice(&[i as u8, 1, 2, 3]),
                short::Id::from_slice(&[i as u8, 2, 2, 3]),
            ],
        });
        owners.push(OutputOwners {
            locktime: i as u64,
            threshold: u32::try_from(i).unwrap(),
            addresses: vec![
                short::Id::from_slice(&[i as u8, 1, 2, 3]),
                short::Id::from_slice(&[i as u8, 1, 2, 3]),
            ],
        });
        owners.push(OutputOwners {
            locktime: i as u64,
            threshold: u32::try_from(i).unwrap(),
            addresses: vec![short::Id::from_slice(&[i as u8, 2, 2, 3])],
        });
        owners.push(OutputOwners {
            locktime: i as u64,
            threshold: u32::try_from(i).unwrap(),
            addresses: vec![short::Id::from_slice(&[i as u8, 1, 2, 3])],
        });
    }
    assert!(!cmp_manager::is_sorted_and_unique(&owners));
    owners.sort();

    let mut sorted_owners: Vec<OutputOwners> = Vec::new();
    for i in 0..10 {
        sorted_owners.push(OutputOwners {
            locktime: i as u64,
            threshold: u32::try_from(i).unwrap(),
            addresses: vec![short::Id::from_slice(&[i as u8, 1, 2, 3])],
        });
        sorted_owners.push(OutputOwners {
            locktime: i as u64,
            threshold: u32::try_from(i).unwrap(),
            addresses: vec![short::Id::from_slice(&[i as u8, 2, 2, 3])],
        });
        sorted_owners.push(OutputOwners {
            locktime: i as u64,
            threshold: u32::try_from(i).unwrap(),
            addresses: vec![
                short::Id::from_slice(&[i as u8, 1, 2, 3]),
                short::Id::from_slice(&[i as u8, 1, 2, 3]),
            ],
        });
        sorted_owners.push(OutputOwners {
            locktime: i as u64,
            threshold: u32::try_from(i).unwrap(),
            addresses: vec![
                short::Id::from_slice(&[i as u8, 1, 2, 3]),
                short::Id::from_slice(&[i as u8, 2, 2, 3]),
            ],
        });
    }
    assert!(cmp_manager::is_sorted_and_unique(&sorted_owners));
    assert_eq!(owners, sorted_owners);
}

/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/secp256k1fx#Input>
#[derive(Debug, Serialize, Deserialize, Eq, Clone, Default)]
pub struct Input {
    pub sig_indices: Vec<u32>,
}

impl Input {
    #[must_use]
    pub const fn new(sig_indices: Vec<u32>) -> Self {
        Self { sig_indices }
    }

    #[must_use]
    pub fn type_name() -> String {
        "secp256k1fx.Input".to_string()
    }

    /// Returns the type ID for this input.
    ///
    /// # Panics
    ///
    /// Panics if the type name is not found in the codec types map.
    #[must_use]
    pub fn type_id() -> u32 {
        u32::try_from(*(codec::P_TYPES.get(&Self::type_name()).unwrap())).unwrap()
    }
}

impl Ord for Input {
    fn cmp(&self, other: &Self) -> Ordering {
        SigIndices::new(&self.sig_indices).cmp(&SigIndices::new(&other.sig_indices))
    }
}

impl PartialOrd for Input {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Input {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

/// `RUST_LOG=debug` cargo test --package avalanche-types --lib -- `key::secp256k1::txs::test_sort_inputs` --exact --show-output
#[test]
#[allow(clippy::cast_sign_loss)]
fn test_sort_inputs() {
    let mut inputs: Vec<Input> = Vec::new();
    for i in (0..10).rev() {
        inputs.push(Input {
            sig_indices: vec![i as u32, 2, 2, 3, 4, 5, 6, 7, 8, 9],
        });
        inputs.push(Input {
            sig_indices: vec![i as u32, 1, 2, 3, 4, 5, 6, 7, 8, 9],
        });
        inputs.push(Input {
            sig_indices: vec![i as u32, 1, 2, 3, 4, 5],
        });
    }
    assert!(!cmp_manager::is_sorted_and_unique(&inputs));
    inputs.sort();

    let mut sorted_inputs: Vec<Input> = Vec::new();
    for i in 0..10 {
        sorted_inputs.push(Input {
            sig_indices: vec![i as u32, 1, 2, 3, 4, 5],
        });
    }
    for i in 0..10 {
        sorted_inputs.push(Input {
            sig_indices: vec![i as u32, 1, 2, 3, 4, 5, 6, 7, 8, 9],
        });
        sorted_inputs.push(Input {
            sig_indices: vec![i as u32, 2, 2, 3, 4, 5, 6, 7, 8, 9],
        });
    }
    assert!(cmp_manager::is_sorted_and_unique(&sorted_inputs));
    assert_eq!(inputs, sorted_inputs);
}

//! Implements the utils/formatting package of avalanchego.
use std::io::{Error, ErrorKind};

use crate::hash;
use bech32::{ToBase32, Variant};
use bs58::{decode::DecodeBuilder, encode::EncodeBuilder, Alphabet};

/// CB58 checksum length
const CHECKSUM_LENGTH: usize = 4;

/// Encodes bytes into CB58 with checksum.
///
/// # Returns
/// A string of CB58 encoded bytes with checksum.
#[must_use]
pub fn encode_cb58_with_checksum_string(d: &[u8]) -> String {
    EncodeBuilder::new(d, Alphabet::DEFAULT)
        .as_cb58(None)
        .into_string()
}

/// Encodes bytes into CB58 with checksum.
///
/// # Returns
/// A vector of CB58 encoded bytes with checksum.
#[must_use]
pub fn encode_cb58_with_checksum_vec(d: &[u8]) -> Vec<u8> {
    EncodeBuilder::new(d, Alphabet::DEFAULT)
        .as_cb58(None)
        .into_vec()
}

/// Decodes CB58 with checksum into bytes.
///
/// # Errors
/// Returns `Err` if the input is not valid CB58 encoding
pub fn decode_cb58_with_checksum(s: &str) -> Result<Vec<u8>, Error> {
    DecodeBuilder::new(s, Alphabet::DEFAULT)
        .as_cb58(None)
        .into_vec()
        .map_err(|err| {
            let msg = match err {
                bs58::decode::Error::InvalidChecksum {
                    checksum,
                    expected_checksum,
                } => format!("invalid checksum {checksum:?} != {expected_checksum:?}"),
                _ => format!("failed to decode base58 ({err})"),
            };

            Error::new(ErrorKind::InvalidInput, msg)
        })
}

/// Tests encoding and decoding CB58 with checksum.
#[test]
fn test_encode_c58_with_checksum() {
    // ref. <https://github.com/ava-labs/avalanchego/blob/v1.9.7/utils/formatting/encoding_test.go#>
    let d: Vec<u8> = Vec::new();
    let hashed = encode_cb58_with_checksum_string(&d);
    assert_eq!(hashed, "45PJLL");
    let decoded = decode_cb58_with_checksum(&hashed).unwrap();
    assert_eq!(d, decoded);

    let d: Vec<u8> = vec![0];
    let hashed = encode_cb58_with_checksum_string(&d);
    assert_eq!(hashed, "1c7hwa");
    let decoded = decode_cb58_with_checksum(&hashed).unwrap();
    assert_eq!(d, decoded);

    let d: Vec<u8> = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 255];
    let hashed = encode_cb58_with_checksum_string(&d);
    assert_eq!(hashed, "1NVSVezva3bAtJesnUj");
    let decoded = decode_cb58_with_checksum(&hashed).unwrap();
    assert_eq!(d, decoded);

    let d: Vec<u8> = vec![
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
        26, 27, 28, 29, 30, 31, 32,
    ];
    let hashed = encode_cb58_with_checksum_string(&d);
    assert_eq!(hashed, "SkB92YpWm4Q2ijQHH34cqbKkCZWszsiQgHVjtNeFF2HdvDQU");
    let decoded = decode_cb58_with_checksum(&hashed).unwrap();
    assert_eq!(d, decoded);
}

/// Encodes bytes into Hex with checksum.
///
/// # Returns
/// A string of Hex encoded bytes with checksum.
#[must_use]
pub fn encode_hex_with_checksum(d: &[u8]) -> String {
    // "hashing.Checksum" of "sha256.Sum256"
    let checksum = hash::sha256(d);
    let checksum_length = checksum.len();
    let checksum = &checksum[checksum_length - CHECKSUM_LENGTH..];

    let mut checked = d.to_vec();
    let mut checksum = checksum.to_vec();
    checked.append(&mut checksum);

    hex::encode(&checked)
}

/// Decodes Hex with checksum into bytes.
///
/// # Errors
/// Returns `Err` if the input is not valid Hex encoding
pub fn decode_hex_with_checksum(s: &[u8]) -> Result<Vec<u8>, Error> {
    let decoded = match hex::decode(s) {
        Ok(v) => v,
        Err(e) => {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!("failed to decode base58 ({e})"),
            ));
        }
    };
    let decoded_length = decoded.len();

    // verify checksum
    let checksum = &decoded[decoded_length - CHECKSUM_LENGTH..];
    let orig = &decoded[..decoded_length - CHECKSUM_LENGTH];

    // "hashing.Checksum" of "sha256.Sum256"
    let orig_checksum = hash::sha256(orig);
    let orig_checksum_length = orig_checksum.len();
    let orig_checksum = &orig_checksum[orig_checksum_length - CHECKSUM_LENGTH..];
    if !cmp_manager::eq_vectors(checksum, orig_checksum) {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            format!("invalid checksum {checksum:?} != {orig_checksum:?}"),
        ));
    }

    Ok(orig.to_vec())
}

/// Tests encoding and decoding Hex with checksum.
#[test]
fn test_encode_hex_with_checksum() {
    // ref. <https://github.com/ava-labs/avalanchego/blob/v1.9.7/utils/formatting/encoding_test.go>
    let d: Vec<u8> = Vec::new();
    let hashed = encode_hex_with_checksum(&d);
    assert_eq!(hashed, "7852b855");
    let decoded = decode_hex_with_checksum(hashed.as_bytes()).unwrap();
    assert_eq!(d, decoded);

    let d: Vec<u8> = vec![0];
    let hashed = encode_hex_with_checksum(&d);
    assert_eq!(hashed, "0017afa01d");
    let decoded = decode_hex_with_checksum(hashed.as_bytes()).unwrap();
    assert_eq!(d, decoded);

    let d: Vec<u8> = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 255];
    let hashed = encode_hex_with_checksum(&d);
    assert_eq!(hashed, "00010203040506070809ff4482539c");
    let decoded = decode_hex_with_checksum(hashed.as_bytes()).unwrap();
    assert_eq!(d, decoded);

    let d: Vec<u8> = vec![
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
        26, 27, 28, 29, 30, 31, 32,
    ];
    let hashed = encode_hex_with_checksum(&d);
    assert_eq!(
        hashed,
        "0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20b7a612c9"
    );
    let decoded = decode_hex_with_checksum(hashed.as_bytes()).unwrap();
    assert_eq!(d, decoded);
}

/// Formats an address with the given chain ID alias, HRP, and bytes.
///
/// # Errors
/// Returns `Err` if the input is not valid
///
/// # Panics
/// Panics if the input length is not 20 bytes
pub fn address(chain_id_alias: &str, hrp: &str, d: &[u8]) -> Result<String, Error> {
    assert_eq!(d.len(), 20);

    // No need to call "bech32.ConvertBits(payload, 8, 5, true)"
    // ".to_base32()" already does "bech32::convert_bits(d, 8, 5, true)"
    let encoded = match bech32::encode(hrp, d.to_base32(), Variant::Bech32) {
        Ok(enc) => enc,
        Err(e) => {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!("failed bech32::encode {e}"),
            ));
        }
    };
    Ok(format!("{chain_id_alias}-{encoded}"))
}

/// 可能panic的函数
///
/// # Panics
/// 当输入无效时会panic
pub const fn may_panic_func(_input: &str) {
    // ...
}

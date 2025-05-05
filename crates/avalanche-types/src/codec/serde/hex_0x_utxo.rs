use crate::txs::utxo::Utxo;
use serde::{self, Deserialize, Deserializer, Serializer};
use serde_with::{DeserializeAs, SerializeAs};

/// 将 UTXO 序列化为十六进制字符串。
///
/// # Errors
///
/// 如果序列化失败，则返回错误。
pub fn serialize<S>(x: &Utxo, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let utxo_hex = x.to_hex().map_err(serde::ser::Error::custom)?;
    serializer.serialize_str(&utxo_hex)
}

/// 从十六进制字符串反序列化为 UTXO。
///
/// # Errors
///
/// 如果反序列化失败，则返回错误。
pub fn deserialize<'de, D>(deserializer: D) -> Result<Utxo, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;

    Utxo::from_hex(&s).map_err(serde::de::Error::custom)
}

pub struct Hex0xUtxo;

impl SerializeAs<Utxo> for Hex0xUtxo {
    /// 将 UTXO 序列化为十六进制字符串。
    ///
    /// # Errors
    ///
    /// 如果序列化失败，则返回错误。
    fn serialize_as<S>(x: &Utxo, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = x.to_hex().map_err(serde::ser::Error::custom)?;

        serializer.serialize_str(&s)
    }
}

impl<'de> DeserializeAs<'de, Utxo> for Hex0xUtxo {
    /// 从十六进制字符串反序列化为 UTXO。
    ///
    /// # Errors
    ///
    /// 如果反序列化失败，则返回错误。
    fn deserialize_as<D>(deserializer: D) -> Result<Utxo, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        Utxo::from_hex(&s).map_err(serde::de::Error::custom)
    }
}

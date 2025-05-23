use chrono::{DateTime, SecondsFormat, TimeZone, Utc};
use serde::{self, Deserialize, Deserializer, Serializer};
use serde_with::{DeserializeAs, SerializeAs};

/// 将`DateTime&lt;Utc&gt;`序列化为RFC3339格式字符串。
///
/// # Errors
/// 序列化器失败时返回错误。
pub fn serialize<S>(x: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    // ref. <https://docs.rs/chrono/0.4.19/chrono/struct.DateTime.html#method.to_rfc3339_opts>
    serializer.serialize_str(&x.to_rfc3339_opts(SecondsFormat::Millis, true))
}

/// 从RFC3339格式字符串反序列化为 `DateTime<Utc>`。
///
/// # Errors
/// 字符串格式不合法或反序列化失败时返回错误。
pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;

    // ref. <https://docs.rs/chrono/0.4.19/chrono/struct.DateTime.html#method.to_rfc3339_opts>
    match DateTime::parse_from_rfc3339(&s).map_err(serde::de::Error::custom) {
        Ok(dt) => Ok(Utc.from_utc_datetime(&dt.naive_utc())),
        Err(e) => Err(e),
    }
}

pub struct DateTimeUtc;

impl SerializeAs<DateTime<Utc>> for DateTimeUtc {
    fn serialize_as<S>(x: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&x.to_rfc3339_opts(SecondsFormat::Millis, true))
    }
}

impl<'de> DeserializeAs<'de, DateTime<Utc>> for DateTimeUtc {
    fn deserialize_as<D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        match DateTime::parse_from_rfc3339(&s).map_err(serde::de::Error::custom) {
            Ok(dt) => Ok(Utc.from_utc_datetime(&dt.naive_utc())),
            Err(e) => Err(e),
        }
    }
}

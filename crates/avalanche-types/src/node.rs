//! Defines the node type.
use serde::{Deserialize, Serialize};

/// Defines the node type.
/// MUST BE either "anchor" or "non-anchor"
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
pub enum Kind {
    #[serde(rename = "anchor")]
    Anchor,
    #[serde(rename = "non-anchor")]
    NonAnchor,
    Unknown(String),
}

impl std::convert::From<&str> for Kind {
    fn from(s: &str) -> Self {
        match s {
            "anchor" => Self::Anchor,
            "non-anchor" | "non_anchor" => Self::NonAnchor,

            other => Self::Unknown(other.to_owned()),
        }
    }
}

impl std::str::FromStr for Kind {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Self::from(s))
    }
}

impl Kind {
    /// Returns the `&str` value of the enum member.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Anchor => "anchor",
            Self::NonAnchor => "non-anchor",

            Self::Unknown(s) => s.as_ref(),
        }
    }

    /// Returns all the `&str` values of the enum members.
    #[must_use]
    pub const fn values() -> &'static [&'static str] {
        &[
            "anchor",     //
            "non-anchor", //
        ]
    }
}

impl AsRef<str> for Kind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

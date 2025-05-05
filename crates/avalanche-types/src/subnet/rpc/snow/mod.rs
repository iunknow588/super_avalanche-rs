//! Current runtime state of a VM.
pub mod engine;
pub mod validators;

/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/snow#State>
#[derive(PartialEq, Eq)]
pub enum State {
    Initializing = 0,
    StateSyncing = 1,
    Bootstrapping = 2,
    NormalOp = 3,
}

impl State {
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/snow#State.String>
    #[must_use]
    pub const fn as_str(&self) -> &str {
        match self {
            Self::Initializing => "Initializing state",
            Self::StateSyncing => "State syncing state",
            Self::Bootstrapping => "Bootstrapping state",
            Self::NormalOp => "Normal operations state",
        }
    }

    /// Returns the u32 primitive representation of the state.
    #[must_use]
    pub const fn to_i32(&self) -> i32 {
        match self {
            Self::Initializing => 1,
            Self::StateSyncing => 2,
            Self::Bootstrapping => 3,
            Self::NormalOp => 0,
        }
    }
}

impl TryFrom<u32> for State {
    type Error = ();

    fn try_from(kind: u32) -> std::result::Result<Self, Self::Error> {
        match kind {
            kind if kind == Self::Initializing as u32 => Ok(Self::Initializing),
            kind if kind == Self::StateSyncing as u32 => Ok(Self::StateSyncing),
            kind if kind == Self::Bootstrapping as u32 => Ok(Self::Bootstrapping),
            kind if kind == Self::NormalOp as u32 => Ok(Self::NormalOp),
            _ => Err(()),
        }
    }
}

impl TryFrom<i32> for State {
    type Error = ();

    fn try_from(kind: i32) -> std::result::Result<Self, Self::Error> {
        match kind {
            kind if kind == Self::Initializing as i32 => Ok(Self::Initializing),
            kind if kind == Self::StateSyncing as i32 => Ok(Self::StateSyncing),
            kind if kind == Self::Bootstrapping as i32 => Ok(Self::Bootstrapping),
            kind if kind == Self::NormalOp as i32 => Ok(Self::NormalOp),
            _ => Err(()),
        }
    }
}

#[test]
fn test_state() {
    let s = State::try_from(0).unwrap();
    assert!(matches!(s, State::Initializing));
    assert!(s.as_str() == "Initializing state");

    let s = State::try_from(1).unwrap();
    assert!(matches!(s, State::StateSyncing));
    assert!(s.as_str() == "State syncing state");

    let s = State::try_from(2).unwrap();
    assert!(matches!(s, State::Bootstrapping));
    assert!(s.as_str() == "Bootstrapping state");

    let s = State::try_from(3).unwrap();
    assert!(matches!(s, State::NormalOp));
    assert!(s.as_str() == "Normal operations state");
}

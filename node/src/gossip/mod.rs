//! Contains utilities related
//! to the Gossip protocol.

pub(crate) mod protocol;

/// Represents the data to disseminate
/// using the Gossip protocol.
pub(crate) struct State {
    pub data: String,
    pub timestamp: u128,
}

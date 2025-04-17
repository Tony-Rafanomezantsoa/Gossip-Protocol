//! Contains utilities related
//! to the Gossip protocol.

pub(crate) mod protocol;
pub(crate) mod request_handler;
pub(crate) mod request_initiator;

/// Represents the data to disseminate
/// using the Gossip protocol.
#[derive(PartialEq, Eq, Debug, Clone)]
pub(crate) struct State {
    pub data: String,
    pub timestamp: u128,
}

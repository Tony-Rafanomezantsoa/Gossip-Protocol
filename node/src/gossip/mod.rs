//! Contains utilities related
//! to the Gossip protocol.

pub(crate) mod protocol;
pub(crate) mod request_handler;

/// Represents the data to disseminate
/// using the Gossip protocol.
#[derive(PartialEq, Eq, Debug)]
pub(crate) struct State {
    pub data: String,
    pub timestamp: u128,
}

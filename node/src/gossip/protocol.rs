//! Contains abstractions related
//! to the Gossip protocol.

use std::error::Error;

use super::State;

/// Request abstraction for
/// the Gossip protocol.
pub(crate) enum GossipRequest {
    UpdateData(String),
    ShareData(Option<State>),
}

impl GossipRequest {
    pub(crate) fn parse(request: &str) -> Result<Self, Box<dyn Error>> {
        todo!()
    }
}

#[cfg(test)]
mod gossip_request_protocol_test {
    use super::GossipRequest;

    #[test]
    fn update_data_request_parse_test() {
        let request = "UPDATE=[Some data ...];";

        let gossip_request = GossipRequest::parse(request).unwrap();

        if let GossipRequest::UpdateData(data) = gossip_request {
            assert_eq!(data, String::from("Some data ..."));
        } else {
            panic!("parsing error");
        }
    }
}
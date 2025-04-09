//! Contains abstractions related
//! to the Gossip protocol.

use std::error::Error;

use regex::Regex;

use super::State;

/// Request abstraction for
/// the Gossip protocol.
pub(crate) enum GossipRequest {
    UpdateData(String),
    ShareData(Option<State>),
}

impl GossipRequest {
    pub(crate) fn parse(request: &str) -> Result<Self, Box<dyn Error>> {
        // UPDATE_DATA text protocol parsing
        let update_data_request_regex = Regex::new(r"^UPDATE_DATA=\[(.+)\];$").unwrap();

        if update_data_request_regex.is_match(request) {
            let request_datas = update_data_request_regex.captures(request).unwrap();
            let data = request_datas[1].to_string();
            return Ok(Self::UpdateData(data));
        }

        Err(From::from("invalid request (protocol error)"))
    }
}

#[cfg(test)]
mod gossip_request_protocol_test {
    use super::GossipRequest;

    #[test]
    fn update_data_request_parse_test() {
        let request = "UPDATE_DATA=[Some data ...];";

        let gossip_request = GossipRequest::parse(request).unwrap();

        if let GossipRequest::UpdateData(data) = gossip_request {
            assert_eq!(data, String::from("Some data ..."));
        } else {
            panic!("parsing error");
        }
    }
}
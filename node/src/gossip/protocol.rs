//! Contains abstractions related
//! to the Gossip protocol.

use std::error::Error;

use regex::Regex;

use super::State;

/// Request abstraction for
/// the Gossip protocol.
#[derive(PartialEq, Eq, Debug)]
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

        // SHARE_DATA text protocol parsing
        if request == "SHARE_DATA=NONE;" {
            return Ok(Self::ShareData(None));
        }

        let share_data_request_regex = Regex::new(r"^SHARE_DATA=\[(.+)\]\[([0-9]+)\];$").unwrap();

        if share_data_request_regex.is_match(request) {
            let request_datas = share_data_request_regex.captures(request).unwrap();
            let data = request_datas[1].to_string();
            let timestamp = request_datas[2].parse::<u128>().unwrap();
            return Ok(Self::ShareData(Some(State { data, timestamp })));
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

    #[test]
    fn share_data_request_parse_test() {
        // SHARE_DATA request protocol with NONE
        let request = "SHARE_DATA=NONE;";

        let gossip_request = GossipRequest::parse(request).unwrap();

        assert_eq!(gossip_request, GossipRequest::ShareData(None));

        // SHARE_DATA request protocol with DATA
        let request = "SHARE_DATA=[Some data ...][7851391275623];";

        let gossip_request = GossipRequest::parse(request).unwrap();

        if let GossipRequest::ShareData(data) = gossip_request {
            let data = data.unwrap();
            assert_eq!(data.data, String::from("Some data ..."));
            assert_eq!(data.timestamp, 7851391275623);
        } else {
            panic!("parsing error");
        }
    }
}

//! Contains abstractions related
//! to the Gossip protocol.

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
    /// Parses a string slice into a `GossipRequest`
    /// according to the protocol specification.
    pub(crate) fn parse(request: &str) -> Result<Self, &'static str> {
        // UPDATE_DATA request protocol parsing
        if let Some(gossip_request) = Self::parse_update_data_request_protocol(request) {
            return Ok(gossip_request);
        }

        // SHARE_DATA request protocol parsing
        if let Some(gossip_request) = Self::parse_share_data_request_protocol(request) {
            return Ok(gossip_request);
        }

        Err("invalid request (protocol error)")
    }

    fn parse_update_data_request_protocol(request: &str) -> Option<Self> {
        let update_data_request_regex = Regex::new(r"^UPDATE_DATA=\[(.+)\];$").unwrap();

        if update_data_request_regex.is_match(request) {
            let request_datas = update_data_request_regex.captures(request).unwrap();
            let data = request_datas[1].to_string();
            return Some(Self::UpdateData(data));
        }

        None
    }

    fn parse_share_data_request_protocol(request: &str) -> Option<Self> {
        if request == "SHARE_DATA=NONE;" {
            return Some(Self::ShareData(None));
        }

        let share_data_request_regex = Regex::new(r"^SHARE_DATA=\[(.+)\]\[([0-9]+)\];$").unwrap();

        if share_data_request_regex.is_match(request) {
            let request_datas = share_data_request_regex.captures(request).unwrap();
            let data = request_datas[1].to_string();
            let timestamp = request_datas[2].parse::<u128>().unwrap();
            return Some(Self::ShareData(Some(State { data, timestamp })));
        }

        None
    }
}

#[cfg(test)]
mod gossip_request_protocol_test {
    use super::GossipRequest;

    #[test]
    fn update_data_request_protocol_parse_test() {
        let request = "UPDATE_DATA=[Some data ...];";

        let gossip_request = GossipRequest::parse(request).unwrap();

        if let GossipRequest::UpdateData(data) = gossip_request {
            assert_eq!(data, String::from("Some data ..."));
        } else {
            panic!("parsing error");
        }
    }

    #[test]
    fn share_data_request_protocol_parse_test() {
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

/// Response abstraction for
/// the Gossip protocol.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum GossipResponse {
    Ignore,
    ResponseWithData(State),
}

impl GossipResponse {
    /// Parses a string slice into a `GossipResponse`
    /// according to the protocol specification.
    pub(crate) fn parse(response: &str) -> Result<Self, &'static str> {
        todo!()
    }
}

#[cfg(test)]
mod gossip_response_protocol_test {
    use crate::gossip::protocol::GossipResponse;

    #[test]
    fn ignore_response_protocol_parse_test() {
        let response = "RESPONSE=IGNORE;";

        assert_eq!(
            GossipResponse::parse(response).unwrap(),
            GossipResponse::Ignore
        );
    }
}

//! Contains utilities for the protocol used in the Chord network.
//!
//! A custom and application-specific protocol tailored to the Chord mechanism.

use std::net::SocketAddr;

use regex::Regex;

use super::{Node, SUCCESSOR_LIST_LENGTH};

/// Represents a response for the
/// protocol used in the Chord Network.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ChordResponse {
    Successor(Node),
    SuccessorList([Node; SUCCESSOR_LIST_LENGTH]),
    Predecessor(Option<Node>),
    Active,
    Error(String),
}

impl ChordResponse {
    /// Parses a string slice into a `ChordResponse`
    /// according to the protocol specification.
    pub(crate) fn parse(response: &str) -> Result<Self, &'static str> {
        // SUCCESSOR text protocol parsing
        if let Some(chord_response) = Self::parse_successor_response_protocol(response)? {
            return Ok(chord_response);
        }

        // SUCCESSOR LIST text protocol parsing
        if let Some(chord_response) = Self::parse_successor_list_response_protocol(response)? {
            return Ok(chord_response);
        }

        // PREDECESSOR text protocol parsing
        if let Some(chord_response) = Self::parse_predecessor_response_protocol(response)? {
            return Ok(chord_response);
        }

        // ACTIVE text protocol parsing
        if let Some(gossip_response) = Self::parse_active_response_protocol(response) {
            return Ok(gossip_response);
        }

        // ERROR text protocol parsing
        if let Some(gossip_response) = Self::parse_error_response_protocol(response) {
            return Ok(gossip_response);
        }

        Err("invalid response (protocol error)")
    }

    fn parse_successor_response_protocol(response: &str) -> Result<Option<Self>, &'static str> {
        let successor_response_regex =
            Regex::new(r"^SUCCESSOR=\[([0-9a-f]{32})\]\[([0-9a-f:.\[\]]+)\];$").unwrap();

        if successor_response_regex.is_match(response) {
            let response_datas = successor_response_regex.captures(response).unwrap();
            let successor_id = response_datas[1].to_string();
            let successor_public_addr = response_datas[2]
                .parse::<SocketAddr>()
                .map_err(|_| "invalid response (invalid socket address)")?;

            return Ok(Some(Self::Successor(Node::create_from(
                hex::decode(successor_id).unwrap().try_into().unwrap(),
                successor_public_addr,
            ))));
        }

        Ok(None)
    }

    fn parse_successor_list_response_protocol(
        response: &str,
    ) -> Result<Option<Self>, &'static str> {
        let successor_list_response_regex = Regex::new(
            r"^SUCCESSOR_LIST=\{\[([0-9a-f]{32})\]\[([0-9a-f:.\[\]]+)\],\[([0-9a-f]{32})\]\[([0-9a-f:.\[\]]+)\],\[([0-9a-f]{32})\]\[([0-9a-f:.\[\]]+)\],\[([0-9a-f]{32})\]\[([0-9a-f:.\[\]]+)\],\[([0-9a-f]{32})\]\[([0-9a-f:.\[\]]+)\]\};$"
        ).unwrap();

        if successor_list_response_regex.is_match(response) {
            let response_datas = successor_list_response_regex.captures(response).unwrap();
            let mut successor_list = Vec::new();

            for i in 1..=SUCCESSOR_LIST_LENGTH {
                let successor_id = response_datas[2 * i - 1].to_string();
                let successor_public_addr = response_datas[2 * i]
                    .parse::<SocketAddr>()
                    .map_err(|_| "invalid response (invalid socket address)")?;

                successor_list.push(Node::create_from(
                    hex::decode(successor_id).unwrap().try_into().unwrap(),
                    successor_public_addr,
                ));
            }

            return Ok(Some(Self::SuccessorList(
                successor_list.try_into().unwrap(),
            )));
        }

        Ok(None)
    }

    fn parse_predecessor_response_protocol(response: &str) -> Result<Option<Self>, &'static str> {
        if response == "PREDECESSOR=NONE;" {
            return Ok(Some(Self::Predecessor(None)));
        }

        let predecessor_exist_response_regex =
            Regex::new(r"^PREDECESSOR=\[([0-9a-f]{32})\]\[([0-9a-f:.\[\]]+)\];$").unwrap();

        if predecessor_exist_response_regex.is_match(response) {
            let response_datas = predecessor_exist_response_regex.captures(response).unwrap();
            let predecessor_id = response_datas[1].to_string();
            let predecessor_public_addr = response_datas[2]
                .parse::<SocketAddr>()
                .map_err(|_| "invalid response (invalid socket address)")?;

            return Ok(Some(Self::Predecessor(Some(Node::create_from(
                hex::decode(predecessor_id).unwrap().try_into().unwrap(),
                predecessor_public_addr,
            )))));
        }

        Ok(None)
    }

    fn parse_active_response_protocol(response: &str) -> Option<Self> {
        if response == "ACTIVE;" {
            return Some(Self::Active);
        }
        None
    }

    fn parse_error_response_protocol(response: &str) -> Option<Self> {
        let error_response_regex = Regex::new(r"^ERROR=\[(.+)\];$").unwrap();

        if error_response_regex.is_match(response) {
            let response_datas = error_response_regex.captures(response).unwrap();
            let error_msg = response_datas[1].to_string();
            return Some(Self::Error(error_msg));
        }

        None
    }

    /// Converts the current `ChordResponse` abstraction
    /// into a text-based representation,
    /// according to the protocol specification.
    pub(crate) fn to_protocol_text(&self) -> String {
        match *self {
            Self::Successor(ref successor) => {
                format!(
                    "SUCCESSOR=[{}][{:?}];",
                    successor.get_hash_id(),
                    successor.get_public_addr()
                )
            }
            Self::SuccessorList(ref successors) => {
                let successors_string = successors
                    .iter()
                    .map(|node| format!("[{}][{:?}]", node.get_hash_id(), node.get_public_addr()))
                    .collect::<Vec<_>>()
                    .join(",");

                format!("SUCCESSOR_LIST={{{}}};", successors_string)
            }
            Self::Predecessor(None) => "PREDECESSOR=NONE;".to_string(),
            Self::Predecessor(Some(ref predecessor)) => {
                format!(
                    "PREDECESSOR=[{}][{:?}];",
                    predecessor.get_hash_id(),
                    predecessor.get_public_addr()
                )
            }
            Self::Error(ref err) => format!("ERROR=[{}];", err),
            Self::Active => "ACTIVE;".to_string(),
        }
    }
}

#[cfg(test)]
mod chord_response_protocol_test {
    use std::net::SocketAddr;

    use crate::chord::Node;

    use super::ChordResponse;

    #[test]
    fn successor_ipv4_response_parse_test() {
        let response = "SUCCESSOR=[cf4b19e32ce29fef04468ac9d2a6787d][17.5.7.3:1450];";

        let chord_response = ChordResponse::parse(response).unwrap();

        if let ChordResponse::Successor(node) = chord_response {
            assert_eq!(node.get_hash_id(), "cf4b19e32ce29fef04468ac9d2a6787d");
            assert_eq!(
                node.get_public_addr(),
                "17.5.7.3:1450".parse::<SocketAddr>().unwrap()
            );
        } else {
            panic!("parsing error");
        }
    }

    #[test]
    fn successor_ipv6_response_parse_test() {
        let response = "SUCCESSOR=[cf4b19e32ce29fef04468ac9d2a6787d][[2001:0db8:85a3:0000:0000:8a2e:0370:7334]:8080];";

        let chord_response = ChordResponse::parse(response).unwrap();

        if let ChordResponse::Successor(node) = chord_response {
            assert_eq!(node.get_hash_id(), "cf4b19e32ce29fef04468ac9d2a6787d");
            assert_eq!(
                node.get_public_addr(),
                "[2001:0db8:85a3:0000:0000:8a2e:0370:7334]:8080"
                    .parse::<SocketAddr>()
                    .unwrap()
            );
        } else {
            panic!("parsing error");
        }
    }

    #[test]
    fn sucessor_list_response_parse_test() {
        let response = "SUCCESSOR_LIST={[6e4bfa7e2180a1cf55db0e38c12b9979][[2001:0db8:85a3:0000:0000:8a2e:0370:7334]:8080],[6e4bfa7e2180a1cf55db0e38c12b9979][192.168.42.120:2069],[b2c7f1a82d3452f0a8577f7d3b9e38f5][172.16.8.53:9876],[98e317b512a1391a9e0eabf8e3f1c6b4][[2001:db8::1]:4040],[98e317b512a1391a9e0eabf8e3f1c6b4][10.0.0.33:443]};";

        let chord_response = ChordResponse::parse(response).unwrap();

        if let ChordResponse::SuccessorList(successor_list) = chord_response {
            assert_eq!(
                successor_list[0].get_hash_id(),
                "6e4bfa7e2180a1cf55db0e38c12b9979"
            );
            assert_eq!(
                successor_list[0].get_public_addr(),
                "[2001:0db8:85a3:0000:0000:8a2e:0370:7334]:8080"
                    .parse::<SocketAddr>()
                    .unwrap()
            );

            assert_eq!(
                successor_list[2].get_hash_id(),
                "b2c7f1a82d3452f0a8577f7d3b9e38f5"
            );
            assert_eq!(
                successor_list[2].get_public_addr(),
                "172.16.8.53:9876".parse::<SocketAddr>().unwrap()
            );

            assert_eq!(
                successor_list[4].get_hash_id(),
                "98e317b512a1391a9e0eabf8e3f1c6b4"
            );
            assert_eq!(
                successor_list[4].get_public_addr(),
                "10.0.0.33:443".parse::<SocketAddr>().unwrap()
            );
        } else {
            panic!("parsing error");
        }
    }

    #[test]
    fn predecessor_response_parse_test() {
        // PREDECESSOR with NONE value
        let response = "PREDECESSOR=NONE;";

        assert_eq!(
            ChordResponse::parse(response).unwrap(),
            ChordResponse::Predecessor(None)
        );

        // PREDECESSOR with NODE value
        let response = "PREDECESSOR=[cf4b19e32ce29fef04468ac9d2a6787d][[2001:0db8:85a3:0000:0000:8a2e:0370:7334]:8080];";

        let chord_response = ChordResponse::parse(response).unwrap();

        if let ChordResponse::Predecessor(Some(node)) = chord_response {
            assert_eq!(node.get_hash_id(), "cf4b19e32ce29fef04468ac9d2a6787d");
            assert_eq!(
                node.get_public_addr(),
                "[2001:0db8:85a3:0000:0000:8a2e:0370:7334]:8080"
                    .parse::<SocketAddr>()
                    .unwrap()
            );
        } else {
            panic!("parsing error");
        }
    }

    #[test]
    fn active_response_parse_test() {
        let response = "ACTIVE;";

        assert_eq!(
            ChordResponse::Active,
            ChordResponse::parse(response).unwrap()
        );
    }

    #[test]
    fn error_response_parse_test() {
        let response = "ERROR=[Some error message ...];";

        let chord_response = ChordResponse::parse(response).unwrap();

        if let ChordResponse::Error(err) = chord_response {
            assert_eq!(err, "Some error message ...");
        } else {
            panic!("parsing error");
        }
    }

    #[test]
    fn chord_response_to_protocol_text_test() {
        // SUCCESSOR LIST response abstraction
        // to text-based protocol
        let sucessor_list = [
            Node::create_from(
                hex::decode("6e4bfa7e2180a1cf55db0e38c12b9979")
                    .unwrap()
                    .try_into()
                    .unwrap(),
                "[2001:0db8:85a3:0000:0000:8a2e:0370:7334]:8080"
                    .parse::<SocketAddr>()
                    .unwrap(),
            ),
            Node::create_from(
                hex::decode("6e4bfa7e2180a1cf55db0e38c12b9979")
                    .unwrap()
                    .try_into()
                    .unwrap(),
                "192.168.42.120:2069".parse::<SocketAddr>().unwrap(),
            ),
            Node::create_from(
                hex::decode("b2c7f1a82d3452f0a8577f7d3b9e38f5")
                    .unwrap()
                    .try_into()
                    .unwrap(),
                "172.16.8.53:9876".parse::<SocketAddr>().unwrap(),
            ),
            Node::create_from(
                hex::decode("98e317b512a1391a9e0eabf8e3f1c6b4")
                    .unwrap()
                    .try_into()
                    .unwrap(),
                "[2001:db8::1]:4040".parse::<SocketAddr>().unwrap(),
            ),
            Node::create_from(
                hex::decode("98e317b512a1391a9e0eabf8e3f1c6b4")
                    .unwrap()
                    .try_into()
                    .unwrap(),
                "10.0.0.33:443".parse::<SocketAddr>().unwrap(),
            ),
        ];

        let chord_response = ChordResponse::parse(
            &ChordResponse::SuccessorList(sucessor_list.clone()).to_protocol_text(),
        )
        .unwrap();

        if let ChordResponse::SuccessorList(result_successor_list) = chord_response {
            assert_eq!(result_successor_list, sucessor_list);
        } else {
            panic!("parsing error");
        }

        // SUCCESSOR response abstraction
        // to text-based protocol
        let successor = Node::create_from(
            hex::decode("98e317b512a1391a9e0eabf8e3f1c6b4")
                .unwrap()
                .try_into()
                .unwrap(),
            "10.0.0.33:443".parse::<SocketAddr>().unwrap(),
        );

        let chord_response =
            ChordResponse::parse(&ChordResponse::Successor(successor.clone()).to_protocol_text())
                .unwrap();

        if let ChordResponse::Successor(result_successor) = chord_response {
            assert_eq!(result_successor, successor);
        } else {
            panic!("parsing error");
        }

        // PREDECESSOR response abstraction
        // to text-based protocol
        assert_eq!(
            ChordResponse::parse(&ChordResponse::Predecessor(None).to_protocol_text()).unwrap(),
            ChordResponse::Predecessor(None)
        );

        let predecessor = Node::create_from(
            hex::decode("98e317b512a1391a9e0eabf8e3f1c6b4")
                .unwrap()
                .try_into()
                .unwrap(),
            "10.0.0.33:443".parse::<SocketAddr>().unwrap(),
        );

        let chord_response = ChordResponse::parse(
            &ChordResponse::Predecessor(Some(predecessor.clone())).to_protocol_text(),
        )
        .unwrap();

        if let ChordResponse::Predecessor(Some(result_predecessor)) = chord_response {
            assert_eq!(result_predecessor, predecessor);
        } else {
            panic!("parsing error");
        }

        // ACTIVE response abstraction
        // to text-based protocol
        assert_eq!(ChordResponse::Active.to_protocol_text(), "ACTIVE;");

        // ERROR response abstraction
        // to text-based protocol
        let error_msg = String::from("some error ...");

        let chord_response =
            ChordResponse::parse(&ChordResponse::Error(error_msg.clone()).to_protocol_text())
                .unwrap();

        if let ChordResponse::Error(result_error_msg) = chord_response {
            assert_eq!(result_error_msg, error_msg);
        } else {
            panic!("parsing error");
        }
    }
}

/// Represents a request for the
/// protocol used in the Chord Network.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ChordRequest {
    FindSuccessorOfNode(Node),
    GetSuccessorList,
    GetPredecessor,
    NotificationBy(Node),
    CheckNode,
}

impl ChordRequest {
    /// Parses a string slice into a `ChordRequest`
    /// according to the protocol specification.
    pub(crate) fn parse(request: &str) -> Result<Self, &'static str> {
        // FIND_SUCCESSOR_OF_NODE text protocol parsing
        if let Some(chord_request) = Self::parse_find_successor_of_node_request_protocol(request)? {
            return Ok(chord_request);
        }

        // GET_SUCCESSOR_LIST text protocol parsing
        if let Some(chord_request) = Self::parse_get_successor_list_request_protocol(request) {
            return Ok(chord_request);
        }

        // GET_PREDECESSOR text protocol parsing
        if let Some(chord_request) = Self::parse_get_predecessor_request_protocol(request) {
            return Ok(chord_request);
        }

        // NOTIFICATION_BY text protocol parsing
        if let Some(chord_request) = Self::parse_notification_by_request_protocol(request)? {
            return Ok(chord_request);
        }

        // CHECK_NODE text protocol parsing
        if let Some(chord_request) = Self::parse_check_node_request_protocol(request) {
            return Ok(chord_request);
        }

        Err("invalid request (protocol error)")
    }

    fn parse_find_successor_of_node_request_protocol(
        request: &str,
    ) -> Result<Option<Self>, &'static str> {
        let find_successor_of_node_regex =
            Regex::new(r"^FIND_SUCCESSOR_OF_NODE=\[([0-9a-f]{32})\]\[([0-9a-f:.\[\]]+)\];$")
                .unwrap();

        if find_successor_of_node_regex.is_match(request) {
            let request_datas = find_successor_of_node_regex.captures(request).unwrap();
            let node_id = request_datas[1].to_string();
            let node_public_addr = request_datas[2]
                .parse::<SocketAddr>()
                .map_err(|_| "invalid request (invalid socket address)")?;
            return Ok(Some(ChordRequest::FindSuccessorOfNode(Node::create_from(
                hex::decode(node_id).unwrap().try_into().unwrap(),
                node_public_addr,
            ))));
        }

        Ok(None)
    }

    fn parse_get_successor_list_request_protocol(request: &str) -> Option<Self> {
        if request == "GET_SUCCESSOR_LIST;" {
            return Some(Self::GetSuccessorList);
        }

        None
    }

    fn parse_get_predecessor_request_protocol(request: &str) -> Option<Self> {
        if request == "GET_PREDECESSOR;" {
            return Some(Self::GetPredecessor);
        }

        None
    }

    fn parse_notification_by_request_protocol(request: &str) -> Result<Option<Self>, &'static str> {
        let notification_by_regex =
            Regex::new(r"^NOTIFICATION_BY=\[([0-9a-f]{32})\]\[([0-9a-f:.\[\]]+)\];$").unwrap();

        if notification_by_regex.is_match(request) {
            let request_datas = notification_by_regex.captures(request).unwrap();
            let node_id = request_datas[1].to_string();
            let node_public_addr = request_datas[2]
                .parse::<SocketAddr>()
                .map_err(|_| "invalid request (invalid socket address)")?;

            return Ok(Some(Self::NotificationBy(Node::create_from(
                hex::decode(node_id).unwrap().try_into().unwrap(),
                node_public_addr,
            ))));
        }

        Ok(None)
    }

    fn parse_check_node_request_protocol(request: &str) -> Option<Self> {
        if request == "CHECK_NODE;" {
            return Some(ChordRequest::CheckNode);
        }

        None
    }

    /// Converts the current `ChordRequest` abstraction
    /// into a text-based representation,
    /// according to the protocol specification.
    pub(crate) fn to_protocol_text(&self) -> String {
        match *self {
            Self::FindSuccessorOfNode(ref target_node) => {
                format!(
                    "FIND_SUCCESSOR_OF_NODE=[{}][{:?}];",
                    target_node.get_hash_id(),
                    target_node.get_public_addr()
                )
            }
            Self::GetSuccessorList => "GET_SUCCESSOR_LIST;".to_string(),
            Self::GetPredecessor => "GET_PREDECESSOR;".to_string(),
            Self::NotificationBy(ref node) => {
                format!(
                    "NOTIFICATION_BY=[{}][{:?}];",
                    node.get_hash_id(),
                    node.get_public_addr()
                )
            }
            Self::CheckNode => "CHECK_NODE;".to_string(),
        }
    }
}

#[cfg(test)]
mod chord_request_protocol_test {
    use std::net::SocketAddr;

    use crate::chord::Node;

    use super::ChordRequest;

    #[test]
    fn find_successor_of_node_request_parse_test() {
        let request = "FIND_SUCCESSOR_OF_NODE=[080501321f1d3ab94c90052a1938e7dc][[2001:0db8:85a3:0000:0000:8a2e:0370:7334]:8080];";

        let chord_request = ChordRequest::parse(request).unwrap();

        if let ChordRequest::FindSuccessorOfNode(target_node) = chord_request {
            assert_eq!(
                target_node.get_hash_id(),
                "080501321f1d3ab94c90052a1938e7dc"
            );
            assert_eq!(
                target_node.get_public_addr(),
                "[2001:0db8:85a3:0000:0000:8a2e:0370:7334]:8080"
                    .parse::<SocketAddr>()
                    .unwrap()
            )
        } else {
            panic!("parsing error");
        }
    }

    #[test]
    fn get_successor_list_request_parse_test() {
        let request = "GET_SUCCESSOR_LIST;";

        assert_eq!(
            ChordRequest::parse(request).unwrap(),
            ChordRequest::GetSuccessorList
        );
    }

    #[test]
    fn get_predecessor_request_parse_test() {
        let request = "GET_PREDECESSOR;";

        assert_eq!(
            ChordRequest::parse(request).unwrap(),
            ChordRequest::GetPredecessor
        );
    }

    #[test]
    fn notification_by_request_parse_test() {
        let request = "NOTIFICATION_BY=[080501321f1d3ab94c90052a1938e7dc][[2001:0db8:85a3:0000:0000:8a2e:0370:7334]:8080];";

        let chord_request = ChordRequest::parse(request).unwrap();

        if let ChordRequest::NotificationBy(node) = chord_request {
            assert_eq!(node.get_hash_id(), "080501321f1d3ab94c90052a1938e7dc");
            assert_eq!(
                node.get_public_addr(),
                "[2001:0db8:85a3:0000:0000:8a2e:0370:7334]:8080"
                    .parse::<SocketAddr>()
                    .unwrap()
            )
        } else {
            panic!("parsing error");
        }
    }

    #[test]
    fn check_node_request_parse_test() {
        let request = "CHECK_NODE;";

        assert_eq!(
            ChordRequest::parse(request).unwrap(),
            ChordRequest::CheckNode
        );
    }

    #[test]
    fn chord_request_to_protocol_text_test() {
        let node = Node::create_from(
            hex::decode("080501321f1d3ab94c90052a1938e7dc")
                .unwrap()
                .try_into()
                .unwrap(),
            "[2001:0db8:85a3:0000:0000:8a2e:0370:7334]:8080"
                .parse::<SocketAddr>()
                .unwrap(),
        );

        // FIND_SUCCESSOR_OF_NODE request abstraction
        // to text-based protocol
        let chord_request = ChordRequest::parse(
            &ChordRequest::FindSuccessorOfNode(node.clone()).to_protocol_text(),
        )
        .unwrap();

        if let ChordRequest::FindSuccessorOfNode(target_node) = chord_request {
            assert_eq!(target_node, node)
        } else {
            panic!("parsing error");
        }

        // GET_SUCCESSOR_LIST request abstraction
        // to text-based protocol
        assert_eq!(
            ChordRequest::GetSuccessorList.to_protocol_text(),
            "GET_SUCCESSOR_LIST;"
        );

        // GET_PREDECESSOR request abstraction
        // to text-based protocol
        assert_eq!(
            ChordRequest::GetPredecessor.to_protocol_text(),
            "GET_PREDECESSOR;"
        );

        // NOTIFICATION_BY request abstraction
        // to text-based protocol
        let chord_request =
            ChordRequest::parse(&ChordRequest::NotificationBy(node.clone()).to_protocol_text())
                .unwrap();

        if let ChordRequest::NotificationBy(result_node) = chord_request {
            assert_eq!(result_node, node);
        } else {
            panic!("parsing error");
        }

        // CHECK_NODE request abstraction
        // to text-based protocol
        assert_eq!(ChordRequest::CheckNode.to_protocol_text(), "CHECK_NODE;");
    }
}

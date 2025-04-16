use std::{
    io::{Read, Write},
    net::TcpStream,
    sync::{Arc, RwLock},
};

use crate::{
    chord::{self, protocol::ChordRequest, Node, SUCCESSOR_LIST_LENGTH},
    gossip::{self, protocol::GossipRequest, State},
};

enum Request {
    ChordRequest(ChordRequest),
    GossipRequest(GossipRequest),
}

impl Request {
    fn parse(request: &str) -> Option<Self> {
        if let Ok(chord_request) = ChordRequest::parse(request) {
            return Some(Self::ChordRequest(chord_request));
        }

        if let Ok(gossip_request) = GossipRequest::parse(request) {
            return Some(Self::GossipRequest(gossip_request));
        }

        None
    }
}

pub(crate) fn build_request_handler(
    mut stream: TcpStream,
    self_node: Node,
    self_node_successor_list: Arc<RwLock<[Node; SUCCESSOR_LIST_LENGTH]>>,
    self_node_predecessor: Arc<RwLock<Option<Node>>>,
    self_node_gossip_data: Arc<RwLock<Option<State>>>,
) -> impl FnOnce() + Send + 'static {
    move || {
        let mut request_msg = String::new();

        if stream.read_to_string(&mut request_msg).is_err() {
            return;
        }

        let request = if let Some(request) = Request::parse(&request_msg) {
            request
        } else {
            return;
        };

        match request {
            Request::ChordRequest(chord_request) => {
                let self_node_successor_list = self_node_successor_list.read().unwrap().clone();

                let response = match chord_request {
                    ChordRequest::FindSuccessorOfNode(target_node) => {
                        chord::request_handler::find_successor_of_node_request_handler(
                            self_node,
                            self_node_successor_list,
                            target_node,
                        )
                    }
                    ChordRequest::GetSuccessorList => {
                        chord::request_handler::get_successor_list_request_handler(
                            self_node_successor_list,
                        )
                    }
                    ChordRequest::GetPredecessor => {
                        let self_node_predecessor = self_node_predecessor.read().unwrap().clone();
                        chord::request_handler::get_predecessor_request_handler(
                            self_node_predecessor,
                        )
                    }
                    ChordRequest::CheckNode => chord::request_handler::check_node_request_handler(),
                    ChordRequest::NotificationBy(external_node) => {
                        chord::request_handler::node_notification_request_handler(
                            self_node,
                            self_node_predecessor,
                            self_node_successor_list,
                            external_node,
                        )
                    }
                };

                let _ = stream.write(response.to_protocol_text().as_bytes());
            }
            Request::GossipRequest(gossip_request) => {
                let response = match gossip_request {
                    GossipRequest::UpdateData(received_data) => {
                        gossip::request_handler::update_data_request_handler(
                            self_node_gossip_data,
                            received_data,
                        )
                    }
                    GossipRequest::ShareData(received_data) => {
                        gossip::request_handler::share_data_request_handler(
                            self_node_gossip_data,
                            received_data,
                        )
                    }
                };

                let _ = stream.write(response.to_protocol_text().as_bytes());
            }
        }
    }
}

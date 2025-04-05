//! Responsible for processing and handling various types
//! of requests in the Chord network.

use std::{
    io::{Read, Write},
    net::TcpStream,
    sync::{Arc, RwLock},
};

use super::{
    protocol::{ChordRequest, ChordResponse},
    request_initiator, Node, RING_BIT_LENGTH, SUCCESSOR_LIST_LENGTH,
};

/// Creates a request handler for processing incoming requests
/// in the Chord network, using the provided Chord core component states.
pub(crate) fn build_request_handler(
    mut stream: TcpStream,
    self_node: Node,
    self_node_successor_list: Arc<RwLock<[Node; SUCCESSOR_LIST_LENGTH]>>,
    self_node_finger_table: Arc<RwLock<[Option<Node>; RING_BIT_LENGTH]>>,
    self_node_predecessor: Arc<RwLock<Option<Node>>>,
) -> impl FnOnce() + Send + 'static {
    // A request handler should never panic.
    // This is because request handlers are executed in separate threads.
    // A panic can terminate the thread, potentially causing the entire process to stop.
    move || {
        let mut request_msg = String::new();

        if let Err(err) = stream.read_to_string(&mut request_msg) {
            send_error_response(stream, format!("failed to handle the request: {}", err));
            return;
        };

        let request = match ChordRequest::parse(&request_msg) {
            Ok(request) => request,
            Err(err) => {
                send_error_response(stream, format!("failed to handle the request: {}", err));
                return;
            }
        };

        let self_node_successor_list_value = self_node_successor_list.read().unwrap().clone();
        let self_node_finger_table_value = self_node_finger_table.read().unwrap().clone();
        let self_node_predecessor_value = self_node_predecessor.read().unwrap().clone();

        let response = match request {
            ChordRequest::FindSuccessorOfNode(target_node) => {
                find_successor_of_node_request_handler(
                    self_node,
                    self_node_successor_list_value,
                    self_node_finger_table_value,
                    target_node,
                )
            }
            ChordRequest::GetSuccessorList => {
                get_successor_list_request_handler(self_node_successor_list_value)
            }
            ChordRequest::GetPredecessor => {
                get_predecessor_request_handler(self_node_predecessor_value)
            }
            ChordRequest::NotificationBy(external_node) => node_notification_request_handler(
                self_node,
                self_node_predecessor,
                self_node_successor_list_value,
                external_node,
            ),
            ChordRequest::CheckNode => check_node_request_handler(),
        };

        let _ = stream.write(response.to_protocol_text().as_bytes());
    }
}

fn find_successor_of_node_request_handler(
    self_node: Node,
    self_node_successor_list: [Node; SUCCESSOR_LIST_LENGTH],
    self_node_finger_table: [Option<Node>; RING_BIT_LENGTH],
    target_node: Node,
) -> ChordResponse {
    let self_node_successor = self_node_successor_list[0].clone();

    if target_node.get_ring_position() == self_node.get_ring_position()
        || target_node.get_ring_position() == self_node_successor.get_ring_position()
    {
        return ChordResponse::Error(
            "the node's identifier already exists in the network".to_string(),
        );
    }

    if self_node.get_ring_position() == self_node_successor.get_ring_position() {
        return ChordResponse::Successor(self_node_successor);
    }

    if target_node.is_position_stictly_between(
        self_node.get_ring_position(),
        self_node_successor.get_ring_position(),
    ) {
        return ChordResponse::Successor(self_node_successor);
    }

    let table = if target_node.is_position_stictly_between(
        self_node.get_ring_position(),
        self_node_successor_list[SUCCESSOR_LIST_LENGTH - 1].get_ring_position(),
    ) {
        self_node_successor_list.into_iter().collect::<Vec<_>>()
    } else {
        self_node_finger_table
            .into_iter()
            .filter(|node| node.is_some())
            .map(|node| node.unwrap())
            .collect::<Vec<_>>()
    };

    let mut closest_preceding_node_to_target: Option<Node> = None;

    for entry in table.into_iter().rev() {
        if entry.is_position_stictly_between(
            self_node.get_ring_position(),
            target_node.get_ring_position(),
        ) {
            if let ChordResponse::Active =
                request_initiator::check_remote_node(entry.get_public_addr())
            {
                closest_preceding_node_to_target = Some(entry);
                break;
            }
        }
    }

    request_initiator::find_successor_of_node(
        &target_node,
        closest_preceding_node_to_target.unwrap().get_public_addr(),
    )
}

fn get_successor_list_request_handler(
    self_node_successor_list: [Node; SUCCESSOR_LIST_LENGTH],
) -> ChordResponse {
    ChordResponse::SuccessorList(self_node_successor_list)
}

fn get_predecessor_request_handler(self_node_predecessor: Option<Node>) -> ChordResponse {
    ChordResponse::Predecessor(self_node_predecessor)
}

fn node_notification_request_handler(
    self_node: Node,
    self_node_predecessor: Arc<RwLock<Option<Node>>>,
    self_node_successor_list: [Node; SUCCESSOR_LIST_LENGTH],
    external_node: Node,
) -> ChordResponse {
    let self_node_predecessor_value = self_node_predecessor.read().unwrap().clone();
    let mut self_node_predecessor_lock = self_node_predecessor.write().unwrap();

    match self_node_predecessor_value {
        Some(predecessor)
            if predecessor.get_ring_position() == self_node.get_ring_position()
                || external_node.is_position_stictly_between(
                    predecessor.get_ring_position(),
                    self_node.get_ring_position(),
                ) =>
        {
            *self_node_predecessor_lock = Some(external_node);
        }
        None => {
            *self_node_predecessor_lock = Some(external_node);
        }
        _ => (),
    }

    ChordResponse::SuccessorList(self_node_successor_list)
}

fn check_node_request_handler() -> ChordResponse {
    ChordResponse::Active
}

fn send_error_response(mut stream: TcpStream, error_msg: String) {
    let response = ChordResponse::Error(error_msg).to_protocol_text();
    let _ = stream.write(response.as_bytes());
}

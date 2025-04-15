//! Responsible for processing and handling various types
//! of requests in the Chord network.

use std::sync::{Arc, RwLock};

use super::{protocol::ChordResponse, request_initiator, Node, SUCCESSOR_LIST_LENGTH};

pub(crate) fn find_successor_of_node_request_handler(
    self_node: Node,
    self_node_successor_list: [Node; SUCCESSOR_LIST_LENGTH],
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

    let mut closest_preceding_node_to_target: Option<Node> = None;

    for entry in self_node_successor_list.into_iter().rev() {
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

pub(crate) fn get_successor_list_request_handler(
    self_node_successor_list: [Node; SUCCESSOR_LIST_LENGTH],
) -> ChordResponse {
    ChordResponse::SuccessorList(self_node_successor_list)
}

pub(crate) fn get_predecessor_request_handler(
    self_node_predecessor: Option<Node>,
) -> ChordResponse {
    ChordResponse::Predecessor(self_node_predecessor)
}

pub(crate) fn node_notification_request_handler(
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

pub(crate) fn check_node_request_handler() -> ChordResponse {
    ChordResponse::Active
}

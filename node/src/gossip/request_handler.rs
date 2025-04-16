use std::{
    sync::{Arc, RwLock},
    time::{SystemTime, UNIX_EPOCH},
};

use super::{protocol::GossipResponse, State};

pub(crate) fn update_data_request_handler(
    self_node_gossip_data: Arc<RwLock<Option<State>>>,
    received_data: String,
) -> GossipResponse {
    let data = State {
        data: received_data,
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis(),
    };

    let mut self_node_gossip_data_lock = self_node_gossip_data.write().unwrap();
    *self_node_gossip_data_lock = Some(data);

    GossipResponse::Ignore
}

pub(crate) fn share_data_request_handler(
    self_node_gossip_data: Arc<RwLock<Option<State>>>,
    received_data: Option<State>,
) -> GossipResponse {
    let self_node_gossip_data_content = self_node_gossip_data.read().unwrap().clone();

    if received_data.is_none() && self_node_gossip_data_content.is_some() {
        return GossipResponse::ResponseWithData(self_node_gossip_data_content.unwrap());
    }

    if received_data.is_some() && self_node_gossip_data_content.is_none() {
        let mut self_node_gossip_data_lock = self_node_gossip_data.write().unwrap();
        *self_node_gossip_data_lock = Some(received_data.unwrap());
        return GossipResponse::Ignore;
    }

    if received_data.is_some() && self_node_gossip_data_content.is_some() {
        let received_data = received_data.unwrap();
        let self_node_gossip_data_content = self_node_gossip_data_content.unwrap();

        if received_data.timestamp > self_node_gossip_data_content.timestamp {
            let mut self_node_gossip_data_lock = self_node_gossip_data.write().unwrap();
            *self_node_gossip_data_lock = Some(received_data);
            return GossipResponse::Ignore;
        }

        if received_data.timestamp < self_node_gossip_data_content.timestamp {
            return GossipResponse::ResponseWithData(self_node_gossip_data_content);
        }
    }

    GossipResponse::Ignore
}

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

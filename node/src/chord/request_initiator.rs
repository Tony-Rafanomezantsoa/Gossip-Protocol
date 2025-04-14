//! Responsible for initiating requests
//! in the Chord network.

use std::{
    io::{Read, Write},
    net::{Shutdown, SocketAddr, TcpStream},
    time::Duration,
};

use super::{
    protocol::{ChordRequest, ChordResponse},
    Node,
};

fn init_chord_request(remote_addr: SocketAddr, request: ChordRequest) -> ChordResponse {
    let mut request_stream = match TcpStream::connect(remote_addr) {
        Ok(stream) => stream,
        Err(err) => return ChordResponse::Error(err.to_string()),
    };

    let request_msg = request.to_protocol_text();

    if let Err(err) = request_stream.write(request_msg.as_bytes()) {
        return ChordResponse::Error(err.to_string());
    }

    if let Err(err) = request_stream.shutdown(Shutdown::Write) {
        return ChordResponse::Error(err.to_string());
    }

    request_stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();

    let mut response_msg = String::new();

    if let Err(err) = request_stream.read_to_string(&mut response_msg) {
        return ChordResponse::Error(err.to_string());
    }

    match ChordResponse::parse(&response_msg) {
        Ok(response) => response,
        Err(err) => ChordResponse::Error(err.to_string()),
    }
}

/// Sends a request to `remote_addr`
/// (a Chord node) to locate the successor
/// of the `target_node` in the network
/// and returns a `ChordResponse`.
pub(crate) fn find_successor_of_node(target_node: &Node, remote_addr: SocketAddr) -> ChordResponse {
    init_chord_request(
        remote_addr,
        ChordRequest::FindSuccessorOfNode(target_node.clone()),
    )
}

/// Sends a request to `remote_addr`
/// to retrieve the successor list
/// of this remote node (a Chord node)
/// and returns a `ChordResponse`.
pub(crate) fn get_successor_list(remote_addr: SocketAddr) -> ChordResponse {
    init_chord_request(remote_addr, ChordRequest::GetSuccessorList)
}

/// Sends a request to `remote_addr`
/// to retrieve the predecessor
/// of this remote node (a Chord node)
/// and returns a `ChordResponse`.
pub(crate) fn get_predecessor(remote_addr: SocketAddr) -> ChordResponse {
    init_chord_request(remote_addr, ChordRequest::GetPredecessor)
}

/// Notifies a remote node about
/// the existence of `self_node` in the network,
/// and returns a `ChordResponse`.
pub(crate) fn notify_remote_node(self_node: &Node, remote_addr: SocketAddr) -> ChordResponse {
    init_chord_request(remote_addr, ChordRequest::NotificationBy(self_node.clone()))
}

/// Sends a request to `remote_addr` to ckeck
/// if this remote node (a Chord node) is active.
pub(crate) fn check_remote_node(remote_addr: SocketAddr) -> ChordResponse {
    init_chord_request(remote_addr, ChordRequest::CheckNode)
}

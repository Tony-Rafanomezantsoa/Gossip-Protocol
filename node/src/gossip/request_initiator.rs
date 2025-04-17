use std::{io::{Write, Read}, net::{Shutdown, SocketAddr, TcpStream}, time::Duration};

use super::{protocol::GossipResponse, State};

/// Initiates a request to `remote_addr` to share `data`.
pub(crate) fn share_data(data: Option<State>, remote_addr: SocketAddr) -> GossipResponse {
    let mut request_stream = match TcpStream::connect(remote_addr) {
        Ok(stream) => stream,
        Err(_) => return GossipResponse::Ignore,
    };

    let request_msg = match data {
        Some(state) => format!("SHARE_DATA=[{}][{}];", state.data, state.timestamp),
        None => "SHARE_DATA=NONE;".to_string(),
    };

    if request_stream.write(request_msg.as_bytes()).is_err() {
        return GossipResponse::Ignore;
    }

    if request_stream.shutdown(Shutdown::Write).is_err() {
        return GossipResponse::Ignore;
    }

    request_stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();

    let mut response_msg = String::new();

    if request_stream.read_to_string(&mut response_msg).is_err() {
        return GossipResponse::Ignore;
    }

    match GossipResponse::parse(&response_msg) {
        Ok(response) => response,
        Err(_) => GossipResponse::Ignore,
    }
}
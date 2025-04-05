//! Contains utilities related
//! to the Chord network.

use std::{
    error::Error,
    io::{self, Write},
    net::{IpAddr, SocketAddr, TcpListener, TcpStream},
};

use protocol::ChordResponse;

use crate::cli::Args;

pub mod protocol;
pub mod request_handler;
pub mod request_initiator;
pub mod utils;

pub const RING_BIT_LENGTH: usize = 128;
pub const RING_BYTE_LENGTH: usize = RING_BIT_LENGTH / 8;
pub const SUCCESSOR_LIST_LENGTH: usize = 5;
pub const RING_MAX_POSITION: u128 = u128::MAX;

/// Contains information about a Chord Node,
/// including identifier and the public socket
/// address for accessing the node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Node {
    id: [u8; RING_BYTE_LENGTH],
    public_addr: SocketAddr,
}

impl Node {
    /// Creates a new Chord node with the given
    /// public socket address for accessing the node.
    ///
    /// The node's identifier is an MD5 hash
    /// of the public socket address.
    pub(crate) fn new(public_addr: SocketAddr) -> Self {
        Self {
            id: Self::generate_identifier(public_addr),
            public_addr,
        }
    }

    /// Creates a Chord node using the provided
    /// identifier and public socket address.
    pub(crate) fn create_from(id: [u8; RING_BYTE_LENGTH], public_addr: SocketAddr) -> Self {
        Self { id, public_addr }
    }

    /// Returns the position of the current
    /// node in the Chord ring.
    pub(crate) fn get_ring_position(&self) -> u128 {
        u128::from_be_bytes(self.id)
    }

    /// Returns the current node's identifier
    /// as a hash string (hexadecimal format).
    pub(crate) fn get_hash_id(&self) -> String {
        hex::encode(self.id)
    }

    /// Returns the current node's public
    /// socket address.
    pub(crate) fn get_public_addr(&self) -> SocketAddr {
        self.public_addr
    }

    /// Generates an identifier, in raw bytes format,
    /// for a Chord node, by hashing the given public
    /// socket address with MD5 hash function.
    pub(crate) fn generate_identifier(public_addr: SocketAddr) -> [u8; 16] {
        let mut socket_addr_bytes = Vec::new();

        match public_addr.ip() {
            IpAddr::V4(ip_v4) => socket_addr_bytes.extend_from_slice(&ip_v4.octets()),
            IpAddr::V6(ip_v6) => socket_addr_bytes.extend_from_slice(&ip_v6.octets()),
        }

        socket_addr_bytes.extend_from_slice(&public_addr.port().to_be_bytes());

        let socket_addr_hash = md5::compute(&socket_addr_bytes);

        socket_addr_hash.0
    }

    /// Checks if the current node's position is strictly between `start` and `end`
    /// in a circular range on the Chord ring.
    ///
    /// - `true` if the position is strictly between `start` and `end` (in a clockwise direction).
    /// - `false` otherwise.
    ///
    /// Return `false` if `start` and `end` are equal.
    pub(crate) fn is_position_stictly_between(&self, start: u128, end: u128) -> bool {
        if start > end {
            return self.get_ring_position() > start
                && self.get_ring_position() <= RING_MAX_POSITION
                || self.get_ring_position() < end;
        }

        if start < end {
            return self.get_ring_position() > start && self.get_ring_position() < end;
        }

        false
    }
}

/// Initializes the core components of the current node
/// `self_node`, including successor list
/// and finger table based on the provided argument.
pub(crate) fn initialize_self_node_core_components(
    self_node: &Node,
    args: &Args,
) -> Result<
    (
        [Node; SUCCESSOR_LIST_LENGTH],
        [Option<Node>; RING_BIT_LENGTH],
    ),
    Box<dyn Error>,
> {
    match *args {
        Args::Init {
            self_port: _,
            public_addr: _,
        } => {
            let sucessor_list: [Node; SUCCESSOR_LIST_LENGTH] =
                std::array::from_fn(|_| self_node.clone());

            let finger_table: [Option<Node>; RING_BIT_LENGTH] = std::array::from_fn(|i| {
                if i == 0 {
                    Some(self_node.clone())
                } else {
                    None
                }
            });

            Ok((sucessor_list, finger_table))
        }
        Args::Join {
            self_port: _,
            public_addr: _,
            remote_addr,
        } => {
            let successor = match request_initiator::find_successor_of_node(self_node, remote_addr) {
                ChordResponse::Successor(node) => node,
                ChordResponse::Error(err) => return  Err(From::from(format!("failed to locate the successor of node [{:?}]: {}", self_node.get_public_addr(), err))),
                _ => return  Err(From::from(format!("failed to locate the successor of node [{:?}]: invalid response (protocol error)", self_node.get_public_addr()))),
            };

            let remote_successor_list = match request_initiator::get_successor_list(successor.get_public_addr()) {
                ChordResponse::SuccessorList(successor_list) => successor_list,
                ChordResponse::Error(err) => return Err(From::from(format!("failed to retrieve the successor list of the remote node [{:?}]: {}", successor.get_public_addr(), err))),
                _ => return Err(From::from(format!("failed to retrieve the successor list of the remote node [{:?}]: invalid response (protocol error)", successor.get_public_addr()))),
            };

            let mut successor_list = Vec::new();
            successor_list.push(successor.clone());
            successor_list.extend_from_slice(&remote_successor_list[0..4]);

            let finger_table: [Option<Node>; RING_BIT_LENGTH] = std::array::from_fn(|i| {
                if i == 0 {
                    Some(successor.clone())
                } else {
                    None
                }
            });

            Ok((successor_list.try_into().unwrap(), finger_table))
        }
    }
}

/// Verifies if the current node's (`self_node`) public socket
/// address refers to the specified local listener (server).
pub(crate) fn verify_self_node_public_addr(
    self_node_public_addr: SocketAddr,
    local_listener: &TcpListener,
) -> Result<(), io::Error> {
    let mut request_stream = TcpStream::connect(self_node_public_addr)?;

    request_stream.write(&[])?;

    local_listener.set_nonblocking(true)?;

    local_listener.accept()?;

    local_listener.set_nonblocking(false)?;

    Ok(())
}

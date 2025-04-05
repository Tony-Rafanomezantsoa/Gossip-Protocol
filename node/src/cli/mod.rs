//! Utilities for the Command Line Interface (CLI)
//! that represents a Chord node.

use std::{env, error::Error, net::SocketAddr};

/// Contains differents arguments,
/// required to run a Chord node.
#[derive(Debug, PartialEq, Eq)]
pub enum Args {
    /// Used to initiate a new Chord network.
    Init {
        self_port: u16,
        public_addr: SocketAddr,
    },
    /// Used to join an existing Chord network.
    Join {
        self_port: u16,
        public_addr: SocketAddr,
        remote_addr: SocketAddr,
    },
}

impl Args {
    /// Parses all received arguments, performs types
    /// verification and build `Args` instance.
    pub fn parse() -> Result<Self, Box<dyn Error>> {
        let mut args = env::args().skip(1);
        let action = args.next().ok_or("invalid argument(s)")?;

        if action != "init" && action != "join" {
            return Err(From::from("invalid argument(s)"));
        }

        let self_port_arg = args.next().ok_or("self-port argument is missing")?;
        let self_port_value = self_port_arg.split("self-port=").last().unwrap(); // Safe unwrap
        let self_port = self_port_value
            .parse::<u16>()
            .map_err(|_| "self-port argument is missing or invalid")?;

        let public_addr_arg = args.next().ok_or("public-addr argument is missing")?;
        let public_addr_value = public_addr_arg.split("public-addr=").last().unwrap(); // Safe unwrap
        let public_addr = public_addr_value
            .parse::<SocketAddr>()
            .map_err(|_| "public-addr argument is missing or invalid")?;

        if action == "init" {
            return Ok(Self::Init {
                self_port,
                public_addr,
            });
        }

        let remote_addr_arg = args.next().ok_or("remote-addr argument is missing")?;
        let remote_addr_value = remote_addr_arg.split("remote-addr=").last().unwrap(); // Safe unwrap
        let remote_addr = remote_addr_value
            .parse::<SocketAddr>()
            .map_err(|_| "remote-addr argument is missing or invalid")?;

        Ok(Self::Join {
            self_port,
            public_addr,
            remote_addr,
        })
    }

    /// Gets the value of the `self-port` argument.
    pub fn get_self_port(&self) -> u16 {
        match *self {
            Self::Init {
                self_port,
                public_addr: _,
            } => self_port,
            Self::Join {
                self_port,
                public_addr: _,
                remote_addr: _,
            } => self_port,
        }
    }

    /// Gets the value of the `public-addr` argument.
    pub fn get_public_addr(&self) -> SocketAddr {
        match *self {
            Self::Init {
                self_port: _,
                public_addr,
            } => public_addr,
            Self::Join {
                self_port: _,
                public_addr,
                remote_addr: _,
            } => public_addr,
        }
    }

    /// Gets the value of the `remote-addr` argument.
    /// Only available with join.
    pub fn get_remote_addr(&self) -> Option<SocketAddr> {
        match *self {
            Self::Init {
                self_port: _,
                public_addr: _,
            } => None,
            Self::Join {
                self_port: _,
                public_addr: _,
                remote_addr,
            } => Some(remote_addr),
        }
    }
}

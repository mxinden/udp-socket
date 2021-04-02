use std::net::{IpAddr, Ipv6Addr, SocketAddr};

/// The capabilities a UDP socket supports on a certain platform.
#[derive(Clone, Copy, Debug)]
pub struct UdpCapabilities {
    /// The maximum amount of segments which can be transmitted if a platform
    /// supports Generic Send Offload (GSO).
    /// This is 1 if the platform doesn't support GSO.
    pub max_gso_segments: usize,
}

/// An outgoing packet
#[derive(Debug)]
pub struct Transmit {
    /// The socket this datagram should be sent to.
    pub destination: SocketAddr,
    /// Explicit congestion notification bits to set on the packet.
    pub ecn: Option<EcnCodepoint>,
    /// Contents of the datagram.
    pub contents: Vec<u8>,
    /// The segment size if this transmission contains multiple datagrams.
    /// This is `None` if the transmit only contains a single datgram.
    pub segment_size: Option<usize>,
    /// Optional source IP address for the datagram.
    pub src_ip: Option<IpAddr>,
}

/// An incoming packet
#[derive(Clone, Copy, Debug)]
pub struct RecvMeta {
    /// The socket this datagram was sent from.
    pub source: SocketAddr,
    /// Length of the payload of the packet.
    pub len: usize,
    /// Explicit congestion notification bits set on the packet.
    pub ecn: Option<EcnCodepoint>,
    /// Optional destination IP address of the datagram.
    pub dst_ip: Option<IpAddr>,
}

impl Default for RecvMeta {
    fn default() -> Self {
        Self {
            source: SocketAddr::new(Ipv6Addr::UNSPECIFIED.into(), 0),
            len: 0,
            ecn: None,
            dst_ip: None,
        }
    }
}

/// Explicit congestion notification codepoint
#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum EcnCodepoint {
    #[doc(hidden)]
    ECT0 = 0b10,
    #[doc(hidden)]
    ECT1 = 0b01,
    #[doc(hidden)]
    CE = 0b11,
}

impl EcnCodepoint {
    /// Create new object from the given bits
    pub fn from_bits(x: u8) -> Option<Self> {
        use self::EcnCodepoint::*;
        Some(match x & 0b11 {
            0b10 => ECT0,
            0b01 => ECT1,
            0b11 => CE,
            _ => {
                return None;
            }
        })
    }
}

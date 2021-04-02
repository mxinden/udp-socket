#[cfg(unix)]
mod cmsg;
mod proto;
mod socket;
#[cfg(unix)]
mod unix;

pub use proto::{EcnCodepoint, RecvMeta, Transmit, UdpCapabilities};
pub use socket::UdpSocket;

/// Number of UDP packets to send/receive at a  time.
pub const BATCH_SIZE: usize = {
    if cfg!(target_os = "linux") {
        // Chosen somewhat arbitrarily; might benefit from additional tuning.
        32
    } else {
        1
    }
};

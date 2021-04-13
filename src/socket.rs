use crate::proto::{RecvMeta, SocketType, Transmit, UdpCapabilities};
use async_io::Async;
use futures_lite::future::poll_fn;
use std::io::{IoSliceMut, Result};
use std::net::SocketAddr;
use std::task::{Context, Poll};

#[derive(Debug)]
pub struct UdpSocket {
    inner: Async<std::net::UdpSocket>,
    ty: SocketType,
}

impl UdpSocket {
    pub fn capabilities() -> Result<UdpCapabilities> {
        Ok(UdpCapabilities {
            max_gso_segments: if cfg!(unix) {
                crate::unix::max_gso_segments()?
            } else {
                1
            },
        })
    }

    pub fn bind(addr: SocketAddr) -> Result<Self> {
        let socket = std::net::UdpSocket::bind(addr)?;
        let ty = if cfg!(unix) {
            crate::unix::init(&socket)?
        } else if addr.is_ipv4() {
            SocketType::Ipv4
        } else {
            SocketType::Ipv6Only
        };
        Ok(Self {
            inner: Async::new(socket)?,
            ty,
        })
    }

    pub fn socket_type(&self) -> SocketType {
        self.ty
    }

    pub fn local_addr(&self) -> Result<SocketAddr> {
        self.inner.get_ref().local_addr()
    }

    pub fn ttl(&self) -> Result<u8> {
        let ttl = self.inner.get_ref().ttl()?;
        Ok(ttl as u8)
    }

    pub fn set_ttl(&self, ttl: u8) -> Result<()> {
        self.inner.get_ref().set_ttl(ttl as u32)
    }

    pub fn poll_send(&self, cx: &mut Context, transmits: &[Transmit]) -> Poll<Result<usize>> {
        match self.inner.poll_writable(cx) {
            Poll::Ready(Ok(())) => {}
            Poll::Pending => return Poll::Pending,
            Poll::Ready(Err(err)) => return Poll::Ready(Err(err)),
        }
        let socket = self.inner.get_ref();
        let res = if cfg!(unix) {
            crate::unix::send(socket, transmits)
        } else {
            fallback_send(socket, transmits)
        };
        match res {
            Ok(len) => Poll::Ready(Ok(len)),
            Err(err) => Poll::Ready(Err(err)),
        }
    }

    pub fn poll_recv(
        &self,
        cx: &mut Context,
        buffers: &mut [IoSliceMut<'_>],
        meta: &mut [RecvMeta],
    ) -> Poll<Result<usize>> {
        match self.inner.poll_readable(cx) {
            Poll::Ready(Ok(())) => {}
            Poll::Pending => return Poll::Pending,
            Poll::Ready(Err(err)) => return Poll::Ready(Err(err)),
        }
        let socket = self.inner.get_ref();
        let res = if cfg!(unix) {
            crate::unix::recv(socket, buffers, meta)
        } else {
            fallback_recv(socket, buffers, meta)
        };
        Poll::Ready(res)
    }

    pub async fn send(&self, transmits: &[Transmit]) -> Result<usize> {
        let mut i = 0;
        while i < transmits.len() {
            i += poll_fn(|cx| self.poll_send(cx, &transmits[i..])).await?;
        }
        Ok(i)
    }

    pub async fn recv(
        &self,
        buffers: &mut [IoSliceMut<'_>],
        meta: &mut [RecvMeta],
    ) -> Result<usize> {
        poll_fn(|cx| self.poll_recv(cx, buffers, meta)).await
    }
}

fn fallback_send(socket: &std::net::UdpSocket, transmits: &[Transmit]) -> Result<usize> {
    let mut sent = 0;
    for transmit in transmits {
        match socket.send_to(&transmit.contents, &transmit.destination) {
            Ok(_) => {
                sent += 1;
            }
            Err(_) if sent != 0 => {
                // We need to report that some packets were sent in this case, so we rely on
                // errors being either harmlessly transient (in the case of WouldBlock) or
                // recurring on the next call.
                return Ok(sent);
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
    Ok(sent)
}

fn fallback_recv(
    socket: &std::net::UdpSocket,
    buffers: &mut [IoSliceMut<'_>],
    meta: &mut [RecvMeta],
) -> Result<usize> {
    let (len, source) = socket.recv_from(&mut buffers[0])?;
    meta[0] = RecvMeta {
        source,
        len,
        ecn: None,
        dst_ip: None,
    };
    Ok(1)
}

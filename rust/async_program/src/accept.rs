use std::{
    future::Future,
    io::BufWriter,
    net::TcpStream,
    os::{fd::AsRawFd, unix::net::SocketAddr},
    pin::Pin,
    task::{Context, Poll},
};

use nix::sys::epoll::EpollFlags;

use crate::async_listner::AsyncListner;

pub struct Accept<'a> {
    listener: &'a AsyncListner,
}

impl<'a> Future for Accept<'a> {
    type Output = (AsyncReader, BufWriter<TcpStream>, SocketAddr);

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.listener.listener.accept() {
            Ok((stream, addr)) => {
                let stream0 = stream.try_clone().unwrap();
                Poll::Ready((self.listener.selector.clone(), BufWriter::new(stream), addr))
            }
            Err(err) => {
                if err.kind() == std::io::ErrorKind::WouldBlock {
                    self.listener.selector.register(
                        EpollFlags::EPOLLIN,
                        self.listener.listener.as_raw_fd(),
                        cx.waker().clone(),
                    );
                    Poll::Pending
                } else {
                    panic!("accept: {}", err)
                }
            }
        }
    }
}

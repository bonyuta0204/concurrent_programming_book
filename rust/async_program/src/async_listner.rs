use std::{net::TcpListener, os::fd::AsRawFd, sync::Arc};

use crate::io_selector::IOSelector;

pub struct AsyncListner {
    pub listener: TcpListener,
    pub selector: Arc<IOSelector>,
}

impl AsyncListner {
    fn listen(addr: &str, selector: Arc<IOSelector>) -> AsyncListner {
        let liestenr = TcpListener::bind(addr).unwrap();

        liestenr.set_nonblocking(true).unwrap();

        AsyncListner {
            listener: liestenr,
            selector: selector,
        }
    }

    fn accept(&self) -> Accept {
        Accept { listener: self }
    }
}

impl Drop for AsyncListner {
    fn drop(&mut self) {
        self.selector.unregister(self.listener.as_raw_fd());
    }
}

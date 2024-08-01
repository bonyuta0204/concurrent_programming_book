use std::{
    collections::{HashMap, VecDeque},
    os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd},
    sync::{Arc, Mutex},
    task::Waker,
};

use nix::{
    errno::Errno,
    sys::{
        epoll::{
            epoll_create1, epoll_ctl, epoll_wait, EpollCreateFlags, EpollEvent, EpollFlags, EpollOp,
        },
        eventfd::{eventfd, EfdFlags},
    },
    unistd::write,
};

fn write_eventfd(fd: RawFd, n: usize) {
    let ptr = &n as *const usize as *const u8;

    let val = unsafe { std::slice::from_raw_parts(ptr, std::mem::size_of_val(&n)) };
    
    let fd = unsafe{ OwnedFd::from_raw_fd(fd)}

    write(&fd, &val).unwrap();
}

enum IOOps {
    ADD(EpollFlags, RawFd, Waker),
    REMOVE(RawFd),
}

struct IOSelector {
    wakers: Mutex<HashMap<RawFd, Waker>>,
    queue: Mutex<VecDeque<IOOps>>,
    epfd: RawFd,
    event: RawFd,
}

impl IOSelector {
    fn new() -> Arc<Self> {
        let s = IOSelector {
            wakers: Mutex::new(HashMap::new()),
            queue: Mutex::new(VecDeque::new()),
            epfd: epoll_create1(EpollCreateFlags::empty()).unwrap(),
            event: eventfd(0, EfdFlags::empty()).unwrap().as_raw_fd(),
        };

        let result = Arc::new(s);
        let s = result.clone();

        std::thread::spawn(move || s.select());

        result
    }

    fn add_event(
        &self,
        flag: EpollFlags,
        fd: RawFd,
        waker: Waker,
        wakers: &mut HashMap<RawFd, Waker>,
    ) {
        let epoll_add = EpollOp::EpollCtlAdd;
        let epoll_mod = EpollOp::EpollCtlMod;
        let epoll_one = EpollFlags::EPOLLONESHOT;

        let mut ev = EpollEvent::new(flag | epoll_one, fd as u64);

        if let Err(err) = epoll_ctl(self.epfd, epoll_add, fd, &mut ev) {
            match err {
                Errno::EEXIST => epoll_ctl(self.epfd, epoll_mod, fd, &mut ev).unwrap(),
                _ => {
                    panic!("epoll_ctl: {}", err);
                }
            }
        }

        assert!(!wakers.contains_key(&fd));
        wakers.insert(fd, waker);
    }

    fn rm_event(&self, fd: RawFd, wakers: &mut HashMap<RawFd, Waker>) {
        let epoll_del = EpollOp::EpollCtlDel;
        let mut ev = EpollEvent::new(EpollFlags::empty(), fd as u64);
        epoll_ctl(self.epfd, epoll_del, fd, &mut ev).ok();
        wakers.remove(&fd);
    }

    fn select(&self) {
        let epoll_in = EpollFlags::EPOLLIN;
        let epoll_add = EpollOp::EpollCtlAdd;

        let mut ev = EpollEvent::new(epoll_in, self.event as u64);

        epoll_ctl(self.epfd, epoll_add, self.event, &mut ev).unwrap();

        let mut events = vec![EpollEvent::empty(); 1024];

        while let Ok(nfds) = epoll_wait(self.epfd, &mut events, -1) {
            let mut t = self.wakers.lock().unwrap();

            for n in 0..nfds {
                if events[n].data() == self.event as u64 {
                    let mut q = self.queue.lock().unwrap();
                    while let Some(op) = q.pop_front() {
                        match op {
                            IOOps::ADD(flag, fd, waker) => self.add_event(flag, fd, waker, &mut t),
                            IOOps::REMOVE(fd) => self.rm_event(fd, &mut t),
                        }
                    }
                } else {
                    let data = events[n].data() as i32;
                    let waker = t.remove(&data).unwrap();
                    waker.wake_by_ref();
                }
            }
        }
    }

    fn register(&self, flags: EpollFlags, fd: RawFd, waker: Waker) {
        let mut q = self.queue.lock().unwrap();
        q.push_back(IOOps::ADD(flags, fd, waker));
        write_eventfd(self.event, 1);
    }

    fn unregister(&self, fd: RawFd) {
        let mut q = self.queue.lock().unwrap();
        q.push_back(IOOps::REMOVE(fd));
        write_eventfd(self.event, 1);
    }
}

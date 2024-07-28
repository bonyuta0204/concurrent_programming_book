use nix::sys::epoll::EpollFlags;
fn main() {
    println!("Hello, world!");
    let epoll_in = EpollFlags::EPOLLIN;
    let epoll_out: = EpollFlags::EPOLLOUT;

    println!("epoll_in: {:#?}", epoll_in);
}

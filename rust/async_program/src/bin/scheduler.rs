use futures::{executor, FutureExt};
use std::{
    pin::Pin,
    sync::{
        mpsc::{sync_channel, Receiver, SyncSender},
        Arc, Mutex,
    },
    task::{Context, Poll},
};

use futures::{
    future::BoxFuture,
    task::{waker_ref, ArcWake},
    Future,
};

struct Hello {
    state: StateHello,
}

enum StateHello {
    HELLO,
    WORLD,
    END,
}

impl Hello {
    fn new() -> Self {
        Hello {
            state: StateHello::HELLO,
        }
    }
}

impl Future for Hello {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        match (*self).state {
            StateHello::HELLO => {
                println!("Hello");
                (*self).state = StateHello::WORLD;
                cx.waker().wake_by_ref();
                Poll::Pending
            }
            StateHello::WORLD => {
                println!("WORLD");
                (*self).state = StateHello::END;
                cx.waker().wake_by_ref();
                Poll::Pending
            }
            StateHello::END => Poll::Ready(()),
        }
    }
}

fn main() {
    let executor = Executor::new();
    executor.get_spawner().spawn(Hello::new());
    executor.run()
}

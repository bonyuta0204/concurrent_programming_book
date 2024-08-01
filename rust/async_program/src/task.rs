use std::sync::{mpsc::SyncSender, Arc, Mutex};

use futures::{future::BoxFuture, task::ArcWake};

pub struct Task {
    pub future: Mutex<BoxFuture<'static, ()>>,
    pub sender: SyncSender<Arc<Task>>,
}

impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        let self0 = arc_self.clone();
        arc_self.sender.send(self0).unwrap();
    }
}

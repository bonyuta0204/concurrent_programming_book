use std::{
    future::Future,
    sync::{mpsc::SyncSender, Arc, Mutex},
};

use crate::task::Task;
use futures::FutureExt;

pub struct Spawner {
    pub sender: SyncSender<Arc<Task>>,
}

impl Spawner {
    fn spawn(&self, future: impl Future<Output = ()> + 'static + Send) {
        let future = future.boxed();
        let task = Arc::new(Task {
            future: Mutex::new(future),
            sender: self.sender.clone(),
        });
        self.sender.send(task).unwrap();
    }
}

use std::future::Future;

use tokio::{runtime::Handle as AsyncHandle, task::JoinHandle};
use tokio_util::{sync::CancellationToken, task::TaskTracker};

#[derive(Clone)]
pub struct Context {
    handle: AsyncHandle,
    pub ct: CancellationToken,
    pub tt: TaskTracker,
}

impl Context {
    pub fn new(handle: &AsyncHandle) -> Self {
        Self {
            handle: handle.clone(),
            ct:     CancellationToken::new(),
            tt:     TaskTracker::new(),
        }
    }

    pub fn spawn<F>(&self, task: F) -> JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.tt.spawn_on(task, &self.handle)
    }

    pub fn spawn_blocking<F, T>(&self, task: F) -> JoinHandle<T>
    where
        F: FnOnce() -> T,
        F: Send + 'static,
        T: Send + 'static,
    {
        self.tt.spawn_blocking_on(task, &self.handle)
    }

    pub fn cancel(&self) {
        self.ct.cancel();
    }

    pub fn is_cancelled(&self) -> bool {
        self.ct.is_cancelled()
    }

    pub async fn cancelled(&self) {
        self.ct.cancelled().await;
    }

    pub fn child_token(&self) -> Self {
        Self {
            ct: self.ct.child_token(),
            ..self.clone()
        }
    }
}

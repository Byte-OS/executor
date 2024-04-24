use core::future::Future;

use alloc::{boxed::Box, sync::Arc};

use crate::{
    task::{AsyncTask, AsyncTaskItem, BlankKernelTask, TaskType},
    task_id_alloc, TASK_QUEUE,
};

#[inline]
pub fn spawn(
    task: Arc<dyn AsyncTask>,
    future: impl Future<Output = ()> + Send + 'static,
    task_type: TaskType,
) {
    TASK_QUEUE.lock().push_back(AsyncTaskItem {
        future: Box::pin(future),
        task_type,
        task,
    });
}

#[inline]
pub fn spawn_blank(future: impl Future<Output = ()> + Send + 'static) {
    TASK_QUEUE.lock().push_back(AsyncTaskItem {
        future: Box::pin(future),
        task_type: TaskType::BlankKernel,
        task: Arc::new(BlankKernelTask(task_id_alloc())),
    })
}

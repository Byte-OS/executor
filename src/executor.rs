use alloc::{collections::VecDeque, sync::Arc, task::Wake, vec::Vec};
use arch::once::LazyInit;
use core::{
    sync::atomic::{AtomicBool, Ordering},
    task::{Context, Poll},
};
use log::info;
use sync::Mutex;

use crate::task::{AsyncTask, AsyncTaskItem, PinedFuture, TaskType};

pub struct TaskFutureItem(pub PinedFuture);

unsafe impl Send for TaskFutureItem {}
unsafe impl Sync for TaskFutureItem {}

pub type TaskId = usize;
pub static CURRENT_TASK: Mutex<Option<Arc<dyn AsyncTask>>> = Mutex::new(None);

/// FIFO task queue, Items will be pushed to the end of the queue after being called.
pub static TASK_QUEUE: Mutex<VecDeque<AsyncTaskItem>> = Mutex::new(VecDeque::new());
/// wake queue, not use at current.

pub static DEFAULT_EXECUTOR: Executor = Executor::new();

pub struct Executor {
    cores: LazyInit<Vec<Mutex<Option<Arc<dyn AsyncTask>>>>>,
    inited: AtomicBool,
}

impl Executor {
    pub const fn new() -> Self {
        Executor {
            cores: LazyInit::new(),
            inited: AtomicBool::new(false),
        }
    }

    pub fn init(&self, cores: usize) {
        let mut core_container = Vec::with_capacity(cores);
        (0..cores).for_each(|_| core_container.push(Mutex::new(None)));
        self.cores.init_by(core_container);
        self.inited.store(true, Ordering::SeqCst);
    }

    pub fn spawn(&mut self, task: Arc<dyn AsyncTask>, task_type: TaskType, future: PinedFuture) {
        TASK_QUEUE.lock().push_back(AsyncTaskItem {
            future,
            task_type,
            task,
        })
    }

    pub fn run(&self) {
        info!("fetch atomic data: {}", self.inited.load(Ordering::SeqCst));
        info!(
            "fetch atomic data not: {}",
            self.inited.load(Ordering::SeqCst)
        );
        // Waiting for executor's initialisation finish.
        while !self.inited.load(Ordering::SeqCst) {}
        loop {
            if TASK_QUEUE.lock().len() == 0 {
                break;
            }
            self.run_ready_task();
            self.hlt_if_idle();
        }
    }

    fn run_ready_task(&self) {
        let task = TASK_QUEUE.lock().pop_front();
        if let Some(task_item) = task {
            let AsyncTaskItem {
                task,
                mut future,
                task_type,
            } = task_item;
            task.before_run();

            *CURRENT_TASK.lock() = Some(task.clone());
            // let waker = self.create_waker(task.as_ref()).into();
            // Create Waker
            let waker = Arc::new(Waker {
                task_id: task.get_task_id(),
            })
            .into();
            let mut context = Context::from_waker(&waker);

            match future.as_mut().poll(&mut context) {
                Poll::Ready(()) => {} // task done
                Poll::Pending => TASK_QUEUE.lock().push_back(AsyncTaskItem {
                    future,
                    task_type,
                    task,
                }),
            }
        }
    }

    /// Executes the `hlt` instruction if there are no ready tasks
    fn hlt_if_idle(&self) {
        // let len = TASK_QUEUE.lock().len();
        // if len != 0 {
        //     arch::wfi();
        // }

        // log::error!("hlt if idle");
        arch::wfi();
        // log::error!("end");
        // log::error!("hlt if idle end: {}", TASK_QUEUE.lock().len());
    }
}

#[allow(dead_code)]
pub struct Waker {
    task_id: TaskId,
}

impl Wake for Waker {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref();
    }

    fn wake_by_ref(self: &Arc<Self>) {}
}

/// Alloc a task id.
pub fn task_id_alloc() -> TaskId {
    static TASK_ID: Mutex<usize> = Mutex::new(0);
    let mut task_id = TASK_ID.lock();
    *task_id += 1;
    *task_id
}

#[inline]
pub fn current_task() -> Arc<dyn AsyncTask> {
    CURRENT_TASK.lock().as_ref().map(|x| x.clone()).unwrap()
}

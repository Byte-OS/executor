use core::{future::Future, pin::Pin};

use alloc::{boxed::Box, sync::Arc};
use arch::kernel_page_table;
use downcast_rs::{impl_downcast, DowncastSync};

use crate::TaskId;

/// Default is kernel task
pub const TYPE_KERNEL_TASK: u8 = 0;

/// This is a trait the for generic task.
pub trait AsyncTask: DowncastSync {
    /// Get the id of the task
    fn get_task_id(&self) -> TaskId;
    /// Run befire the kernel
    fn before_run(&self);
}

/// This is a enum that indicates the task type.
#[derive(Debug, PartialEq, PartialOrd)]
pub enum TaskType {
    /// Blank Kernel Task Type, Just run in the kernel,
    /// No extra pagetable
    BlankKernel,
    /// Monolithic Task Type, Will have a independent pagetable.
    MonolithicTask,
    /// Microkernel task
    MicroTask,
    /// Unikernel task
    UnikernelTask,
    /// RTOS task
    RTOSTask,
    /// User defined task 1
    UserDefinedTask1,
    /// User defined task 2
    UserDefinedTask2,
}

pub type PinedFuture = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

/// This is a async task container will be called in the Async Task Item
pub struct AsyncTaskItem {
    pub future: PinedFuture,
    pub task_type: TaskType,
    pub task: Arc<dyn AsyncTask>,
}

/// This is a blank kernel task.
pub struct BlankKernelTask(pub usize);
impl AsyncTask for BlankKernelTask {
    /// Get task identifier
    fn get_task_id(&self) -> TaskId {
        self.0
    }

    /// before run switch to kernel page table.
    /// maybe I don't need to do this.
    fn before_run(&self) {
        kernel_page_table().change()
    }
}

impl_downcast!(sync AsyncTask);

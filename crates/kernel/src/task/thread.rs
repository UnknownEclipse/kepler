use alloc::{alloc::Global, boxed::Box};
use core::{
    alloc::{AllocError, Allocator, Layout},
    cell::{Cell, SyncUnsafeCell},
    mem::{self, ManuallyDrop, MaybeUninit},
    ptr::{self, addr_of_mut, NonNull},
    sync::atomic::{AtomicBool, AtomicPtr, AtomicU32, AtomicUsize},
};

use hal::task::Context;
use log::trace;

use super::{
    current, exit,
    task_types::{allocate_id, AtomicState, Head, Policy, Task, TaskVTable},
};
use crate::{
    error::KernResult,
    memory::{self, AllocOptions},
};

pub fn spawn<F, T>(f: F) -> KernResult<Thread>
where
    F: FnOnce() -> T + 'static + Send,
{
    Builder::new().spawn(f)
}

#[derive(Debug, Clone)]
pub struct Thread(pub Task);

impl Thread {
    pub fn unpark(self) {
        self.0.unpark();
    }
}

#[derive(Debug)]
pub struct Builder {
    stack_size: usize,
    policy: Policy,
}

impl Builder {
    pub fn new() -> Self {
        Self {
            stack_size: 16384,
            policy: Policy::Normal(127),
        }
    }

    pub fn spawn<F, T>(self, f: F) -> KernResult<Thread>
    where
        F: FnOnce() -> T + 'static + Send,
    {
        self.spawn_in(f, KAlloc)
    }

    pub fn spawn_in<F, T, A>(self, f: F, allocator: A) -> KernResult<Thread>
    where
        A: Allocator + Clone,
        F: FnOnce() -> T + 'static + Send,
    {
        let thread = allocate_thread_in(self, f, allocator)?;
        thread.clone().unpark();
        Ok(thread)
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

// #[derive(Debug)]
// pub struct JoinHandle<T>(Thread, PhantomData<fn(T)>);

// impl<T> JoinHandle<T> {
//     pub fn join(self) -> KernResult<T> {
//         let inner: NonNull<ThreadInner<(), T, Global>> = self.0 .0 .0.cast();
//         let futex = unsafe { &inner.as_ref().finished };
//         while futex.load(Ordering::Acquire) == 0 {
//             futex::wait(futex, 0);
//         }
//     }
// }

fn allocate_thread_in<F, T, A>(builder: Builder, f: F, allocator: A) -> KernResult<Thread>
where
    A: Allocator + Clone,
    F: FnOnce() -> T + 'static + Send,
{
    let mut stack = Box::new_uninit_slice_in(builder.stack_size, allocator.clone());

    let sp = init_stack(entry::<F, T, A>, &mut stack);
    let allocation = Box::new_uninit_in(allocator);
    let (ptr, allocator) = Box::into_raw_with_allocator(allocation);

    let inner = ThreadInner {
        head: Head {
            state: AtomicState::new(super::task_types::State::Parked),
            refs: AtomicUsize::new(1),
            id: allocate_id(),
            link: Default::default(),
            vtable: &ThreadInner::<F, T, A>::VTABLE,
            stack_ptr: AtomicPtr::new(sp.as_ptr()),
            policy: builder.policy,
            preemptible: AtomicBool::new(true),
        },
        stack: SyncUnsafeCell::new(stack),
        allocator: ManuallyDrop::new(allocator),
        func: Cell::new(Some(f)),
        result: Cell::new(None::<T>),
        finished: AtomicU32::new(0),
    };

    unsafe {
        ptr::write(ptr, MaybeUninit::new(inner));
        let raw = NonNull::new(ptr.cast()).unwrap();
        Ok(Thread(Task::from_raw(raw)))
    }
}

extern "C" fn entry<F, T, A>(_: *mut ()) -> !
where
    A: Allocator + Clone,
    F: FnOnce() -> T + 'static + Send,
{
    let task = current();
    let ptr: NonNull<ThreadInner<F, T, A>> = task.0.cast();
    let f = unsafe { ptr.as_ref().func.take().unwrap_unchecked() };
    mem::drop(task);
    f();
    exit();
}

fn init_stack(
    entry: extern "C" fn(*mut ()) -> !,
    stack: &mut [MaybeUninit<u8>],
) -> NonNull<Context> {
    let ctx = Context::with_initial(entry, ptr::null_mut());
    let top: *mut Context = stack.as_mut_ptr_range().end.cast();
    unsafe {
        let sp = top.sub(3);
        ptr::write(sp, ctx);
        NonNull::new_unchecked(sp)
    }
}

#[repr(C)]
struct ThreadInner<F, T, A = Global>
where
    A: Allocator,
{
    head: Head,
    result: Cell<Option<T>>,
    finished: AtomicU32,
    stack: SyncUnsafeCell<Box<[MaybeUninit<u8>], A>>,
    func: Cell<Option<F>>,
    allocator: ManuallyDrop<A>,
}

impl<F, T, A> Drop for ThreadInner<F, T, A>
where
    A: Allocator,
{
    fn drop(&mut self) {
        trace!("thread drop");
    }
}

impl<F, T, A> ThreadInner<F, T, A>
where
    A: Allocator,
{
    const VTABLE: TaskVTable = TaskVTable {
        deallocate: deallocate::<F, T, A>,
        drop_in_place: drop_in_place::<F, T, A>,
    };
}

unsafe fn drop_in_place<F, T, A>(head: NonNull<Head>)
where
    A: Allocator,
{
    let inner: NonNull<ThreadInner<F, T, A>> = head.cast();
    ptr::drop_in_place(inner.as_ptr());
}

unsafe fn deallocate<F, T, A>(head: NonNull<u8>)
where
    A: Allocator,
{
    let inner: NonNull<ThreadInner<F, T, A>> = head.cast();
    let layout = Layout::new::<ThreadInner<F, T, A>>();

    // Deallocate will be called after drop_in_place, so we use a ManuallyDrop to
    // prevent the allocator from being dropped, then use addr_of_mut!() to access the
    // location of the allocator, take it out of the object, then deallocate.
    let slot = addr_of_mut!((*inner.as_ptr()).allocator);
    let allocator = ManuallyDrop::take(&mut *slot);
    allocator.deallocate(head.cast(), layout);
}

#[derive(Debug, Clone, Copy)]
struct KAlloc;

unsafe impl Allocator for KAlloc {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        AllocOptions::new(layout.size())
            .start_guard_pages(1)
            .end_guard_pages(1)
            .allocate_in_address_space(&memory::AddrSpace::Kernel)
            .map_err(|_| AllocError)
    }

    unsafe fn deallocate(&self, _ptr: NonNull<u8>, _layout: Layout) {}
}

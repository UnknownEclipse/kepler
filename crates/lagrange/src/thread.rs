use alloc::{alloc::dealloc, borrow::Cow, boxed::Box};
use core::{
    alloc::Layout,
    cell::{Cell, UnsafeCell},
    mem::{self, MaybeUninit},
    num::NonZeroU64,
    ptr::{self, NonNull},
    sync::atomic::Ordering,
    task::Waker,
};

use hal::task::Context;

use crate::{
    scheduler::global,
    task::{
        header::{Header, TaskVTable},
        raw_task::RawTask,
        JoinHandle,
    },
};

#[inline]
pub fn spawn<F, R>(f: F) -> JoinHandle<R>
where
    F: 'static + Send + FnOnce() -> R,
    R: 'static + Send,
{
    Builder::new().spawn(f)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ThreadId(NonZeroU64);

#[derive(Debug)]
pub struct Builder {
    stack_size: usize,
    name: Option<&'static str>,
}

impl Builder {
    pub fn new() -> Self {
        Self {
            stack_size: 8192,
            name: None,
        }
    }

    pub fn stack_size(&mut self, stack_size: usize) -> &mut Self {
        self.stack_size = stack_size;
        self
    }

    pub fn name(&mut self, name: &'static str) -> &mut Self {
        self.name = Some(name);
        self
    }

    pub fn spawn<F, T>(self, f: F) -> JoinHandle<T>
    where
        F: 'static + Send + FnOnce() -> T,
        T: 'static + Send,
    {
        let task = allocate_task(f, self);
        let join_handle = unsafe { JoinHandle::from_raw(task.clone()) };
        global().schedule(task);
        join_handle
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct Thread(pub(crate) RawTask);

impl Thread {
    pub fn unpark(self) {
        self.0.schedule();
    }

    pub fn name(&self) -> Option<&str> {
        self.0.name()
    }

    pub fn id(&self) -> ThreadId {
        ThreadId(self.0.id())
    }

    pub fn into_waker(self) -> Waker {
        self.0.into_waker()
    }
}

pub fn current() -> Thread {
    let task = global().current();
    Thread(task)
}

#[inline]
pub fn park() {
    global().park();
}

#[inline]
pub fn park_if<F>(f: F)
where
    F: FnOnce() -> bool,
{
    let mut f = Some(f);
    global().park_if(&mut || unsafe { (f.take().unwrap_unchecked())() });
}

pub fn yield_now() {
    global().yield_now();
}

fn allocate_task<F, R>(f: F, builder: Builder) -> RawTask
where
    F: FnOnce() -> R + Send,
    R: Send,
{
    let stack = Box::new_uninit_slice(builder.stack_size);
    let stack = wrap_slice_box(stack);

    let mut full: FullTaskState<F, R> = FullTaskState {
        header: Header::new(&FullTaskState::<F, R>::VTABLE),
        func: Cell::new(Some(f)),
        return_value: Cell::new(None),
        stack,
    };

    full.header.refs.store(2, Ordering::Relaxed);
    full.header.name = builder.name.map(Cow::Borrowed);

    let full = Box::new(full);

    let ptr: *const FullTaskState<F, R> = &*full;

    let initial_context = Context::with_initial(start::<F, R>, ptr.cast_mut().cast());
    let stack = full.stack.get();
    let top = unsafe { (stack as *mut u8).add(stack.len()) };
    let top: *mut Context = top.cast();
    let stack_ptr = unsafe { top.sub(1) };
    unsafe { stack_ptr.write(initial_context) };
    full.header.context.store(stack_ptr, Ordering::Release);

    unsafe { RawTask::from_raw(NonNull::new(Box::into_raw(full)).unwrap().cast()) }
}

#[repr(C)]
struct FullTaskState<F, R> {
    header: Header,
    func: Cell<Option<F>>,
    return_value: Cell<Option<R>>,
    stack: Box<UnsafeCell<[MaybeUninit<u8>]>>,
}

impl<F, R> FullTaskState<F, R> {
    const VTABLE: TaskVTable = TaskVTable {
        drop_in_place: drop_in_place::<F, R>,
        deallocate: deallocate::<F, R>,
        read_value_into: read_value_into::<F, R>,
    };
}

unsafe fn drop_in_place<F, R>(ptr: NonNull<Header>) {
    let state: NonNull<FullTaskState<F, R>> = ptr.cast();
    ptr::drop_in_place(state.as_ptr());
}

unsafe fn deallocate<F, R>(ptr: *mut u8) {
    let layout = Layout::new::<FullTaskState<F, R>>();
    dealloc(ptr, layout);
}

unsafe fn read_value_into<F, R>(hdr: NonNull<Header>, ptr: *mut u8) {
    let state: NonNull<FullTaskState<F, R>> = hdr.cast();
    let value = state
        .as_ref()
        .return_value
        .take()
        .expect("no return value available");

    ptr::write(ptr.cast(), value);
}

fn wrap_slice_box<T>(b: Box<[T]>) -> Box<UnsafeCell<[T]>> {
    let p = Box::into_raw(b);
    unsafe { Box::from_raw(p as *mut UnsafeCell<[T]>) }
}

extern "C" fn start<F, R>(ptr: *mut ()) -> !
where
    F: FnOnce() -> R,
{
    let task = unsafe {
        let raw = NonNull::new_unchecked(ptr).cast();
        RawTask::from_raw(raw)
    };

    let full = unsafe {
        let full: *mut FullTaskState<F, R> = ptr.cast();
        &*full
    };

    let f = full.func.take().unwrap();
    let result = f();
    full.return_value.set(Some(result));
    full.header.finished.notify();

    mem::drop(task);
    park();
    unreachable!("thread continued after execution completed");
}

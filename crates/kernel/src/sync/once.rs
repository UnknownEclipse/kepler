use core::{
    convert::Infallible,
    hint::unreachable_unchecked,
    sync::atomic::{AtomicU32, Ordering},
};

use super::futex::{wait, wake_all, wake_one};

#[derive(Debug)]
pub struct Once {
    state: AtomicU32,
}

impl Once {
    pub const fn new() -> Self {
        Self {
            state: AtomicU32::new(EMPTY),
        }
    }

    pub const fn completed() -> Self {
        Self {
            state: AtomicU32::new(READY),
        }
    }

    pub fn is_completed(&self) -> bool {
        self.state.load(Ordering::Acquire) == READY
    }

    pub fn call_once<F>(&self, f: F)
    where
        F: FnOnce(),
    {
        self.try_call_once(|| {
            f();
            Ok::<_, Infallible>(())
        })
        .unwrap()
    }

    pub fn try_call_once<F, E>(&self, f: F) -> Result<(), E>
    where
        F: FnOnce() -> Result<(), E>,
    {
        if self.is_completed() {
            Ok(())
        } else {
            let mut f = Some(f);
            let mut err = None;

            call_once(&self.state, &mut || {
                let f = unsafe { f.take().unwrap_unchecked() };
                match f() {
                    Ok(_) => true,
                    Err(e) => {
                        err = Some(e);
                        false
                    }
                }
            });

            match err {
                Some(err) => Err(err),
                None => Ok(()),
            }
        }
    }

    pub fn wait(&self) {
        if !self.is_completed() {
            self.wait_slow();
        }
    }

    #[cold]
    fn wait_slow(&self) {
        loop {
            let state = self.state.load(Ordering::Acquire);
            if state == READY {
                return;
            }
            wait(&self.state, state);
        }
    }
}

const EMPTY: u32 = 0;
const BUSY: u32 = 1;
const READY: u32 = 2;

#[cold]
pub(crate) fn call_once(state: &AtomicU32, f: &mut dyn FnMut() -> bool) {
    loop {
        let current = state.load(Ordering::Acquire);
        match current {
            EMPTY => {
                if state
                    .compare_exchange(EMPTY, BUSY, Ordering::Acquire, Ordering::Relaxed)
                    .is_err()
                {
                    continue;
                }

                if f() {
                    state.store(READY, Ordering::Release);
                    wake_all(state);
                } else {
                    state.store(EMPTY, Ordering::Release);
                    wake_one(state);
                }

                return;
            }
            BUSY => {
                wait(state, BUSY);
            }
            READY => return,
            _ => unsafe { unreachable_unchecked() },
        }
    }
}

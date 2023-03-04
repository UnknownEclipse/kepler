use core::sync::atomic::AtomicU32;

pub fn wait(atomic: &AtomicU32, value: u32) {
    todo!()
}

pub fn wake_one(atomic: *const AtomicU32) {
    todo!()
}

pub fn wake_all(atomic: *const AtomicU32) {
    todo!()
}

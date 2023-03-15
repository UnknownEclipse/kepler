use core::iter::repeat_with;

use entropy::Blake2bPool;
use hal::{interrupts, x86_64::random::RdSeed};
use rand_core::{RngCore, SeedableRng};
use spin::{mutex::SpinMutex, Lazy};

static POOL: Lazy<SpinMutex<Blake2bPool>> = Lazy::new(Default::default);

pub fn init() {
    write_cpu_randomness();
}

pub fn write_entropy(bytes: &[u8]) {
    interrupts::without(|_| POOL.lock().mix(bytes));
}

pub fn get_entropy(buf: &mut [u8]) {
    interrupts::without(|_| POOL.lock().extract(buf));
}

pub fn seeded<R>(remix: bool) -> R
where
    R: SeedableRng + RngCore,
{
    let mut seed = R::Seed::default();
    get_entropy(seed.as_mut());

    let mut rng = R::from_seed(seed);

    if remix {
        let mut temp = [0; 32];
        rng.fill_bytes(&mut temp);
        write_entropy(&temp);
    }
    write_cpu_randomness();

    rng
}

fn write_cpu_randomness() {
    let Some(rng) = RdSeed::new() else { return };

    interrupts::without(|_| {
        let mut pool = POOL.lock();

        repeat_with(|| rng.random_u64())
            .take_while(|v| v.is_some())
            .take(20)
            .flatten()
            .for_each(|v| {
                pool.mix(&v.to_ne_bytes());
            });
    });
}

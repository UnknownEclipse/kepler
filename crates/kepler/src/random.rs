use ahash::random_state::set_random_source;
use entropy::{Blake2bPool, Pool};
use hal::interrupts;
use rand_chacha::{
    rand_core::{self, RngCore},
    ChaCha20Rng,
};
use spin::{mutex::SpinMutex, Lazy};

use crate::irq_mutex::SpinIrqMutex;

static POOL: Lazy<SpinIrqMutex<Blake2bPool>> = Lazy::new(|| SpinIrqMutex::new(Pool::empty()));

pub fn mix_entropy(data: &[u8]) {
    POOL.lock(|pool, _| {
        pool.mix(data);
    });
}

pub fn crypto_rng() -> CryptoRng {
    POOL.lock(|pool, _| CryptoRng(pool.seeded()))
}

#[derive(Debug)]
pub struct CryptoRng(ChaCha20Rng);

impl RngCore for CryptoRng {
    fn next_u32(&mut self) -> u32 {
        self.0.next_u32()
    }

    fn next_u64(&mut self) -> u64 {
        self.0.next_u64()
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.0.fill_bytes(dest)
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core::Error> {
        self.0.try_fill_bytes(dest)
    }
}

impl rand_core::CryptoRng for CryptoRng {}

#[inline]
pub fn getrandom(buf: &mut [u8]) {
    crypto_rng().fill_bytes(buf);
}

#[derive(Debug)]
struct AHashRandomSource {
    rng: SpinMutex<CryptoRng>,
}

impl ahash::random_state::RandomSource for AHashRandomSource {
    fn gen_hasher_seed(&self) -> usize {
        interrupts::without(|_| self.rng.lock().next_u64() as usize)
    }
}

pub fn init_ahash() {
    let rng = SpinMutex::new(crypto_rng());
    let source = AHashRandomSource { rng };
    set_random_source(source).expect("failed to set random source");
}

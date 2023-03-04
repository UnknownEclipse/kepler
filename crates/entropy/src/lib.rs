#![no_std]

use blake2::{Blake2b512, Blake2s256, Digest};
use rand_core::{RngCore, SeedableRng};

pub type Blake2sPool = Pool<Blake2s256>;
pub type Blake2bPool = Pool<Blake2b512>;

/// A pool for accumulating randomness based on a cryptographic hash function.
///
/// # Usage
///
/// ```
/// use rand_chacha::ChaCha20Rng;
///
/// let mut pool = Blake2bPool::empty();
/// pool.mix(b"some randomness... ooohhhh randommmmm");
///
/// /// Create a new RNG using randomness from the pool. This method will also mix
/// ///randomness from the RNG back into the pool.
/// let mut rng: ChaCha20Rng = pool.seeded();
///
/// /// Use the RNG
/// let random_u32: u32 = rng.gen();
/// ```
///
/// # Security
///
/// Once the pool has reached a threshold of having enough entropy to fully seed a CRNG,
/// the pool will never run out of entropy.
///
/// This pool does not keep track of available entropy, as entropy estimation is more
/// or less black magic and generally shouldn't be relied upon. Instead, the pool should
/// be used in ways such that attackers cannot learn information about other pool users
/// from their own pool. This typically involves having per user or per process pools.
///
/// ## Low Entropy Behavior
///
///
///
#[derive(Debug)]
pub struct Pool<D> {
    hash: D,
}

impl<D> Pool<D>
where
    D: Digest + Clone,
{
    /// Create a new empty pool.
    pub fn empty() -> Self {
        Pool { hash: D::new() }
    }

    /// Mix in some data.
    ///
    /// Even if the data is not random, the overall entropy of the pool will not
    /// diminish.
    pub fn mix(&mut self, data: &[u8]) {
        self.hash.update(data);
    }

    /// Drain entropy from the pool
    pub fn extract(&mut self, mut buf: &mut [u8]) {
        let output_size = <D as Digest>::output_size();
        let block_size = output_size / 2;

        while block_size <= buf.len() {
            let out = self.hash.clone().finalize();
            let (block, remix) = out.split_at(block_size);
            buf[..block_size].copy_from_slice(block);
            buf = &mut buf[block_size..];
            self.mix(remix);
        }

        if !buf.is_empty() {
            let out = self.hash.clone().finalize();
            let (block, rest) = out.split_at(buf.len());
            buf.copy_from_slice(block);
            self.mix(rest);
        }
    }

    /// Create an RNG seeded with randomness from the pool.
    /// This method will also mix some randomness from the RNG back into the pool.
    /// This is secure even without the use of a CRNG because no matter how non-random
    /// the generated data is, the overall pool entropy will not decrease. In other words,
    /// a CRNG will refill the pool, a non-CRNG won't make things any worse.
    pub fn seeded<R>(&mut self) -> R
    where
        R: SeedableRng + RngCore,
    {
        let mut seed = R::Seed::default();
        self.extract(seed.as_mut());
        let mut rng = R::from_seed(seed);

        let mut remix = R::Seed::default();
        if rng.try_fill_bytes(remix.as_mut()).is_ok() {
            self.mix(remix.as_mut());
        }

        rng
    }
}

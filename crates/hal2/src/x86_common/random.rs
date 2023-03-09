//! High level RNGs using the RDRAND and RDSEED instructions.
//!
//! # Usage
//!
//! As some CPUs may not have support for these instructions, the constructor may return
//! `None`. This also helps ensure we don't need to check for support during each
//! invocation.
//!
//! ## RDRAND
//!
//! ```
//! if let Some(rdrand) = RdRand::new() {
//!     let random_value = rdrand.random_u32();
//! }
//! ```
//!
//! ## RDSEED
//!
//! ```
//! if let Some(rdseed) = RdSeed::new() {
//!     let random_value = rdseed.random_u32();
//! }
//! ```
//!
//! # Security
//!
//! While the intended purpose of these generators is to be cryptographically secure,
//! there have been numerous [vulnerabilities](https://en.wikipedia.org/wiki/RDRAND#Security_issues)
//! reported. As such, these RNGs (as of the moment) do *not* implement the [CryptoRng]
//! trait and should not be assumed to be secure on their own.

use core::arch::x86_64::__cpuid;

use rand_core::RngCore;

use super::instr::{rdrand16, rdrand32, rdrand64, rdseed16, rdseed32, rdseed64};

#[non_exhaustive]
#[derive(Debug, Clone, Copy)]
pub struct RdRand;

impl RdRand {
    pub fn new() -> Option<Self> {
        is_rdrand_supported().then_some(RdRand)
    }

    #[inline]
    pub fn random_u16(&self) -> Option<u16> {
        unsafe {
            // SAFETY: The existence of `self` guarantees that RDRAND is supported
            rdrand16()
        }
    }

    #[inline]
    pub fn random_u32(&self) -> Option<u32> {
        unsafe {
            // SAFETY: The existence of `self` guarantees that RDRAND is supported
            rdrand32()
        }
    }

    #[inline]
    pub fn random_u64(&self) -> Option<u64> {
        unsafe {
            // SAFETY: The existence of `self` guarantees that RDRAND is supported
            rdrand64()
        }
    }
}

#[cfg(feature = "rand_core")]
impl RngCore for RdRand {
    fn next_u32(&mut self) -> u32 {
        loop {
            if let Some(v) = self.random_u32() {
                return v;
            }
        }
    }

    fn next_u64(&mut self) -> u64 {
        loop {
            if let Some(v) = self.random_u64() {
                return v;
            }
        }
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        for chunk in dest.chunks_mut(8) {
            let v = self.next_u64();
            chunk.copy_from_slice(&v.to_ne_bytes()[..chunk.len()]);
        }
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core::Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy)]
pub struct RdSeed;

impl RdSeed {
    pub fn new() -> Option<Self> {
        is_rdseed_supported().then_some(RdSeed)
    }

    #[inline]
    pub fn random_u16(&self) -> Option<u16> {
        unsafe {
            // SAFETY: The existence of `self` guarantees that RDSEED is supported
            rdseed16()
        }
    }

    #[inline]
    pub fn random_u32(&self) -> Option<u32> {
        unsafe {
            // SAFETY: The existence of `self` guarantees that RDSEED is supported
            rdseed32()
        }
    }

    #[inline]
    pub fn random_u64(&self) -> Option<u64> {
        unsafe {
            // SAFETY: The existence of `self` guarantees that RDSEED is supported
            rdseed64()
        }
    }
}

#[cfg(feature = "rand_core")]
impl RngCore for RdSeed {
    fn next_u32(&mut self) -> u32 {
        loop {
            if let Some(v) = self.random_u32() {
                return v;
            }
        }
    }

    fn next_u64(&mut self) -> u64 {
        loop {
            if let Some(v) = self.random_u64() {
                return v;
            }
        }
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        for chunk in dest.chunks_mut(8) {
            let v = self.next_u64();
            chunk.copy_from_slice(&v.to_ne_bytes()[..chunk.len()]);
        }
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core::Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}

fn is_rdrand_supported() -> bool {
    let cpuid = unsafe { __cpuid(1) };
    cpuid.ecx & CPUID_ECX_RDRAND != 0
}

fn is_rdseed_supported() -> bool {
    let cpuid = unsafe { __cpuid(0x7) };
    cpuid.ebx & CPUID_EBX_RDSEED != 0
}

const CPUID_ECX_RDRAND: u32 = 1 << 30;
const CPUID_EBX_RDSEED: u32 = 1 << 18;

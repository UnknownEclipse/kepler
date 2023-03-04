use core::arch::x86_64::__cpuid;

pub type Feature = hal_core::features::Feature<X86Feature>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum X86Feature {}

macro_rules! cpuid {
    ($leaf:expr, $reg:ident, $bit:expr) => {
        unsafe { (__cpuid($leaf).$reg & (1 << $bit)) != 0 }
    };
}
pub fn is_detected(feature: Feature) -> bool {
    match feature {
        Feature::Random => cpuid!(1, ecx, 30),
        Feature::Arch(_) => todo!(),
    }
}

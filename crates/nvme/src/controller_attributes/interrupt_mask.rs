use bitfrob::{u32_get_bit, u32_with_bit};

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InterruptMaskSet(u32);

impl InterruptMaskSet {
    pub fn mask(&mut self, interrupt: u8) {
        self.0 = u32_with_bit(interrupt.into(), self.0, true);
    }

    pub fn unmask(&mut self, interrupt: u8) {
        self.0 = u32_with_bit(interrupt.into(), self.0, false);
    }

    pub fn is_masked(&mut self, interrupt: u8) -> bool {
        u32_get_bit(interrupt.into(), self.0)
    }
}

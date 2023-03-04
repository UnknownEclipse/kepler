use core::fmt::Debug;

use pic8259::ChainedPics;
use x2apic::lapic::{LocalApic, LocalApicBuilder};

pub struct InterruptController {
    pics: ChainedPics,
    apic: Option<LocalApic>,
}

impl InterruptController {
    pub unsafe fn new() -> Self {
        Self {
            pics: ChainedPics::new(32, 40),
            apic: None,
        }
    }

    pub unsafe fn with_local_apic(mut builder: LocalApicBuilder) -> Self {
        InterruptController {
            pics: ChainedPics::new(32, 40),
            apic: Some(builder.build().unwrap()),
        }
    }

    pub unsafe fn init(&mut self) {
        self.pics.initialize();
        if let Some(apic) = &mut self.apic {
            self.pics.disable();
            apic.enable();
        } else {
            self.pics.write_masks(0, 0);
        }
    }

    pub unsafe fn end_of_interrupt(&mut self, interrupt: u8) {
        if let Some(apic) = &mut self.apic {
            apic.end_of_interrupt();
        } else {
            self.pics.notify_end_of_interrupt(interrupt + 32);
        }
    }
}

impl Debug for InterruptController {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("InterruptController")
            .field("pics", &..)
            .field("apic", &self.apic)
            .finish()
    }
}

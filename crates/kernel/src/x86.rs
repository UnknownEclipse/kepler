use hal::x86_64::gdt::{Gdt, Segment, Selector, Tss64};
use spin::Lazy;

static GDT: Lazy<GdtData> = Lazy::new(build_gdt);
static TSS: Lazy<Tss64> = Lazy::new(build_tss);

pub unsafe fn init() {
    todo!("GDT init")
}

struct GdtData {
    table: Gdt<16>,
    code_segment: Selector,
}

fn build_gdt() -> GdtData {
    let mut gdt = Gdt::new();
    gdt.push(Segment::kernel_code16());
    gdt.push(Segment::kernel_code32());
    let cs = gdt.push(Segment::kernel_code64());
    gdt.push(Segment::kernel_data16());
    gdt.push(Segment::kernel_data32());
    gdt.push(Segment::kernel_data64());
    gdt.push(Segment::tss64(&*TSS));

    GdtData {
        table: gdt,
        code_segment: cs,
    }
}

fn build_tss() -> Tss64 {
    todo!()
}

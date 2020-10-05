use core::fmt;

#[repr(C)]
#[derive(Default, Copy, Clone, Debug)]
pub struct TrapFrame {
    // FIXME: Fill me in.
    pub ttbr0_el: u64,
    pub ttbr1_el: u64,
    pub elr_el: u64,
    pub spsr_el: u64,
    pub sp_el: u64,
    pub tpidr_el: u64,
    pub qs: [u128; 32],
    pub xs: [u64; 32],
}


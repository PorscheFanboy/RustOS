use core::time::Duration;

use volatile::prelude::*;
use volatile::Volatile;

const INT_BASE: usize = 0x40000000;

/// Core interrupt sources (QA7: 4.10)
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum LocalInterrupt {
    // Lab 5 1.C
    // FIXME: please fill in the definition
    CNTPSIRQ = 0,
    CNTPNSIRQ = 1,
    CNTHPIRQ = 2,
    CNTVIRQ = 3,
    Mailbox0 = 4,
    Mailbox1 = 5,
    Mailbox2 = 6,
    Mailbox3 = 7,
    Gpu = 8,
    Pmu = 9,
    AxiOutstanding = 10,
    LocalTimer = 11,
}

impl LocalInterrupt {
    pub const MAX: usize = 12;

    pub fn iter() -> impl Iterator<Item = LocalInterrupt> {
        (0..LocalInterrupt::MAX).map(|n| LocalInterrupt::from(n))
    }
}

impl From<usize> for LocalInterrupt {
    fn from(irq: usize) -> LocalInterrupt {
        // Lab 5 1.C
        // unimplemented!("LocalInterrupt")
        match irq {
            0 => LocalInterrupt::CNTPSIRQ,
            1 => LocalInterrupt::CNTPNSIRQ,
            2 => LocalInterrupt::CNTHPIRQ,
            3 => LocalInterrupt::CNTVIRQ,
            4 => LocalInterrupt::Mailbox0,
            5 => LocalInterrupt::Mailbox1,
            6 => LocalInterrupt::Mailbox2,
            7 => LocalInterrupt::Mailbox3,
            8 => LocalInterrupt::Gpu,
            9 => LocalInterrupt::Pmu,
            10 => LocalInterrupt::AxiOutstanding,
            _ => LocalInterrupt::LocalTimer,
        }
    }
}

/// BCM2837 Local Peripheral Registers (QA7: Chapter 4)
#[repr(C)]
#[allow(non_snake_case)]
struct Registers {
    // Lab 5 1.C
    // FIXME: please fill in the definition
    CONTROL: Volatile<u32>,
    _unused1: [Volatile<u32>; 8],
    LOCAL_IRQ: Volatile<u32>,
    _unused2: [Volatile<u32>; 3],
    LOCAL_TIMER_CTL: Volatile<u32>,
    LOCAL_TIMER_FLAGS: Volatile<u32>,
    _unused3: [Volatile<u32>; 1],
    CORE_TIMER_IRQCNTL: [Volatile<u32>; 4],
    CORE_MAILBOX_IRQCNTL: [Volatile<u32>; 4],
    CORE_IRQ_SRC: [Volatile<u32>; 4],
}

pub struct LocalController {
    core: usize,
    registers: &'static mut Registers,
}

pub fn set_cntp_ctl_el0(x: u64) {
    unsafe {
        asm!("msr cntp_ctl_el0, $0" :: "r"(x));
    }
}

pub fn set_cntk_ctl_el1(x: u64) {
    unsafe {
        asm!("msr cntkctl_el1, $0" :: "r"(x));
    }
}

pub fn set_cntp_tval_el0(x: u64) {
    unsafe {
        asm!("msr cntp_tval_el0, $0" :: "r"(x));
    }
}

pub fn get_cntfrq_el0() -> u64 {
    let x: u64;
    unsafe {
        asm!("mrs $0, cntfrq_el0"
            : "=r"(x));
    }
    x
}

pub fn get_cntpct_el0() -> u64 {
    let x: u64;
    unsafe {
        asm!("isb
              mrs $0, cntpct_el0"
            : "=r"(x) : : : "volatile");
    }
    x
}

impl LocalController {
    /// Returns a new handle to the interrupt controller.
    pub fn new(core: usize) -> LocalController {
        LocalController {
            core: core,
            registers: unsafe { &mut *(INT_BASE as *mut Registers) },
        }
    }

    pub fn read(&self) -> u64 {
        let cntfrq = get_cntfrq_el0(); // 62500000
        (get_cntpct_el0() * 1000000 / (cntfrq as u64)) as u64
    }

    pub fn enable_local_timer(&mut self) {
        // Lab 5 1.C
        self.registers.CORE_TIMER_IRQCNTL[self.core].write(1 << (LocalInterrupt::CNTPNSIRQ as u8));
        set_cntp_ctl_el0(0x1); // enable timer interrupt and do not mask it
        set_cntk_ctl_el1(0x3); // allow EL0 to read timer counter
    }

    pub fn is_pending(&self, int: LocalInterrupt) -> bool {
        // Lab 5 1.C
        self.registers.CORE_IRQ_SRC[self.core].read() & (1 << (LocalInterrupt::CNTPNSIRQ as u8)) != 0
    }

    pub fn tick_in(&mut self, t: Duration) {
        // Lab 5 1.C
        // See timer: 3.1 to 3.3
        let cntfrq = get_cntfrq_el0(); // 62500000
        set_cntp_tval_el0(((cntfrq as f64) * (t.as_micros() as f64) / 1000000.0) as u64);
    }
}

pub fn local_tick_in(core: usize, t: Duration) {
    LocalController::new(core).tick_in(t);
}

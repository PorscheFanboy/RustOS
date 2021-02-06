mod frame;
mod syndrome;
mod syscall;

pub mod irq;
pub use self::frame::TrapFrame;

use pi::interrupt::{Controller, Interrupt};
use pi::local_interrupt::{LocalController, LocalInterrupt};

use self::syndrome::Syndrome;
use self::syscall::handle_syscall;
use crate::percore;
use crate::traps::irq::IrqHandlerRegistry;

use crate::console::{kprintln, kprint};
use crate::shell;
use crate::GLOBAL_IRQ;
use crate::LOCAL_IRQ;

#[repr(u16)]
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Kind {
    Synchronous = 0,
    Irq = 1,
    Fiq = 2,
    SError = 3,
}

#[repr(u16)]
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Source {
    CurrentSpEl0 = 0,
    CurrentSpElx = 1,
    LowerAArch64 = 2,
    LowerAArch32 = 3,
}

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct Info {
    source: Source,
    kind: Kind,
}

/// This function is called when an exception occurs. The `info` parameter
/// specifies the source and kind of exception that has occurred. The `esr` is
/// the value of the exception syndrome register. Finally, `tf` is a pointer to
/// the trap frame for the exception.
#[no_mangle]
pub extern "C" fn handle_exception(info: Info, esr: u32, tf: &mut TrapFrame) {
    // kprintln!("{:?} {:?} {:b}", info.source, info.kind, esr);
    match info.kind {
        Kind::Synchronous => {
            let syn = Syndrome::from(esr);
           // kprintln!("{:?}", syn);
            match syn {
                Syndrome::Brk(k) => {
                    tf.elr_el += 4;
                    shell::shell("Brk! ");
                    return;
                },
                Syndrome::Svc(n) => {
                    handle_syscall(n, tf);
                    return;
                }
                k => {
                    // kprintln!("Error");
                    return;
                },
            }
        },
        Kind::Irq => {
            // GLOBAL_IRQ.invoke(Interrupt::Timer1, tf);
            // kprintln!("sss {}", tf.tpidr_el);
            LOCAL_IRQ.invoke(LocalInterrupt::LocalTimer, tf);
            use aarch64::*;
            // kprint!("{}", affinity());

            // kprintln!("ttt {}", tf.tpidr_el);
            return;
        }
        _ => {
            return;
        }
    }
}

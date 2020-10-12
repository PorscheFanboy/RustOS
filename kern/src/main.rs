#![feature(alloc_error_handler)]
#![feature(const_fn)]
#![feature(decl_macro)]
#![feature(asm)]
#![feature(global_asm)]
#![feature(optin_builtin_traits)]
#![feature(ptr_internals)]
#![feature(raw_vec_internals)]
#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

#[cfg(not(test))]
mod init;

#[macro_use]
extern crate alloc;
#[macro_use]
extern crate log;

pub mod allocator;
pub mod console;
pub mod fs;
pub mod logger;
pub mod mutex;
pub mod net;
pub mod param;
pub mod percore;
pub mod process;
pub mod shell;
pub mod traps;
pub mod vm;

use console::kprintln;
use pi::timer::*;
use core::time::Duration;

const MBOX_BASE: u32 = 0x3F00B880;
const MBOX_STATUS: *mut u32 = (MBOX_BASE as u32 + 0x18) as *mut u32;
const MBOX_WRITE: *mut u32 = (MBOX_BASE as u32 + 0x20) as *mut u32;
static mut MBOX: [u32; 36] = [0; 36];

unsafe fn set_led_state(x: u32) {
    MBOX[0] = 8*4;                  // length of the message
    MBOX[1] = 0;         // this is a request message
    MBOX[2] = 0x38041;   // get serial number command
    MBOX[3] = 8;                    // buffer size
    MBOX[4] = 0;
    MBOX[5] = 130;                    // clear output buffer
    MBOX[6] = x;
    let ptr : *const u32 = &MBOX as *const u32;
    let ch : u32 = 8;
    let mboxaddr : u32 = ((ptr as u32) & (!0xF)) | (ch & 0xF);
    loop {
        if MBOX_STATUS.read_volatile() == 0x40000000 {
            break;
        }
    }
    MBOX_WRITE.write_volatile(mboxaddr);
    loop {
        loop {
            if MBOX_STATUS.read_volatile() != 0x40000000 {
                break;
            }
        }
        if (MBOX_BASE as *const u32).read_volatile() & 0xF == ch {
            return;
        }
    }
}

// FIXME: You need to add dependencies here to
// test your drivers (Phase 2). Add them as needed.

use allocator::Allocator;
use fs::FileSystem;
use net::uspi::Usb;
use net::GlobalEthernetDriver;
use process::GlobalScheduler;
use traps::irq::{Fiq, GlobalIrq};
use vm::VMManager;

#[cfg_attr(not(test), global_allocator)]
pub static ALLOCATOR: Allocator = Allocator::uninitialized();
pub static FILESYSTEM: FileSystem = FileSystem::uninitialized();
pub static SCHEDULER: GlobalScheduler = GlobalScheduler::uninitialized();
pub static VMM: VMManager = VMManager::uninitialized();
pub static USB: Usb = Usb::uninitialized();
pub static GLOBAL_IRQ: GlobalIrq = GlobalIrq::new();
pub static FIQ: Fiq = Fiq::new();
pub static ETHERNET: GlobalEthernetDriver = GlobalEthernetDriver::uninitialized();

extern "C" {
    static __text_beg: u64;
    static __text_end: u64;
    static __bss_beg: u64;
    static __bss_end: u64;
}

unsafe fn kmain() -> ! {
    crate::logger::init_logger();

    info!(
        "text beg: {:016x}, end: {:016x}",
        &__text_beg as *const _ as u64, &__text_end as *const _ as u64
    );
    info!(
        "bss  beg: {:016x}, end: {:016x}",
        &__bss_beg as *const _ as u64, &__bss_end as *const _ as u64
    );

    for i in 0..1 {
        set_led_state(1);
        spin_sleep(Duration::from_secs(1));
        set_led_state(0);
        spin_sleep(Duration::from_secs(1));
    }
    let mut atag = pi::atags::Atags::get();
    while let tag = atag.next() {
        match tag {
            None => break,
            Some(t) => kprintln!("{:#?}", t),
        }
    }

    ALLOCATOR.initialize();
    FILESYSTEM.initialize();

    /*
    use fat32::traits::{FileSystem, Dir};
    use fat32::vfat::{Entry, File, VFat, VFatHandle};
    use shim::io::Read;
    use shim::path::*;
    let path = Path::new("/");
    let entry = FILESYSTEM.open(&path).unwrap();
    // kprintln!("{:?}", entry);
    match entry {
        Entry::Dossier(d) => {
            for ent in d.entries().unwrap() {
                match ent {
                    Entry::Dossier(dd) => {
                        kprintln!("dossier {}", dd.name);
                    },
                    Entry::Fichier(ff) => {
                        kprintln!("fichier {}", ff.name);
                    }
                }
            }
        },
        Entry::Fichier(mut f) => {
            kprintln!("{:?} {}", f.name, f.size);
            kprintln!("FICHIER");
            let mut buf: [u8; 50] = [0; 50];
            f.read(&mut buf);
            /*

            use core::str;
            let s = match str::from_utf8(&buf[0..f.size as usize]) {
                Ok(v) => v,
                Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
            };
            kprintln!("{}", s);
            */
            kprintln!("DONE reading file");
        },
    }
    */
    VMM.initialize();
    // kprintln!("VMM INITIALIZED");
    SCHEDULER.initialize();
    init::initialize_app_cores();
    VMM.wait();
    // VMM.setup();
    // kprintln!("HERE");
    SCHEDULER.start();
    use aarch64::*;
    loop {
        kprintln!("CORE {}",MPIDR_EL1.get_value(MPIDR_EL1::Aff0));
    }

    kprintln!("Welcome to cs3210!");
    shell::shell("> ");
}

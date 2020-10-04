#![feature(alloc_error_handler)]
#![feature(const_fn)]
#![feature(decl_macro)]
#![feature(asm)]
#![feature(global_asm)]
#![feature(optin_builtin_traits)]
#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

#[cfg(not(test))]
mod init;

pub mod console;
pub mod mutex;
pub mod shell;

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

fn kmain() -> ! {
    unsafe {
        for i in 0..1 {
            set_led_state(1);
            spin_sleep(Duration::from_secs(1));
            set_led_state(0);
            spin_sleep(Duration::from_secs(1));
        }
    }
    // FIXME: Start the shell.
    kprintln!("Welcome to cs3210!");
    loop {
        shell::shell("> ");
    }
}

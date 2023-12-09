#![feature(asm)]
#![feature(global_asm)]

#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

#[cfg(not(test))]
mod init;

use xmodem::Xmodem;
use core::time::Duration;
use pi;
use pi::timer::spin_sleep;

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

/// Start address of the binary to load and of the bootloader.
const BINARY_START_ADDR: usize = 0x80000;
const BOOTLOADER_START_ADDR: usize = 0x4000000;

/// Pointer to where the loaded binary expects to be laoded.
const BINARY_START: *mut u8 = BINARY_START_ADDR as *mut u8;

/// Free space between the bootloader and the loaded binary's start address.
const MAX_BINARY_SIZE: usize = BOOTLOADER_START_ADDR - BINARY_START_ADDR;

/// Branches to the address `addr` unconditionally.
unsafe fn jump_to(addr: *mut u8) -> ! {
    asm!("br $0" : : "r"(addr as usize));
    loop {
        asm!("wfe" :::: "volatile")
    }
}

fn kmain() -> ! {
    for i in 0..1 {
        unsafe {
            set_led_state(1);
            spin_sleep(Duration::from_secs(1));
            set_led_state(0);
            spin_sleep(Duration::from_secs(1));
        }
    }
    // FIXME: Implement the bootloader.
    let mut mu = pi::uart::MiniUart::new();
    mu.set_read_timeout(Duration::from_millis(750));
    let mut bin = unsafe { core::slice::from_raw_parts_mut(BINARY_START, MAX_BINARY_SIZE) };
    loop {
        /*
        while mu.has_byte() {
            let _ = mu.read_byte();
        }
        */
        match Xmodem::receive(&mut mu , &mut bin) {
            Ok(_) => unsafe { jump_to(BINARY_START); },
            Err(_) => continue,
        }
    }
}

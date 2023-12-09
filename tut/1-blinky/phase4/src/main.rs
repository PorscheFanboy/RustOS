#![feature(asm)]
#![feature(global_asm)]

#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

#[cfg(not(test))]
mod init;

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


#[inline(never)]
fn spin_sleep_ms(ms: usize) {
    for _ in 0..(ms * 6000) {
        unsafe { asm!("nop" :::: "volatile"); }
    }
}

unsafe fn kmain() -> ! {
    // FIXME: STEP 1: Set GPIO Pin 16 as output.
    // FIXME: STEP 2: Continuously set and clear GPIO 16.
    loop {
        set_led_state(1);
        spin_sleep_ms(2000);
        set_led_state(0);
        spin_sleep_ms(2000);
    }
}

use core::mem::zeroed;
use core::panic::PanicInfo;
use core::ptr::write_volatile;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    use kernel_api::println;
    if let Some(s) = _info.payload().downcast_ref::<&str>() {
        println!("panic occurred: {:?}", s);
    } else {
        println!("panic occurred");
    }
    if let Some(location) = _info.location() {
        println!("panic occurred in file '{}' at line {}", location.file(),
            location.line());
    } else {
        println!("No idea where the panic occured");
    }
    loop {}
}

unsafe fn zeros_bss() {
    extern "C" {
        static mut __bss_beg: u64;
        static mut __bss_end: u64;
    }

    let mut iter: *mut u64 = &mut __bss_beg;
    let end: *mut u64 = &mut __bss_end;

    while iter < end {
        write_volatile(iter, zeroed());
        iter = iter.add(1);
    }
}

#[no_mangle]
pub unsafe extern "C" fn _start() -> ! {
    zeros_bss();
    crate::main();
    kernel_api::syscall::exit();
}

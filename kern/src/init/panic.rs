use core::panic::PanicInfo;
use crate::console::kprintln;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    kprintln!("    The pi is overdone.");
    kprintln!("");
    kprintln!("---------- PANIC ----------");
    if let Some(s) = _info.payload().downcast_ref::<&str>() {
        kprintln!("panic occurred: {:?}", s);
    } else {
        kprintln!("panic occurred");
    }
    if let Some(location) = _info.location() {
        kprintln!("panic occurred in file '{}' at line {}", location.file(),
            location.line());
    } else {
        kprintln!("No idea where the panic occured");
    }
    loop {}
}

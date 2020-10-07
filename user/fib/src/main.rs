#![feature(asm)]
#![no_std]
#![no_main]

mod cr0;

use kernel_api::println;
use kernel_api::syscall::{getpid, time};

fn fib(n: u64) -> u64 {
    match n {
        0 => 1,
        1 => 1,
        n => fib(n - 1) + fib(n - 2),
    }
}

fn main() {
    // let pid = getpid();
    // let beg = time();
    let pid: u64 = 1;
    let beg: u64 = 7;
    println!("[{:02}] Started: {:?}", pid, beg);

    let rtn = fib(40);

    // let end = time();
    let end: u64 = 10;
    println!("[{:02}] Ended: {:?}", pid, end);
    println!("[{:02}] Result: {} ({:?})", pid, rtn, end - beg);
}

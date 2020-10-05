use shim::io;
use shim::path::{Path, PathBuf};

use stack_vec::StackVec;

use pi::atags::Atags;

use fat32::traits::FileSystem;
use fat32::traits::{Dir, Entry};

use crate::console::{kprint, kprintln, CONSOLE};
use crate::ALLOCATOR;
use crate::FILESYSTEM;

/// Error type for `Command` parse failures.
#[derive(Debug)]
enum Error {
    Empty,
    TooManyArgs,
}

/// A structure representing a single shell command.
struct Command<'a> {
    args: StackVec<'a, &'a str>,
}

impl<'a> Command<'a> {
    /// Parse a command from a string `s` using `buf` as storage for the
    /// arguments.
    ///
    /// # Errors
    ///
    /// If `s` contains no arguments, returns `Error::Empty`. If there are more
    /// arguments than `buf` can hold, returns `Error::TooManyArgs`.
    fn parse(s: &'a str, buf: &'a mut [&'a str]) -> Result<Command<'a>, Error> {
        let mut args = StackVec::new(buf);
        for arg in s.split(' ').filter(|a| !a.is_empty()) {
            args.push(arg).map_err(|_| Error::TooManyArgs)?;
        }

        if args.is_empty() {
            return Err(Error::Empty);
        }

        Ok(Command { args })
    }

    /// Returns this command's path. This is equivalent to the first argument.
    fn path(&self) -> &str {
        return self.args[0];
    }
}

/// Starts a shell using `prefix` as the prefix for each line. This function
/// never returns.
pub fn shell(prefix: &str) -> ! {
    let mut console = CONSOLE.lock();
    loop {
        let mut buffer = [""; 520];
        let mut s : [u8; 520] = [0u8; 520];
        kprint!("{}", prefix);
        let mut idx: usize = 0;
        loop {
            let b = console.read_byte();
            if b == b'\r' || b == b'\n' {
                s[idx] = ' ' as u8;
                break;
            }
            if b == 8 {
                kprint!("{}", 8 as char);
                kprint!("{}", ' ');
                kprint!("{}", 8 as char);
                idx -= 1;
                s[idx] = 0;
                continue;
            }
            kprint!("{}", b as char);
            s[idx] = b;
            idx += 1;
        }
        let ss = &core::str::from_utf8(&s).unwrap()[..];
        match Command::parse(ss, &mut buffer) {
            Err(_) => {
                kprintln!();
                kprintln!("ASDF");
            },
            Ok(cmd) => {
                match cmd.path() {
                    "exit" => (),
                    "echo" => {
                        kprintln!();
                        for i in 1..cmd.args.len() {
                            kprint!("{} ", cmd.args[i]);
                        }
                        kprintln!();
                    },
                    _ => {
                        kprintln!();
                        kprintln!("unknown command {}", cmd.path());
                    }
                }
            },
        }
    }
}

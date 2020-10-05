use crate::atags::raw;

pub use crate::atags::raw::{Core, Mem};

/// An ATAG.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Atag {
    Core(raw::Core),
    Mem(raw::Mem),
    Cmd(&'static str),
    Unknown(u32),
    None,
}

impl Atag {
    /// Returns `Some` if this is a `Core` ATAG. Otherwise returns `None`.
    pub fn core(self) -> Option<Core> {
        match self {
            Atag::Core(c) => Some(c),
            _ => None,
        }
    }

    /// Returns `Some` if this is a `Mem` ATAG. Otherwise returns `None`.
    pub fn mem(self) -> Option<Mem> {
        match self {
            Atag::Mem(m) => Some(m),
            _ => None,
        }
    }

    /// Returns `Some` with the command line string if this is a `Cmd` ATAG.
    /// Otherwise returns `None`.
    pub fn cmd(self) -> Option<&'static str> {
        match self {
            Atag::Cmd(c) => Some(c),
            _ => None,
        }
    }
}

// FIXME: Implement `From<&raw::Atag> for `Atag`.
impl From<&'static raw::Atag> for Atag {
    fn from(atag: &'static raw::Atag) -> Atag {
        // FIXME: Complete the implementation below.

        unsafe {
            match (atag.tag, &atag.kind) {
                (raw::Atag::CORE, &raw::Kind { core }) => Atag::Core(core),
                (raw::Atag::MEM, &raw::Kind { mem }) => Atag::Mem(mem),
                (raw::Atag::CMDLINE, &raw::Kind { ref cmd }) => {
                    let mut ptr = &(cmd.cmd) as *const u8;
                    let mut sz = 0;
                    while *ptr != 0 {
                        ptr = ptr.add(1);
                        sz += 1;
                    }
                    return Atag::Cmd(core::str::from_utf8_unchecked(core::slice::from_raw_parts(&(cmd.cmd) as *const u8, sz)));
                },
                (raw::Atag::NONE, _) => Atag::None,
                (id, _) => Atag::Unknown(id),
            }
        }
    }
}

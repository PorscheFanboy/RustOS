use alloc::boxed::Box;
use alloc::vec::Vec;
use shim::io;
use shim::path::Path;

use aarch64;
use smoltcp::socket::SocketHandle;

use crate::param::*;
use crate::process::{Stack, State};
use crate::traps::TrapFrame;
use crate::vm::*;
use kernel_api::{OsError, OsResult};

use core::mem::replace;
use crate::FILESYSTEM;

/// Type alias for the type of a process ID.
pub type Id = u64;

/// A structure that represents the complete state of a process.
#[derive(Debug)]
pub struct Process {
    /// The saved trap frame of a process.
    pub context: Box<TrapFrame>,
    /// The memory allocation used for the process's stack.
    pub stack: Stack,
    /// The page table describing the Virtual Memory of the process
    pub vmap: Box<UserPageTable>,
    /// The scheduling state of the process.
    pub state: State,
    // Lab 5 2.C
    // Socket handles held by the current process
    // pub sockets: Vec<SocketHandle>,
}

impl Process {
    /// Creates a new process with a zeroed `TrapFrame` (the default), a zeroed
    /// stack of the default size, and a state of `Ready`.
    ///
    /// If enough memory could not be allocated to start the process, returns
    /// `None`. Otherwise returns `Some` of the new `Process`.
    pub fn new() -> OsResult<Process> {
        let tf = TrapFrame{
            ttbr0_el: 0,
            ttbr1_el: 0,
            elr_el: 0,
            spsr_el: 0,
            sp_el: 0,
            tpidr_el: 0,
            qs: [0; 32],
            xs: [0; 32],
        };
        return Ok(Process{
            context: Box::<TrapFrame>::new(tf),
            stack: Stack::new().unwrap(),
            vmap: Box::new(UserPageTable::new()),
            state: State::Ready,
        });
    }

    /// Loads a program stored in the given path by calling `do_load()` method.
    /// Sets trapframe `context` corresponding to its page table.
    /// `sp` - the address of stack top
    /// `elr` - the address of image base.
    /// `ttbr0` - the base address of kernel page table
    /// `ttbr1` - the base address of user page table
    /// `spsr` - `F`, `A`, `D` bit should be set.
    ///
    /// Returns Os Error if do_load fails.
    pub fn load<P: AsRef<Path>>(pn: P) -> OsResult<Process> {
        use crate::VMM;

        let mut p = Process::do_load(pn)?;

        // FIXME: Set trapframe for the process.
        p.context.elr_el = Process::get_image_base().as_u64();
        p.context.ttbr0_el = VMM.get_baddr().as_u64();
        p.context.ttbr1_el = p.vmap.get_baddr().as_u64();
        p.context.sp_el = Process::get_stack_base().as_u64();

        Ok(p)
    }

    /// Creates a process and open a file with given path.
    /// Allocates one page for stack with read/write permission, and N pages with read/write/execute
    /// permission to load file's contents.
    fn do_load<P: AsRef<Path>>(pn: P) -> OsResult<Process> {
        use fat32::traits::{FileSystem, Entry};
        use shim::io::Read;
        let entry = FILESYSTEM.open(pn.as_ref()).unwrap();
        let mut f = entry.into_file().unwrap();
        let mut buf: [u8; 10000] = [0; 10000];
        f.read(&mut buf);
        let mut p = Process::new().unwrap();
        // let stack = p.vmap.alloc(Process::get_stack_base() - VirtualAddr::from(PAGE_SIZE), PagePerm::RW);
        for i in 0..16 {
            p.vmap.alloc(Process::get_stack_base() - VirtualAddr::from(PAGE_SIZE * i), PagePerm::RW);
        }
        let mut idx: usize = 0;
        for i in 0..1000 {
            if idx == f.size as usize {
                return Ok(p);
            }
            let ptr = p.vmap.alloc(Process::get_image_base() + VirtualAddr::from(PAGE_SIZE * i as usize), PagePerm::RWX);
            for j in 0..PAGE_SIZE {
                if idx == f.size as usize {
                    return Ok(p);
                }
                ptr[j] = buf[idx];
                idx += 1;
            }
        }
        return Ok(p);
    }

    /// Returns the highest `VirtualAddr` that is supported by this system.
    pub fn get_max_va() -> VirtualAddr {
        return VirtualAddr::from(USER_IMG_BASE) + VirtualAddr::from(USER_MAX_VM_SIZE);
    }

    /// Returns the `VirtualAddr` represents the base address of the user
    /// memory space.
    pub fn get_image_base() -> VirtualAddr {
        return VirtualAddr::from(USER_IMG_BASE);
    }

    /// Returns the `VirtualAddr` represents the base address of the user
    /// process's stack.
    pub fn get_stack_base() -> VirtualAddr {
        return VirtualAddr::from(USER_STACK_BASE);
    }

    /// Returns the `VirtualAddr` represents the top of the user process's
    /// stack.
    pub fn get_stack_top() -> VirtualAddr {
        return VirtualAddr::from(USER_STACK_BASE) - VirtualAddr::from(Stack::SIZE as u64);
    }

    /// Returns `true` if this process is ready to be scheduled.
    ///
    /// This functions returns `true` only if one of the following holds:
    ///
    ///   * The state is currently `Ready`.
    ///
    ///   * An event being waited for has arrived.
    ///
    ///     If the process is currently waiting, the corresponding event
    ///     function is polled to determine if the event being waiting for has
    ///     occured. If it has, the state is switched to `Ready` and this
    ///     function returns `true`.
    ///
    /// Returns `false` in all other cases.
    pub fn is_ready(&mut self) -> bool {
        let mut state = replace(&mut self.state, State::Ready);
        match &mut state {
            State::Ready => {
                return true;
            },
            State::Waiting(poll_fun) => {
                if poll_fun(self) {
                    return true;
                } else {
                    replace(&mut self.state, state);
                    return false;
                }
            }
            _ => {
                replace(&mut self.state, state);
                return false;
            }
        }
    }
}

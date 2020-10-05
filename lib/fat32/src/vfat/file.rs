use alloc::string::String;
use alloc::vec::Vec;

use shim::io::{self, SeekFrom};

use crate::traits;
use crate::vfat::{Cluster, Metadata, VFatHandle};


#[derive(Debug)]
pub struct File<HANDLE: VFatHandle> {
    pub vfat: HANDLE,
    // FIXME: Fill me in.
    pub first_cluster: Cluster,
    pub name: String,
    pub metadata: Metadata,
    pub size: u32,
    pub read_idx: usize,
    pub content: Vec<u8>,
    pub already_read: bool,
}

impl<HANDLE: VFatHandle> io::Write for File<HANDLE> {
    fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
        panic!("Dummy")
    }
    fn flush(&mut self) -> io::Result<()> {
        panic!("Dummy")
    }
}

impl<HANDLE: VFatHandle> io::Read for File<HANDLE> {
    fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
        let mut vv: Vec<u8> = Vec::new(); 
        if !self.already_read {
            let call = |x: &mut crate::vfat::vfat::VFat<HANDLE>| {
                x.read_chain(self.first_cluster, &mut vv);
            };
            self.vfat.lock(call);
            self.content = vv.clone();
            self.already_read = true;
        }
        let mut cnt = 0;
        for i in 0.._buf.len() {
            if self.read_idx >= self.size as usize {
                break;
            }
            cnt += 1;
            _buf[i] = self.content[self.read_idx];
            self.read_idx += 1;
        }
        Ok(cnt)
    }
}

// FIXME: Implement `traits::File` (and its supertraits) for `File`.
impl<HANDLE: VFatHandle> traits::File for File<HANDLE> {
    fn sync(&mut self) -> io::Result<()> {
        panic!("Dummy")
    }

    fn size(&self) -> u64 {
        self.size as u64
    }
}


impl<HANDLE: VFatHandle> io::Seek for File<HANDLE> {
    /// Seek to offset `pos` in the file.
    ///
    /// A seek to the end of the file is allowed. A seek _beyond_ the end of the
    /// file returns an `InvalidInput` error.
    ///
    /// If the seek operation completes successfully, this method returns the
    /// new position from the start of the stream. That position can be used
    /// later with SeekFrom::Start.
    ///
    /// # Errors
    ///
    /// Seeking before the start of a file or beyond the end of the file results
    /// in an `InvalidInput` error.
    fn seek(&mut self, _pos: SeekFrom) -> io::Result<u64> {
        unimplemented!("File::seek()")
    }
}

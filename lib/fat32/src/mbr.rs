use core::fmt;
use shim::const_assert_size;
use shim::io;

use crate::traits::BlockDevice;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct CHS {
    // FIXME: Fill me in.
    head: u8,
    sector_cylinder: [u8; 2],
}

// FIXME: implement Debug for CHS
impl fmt::Debug for CHS {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("CHS")
            .finish()
    }
}

const_assert_size!(CHS, 3);

#[repr(C, packed)]
pub struct PartitionEntry {
    // FIXME: Fill me in.
    bootable: u8,
    starting_chs: CHS,
    partition_type: u8,
    ending_chd: CHS,
    pub relative_sector: u32,
    pub total_sectors: u32,
}

// FIXME: implement Debug for PartitionEntry
impl fmt::Debug for PartitionEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("PartitionEntry")
            .field("bootable", &self.bootable)
            .field("partition_type", &self.partition_type)
            .field("relative_sector", &self.relative_sector)
            .field("total_sectors", &self.total_sectors)
            .finish()
    }
}

const_assert_size!(PartitionEntry, 16);

/// The master boot record (MBR).
#[repr(C, packed)]
pub struct MasterBootRecord {
    // FIXME: Fill me in.
    bootstrap: [u8; 436],
    disk_id: [u8; 10],
    pub partition_table: [PartitionEntry; 4],
    valid_bootsector: [u8; 2],
}

// FIXME: implemente Debug for MaterBootRecord
impl fmt::Debug for MasterBootRecord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("MasterBootRecord")
            .field("disk_id", &self.disk_id)
            .field("partition_table", &self.partition_table)
            .field("valid_bootsector", &self.valid_bootsector)
            .finish()
    }
}

const_assert_size!(MasterBootRecord, 512);

#[derive(Debug)]
pub enum Error {
    /// There was an I/O error while reading the MBR.
    Io(io::Error),
    /// Partiion `.0` (0-indexed) contains an invalid or unknown boot indicator.
    UnknownBootIndicator(u8),
    /// The MBR magic signature was invalid.
    BadSignature,
}

impl MasterBootRecord {
    /// Reads and returns the master boot record (MBR) from `device`.
    ///
    /// # Errors
    ///
    /// Returns `BadSignature` if the MBR contains an invalid magic signature.
    /// Returns `UnknownBootIndicator(n)` if partition `n` contains an invalid
    /// boot indicator. Returns `Io(err)` if the I/O error `err` occured while
    /// reading the MBR.
    pub fn from<T: BlockDevice>(mut device: T) -> Result<MasterBootRecord, Error> {
        let mut buf = [0u8; 512];
        device.read_sector(0, &mut buf);
        /*
        let p = (&buf[..]).as_ptr() as *const MasterBootRecord;
        let m_ref: &MasterBootRecord = unsafe { &*p };
        let mbr = unsafe { core::mem::transmute_copy::<MasterBootRecord, MasterBootRecord>(m_ref) };
        */
        let mbr = unsafe { core::mem::transmute::<[u8; 512], MasterBootRecord>(buf) };
        if mbr.valid_bootsector[0] != 0x55 || mbr.valid_bootsector[1] != 0xAA {
            return Err(Error::BadSignature);
        }
        for i in 0..4 {
            let boot_ind = mbr.partition_table[i].bootable;
            if boot_ind != 0x80 && boot_ind != 0 { //????
                return Err(Error::UnknownBootIndicator(i as u8));
            }
        }
        return Ok(mbr);
    }
}

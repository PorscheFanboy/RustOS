use crate::vfat::*;
use core::fmt;

use self::Status::*;

#[derive(Debug, PartialEq)]
pub enum Status {
    /// The FAT entry corresponds to an unused (free) cluster.
    Free,
    /// The FAT entry/cluster is reserved.
    Reserved,
    /// The FAT entry corresponds to a valid data cluster. The next cluster in
    /// the chain is `Cluster`.
    Data(Cluster),
    /// The FAT entry corresponds to a bad (disk failed) cluster.
    Bad,
    /// The FAT entry corresponds to a valid data cluster. The corresponding
    /// cluster is the last in its chain.
    Eoc(u32),
}

#[repr(C, packed)]
pub struct FatEntry(pub u32);

impl FatEntry {
    /// Returns the `Status` of the FAT entry `self`.
    pub fn status(&self) -> Status {
        let flag = 0xFFFFFFF;
        if self.0 & flag == 0 {
            return Free;
        } else if (self.0 & flag) == 1 {
            return Reserved;
        } else if (self.0 & flag) == 0xFFFFFF7 {
            return Bad;
        } else if (self.0 & flag) >= 0xFFFFFF8 && (self.0 & flag) <= 0xFFFFFFF {
            return Eoc(self.0 & flag);
        } else {
            return Data(Cluster::from(self.0));
        }
    }
}

impl fmt::Debug for FatEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("FatEntry")
            .field("value", &{ self.0 })
            .field("status", &self.status())
            .finish()
    }
}

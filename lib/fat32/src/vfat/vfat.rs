use core::fmt::Debug;
use core::marker::PhantomData;
use core::mem::size_of;
use core::ops::{Deref, DerefMut};

use alloc::vec::Vec;

use shim::io;
use shim::ioerr;
use shim::newioerr;
use shim::path;
use shim::path::Path;

use crate::alloc::string::ToString;
use crate::mbr::MasterBootRecord;
use crate::traits::{BlockDevice, FileSystem};
use crate::util::SliceExt;
use crate::vfat::{BiosParameterBlock, CachedPartition, Partition};
use crate::vfat::{Cluster, Dir, Entry, Error, FatEntry, File, Status};

/// A generic trait that handles a critical section as a closure
pub trait VFatHandle: Clone + Debug + Send + Sync {
    fn new(val: VFat<Self>) -> Self;
    fn lock<R>(&self, f: impl FnOnce(&mut VFat<Self>) -> R) -> R;
}

#[derive(Debug)]
pub struct VFat<HANDLE: VFatHandle> {
    phantom: PhantomData<HANDLE>,
    pub device: CachedPartition,
    pub bytes_per_sector: u16,
    pub sectors_per_cluster: u8,
    pub sectors_per_fat: u32,
    pub fat_start_sector: u64,
    data_start_sector: u64,
    rootdir_cluster: Cluster,
}

impl<HANDLE: VFatHandle> VFat<HANDLE> {
    pub fn from<T>(mut device: T) -> Result<HANDLE, Error>
    where
        T: BlockDevice + 'static,
    {
        let mbr = MasterBootRecord::from(&mut device)?;
        let bpb = BiosParameterBlock::from(&mut device, mbr.partition_table[0].relative_sector as u64)?;
        let mut sectors = bpb.logical_sectors as u32;
        if bpb.logical_sectors == 0 {
            sectors = bpb.tot_logical_sectors;
        }
        let part = Partition {
            start: mbr.partition_table[0].relative_sector as u64,
            num_sectors: sectors as u64, // ??????
            sector_size: bpb.bytes_per_sector as u64,
        };
        let vf = VFat {
            phantom: PhantomData,
            device: CachedPartition::new(device, part),
            bytes_per_sector: bpb.bytes_per_sector,
            sectors_per_cluster: bpb.sectors_per_cluster,
            sectors_per_fat: bpb.e_sectors_per_fat,
            fat_start_sector: bpb.reserved_sectors as u64,
            data_start_sector: bpb.e_sectors_per_fat as u64 * bpb.fats as u64 + bpb.reserved_sectors as u64, // ???? 
            rootdir_cluster: Cluster::from(bpb.root_cluster),
        };
        return Ok(VFatHandle::new(vf));
    }

    // TODO: The following methods may be useful here:
    //
    //  * A method to read from an offset of a cluster into a buffer.
    //
    fn read_cluster(&mut self, cluster: Cluster, offset: usize, buf: &mut [u8]) -> io::Result<usize> {
        /*
        if cluster.0 < 2 {
            return Ok(0);
        }
        */
        let start = cluster.to_sector(self.data_start_sector, self.sectors_per_cluster.into()); 
        let mut cnt = 0;
        for i in 0..self.sectors_per_cluster as usize {
            let v = self.device.get(start + i as u64).unwrap();
            for j in 0..self.bytes_per_sector as usize {
                let idx = i * self.bytes_per_sector as usize + j;
                /*
                if idx >= buf.len() {
                    panic!("idx {} > buflen {}", idx, buf.len());
                }
                if j >= v.len() {
                    panic!("j > v len");
                }
                */
                buf[idx] = v[j];
                cnt += 1;
            }
        }
        Ok(cnt)
    }

    //
    //  * A method to read all of the clusters chained from a starting cluster
    //    into a vector.
    pub fn read_chain(&mut self, start: Cluster, buf: &mut Vec<u8>) -> io::Result<usize> {
        let mut cur = start;
        let mut size = 0;
        let cluster_size = self.sectors_per_cluster.clone() as usize* self.bytes_per_sector.clone() as usize;
        loop {
            let fat_entry = self.fat_entry(cur);
            if cur.0 < 2 {
                break;
            }
            // let mut buffer: [u8; 40000] = [0; 40000];
            let mut buffer: Vec<u8> = vec![0; cluster_size];
            match fat_entry.unwrap().status() {
                Status::Eoc(_) => {
                    let sz = self.read_cluster(cur, 0, &mut buffer).unwrap();
                    buf.extend_from_slice(&buffer[..sz]);
                    size += sz;
                    break;
                },
                Status::Data(c) => {
                    let sz = self.read_cluster(cur, 0, &mut buffer).unwrap();
                    buf.extend_from_slice(&buffer[..sz]);
                    size += sz;
                    cur = c;
                },
                _ => {
                    break;
                }
            }
        }
        Ok(size)
    }

    //  * A method to return a reference to a `FatEntry` for a cluster where the
    //    reference points directly into a cached sector.
    //
    //    fn fat_entry(&mut self, cluster: Cluster) -> io::Result<&FatEntry>;
    fn fat_entry(&mut self, cluster: Cluster) -> io::Result<&FatEntry> {
        // let sector = self.fat_start_sector + cluster.0 * 4 / self.bytes_per_sector; // FIX
        let sector = cluster.to_fatentry_sector(self.fat_start_sector, self.bytes_per_sector as u64);
        let buff = self.device.get(sector as u64)?;
        // let idx = cluster.to_sector(self.fat_start_sector) as usize;
        let idx = cluster.index_in_sector(self.bytes_per_sector as u64);
        let b = &buff[idx] as *const u8;
        let bb = b as *const FatEntry;
        // let a = unsafe{ &*bb };
        return unsafe { Ok(&*bb) };
    }

}

impl<'a, HANDLE: VFatHandle> FileSystem for &'a HANDLE {
    type File = File<HANDLE>;
    type Dir = Dir<HANDLE>;
    type Entry = Entry<HANDLE>;

    fn open<P: AsRef<Path>>(self, path: P) -> io::Result<Self::Entry> {
        let components = path.as_ref().components();
        let mut root_cluster: Cluster = Cluster::from(0);
        let call = |x: &mut crate::vfat::vfat::VFat<HANDLE>| {
            root_cluster = x.rootdir_cluster;
        };
        self.lock(call);
        
        use crate::vfat::{Attributes, Date, Metadata, Time, Timestamp};
        
        let metadata = Metadata {
            created: Timestamp {
                date: Date(0),
                time: Time(0),
            },
            accessed: Timestamp {
                date: Date(0),
                time: Time(0),
            },
            modified: Timestamp {
                date: Date(0),
                time: Time(0),
            },
            attr: Attributes(0),
        };

        let mut entry = Entry::Dossier(Dir {
            vfat: self.clone(),
            first_cluster: root_cluster,
            name: "root".to_string(),
            metadata: metadata,
            size: 0,
        });

        let mut cnt = 0;
        for comp in components {
            if cnt == 0 {
                cnt = 1;
                continue;
            }
            match &entry {
                Entry::Dossier(d) => {
                    match d.find(comp) {
                        Err(_) => return Err(io::Error::new(io::ErrorKind::NotFound, "not found")),
                        Ok(e) => entry = e,
                    }
                    // entry = d.find(comp).unwrap();
                },
                Entry::Fichier(_) => {
                    return Ok(entry);
                }
            }
        }
        Ok(entry)
        // Err(io::Error::new(io::ErrorKind::NotFound, "not found"))
    }
}

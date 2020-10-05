use alloc::string::String;
use crate::alloc::string::ToString;
use alloc::vec::Vec;

use shim::const_assert_size;
use shim::ffi::OsStr;
use shim::io;
use shim::newioerr;

use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};

use crate::traits;
use crate::util::VecExt;
use crate::vfat::{Attributes, Date, Metadata, Time, Timestamp};
use crate::vfat::{Cluster, Entry, File, VFatHandle};
use crate::vfat::{BiosParameterBlock, CachedPartition, Partition};

#[derive(Debug)]
pub struct Dir<HANDLE: VFatHandle> {
    pub vfat: HANDLE,
    // FIXME: Fill me in.
    pub first_cluster: Cluster,
    pub name: String,
    pub metadata: Metadata,
    pub size: u32,
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct VFatRegularDirEntry {
    // FIXME: Fill me in.
    file_name: [u8; 8],
    file_ext: [u8; 3],
    attribute: u8,
    win_nt: u8,
    creation_time_tsec: u8,
    creation_time_sec: u16,
    creation_date: u16,
    last_acc_date: u16,
    cluster_high_bits: u16,
    last_mod_time: u16,
    last_mod_date: u16,
    cluster_low_bits: u16,
    file_size: u32,
}

const_assert_size!(VFatRegularDirEntry, 32);

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct VFatLfnDirEntry {
    // FIXME: Fill me in.
    sequence_num: u8,
    name_chars: [u16; 5],
    attributes: u8,
    file_type: u8,
    checksum: u8,
    second_name_chars: [u16; 6],
    zeroes: u16,
    third_name_chars: [u16; 2],
}

const_assert_size!(VFatLfnDirEntry, 32);

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct VFatUnknownDirEntry {
    // FIXME: Fill me in.
    content: [u8; 32],
}

const_assert_size!(VFatUnknownDirEntry, 32);

pub union VFatDirEntry {
    unknown: VFatUnknownDirEntry,
    regular: VFatRegularDirEntry,
    long_filename: VFatLfnDirEntry,
}

impl<HANDLE: VFatHandle> Dir<HANDLE> {
    /// Finds the entry named `name` in `self` and returns it. Comparison is
    /// case-insensitive.
    ///
    /// # Errors
    ///
    /// If no entry with name `name` exists in `self`, an error of `NotFound` is
    /// returned.
    ///
    /// If `name` contains invalid UTF-8 characters, an error of `InvalidInput`
    /// is returned.
    pub fn find<P: AsRef<OsStr>>(&self, name: P) -> io::Result<Entry<HANDLE>> {
        use crate::traits::Dir;
        for entry in self.entries()? {
            match &entry {
                Entry::Dossier(dir) => {
                    if dir.name.eq_ignore_ascii_case(name.as_ref().to_str().unwrap()) {
                        return Ok(entry);
                    }
                },
                Entry::Fichier(file) => {
                    if file.name.eq_ignore_ascii_case(name.as_ref().to_str().unwrap()) {
                        return Ok(entry);
                    }
                },
            }
        }
        return Err(io::Error::new(io::ErrorKind::NotFound, "not found"));
    }

}

pub struct EntryIterator<HANDLE: VFatHandle> {
    phantom: PhantomData<HANDLE>,
    vfat: HANDLE,
    v: Vec<VFatDirEntry>,
    idx: usize,
}

impl<HANDLE: VFatHandle> Iterator for EntryIterator<HANDLE> {
    type Item = Entry<HANDLE>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.v.len() {
            return None;
        }
        let mut entry = unsafe { self.v[self.idx].unknown };
        if entry.content[0] == 0 {
            return None;
        }
        while entry.content[0] == 0xE5 {
            self.idx += 1;
            entry = unsafe { self.v[self.idx].unknown };
        }
        if entry.content[11] == 0xF { // Long file name entry
            let mut vec_entries: Vec<VFatLfnDirEntry> = Vec::new();

            while entry.content[11] == 0xF {
                let lfn_entry = unsafe { self.v[self.idx].long_filename };
                if lfn_entry.sequence_num == 0 {
                    self.idx += 1;
                    vec_entries.push(lfn_entry);
                    break;
                }
                vec_entries.push(lfn_entry);
                self.idx += 1;
                entry = unsafe { self.v[self.idx].unknown };
            }
            vec_entries.sort_by(|a, b| b.sequence_num.partial_cmp(&a.sequence_num).unwrap());
            let mut name: String = "".to_string();
            for e in vec_entries {
                let mut vec: Vec<u16> = Vec::new();
                for i in 0..5 {
                    let c = e.name_chars[i];
                    if c == 0 || c  & 0xFF00 == 0xFF00 {
                        break;
                    }
                    vec.push(c);
                }
                for i in 0..6 {
                    let c = e.second_name_chars[i];
                    if c == 0 || c  & 0xFF00 == 0xFF00 {
                        break;
                    }
                    vec.push(c);
                }
                for i in 0..2 {
                    let c = e.third_name_chars[i];
                    if c == 0 || c  & 0xFF00 == 0xFF00 {
                        break;
                    }
                    vec.push(c);
                }
                use core::char::{decode_utf16, REPLACEMENT_CHARACTER};
                let lfn_name: String = decode_utf16(vec.iter().cloned()).
                    map(|r| r.unwrap_or(REPLACEMENT_CHARACTER)).
                    collect::<String>();
                name = lfn_name + &name;
            }
            let reg_entry = unsafe { self.v[self.idx].regular };
            self.idx += 1;

            let metadata = Metadata {
                created: Timestamp {
                    date: Date(reg_entry.creation_date),
                    time: Time(reg_entry.creation_time_sec),
                },
                accessed: Timestamp {
                    date: Date(reg_entry.last_acc_date),
                    time: Time(0),
                },
                modified: Timestamp {
                    date: Date(reg_entry.last_mod_date),
                    time: Time(reg_entry.last_mod_time),
                },
                attr: Attributes(reg_entry.attribute),
            };
            if reg_entry.attribute & 0x10 > 0 { // Directory
                return Some(Entry::Dossier(Dir {
                    vfat: self.vfat.clone(),
                    first_cluster: Cluster::from(((reg_entry.cluster_high_bits as u64) << 16) as u32 + reg_entry.cluster_low_bits as u32),
                    name: name,
                    metadata: metadata,
                    size: reg_entry.file_size,
                }));

            } else { // File
                return Some(Entry::Fichier(File {
                    vfat: self.vfat.clone(),
                    first_cluster: Cluster::from(((reg_entry.cluster_high_bits as u64) << 16) as u32 + reg_entry.cluster_low_bits as u32),
                    name: name,
                    metadata: metadata,
                    size: reg_entry.file_size,
                    read_idx: 0,
                    content: Vec::new(),
                    already_read: false,
                }));
            }
        } else {
            let reg_entry = unsafe { self.v[self.idx].regular };
            self.idx += 1;
            let mut name_vec = reg_entry.file_name.to_vec();
            while name_vec[name_vec.len() - 1] == 0x00 || name_vec[name_vec.len() - 1] == 0x20 {
                name_vec.pop();
            }
            let mut ext_vec = reg_entry.file_ext.to_vec();
            while ext_vec.len() > 0 && (ext_vec[ext_vec.len() - 1] == 0x00 || ext_vec[ext_vec.len() - 1] == 0x20) {
                ext_vec.pop();
            }
            let name: String = String::from_utf8(name_vec).unwrap();
            let ext: String = String::from_utf8(ext_vec).unwrap();

            let metadata = Metadata {
                created: Timestamp {
                    date: Date(reg_entry.creation_date),
                    time: Time(reg_entry.creation_time_sec),
                },
                accessed: Timestamp {
                    date: Date(reg_entry.last_acc_date),
                    time: Time(0),
                },
                modified: Timestamp {
                    date: Date(reg_entry.last_mod_date),
                    time: Time(reg_entry.last_mod_time),
                },
                attr: Attributes(reg_entry.attribute),
            };
            if reg_entry.attribute & 0x10 > 0 { // Directory
                let full_name;
                if ext != "".to_string() {
                    full_name = name + &".".to_string() + &ext;
                } else {
                    full_name = name;
                }
                return Some(Entry::Dossier(Dir {
                    vfat: self.vfat.clone(),
                    first_cluster: Cluster::from(((reg_entry.cluster_high_bits as u64) << 16) as u32 + reg_entry.cluster_low_bits as u32),
                    name: full_name,
                    metadata: metadata,
                    size: reg_entry.file_size,
                }));

            } else { // File
                let full_name: String;
                if ext != "".to_string() {
                    full_name = name + &".".to_string() + &ext;
                } else {
                    full_name = name;
                }
                return Some(Entry::Fichier(File {
                    vfat: self.vfat.clone(),
                    first_cluster: Cluster::from(((reg_entry.cluster_high_bits as u64) << 16) as u32 + reg_entry.cluster_low_bits as u32),
                    name: full_name,
                    metadata: metadata,
                    size: reg_entry.file_size,
                    read_idx: 0,
                    content: Vec::new(),
                    already_read: false,
                }));
            }
        }

    }

}



impl<HANDLE: VFatHandle> traits::Dir for Dir<HANDLE> {
    // FIXME: Implement `trait::Dir` for `Dir`.
    type Entry = Entry<HANDLE>;
    type Iter = EntryIterator<HANDLE>;

    /// Returns an interator over the entries in this directory.
    fn entries(&self) -> io::Result<Self::Iter> {
        let mut vv: Vec<u8> = Vec::new(); 
        let call = |x: &mut crate::vfat::vfat::VFat<HANDLE>| {
            x.read_chain(self.first_cluster, &mut vv);
        };
        self.vfat.lock(call);
        let v: Vec<VFatDirEntry> = unsafe { vv.cast() };
        Ok(EntryIterator {
            phantom: PhantomData,
            vfat: self.vfat.clone(),
            v: v,
            idx: 0,
        })

    }
}

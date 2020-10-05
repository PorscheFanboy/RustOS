use core::fmt;

use alloc::string::String;

use crate::traits;

/// A date as represented in FAT32 on-disk structures.
#[repr(C, packed)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Date(pub u16);

/// Time as represented in FAT32 on-disk structures.
#[repr(C, packed)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Time(pub u16);

/// File attributes as represented in FAT32 on-disk structures.
#[repr(C, packed)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Attributes(pub u8);

/// A structure containing a date and time.
#[derive(Default, Copy, Clone, Debug, PartialEq, Eq)]
pub struct Timestamp {
    pub date: Date,
    pub time: Time,
}

/// Metadata for a directory entry.
#[derive(Default, Debug, Clone)]
pub struct Metadata {
    // FIXME: Fill me in.
    pub created: Timestamp,
    pub accessed: Timestamp,
    pub modified: Timestamp,
    pub attr: Attributes,
}

// FIXME: Implement `traits::Timestamp` for `Timestamp`.
impl traits::Timestamp for Timestamp {
    fn year(&self) -> usize {
        return (((self.date.0 >> 9) & 0b1111111) + 1980) as usize;
    }

    fn month(&self) -> u8 {
        return ((self.date.0 >> 5) & 0b1111) as u8;
    }

    fn day(&self) -> u8 {
        return (self.date.0 & 0b11111) as u8;
    }

    fn hour(&self) -> u8 {
        return ((self.time.0 >> 11) & 0b11111) as u8;
    }

    fn minute(&self) -> u8 {
        return ((self.time.0 >> 5) & 0b111111) as u8;
    }

    fn second(&self) -> u8 {
        return (self.time.0 & 0b11111) as u8 * 2;
    }

}

// FIXME: Implement `traits::Metadata` for `Metadata`.
impl traits::Metadata for Metadata {
    type Timestamp = Timestamp;

    fn read_only(&self) -> bool {
        return (self.attr.0 & 1) > 0;
    }

    fn hidden(&self) -> bool {
        return (self.attr.0 & 2) > 0;
    }

    fn created(&self) -> Self::Timestamp {
        return self.created;
    }

    fn accessed(&self) -> Self::Timestamp {
        return self.accessed;
    }

    fn modified(&self) -> Self::Timestamp {
        return self.modified;
    }
}

// FIXME: Implement `fmt::Display` (to your liking) for `Metadata`.

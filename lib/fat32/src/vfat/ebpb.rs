use core::fmt;
use shim::const_assert_size;

use crate::traits::BlockDevice;
use crate::vfat::Error;

#[repr(C, packed)]
pub struct BiosParameterBlock {
    // FIXME: Fill me in.
    first_three: [u8; 3],
    oem_identifier: [u8; 8],
    pub bytes_per_sector: u16,
    pub sectors_per_cluster: u8,
    pub reserved_sectors: u16,
    pub fats: u8,
    max_dir_entries: u16,
    pub logical_sectors: u16,
    fat_id: u8,
    sectors_per_fat: u16,
    sectors_per_track: u16,
    heads: u16,
    hidden_sectors: u32,
    pub tot_logical_sectors: u32,
    pub e_sectors_per_fat: u32,
    flags: u16,
    fat_ver_num: u16,
    pub root_cluster: u32,
    fsinfo_sector: u16,
    backup_sector: u16,
    reserved: [u8; 12],
    drive_num: u8,
    nt_flags: u8,
    signature: u8,
    vol_serial: u32,
    vol_label_str: [u8; 11],
    sys_ident_str: [u8; 8],
    boot_code: [u8; 420],
    boot_sig: [u8; 2],
}

const_assert_size!(BiosParameterBlock, 512);

impl BiosParameterBlock {
    /// Reads the FAT32 extended BIOS parameter block from sector `sector` of
    /// device `device`.
    ///
    /// # Errors
    ///
    /// If the EBPB signature is invalid, returns an error of `BadSignature`.
    pub fn from<T: BlockDevice>(mut device: T, sector: u64) -> Result<BiosParameterBlock, Error> {
        let mut buf = [0u8; 512];
        device.read_sector(sector, &mut buf);
        /*
        let p = (&buf[..]).as_ptr() as *const BiosParameterBlock;
        let b_ref: &BiosParameterBlock = unsafe { &*p };
        let bpb = unsafe { core::mem::transmute_copy::<BiosParameterBlock, BiosParameterBlock>(b_ref) };
        */
        let bpb = unsafe { core::mem::transmute::<[u8; 512], BiosParameterBlock>(buf) };
        if bpb.boot_sig[0] != 0x55 || bpb.boot_sig[1] != 0xAA {
            return Err(Error::BadSignature);
        }
        return Ok(bpb);
    }
}

impl fmt::Debug for BiosParameterBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("BiosParameterBlock")
            .field("boot_sig", &self.boot_sig)
            .finish()
    }
}

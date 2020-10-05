#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Copy, Clone, Hash)]
pub struct Cluster(pub u32);

impl From<u32> for Cluster {
    fn from(raw_num: u32) -> Cluster {
        Cluster(raw_num & !(0xF << 28))
    }
}

// TODO: Implement any useful helper methods on `Cluster`.
impl Cluster {
    pub fn to_fatentry_sector(&self, fat_start_sector: u64, bytes_per_sector: u64) -> u64 {
        return fat_start_sector + self.0 as u64 * 4 / bytes_per_sector;
    }

    pub fn to_sector(&self, data_start_sector: u64, sectors_per_cluster: u64) -> u64 {
        return data_start_sector + (self.0 - 2) as u64 * sectors_per_cluster;
    }

    pub fn index_in_sector(&self, bytes_per_sector: u64) -> usize {
        return (self.0 as usize * 4) % bytes_per_sector as usize;
    }
}

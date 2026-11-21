pub mod backend {
    #[cfg(feature = "fast")]
    pub fn checksum(data: &[u8]) -> u32 {
        data.len() as u32
    }

    #[cfg(not(feature = "fast"))]
    pub use crate::portable::checksum;
}

pub mod portable {
    pub fn checksum(data: &[u8]) -> u32 {
        data.iter().map(|byte| u32::from(*byte)).sum()
    }
}

use crate::backend::checksum;

pub fn entry_point(data: &[u8]) -> u32 {
    checksum(data)
}

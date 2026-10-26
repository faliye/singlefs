use crate::read_u64;

pub fn check_pool_image(root: &[u8]) -> bool {
    root.len() >= 8 && read_u64(root, 0) != 0
}

fn scanned_units(headers: &[Vec<u8>]) -> Vec<u64> {
    headers.iter().map(|header| read_u64(header, 0)).collect()
}

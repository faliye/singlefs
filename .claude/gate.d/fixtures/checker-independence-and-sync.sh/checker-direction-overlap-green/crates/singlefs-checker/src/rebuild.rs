use crate::read_u64;

pub(crate) fn rebuild_index(scanned: &[Vec<u8>]) -> Vec<u64> {
    scanned.iter().map(|unit| read_u64(unit, 0)).collect()
}

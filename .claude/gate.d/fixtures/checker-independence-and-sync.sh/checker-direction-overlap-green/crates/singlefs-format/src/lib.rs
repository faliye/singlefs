pub const SLOT_BYTES: u64 = 4096;

pub fn slot_offset(slot: u64) -> u64 {
    slot * SLOT_BYTES
}

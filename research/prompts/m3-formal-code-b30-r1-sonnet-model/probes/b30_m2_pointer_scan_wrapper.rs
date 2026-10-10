
/// b30 正推腿 M2 探针要的公开口子（追加到 common_pointer_head_corruption/mod.rs 末尾）。
pub fn all_units_in_the_unit_area(image: &MemoryPool) -> Vec<(SlotNumber, usize)> {
    units_in_the_unit_area(image)
}
pub fn change_the_unit_at(image: &mut MemoryPool, slot: SlotNumber, length_in_bytes: usize, change: impl FnOnce(&mut Vec<u8>)) {
    let mut bytes = unit_bytes_at(image, slot, length_in_bytes);
    change(&mut bytes);
    reseal_unit(&mut bytes);
    replace_the_unit_and_propagate_its_checksum(image, slot, &bytes);
}

//! 叠加盘：底盘的扇区、叠在上面的写、按策略拒数据单元的写与读不了的范围的单测。

use super::*;

fn block_of(byte: u8, sectors: usize) -> Vec<u8> {
    vec![byte; sectors * SECTOR_BYTES]
}

fn read_back(disk: &Disk<'_>, offset: u64, sectors: usize) -> Vec<u8> {
    let mut buffer = vec![0u8; sectors * SECTOR_BYTES];
    disk.read_at(DeviceOffsetInBytes(offset), &mut buffer)
        .expect("读得出");
    buffer
}

#[test]
fn blank_disk_reads_zero_and_keeps_what_was_written() {
    let mut disk = Disk::blank(1 << 20);
    disk.write_at(
        DeviceOffsetInBytes(1024),
        &block_of(7, 2),
        WriteDurability::Plain,
    )
    .expect("写得进");
    assert_eq!(read_back(&disk, 0, 2), block_of(0, 2));
    assert_eq!(read_back(&disk, 1024, 2), block_of(7, 2));
    assert_eq!(disk.own.sectors.len(), 2, "只存写过的非零扇区");
}

#[test]
fn base_disk_does_not_store_all_zero_sectors() {
    let mut disk = Disk::blank(1 << 20);
    disk.write_at(
        DeviceOffsetInBytes(0),
        &block_of(0, 4),
        WriteDurability::Plain,
    )
    .expect("写得进");
    assert!(disk.own.sectors.is_empty());
}

#[test]
fn an_overlay_write_does_not_reach_the_base_and_reads_back_over_it() {
    let mut base = Disk::blank(1 << 20);
    base.write_at(
        DeviceOffsetInBytes(0),
        &block_of(1, 4),
        WriteDurability::Plain,
    )
    .expect("写得进");
    let mut overlay = base.overlay(DataWritePolicy::AllowDataWrites);
    overlay
        .write_at(
            DeviceOffsetInBytes(512),
            &block_of(9, 1),
            WriteDurability::Plain,
        )
        .expect("写得进");
    assert_eq!(
        read_back(&overlay, 0, 1),
        block_of(1, 1),
        "没写的扇区读到底下基底"
    );
    assert_eq!(
        read_back(&overlay, 512, 1),
        block_of(9, 1),
        "写过的扇区读到叠层"
    );
    assert_eq!(read_back(&base, 512, 1), block_of(1, 1), "基底一个字节没动");
}

#[test]
fn an_overlay_zero_fill_hides_the_base_bytes_without_touching_them() {
    let mut base = Disk::blank(1 << 20);
    base.write_at(
        DeviceOffsetInBytes(0),
        &block_of(5, 8),
        WriteDurability::Plain,
    )
    .expect("写得进");
    let mut overlay = base.overlay(DataWritePolicy::AllowDataWrites);
    overlay
        .write_zeroes_at(DeviceOffsetInBytes(1024), 1024)
        .expect("清零");
    assert_eq!(read_back(&overlay, 1024, 2), block_of(0, 2));
    assert_eq!(read_back(&overlay, 0, 2), block_of(5, 2));
    assert_eq!(read_back(&base, 1024, 2), block_of(5, 2));
}

#[test]
fn the_first_data_unit_write_is_refused_and_noted_but_other_writes_pass() {
    let base = Disk::blank(1 << 20);
    let mut overlay = base.overlay(DataWritePolicy::FailTheFirstDataWriteAndNoteIt);
    let mut data_unit = vec![0u8; usize::try_from(DATA_UNIT_BYTES).expect("32768")];
    data_unit[..4].copy_from_slice(&UNIT_MAGIC_BYTES);
    data_unit[UNIT_CLASS_BYTE_INDEX] = UNIT_CLASS_DATA_CODE;
    let mut index_node = data_unit.clone();
    index_node[UNIT_CLASS_BYTE_INDEX] = 2;
    overlay
        .write_at(DeviceOffsetInBytes(0), &index_node, WriteDurability::Plain)
        .expect("索引节点的写放行");
    assert!(!overlay.data_write_reached);
    assert!(overlay
        .write_at(
            DeviceOffsetInBytes(32768),
            &data_unit,
            WriteDurability::Plain
        )
        .is_err());
    assert!(overlay.data_write_reached);
}

#[test]
fn reads_inside_an_unreadable_range_fail_and_reads_outside_succeed() {
    let mut disk = Disk::blank(1 << 20);
    disk.unreadable_byte_ranges.push(1024..2048);
    let mut buffer = vec![0u8; SECTOR_BYTES];
    assert!(disk
        .read_at(DeviceOffsetInBytes(1536), &mut buffer)
        .is_err());
    assert!(disk.read_at(DeviceOffsetInBytes(2048), &mut buffer).is_ok());
}

#[test]
fn the_journal_hint_lists_record_starts_the_overlay_wrote_and_hides_zeroed_ones() {
    let mut base = Disk::blank(1 << 24);
    let ring_start = 4096u64;
    base.write_at(
        DeviceOffsetInBytes(ring_start),
        &block_of(1, 8),
        WriteDurability::Plain,
    )
    .expect("写得进");
    base.write_at(
        DeviceOffsetInBytes(ring_start + 4096),
        &block_of(1, 8),
        WriteDurability::Plain,
    )
    .expect("写得进");
    let mut overlay = base.overlay(DataWritePolicy::AllowDataWrites);
    overlay
        .write_at(
            DeviceOffsetInBytes(ring_start + 8192),
            &block_of(2, 8),
            WriteDurability::Plain,
        )
        .expect("写得进");
    overlay
        .write_zeroes_at(DeviceOffsetInBytes(ring_start), 4096)
        .expect("清零");
    let hint = overlay
        .journal_record_offsets_hint(DeviceOffsetInBytes(ring_start), 3 * 4096)
        .expect("内存盘答得出");
    assert_eq!(
        hint,
        vec![
            DeviceOffsetInBytes(ring_start + 4096),
            DeviceOffsetInBytes(ring_start + 8192)
        ]
    );
}

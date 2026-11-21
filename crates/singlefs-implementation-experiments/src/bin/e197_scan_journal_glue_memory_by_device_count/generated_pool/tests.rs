//! `generated_pool` 的单测。
use super::*;

#[test]
fn the_reader_answers_only_whole_record_slots_inside_the_ring() {
    let pool = GeneratedPool::new(
        2,
        TEST_IMAGE_DEFAULT_BYTES,
        16,
        production_slot_source(8, 2),
    );
    let ring_start = JOURNAL_RING_START_SLOT * SLOT_BYTES;
    let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
    let read = |device: u32, offset: u64, length: usize| {
        pool.read(DeviceIdentity(device), DeviceOffsetInBytes(offset), length)
    };
    assert!(read(0, ring_start - JOURNAL_RECORD_BYTES, record_bytes).is_none());
    assert!(read(0, ring_start + 16 * JOURNAL_RECORD_BYTES, record_bytes).is_none());
    assert!(read(0, ring_start + 1, record_bytes).is_none());
    assert!(read(0, ring_start, record_bytes - 1).is_none());
    assert!(read(2, ring_start, record_bytes).is_none());
    let written =
        read(1, ring_start + 7 * JOURNAL_RECORD_BYTES, record_bytes).expect("槽 7 有记录");
    let parsed = JournalRecord::parse(&written, unit_filesystem_identifier(&FILESYSTEM_IDENTIFIER))
        .expect("槽 7 的字节过得了 parse");
    assert_eq!((parsed.counter, parsed.named.len()), (8, 2));
    let never_written =
        read(1, ring_start + 8 * JOURNAL_RECORD_BYTES, record_bytes).expect("槽 8 读得出");
    assert_eq!(never_written, vec![0u8; record_bytes]);
    assert_eq!(pool.reads.get(), 7);
    assert!(pool
        .journal_record_offsets_hint(
            DeviceIdentity(0),
            DeviceOffsetInBytes(ring_start),
            16 * JOURNAL_RECORD_BYTES
        )
        .is_none());
}

#[test]
fn the_production_slot_source_follows_the_fill_and_the_named_count() {
    let source = production_slot_source(100, 7);
    assert_eq!(source(0, 99).expect("槽 99 有记录").named.len(), 7);
    assert_eq!(source(3, 0).expect("每块盘同一槽字节相同").counter, 1);
    assert!(source(0, 100).is_none());
    let single = production_slot_source(100, 1);
    assert_eq!(single(0, 0).expect("槽 0 有记录").named.len(), 1);
    let full_named = production_slot_source(100, 64);
    assert_eq!(full_named(1, 5).expect("槽 5 有记录").named.len(), 64);
    assert_eq!(source(0, 5), source(1, 5));
    assert_eq!(Fill::Full.filled_slots_per_device(RING_SLOTS), 196_608);
    assert_eq!(Fill::Half.filled_slots_per_device(RING_SLOTS), 98_304);
}

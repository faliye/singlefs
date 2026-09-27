//! checker 档模块：无（自己逐个造崩溃状态，只用 harness 档的内存池与池级 checker）
//! 代码审阅第 14 条后一半：崩溃镜像交给恢复的 journal 提示（`CrashImage::journal_record_offsets_hint`）要与真盘上的全环扫描等价。
//!
//! 真盘不给提示，恢复（`scan_journal`）按 4096 对齐逐槽扫全环；层 0 与崩溃注入给提示只为了不在每个状态上读 768 MiB。
//! 提示与全环扫描不等价时，层 0 判的就不是真盘上会发生的事：一条写在不对齐偏移上的记录，真盘扫不到，按提示却读得到、施加它，层 0 绿；
//! 一次写罩两个记录槽，提示只给第一个，按提示读少一条。
//!
//! 判法：同一个镜像上跑两遍 `scan_journal`，一遍照常（给提示），一遍把提示换成 None（全环扫描），读出的记录逐条相同。
//! 镜像取两批：新池新建文件那条流在每个段边界上的状态与只落一份 journal 记录的状态（真录下来的写），
//! 再加两个手摆的（一条记录错开一个扇区写、两条记录一次写进相邻两槽）。

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;

use std::collections::BTreeSet;

use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::recovery::{choose_system_configuration, scan_journal, PoolReader};
use singlefs_format::{JOURNAL_RECORD_BYTES, JOURNAL_RING_START_SLOT, SLOT_BYTES};
use singlefs_harness::memory_pool::{
    writes_and_segments, CrashImage, MemoryPool, RetainedWrite, WrittenContents, SECTOR_BYTES,
};
use singlefs_harness::segments::StepKind;

/// 手摆的记录放在环里第 1000 个记录槽上：新池新建文件那条流只写了环头几个槽，离它们远。
const FAR_RECORD_SLOT_INDEX: u64 = 1000;

/// 同一个读口子，只是不给 journal 提示：恢复照真盘那样按 4096 对齐逐槽扫全环。
struct WithoutJournalHint<'reader, Reader: PoolReader>(&'reader Reader);

impl<Reader: PoolReader> PoolReader for WithoutJournalHint<'_, Reader> {
    fn device_identities(&self) -> Vec<DeviceIdentity> {
        self.0.device_identities()
    }
    fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> {
        self.0.device_size_in_bytes(device)
    }
    fn read(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        length: usize,
    ) -> Option<Vec<u8>> {
        self.0.read(device, offset, length)
    }
    fn journal_record_offsets_hint(
        &self,
        _device: DeviceIdentity,
        _ring_start: DeviceOffsetInBytes,
        _ring_bytes: u64,
    ) -> Option<Vec<DeviceOffsetInBytes>> {
        None
    }
}

/// 在一个崩溃镜像上比两种扫法：按提示扫与全环扫描读出的记录要逐条相同。交回全环扫描读出几条（调用方拿它反推：两边都空的比对不算数）。
fn records_found_by_the_hint_and_by_the_full_ring_scan_agree(
    image: &CrashImage<'_>,
    state: &str,
) -> usize {
    let system_configuration = choose_system_configuration(image)
        .expect("基镜像是 mkfs 之后的池，崩溃镜像上择得出系统配置");
    let by_the_hint = scan_journal(image, &system_configuration);
    let by_the_full_ring_scan = scan_journal(&WithoutJournalHint(image), &system_configuration);
    assert_eq!(
        by_the_hint, by_the_full_ring_scan,
        "{state}：按提示扫与全环扫描读出的记录要逐条相同"
    );
    by_the_full_ring_scan.len()
}

fn persisted_through_segments(
    write_count: usize,
    segments: &[Vec<usize>],
    persisted_segment_count: usize,
) -> Vec<bool> {
    let mut persisted = vec![false; write_count];
    for segment in &segments[..persisted_segment_count] {
        for write_index in segment {
            persisted[*write_index] = true;
        }
    }
    persisted
}

/// 新池新建文件那条流（取号、两次暖机、新池新建文件）的崩溃状态上逐个比：12 个段边界（前 k 段全持久；C577 之后暖机第二次的轮换自成一段，
/// 改之前 11 个），
/// 再加每段 journal 记录写里只落一份的那几个状态（两块盘各一份，只落一块盘上那份时提示只该给那块盘）。
#[test]
fn on_the_new_pool_file_creation_stream_the_hint_finds_what_the_full_ring_scan_finds_in_every_compared_state(
) {
    let pool = common::build_pool("journal-hint-first-stream");
    let base = pool.memory_pool_after_mkfs();
    let (writes, segments) = writes_and_segments(
        &pool.retained_operations()[pool.mkfs_operation_count..],
        &common::geometry(),
    );
    assert_eq!(
        segments.len(),
        11,
        "新池新建文件那条流 2+2+1+2+2+1+2+24+2+1+2（C577 之前轮换与单元写同段，2+2+1+2+2+1+26+2+1+2）"
    );

    let mut records_found_on_each_boundary = Vec::new();
    for persisted_segment_count in 0..=segments.len() {
        let image = CrashImage {
            base: &base,
            writes: &writes,
            persisted: persisted_through_segments(writes.len(), &segments, persisted_segment_count),
        };
        records_found_on_each_boundary.push(
            records_found_by_the_hint_and_by_the_full_ring_scan_agree(
                &image,
                &format!("前 {persisted_segment_count} 段全持久"),
            ),
        );
    }
    assert_eq!(
        records_found_on_each_boundary,
        vec![0, 0, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3],
        "三次发布（两次暖机、新池新建文件）的记录各在它的记录段落盘之后读得出：两种扫法都读到了记录，比对不是两边都空"
    );

    let mut single_record_states = 0usize;
    for (segment_index, segment) in segments.iter().enumerate() {
        for write_index in segment {
            if writes[*write_index].kind != StepKind::JournalRecord {
                continue;
            }
            let mut persisted = persisted_through_segments(writes.len(), &segments, segment_index);
            persisted[*write_index] = true;
            let image = CrashImage {
                base: &base,
                writes: &writes,
                persisted,
            };
            records_found_by_the_hint_and_by_the_full_ring_scan_agree(
                &image,
                &format!(
                    "前 {segment_index} 段全持久，第 {segment_index} 段只落盘 {} 上那份记录",
                    writes[*write_index].device.0
                ),
            );
            single_record_states += 1;
        }
    }
    assert_eq!(
        single_record_states, 6,
        "三次发布的记录写各两份（两块盘各一份）"
    );
}

/// 新池新建文件那条流里盘 0 上的前两条 journal 记录写的字节（两次暖机的记录：同一实例、计数器不同）。
fn two_records_written_on_device_zero(writes: &[RetainedWrite]) -> (Vec<u8>, Vec<u8>) {
    let records: Vec<&RetainedWrite> = writes
        .iter()
        .filter(|write| write.kind == StepKind::JournalRecord && write.device == DeviceIdentity(0))
        .collect();
    let bytes_of = |write: &RetainedWrite| {
        write
            .bytes()
            .expect("journal 记录写是普通写，带着字节")
            .to_vec()
    };
    (bytes_of(records[0]), bytes_of(records[1]))
}

/// 手摆的几次写全部持久、此外就是 mkfs 之后的池的崩溃镜像。
fn image_after_mkfs_with_these_writes_persisted<'image>(
    base: &'image MemoryPool,
    crafted_writes: &'image [RetainedWrite],
) -> CrashImage<'image> {
    CrashImage {
        base,
        writes: crafted_writes,
        persisted: vec![true; crafted_writes.len()],
    }
}

/// 手摆一次普通写：落在盘 0 的 journal 环里、离环起点 `offset_in_the_ring` 字节处。
fn journal_write_on_device_zero(offset_in_the_ring: u64, bytes: Vec<u8>) -> RetainedWrite {
    RetainedWrite {
        device: DeviceIdentity(0),
        kind: StepKind::JournalRecord,
        is_force_unit_access: false,
        offset: DeviceOffsetInBytes(JOURNAL_RING_START_SLOT * SLOT_BYTES + offset_in_the_ring),
        contents: WrittenContents::Bytes(bytes),
    }
}

/// 一条记录错开一个扇区写（不在 4096 的记录槽网格上）：全环扫描读不到它，按提示扫也不许读到；
/// 同一条记录写回对齐的槽上，两种扫法都读到它（正对照：这条记录本身读得出，读不到只因为错了位）。
#[test]
fn a_record_written_off_the_record_slot_grid_is_read_by_neither_scan() {
    let pool = common::build_pool("journal-hint-off-grid");
    let base = pool.memory_pool_after_mkfs();
    let (writes, _segments) = writes_and_segments(
        &pool.retained_operations()[pool.mkfs_operation_count..],
        &common::geometry(),
    );
    let (record, _second_record) = two_records_written_on_device_zero(&writes);

    let off_grid = [journal_write_on_device_zero(
        FAR_RECORD_SLOT_INDEX * JOURNAL_RECORD_BYTES + SECTOR_BYTES,
        record.clone(),
    )];
    assert_eq!(
        records_found_by_the_hint_and_by_the_full_ring_scan_agree(
            &image_after_mkfs_with_these_writes_persisted(&base, &off_grid),
            "一条记录错开一个扇区写"
        ),
        0,
        "全环扫描按 4096 对齐逐槽读，错开一个扇区的记录两个槽都读不成一条"
    );

    let on_grid = [journal_write_on_device_zero(
        FAR_RECORD_SLOT_INDEX * JOURNAL_RECORD_BYTES,
        record,
    )];
    assert_eq!(
        records_found_by_the_hint_and_by_the_full_ring_scan_agree(
            &image_after_mkfs_with_these_writes_persisted(&base, &on_grid),
            "同一条记录写在对齐的槽上"
        ),
        1
    );
}

/// 两条记录一次写进相邻两个记录槽（8192 字节一次写）：全环扫描两条都读到，按提示扫也得两条都读到（提示给两个槽，不是只给写的起点）。
#[test]
fn one_write_covering_two_record_slots_hands_both_slots_to_the_scan() {
    let pool = common::build_pool("journal-hint-two-slots");
    let base = pool.memory_pool_after_mkfs();
    let (writes, _segments) = writes_and_segments(
        &pool.retained_operations()[pool.mkfs_operation_count..],
        &common::geometry(),
    );
    let (first_record, second_record) = two_records_written_on_device_zero(&writes);
    let two_records_in_one_write = [journal_write_on_device_zero(
        FAR_RECORD_SLOT_INDEX * JOURNAL_RECORD_BYTES,
        [first_record, second_record].concat(),
    )];
    let image = image_after_mkfs_with_these_writes_persisted(&base, &two_records_in_one_write);
    assert_eq!(
        records_found_by_the_hint_and_by_the_full_ring_scan_agree(
            &image,
            "两条记录一次写进相邻两槽"
        ),
        2,
        "两次暖机的两条记录计数器不同，两条都读得出"
    );
    let ring_start = DeviceOffsetInBytes(JOURNAL_RING_START_SLOT * SLOT_BYTES);
    let system_configuration = choose_system_configuration(&image).expect("择得出系统配置");
    let hinted_offsets: BTreeSet<DeviceOffsetInBytes> = image
        .journal_record_offsets_hint(
            DeviceIdentity(0),
            ring_start,
            system_configuration.immutable.sizes.journal_ring_bytes,
        )
        .expect("崩溃镜像总给得出提示")
        .into_iter()
        .collect();
    assert_eq!(
        hinted_offsets,
        BTreeSet::from([
            DeviceOffsetInBytes(ring_start.0 + FAR_RECORD_SLOT_INDEX * JOURNAL_RECORD_BYTES),
            DeviceOffsetInBytes(ring_start.0 + (FAR_RECORD_SLOT_INDEX + 1) * JOURNAL_RECORD_BYTES),
        ]),
        "提示恰好是这次写罩住的两个记录槽（mkfs 之后环里没有别的记录）"
    );
}

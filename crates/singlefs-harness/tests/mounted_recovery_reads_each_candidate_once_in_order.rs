//! 只读挂载（`mount_read_only`）时取前缀，水位之上每个候选读几次盘、按什么次序读：反向链重读上一条记录
//! （D23（journal 的角色与格式） 已定项 7、I-8.6）与点名单元的两份（已定项 14 五条口径第四条）。
//!
//! 取前缀的核（`singlefs_replay_prefix::replay_core`）走到那一判才调两个读盘回调；Verus 的 spec 只说「据以判的那一次回调交回了什么」，
//! 不管调了几次、按什么次序调（代码轮第二轮判决 `research/prompts/m3-verus-land-code-r2-main-verification.md` 第二节 H2、H4 两行）。
//! 这里在块层逐次记下读，每条用例钉一种形状下扫完环之后的读：判候选的读的绝对次数与次序，之后环里再没有读。
//! 一条都没施加的那几形，判完候选之后、打开之前另有判退那一步读一遍见证（每块盘两个系统配置槽，D23（journal 的角色与格式） 已定项 14 F2 段；
//! 读法在 `reads_of_the_system_configuration_witness_before_judging_the_fall_back`），不在环里。
//! 钉住的只有这几种形状（代码轮第三轮判决 `research/prompts/m3-verus-land-code-r3-main-verification.md` J2、用户定案 U2）：
//! 锚点读得出、两次单条发布都施加；锚点那条记录撕了；锚点读不出、链首序号不是 1；一次发布两条记录；
//! 候选在末条的提交标记上被断；候选的反向链断了；到了在飞上限（3 槽的环，6-1 交回第 2 条的 H08、H09 两形）。
//! 在飞上限那一格的环过不了只读挂载择系统配置，照只读挂载同一串直接调 `choose_root`、`scan_journal`、`replay_journal`，只记重放那一截的读。
//! 可写挂载的读阶段走缓存（`ReadStageCache`）那一半在 `writable_mount_reads_each_candidate_in_order_through_the_read_stage_cache.rs`；
//! `recover` 不验点名单元那一臂读几次盘不在这里钉。
//!
//! 单条发布的历史造法同 `journal_replay_applies_no_more_records_than_the_in_flight_limit.rs` 的阳性对照：A（txg 3、jsn 3）的根落了，
//! B（txg 4、jsn 4）、C（txg 5、jsn 5）两次单条发布的记录与单元都在、根槽都坏了 ⇒ 所选根是 A 的。
//! 两条记录的历史：A 之后顺序写两个数据单元的 B（txg 4、jsn 4 与 5），根槽坏了 ⇒ 所选根是 A 的。

mod common;

use std::cell::RefCell;
use std::collections::BTreeMap;

use common::{
    block_read_rereading_the_record_before, block_reads_of_the_named_units_of, build_pool,
    image_with_the_records_relocated_onto_a_ring_of, is_inside_the_journal_ring,
    overwrite_in_process_as_instance_one_and_advance_the_pool, parameters,
    rewrite_journal_record_and_rechain_the_records_after_it,
    sequential_write_in_process_as_instance_one_and_advance_the_pool,
    tear_the_root_slot_written_for_txg, BlockLayerRead, CONTENT_BYTES_OF_PUBLISH_B,
    CONTENT_BYTES_OF_PUBLISH_C,
};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::journal::{
    record_offset, JournalRecord, JournalRecordOrdinalWithinPublish, JournalRecordPlaceInPublish,
};
use singlefs_core::mounted_read::{mount_read_only, open_pool_for_read, MountedReadOnly};
use singlefs_core::recovery::{
    choose_root, choose_system_configuration, replay_journal, scan_journal, NamedUnitVerification,
    PoolReader,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::system_configuration::journal_in_flight_record_limit;
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_format::{JOURNAL_RECORD_BYTES, SYSTEM_CONFIGURATION_SLOT_BYTES};
use singlefs_harness::memory_pool::MemoryPool;

/// B、C 两版文件的填充字节：两版长度（在 `common`）与内容各不相同。
const CONTENT_FILL_BYTE_OF_PUBLISH_B: u8 = 0xB0;
const CONTENT_FILL_BYTE_OF_PUBLISH_C: u8 = 0xC0;
/// 两条记录那段历史里 B 的填充字节。
const CONTENT_FILL_BYTE_OF_THE_TWO_RECORD_PUBLISH: u8 = 0xD0;
/// 一个候选的上一条记录在环里读得出时，反向链在第一块盘上重读它这么多次。
const REREADS_OF_THE_PREVIOUS_RECORD_PER_CANDIDATE: usize = 1;
/// 点名验证把每个点名单元的两份落点各读一次（`NamedUnit::locations` 是两项）。
const READS_PER_NAMED_UNIT: usize = 2;
/// 环按提示扫，两块盘各扫一遍。
const DEVICES_THE_JOURNAL_RING_IS_SCANNED_ON: usize = 2;
/// 两段历史都写到 jsn 5：环里写过的槽是 jsn 1–5。
const RING_SLOTS_WRITTEN_UP_TO_JSN_FIVE: usize = 5;
/// 一次单条覆盖写的记录点名的单元数。
const NAMED_UNITS_OF_A_SINGLE_RECORD_OVERWRITE: usize = 12;
/// 顺序写两个数据单元：前一条只点名自己那个数据单元，后一条点名这次发布其余的单元。
const NAMED_UNITS_OF_THE_FIRST_RECORD_OF_THE_TWO_RECORD_PUBLISH: usize = 1;
const NAMED_UNITS_OF_THE_LAST_RECORD_OF_THE_TWO_RECORD_PUBLISH: usize = 13;
/// 撕一条 journal 记录时翻的字节：在记录头之后、头部校验和罩着的整条 4096 之内（翻了自证就不过）。
const BYTE_INSIDE_A_JOURNAL_RECORD_AFTER_ITS_HEADER: u64 = 300;
/// 改坏一条记录的反向链时异或进去的位：链值变了、jsn 照样接得上（重封之后自证照过）。
const BIT_FLIPPED_IN_A_BACK_CHAIN: u32 = 1;
/// 3 个记录槽的环：在飞上限 = 3 ÷ F（3）= 1（同 `journal_replay_applies_no_more_records_than_the_in_flight_limit.rs`）。
const THREE_RECORD_RING_BYTES: u64 = 3 * JOURNAL_RECORD_BYTES;

/// 挂载那一路对读者做的事，按发生的次序。
#[derive(Clone, Debug, PartialEq, Eq)]
enum ReaderEvent {
    /// 扫环之前问一次「环里哪几个槽可能有字节」；交回的槽数就是这块盘上扫环要读几次。
    JournalRecordOffsetsHint {
        device: DeviceIdentity,
        offsets: Vec<DeviceOffsetInBytes>,
    },
    Read(BlockLayerRead),
}

/// 包在 [`MemoryPool`] 外面，把每次读与每次要环提示都原样记下来（读是 `&self`，记录走 [`RefCell`]）。
struct ReadLoggingPoolReader<'inner> {
    inner: &'inner MemoryPool,
    events: RefCell<Vec<ReaderEvent>>,
}

impl PoolReader for ReadLoggingPoolReader<'_> {
    fn device_identities(&self) -> Vec<DeviceIdentity> {
        self.inner.device_identities()
    }

    fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> {
        self.inner.device_size_in_bytes(device)
    }

    fn read(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        length: usize,
    ) -> Option<Vec<u8>> {
        self.events
            .borrow_mut()
            .push(ReaderEvent::Read(BlockLayerRead {
                device,
                offset,
                length_in_bytes: length,
            }));
        self.inner.read(device, offset, length)
    }

    fn journal_record_offsets_hint(
        &self,
        device: DeviceIdentity,
        ring_start: DeviceOffsetInBytes,
        ring_bytes: u64,
    ) -> Option<Vec<DeviceOffsetInBytes>> {
        let offsets = self
            .inner
            .journal_record_offsets_hint(device, ring_start, ring_bytes);
        self.events
            .borrow_mut()
            .push(ReaderEvent::JournalRecordOffsetsHint {
                device,
                offsets: offsets
                    .clone()
                    .expect("内存池交得出环提示：它知道哪些扇区写过"),
            });
        offsets
    }
}

/// 候选 `record` 的反向链那一判：在 0 号盘上重读它的上一条记录一次。
fn reread_of_the_record_before(record: &JournalRecord, ring_bytes: u64) -> ReaderEvent {
    ReaderEvent::Read(block_read_rereading_the_record_before(record, ring_bytes))
}

/// 候选 `record` 的点名验证：它点名的每个单元的两份，按点名次序。
fn named_unit_reads_of(record: &JournalRecord) -> Vec<ReaderEvent> {
    block_reads_of_the_named_units_of(record)
        .into_iter()
        .map(ReaderEvent::Read)
        .collect()
}

/// 候选 `record` 一判过到底该有的读：上一条记录的重读一次，再是它点名的每个单元的两份。
fn reads_judging_the_candidate(record: &JournalRecord, ring_bytes: u64) -> Vec<ReaderEvent> {
    let mut reads = vec![reread_of_the_record_before(record, ring_bytes)];
    reads.extend(named_unit_reads_of(record));
    reads
}

/// 一段事件里落在环里的读的次数；要环提示的每一次也算一次（扫完环之后不该再有）。
fn ring_reads_and_hints_among(events: &[ReaderEvent], ring_bytes: u64) -> usize {
    events
        .iter()
        .map(|event| match event {
            ReaderEvent::JournalRecordOffsetsHint { .. } => 1,
            ReaderEvent::Read(read) => {
                usize::from(is_inside_the_journal_ring(read.offset, ring_bytes))
            }
        })
        .sum()
}

/// 一次只读挂载的读，切在扫完环的那一刻。
struct ReadsOfOneMount {
    /// 扫环时落在环里的读的次数（要环提示不算）。
    scan_reads_inside_the_ring: usize,
    /// 扫完环之后的事件：判候选的读，之后是沿施加之后的根打开挂载态的读。
    events_after_the_scan: Vec<ReaderEvent>,
}

/// 在 `image` 上只读挂载一次，块层的每次读与每次要环提示按发生的次序记下来，切在最后一块盘的环扫完的那一刻。
fn mount_read_only_logging_every_read(
    image: &MemoryPool,
    ring_bytes: u64,
) -> (MountedReadOnly, ReadsOfOneMount) {
    let logging = ReadLoggingPoolReader {
        inner: image,
        events: RefCell::new(Vec::new()),
    };
    let mounted = mount_read_only(&logging).expect("只读挂载");
    let mut events = logging.events.into_inner();
    let last_hint_position = events
        .iter()
        .rposition(|event| matches!(event, ReaderEvent::JournalRecordOffsetsHint { .. }))
        .expect("扫环之前要过环提示");
    let ReaderEvent::JournalRecordOffsetsHint {
        offsets: offsets_scanned_on_the_last_device,
        ..
    } = &events[last_hint_position]
    else {
        unreachable!("rposition 找的就是环提示");
    };
    let scan_end = last_hint_position + 1 + offsets_scanned_on_the_last_device.len();
    let events_after_the_scan = events.split_off(scan_end);
    let scan_reads_inside_the_ring = events
        .iter()
        .map(|event| match event {
            ReaderEvent::JournalRecordOffsetsHint { .. } => 0,
            ReaderEvent::Read(read) => {
                usize::from(is_inside_the_journal_ring(read.offset, ring_bytes))
            }
        })
        .sum();
    (
        mounted,
        ReadsOfOneMount {
            scan_reads_inside_the_ring,
            events_after_the_scan,
        },
    )
}

/// 一条记录都没施加（施加之后那一版就是所选根）时，判完候选之后、打开挂载态之前判退那一步先读一遍见证：每块盘两个系统配置槽各读一次，
/// 按盘的次序（`recovery::fall_back_judged_on_the_three_readers` 先问的那一次 F2 判法）。这几次读是条款要的：三处判退看「三样读者的并集里有一样
/// 读不出，且系统配置没见证它」（D23（journal 的角色与格式） 已定项 14 F2 段，C755），只读挂载打得开所选那一版也要判退不退；两样里先判见证、
/// 见证过就不读另两样读者，是读得最少的那个次序。施加过记录的那一版不退（判法第一句），这一遍不读。
fn reads_of_the_system_configuration_witness_before_judging_the_fall_back(
    image: &MemoryPool,
) -> Vec<ReaderEvent> {
    let configuration = choose_system_configuration(image).expect("镜像上择得到系统配置");
    let slot_spacing_in_bytes =
        u64::from(configuration.immutable.sizes.fixed_structure_slot_spacing);
    let slot_bytes =
        usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("系统配置槽 4096 字节");
    image
        .device_identities()
        .into_iter()
        .flat_map(|device| {
            [0, slot_spacing_in_bytes].map(|offset| {
                ReaderEvent::Read(BlockLayerRead {
                    device,
                    offset: DeviceOffsetInBytes(offset),
                    length_in_bytes: slot_bytes,
                })
            })
        })
        .collect()
}

/// 单独沿 `root` 打开一次挂载态时块层的读：挂载在判完候选之后正是做这一步。
fn reads_of_opening_the_pool_at(image: &MemoryPool, root: &RootRecord) -> Vec<ReaderEvent> {
    let logging = ReadLoggingPoolReader {
        inner: image,
        events: RefCell::new(Vec::new()),
    };
    open_pool_for_read(&logging, root).expect("沿施加之后的根打得开");
    logging.events.into_inner()
}

/// 单条发布的历史：A 的根落了、B（jsn 4）与 C（jsn 5）只剩记录与单元。交回镜像与 A 那一版。
fn single_record_publishes_b_and_c_with_their_roots_torn(
    tag: &str,
) -> (MemoryPool, (InstanceGeneration, CheckpointTxg)) {
    let mut pool = build_pool(tag);
    let first_version = (pool.output.root.instance, pool.output.root.checkpoint_txg);
    let second = overwrite_in_process_as_instance_one_and_advance_the_pool(
        &mut pool,
        &vec![CONTENT_FILL_BYTE_OF_PUBLISH_B; CONTENT_BYTES_OF_PUBLISH_B],
    );
    let third = overwrite_in_process_as_instance_one_and_advance_the_pool(
        &mut pool,
        &vec![CONTENT_FILL_BYTE_OF_PUBLISH_C; CONTENT_BYTES_OF_PUBLISH_C],
    );
    assert_eq!(
        (
            first_version,
            (second.root.checkpoint_txg, second.record.counter),
            (third.root.checkpoint_txg, third.record.counter)
        ),
        (
            (InstanceGeneration(1), CheckpointTxg(3)),
            (CheckpointTxg(4), 4),
            (CheckpointTxg(5), 5)
        ),
        "A 的根在 txg 3、jsn 3；B、C 各一条记录"
    );
    let mut image = pool.memory_pool();
    for txg in [4u64, 5] {
        tear_the_root_slot_written_for_txg(&mut image, txg);
    }
    (image, first_version)
}

/// 两条记录的历史：A 的根落了、B 顺序写两个数据单元（jsn 4、5，txg 4）只剩记录与单元。交回镜像与 A 那一版。
fn two_record_publish_b_with_its_root_torn(
    tag: &str,
) -> (MemoryPool, (InstanceGeneration, CheckpointTxg)) {
    let mut pool = build_pool(tag);
    let first_version = (pool.output.root.instance, pool.output.root.checkpoint_txg);
    let payload_capacity = data_unit_payload_capacity();
    let second = sequential_write_in_process_as_instance_one_and_advance_the_pool(
        &mut pool,
        &vec![CONTENT_FILL_BYTE_OF_THE_TWO_RECORD_PUBLISH; payload_capacity + payload_capacity / 2],
    );
    let second_counters: Vec<u64> = second
        .earlier_records_of_this_publish
        .iter()
        .map(|written| written.record.counter)
        .chain([second.record.counter])
        .collect();
    assert_eq!(
        (first_version, second.root.checkpoint_txg, second_counters),
        (
            (InstanceGeneration(1), CheckpointTxg(3)),
            CheckpointTxg(4),
            vec![4, 5]
        ),
        "A 的根在 txg 3、jsn 3；B 两个数据单元切出两条记录"
    );
    let mut image = pool.memory_pool();
    tear_the_root_slot_written_for_txg(&mut image, 4);
    (image, first_version)
}

/// 镜像上择出的系统配置里的环长，与按它扫出来的记录。
fn journal_records_of(
    image: &MemoryPool,
) -> (u64, BTreeMap<(InstanceGeneration, u64), JournalRecord>) {
    let system_configuration = choose_system_configuration(image).expect("系统配置择得出");
    (
        system_configuration.immutable.sizes.journal_ring_bytes,
        scan_journal(image, &system_configuration),
    )
}

/// A 的根落了、B（jsn 4）与 C（jsn 5）只剩记录与单元：挂载时扫完环，jsn 4 先重读 jsn 3 一次、再读它点名的单元，
/// 然后 jsn 5 先重读 jsn 4 一次、再读它点名的单元；环里一共只多出这两次读。
#[test]
fn mounting_rereads_the_previous_record_once_then_each_named_unit_of_every_candidate_in_journal_order(
) {
    let (image, first_version) = single_record_publishes_b_and_c_with_their_roots_torn(
        "mounted-recovery-reads-each-candidate-once",
    );
    let (ring_bytes, records) = journal_records_of(&image);
    let candidate_b = records
        .get(&(InstanceGeneration(1), 4))
        .expect("jsn 4 在环里");
    let candidate_c = records
        .get(&(InstanceGeneration(1), 5))
        .expect("jsn 5 在环里");

    let (mounted, reads) = mount_read_only_logging_every_read(&image, ring_bytes);
    assert_eq!(
        (
            (
                mounted.chosen_root.instance,
                mounted.chosen_root.checkpoint_txg
            ),
            (
                mounted.effective_root.instance,
                mounted.effective_root.checkpoint_txg
            ),
            mounted.journal.above_water,
            mounted.journal.prefix_applied,
            mounted.journal.verification_passed
        ),
        (
            first_version,
            (InstanceGeneration(1), CheckpointTxg(5)),
            2,
            2,
            2
        ),
        "所选根是 A 的；水位之上 B、C 两条，都过了点名验证、都施加，挂载到 C"
    );

    let expected_reads_judging_the_candidates: Vec<ReaderEvent> = [candidate_b, candidate_c]
        .into_iter()
        .flat_map(|record| reads_judging_the_candidate(record, ring_bytes))
        .collect();
    let candidates_judged_through = 2;
    assert_eq!(
        (
            candidate_b.named.len(),
            candidate_c.named.len(),
            expected_reads_judging_the_candidates.len()
        ),
        (
            NAMED_UNITS_OF_A_SINGLE_RECORD_OVERWRITE,
            NAMED_UNITS_OF_A_SINGLE_RECORD_OVERWRITE,
            candidates_judged_through
                * (REREADS_OF_THE_PREVIOUS_RECORD_PER_CANDIDATE
                    + NAMED_UNITS_OF_A_SINGLE_RECORD_OVERWRITE * READS_PER_NAMED_UNIT)
        ),
        "B、C 各点名 12 个单元；判两个候选的读：各一次反向链重读，加各自点名单元的两份各一次"
    );
    assert_eq!(
        reads
            .events_after_the_scan
            .get(..expected_reads_judging_the_candidates.len()),
        Some(expected_reads_judging_the_candidates.as_slice()),
        "扫完环之后：jsn 4 重读 jsn 3 一次、读它点名的单元，再 jsn 5 重读 jsn 4 一次、读它点名的单元，不多读、不换次序"
    );
    assert_eq!(
        (
            reads.scan_reads_inside_the_ring,
            ring_reads_and_hints_among(
                &reads.events_after_the_scan[expected_reads_judging_the_candidates.len()..],
                ring_bytes
            )
        ),
        (
            RING_SLOTS_WRITTEN_UP_TO_JSN_FIVE * DEVICES_THE_JOURNAL_RING_IS_SCANNED_ON,
            0
        ),
        "扫环按提示读 jsn 1–5 五个槽、两块盘各一遍；判完两个候选之后环里不再读、也不再要提示"
    );
}

/// 锚点那条记录（所选根自己那条 jsn 3）两份都撕了：链首走「txg = 根 + 1、序号 1」那一支接上 jsn 4（D23 已定项 14 注 1）。
/// jsn 4 的上一条 jsn 3 不在扫出来的记录里，反向链不读盘就放过；它点名的单元照读。jsn 5 重读 jsn 4 一次、再读它点名的单元。
/// 扫完环之后判候选的读就这么多、这个次序，之后只剩沿 C 打开挂载态的读。
#[test]
fn with_the_anchor_record_torn_the_chain_head_rereads_nothing_and_the_next_candidate_rereads_the_previous_record_once(
) {
    let (mut image, first_version) =
        single_record_publishes_b_and_c_with_their_roots_torn("mounted-recovery-reads-anchor-torn");
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        image.flip_byte(
            device,
            record_offset(3, parameters().geometry.journal_ring_bytes),
            BYTE_INSIDE_A_JOURNAL_RECORD_AFTER_ITS_HEADER,
        );
    }
    let (ring_bytes, records) = journal_records_of(&image);
    assert!(
        !records.contains_key(&(InstanceGeneration(1), 3)),
        "jsn 3 两份都撕了，扫不出来"
    );
    let candidate_b = records
        .get(&(InstanceGeneration(1), 4))
        .expect("jsn 4 在环里");
    let candidate_c = records
        .get(&(InstanceGeneration(1), 5))
        .expect("jsn 5 在环里");

    let (mounted, reads) = mount_read_only_logging_every_read(&image, ring_bytes);
    assert_eq!(
        (
            (
                mounted.chosen_root.instance,
                mounted.chosen_root.checkpoint_txg
            ),
            (
                mounted.effective_root.instance,
                mounted.effective_root.checkpoint_txg
            ),
            mounted.journal.above_water,
            mounted.journal.prefix_applied,
            mounted.journal.verification_passed
        ),
        (
            first_version,
            (InstanceGeneration(1), CheckpointTxg(5)),
            2,
            2,
            2
        ),
        "锚点读不出：链首 jsn 4 的 txg 是 3 + 1、序号 1，接上；B、C 都过了点名验证、都施加，挂载到 C"
    );

    let mut expected_reads_judging_the_candidates = named_unit_reads_of(candidate_b);
    expected_reads_judging_the_candidates
        .extend(reads_judging_the_candidate(candidate_c, ring_bytes));
    assert_eq!(
        expected_reads_judging_the_candidates.len(),
        NAMED_UNITS_OF_A_SINGLE_RECORD_OVERWRITE * READS_PER_NAMED_UNIT
            + REREADS_OF_THE_PREVIOUS_RECORD_PER_CANDIDATE
            + NAMED_UNITS_OF_A_SINGLE_RECORD_OVERWRITE * READS_PER_NAMED_UNIT,
        "判两个候选的读：jsn 4 只读点名单元的两份，jsn 5 一次反向链重读加点名单元的两份"
    );
    let mut expected_events_after_the_scan = expected_reads_judging_the_candidates;
    expected_events_after_the_scan.extend(reads_of_opening_the_pool_at(
        &image,
        &mounted.effective_root,
    ));
    assert_eq!(
        reads.events_after_the_scan, expected_events_after_the_scan,
        "扫完环之后：jsn 4 不重读、读它点名的单元，jsn 5 重读 jsn 4 一次、读它点名的单元，然后只剩沿 C 打开挂载态的读"
    );
    assert_eq!(
        reads.scan_reads_inside_the_ring,
        RING_SLOTS_WRITTEN_UP_TO_JSN_FIVE * DEVICES_THE_JOURNAL_RING_IS_SCANNED_ON,
        "扫环按提示读 jsn 1–5 五个槽（撕了的 jsn 3 照读）、两块盘各一遍"
    );
}

/// 所选根那次发布读得出、却没有一条带末条标志（jsn 3 的标志改成「后面还有」、重封）⇒ 锚点读不出（D23 已定项 14 注 1，读法乙）；
/// 链首候选 jsn 4 的 txg 是 3 + 1，序号改成 2（重封，jsn 5 的反向链跟着重算）。链首序号不是 1 在反向链回调之前就断（次序同原文
/// fcc4d340）：一个候选都不施加，判候选的读一次都没有——jsn 4 的上一条 jsn 3 读得出，先调回调的读者会在 0 号盘上多重读它一次。
#[test]
fn without_a_last_record_flag_in_the_chosen_roots_publish_a_chain_head_whose_ordinal_is_not_one_is_cut_before_any_read(
) {
    let (mut image, first_version) = single_record_publishes_b_and_c_with_their_roots_torn(
        "mounted-recovery-reads-chain-head-ordinal-two",
    );
    rewrite_journal_record_and_rechain_the_records_after_it(&mut image, 3, |record| {
        record.place_in_publish = JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow;
    });
    rewrite_journal_record_and_rechain_the_records_after_it(&mut image, 4, |record| {
        record.ordinal_within_publish = JournalRecordOrdinalWithinPublish(2);
    });
    let (ring_bytes, records) = journal_records_of(&image);
    assert_eq!(
        (
            records
                .get(&(InstanceGeneration(1), 3))
                .map(|record| record.place_in_publish),
            records
                .get(&(InstanceGeneration(1), 4))
                .map(|record| record.ordinal_within_publish)
        ),
        (
            Some(JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow),
            Some(JournalRecordOrdinalWithinPublish(2))
        ),
        "jsn 3 读得出、不带末条标志；jsn 4 读得出、序号 2"
    );

    let (mounted, reads) = mount_read_only_logging_every_read(&image, ring_bytes);
    assert_eq!(
        (
            (
                mounted.chosen_root.instance,
                mounted.chosen_root.checkpoint_txg
            ),
            (
                mounted.effective_root.instance,
                mounted.effective_root.checkpoint_txg
            ),
            mounted.journal.above_water,
            mounted.journal.prefix_applied,
            mounted.journal.verification_passed
        ),
        (first_version, first_version, 2, 0, 0),
        "锚点读不出、链首序号不是 1：一条都不施加、一条都没验，挂载在 A"
    );
    let mut expected_events_after_the_scan =
        reads_of_the_system_configuration_witness_before_judging_the_fall_back(&image);
    expected_events_after_the_scan.extend(reads_of_opening_the_pool_at(
        &image,
        &mounted.effective_root,
    ));
    assert_eq!(
        reads.events_after_the_scan, expected_events_after_the_scan,
        "扫完环之后判候选的读一次都没有（不重读 jsn 3、不读点名单元）；一条都没施加，判退先读一遍见证，然后只剩沿 A 打开挂载态的读"
    );
    assert_eq!(
        reads.scan_reads_inside_the_ring,
        RING_SLOTS_WRITTEN_UP_TO_JSN_FIVE * DEVICES_THE_JOURNAL_RING_IS_SCANNED_ON,
        "扫环按提示读 jsn 1–5 五个槽、两块盘各一遍"
    );
}

/// 一次发布两条记录（B 顺序写两个数据单元：jsn 4 不带末条标志、只点名自己那个数据单元，jsn 5 带标志、点名其余的）：
/// jsn 4 重读 jsn 3 一次、读它点名的单元一遍，jsn 5 重读 jsn 4 一次、读它点名的单元一遍；不带末条标志的那条也只读一遍。
#[test]
fn a_two_record_publish_rereads_the_previous_record_once_and_reads_the_named_units_once_for_each_of_its_records(
) {
    let (image, first_version) =
        two_record_publish_b_with_its_root_torn("mounted-recovery-reads-two-record-publish");
    let (ring_bytes, records) = journal_records_of(&image);
    let first_record_of_b = records
        .get(&(InstanceGeneration(1), 4))
        .expect("jsn 4 在环里");
    let last_record_of_b = records
        .get(&(InstanceGeneration(1), 5))
        .expect("jsn 5 在环里");

    let (mounted, reads) = mount_read_only_logging_every_read(&image, ring_bytes);
    assert_eq!(
        (
            (
                mounted.chosen_root.instance,
                mounted.chosen_root.checkpoint_txg
            ),
            (
                mounted.effective_root.instance,
                mounted.effective_root.checkpoint_txg
            ),
            mounted.journal.above_water,
            mounted.journal.prefix_applied,
            mounted.journal.verification_passed
        ),
        (
            first_version,
            (InstanceGeneration(1), CheckpointTxg(4)),
            2,
            2,
            2
        ),
        "所选根是 A 的；水位之上 B 的两条，都过了点名验证，整次施加，挂载到 B"
    );

    let mut expected_reads_judging_the_candidates =
        reads_judging_the_candidate(first_record_of_b, ring_bytes);
    expected_reads_judging_the_candidates
        .extend(reads_judging_the_candidate(last_record_of_b, ring_bytes));
    assert_eq!(
        (
            first_record_of_b.named.len(),
            last_record_of_b.named.len(),
            expected_reads_judging_the_candidates.len()
        ),
        (
            NAMED_UNITS_OF_THE_FIRST_RECORD_OF_THE_TWO_RECORD_PUBLISH,
            NAMED_UNITS_OF_THE_LAST_RECORD_OF_THE_TWO_RECORD_PUBLISH,
            REREADS_OF_THE_PREVIOUS_RECORD_PER_CANDIDATE
                + NAMED_UNITS_OF_THE_FIRST_RECORD_OF_THE_TWO_RECORD_PUBLISH * READS_PER_NAMED_UNIT
                + REREADS_OF_THE_PREVIOUS_RECORD_PER_CANDIDATE
                + NAMED_UNITS_OF_THE_LAST_RECORD_OF_THE_TWO_RECORD_PUBLISH * READS_PER_NAMED_UNIT
        ),
        "jsn 4 点名 1 个单元、jsn 5 点名 13 个；判两条的读：各一次反向链重读，加各自点名单元的两份各一次"
    );
    let mut expected_events_after_the_scan = expected_reads_judging_the_candidates;
    expected_events_after_the_scan.extend(reads_of_opening_the_pool_at(
        &image,
        &mounted.effective_root,
    ));
    assert_eq!(
        reads.events_after_the_scan, expected_events_after_the_scan,
        "扫完环之后：jsn 4 重读 jsn 3 一次、读它点名的单元一遍，jsn 5 重读 jsn 4 一次、读它点名的单元一遍，然后只剩沿 B 打开挂载态的读"
    );
    assert_eq!(
        reads.scan_reads_inside_the_ring,
        RING_SLOTS_WRITTEN_UP_TO_JSN_FIVE * DEVICES_THE_JOURNAL_RING_IS_SCANNED_ON,
        "扫环按提示读 jsn 1–5 五个槽、两块盘各一遍"
    );
}

/// 两条记录的 B，末条 jsn 5 的提交标记改成 0（重封）：jsn 4 照常重读 jsn 3、读它点名的单元；jsn 5 先重读 jsn 4 一次，
/// 再在「末条不带提交标记」上断（D23 已定项 7、已定项 14 五条口径第三条；判的次序同原文 fcc4d340：反向链在提交标记之前），
/// 它点名的单元一个都不读。B 整次不施加，挂载在 A。
#[test]
fn a_last_record_without_its_commit_marker_is_cut_after_rereading_its_previous_record_and_before_reading_its_named_units(
) {
    let (mut image, first_version) = two_record_publish_b_with_its_root_torn(
        "mounted-recovery-reads-last-record-without-commit",
    );
    rewrite_journal_record_and_rechain_the_records_after_it(&mut image, 5, |record| {
        record.is_commit = false;
    });
    let (ring_bytes, records) = journal_records_of(&image);
    let first_record_of_b = records
        .get(&(InstanceGeneration(1), 4))
        .expect("jsn 4 在环里");
    let last_record_of_b = records
        .get(&(InstanceGeneration(1), 5))
        .expect("jsn 5 在环里");
    assert_eq!(
        (
            last_record_of_b.place_in_publish,
            last_record_of_b.is_commit
        ),
        (JournalRecordPlaceInPublish::LastRecordOfThePublish, false),
        "jsn 5 带末条标志、不带提交标记"
    );

    let (mounted, reads) = mount_read_only_logging_every_read(&image, ring_bytes);
    assert_eq!(
        (
            (
                mounted.chosen_root.instance,
                mounted.chosen_root.checkpoint_txg
            ),
            (
                mounted.effective_root.instance,
                mounted.effective_root.checkpoint_txg
            ),
            mounted.journal.above_water,
            mounted.journal.prefix_applied,
            mounted.journal.verification_passed
        ),
        (first_version, first_version, 2, 0, 1),
        "jsn 4 过了点名验证；jsn 5 断在提交标记上，B 整次不施加，挂载在 A"
    );

    let mut expected_reads_judging_the_candidates =
        reads_judging_the_candidate(first_record_of_b, ring_bytes);
    expected_reads_judging_the_candidates
        .push(reread_of_the_record_before(last_record_of_b, ring_bytes));
    assert_eq!(
        expected_reads_judging_the_candidates.len(),
        REREADS_OF_THE_PREVIOUS_RECORD_PER_CANDIDATE
            + NAMED_UNITS_OF_THE_FIRST_RECORD_OF_THE_TWO_RECORD_PUBLISH * READS_PER_NAMED_UNIT
            + REREADS_OF_THE_PREVIOUS_RECORD_PER_CANDIDATE,
        "判两条的读：jsn 4 一次反向链重读加它点名单元的两份，jsn 5 只有一次反向链重读"
    );
    let mut expected_events_after_the_scan = expected_reads_judging_the_candidates;
    expected_events_after_the_scan
        .extend(reads_of_the_system_configuration_witness_before_judging_the_fall_back(&image));
    expected_events_after_the_scan.extend(reads_of_opening_the_pool_at(
        &image,
        &mounted.effective_root,
    ));
    assert_eq!(
        reads.events_after_the_scan, expected_events_after_the_scan,
        "扫完环之后：jsn 4 重读 jsn 3、读它点名的单元，jsn 5 重读 jsn 4 一次就断、不读点名单元；一条都没施加，判退先读一遍见证，\
         然后只剩沿 A 打开挂载态的读"
    );
    assert_eq!(
        reads.scan_reads_inside_the_ring,
        RING_SLOTS_WRITTEN_UP_TO_JSN_FIVE * DEVICES_THE_JOURNAL_RING_IS_SCANNED_ON,
        "扫环按提示读 jsn 1–5 五个槽、两块盘各一遍"
    );
}

/// 单条发布的 B、C，C（jsn 5）的反向链改一位、重封（jsn 接得上、链值与 jsn 4 的头算出来的不等，D23 已定项 7、I-8.6）：
/// jsn 4 照常重读 jsn 3、读它点名的单元，施加；jsn 5 重读 jsn 4 一次就在反向链上断，它点名的单元一个都不读。挂载到 B。
#[test]
fn a_candidate_whose_back_chain_is_broken_is_cut_after_rereading_its_previous_record_and_before_reading_its_named_units(
) {
    let (mut image, first_version) = single_record_publishes_b_and_c_with_their_roots_torn(
        "mounted-recovery-reads-back-chain-broken",
    );
    rewrite_journal_record_and_rechain_the_records_after_it(&mut image, 5, |record| {
        record.back_chain ^= BIT_FLIPPED_IN_A_BACK_CHAIN;
    });
    let (ring_bytes, records) = journal_records_of(&image);
    let candidate_b = records
        .get(&(InstanceGeneration(1), 4))
        .expect("jsn 4 在环里");
    let candidate_c = records
        .get(&(InstanceGeneration(1), 5))
        .expect("jsn 5 改过反向链、重封，照样自证得过");

    let (mounted, reads) = mount_read_only_logging_every_read(&image, ring_bytes);
    assert_eq!(
        (
            (
                mounted.chosen_root.instance,
                mounted.chosen_root.checkpoint_txg
            ),
            (
                mounted.effective_root.instance,
                mounted.effective_root.checkpoint_txg
            ),
            mounted.journal.above_water,
            mounted.journal.prefix_applied,
            mounted.journal.verification_passed
        ),
        (
            first_version,
            (InstanceGeneration(1), CheckpointTxg(4)),
            2,
            1,
            1
        ),
        "jsn 4 过了点名验证、施加；jsn 5 断在反向链上，挂载到 B"
    );

    let mut expected_reads_judging_the_candidates =
        reads_judging_the_candidate(candidate_b, ring_bytes);
    expected_reads_judging_the_candidates
        .push(reread_of_the_record_before(candidate_c, ring_bytes));
    assert_eq!(
        expected_reads_judging_the_candidates.len(),
        REREADS_OF_THE_PREVIOUS_RECORD_PER_CANDIDATE
            + NAMED_UNITS_OF_A_SINGLE_RECORD_OVERWRITE * READS_PER_NAMED_UNIT
            + REREADS_OF_THE_PREVIOUS_RECORD_PER_CANDIDATE,
        "判两个候选的读：jsn 4 一次反向链重读加它点名单元的两份，jsn 5 只有一次反向链重读"
    );
    let mut expected_events_after_the_scan = expected_reads_judging_the_candidates;
    expected_events_after_the_scan.extend(reads_of_opening_the_pool_at(
        &image,
        &mounted.effective_root,
    ));
    assert_eq!(
        reads.events_after_the_scan, expected_events_after_the_scan,
        "扫完环之后：jsn 4 重读 jsn 3、读它点名的单元，jsn 5 重读 jsn 4 一次就断、不读点名单元，然后只剩沿 B 打开挂载态的读"
    );
    assert_eq!(
        reads.scan_reads_inside_the_ring,
        RING_SLOTS_WRITTEN_UP_TO_JSN_FIVE * DEVICES_THE_JOURNAL_RING_IS_SCANNED_ON,
        "扫环按提示读 jsn 1–5 五个槽、两块盘各一遍"
    );
}

/// 单条发布的 B、C 搬到 3 槽的环上（在飞上限 3 ÷ F = 1，D23 已定项 18）：jsn 4 重读 jsn 3、读它点名的单元，施加；
/// 取了 1 条就到上限，jsn 5 在任何读之前停下——不重读 jsn 4、不读它点名的单元。重放那一截块层的读就只有 jsn 4 的这几次。
#[test]
fn at_the_in_flight_limit_the_next_candidate_is_cut_before_rereading_its_previous_record() {
    let (image, first_version) = single_record_publishes_b_and_c_with_their_roots_torn(
        "mounted-recovery-reads-in-flight-limit",
    );
    assert_eq!(
        journal_in_flight_record_limit(THREE_RECORD_RING_BYTES),
        1,
        "3 槽 ÷ F（3）= 1"
    );
    let relocated = image_with_the_records_relocated_onto_a_ring_of(
        &image,
        &[3, 4, 5],
        THREE_RECORD_RING_BYTES,
    );
    let system_configuration =
        choose_system_configuration(&relocated).expect("系统配置择得出（它记着的是默认环）");
    let mut three_record_ring_configuration = system_configuration;
    three_record_ring_configuration
        .immutable
        .sizes
        .journal_ring_bytes = THREE_RECORD_RING_BYTES;
    let chosen_root = choose_root(&relocated, &system_configuration).expect("择得出根");
    let records = scan_journal(&relocated, &three_record_ring_configuration);
    assert_eq!(
        (
            (chosen_root.instance, chosen_root.checkpoint_txg),
            records
                .keys()
                .map(|(_, counter)| *counter)
                .collect::<Vec<_>>()
        ),
        (first_version, vec![3, 4, 5]),
        "所选根是 A 的；3 槽的环里扫到 jsn 3、4、5"
    );
    let candidate_b = records
        .get(&(InstanceGeneration(1), 4))
        .expect("jsn 4 在环里");

    let logging = ReadLoggingPoolReader {
        inner: &relocated,
        events: RefCell::new(Vec::new()),
    };
    let (report, rebuilt) = replay_journal(
        &logging,
        &chosen_root,
        THREE_RECORD_RING_BYTES,
        &records,
        NamedUnitVerification::Verify,
    )
    .expect("所选根那次发布只有 jsn 3 带末条标志：锚点认得出");
    assert_eq!(
        (
            report.above_water,
            report.prefix_applied,
            report.verification_passed,
            (rebuilt.instance, rebuilt.checkpoint_txg)
        ),
        (2, 1, 1, (InstanceGeneration(1), CheckpointTxg(4))),
        "水位之上两条，上限 1：只施加 jsn 4、验过它的点名单元，重建到 B"
    );
    let expected_reads_of_the_replay =
        reads_judging_the_candidate(candidate_b, THREE_RECORD_RING_BYTES);
    assert_eq!(
        expected_reads_of_the_replay.len(),
        REREADS_OF_THE_PREVIOUS_RECORD_PER_CANDIDATE
            + NAMED_UNITS_OF_A_SINGLE_RECORD_OVERWRITE * READS_PER_NAMED_UNIT,
        "重放那一截的读：jsn 4 一次反向链重读加它点名单元的两份，jsn 5 一次都没有"
    );
    assert_eq!(
        logging.events.into_inner(),
        expected_reads_of_the_replay,
        "重放那一截：jsn 4 在 3 槽环的落点上重读 jsn 3 一次、读它点名的单元，到了上限 jsn 5 不重读 jsn 4、不读点名单元"
    );
}

//! 可写挂载（`mount::mount_writable`）的读阶段取前缀时，水位之上每个候选在块层读几次盘、按什么次序读（C648）。
//! 只读挂载那一半在 `mounted_recovery_reads_each_candidate_once_in_order.rs`。
//!
//! 可写挂载的读阶段经读缓存（`mount.rs` 的 `ReadStageCache`）读：同一个 (盘, 偏移, 长度) 读出来不全是 0 的，第二次起不落到块层。
//! 扫环（块设备不交环提示，每块盘整环逐槽读）已经把每条记录读进缓存，反向链重读上一条记录因此一次都不落到块层；
//! 点名单元在扫环时没读过，每个点名单元的两份各落到块层一次，按候选次序、按点名次序。
//! 这里在块层逐次记读（写、写零、屏障也记一项，不比），钉住扫完环之后、判据 N-配置 直接读系统配置槽（每块盘两槽，不经缓存）之前的那一截。
//! 健康镜像上反向链回调多调一次（攻方 S03）、末条提交标记那一判挪到反向链回调之前（攻方 S07）都只多读或少读一次缓存里有的记录，块层看不出；
//! 缓存不收读回全零的落点，所以另有两条用例把上一条记录在 0 号盘（反向链重读先试的那块）上的一份写成全零：
//! 那一份每次反向链重读都落到块层一次，S03、S07 在块层看得出。让本该在点名单元之前断的候选去读点名单元，在健康镜像上就看得出。
//!
//! 造法：A（txg 3、jsn 3）的根落了；之后的发布只剩记录与单元——根槽改坏，系统配置槽写回 A 那次发布之后的字节
//! （D16（发布语义） 已定项 7：轮换在根槽 FUA 之后，根槽没落、轮换也没落），判据 N-配置 因此判假，读阶段只走一遍。

mod common;

use std::cell::RefCell;
use std::rc::Rc;

use common::{
    block_read_rereading_the_record_before, block_reads_of_the_named_units_of, build_pool,
    crash_state_devices, is_inside_the_journal_ring,
    overwrite_in_process_as_instance_one_and_advance_the_pool, parameters,
    rewrite_journal_record_and_rechain_the_records_after_it,
    sequential_write_in_process_as_instance_one_and_advance_the_pool,
    system_configuration_slots_of_every_device, tear_the_root_slot_written_for_txg,
    write_back_system_configuration_slots, BlockLayerRead, CONTENT_BYTES_OF_PUBLISH_B,
    CONTENT_BYTES_OF_PUBLISH_C, FIRST_DEVICE_TRIED_BY_THE_BACK_CHAIN_REREAD,
};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::journal::{record_offset, JournalRecord, JournalRecordPlaceInPublish};
use singlefs_core::mount::{mount_writable, Mounted};
use singlefs_core::recovery::{choose_system_configuration, scan_journal};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_format::{
    JOURNAL_RECORD_BYTES, SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE, SYSTEM_CONFIGURATION_SLOT_BYTES,
};
use singlefs_harness::memory_pool::MemoryPool;
use singlefs_harness::SharedStream;

/// B、C 两版文件的填充字节。
const CONTENT_FILL_BYTE_OF_PUBLISH_B: u8 = 0xB0;
const CONTENT_FILL_BYTE_OF_PUBLISH_C: u8 = 0xC0;
/// 两条记录那段历史里 B 的填充字节。
const CONTENT_FILL_BYTE_OF_THE_TWO_RECORD_PUBLISH: u8 = 0xD0;
/// 池里两块盘：扫环每块盘各扫一遍整环。
const DEVICES_THE_JOURNAL_RING_IS_SCANNED_ON: usize = 2;
/// 点名验证把每个点名单元的两份落点各读一次（`NamedUnit::locations` 是两项）。
const READS_PER_NAMED_UNIT: usize = 2;
/// 一次单条覆盖写的记录点名的单元数。
const NAMED_UNITS_OF_A_SINGLE_RECORD_OVERWRITE: usize = 12;
/// 顺序写两个数据单元：前一条只点名自己那个数据单元。
const NAMED_UNITS_OF_THE_FIRST_RECORD_OF_THE_TWO_RECORD_PUBLISH: usize = 1;
/// 上一条记录在 0 号盘上那一份读回全零时，一个候选的反向链那一判在块层重读它这么多次（缓存不收全零，1 号盘那一份从缓存交回）。
const BLOCK_LAYER_REREADS_OF_A_ZEROED_PREVIOUS_COPY_PER_CANDIDATE: usize = 1;
/// 单条发布的 B、C 两个候选。
const CANDIDATES_OF_THE_SINGLE_RECORD_PUBLISHES: usize = 2;

/// 块层的一件事：读原样记下；写、写零、屏障只记发生过（这几条用例不比它们）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BlockLayerEvent {
    Read(BlockLayerRead),
    WriteOrWriteZeroesOrBarrier { device: DeviceIdentity },
}

/// 包在一块盘外面，把每次读按发生的次序记进几块盘共用的一份记录里。
struct ReadLoggingBlockDevice<Inner: BlockDevice> {
    identity: DeviceIdentity,
    inner: Inner,
    events: Rc<RefCell<Vec<BlockLayerEvent>>>,
}

impl<Inner: BlockDevice> BlockDevice for ReadLoggingBlockDevice<Inner> {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        self.events
            .borrow_mut()
            .push(BlockLayerEvent::Read(BlockLayerRead {
                device: self.identity,
                offset,
                length_in_bytes: buffer.len(),
            }));
        self.inner.read_at(offset, buffer)
    }

    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        self.events
            .borrow_mut()
            .push(BlockLayerEvent::WriteOrWriteZeroesOrBarrier {
                device: self.identity,
            });
        self.inner.write_at(offset, bytes, durability)
    }

    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.events
            .borrow_mut()
            .push(BlockLayerEvent::WriteOrWriteZeroesOrBarrier {
                device: self.identity,
            });
        self.inner.write_zeroes_at(offset, length)
    }

    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        self.events
            .borrow_mut()
            .push(BlockLayerEvent::WriteOrWriteZeroesOrBarrier {
                device: self.identity,
            });
        self.inner.barrier()
    }

    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }

    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }
}

/// 一次可写挂载的块层事件，切在扫环最后一次读之后。
struct BlockLayerEventsOfOneWritableMount {
    /// 扫环那一段连着的环内读的次数。
    scan_reads_inside_the_ring: usize,
    /// 扫完环之后的事件。
    events_after_the_scan: Vec<BlockLayerEvent>,
}

/// 在 `image` 上可写挂载一次，块层每件事按发生的次序记下来；扫环是第一次环内读起、两块盘各整环的那一段。
fn mount_writable_logging_every_block_layer_event(
    image: &MemoryPool,
    ring_bytes: u64,
) -> (Mounted, BlockLayerEventsOfOneWritableMount) {
    let events = Rc::new(RefCell::new(Vec::new()));
    let mut devices: Vec<(DeviceIdentity, ReadLoggingBlockDevice<_>)> =
        crash_state_devices(image, &[], &[], &SharedStream::retaining_contents())
            .into_iter()
            .map(|(identity, inner)| {
                (
                    identity,
                    ReadLoggingBlockDevice {
                        identity,
                        inner,
                        events: Rc::clone(&events),
                    },
                )
            })
            .collect();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    drop(devices);
    let mut events = Rc::try_unwrap(events)
        .expect("盘都丢了，只剩这一份")
        .into_inner();
    let is_ring_read = |event: &BlockLayerEvent| match event {
        BlockLayerEvent::Read(read) => is_inside_the_journal_ring(read.offset, ring_bytes),
        BlockLayerEvent::WriteOrWriteZeroesOrBarrier { .. } => false,
    };
    let scan_start = events.iter().position(is_ring_read).expect("读阶段扫过环");
    let ring_slots_per_device = usize::try_from(ring_bytes / JOURNAL_RECORD_BYTES).expect("槽数");
    let scan_end = scan_start + ring_slots_per_device * DEVICES_THE_JOURNAL_RING_IS_SCANNED_ON;
    let scan_reads_inside_the_ring = events[scan_start..scan_end.min(events.len())]
        .iter()
        .filter(|event| is_ring_read(event))
        .count();
    let events_after_the_scan = events.split_off(scan_end.min(events.len()));
    (
        mounted,
        BlockLayerEventsOfOneWritableMount {
            scan_reads_inside_the_ring,
            events_after_the_scan,
        },
    )
}

/// 判据 N-配置 在取前缀之后直接读系统配置槽：按盘号次序，每块盘两槽各一次。
fn witness_reads_of_the_system_configuration_slots() -> Vec<BlockLayerEvent> {
    let slot_spacing_in_bytes = u64::from(parameters().geometry.fixed_structure_slot_spacing);
    [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .flat_map(|device| {
            (0..SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE).map(move |slot| {
                BlockLayerEvent::Read(BlockLayerRead {
                    device,
                    offset: DeviceOffsetInBytes(slot * slot_spacing_in_bytes),
                    length_in_bytes: usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES)
                        .expect("4096"),
                })
            })
        })
        .collect()
}

fn named_unit_events_of(record: &JournalRecord) -> Vec<BlockLayerEvent> {
    block_reads_of_the_named_units_of(record)
        .into_iter()
        .map(BlockLayerEvent::Read)
        .collect()
}

/// 候选 `record` 的反向链那一判在 0 号盘上重读它的上一条记录、落到块层的那一次。
fn back_chain_reread_event_of(record: &JournalRecord, ring_bytes: u64) -> BlockLayerEvent {
    BlockLayerEvent::Read(block_read_rereading_the_record_before(record, ring_bytes))
}

/// 把 jsn `counter` 那条记录在 0 号盘（反向链重读先试的那块）上的一份写成全零，1 号盘那一份不动：
/// 扫环从 1 号盘收下这条；反向链重读先读 0 号盘那一份，读回全零、读缓存不收，每次都落到块层，再从缓存里的 1 号盘那一份判链。
fn zero_the_copy_the_back_chain_reread_tries_first(
    image: &mut MemoryPool,
    counter: u64,
    ring_bytes: u64,
) {
    image
        .devices
        .get_mut(&FIRST_DEVICE_TRIED_BY_THE_BACK_CHAIN_REREAD)
        .expect("有这块盘")
        .write(
            record_offset(counter, ring_bytes),
            &vec![0; usize::try_from(JOURNAL_RECORD_BYTES).expect("4096")],
        );
}

/// 镜像上择出的系统配置里的环长，与 jsn `counters` 那几条扫出来的记录。
fn ring_bytes_and_records(image: &MemoryPool, counters: &[u64]) -> (u64, Vec<JournalRecord>) {
    let system_configuration = choose_system_configuration(image).expect("系统配置择得出");
    let records = scan_journal(image, &system_configuration);
    (
        system_configuration.immutable.sizes.journal_ring_bytes,
        counters
            .iter()
            .map(|counter| {
                records
                    .get(&(InstanceGeneration(1), *counter))
                    .expect("这条记录在环里")
                    .clone()
            })
            .collect(),
    )
}

/// A 的根落了，B（jsn 4）、C（jsn 5）两次单条发布只剩记录与单元：扫完环之后，jsn 4 的反向链重读 jsn 3、jsn 5 的重读 jsn 4
/// 都从读缓存交回、不落到块层；块层只有 jsn 4 点名的单元各两份、再 jsn 5 点名的单元各两份，之后是判据 N-配置 读系统配置槽。
#[test]
fn writable_mount_reads_each_named_unit_once_per_candidate_in_journal_order_and_the_back_chain_reread_stays_in_the_read_stage_cache(
) {
    let mut pool = build_pool("writable-mount-reads-single-record-publishes");
    let first_version = (pool.output.root.instance, pool.output.root.checkpoint_txg);
    let system_configuration_after_the_first_version =
        system_configuration_slots_of_every_device(&pool);
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
    write_back_system_configuration_slots(
        &mut image,
        &system_configuration_after_the_first_version,
    );
    let (ring_bytes, candidates) = ring_bytes_and_records(&image, &[4, 5]);

    let (mounted, events) = mount_writable_logging_every_block_layer_event(&image, ring_bytes);
    assert_eq!(
        (
            (
                mounted.output.chosen_root.instance,
                mounted.output.chosen_root.checkpoint_txg
            ),
            (
                mounted.output.effective_root.instance,
                mounted.output.effective_root.checkpoint_txg
            ),
            mounted.output.journal.above_water,
            mounted.output.journal.prefix_applied,
            mounted.output.journal.verification_passed
        ),
        (
            first_version,
            (InstanceGeneration(1), CheckpointTxg(5)),
            2,
            2,
            2
        ),
        "所选根是 A 的；水位之上 B、C 两条，都过了点名验证、都施加"
    );

    let mut expected_events_after_the_scan: Vec<BlockLayerEvent> =
        candidates.iter().flat_map(named_unit_events_of).collect();
    assert_eq!(
        expected_events_after_the_scan.len(),
        candidates.len() * NAMED_UNITS_OF_A_SINGLE_RECORD_OVERWRITE * READS_PER_NAMED_UNIT,
        "判两个候选在块层的读：各自点名的 12 个单元两份各一次，反向链重读一次都不落到块层"
    );
    expected_events_after_the_scan.extend(witness_reads_of_the_system_configuration_slots());
    assert_eq!(
        (
            events.scan_reads_inside_the_ring,
            events
                .events_after_the_scan
                .get(..expected_events_after_the_scan.len())
        ),
        (
            usize::try_from(ring_bytes / JOURNAL_RECORD_BYTES).expect("槽数")
                * DEVICES_THE_JOURNAL_RING_IS_SCANNED_ON,
            Some(expected_events_after_the_scan.as_slice())
        ),
        "扫环两块盘各整环逐槽读一遍；之后 jsn 4 点名的单元、jsn 5 点名的单元各读一次（不重读 jsn 3、jsn 4），再是系统配置槽"
    );
}

/// 两条记录的 B（jsn 4 不带末条标志、点名 1 个数据单元；jsn 5 带末条标志），jsn 5 的提交标记改成 0（重封）：
/// jsn 4 点名的单元两份各落到块层一次；jsn 5 的反向链重读从读缓存交回，随即断在「末条不带提交标记」上，它点名的单元一个都不读。
/// B 整次不施加，挂载在 A 上写行。
#[test]
fn writable_mount_cuts_a_last_record_without_its_commit_marker_before_reading_its_named_units() {
    let mut pool = build_pool("writable-mount-reads-last-record-without-commit");
    let first_version = (pool.output.root.instance, pool.output.root.checkpoint_txg);
    let system_configuration_after_the_first_version =
        system_configuration_slots_of_every_device(&pool);
    let payload_capacity = data_unit_payload_capacity();
    let second = sequential_write_in_process_as_instance_one_and_advance_the_pool(
        &mut pool,
        &vec![CONTENT_FILL_BYTE_OF_THE_TWO_RECORD_PUBLISH; payload_capacity + payload_capacity / 2],
    );
    assert_eq!(
        (
            first_version,
            second.root.checkpoint_txg,
            second.record.counter
        ),
        (
            (InstanceGeneration(1), CheckpointTxg(3)),
            CheckpointTxg(4),
            5
        ),
        "A 的根在 txg 3、jsn 3；B 两个数据单元切出 jsn 4、5"
    );
    let mut image = pool.memory_pool();
    tear_the_root_slot_written_for_txg(&mut image, 4);
    write_back_system_configuration_slots(
        &mut image,
        &system_configuration_after_the_first_version,
    );
    rewrite_journal_record_and_rechain_the_records_after_it(&mut image, 5, |record| {
        record.is_commit = false;
    });
    let (ring_bytes, candidates) = ring_bytes_and_records(&image, &[4, 5]);
    let [first_record_of_b, last_record_of_b] =
        <[_; 2]>::try_from(candidates).expect("jsn 4、5 两条");
    assert_eq!(
        (
            first_record_of_b.named.len(),
            last_record_of_b.place_in_publish,
            last_record_of_b.is_commit
        ),
        (
            NAMED_UNITS_OF_THE_FIRST_RECORD_OF_THE_TWO_RECORD_PUBLISH,
            JournalRecordPlaceInPublish::LastRecordOfThePublish,
            false
        ),
        "jsn 4 点名 1 个单元；jsn 5 带末条标志、不带提交标记"
    );

    let (mounted, events) = mount_writable_logging_every_block_layer_event(&image, ring_bytes);
    assert_eq!(
        (
            (
                mounted.output.chosen_root.instance,
                mounted.output.chosen_root.checkpoint_txg
            ),
            (
                mounted.output.effective_root.instance,
                mounted.output.effective_root.checkpoint_txg
            ),
            mounted.output.journal.above_water,
            mounted.output.journal.prefix_applied,
            mounted.output.journal.verification_passed
        ),
        (first_version, first_version, 2, 0, 1),
        "jsn 4 过了点名验证；jsn 5 断在提交标记上，B 整次不施加，挂载在 A"
    );

    let mut expected_events_after_the_scan = named_unit_events_of(&first_record_of_b);
    assert_eq!(
        expected_events_after_the_scan.len(),
        NAMED_UNITS_OF_THE_FIRST_RECORD_OF_THE_TWO_RECORD_PUBLISH * READS_PER_NAMED_UNIT,
        "判两条在块层的读：只有 jsn 4 点名单元的两份；两条的反向链重读都在读缓存里，jsn 5 不读点名单元"
    );
    expected_events_after_the_scan.extend(witness_reads_of_the_system_configuration_slots());
    assert_eq!(
        events
            .events_after_the_scan
            .get(..expected_events_after_the_scan.len()),
        Some(expected_events_after_the_scan.as_slice()),
        "扫完环之后：jsn 4 点名的单元读一次，jsn 5 一个都不读，再是系统配置槽"
    );
}

/// 单条发布的 B（jsn 4）、C（jsn 5），jsn 3、jsn 4 在 0 号盘上那一份写成全零：jsn 4 的反向链重读 jsn 3、jsn 5 的重读 jsn 4，
/// 各自在 0 号盘上落到块层恰好一次（读回全零、缓存不收），再从缓存里 1 号盘那一份判链、链对得上。
/// 块层：jsn 3 的零份一次、jsn 4 点名的单元各两份、jsn 4 的零份一次、jsn 5 点名的单元各两份，之后是判据 N-配置 读系统配置槽。
#[test]
fn writable_mount_rereads_a_previous_record_whose_first_copy_reads_back_as_zeros_from_the_block_layer_once_per_candidate(
) {
    let mut pool = build_pool("writable-mount-reads-zeroed-first-copies");
    let first_version = (pool.output.root.instance, pool.output.root.checkpoint_txg);
    let system_configuration_after_the_first_version =
        system_configuration_slots_of_every_device(&pool);
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
    write_back_system_configuration_slots(
        &mut image,
        &system_configuration_after_the_first_version,
    );
    let ring_bytes = choose_system_configuration(&image)
        .expect("系统配置择得出")
        .immutable
        .sizes
        .journal_ring_bytes;
    for counter in [3u64, 4] {
        zero_the_copy_the_back_chain_reread_tries_first(&mut image, counter, ring_bytes);
    }
    let (_, candidates) = ring_bytes_and_records(&image, &[4, 5]);
    let [candidate_b, candidate_c] = <[_; 2]>::try_from(candidates).expect("jsn 4、5 两条");

    let (mounted, events) = mount_writable_logging_every_block_layer_event(&image, ring_bytes);
    assert_eq!(
        (
            (
                mounted.output.chosen_root.instance,
                mounted.output.chosen_root.checkpoint_txg
            ),
            (
                mounted.output.effective_root.instance,
                mounted.output.effective_root.checkpoint_txg
            ),
            mounted.output.journal.above_water,
            mounted.output.journal.prefix_applied,
            mounted.output.journal.verification_passed
        ),
        (
            first_version,
            (InstanceGeneration(1), CheckpointTxg(5)),
            2,
            2,
            2
        ),
        "所选根是 A 的；jsn 3、4 的 1 号盘那一份在，反向链对得上，B、C 都过了点名验证、都施加"
    );

    let mut expected_events_after_the_scan =
        vec![back_chain_reread_event_of(&candidate_b, ring_bytes)];
    expected_events_after_the_scan.extend(named_unit_events_of(&candidate_b));
    expected_events_after_the_scan.push(back_chain_reread_event_of(&candidate_c, ring_bytes));
    expected_events_after_the_scan.extend(named_unit_events_of(&candidate_c));
    assert_eq!(
        expected_events_after_the_scan.len(),
        CANDIDATES_OF_THE_SINGLE_RECORD_PUBLISHES
            * (BLOCK_LAYER_REREADS_OF_A_ZEROED_PREVIOUS_COPY_PER_CANDIDATE
                + NAMED_UNITS_OF_A_SINGLE_RECORD_OVERWRITE * READS_PER_NAMED_UNIT),
        "判两个候选在块层的读：各自一次 0 号盘零份的反向链重读，加点名的 12 个单元两份各一次"
    );
    expected_events_after_the_scan.extend(witness_reads_of_the_system_configuration_slots());
    assert_eq!(
        (
            events.scan_reads_inside_the_ring,
            events
                .events_after_the_scan
                .get(..expected_events_after_the_scan.len())
        ),
        (
            usize::try_from(ring_bytes / JOURNAL_RECORD_BYTES).expect("槽数")
                * DEVICES_THE_JOURNAL_RING_IS_SCANNED_ON,
            Some(expected_events_after_the_scan.as_slice())
        ),
        "扫完环之后：jsn 4 在 0 号盘重读 jsn 3 一次、读它点名的单元，jsn 5 在 0 号盘重读 jsn 4 一次、读它点名的单元，再是系统配置槽"
    );
}

/// 两条记录的 B（jsn 4、5），jsn 5 的提交标记改成 0（重封），jsn 4 在 0 号盘上那一份写成全零：jsn 4 的反向链重读 jsn 3 从缓存交回，
/// 读它点名的单元；jsn 5 先在 0 号盘上重读 jsn 4、落到块层一次，链对得上，随即断在「末条不带提交标记」上，它点名的单元一个都不读。
#[test]
fn writable_mount_rereads_the_zeroed_first_copy_of_the_previous_record_once_before_cutting_a_last_record_without_its_commit_marker(
) {
    let mut pool = build_pool("writable-mount-reads-zeroed-copy-before-the-commit-cut");
    let first_version = (pool.output.root.instance, pool.output.root.checkpoint_txg);
    let system_configuration_after_the_first_version =
        system_configuration_slots_of_every_device(&pool);
    let payload_capacity = data_unit_payload_capacity();
    let second = sequential_write_in_process_as_instance_one_and_advance_the_pool(
        &mut pool,
        &vec![CONTENT_FILL_BYTE_OF_THE_TWO_RECORD_PUBLISH; payload_capacity + payload_capacity / 2],
    );
    assert_eq!(
        (
            first_version,
            second.root.checkpoint_txg,
            second.record.counter
        ),
        (
            (InstanceGeneration(1), CheckpointTxg(3)),
            CheckpointTxg(4),
            5
        ),
        "A 的根在 txg 3、jsn 3；B 两个数据单元切出 jsn 4、5"
    );
    let mut image = pool.memory_pool();
    tear_the_root_slot_written_for_txg(&mut image, 4);
    write_back_system_configuration_slots(
        &mut image,
        &system_configuration_after_the_first_version,
    );
    rewrite_journal_record_and_rechain_the_records_after_it(&mut image, 5, |record| {
        record.is_commit = false;
    });
    let ring_bytes = choose_system_configuration(&image)
        .expect("系统配置择得出")
        .immutable
        .sizes
        .journal_ring_bytes;
    zero_the_copy_the_back_chain_reread_tries_first(&mut image, 4, ring_bytes);
    let (_, candidates) = ring_bytes_and_records(&image, &[4, 5]);
    let [first_record_of_b, last_record_of_b] =
        <[_; 2]>::try_from(candidates).expect("jsn 4、5 两条");
    assert_eq!(
        (
            first_record_of_b.named.len(),
            last_record_of_b.place_in_publish,
            last_record_of_b.is_commit
        ),
        (
            NAMED_UNITS_OF_THE_FIRST_RECORD_OF_THE_TWO_RECORD_PUBLISH,
            JournalRecordPlaceInPublish::LastRecordOfThePublish,
            false
        ),
        "jsn 4 点名 1 个单元；jsn 5 带末条标志、不带提交标记"
    );

    let (mounted, events) = mount_writable_logging_every_block_layer_event(&image, ring_bytes);
    assert_eq!(
        (
            (
                mounted.output.chosen_root.instance,
                mounted.output.chosen_root.checkpoint_txg
            ),
            (
                mounted.output.effective_root.instance,
                mounted.output.effective_root.checkpoint_txg
            ),
            mounted.output.journal.above_water,
            mounted.output.journal.prefix_applied,
            mounted.output.journal.verification_passed
        ),
        (first_version, first_version, 2, 0, 1),
        "jsn 4 过了点名验证；jsn 5 断在提交标记上，B 整次不施加，挂载在 A"
    );

    let mut expected_events_after_the_scan = named_unit_events_of(&first_record_of_b);
    expected_events_after_the_scan.push(back_chain_reread_event_of(&last_record_of_b, ring_bytes));
    assert_eq!(
        expected_events_after_the_scan.len(),
        NAMED_UNITS_OF_THE_FIRST_RECORD_OF_THE_TWO_RECORD_PUBLISH * READS_PER_NAMED_UNIT
            + BLOCK_LAYER_REREADS_OF_A_ZEROED_PREVIOUS_COPY_PER_CANDIDATE,
        "判两条在块层的读：jsn 4 点名单元的两份，jsn 5 在 0 号盘重读 jsn 4 零份一次，jsn 5 不读点名单元"
    );
    expected_events_after_the_scan.extend(witness_reads_of_the_system_configuration_slots());
    assert_eq!(
        events
            .events_after_the_scan
            .get(..expected_events_after_the_scan.len()),
        Some(expected_events_after_the_scan.as_slice()),
        "扫完环之后：jsn 4 点名的单元读一次，jsn 5 在提交标记那一判之前重读 jsn 4 的零份一次、一个点名单元都不读，再是系统配置槽"
    );
}

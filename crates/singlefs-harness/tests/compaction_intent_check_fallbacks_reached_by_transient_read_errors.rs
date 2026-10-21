//! 整理两条「删意图那一核不过就照抄意图」的退路，在产品路径上由瞬时读错走到（判决 `research/prompts/m4-segment-code-r2-main-verification.md`
//! 第三节 H2；攻方 `research/prompts/m4-segment-code-r2-opus-model/out/transient-completing.txt` 与 `out/transient-1055.txt`）：
//! 一份副本读与重读都读不出、下一次又读得出时，删意图那一遍算账在释放之前读盘核把这一份隔离、那条记录留在已分配，R 就不按记录全空；
//! 照抄意图再算的那一遍读得出，发出去的那一版 R 按记录全空、意图还在，下一次发布照「R 已按记录全空」只删意图。
//! 两格各强制一条：最后一批（计划罩住 R 里每一条记录）删意图那一核不过；照抄那一支核出腾空了 R、照同一批删意图那一核不过。
//! 都断言之后的整理照常收：这一趟照判据停、意图删了、分配器上没有活意图。注入只用故障层，开关全是产品值（R 排他开着、删意图那一核照记录判）。
//!
//! 读错落在哪一次读：先不注入跑一遍、记下每一次读的落点；被点名那一份（R 那块盘上的一个槽）的最后一次读就是删意图那一遍算账的读盘核
//! （之后 R 腾空、它不再被读），注入的那一遍让它与紧接着的重读一起读不出。两遍从同一张镜像开、走同一串读，到注入那一刻为止逐个相同。

use std::cell::RefCell;
use std::collections::BTreeSet;
use std::rc::Rc;

use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes, SlotNumber};
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, DiscardOutcome, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::compaction_plan::{
    region_is_empty_by_the_records, CompactionBatchSize, CompactionBatchUnits, CompactionSwitches,
};
use singlefs_core::logged_operations::{live_compaction_intent_of_the_entries, CompactionRegion};
use singlefs_core::mount::mount_writable;
use singlefs_core::mounted_session::{CompactionRun, CompactionRunOutcome, MountedSession};
use singlefs_core::transaction::TransactionOutput;
use singlefs_harness::history::{
    execute_history_with, CompactionChoice, CompactionRunEnding, ContentChoice, ContentLength,
    GeneratedHistory, HistoryDeviceWidth, HistoryEnding, HistoryExecution, HistoryOperation,
    HistorySeed, HistoryStartingPoint, SnapshotChoice,
};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
use singlefs_harness::SharedStream;

/// 两次顺序写之后装着用户数据的那一段（攻方 `research/prompts/m4-segment-code-r2-opus-model/src/attack_r2_transient_read.rs` 的前缀，`ATTACK_REGION_START`）。
const REGION_START_SLOT_AFTER_THE_TWO_SEQUENTIAL_WRITES: u64 = 50_496;
/// 两次顺序写那一格一批搬几个（同攻方 `ATTACK_BATCH_UNITS`）：第一批三个、最后一批两个罩住 R 里剩下的每一条记录。
const BATCH_UNITS_AFTER_THE_TWO_SEQUENTIAL_WRITES: usize = 3;
/// 第一轮攻方种子下标 1055 那段历史最后那一趟整理签发的 R（同 `compaction_batch_emptying_the_region_deletes_the_intent_in_the_same_publish.rs`）。
const REGION_START_SLOT_OF_THE_LAST_COMPACTION_OF_THE_ATTACK_SEED_1055: u64 = 50_432;
/// 那一趟一批搬几个：R 最后只剩这次发布本来就重写的分配记录树叶时，计划的那一个不另搬、发布之后 R 按记录全空。
const BATCH_UNITS_OF_THE_LAST_COMPACTION_OF_THE_ATTACK_SEED_1055: usize = 1;

/// 故障层记的与注入的：开跑整理那一刻起（`is_armed`）记每一次读的落点；点名的那一份（盘、偏移）第几次读（从 0 数）交 I/O 错。
#[derive(Default)]
struct TransientReadFaults {
    is_armed: bool,
    reads: Vec<(DeviceIdentity, DeviceOffsetInBytes)>,
    target: Option<(DeviceIdentity, DeviceOffsetInBytes)>,
    failing_reads_of_the_target: BTreeSet<usize>,
    reads_of_the_target: usize,
    failed_reads: usize,
}

/// 内存稀疏盘外面套一层，读按 [`TransientReadFaults`] 记与注入；写、屏障、discard 原样交给里面那一层。
struct TransientlyFailingDevice {
    inner: SparseBlockDevice,
    identity: DeviceIdentity,
    faults: Rc<RefCell<TransientReadFaults>>,
}

impl BlockDevice for TransientlyFailingDevice {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        {
            let mut faults = self.faults.borrow_mut();
            if faults.is_armed {
                faults.reads.push((self.identity, offset));
                if faults.target == Some((self.identity, offset)) {
                    let ordinal = faults.reads_of_the_target;
                    faults.reads_of_the_target += 1;
                    if faults.failing_reads_of_the_target.contains(&ordinal) {
                        faults.failed_reads += 1;
                        return Err(BlockDeviceError::InputOutput(std::io::Error::other(
                            "注入的瞬时读错",
                        )));
                    }
                }
            }
        }
        self.inner.read_at(offset, buffer)
    }

    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        self.inner.write_at(offset, bytes, durability)
    }

    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.inner.write_zeroes_at(offset, length)
    }

    fn discard_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<DiscardOutcome, BlockDeviceError> {
        self.inner.discard_at(offset, length)
    }

    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        self.inner.barrier()
    }

    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }

    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }

    fn journal_record_offsets_hint(
        &self,
        ring_start: DeviceOffsetInBytes,
        ring_bytes: u64,
    ) -> Option<Vec<DeviceOffsetInBytes>> {
        self.inner
            .journal_record_offsets_hint(ring_start, ring_bytes)
    }
}

/// 走完一段从第一个文件起的历史（每一版与模型对得上），交回最后那一步之后的整份镜像。
fn image_after(operations: Vec<HistoryOperation>) -> MemoryPool {
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations,
    };
    let mut image_after_the_last_step: Option<MemoryPool> = None;
    let run = execute_history_with(
        &history,
        HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
        &SharedStream::new(),
        &mut |observation| image_after_the_last_step = Some(observation.image.clone()),
    );
    assert_eq!(
        run.ending,
        HistoryEnding::Completed,
        "前缀每一版与模型对得上：{:?}",
        run.outcomes
    );
    image_after_the_last_step.expect("走完的历史每一步之后都交镜像")
}

/// 从第一个文件起两次顺序写（152 个、290 个数据单元）。
fn two_sequential_writes() -> Vec<HistoryOperation> {
    vec![
        HistoryOperation::PublishOverwrite(ContentChoice {
            length: ContentLength::SpanningDataUnitsThroughTheSequentialWrite {
                data_units: 152,
                selector: 9_852_620_462_860_606_767,
            },
            fill_seed: 2_832_512_656_265_544_978,
        }),
        HistoryOperation::PublishOverwrite(ContentChoice {
            length: ContentLength::SpanningDataUnitsThroughTheSequentialWrite {
                data_units: 290,
                selector: 15_334_540_613_569_234_429,
            },
            fill_seed: 10_383_977_899_441_184_023,
        }),
    ]
}

fn destroy_snapshot(index_among_the_snapshots: u64) -> HistoryOperation {
    HistoryOperation::DestroySnapshot(SnapshotChoice {
        index_among_the_snapshots,
    })
}

fn compaction(
    batch_units: u64,
    region_selector: u64,
    ending: CompactionRunEnding,
) -> HistoryOperation {
    HistoryOperation::CompactWhileMounted(CompactionChoice {
        batch_units,
        region_selector,
        ending,
    })
}

/// 第一轮攻方种子下标 1055 缩出的 13 步里最后那一趟整理之前的 12 步（逐个照 `compaction_batch_emptying_the_region_deletes_the_intent_in_the_same_publish.rs`）。
fn the_twelve_steps_before_the_last_compaction_of_the_attack_seed_1055() -> Vec<HistoryOperation> {
    vec![
        HistoryOperation::CloseAndMountWritable,
        HistoryOperation::CloseAndMountWritable,
        HistoryOperation::CreateSnapshot,
        HistoryOperation::CreateSnapshot,
        HistoryOperation::CreateSnapshot,
        HistoryOperation::CreateSnapshot,
        destroy_snapshot(16_576_431_429_050_751_641),
        destroy_snapshot(8_608_203_070_865_410_994),
        compaction(
            1,
            14_751_488_729_257_944_391,
            CompactionRunEnding::RunsUntilItStops,
        ),
        HistoryOperation::CreateSnapshot,
        destroy_snapshot(13_548_285_845_384_774_606),
        compaction(
            2,
            15_468_272_257_618_924_935,
            CompactionRunEnding::CrashesAfterBatchesThenMountsWritable { batches: 2 },
        ),
    ]
}

/// 一趟整理的结局：会话交回的这一趟，与这一趟停下时分配器上还有没有活意图。
struct CompactionOnTheImage {
    run: CompactionRun,
    is_an_intent_live_on_the_allocator: bool,
}

/// 从镜像开一次可写挂载（故障层不记挂载那几次读），在 `region` 上一批 `batch_units` 个整理一趟（开关除批大小与强制选段外全是产品值）。
fn compact_once(
    image: &MemoryPool,
    region: CompactionRegion,
    batch_units: usize,
    faults: &Rc<RefCell<TransientReadFaults>>,
) -> CompactionOnTheImage {
    let width = HistoryDeviceWidth::FourGibibytes;
    let mut devices: Vec<(DeviceIdentity, TransientlyFailingDevice)> = image
        .devices
        .iter()
        .map(|(identity, sparse)| {
            let mut inner =
                SparseBlockDevice::new(width.device_bytes(), PhysicalBlockSizeInBytes(512));
            inner.image = sparse.clone();
            (
                *identity,
                TransientlyFailingDevice {
                    inner,
                    identity: *identity,
                    faults: Rc::clone(faults),
                },
            )
        })
        .collect();
    let mounted = mount_writable(&width.parameters(), &mut devices).expect("镜像可写挂载");
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    let CompactionSwitches {
        batch_size: _,
        forced_region: _,
        run_forcing,
        region_exclusion,
        destination_policy,
        release_checksum_mismatch_after_issue,
        empty_cluster_segments_floor,
        relocation_publish_order,
    } = CompactionSwitches::PRODUCT;
    session
        .allocator
        .set_compaction_switches(CompactionSwitches {
            batch_size: CompactionBatchSize::ForcedForTests(
                CompactionBatchUnits::new(batch_units).expect("至少 1"),
            ),
            forced_region: Some(region),
            run_forcing,
            region_exclusion,
            destination_policy,
            release_checksum_mismatch_after_issue,
            empty_cluster_segments_floor,
            relocation_publish_order,
        });
    faults.borrow_mut().is_armed = true;
    let run = session.run_compaction(&mut devices).expect("整理一趟");
    CompactionOnTheImage {
        run,
        is_an_intent_live_on_the_allocator: session.allocator.compaction().live_intent().is_some(),
    }
}

/// 一版里还活着的整理意图是哪一次发布签的（签发 txg）；没有活意图是 `None`。
fn live_intent_issued_txg(version: &TransactionOutput) -> Option<u64> {
    live_compaction_intent_of_the_entries(&version.logged_operations)
        .map(|(issued_txg, _intent)| issued_txg.0)
}

/// 先不注入跑一遍，按 `faulted_copy_of_the_baseline`（从不注入那一趟的各版挑出被点名那一份）定落点，再从同一张镜像开一遍，
/// 让那一份的最后一次读（删意图那一遍算账的读盘核）与紧接着的重读一起读不出。交回注入的那一趟与注入的次数。
fn compact_with_the_last_read_of_one_copy_failing_with_its_reread(
    image: &MemoryPool,
    region: CompactionRegion,
    batch_units: usize,
    faulted_copy_of_the_baseline: impl Fn(&CompactionRun) -> (DeviceIdentity, SlotNumber),
) -> (CompactionOnTheImage, usize) {
    let baseline_faults = Rc::new(RefCell::new(TransientReadFaults::default()));
    let baseline = compact_once(image, region, batch_units, &baseline_faults);
    assert!(
        matches!(baseline.run.outcome, CompactionRunOutcome::Stopped(_))
            && !baseline.is_an_intent_live_on_the_allocator,
        "不注入的那一趟照常停、意图收掉：{:?}",
        baseline.run.outcome
    );
    let (device, slot) = faulted_copy_of_the_baseline(&baseline.run);
    let target = (device, slot.to_device_offset());
    let reads_of_the_target = baseline_faults
        .borrow()
        .reads
        .iter()
        .filter(|read| **read == target)
        .count();
    let last_read_of_the_target = reads_of_the_target
        .checked_sub(1)
        .expect("不注入的那一趟读过被点名的那一份");
    let faults = Rc::new(RefCell::new(TransientReadFaults {
        target: Some(target),
        failing_reads_of_the_target: [last_read_of_the_target, last_read_of_the_target + 1]
            .into_iter()
            .collect(),
        ..TransientReadFaults::default()
    }));
    let faulted = compact_once(image, region, batch_units, &faults);
    let failed_reads = faults.borrow().failed_reads;
    (faulted, failed_reads)
}

/// 注入的那一趟里那一次照抄意图的发布：它之前那一版 R 里还有未释放的记录、意图活着，它之后 R 按记录全空、意图照抄；
/// 下一次发布只删意图（不搬单元，删的那一刻 R 按记录全空）。交回照抄的那一次是这一趟第几次发布。
fn the_publish_carrying_the_intent_over_an_emptied_region_and_the_deletion_after_it(
    run: &CompactionRun,
    region: CompactionRegion,
) -> usize {
    let issued_txg = run
        .versions_published
        .first()
        .and_then(live_intent_issued_txg)
        .expect("这一趟头一次发布签发意图");
    let carrying = run
        .versions_published
        .iter()
        .position(|version| {
            region_is_empty_by_the_records(&version.allocation_records, region)
                && live_intent_issued_txg(version) == Some(issued_txg)
        })
        .expect("删意图那一核不过、照抄意图发出去的那一版：R 按记录全空而意图还在");
    let before_the_carrying = &run.versions_published[carrying
        .checked_sub(1)
        .expect("头一次发布是签发：签发不碰 R 里的记录，R 按记录全空的不是它")];
    assert!(
        !region_is_empty_by_the_records(&before_the_carrying.allocation_records, region)
            && live_intent_issued_txg(before_the_carrying) == Some(issued_txg),
        "腾空 R 的是照抄意图的那一次：它之前那一版 R 里还有未释放的记录、意图活着"
    );
    let deleting = run
        .versions_published
        .get(carrying + 1)
        .expect("照抄意图的那一次之后还有一次发布：只删意图");
    assert!(
        live_intent_issued_txg(deleting).is_none()
            && deleting.units_relocated_by_this_publish.is_empty()
            && region_is_empty_by_the_records(&deleting.allocation_records, region),
        "下一次发布照「R 已按记录全空」只删意图、不搬单元：{:?}",
        deleting.units_relocated_by_this_publish
    );
    carrying
}

/// 最后一批（计划罩住 R 里剩下的每一条记录、同一次发布删意图）删意图那一遍算账时，搬走的一个源落点在 0 号盘那一份读与重读都读不出：
/// 隔离、记录留在已分配，R 不按记录全空，删意图那一核不过；照这一批照抄意图再算的那一遍读得出，这一批照搬、发出去的那一版 R 按记录全空、
/// 意图还在，下一次发布只删意图，这一趟照常停。
#[test]
fn a_transient_read_error_while_the_last_batch_deletes_the_intent_publishes_the_batch_carrying_it_and_the_next_publish_deletes_it(
) {
    let image = image_after(two_sequential_writes());
    let region = CompactionRegion::of_one_cluster_segment(
        DeviceIdentity(0),
        SlotNumber(REGION_START_SLOT_AFTER_THE_TWO_SEQUENTIAL_WRITES),
    )
    .expect("64 的倍数");
    let (faulted, failed_reads) = compact_with_the_last_read_of_one_copy_failing_with_its_reread(
        &image,
        region,
        BATCH_UNITS_AFTER_THE_TWO_SEQUENTIAL_WRITES,
        |baseline| {
            // 不注入时最后一批同一次发布删意图：它搬走的头一个单元在 0 号盘上的源落点。
            let last_batch = baseline
                .versions_published
                .iter()
                .find(|version| {
                    live_intent_issued_txg(version).is_none()
                        && !version.units_relocated_by_this_publish.is_empty()
                })
                .expect("不注入时最后一批搬了单元、同一次发布删了意图");
            let source = last_batch.units_relocated_by_this_publish[0]
                .source
                .iter()
                .find(|location| location.device == DeviceIdentity(0))
                .expect("两盘同槽：0 号盘上有一份");
            (source.device, source.slot)
        },
    );
    assert_eq!(
        failed_reads, 2,
        "注入落在被点名那一份的读与它的重读上，两次都交了错"
    );
    assert!(
        matches!(faulted.run.outcome, CompactionRunOutcome::Stopped(_)),
        "之后的整理照常收：这一趟照判据停，不是一次发布没发成：{:?}",
        faulted.run.outcome
    );
    let carrying = the_publish_carrying_the_intent_over_an_emptied_region_and_the_deletion_after_it(
        &faulted.run,
        region,
    );
    assert!(
        !faulted.run.versions_published[carrying]
            .units_relocated_by_this_publish
            .is_empty(),
        "照抄意图发出去的就是那最后一批：它照搬"
    );
    assert!(
        !faulted.is_an_intent_live_on_the_allocator,
        "这一趟停下时分配器上没有活意图"
    );
}

/// 第一轮攻方种子下标 1055 那段历史最后那一趟整理：计划的分配记录树叶是这次发布本来就重写的节点，照抄那一支核出这一批腾空了 R，
/// 照同一批删意图再算的那一遍里 R 里一份在 0 号盘上读与重读都读不出：隔离、记录留在已分配，删意图那一核不过；照这一批照抄意图再算的那一遍
/// 读得出，发出去的那一版 R 按记录全空、意图还在，下一次发布只删意图，这一趟照常停。
#[test]
fn a_transient_read_error_while_deleting_the_intent_after_a_batch_emptying_the_region_publishes_the_batch_carrying_it(
) {
    let image = image_after(the_twelve_steps_before_the_last_compaction_of_the_attack_seed_1055());
    let region = CompactionRegion::of_one_cluster_segment(
        DeviceIdentity(0),
        SlotNumber(REGION_START_SLOT_OF_THE_LAST_COMPACTION_OF_THE_ATTACK_SEED_1055),
    )
    .expect("64 的倍数");
    let (faulted, failed_reads) = compact_with_the_last_read_of_one_copy_failing_with_its_reread(
        &image,
        region,
        BATCH_UNITS_OF_THE_LAST_COMPACTION_OF_THE_ATTACK_SEED_1055,
        |baseline| {
            // 不注入时腾空 R 的那一批不搬单元、同一次发布删意图：它之前那一版 R 里 0 号盘上的头一条未释放记录。
            let emptying = baseline
                .versions_published
                .iter()
                .position(|version| {
                    live_intent_issued_txg(version).is_none()
                        && version.units_relocated_by_this_publish.is_empty()
                        && region_is_empty_by_the_records(&version.allocation_records, region)
                })
                .expect("不注入时有一次不搬单元、删了意图的发布");
            let before_the_emptying =
                &baseline.versions_published[emptying.checked_sub(1).expect("头一次发布是签发")];
            let record = before_the_emptying
                .allocation_records
                .iter()
                .find(|record| {
                    record.device == DeviceIdentity(0)
                        && !record.is_released
                        && region.contains_slot(record.slot)
                })
                .expect("腾空 R 的那一批之前 R 里还有未释放的记录");
            (record.device, record.slot)
        },
    );
    assert_eq!(
        failed_reads, 2,
        "注入落在被点名那一份的读与它的重读上，两次都交了错"
    );
    assert!(
        matches!(faulted.run.outcome, CompactionRunOutcome::Stopped(_)),
        "之后的整理照常收：这一趟照判据停，不是一次发布没发成：{:?}",
        faulted.run.outcome
    );
    let carrying = the_publish_carrying_the_intent_over_an_emptied_region_and_the_deletion_after_it(
        &faulted.run,
        region,
    );
    assert!(
        faulted.run.versions_published[carrying]
            .units_relocated_by_this_publish
            .is_empty(),
        "照抄意图发出去的那一批计划的是这次发布本来就重写的节点，不另搬"
    );
    assert!(
        !faulted.is_an_intent_live_on_the_allocator,
        "这一趟停下时分配器上没有活意图"
    );
}

//! 攻方腿原型（m4-segment-code-r2，R3 / R2）：整理的一次发布在会话里先算一遍账、核不过换个说法再算一遍（F3 签发照记账换 `segs0`、
//! F2 照抄核出腾空了 R 换删意图）。两遍都按发布路径「释放之前读盘核」读一遍被换下的单元：一次瞬时的读错只落在后一遍上时，两遍算出的账不同。
//! 盘用内存稀疏盘外面套一层：挂载与前缀都不碰它，`ATTACK_FAIL_READS` 点名的那几次读（从整理开跑起数，从 0 起）交 I/O 错，别的照读。
//! 前缀：起点第一个文件 → 顺序写 152 → 顺序写 290（第一轮攻方种子下标 126 那两次）；之后在装着用户数据的那一段（强制 R）上整理一趟。
use std::cell::RefCell;
use std::collections::BTreeSet;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::rc::Rc;

use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes, SlotNumber};
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, DiscardOutcome, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::compaction_plan::{CompactionBatchSize, CompactionBatchUnits, CompactionSwitches};
use singlefs_core::logged_operations::CompactionRegion;
use singlefs_core::mount::mount_writable;
use singlefs_core::mounted_session::MountedSession;
use singlefs_harness::history::{
    execute_history_with, ContentChoice, ContentLength, GeneratedHistory, HistoryDeviceWidth,
    HistoryEnding, HistoryExecution, HistoryOperation, HistorySeed, HistoryStartingPoint,
};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
use singlefs_harness::SharedStream;

#[derive(Default)]
struct Shared {
    armed: bool,
    reads: u64,
    fail: BTreeSet<u64>,
    log: bool,
    failed: Vec<String>,
}

struct TransientlyFailingDevice {
    inner: SparseBlockDevice,
    identity: u64,
    shared: Rc<RefCell<Shared>>,
}

impl BlockDevice for TransientlyFailingDevice {
    fn read_at(&self, offset: DeviceOffsetInBytes, buffer: &mut [u8]) -> Result<(), BlockDeviceError> {
        {
            let mut shared = self.shared.borrow_mut();
            if shared.armed {
                let number = shared.reads;
                shared.reads += 1;
                let slot = offset.0 / 16384;
                if shared.log {
                    eprintln!("READ {number} dev={} slot={slot} len={}", self.identity, buffer.len());
                }
                if shared.fail.contains(&number) {
                    shared.failed.push(format!("{number}:dev{}:slot{slot}", self.identity));
                    return Err(BlockDeviceError::InputOutput(std::io::Error::other("注入的瞬时读错")));
                }
            }
        }
        self.inner.read_at(offset, buffer)
    }
    fn write_at(&mut self, offset: DeviceOffsetInBytes, bytes: &[u8], durability: WriteDurability) -> Result<(), BlockDeviceError> {
        self.inner.write_at(offset, bytes, durability)
    }
    fn write_zeroes_at(&mut self, offset: DeviceOffsetInBytes, length: u64) -> Result<(), BlockDeviceError> {
        self.inner.write_zeroes_at(offset, length)
    }
    fn discard_at(&mut self, offset: DeviceOffsetInBytes, length: u64) -> Result<DiscardOutcome, BlockDeviceError> {
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
    fn journal_record_offsets_hint(&self, ring_start: DeviceOffsetInBytes, ring_bytes: u64) -> Option<Vec<DeviceOffsetInBytes>> {
        self.inner.journal_record_offsets_hint(ring_start, ring_bytes)
    }
}

fn env_u64(name: &str, default: u64) -> u64 {
    std::env::var(name).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}

/// 第一轮攻方种子下标 1055 缩出的 13 步的前 12 步（修复批 F2 用例 `compaction_batch_emptying_the_region_deletes_the_intent_in_the_same_publish.rs`
/// 逐个照抄的那一段），之后最后那一趟整理在盘 0 槽 50432 那一段上一批一个推：第四次发布计划的是这次发布本来就重写的节点。
fn operations_before_the_last_compaction_of_the_seed_1055() -> Vec<HistoryOperation> {
    use singlefs_harness::history::{CompactionChoice, CompactionRunEnding, SnapshotChoice};
    let destroy = |index_among_the_snapshots: u64| HistoryOperation::DestroySnapshot(SnapshotChoice { index_among_the_snapshots });
    let compaction = |batch_units: u64, region_selector: u64, ending: CompactionRunEnding| {
        HistoryOperation::CompactWhileMounted(CompactionChoice { batch_units, region_selector, ending })
    };
    vec![
        HistoryOperation::CloseAndMountWritable,
        HistoryOperation::CloseAndMountWritable,
        HistoryOperation::CreateSnapshot,
        HistoryOperation::CreateSnapshot,
        HistoryOperation::CreateSnapshot,
        HistoryOperation::CreateSnapshot,
        destroy(16_576_431_429_050_751_641),
        destroy(8_608_203_070_865_410_994),
        compaction(1, 14_751_488_729_257_944_391, CompactionRunEnding::RunsUntilItStops),
        HistoryOperation::CreateSnapshot,
        destroy(13_548_285_845_384_774_606),
        compaction(2, 15_468_272_257_618_924_935, CompactionRunEnding::CrashesAfterBatchesThenMountsWritable { batches: 2 }),
    ]
}

fn image_after_the_two_sequential_writes() -> MemoryPool {
    let operations = if std::env::var("ATTACK_PREFIX").as_deref() == Ok("seed1055") {
        operations_before_the_last_compaction_of_the_seed_1055()
    } else {
        two_sequential_writes()
    };
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations,
    };
    let mut last: Option<MemoryPool> = None;
    let run = execute_history_with(
        &history,
        HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
        &SharedStream::new(),
        &mut |observation| last = Some(observation.image.clone()),
    );
    assert_eq!(run.ending, HistoryEnding::Completed, "前缀走完");
    last.expect("每一步都交镜像")
}

fn two_sequential_writes() -> Vec<HistoryOperation> {
        vec![
            HistoryOperation::PublishOverwrite(ContentChoice {
                length: ContentLength::SpanningDataUnitsThroughTheSequentialWrite { data_units: 152, selector: 9_852_620_462_860_606_767 },
                fill_seed: 2_832_512_656_265_544_978,
            }),
            HistoryOperation::PublishOverwrite(ContentChoice {
                length: ContentLength::SpanningDataUnitsThroughTheSequentialWrite { data_units: 290, selector: 15_334_540_613_569_234_429 },
                fill_seed: 10_383_977_899_441_184_023,
            }),
        ]
}

/// 从镜像开一次可写挂载、在强制的 R 上整理一趟；整理开跑那一刻起数读，`fail` 里的那几次交 I/O 错。交回一句结局（panic 也收下）。
fn compact_once(image: &MemoryPool, region_start: u64, batch_units: u64, fail: &BTreeSet<u64>, log: bool) -> String {
    let shared = Rc::new(RefCell::new(Shared { fail: fail.clone(), log, ..Shared::default() }));
    let width = HistoryDeviceWidth::FourGibibytes;
    let mut devices: Vec<(DeviceIdentity, TransientlyFailingDevice)> = image
        .devices
        .iter()
        .map(|(identity, sparse)| {
            let mut inner = SparseBlockDevice::new(width.device_bytes(), PhysicalBlockSizeInBytes(512));
            inner.image = sparse.clone();
            (*identity, TransientlyFailingDevice { inner, identity: u64::from(identity.0), shared: Rc::clone(&shared) })
        })
        .collect();
    let mounted = mount_writable(&width.parameters(), &mut devices).expect("可写挂载");
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    // ATTACK_FILL_THE_OPEN_SEGMENT=1：照 F3 用例（`compaction_intent_issue_writes_the_empty_segments_of_its_own_accounting.rs`）把开放段占到只剩两槽，
    // 签发那次发布自己开新段，`segs0` 对不上记账第 11 项，走「照记账换了再算一遍」那一支。
    if env_u64("ATTACK_FILL_THE_OPEN_SEGMENT", 0) == 1 {
        let open_segment = session.allocator.open_segment().expect("挂载之后有开放段");
        let generation = session.current.version_of_trees().expect("有树").root.checkpoint_txg;
        loop {
            let used: u64 = session
                .allocator
                .records()
                .iter()
                .filter(|record| record.device == DeviceIdentity(0) && (open_segment.0..open_segment.0 + 64).contains(&record.slot.0))
                .map(|record| u64::from(record.span_slots))
                .sum();
            if used + 2 >= 64 {
                break;
            }
            session
                .allocator
                .try_allocate_commit_generated(singlefs_core::allocator::UnitFootprint::OneSlot, generation)
                .expect("开放段里还有空槽");
        }
    }
    let region = CompactionRegion::of_one_cluster_segment(DeviceIdentity(0), SlotNumber(region_start)).expect("64 的倍数");
    let product = CompactionSwitches::PRODUCT;
    session.allocator.set_compaction_switches(CompactionSwitches {
        batch_size: CompactionBatchSize::ForcedForTests(CompactionBatchUnits::new(usize::try_from(batch_units).expect("小")).expect("至少 1")),
        forced_region: Some(region),
        ..product
    });
    shared.borrow_mut().armed = true;
    let outcome = catch_unwind(AssertUnwindSafe(|| session.run_compaction(&mut devices)));
    let shared = shared.borrow();
    match outcome {
        Ok(Ok(run)) => format!(
            "reads={} failed={:?} outcome={:?} publishes={} batches={}",
            shared.reads,
            shared.failed,
            run.outcome,
            run.versions_published.len(),
            run.batches_published
        ),
        Ok(Err(refused)) => format!("reads={} failed={:?} refused={refused:?}", shared.reads, shared.failed),
        Err(panic) => {
            let message = panic
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| panic.downcast_ref::<&str>().map(|text| (*text).to_string()))
                .unwrap_or_default();
            format!("reads={} failed={:?} PANIC {message}", shared.reads, shared.failed)
        }
    }
}

/// ATTACK_FAIL_FROM..ATTACK_FAIL_TO 里每一个读号单独注入一次（每次从同一张镜像重开），打出结局；ATTACK_LOG=1 时只跑不注入的那一次、逐个打读。
#[test]
fn a_transient_read_error_between_the_two_computations_of_one_compaction_publish() {
    let region_start = env_u64("ATTACK_REGION_START", 50496);
    let batch_units = env_u64("ATTACK_BATCH_UNITS", 3);
    let image = image_after_the_two_sequential_writes();
    if env_u64("ATTACK_LOG", 0) == 1 {
        eprintln!("BASELINE {}", compact_once(&image, region_start, batch_units, &BTreeSet::new(), true));
        return;
    }
    eprintln!("BASELINE {}", compact_once(&image, region_start, batch_units, &BTreeSet::new(), false));
    for number in env_u64("ATTACK_FAIL_FROM", 0)..env_u64("ATTACK_FAIL_TO", 0) {
        // ATTACK_FAIL_PAIRS=1：这一次读与紧接着的下一次一起交错（发布路径的读盘核对一份读不出先重读一次，单次瞬时坏读被重读吸收）。
        let fail: BTreeSet<u64> = if env_u64("ATTACK_FAIL_PAIRS", 0) == 1 {
            [number, number + 1].into_iter().collect()
        } else {
            [number].into_iter().collect()
        };
        eprintln!("INJECT {number} {}", compact_once(&image, region_start, batch_units, &fail, false));
    }
}

/// 固定点那一族（ckpt_cost 的 Σ 名单：分配记录树、中央映射树、记账树、树表，加谱系、退役两项）。
fn is_a_fixed_point_role(identity: singlefs_core::transaction::TransactionUnit) -> bool {
    use singlefs_core::transaction::TransactionUnit as U;
    matches!(
        identity,
        U::AllocationTreeNodeBelowTheRoot(_)
            | U::AllocationTree
            | U::MappingTreeNodeBelowTheRoot(_)
            | U::MappingTree
            | U::AccountingTreeNodeBelowTheRoot(_)
            | U::AccountingTree
            | U::TreeTable
            | U::LineageLeafContainer(_)
            | U::LineageRoot
            | U::LineageNodeBelowTheRoot(_)
            | U::RetirementLeafContainer(_)
            | U::RetirementRoot
            | U::RetirementNodeBelowTheRoot(_)
    )
}

/// 后一版里固定点那一族新写的单元占几槽（角色与槽都不在前一版里的）。
fn fixed_point_slots_written(
    previous: &singlefs_core::transaction::TransactionOutput,
    next: &singlefs_core::transaction::TransactionOutput,
) -> u64 {
    let carried: BTreeSet<(String, u64)> = previous.units.iter().map(|unit| (format!("{:?}", unit.identity), unit.slot.0)).collect();
    next.units
        .iter()
        .filter(|unit| is_a_fixed_point_role(unit.identity))
        .filter(|unit| !carried.contains(&(format!("{:?}", unit.identity), unit.slot.0)))
        .map(|unit| unit.identity.span_slots())
        .sum()
}

/// R7「多出来的重写越过 ckpt_cost」：整理每一次发布、之后回退到整理之前那一版那次发布，各自写出的固定点槽数与它接在后面那一版的 ckpt_cost 比。
/// 前缀同上（两次顺序写），强制 R、一批 `ATTACK_BATCH_UNITS` 个推到停，再回退到根环里第 `ATTACK_ROLLBACK_INDEX` 新的根。
#[test]
fn fixed_point_slots_of_the_compaction_publishes_and_the_rollback_against_the_checkpoint_cost() {
    use singlefs_core::admission::checkpoint_cost_of_the_version_to_build_on;
    use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
    use singlefs_core::mount::{roll_back_by_a_forward_publish, RollbackTarget};
    use singlefs_core::transaction::PoolVersion;
    let region_start = env_u64("ATTACK_REGION_START", 50496);
    let batch_units = env_u64("ATTACK_BATCH_UNITS", 3);
    let image = image_after_the_two_sequential_writes();
    let shared = Rc::new(RefCell::new(Shared::default()));
    let width = HistoryDeviceWidth::FourGibibytes;
    let mut devices: Vec<(DeviceIdentity, TransientlyFailingDevice)> = image
        .devices
        .iter()
        .map(|(identity, sparse)| {
            let mut inner = SparseBlockDevice::new(width.device_bytes(), PhysicalBlockSizeInBytes(512));
            inner.image = sparse.clone();
            (*identity, TransientlyFailingDevice { inner, identity: u64::from(identity.0), shared: Rc::clone(&shared) })
        })
        .collect();
    let mounted = mount_writable(&width.parameters(), &mut devices).expect("可写挂载");
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    let region = CompactionRegion::of_one_cluster_segment(DeviceIdentity(0), SlotNumber(region_start)).expect("64 的倍数");
    session.allocator.set_compaction_switches(CompactionSwitches {
        batch_size: CompactionBatchSize::ForcedForTests(CompactionBatchUnits::new(usize::try_from(batch_units).expect("小")).expect("至少 1")),
        forced_region: Some(region),
        ..CompactionSwitches::PRODUCT
    });
    let before = session.current.version_of_trees().expect("有树").clone();
    let run = session.run_compaction(&mut devices).expect("整理一趟");
    let mut previous = before.clone();
    for version in &run.versions_published {
        let written = fixed_point_slots_written(&previous, version);
        let cost = checkpoint_cost_of_the_version_to_build_on(Some(&previous), &session.allocator).0;
        eprintln!("MARGIN compaction txg={} relocated={} fixed_point_slots_written={written} ckpt_cost={cost}", version.root.checkpoint_txg.0, version.units_relocated_by_this_publish.len());
        previous = version.clone();
    }
    let target_txg = env_u64("ATTACK_ROLLBACK_TXG", before.root.checkpoint_txg.0);
    let target = RollbackTarget { instance: InstanceGeneration(before.root.instance.0), checkpoint_txg: CheckpointTxg(target_txg) };
    let current_before_the_rollback = session.current.version_of_trees().expect("有树").clone();
    let PoolVersion::WithFile(current) = &mut session.current else { panic!("带文件") };
    let rolled = roll_back_by_a_forward_publish(&width.parameters(), &mut devices, &mut session.allocator, current, target);
    eprintln!("ROLLBACK {:?}", rolled.as_ref().map(|r| (r.released_user_visible_units.len(), r.resurrected_user_visible_copies.len())));
    let after = session.current.version_of_trees().expect("有树").clone();
    let written = fixed_point_slots_written(&current_before_the_rollback, &after);
    let cost = checkpoint_cost_of_the_version_to_build_on(Some(&current_before_the_rollback), &session.allocator).0;
    eprintln!("MARGIN rollback txg={} target_txg={target_txg} fixed_point_slots_written={written} ckpt_cost={cost}", after.root.checkpoint_txg.0);
}

/// R8（第一轮 T10 恢复读侧，换攻击面）：意图半路崩、可写挂载之后的镜像，再可写挂载一次；挂载那一刻起数读，`ATTACK_FAIL_FROM..TO` 里每一个读号单独
/// （`ATTACK_FAIL_PAIRS=1` 时连同下一次）交 I/O 错，打出挂载的结局：做成时打现行那一版的 txg、活意图、分配记录的指纹，与不注入那一格比。
#[test]
fn a_transient_read_error_during_the_writable_mount_of_a_mid_intent_image() {
    use singlefs_core::logged_operations::live_compaction_intent_of_the_entries;
    use singlefs_harness::history::{CompactionChoice, CompactionRunEnding};
    let mut operations = two_sequential_writes();
    operations.push(HistoryOperation::CompactWhileMounted(CompactionChoice {
        batch_units: 1,
        region_selector: 16_033_315_046_779_662_336,
        ending: CompactionRunEnding::CrashesAfterBatchesThenMountsWritable { batches: 2 },
    }));
    let history = GeneratedHistory { seed: HistorySeed(0), starting_point: HistoryStartingPoint::AfterFirstFile, operations };
    let mut last: Option<MemoryPool> = None;
    let run = execute_history_with(
        &history,
        HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
        &SharedStream::new(),
        &mut |observation| last = Some(observation.image.clone()),
    );
    assert_eq!(run.ending, HistoryEnding::Completed, "前缀走完");
    let image = last.expect("镜像");
    let mount_once = |fail: &BTreeSet<u64>| -> String {
        let shared = Rc::new(RefCell::new(Shared { fail: fail.clone(), armed: true, ..Shared::default() }));
        let width = HistoryDeviceWidth::FourGibibytes;
        let mut devices: Vec<(DeviceIdentity, TransientlyFailingDevice)> = image
            .devices
            .iter()
            .map(|(identity, sparse)| {
                let mut inner = SparseBlockDevice::new(width.device_bytes(), PhysicalBlockSizeInBytes(512));
                inner.image = sparse.clone();
                (*identity, TransientlyFailingDevice { inner, identity: u64::from(identity.0), shared: Rc::clone(&shared) })
            })
            .collect();
        let outcome = catch_unwind(AssertUnwindSafe(|| mount_writable(&width.parameters(), &mut devices)));
        let shared = shared.borrow();
        match outcome {
            Ok(Ok(mounted)) => {
                let (_, session) = MountedSession::of_the_mount(mounted);
                let version = session.current.version_of_trees().expect("有树");
                let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
                for byte in format!("{:?}", version.allocation_records).bytes() {
                    hash ^= u64::from(byte);
                    hash = hash.wrapping_mul(0x0100_0000_01b3);
                }
                format!(
                    "reads={} failed={:?} MOUNTED txg={} instance={} intent={:?} records_fingerprint={:04x} allocator_intent={:?}",
                    shared.reads,
                    shared.failed,
                    version.root.checkpoint_txg.0,
                    version.root.instance.0,
                    live_compaction_intent_of_the_entries(&version.logged_operations).map(|(txg, intent)| (txg.0, intent.region.start_slot().0)),
                    hash & 0xffff,
                    session.allocator.compaction().live_intent().map(|live| live.intent.region.start_slot().0)
                )
            }
            Ok(Err(refused)) => format!("reads={} failed={:?} REFUSED {}", shared.reads, shared.failed, format!("{refused:?}").chars().take(160).collect::<String>()),
            Err(panic) => {
                let message = panic.downcast_ref::<String>().cloned().or_else(|| panic.downcast_ref::<&str>().map(|text| (*text).to_string())).unwrap_or_default();
                format!("reads={} failed={:?} PANIC {message}", shared.reads, shared.failed)
            }
        }
    };
    let baseline = mount_once(&BTreeSet::new());
    eprintln!("BASELINE {baseline}");
    let pairs = env_u64("ATTACK_FAIL_PAIRS", 0) == 1;
    let baseline_tail = baseline.split("MOUNTED").nth(1).unwrap_or("").to_string();
    let (mut same, mut refused, mut different, mut panicked) = (0u64, 0u64, 0u64, 0u64);
    for number in env_u64("ATTACK_FAIL_FROM", 0)..env_u64("ATTACK_FAIL_TO", 0) {
        let fail: BTreeSet<u64> = if pairs { [number, number + 1].into_iter().collect() } else { [number].into_iter().collect() };
        let outcome = mount_once(&fail);
        let class = if outcome.contains("PANIC") {
            panicked += 1;
            "PANIC"
        } else if outcome.contains("REFUSED") {
            refused += 1;
            "REFUSED"
        } else if outcome.split("MOUNTED").nth(1).unwrap_or("").to_string() == baseline_tail {
            same += 1;
            "SAME"
        } else {
            different += 1;
            "DIFFERENT"
        };
        if class != "SAME" {
            eprintln!("INJECT {number} {class} {outcome}");
        }
    }
    eprintln!("MOUNT_SWEEP same={same} refused={refused} different={different} panicked={panicked}");
}

//! m2-final-code-r1 云端攻方腿（Opus）的用例，只在草稿副本上跑。Z1 / Z4 / Z6。

mod common;

use common::{memory_pool_of_sparse_devices, parameters, IMAGE_BYTES};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::DeviceIdentity;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::journal::JournalRecordPlaceInPublish;
use singlefs_core::mount::mount_writable;
use singlefs_core::transaction::{acquire_instance, PoolVersion, PoolWriter};
use singlefs_harness::crash::SparseBlockDevice;
use singlefs_harness::scenario::run_first_transaction;
use singlefs_harness::{RecordingBlockDevice, SharedStream};

const DISKS: [DeviceIdentity; 2] = [DeviceIdentity(0), DeviceIdentity(1)];
type Devices = Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)>;

fn pool_after_the_first_transaction(stream: &SharedStream) -> Devices {
    let mut devices: Devices = DISKS
        .iter()
        .map(|identity| {
            (
                *identity,
                RecordingBlockDevice::with_shared_stream(
                    *identity,
                    SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
                    stream.clone(),
                ),
            )
        })
        .collect();
    run_first_transaction(&parameters(), &mut devices, stream, |_, _| {}).expect("第一个事务");
    devices
}

fn crash_right_after_acquisition(devices: &mut Devices, count: u32) {
    let publish_parameters = parameters();
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    for _ in 0..count {
        acquire_instance(&mut writer).expect("取号");
    }
}

fn violated(devices: &Devices) -> Vec<(&'static str, String)> {
    check_pool_image(&memory_pool_of_sparse_devices(devices))
        .into_iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(detail) => Some((invariant, format!("{detail:?}"))),
            _ => None,
        })
        .collect()
}

/// Z1 × Z6：带文件的一版上写行，实例表链 64 片 ⇒ 点名 64 + 4 = 68 > 67，写行那次发布末条再跨记录（两条）。
/// 取号之前在拷贝上预演的落点与真发相同（establish_instance 那条断言不 panic）、两条记录的序号与标志、池级 checker。
#[test]
fn z6_a1_row_publish_with_sixty_four_instance_table_pages_spills_and_the_copy_matches() {
    let stream = SharedStream::new();
    let mut devices = pool_after_the_first_transaction(&stream);
    // 实例 1 的根；取号之后崩溃 23248 次 ⇒ 下一次挂载取 23250，写 [1, 23250) = 23249 行 ⇒ ⌈23249 / 369⌉ = 64 片。
    crash_right_after_acquisition(&mut devices, 23248);
    let mounted = mount_writable(&parameters(), &mut devices).expect("第一次挂载");
    let PoolVersion::WithFile(row) = &mounted.output.row_publish else {
        panic!("带文件的一版")
    };
    let records: Vec<_> = row
        .earlier_records_of_this_publish
        .iter()
        .map(|written| written.record.clone())
        .chain([row.record.clone()])
        .collect();
    println!(
        "OBS a1 instance={} rows={} records={} named={:?} ordinals={:?} last_flags={:?} commit={:?} transactions={:?} counters={:?}",
        mounted.output.instance.0,
        mounted.output.rows_written.len(),
        records.len(),
        records.iter().map(|record| record.named.len()).collect::<Vec<_>>(),
        records.iter().map(|record| record.ordinal_within_publish.0).collect::<Vec<_>>(),
        records
            .iter()
            .map(|record| record.place_in_publish == JournalRecordPlaceInPublish::LastRecordOfThePublish)
            .collect::<Vec<_>>(),
        records.iter().map(|record| record.is_commit).collect::<Vec<_>>(),
        records.iter().map(|record| record.transaction).collect::<Vec<_>>(),
        records.iter().map(|record| record.counter).collect::<Vec<_>>(),
    );
    let first_violations = violated(&devices);
    println!("OBS a1 checker after first mount: {first_violations:?}");
    let second = mount_writable(&parameters(), &mut devices).expect("第二次挂载");
    println!(
        "OBS a1 second mount instance={} rows_written={}",
        second.output.instance.0,
        second.output.rows_written.len()
    );
    let second_violations = violated(&devices);
    println!("OBS a1 checker after second mount: {second_violations:?}");
    assert_eq!(records.len(), 2, "68 个点名项切成两条");
    assert!(first_violations.is_empty() && second_violations.is_empty());
}

use common::crash_state_devices;
use singlefs_core::recovery::{recover, JournalPolicy};
use singlefs_format::{JOURNAL_RING_DEFAULT_BYTES, JOURNAL_RING_START_SLOT, SLOT_BYTES};
use singlefs_harness::crash::MemoryPool;
use singlefs_harness::{RecordedOperationKind, RetainedOperation};

/// 一个崩溃状态上：冷启动恢复一次、可写挂载一次（挂载成了再跑池级 checker）；panic 都接住。交回一行可比的结局。
fn judge_crash_state(image: &MemoryPool) -> String {
    let recovered = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let report = recover(image, JournalPolicy::Consult);
        format!(
            "recover root={:?} applied={}",
            report.effective_root, report.journal.prefix_applied
        )
    }))
    .unwrap_or_else(|_| "recover PANIC".to_string());
    let mounted = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut devices: Devices = crash_state_devices(image, &[], &[], &SharedStream::new());
        match mount_writable(&parameters(), &mut devices) {
            Ok(mounted) => format!(
                "mount ok instance={} rows={} violated={:?}",
                mounted.output.instance.0,
                mounted.output.rows_written.len(),
                violated(&devices)
                    .into_iter()
                    .map(|(invariant, _)| invariant)
                    .collect::<Vec<_>>()
            ),
            Err(error) => format!("mount err {error:?}").chars().take(160).collect(),
        }
    }))
    .unwrap_or_else(|_| "mount PANIC".to_string());
    format!("{recovered} | {mounted}")
}

fn is_journal_write(operation: &RetainedOperation) -> bool {
    let ring_start = JOURNAL_RING_START_SLOT * SLOT_BYTES;
    matches!(operation.operation.kind, RecordedOperationKind::Write)
        && operation.operation.offset.0 >= ring_start
        && operation.operation.offset.0 < ring_start + JOURNAL_RING_DEFAULT_BYTES
}

/// Z1 × Z6：64 片那条池上再挂一次（写行那次发布末条再跨记录），这次挂载的录制流上每一个前缀崩一次，
/// 外加写行那次发布四个记录写（两条记录 × 两块盘）的全部 16 个子集（单元写全持久、记录写任意落）。
#[test]
fn z1_a1b_crash_sweep_over_a_spilled_row_publish() {
    let stream = SharedStream::new();
    let mut devices = pool_after_the_first_transaction(&stream);
    crash_right_after_acquisition(&mut devices, 23248);
    mount_writable(&parameters(), &mut devices).expect("第一次挂载（64 片）");
    let base = memory_pool_of_sparse_devices(&devices);
    let retaining = SharedStream::retaining_contents();
    let mut second_devices: Devices = crash_state_devices(&base, &[], &[], &retaining);
    let second = mount_writable(&parameters(), &mut second_devices).expect("第二次挂载");
    let operations = retaining.retained_operations();
    println!(
        "OBS a1b second mount instance={} operations={}",
        second.output.instance.0,
        operations.len()
    );
    let mut outcomes: std::collections::BTreeMap<String, Vec<usize>> = Default::default();
    for prefix in 0..=operations.len() {
        let mut image = base.clone();
        image.apply(&operations[..prefix]);
        outcomes.entry(judge_crash_state(&image)).or_default().push(prefix);
    }
    for (outcome, prefixes) in &outcomes {
        println!(
            "OBS a1b prefix outcome [{} states, first {:?}, last {:?}]: {outcome}",
            prefixes.len(),
            prefixes.first(),
            prefixes.last()
        );
    }
    let journal_writes: Vec<usize> = operations
        .iter()
        .enumerate()
        .filter(|(_, operation)| is_journal_write(operation))
        .map(|(index, _)| index)
        .collect();
    let row_record_writes: Vec<usize> = journal_writes.iter().copied().take(4).collect();
    println!("OBS a1b row publish record writes at {row_record_writes:?}");
    let mut subset_outcomes: std::collections::BTreeMap<String, Vec<u32>> = Default::default();
    for mask in 0u32..16 {
        let mut image = base.clone();
        image.apply(&operations[..row_record_writes[0]]);
        for (bit, index) in row_record_writes.iter().enumerate() {
            if mask & (1 << bit) != 0 {
                image.apply(&operations[*index..=*index]);
            }
        }
        subset_outcomes.entry(judge_crash_state(&image)).or_default().push(mask);
    }
    for (outcome, masks) in &subset_outcomes {
        println!("OBS a1b subset outcome masks {masks:?}: {outcome}");
    }
    assert!(
        outcomes.keys().chain(subset_outcomes.keys()).all(|outcome| !outcome.contains("PANIC")
            && !outcome.contains("violated=[\"")),
        "有崩溃状态 panic 或挂载之后 checker 判红"
    );
}

use singlefs_harness::history::{
    run_history_campaign, FindingShrinking, GenerationWeights, HistoryExecution,
};

fn campaign(tag: &str, first_seed: u64, seeds: u64, operations: usize, weights: &GenerationWeights) {
    let started = std::time::Instant::now();
    let report = run_history_campaign(
        first_seed,
        seeds,
        operations,
        weights,
        HistoryExecution::CHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        5,
        FindingShrinking::ReportSeedsOnly,
    );
    println!(
        "OBS {tag} seeds=[{first_seed}, +{seeds}) ops={operations} known_red={} new_findings={} secs={:.1}",
        report.known_red_hits.len(),
        report.new_findings.len(),
        started.elapsed().as_secs_f64()
    );
    println!("{}", report.render());
}

/// Z4 × Z6：随机历史（挂载、回退、抬 F、覆盖写、冷启动）在这一轮新取的种子区间上，每步之后跑 checker 与理想模型；
/// establish_instance 那条断言 panic 会被执行器接住、记成新发现。种子区间与门禁那几窗不重叠（起点 + 10^6）。
#[test]
#[ignore]
fn z6_a2_random_histories_on_fresh_seeds() {
    let base = singlefs_harness::crash_injection::SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE;
    let seeds: u64 = std::env::var("OPUS_SEEDS").ok().and_then(|text| text.parse().ok()).unwrap_or(200);
    let operations: usize = std::env::var("OPUS_OPS").ok().and_then(|text| text.parse().ok()).unwrap_or(60);
    campaign("a2-broad", base.wrapping_add(1_000_000), seeds, operations, &GenerationWeights::BROAD);
    campaign(
        "a2-rollback",
        base.wrapping_add(2_000_000),
        seeds,
        operations,
        &GenerationWeights::ROLLBACK_AFTER_RAISING_THE_FLOOR,
    );
    campaign(
        "a2-reuse",
        base.wrapping_add(3_000_000),
        seeds,
        operations,
        &GenerationWeights::REUSE_AFTER_RAISING_THE_FLOOR,
    );
}

use singlefs_core::make_filesystem::make_filesystem;
use singlefs_core::mount::{mount_rollback, RollbackTarget, ShadowLedger};
use singlefs_core::recovery::{choose_system_configuration, readable_roots};

#[derive(Clone, Copy, Debug)]
enum Start {
    WithFile,
    WithoutFile,
}

fn fresh_devices(stream: &SharedStream) -> Devices {
    DISKS
        .iter()
        .map(|identity| {
            (
                *identity,
                RecordingBlockDevice::with_shared_stream(
                    *identity,
                    SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
                    stream.clone(),
                ),
            )
        })
        .collect()
}

fn build(start: Start, acquisitions: u32, mounts: u32) -> Devices {
    let stream = SharedStream::new();
    let mut devices = match start {
        Start::WithFile => pool_after_the_first_transaction(&stream),
        Start::WithoutFile => {
            let mut devices = fresh_devices(&stream);
            make_filesystem(&parameters(), &mut devices).expect("mkfs");
            mount_writable(&parameters(), &mut devices).expect("mkfs 之后第一次可写挂载");
            devices
        }
    };
    crash_right_after_acquisition(&mut devices, acquisitions);
    for mount_number in 0..mounts {
        mount_writable(&parameters(), &mut devices)
            .unwrap_or_else(|error| panic!("第 {mount_number} 次挂载 {error:?}"));
    }
    devices
}

fn ring_roots(devices: &Devices) -> Vec<(u32, u64)> {
    let system_configuration = choose_system_configuration(devices).expect("系统配置");
    let mut roots: Vec<(u32, u64)> = readable_roots(
        devices,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    )
    .into_iter()
    .map(|root| (root.instance.0, root.checkpoint_txg.0))
    .collect();
    roots.sort_unstable_by_key(|(instance, txg)| (*txg, *instance));
    roots.dedup();
    roots
}

/// 回退到一条根，再普通挂载一次；每一步 panic 接住，挂载成了跑 checker。
fn rollback_then_mount(image: &MemoryPool, target: (u32, u64)) -> String {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut devices: Devices = crash_state_devices(image, &[], &[], &SharedStream::new());
        let rolled = mount_rollback(
            &parameters(),
            &mut devices,
            RollbackTarget {
                instance: singlefs_core::address::InstanceGeneration(target.0),
                checkpoint_txg: singlefs_core::address::CheckpointTxg(target.1),
            },
            ShadowLedger::On,
        );
        let first = match rolled {
            Ok(mounted) => format!(
                "rollback ok pages_rows={} violated={:?}",
                mounted.output.rows_written.len(),
                violated(&devices).into_iter().map(|(name, _)| name).collect::<Vec<_>>()
            ),
            Err(error) => {
                let text = format!("{error:?}");
                return format!("rollback err {}", text.split(['{', '(']).next().unwrap_or(""));
            }
        };
        let second = match mount_writable(&parameters(), &mut devices) {
            Ok(_) => format!(
                "remount ok violated={:?}",
                violated(&devices).into_iter().map(|(name, _)| name).collect::<Vec<_>>()
            ),
            Err(error) => format!("remount err {error:?}").chars().take(120).collect(),
        };
        format!("{first} | {second}")
    }))
    .unwrap_or_else(|_| "PANIC".to_string())
}

/// Z6：回退到环里每一条根（含最旧那条），实例表一片与两片、带文件与树表 0 条，挂载次数 1..=8 放开扫。
#[test]
#[ignore]
fn z6_a4_rollback_to_every_ring_root_across_shapes() {
    let mut outcomes: std::collections::BTreeMap<String, Vec<String>> = Default::default();
    for start in [Start::WithFile, Start::WithoutFile] {
        for acquisitions in [0u32, 366, 367, 368, 369] {
            for mounts in 1u32..=8 {
                let devices = build(start, acquisitions, mounts);
                let image = memory_pool_of_sparse_devices(&devices);
                let roots = ring_roots(&devices);
                for (index, target) in roots.iter().enumerate() {
                    let outcome = rollback_then_mount(&image, *target);
                    outcomes.entry(outcome).or_default().push(format!(
                        "{start:?}/acq{acquisitions}/m{mounts}/root#{index}of{}={target:?}",
                        roots.len()
                    ));
                }
            }
        }
    }
    for (outcome, cases) in &outcomes {
        println!(
            "OBS a4 [{} cases, e.g. {:?}]: {outcome}",
            cases.len(),
            cases.iter().take(3).collect::<Vec<_>>()
        );
    }
    assert!(outcomes.keys().all(|outcome| outcome != "PANIC" && !outcome.contains("violated=[\"")));
}

use std::cell::Cell;
use std::rc::Rc;

use singlefs_core::address::{CheckpointTxg, DeviceOffsetInBytes, InstanceGeneration};
use singlefs_core::block_device::{BlockDevice, BlockDeviceError, WriteDurability};
use singlefs_core::mount::{raise_rollback_floor, rollback_floor_ceiling, MountError};
use singlefs_core::recovery::instance_table_chain_of_root;
use singlefs_core::transaction::{publish_overwrite, FirstFile, TransactionOutput};

/// 第 N 次写（整池数，从 0 数）报块设备错、一个字节不落；之后照常。计数器几块盘共用。
struct FailTheNthWrite {
    inner: RecordingBlockDevice<SparseBlockDevice>,
    writes_before_the_failure: Rc<Cell<Option<u64>>>,
}

impl BlockDevice for FailTheNthWrite {
    fn read_at(&self, offset: DeviceOffsetInBytes, buffer: &mut [u8]) -> Result<(), BlockDeviceError> {
        self.inner.read_at(offset, buffer)
    }
    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        match self.writes_before_the_failure.get() {
            Some(0) => {
                self.writes_before_the_failure.set(None);
                return Err(singlefs_harness::fault_injection::injected_block_device_error("写"));
            }
            Some(remaining) => self.writes_before_the_failure.set(Some(remaining - 1)),
            None => {}
        }
        self.inner.write_at(offset, bytes, durability)
    }
    fn write_zeroes_at(&mut self, offset: DeviceOffsetInBytes, length: u64) -> Result<(), BlockDeviceError> {
        self.inner.write_zeroes_at(offset, length)
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
}

type FailingDevices = Vec<(DeviceIdentity, FailTheNthWrite)>;

fn failing_devices_of(image: &MemoryPool, armed: &Rc<Cell<Option<u64>>>) -> FailingDevices {
    crash_state_devices(image, &[], &[], &SharedStream::new())
        .into_iter()
        .map(|(identity, inner)| {
            (
                identity,
                FailTheNthWrite {
                    inner,
                    writes_before_the_failure: armed.clone(),
                },
            )
        })
        .collect()
}

fn image_of_failing(devices: &FailingDevices) -> MemoryPool {
    MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.inner.inner().image.clone()))
            .collect(),
        device_size_in_bytes: IMAGE_BYTES,
    }
}

fn violated_on(image: &MemoryPool) -> Vec<(&'static str, String)> {
    check_pool_image(image)
        .into_iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(detail) => Some((invariant, format!("{detail:?}").chars().take(400).collect())),
            _ => None,
        })
        .collect()
}

fn overwrite(
    devices: &mut FailingDevices,
    allocator: &mut singlefs_core::allocator::PoolAllocator,
    current: &TransactionOutput,
    seed: u8,
) -> Result<TransactionOutput, singlefs_core::transaction::PublishError> {
    let publish_parameters = parameters();
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let content: Vec<u8> = (0..5000u32).map(|index| (index as u8).wrapping_mul(seed)).collect();
    publish_overwrite(
        &mut writer,
        allocator,
        current,
        FirstFile {
            content: &content,
            write_time_seconds: common::FIXED_WRITE_TIME_SECONDS + u64::from(seed),
        },
        current.root.instance,
    )
}

/// 抬 F 那一串（D16（发布语义） 已定项 1）：`fail_at` = Some(n) 时这一串里整池第 n 次写报错；之后在同一个进程里接着覆盖写一次，
/// 再跑池级 checker；另起一个进程重挂一次再跑 checker。交回每一步的观测。
fn raise_then_continue(fail_at: Option<u64>) -> Vec<String> {
    let stream = SharedStream::new();
    let base_devices = pool_after_the_first_transaction(&stream);
    let base = memory_pool_of_sparse_devices(&base_devices);
    let armed: Rc<Cell<Option<u64>>> = Rc::new(Cell::new(None));
    let mut devices = failing_devices_of(&base, &armed);
    let mounted = mount_writable(&parameters(), &mut devices).expect("挂载");
    let mut allocator = mounted.allocator;
    let PoolVersion::WithFile(mut current) = mounted.current else {
        panic!("带文件")
    };
    for seed in 1..=6u8 {
        current = overwrite(&mut devices, &mut allocator, &current, seed).expect("覆盖写");
    }
    let system_configuration = choose_system_configuration(&devices).expect("系统配置");
    let table = instance_table_chain_of_root(&devices, &current.root).expect("实例表").records;
    let ceiling = rollback_floor_ceiling(&devices, &system_configuration, current.root.rollback_floor, &table)
        .expect("上限");
    let mut observations = vec![format!(
        "before raise: txg={} F={} ceiling={} free_slots_dev0={} deferred_dev0={}",
        current.root.checkpoint_txg.0,
        current.root.rollback_floor.0,
        ceiling.0,
        allocator.devices[0].free_slots(),
        allocator.devices[0].deferred_slots()
    )];
    armed.set(fail_at);
    let raised = raise_rollback_floor(
        &parameters(),
        &mut devices,
        &mut allocator,
        &mut current,
        ceiling,
        ShadowLedger::On,
    );
    armed.set(None);
    observations.push(match &raised {
        Ok(raised) => format!("raise ok publishes={} reclaimed={}", raised.publishes.len(), raised.reclaimed.len()),
        Err(MountError::RaiseFloorSequencePublishFailed(failed)) => format!(
            "raise failed persisted={} failed_accounts={} cause={:?}",
            failed.writes_of_persisted_publishes.len(),
            failed.writes_of_failed_publishes.len(),
            failed.cause
        ),
        Err(other) => format!("raise err {other:?}"),
    });
    observations.push(format!(
        "after raise: current txg={} F={} free_slots_dev0={} deferred_dev0={} checker={:?}",
        current.root.checkpoint_txg.0,
        current.root.rollback_floor.0,
        allocator.devices[0].free_slots(),
        allocator.devices[0].deferred_slots(),
        violated_on(&image_of_failing(&devices)).into_iter().map(|(name, _)| name).collect::<Vec<_>>()
    ));
    let image_after_raise = image_of_failing(&devices);
    let cold = recover(&image_after_raise, JournalPolicy::Consult);
    observations.push(format!(
        "cold recovery right after the raise: effective={:?} applied={}",
        cold.effective_root, cold.journal.prefix_applied
    ));
    match overwrite(&mut devices, &mut allocator, &current, 7) {
        Ok(next) => observations.push(format!(
            "same-process overwrite ok txg={} F={} checker={:?}",
            next.root.checkpoint_txg.0,
            next.root.rollback_floor.0,
            violated_on(&image_of_failing(&devices))
        )),
        Err(error) => observations.push(format!("same-process overwrite err {error:?}")),
    }
    let mut remounted = failing_devices_of(&image_after_raise, &Rc::new(Cell::new(None)));
    match mount_writable(&parameters(), &mut remounted) {
        Ok(mounted) => {
            let mut allocator = mounted.allocator;
            let PoolVersion::WithFile(current) = mounted.current else { panic!("带文件") };
            let next = overwrite(&mut remounted, &mut allocator, &current, 9).expect("重挂之后覆盖写");
            observations.push(format!(
                "remount+overwrite txg={} F={} checker={:?}",
                next.root.checkpoint_txg.0,
                next.root.rollback_floor.0,
                violated_on(&image_of_failing(&remounted)).into_iter().map(|(name, _)| name).collect::<Vec<_>>()
            ));
        }
        Err(error) => observations.push(format!("remount err {error:?}")),
    }
    observations
}

/// Z4：抬 F 那一串发布失败的账。对照：不注入；注入：第一次空发布的第一条 journal 记录写（整池第 8 次写，前 8 次是四个固定点 × 两盘）报错；
/// 第二次空发布的根槽写报错（第一次那 13 次写之后：8 单元 + 2 记录 + 1 根 + 2 系统配置 = 13，第二次 8 + 2 = 10 ⇒ 第 23 次）。
#[test]
fn z4_a3_raise_floor_failure_then_continue_in_the_same_process() {
    for (label, fail_at) in [
        ("control", None),
        ("fail-first-record", Some(8)),
        ("fail-first-sysconfig", Some(11)),
        ("fail-second-root", Some(23)),
        ("fail-second-sysconfig", Some(24)),
    ] {
        for line in raise_then_continue(fail_at) {
            println!("OBS a3 {label}: {line}");
        }
    }
}

/// Z4：树表 0 条的一版上写行，实例表链 `pages` 片（< 67，点名项装得下一条记录，避开已知的「≥ 67 片越界」），之后连着可写挂载，
/// 看有没有一次在取号之后才被分配记录准入拒（`AllocationRecordsExceedOneNode`）——取号之前那道准入对树表 0 条那一臂不判分配记录条数。
fn without_file_mounts_until_refused(pages: u32, mounts: u32) -> Vec<String> {
    let stream = SharedStream::new();
    let mut devices = fresh_devices(&stream);
    make_filesystem(&parameters(), &mut devices).expect("mkfs");
    mount_writable(&parameters(), &mut devices).expect("mkfs 之后第一次可写挂载");
    // 下一次挂载取号 k + 2，写 [1, k + 2) 共 k + 1 行。
    let rows_wanted = (pages - 1) * 369 + 1;
    crash_right_after_acquisition(&mut devices, rows_wanted - 1);
    let mut observations = Vec::new();
    for mount_number in 0..mounts {
        let instances_before = singlefs_core::transaction::instance_generation_to_acquire(
            &PoolWriter::new(&parameters(), devices.as_mut_slice()),
        );
        match mount_writable(&parameters(), &mut devices) {
            Ok(mounted) => {
                let records = mounted.allocator.records().len();
                observations.push(format!(
                    "mount#{mount_number} ok instance={} rows_written={} allocation_records={records} released={}",
                    mounted.output.instance.0,
                    mounted.output.rows_written.len(),
                    mounted.allocator.records().iter().filter(|record| record.is_released).count(),
                ));
            }
            Err(error) => {
                let instances_after = singlefs_core::transaction::instance_generation_to_acquire(
                    &PoolWriter::new(&parameters(), devices.as_mut_slice()),
                );
                observations.push(format!(
                    "mount#{mount_number} err (next instance before {} after {}) {:?}",
                    instances_before.0,
                    instances_after.0,
                    format!("{error:?}").chars().take(300).collect::<String>()
                ));
            }
        }
    }
    observations
}

#[test]
#[ignore]
fn z4_a5_without_file_row_publish_allocation_record_admission_after_acquisition() {
    let pages: u32 = std::env::var("OPUS_PAGES").ok().and_then(|text| text.parse().ok()).unwrap_or(66);
    let mounts: u32 = std::env::var("OPUS_MOUNTS").ok().and_then(|text| text.parse().ok()).unwrap_or(30);
    for line in without_file_mounts_until_refused(pages, mounts) {
        println!("OBS a5 pages={pages}: {line}");
    }
}

/// A5 的扫描：片数从 `OPUS_PAGES_FROM` 到 66，每档连挂 `OPUS_MOUNTS` 次，报第一次失败在第几次、失败前后号差多少。
#[test]
#[ignore]
fn z4_a5b_page_count_sweep() {
    let from: u32 = std::env::var("OPUS_PAGES_FROM").ok().and_then(|text| text.parse().ok()).unwrap_or(2);
    let mounts: u32 = std::env::var("OPUS_MOUNTS").ok().and_then(|text| text.parse().ok()).unwrap_or(14);
    for pages in from..=66 {
        let observations = without_file_mounts_until_refused(pages, mounts);
        let first_error = observations.iter().position(|line| line.contains(" err "));
        println!(
            "OBS a5b pages={pages} mounts={mounts} first_error_at={first_error:?} last_ok={:?} first_err={:?}",
            first_error.and_then(|index| index.checked_sub(1)).and_then(|index| observations.get(index)).or(observations.last()),
            first_error.map(|index| &observations[index]),
        );
    }
}

/// A5 的最朴素形态：只做过 mkfs、从不发文件，一路可写挂载 `OPUS_MOUNTS` 次（不靠取号之后崩溃），看分配记录条数怎么涨、会不会在取号之后被拒。
#[test]
#[ignore]
fn z4_a5c_plain_mounts_on_a_pool_without_file() {
    let mounts: u32 = std::env::var("OPUS_MOUNTS").ok().and_then(|text| text.parse().ok()).unwrap_or(400);
    let stream = SharedStream::new();
    let mut devices = fresh_devices(&stream);
    make_filesystem(&parameters(), &mut devices).expect("mkfs");
    let mut last_ok = String::new();
    for mount_number in 0..mounts {
        let before = singlefs_core::transaction::instance_generation_to_acquire(
            &PoolWriter::new(&parameters(), devices.as_mut_slice()),
        );
        match mount_writable(&parameters(), &mut devices) {
            Ok(mounted) => {
                last_ok = format!(
                    "mount#{mount_number} ok instance={} rows_written={} allocation_records={} released={}",
                    mounted.output.instance.0,
                    mounted.output.rows_written.len(),
                    mounted.allocator.records().len(),
                    mounted.allocator.records().iter().filter(|record| record.is_released).count(),
                );
                if mount_number % 25 == 0 {
                    println!("OBS a5c {last_ok}");
                }
            }
            Err(error) => {
                let after = singlefs_core::transaction::instance_generation_to_acquire(
                    &PoolWriter::new(&parameters(), devices.as_mut_slice()),
                );
                println!("OBS a5c last ok: {last_ok}");
                println!(
                    "OBS a5c mount#{mount_number} err (next instance before {} after {}) {}",
                    before.0,
                    after.0,
                    format!("{error:?}").chars().take(300).collect::<String>()
                );
                return;
            }
        }
    }
    println!("OBS a5c no refusal in {mounts} mounts; last: {last_ok}");
}

/// A5 的对照：同样的片数与挂载节奏，但起点是带文件的一版（`Start::WithFile`）：取号之前那道准入判得到分配记录条数，
/// 被拒时应是 `RowPublishAdmissionRefusedBeforeAcquisition`、号不涨。
#[test]
#[ignore]
fn z4_a5d_control_with_file() {
    let pages: u32 = std::env::var("OPUS_PAGES").ok().and_then(|text| text.parse().ok()).unwrap_or(66);
    let mounts: u32 = std::env::var("OPUS_MOUNTS").ok().and_then(|text| text.parse().ok()).unwrap_or(12);
    let stream = SharedStream::new();
    let mut devices = pool_after_the_first_transaction(&stream);
    let rows_wanted = (pages - 1) * 369 + 1;
    crash_right_after_acquisition(&mut devices, rows_wanted - 1);
    for mount_number in 0..mounts {
        let before = singlefs_core::transaction::instance_generation_to_acquire(
            &PoolWriter::new(&parameters(), devices.as_mut_slice()),
        );
        let outcome = match mount_writable(&parameters(), &mut devices) {
            Ok(mounted) => format!(
                "ok instance={} allocation_records={}",
                mounted.output.instance.0,
                mounted.allocator.records().len()
            ),
            Err(error) => format!("err {}", format!("{error:?}").chars().take(220).collect::<String>()),
        };
        let after = singlefs_core::transaction::instance_generation_to_acquire(
            &PoolWriter::new(&parameters(), devices.as_mut_slice()),
        );
        println!(
            "OBS a5d pages={pages} mount#{mount_number} next instance before {} after {}: {outcome}",
            before.0, after.0
        );
    }
}

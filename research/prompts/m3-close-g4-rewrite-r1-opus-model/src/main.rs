//! 攻方原型（m3-close-g4-rewrite-r1，被攻过零轮）：没有读故障、只有崩溃的几条流里，某块盘两槽系统配置都自证不过的崩溃状态有几个；
//! 这些状态上可写挂载（丙' 补写闸在不在）做成还是拒，拒了判官 G4 那一臂与只判 U6 的判官各判什么。
//! 崩溃状态照崩溃点重放的枚举域取（`enumerate_crash_states_with_expansion` + `full_expansion`，段内整写子集与原地覆写撕裂态都展开）。
//! 流全在这份原型里自己造（harness 的历史执行器、mkfs 入口），不从崩溃枚举用例拷。
//! 运行时开关（只在草稿仓副本里打的补丁认）：`SINGLEFS_PROTO_MUT_B1`（盘 0 轮换同段清另一槽）、`SINGLEFS_PROTO_GATE_NEVER_PASSES`（补写闸恒不过）。
//! 用法：`g4-attack-prototype <流名>`，流名见 `main`。输出一行一个 `name=` 计数。

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use singlefs_checker_tier::crash::{
    crash_state_count_with_torn_in_place_overwrites, full_expansion,
    CrashPointReplayObserverCounts, CrashPointReplayVersionTable,
};
use singlefs_checker_tier::crash_state_enumeration::{
    enumerate_crash_states_with_expansion, CrashPointReplayItem, CrashPointReplayObserver,
};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::{make_filesystem, MakeFilesystemParameters};
use singlefs_core::mount::{mount_writable_with_space_admission, MountError};
use singlefs_core::recovery::{verified_system_configuration_slots, PoolReader, RecoveryOutcome, RecoveryReport};
use singlefs_harness::history::{
    execute_history_with, ContentChoice, ContentLength, GeneratedHistory, HistoryDeviceWidth,
    HistoryExecution, HistoryOperation, HistorySeed, HistoryStartingPoint, PerStepChecker,
};
use singlefs_harness::memory_pool::{
    writes_and_segments, CrashImage, MemoryPool, PublishedVersion, RetainedWrite, SparseBlockDevice,
};
use singlefs_harness::mount_transparency::{
    judge_writable_mount_refusal, judge_writable_mount_refusal_of_a_crash_state,
    DiskAfterTheRefusal, ReadFaultsInTheCrashState, ReadOnlyReadBackOfTheCrashImage,
    WritableMountRefusalVerdict,
};
use singlefs_harness::segments::{FixedGeometry, StepKind};
use singlefs_harness::{RecordingBlockDevice, RetainedOperation, SharedStream};

const SMALL: HistoryDeviceWidth = HistoryDeviceWidth::UnitAreaOf384Slots;
/// 一次跑的崩溃状态数上限（定义第 3b 步：约 10⁶）。
const STATE_LIMIT: u64 = 1_000_000;
const THIS_FILE: &str = "research/prompts/m3-close-g4-rewrite-r1-opus-model/src/main.rs";

fn geometry() -> FixedGeometry {
    let parameters = SMALL.parameters();
    FixedGeometry {
        fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
        journal_ring_bytes: parameters.geometry.journal_ring_bytes,
        root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
    }
}

fn overwrite(fill_seed: u64) -> HistoryOperation {
    HistoryOperation::PublishOverwrite(ContentChoice {
        length: ContentLength::InsideOneDataUnit { selector: 0 },
        fill_seed,
    })
}

/// 从 `starting_point` 起跑 `operations`，交回录制流、mkfs 占的步数与模型认下的带文件的那几版。
fn recorded(
    starting_point: HistoryStartingPoint,
    operations: Vec<HistoryOperation>,
) -> (Vec<RetainedOperation>, usize, Vec<PublishedVersion>) {
    let stream = SharedStream::retaining_contents();
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point,
        operations,
    };
    let run = execute_history_with(
        &history,
        HistoryExecution {
            per_step_checker: PerStepChecker::Skipped,
            device_width: SMALL,
            space_admission: SpaceAdmission::JudgedByTheFormula,
        },
        &stream,
        &mut |_observation| {},
    );
    assert_eq!(run.outcomes.len(), history.operations.len(), "历史每一步都跑完：{:?}", run.ending);
    let versions = run
        .committed_versions_at_the_end
        .iter()
        .filter_map(|(key, content)| {
            content.as_ref().map(|content| PublishedVersion {
                instance: InstanceGeneration(u32::try_from(key.instance.0).expect("实例代号装得进 u32")),
                checkpoint_txg: CheckpointTxg(key.checkpoint_txg.0),
                content: content.clone(),
            })
        })
        .collect();
    (stream.retained_operations(), run.operations_written_by_make_filesystem, versions)
}

fn pool_of(operations: &[RetainedOperation]) -> MemoryPool {
    let mut pool = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], SMALL.device_bytes());
    pool.apply(operations);
    pool
}

/// 计数表：键是 `name=` 之后那一截，值是状态数。观察者在录入段多线程里调，计数放锁里（库里的计数另走 `counts`）。
type Tally = Mutex<BTreeMap<String, u64>>;

fn bump(tally: &Tally, key: String) {
    *tally.lock().expect("计数锁").entry(key).or_insert(0) += 1;
}

/// 拒因的成员名（Debug 串第一个括号之前那一截）。
fn reason_name(error: &MountError) -> String {
    let text = format!("{error:?}");
    text.split(['(', ' ', '{']).next().unwrap_or("").to_string()
}

fn verdict_name(verdict: &WritableMountRefusalVerdict) -> String {
    match verdict {
        WritableMountRefusalVerdict::Legal(_) => "legal".to_string(),
        WritableMountRefusalVerdict::Illegal { conditions_not_met, .. } => {
            format!("illegal{conditions_not_met:?}").replace(' ', "")
        }
        WritableMountRefusalVerdict::ReasonNotOnTheD23List => "reason_not_on_the_list".to_string(),
    }
}

/// 在 `pool` 的副本上起一次可写挂载（读不报错）：交回结局与挂载之后的盘。
/// 调用方参数里的 fsid 取 `filesystem_identifier`（用过的盘上重做 mkfs 那条流是新 fsid）。
fn mount_on(pool: &MemoryPool, filesystem_identifier: &[u8; 16]) -> (Result<(), MountError>, MemoryPool) {
    let mut parameters = SMALL.parameters();
    parameters.filesystem_identifier = *filesystem_identifier;
    let physical_block_size = PhysicalBlockSizeInBytes(parameters.geometry.physical_block_size);
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = pool
        .devices
        .iter()
        .map(|(identity, sectors)| {
            let mut device = SparseBlockDevice::new(pool.device_size_in_bytes, physical_block_size);
            device.image = sectors.clone();
            (*identity, device)
        })
        .collect();
    let result = mount_writable_with_space_admission(&parameters, &mut devices, SpaceAdmission::JudgedByTheFormula)
        .map(|_mounted| ());
    let after = MemoryPool {
        devices: devices.into_iter().map(|(identity, device)| (identity, device.image)).collect(),
        device_size_in_bytes: pool.device_size_in_bytes,
    };
    (result, after)
}

/// 本池自证过的系统配置一份都没有的盘。
fn devices_without_a_self_verified_slot(reader: &dyn PoolReader, filesystem_identifier: &[u8; 16]) -> Vec<DeviceIdentity> {
    let spacing = u64::from(SMALL.parameters().geometry.fixed_structure_slot_spacing);
    reader
        .device_identities()
        .into_iter()
        .filter(|device| verified_system_configuration_slots(reader, *device, spacing, filesystem_identifier).is_empty())
        .collect()
}

/// 一个两槽全废的盘面上起可写挂载、判一次（拒了的照 G4 判官与只判 U6 的判官各判一次；崩溃态按没注入读故障、只读读出按在允许集合里交）。
fn mount_and_judge(pool: &MemoryPool, effective_root: Option<(InstanceGeneration, CheckpointTxg)>, filesystem_identifier: &[u8; 16], tally: &Tally, prefix: &str) {
    let (result, after) = mount_on(pool, filesystem_identifier);
    match result {
        Ok(()) => {
            let still_without = devices_without_a_self_verified_slot(&after, filesystem_identifier);
            bump(tally, format!("{prefix}remount=mounted devices_without_a_self_verified_slot_after_the_mount={}", still_without.len()));
        }
        Err(error) => {
            let disk = if *pool == after { DiskAfterTheRefusal::ByteForByteUnchanged } else { DiskAfterTheRefusal::Changed };
            let with_g4 = judge_writable_mount_refusal_of_a_crash_state(
                pool, effective_root, &error, disk,
                ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet, ReadFaultsInTheCrashState::NoneInjected,
            );
            let u6_only = judge_writable_mount_refusal(pool, effective_root, &error, disk, ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet);
            bump(tally, format!(
                "{prefix}remount=refused reason={} verdict_with_g4={} verdict_u6_only={}",
                reason_name(&error), verdict_name(&with_g4), verdict_name(&u6_only)
            ));
        }
    }
}

fn observe(image: &CrashImage<'_>, report: &RecoveryReport, filesystem_identifier: &[u8; 16], tally: &Tally) {
    bump(tally, "states".to_string());
    let without = devices_without_a_self_verified_slot(image, filesystem_identifier);
    if without.is_empty() {
        return;
    }
    let recovery = match &report.outcome {
        RecoveryOutcome::NoFile { .. } => "no_file".to_string(),
        RecoveryOutcome::FileRead { .. } => "file_read".to_string(),
        RecoveryOutcome::Failed { failure, .. } => format!("failed_{}", format!("{failure:?}").split(['(', ' ', '{']).next().unwrap_or("")),
    };
    let prefix = format!("devices_without_a_self_verified_slot={:?} recovery={recovery} ", without.iter().map(|d| d.0).collect::<Vec<_>>());
    bump(tally, format!("{prefix}states"));
    if without.len() == image.device_identities().len() || matches!(report.outcome, RecoveryOutcome::Failed { .. }) {
        return;
    }
    let pool = singlefs_checker_tier::writable_mount_phases::materialized_crash_image(image);
    mount_and_judge(&pool, report.effective_root, filesystem_identifier, tally, &prefix);
}

/// 按崩溃点重放的枚举域把一条流的全部崩溃状态走一遍（每段都展开），逐状态 [`observe`]；状态数先算，超过 [`STATE_LIMIT`] 不跑。
fn enumerate(name: &str, base: &MemoryPool, writes: &[RetainedWrite], segments: &[Vec<usize>], versions: &[PublishedVersion], filesystem_identifier: [u8; 16]) {
    let count = crash_state_count_with_torn_in_place_overwrites(base, writes, segments, &full_expansion);
    println!("name={name} state_count={count} segments={:?}", segments.iter().map(Vec::len).collect::<Vec<_>>());
    if count > STATE_LIMIT {
        println!("name={name} skipped=over_the_state_limit");
        return;
    }
    let Some(judged_root_index) = writes.iter().rposition(|write| write.kind == StepKind::RootRecordFua) else {
        println!("name={name} skipped=no_root_write");
        return;
    };
    let tally = Tally::default();
    let observed = AtomicU64::new(0);
    let toggles = format!(
        "{name} mut_b1={} gate_never_passes={}",
        std::env::var_os("SINGLEFS_PROTO_MUT_B1").is_some(),
        std::env::var_os("SINGLEFS_PROTO_GATE_NEVER_PASSES").is_some()
    );
    let digest = singlefs_harness::sha256::sha256_digest(toggles.as_bytes());
    let observe_one = |_ordinal: u64, image: &CrashImage<'_>, report: &RecoveryReport, counts: &mut CrashPointReplayObserverCounts| {
        observe(image, report, &filesystem_identifier, &tally);
        observed.fetch_add(1, Ordering::Relaxed);
        counts.add("observed", 1);
    };
    let started = std::time::Instant::now();
    let outcome = enumerate_crash_states_with_expansion(
        CrashPointReplayItem { name, setting_file: THIS_FILE },
        base,
        writes,
        segments,
        judged_root_index,
        CrashPointReplayVersionTable::with_a_file_only(versions),
        &full_expansion,
        &[CrashPointReplayObserver { name: "g4_attack", extra_inputs_digest: digest, observe: &observe_one }],
    );
    let judged = outcome.judged();
    println!(
        "name={name} toggles=[{toggles}] observed={} judged={} seconds={:.1}",
        observed.load(Ordering::Relaxed),
        judged.is_some(),
        started.elapsed().as_secs_f64()
    );
    if let Some(judged) = judged {
        println!("name={name} judged_tally={judged:?}");
    }
    for (key, value) in tally.into_inner().expect("计数锁") {
        println!("name={name} {key} count={value}");
    }
}

/// 第一个文件之后再跑 `operations` 那一截的流：基线是第一个文件之后的盘面。
fn after_first_file(name: &str, operations: Vec<HistoryOperation>) {
    let (prefix, _, _) = recorded(HistoryStartingPoint::AfterFirstFile, Vec::new());
    let (stream, _, versions) = recorded(HistoryStartingPoint::AfterFirstFile, operations);
    assert_eq!(stream[..prefix.len()], prefix[..], "同一个起点跑出同一段录制流");
    let base = pool_of(&prefix);
    let (writes, segments) = writes_and_segments(&stream[prefix.len()..], &geometry());
    enumerate(name, &base, &writes, &segments, &versions, SMALL.parameters().filesystem_identifier);
}

/// 新盘上 mkfs 自己的流（基线是全 0 的两块盘）。
fn make_filesystem_on_fresh_devices() {
    let (stream, written_by_make_filesystem, _) = recorded(HistoryStartingPoint::AfterMakeFilesystem, Vec::new());
    let base = pool_of(&[]);
    let (writes, segments) = writes_and_segments(&stream[..written_by_make_filesystem], &geometry());
    enumerate("make_filesystem_on_fresh_devices", &base, &writes, &segments, &[], SMALL.parameters().filesystem_identifier);
}

/// 用过的盘（第一个文件之后的旧池）上换一个 fsid 重做 mkfs 的流。
fn make_filesystem_over_a_used_pool() {
    let (old, _, old_versions) = recorded(HistoryStartingPoint::AfterFirstFile, Vec::new());
    let base = pool_of(&old);
    let mut parameters: MakeFilesystemParameters = SMALL.parameters();
    parameters.filesystem_identifier[..8].copy_from_slice(b"g4attack");
    assert_ne!(parameters.filesystem_identifier, SMALL.parameters().filesystem_identifier);
    let stream = SharedStream::retaining_contents();
    let physical_block_size = PhysicalBlockSizeInBytes(parameters.geometry.physical_block_size);
    let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = base
        .devices
        .iter()
        .map(|(identity, sectors)| {
            let mut device = SparseBlockDevice::new(base.device_size_in_bytes, physical_block_size);
            device.image = sectors.clone();
            (*identity, RecordingBlockDevice::with_shared_stream(*identity, device, stream.clone()))
        })
        .collect();
    make_filesystem(&parameters, &mut devices).expect("用过的盘上重做 mkfs");
    let (writes, segments) = writes_and_segments(&stream.retained_operations(), &geometry());
    enumerate("make_filesystem_over_a_used_pool", &base, &writes, &segments, &old_versions, parameters.filesystem_identifier);
}

/// 红的那条 checker 档测试同形的盘面（第一个文件之后，盘 0 两槽各在偏移 2048 翻一个字节），`unit_too` 时另按甲把盘 0 上第一个文件那次发布
/// 最后一个单元写翻一个字节（补写闸一不过）。在它上面起可写挂载、判一次。不经 checker 档第三截（那一截的入口不是公开的），只调判官。
fn hand_made_image(name: &str, unit_too: bool) {
    let (first_file, _, _) = recorded(HistoryStartingPoint::AfterFirstFile, Vec::new());
    let mut pool = pool_of(&first_file);
    let spacing = u64::from(SMALL.parameters().geometry.fixed_structure_slot_spacing);
    let slot_bytes = usize::try_from(singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    for slot_index in 0..singlefs_format::SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE {
        let offset = DeviceOffsetInBytes(slot_index * spacing);
        let mut slot = PoolReader::read(&pool, DeviceIdentity(0), offset, slot_bytes).expect("槽读得出");
        slot[2048] ^= 0xFF;
        pool.devices.get_mut(&DeviceIdentity(0)).expect("盘 0").write(offset, &slot);
    }
    if unit_too {
        let (writes, _segments) = writes_and_segments(&first_file, &geometry());
        let last_unit_on_device_zero = writes
            .iter()
            .rev()
            .find(|write| write.kind == StepKind::UnitWrite && write.device == DeviceIdentity(0))
            .expect("第一个文件在盘 0 上写了单元");
        pool.flip_byte(DeviceIdentity(0), last_unit_on_device_zero.offset, 2048);
    }
    let fields = singlefs_harness::mount_transparency::read_recovery_fields(
        &pool,
        singlefs_harness::mount_transparency::ComparedVersion::TheRecoveredVersion,
    );
    let tally = Tally::default();
    mount_and_judge(&pool, fields.report.effective_root, &SMALL.parameters().filesystem_identifier, &tally, "");
    for (key, value) in tally.into_inner().expect("计数锁") {
        println!("name={name} {key} count={value}");
    }
}

/// 补写本身中途崩溃：新盘 mkfs 崩在最后一段、盘 0 两个系统配置槽都没落（两槽全 0）的盘面上起一次可写挂载（丙' 补写盘 0），录下这次挂载的流，
/// 枚举它的崩溃状态。
fn rewrite_after_a_make_filesystem_crash() {
    let (stream, written_by_make_filesystem, _) = recorded(HistoryStartingPoint::AfterMakeFilesystem, Vec::new());
    let (writes, _segments) = writes_and_segments(&stream[..written_by_make_filesystem], &geometry());
    let mut base = pool_of(&[]);
    let kept: Vec<RetainedWrite> = writes
        .iter()
        .filter(|write| !(write.kind == StepKind::SystemConfigurationSlot && write.device == DeviceIdentity(0)))
        .cloned()
        .collect();
    assert_eq!(writes.len() - kept.len(), 2, "盘 0 的两个系统配置槽写");
    base.apply_writes(&kept);
    let parameters = SMALL.parameters();
    let physical_block_size = PhysicalBlockSizeInBytes(parameters.geometry.physical_block_size);
    let mount_stream = SharedStream::retaining_contents();
    let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = base
        .devices
        .iter()
        .map(|(identity, sectors)| {
            let mut device = SparseBlockDevice::new(base.device_size_in_bytes, physical_block_size);
            device.image = sectors.clone();
            (*identity, RecordingBlockDevice::with_shared_stream(*identity, device, mount_stream.clone()))
        })
        .collect();
    let mounted = mount_writable_with_space_admission(&parameters, &mut devices, SpaceAdmission::JudgedByTheFormula);
    println!("name=rewrite_after_a_make_filesystem_crash first_mount_ok={}", mounted.is_ok());
    drop(mounted);
    drop(devices);
    let (mount_writes, mount_segments) = writes_and_segments(&mount_stream.retained_operations(), &geometry());
    enumerate("rewrite_after_a_make_filesystem_crash", &base, &mount_writes, &mount_segments, &[], parameters.filesystem_identifier);
}

fn main() {
    let flow = std::env::args().nth(1).expect("要给流名");
    match flow.as_str() {
        "make_filesystem_on_fresh_devices" => make_filesystem_on_fresh_devices(),
        "make_filesystem_over_a_used_pool" => make_filesystem_over_a_used_pool(),
        "overwrite" => after_first_file("overwrite", vec![overwrite(1)]),
        "overwrite_remount_overwrite" => after_first_file(
            "overwrite_remount_overwrite",
            vec![overwrite(1), HistoryOperation::CloseAndMountWritable, overwrite(2)],
        ),
        // 覆盖写那一条 1048588 个状态，过了约 10⁶ 的上限：缩成零单元发布（同一条轮换写序，少了单元写那一段）。
        "empty_publish" => after_first_file("empty_publish", vec![HistoryOperation::PublishWithoutUnits]),
        "empty_publish_remount_empty_publish" => after_first_file(
            "empty_publish_remount_empty_publish",
            vec![HistoryOperation::PublishWithoutUnits, HistoryOperation::CloseAndMountWritable, HistoryOperation::PublishWithoutUnits],
        ),
        // 零单元发布在第一个文件之后一个写都不发（量过：状态数 1、没有根写）：改缩内容长度，覆盖成空内容。
        "overwrite_empty" => after_first_file(
            "overwrite_empty",
            vec![HistoryOperation::PublishOverwrite(ContentChoice { length: ContentLength::Empty, fill_seed: 1 })],
        ),
        // 空内容的覆盖写同样 1048588 个状态（单元写那一段 20 个写不随内容长度变）：再缩成只有一次关闭再可写挂载（取号写两轮、写行、暖机）。
        "remount" => after_first_file("remount", vec![HistoryOperation::CloseAndMountWritable]),
        "rewrite_after_a_make_filesystem_crash" => rewrite_after_a_make_filesystem_crash(),
        "hand_made_image_both_slots" => hand_made_image("hand_made_image_both_slots", false),
        "hand_made_image_both_slots_and_a_unit" => hand_made_image("hand_made_image_both_slots_and_a_unit", true),
        other => panic!("认不出流名 {other}"),
    }
}

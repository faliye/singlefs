//! 攻方原型（m3-close-g4-rewrite-r2，被攻过零轮）：G4-丙（按盘面判：崩溃态没注入读故障、某块盘一份本池自证过的系统配置都没有就记一条）
//! 在几条流的全部崩溃状态上判出几条，按崩溃注入摆得出的域（整写子集，「撕裂并进没持久」）与崩溃点重放多出的撕裂态分开数；
//! 判出的那几张盘面上再起可写挂载，拒了的照判官 G4 那一臂判一次，数同一张盘面上记几条。
//! 崩溃状态照崩溃点重放的枚举域取（`enumerate_crash_states_with_expansion` + `full_expansion`，段内整写子集与原地覆写撕裂态都展开）。
//! 流全在这份原型里自己造（harness 的历史执行器、可写挂载入口），不从崩溃枚举用例拷。
//! 运行时开关（只在草稿仓副本里打的补丁认）：`SINGLEFS_PROTO_MUT_B1`（盘 0 写完新槽，同一段清另一槽）、`SINGLEFS_PROTO_GATE_NEVER_PASSES`（补写闸恒不过）。
//! 用法：`g4-attack-prototype-r2 <流名>`，流名见 `main`。输出一行一个 `name=` 计数。

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
use singlefs_checker_tier::writable_mount_phases::materialized_crash_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::mount::{mount_writable_with_space_admission, MountError};
use singlefs_core::recovery::{
    choose_system_configuration, verified_system_configuration_slots, PoolReader, RecoveryOutcome, RecoveryReport,
};
use singlefs_core::system_configuration::SystemConfiguration;
use singlefs_harness::history::{
    execute_history_with, ContentChoice, ContentLength, FloorTargetChoice, GeneratedHistory, HistoryDeviceWidth,
    HistoryExecution, HistoryOperation, HistorySeed, HistoryStartingPoint, PerStepChecker, RollbackTargetChoice,
};
use singlefs_harness::memory_pool::{writes_and_segments, CrashImage, MemoryPool, PublishedVersion, RetainedWrite, SparseBlockDevice};
use singlefs_harness::mount_transparency::{
    judge_writable_mount_refusal_of_a_crash_state, DiskAfterTheRefusal, ReadFaultsInTheCrashState,
    ReadOnlyReadBackOfTheCrashImage, RefusalConditionNotMet, WritableMountRefusalVerdict,
};
use singlefs_harness::segments::{FixedGeometry, StepKind};
use singlefs_harness::{RecordingBlockDevice, RetainedOperation, SharedStream};

const SMALL: HistoryDeviceWidth = HistoryDeviceWidth::UnitAreaOf384Slots;
/// 一次跑的崩溃状态数上限（定义第 3b 步：约 10⁶）。
const STATE_LIMIT: u64 = 1_000_000;
/// 二次崩溃那一截最多取几个第一次崩溃状态当基线（每个基线上那次挂载的二次崩溃状态一个不落）。
const SECOND_CRASH_BASES_AT_MOST: usize = 8;
const THIS_FILE: &str = "research/prompts/m3-close-g4-rewrite-r2-opus-model/src/main.rs";

fn geometry() -> FixedGeometry {
    let parameters = SMALL.parameters();
    FixedGeometry {
        fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
        journal_ring_bytes: parameters.geometry.journal_ring_bytes,
        root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
    }
}

fn overwrite(fill_seed: u64) -> HistoryOperation {
    HistoryOperation::PublishOverwrite(ContentChoice { length: ContentLength::InsideOneDataUnit { selector: 0 }, fill_seed })
}

/// 从 `starting_point` 起跑 `operations`，交回录制流与模型认下的带文件的那几版。
fn recorded(starting_point: HistoryStartingPoint, operations: Vec<HistoryOperation>) -> (Vec<RetainedOperation>, Vec<PublishedVersion>, String) {
    let stream = SharedStream::retaining_contents();
    let history = GeneratedHistory { seed: HistorySeed(0), starting_point, operations };
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
    let outcomes: String = format!("{:?}", run.outcomes).chars().take(400).collect();
    let ending = format!("steps_run={} of={} ending={:?} outcomes={outcomes}", run.outcomes.len(), history.operations.len(), run.ending)
        .replace(' ', "_");
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
    (stream.retained_operations(), versions, ending)
}

fn pool_of(operations: &[RetainedOperation]) -> MemoryPool {
    let mut pool = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], SMALL.device_bytes());
    pool.apply(operations);
    pool
}

/// 计数表：键是 `name=` 之后那一截，值是状态数。观察者在录入段多线程里调，计数放锁里。
type Tally = Mutex<BTreeMap<String, u64>>;

fn bump(tally: &Tally, key: String) {
    *tally.lock().expect("计数锁").entry(key).or_insert(0) += 1;
}

fn reason_name(error: &MountError) -> String {
    let text = format!("{error:?}");
    let head = text.split(['(', ' ', '{']).next().unwrap_or("").to_string();
    // 「读阶段判完之后系统配置又读不出」带上在哪一处拒的（逐盘核、取号那一刻）
    if let MountError::SystemConfigurationUnreadableAfterTheReadStage(inner) = error {
        let when = format!("{:?}", inner.unreadable);
        return format!("{head}/{}", when.split(['(', ' ', '{']).next().unwrap_or(""));
    }
    head
}

/// G4-丙 的谓词，照判官今天那一份的读法另写（判官里 `some_device_without_any_self_verified_system_configuration_slot_on_the_crash_image`
/// 与它的读槽函数是私有的）：择系统配置用核心层的 `choose_system_configuration`、自证用核心层的 `SystemConfiguration::parse_slot`
/// （判官今天也是这两样，见报告 W2），每块盘两槽各直接读一次，fsid 与择到的那一份相同才算本池自证过。
/// 择不到系统配置交 `None`（G4-丙 无从判；判官 G4 那一臂这时按真）。交回一份本池自证过的都没有的盘号。
fn devices_without_a_self_verified_slot_read_as_the_judge_reads(image: &dyn PoolReader) -> Option<Vec<u32>> {
    let Ok(chosen) = choose_system_configuration(image) else {
        return None;
    };
    let spacing = u64::from(chosen.immutable.sizes.fixed_structure_slot_spacing);
    let filesystem_identifier = chosen.immutable.filesystem_identifier;
    let slot_bytes = usize::try_from(singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    Some(
        image
            .device_identities()
            .into_iter()
            .filter(|device| {
                (0..singlefs_format::SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE).all(|slot_index| {
                    match image.read(*device, DeviceOffsetInBytes(slot_index * spacing), slot_bytes) {
                        None => true,
                        Some(bytes) => !matches!(
                            SystemConfiguration::parse_slot(&bytes),
                            Ok(slot) if slot.immutable.filesystem_identifier == filesystem_identifier
                        ),
                    }
                })
            })
            .map(|device| device.0)
            .collect(),
    )
}

/// 第一轮那一份（核心层 `verified_system_configuration_slots`，按调用方给的 fsid）：与判官读法对照，两样不同就另记一行。
fn devices_without_a_self_verified_slot_read_by_the_core(reader: &dyn PoolReader) -> Vec<u32> {
    let parameters = SMALL.parameters();
    let spacing = u64::from(parameters.geometry.fixed_structure_slot_spacing);
    reader
        .device_identities()
        .into_iter()
        .filter(|device| {
            verified_system_configuration_slots(reader, *device, spacing, &parameters.filesystem_identifier).is_empty()
        })
        .map(|device| device.0)
        .collect()
}

/// 每块盘本池自证过的槽数（判官读法），按盘号排：二次崩溃那一截挑基线用。
fn self_verified_slot_counts(image: &dyn PoolReader) -> Vec<usize> {
    let Ok(chosen) = choose_system_configuration(image) else {
        return Vec::new();
    };
    let spacing = u64::from(chosen.immutable.sizes.fixed_structure_slot_spacing);
    let slot_bytes = usize::try_from(singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    image
        .device_identities()
        .into_iter()
        .map(|device| {
            (0..singlefs_format::SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE)
                .filter(|slot_index| {
                    image.read(device, DeviceOffsetInBytes(slot_index * spacing), slot_bytes).is_some_and(|bytes| {
                        matches!(SystemConfiguration::parse_slot(&bytes),
                            Ok(slot) if slot.immutable.filesystem_identifier == chosen.immutable.filesystem_identifier)
                    })
                })
                .count()
        })
        .collect()
}

/// 这个崩溃状态在不在崩溃注入摆得出的域里：崩溃注入「撕裂并进没持久」，只摆整写子集；枚举用的写表里录制流的写在前、
/// 撕裂镜像与重放接在后面，持久了后面那一截的任一个就是撕裂态。
fn domain_of(image: &CrashImage<'_>, recorded_writes: usize) -> &'static str {
    if image.persisted.iter().skip(recorded_writes).any(|persisted| *persisted) {
        "torn"
    } else {
        "whole"
    }
}

/// 在 `pool` 的副本上起一次可写挂载（读不报错）：交回结局与挂载之后的盘。
fn mount_on(pool: &MemoryPool) -> (Result<(), MountError>, MemoryPool) {
    let parameters = SMALL.parameters();
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
    let result =
        mount_writable_with_space_admission(&parameters, &mut devices, SpaceAdmission::JudgedByTheFormula).map(|_mounted| ());
    let after = MemoryPool {
        devices: devices.into_iter().map(|(identity, device)| (identity, device.image)).collect(),
        device_size_in_bytes: pool.device_size_in_bytes,
    };
    (result, after)
}

/// 一张 G4-丙 判出的盘面上起可写挂载：挂上了只记结局；拒了照判官（崩溃态按没注入读故障）判一次，记 G4 那一臂报没报、别的没满足的条件。
/// 交回这张盘面上一共记几条（G4-丙 一条，G4 那一臂报了再加一条）。
fn mount_and_judge(pool: &MemoryPool, effective_root: Option<(InstanceGeneration, CheckpointTxg)>) -> String {
    let (result, after) = mount_on(pool);
    match result {
        Ok(()) => {
            let still = devices_without_a_self_verified_slot_read_as_the_judge_reads(&after).map_or(usize::MAX, |d| d.len());
            format!("remount=mounted dead_after_the_mount={still} records_on_this_image=1")
        }
        Err(error) => {
            let disk = if *pool == after { DiskAfterTheRefusal::ByteForByteUnchanged } else { DiskAfterTheRefusal::Changed };
            let verdict = judge_writable_mount_refusal_of_a_crash_state(
                pool, effective_root, &error, disk,
                ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet, ReadFaultsInTheCrashState::NoneInjected,
            );
            let (g4_arm, others) = match &verdict {
                WritableMountRefusalVerdict::Illegal { conditions_not_met, .. } => {
                    let g4 = conditions_not_met.iter().any(|condition| matches!(condition,
                        RefusalConditionNotMet::SystemConfigurationUnreadableAfterTheReadStageCriterionTrueOnACrashImageWithoutReadFaults));
                    (g4, conditions_not_met.len() - usize::from(g4))
                }
                WritableMountRefusalVerdict::Legal(_) => (false, 0),
                WritableMountRefusalVerdict::ReasonNotOnTheD23List => (false, 0),
            };
            format!(
                "remount=refused reason={} verdict={} g4_arm={} other_conditions={} records_on_this_image={}",
                reason_name(&error),
                match verdict { WritableMountRefusalVerdict::Legal(_) => "legal", WritableMountRefusalVerdict::Illegal { .. } => "illegal", WritableMountRefusalVerdict::ReasonNotOnTheD23List => "not_on_list" },
                if g4_arm { "fires" } else { "silent" },
                others,
                1 + usize::from(g4_arm)
            )
        }
    }
}

/// 二次崩溃那一截要的基线：整写域里某块盘只剩一份本池自证过的系统配置的第一次崩溃状态（照崩溃注入第三截，基线取在整写域）。
type Bases = Mutex<Vec<(MemoryPool, String)>>;

fn observe(image: &CrashImage<'_>, report: &RecoveryReport, recorded_writes: usize, tally: &Tally, bases: Option<&Bases>) {
    let domain = domain_of(image, recorded_writes);
    bump(tally, format!("domain={domain} states"));
    let judge_reading = devices_without_a_self_verified_slot_read_as_the_judge_reads(image);
    let core_reading = devices_without_a_self_verified_slot_read_by_the_core(image);
    if judge_reading.as_ref() != Some(&core_reading) {
        bump(tally, format!("domain={domain} readings_differ judge={judge_reading:?} core={core_reading:?}"));
    }
    // 校验那一路：池级 checker（O2，独立解析器）自己的逐盘数（`system_configuration_redundancy_report`，不分 fsid），与判官读法比。
    let o2 = singlefs_checker::image::system_configuration_redundancy_report(image).devices_without_a_self_verified_slot;
    if judge_reading.as_ref().is_some_and(|judge| *judge != o2) || (judge_reading.is_none() && !o2.is_empty()) {
        bump(tally, format!("domain={domain} o2_reading judge={judge_reading:?} o2={o2:?}"));
    }
    let counts = self_verified_slot_counts(image);
    if let Some(bases) = bases {
        if domain == "whole" && counts.iter().any(|count| *count == 1) && counts.iter().all(|count| *count >= 1) {
            let mut bases = bases.lock().expect("基线锁");
            if bases.len() < SECOND_CRASH_BASES_AT_MOST {
                let pool = materialized_crash_image(image);
                if !bases.iter().any(|(known, _)| *known == pool) {
                    let label = format!("slots_per_device={counts:?}");
                    bases.push((pool, label));
                }
            }
        }
    }
    let Some(dead) = judge_reading else {
        bump(tally, format!("domain={domain} g4_c=no_system_configuration_chosen"));
        return;
    };
    if dead.is_empty() {
        return;
    }
    let recovery = match &report.outcome {
        RecoveryOutcome::NoFile { .. } => "no_file".to_string(),
        RecoveryOutcome::FileRead { .. } => "file_read".to_string(),
        RecoveryOutcome::Failed { failure, .. } => format!("failed_{}", format!("{failure:?}").split(['(', ' ', '{']).next().unwrap_or("")),
    };
    let prefix = format!("domain={domain} g4_c=fires dead={dead:?} recovery={recovery}");
    if dead.len() == image.device_identities().len() || matches!(report.outcome, RecoveryOutcome::Failed { .. }) {
        bump(tally, format!("{prefix} remount=not_tried"));
        return;
    }
    let pool = materialized_crash_image(image);
    bump(tally, format!("{prefix} {}", mount_and_judge(&pool, report.effective_root)));
}

/// 按崩溃点重放的枚举域把一条流的全部崩溃状态走一遍（每段都展开），逐状态 [`observe`]；状态数先算，超过 [`STATE_LIMIT`] 不跑。
/// `count_only` 时只打状态数与段形。
fn enumerate(name: &str, base: &MemoryPool, writes: &[RetainedWrite], segments: &[Vec<usize>], versions: &[PublishedVersion], bases: Option<&Bases>) {
    let count = crash_state_count_with_torn_in_place_overwrites(base, writes, segments, &full_expansion);
    let shape: Vec<String> = segments
        .iter()
        .map(|segment| {
            let kinds: Vec<&str> = segment.iter().map(|w| writes[*w].kind.name()).collect();
            format!("{}:{}", segment.len(), if kinds.iter().all(|k| *k == kinds[0]) { kinds[0] } else { "mixed" })
        })
        .collect();
    println!("name={name} state_count={count} segments={}", shape.join(","));
    if std::env::var_os("SINGLEFS_PROTO_COUNT_ONLY").is_some() {
        return;
    }
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
    let recorded_writes = writes.len();
    let observe_one = |_ordinal: u64, image: &CrashImage<'_>, report: &RecoveryReport, counts: &mut CrashPointReplayObserverCounts| {
        observe(image, report, recorded_writes, &tally, bases);
        observed.fetch_add(1, Ordering::Relaxed);
        counts.add("observed", 1);
    };
    let started = std::time::Instant::now();
    let leaked_name: &'static str = Box::leak(name.to_string().into_boxed_str());
    let outcome = enumerate_crash_states_with_expansion(
        CrashPointReplayItem { name: leaked_name, setting_file: THIS_FILE },
        base,
        writes,
        segments,
        judged_root_index,
        CrashPointReplayVersionTable::with_a_file_only(versions),
        &full_expansion,
        &[CrashPointReplayObserver { name: "g4_c_attack", extra_inputs_digest: digest, observe: &observe_one }],
    );
    let judged = outcome.judged();
    println!(
        "name={name} toggles=[{toggles}] observed={} judged={} seconds={:.1}",
        observed.load(Ordering::Relaxed),
        judged.is_some(),
        started.elapsed().as_secs_f64()
    );
    if let Some(judged) = judged {
        println!("name={name} judged_tally={}", format!("{judged:?}").replace(' ', ""));
    }
    for (key, value) in tally.into_inner().expect("计数锁") {
        println!("name={name} {key} count={value}");
    }
}

/// 第三截那一形：一个第一次崩溃状态的镜像上起一次可写挂载（录下它的写），把这次挂载的全部崩溃状态（二次崩溃）走一遍判 G4-丙。
fn second_crashes(name: &str, base: &MemoryPool, label: &str) {
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
    let outcome = match &mounted { Ok(_) => "mounted".to_string(), Err(error) => format!("refused_{}", reason_name(error)) };
    drop(mounted);
    drop(devices);
    println!("name={name} base=[{label}] first_mount={outcome}");
    let (mount_writes, mount_segments) = writes_and_segments(&mount_stream.retained_operations(), &geometry());
    enumerate(name, base, &mount_writes, &mount_segments, &[], None);
}

/// 第一个文件之后再跑 `operations` 那一截的流：基线是第一个文件之后的盘面。`second` 时另取整写域里某块盘只剩一槽的第一次崩溃状态当基线，
/// 每个基线上走一遍 [`second_crashes`]。
fn after_first_file(name: &str, operations: Vec<HistoryOperation>, second: bool) {
    let (prefix, _, _) = recorded(HistoryStartingPoint::AfterFirstFile, Vec::new());
    let (stream, versions, ending) = recorded(HistoryStartingPoint::AfterFirstFile, operations);
    assert_eq!(stream[..prefix.len()], prefix[..], "同一个起点跑出同一段录制流");
    println!("name={name} history={ending}");
    let base = pool_of(&prefix);
    let (writes, segments) = writes_and_segments(&stream[prefix.len()..], &geometry());
    let bases = Bases::default();
    enumerate(name, &base, &writes, &segments, &versions, if second { Some(&bases) } else { None });
    for (index, (pool, label)) in bases.into_inner().expect("基线锁").into_iter().enumerate() {
        second_crashes(&format!("{name}_second_crash_base_{index}"), &pool, &label);
    }
}

/// 第三截的基线（崩溃注入第三截的第一次崩溃都在整写域里）：`remount` 那段历史的写流里，每一个系统配置槽写的段（取号两轮、两次轮换）
/// 摆「这一段只落了一块盘的那一写」两种（落盘 0 那一写 / 落盘 1 那一写），共 8 个；第 `base_index` 个上走一遍 [`second_crashes`]。
/// 基线是直接按写表叠出来的整写子集（选起点，不是判的崩溃状态）；判的二次崩溃状态照枚举域一个不落。
fn second_crash_on_a_base_of_the_remount_history(base_index: usize) {
    let (prefix, _, _) = recorded(HistoryStartingPoint::AfterFirstFile, Vec::new());
    let (stream, _, _) = recorded(HistoryStartingPoint::AfterFirstFile, vec![HistoryOperation::CloseAndMountWritable]);
    let (writes, segments) = writes_and_segments(&stream[prefix.len()..], &geometry());
    let mut candidates: Vec<(usize, usize)> = Vec::new();
    for (segment_index, segment) in segments.iter().enumerate() {
        if segment.len() == 2 && segment.iter().all(|w| writes[*w].kind == StepKind::SystemConfigurationSlot) {
            candidates.push((segment_index, 0));
            candidates.push((segment_index, 1));
        }
    }
    println!("name=second_crash_base candidates={}", candidates.len());
    let (segment_index, landed) = candidates[base_index];
    let mut base = pool_of(&prefix);
    let earlier: Vec<RetainedWrite> = segments[..segment_index].iter().flatten().map(|w| writes[*w].clone()).collect();
    base.apply_writes(&earlier);
    base.apply_writes(std::slice::from_ref(&writes[segments[segment_index][landed]]));
    let label = format!(
        "segment={segment_index} landed_device={} slots_per_device={:?}",
        writes[segments[segment_index][landed]].device.0,
        self_verified_slot_counts(&base)
    );
    second_crashes(&format!("second_crash_base_{base_index}"), &base, &label);
}

/// 甲-二 那一格：红的那条 checker 档测试同形的盘面（第一个文件之后，盘 0 两槽各在偏移 2048 翻一个字节），`unit_too` 时另按甲把盘 0 上
/// 第一个文件那次发布最后一个单元写翻一个字节（补写闸一不过）。起一次可写挂载、拒了就判一次；另打「挂上了」这一句（与 harness 档
/// `a_device_holding_the_version_whose_both_slots_were_zeroed_is_rewritten_by_the_acquisition_and_the_checker_stays_green` 断言同向）成不成立。
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
    let dead = devices_without_a_self_verified_slot_read_as_the_judge_reads(&pool);
    let line = mount_and_judge(&pool, fields.report.effective_root);
    let mounted_assertion = if line.starts_with("remount=mounted") { "holds" } else { "fails" };
    println!(
        "name={name} gate_never_passes={} g4_c_dead={dead:?} {line} assertion_mount_succeeds={mounted_assertion}",
        std::env::var_os("SINGLEFS_PROTO_GATE_NEVER_PASSES").is_some()
    );
}

fn main() {
    let flow = std::env::args().nth(1).expect("要给流名");
    match flow.as_str() {
        "remount" => after_first_file("remount", vec![HistoryOperation::CloseAndMountWritable], false),
        "remount_with_second_crashes" => {
            after_first_file("remount_with_second_crashes", vec![HistoryOperation::CloseAndMountWritable], true);
        }
        "remount_then_raise_floor" => after_first_file(
            "remount_then_raise_floor",
            vec![
                HistoryOperation::CloseAndMountWritable,
                HistoryOperation::RaiseRollbackFloor(FloorTargetChoice { steps_above_current_floor: 1 }),
            ],
            false,
        ),
        "remount_then_roll_back" => after_first_file(
            "remount_then_roll_back",
            vec![
                HistoryOperation::CloseAndMountWritable,
                HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRoot { index_from_newest: 1 }),
            ],
            false,
        ),
        "raise_floor_steps" => {
            let steps: u64 = std::env::args().nth(2).expect("要给步数").parse().expect("步数");
            after_first_file(
                &format!("raise_floor_steps_{steps}"),
                vec![HistoryOperation::RaiseRollbackFloor(FloorTargetChoice { steps_above_current_floor: steps })],
                false,
            );
        }
        "second_crash_base" => {
            let base_index: usize = std::env::args().nth(2).expect("要给基线号").parse().expect("基线号");
            second_crash_on_a_base_of_the_remount_history(base_index);
        }
        "raise_floor" => after_first_file(
            "raise_floor",
            vec![HistoryOperation::RaiseRollbackFloor(FloorTargetChoice { steps_above_current_floor: 1 })],
            false,
        ),
        "roll_back" => after_first_file(
            "roll_back",
            vec![HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRoot { index_from_newest: 1 })],
            false,
        ),
        "roll_back_beyond" => after_first_file(
            "roll_back_beyond",
            vec![HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::BeyondNewestRoot { txg_beyond_newest: 0 })],
            false,
        ),
        "zeros_mount" => after_first_file(
            "zeros_mount",
            vec![HistoryOperation::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros],
            false,
        ),
        "remount_then_zeros_mount" => after_first_file(
            "remount_then_zeros_mount",
            vec![
                HistoryOperation::CloseAndMountWritable,
                HistoryOperation::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros,
            ],
            false,
        ),
        "overwrite_losing_power_then_mount" => after_first_file(
            "overwrite_losing_power_then_mount",
            vec![HistoryOperation::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable(ContentChoice {
                length: ContentLength::Empty,
                fill_seed: 1,
            })],
            false,
        ),
        "remount_twice" => after_first_file(
            "remount_twice",
            vec![HistoryOperation::CloseAndMountWritable, HistoryOperation::CloseAndMountWritable],
            false,
        ),
        "cold_start_then_remount" => after_first_file(
            "cold_start_then_remount",
            vec![HistoryOperation::ColdStartRecover, HistoryOperation::CloseAndMountWritable],
            false,
        ),
        "overwrite" => after_first_file("overwrite", vec![overwrite(1)], false),
        "hand_made_image_both_slots" => hand_made_image("hand_made_image_both_slots", false),
        "hand_made_image_both_slots_and_a_unit" => hand_made_image("hand_made_image_both_slots_and_a_unit", true),
        other => panic!("认不出流名 {other}"),
    }
}

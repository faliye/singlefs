//! m3-formal-code-b16-r1 攻方原型（副本上量，只在攻方模型里用过、被攻过零轮）。
//! 对一段历史：跑执行器录流，按层 0 的枚举域（候选段里每段每个写取它的几态、任意组合、去掉全落那一个；原地覆写取三态）摆崩溃态，
//! 每个态上：只读恢复（R1）取所选那一版，在副本盘上起可写挂载；拒因在 D23 单子上的，调 16-3 新判官
//! `judge_writable_mount_refusal_of_a_crash_state`（读故障按 NoneInjected），数 C670 那一条为真的态（崩溃注入第二截看得到的那一类）。
//! 挂载做成的、只在非撕裂态上：挂载的写流再按段取每个真子集（二次崩溃），在二次崩溃镜像上再起可写挂载，数同一条为真的态
//! （崩溃注入第三截只把「再挂被拒」计数、不判，C670 那一条在这里看不见）。
//! 用法：c670-probe named | random <权重名> <步数> <种子起> <种子止> [--no-second] [--no-torn] [--threads N]
use std::collections::BTreeMap;

use singlefs_checker_tier::crash::{torn_image_of_in_place_overwrite, TearableInPlaceOverwrites};
use singlefs_core::address::DeviceIdentity;
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::MakeFilesystemParameters;
use singlefs_core::mount::{mount_writable_with_space_admission, MountError};
use singlefs_harness::history::{
    execute_history_with, generate_history_with_weights, GeneratedHistory, GenerationWeights,
    HistoryDeviceWidth, HistoryEnding, HistoryExecution, HistoryOperation, HistoryOperationKind,
    HistorySeed, HistoryStartingPoint, PerStepChecker, StepPosition, ContentChoice, ContentLength,
    RollbackTargetChoice, FloorTargetChoice,
};
use singlefs_harness::memory_pool::{
    writes_and_segments_with_stream_indexes, MemoryPool, RetainedWrite, SparseBlockDevice,
};
use singlefs_harness::mount_transparency::{
    judge_writable_mount_refusal_of_a_crash_state, read_recovery_fields,
    refusal_reason_on_the_d23_list, ComparedVersion, DiskAfterTheRefusal,
    ReadFaultsInTheCrashState, ReadOnlyReadBackOfTheCrashImage, RefusalConditionNotMet,
    RefusalReasonOnTheD23List, WritableMountRefusalVerdict,
};
use singlefs_harness::segments::FixedGeometry;
use singlefs_harness::{RecordingBlockDevice, SharedStream};

const WIDTH: HistoryDeviceWidth = HistoryDeviceWidth::UnitAreaOf384Slots;
/// 命名用例那条流里「短段」的上限（与 16-3 那条崩溃注入用例的 segment_writes 相同）。
const SHORT_SEGMENT_WRITES: usize = 4;

#[derive(Default, Clone, Debug)]
struct Counts {
    histories: u64,
    histories_completed: u64,
    first_states: u64,
    first_states_with_a_torn_write: u64,
    first_mounts_succeeded: u64,
    first_refusals_by_member: BTreeMap<String, u64>,
    first_n_config_refusals: u64,
    first_c670_by_branch: BTreeMap<String, u64>,
    first_c670_in_short_segments: u64,
    first_c670_by_step: BTreeMap<String, u64>,
    first_c670_in_torn_states: u64,
    second_states: u64,
    second_n_config_refusals: u64,
    second_c670_by_branch: BTreeMap<String, u64>,
    abandoning_steps: u64,
    abandoning_steps_that_sent_something: u64,
    examples: Vec<String>,
}

impl Counts {
    fn absorb(&mut self, other: Counts) {
        self.histories += other.histories;
        self.histories_completed += other.histories_completed;
        self.first_states += other.first_states;
        self.first_states_with_a_torn_write += other.first_states_with_a_torn_write;
        self.first_mounts_succeeded += other.first_mounts_succeeded;
        for (k, v) in other.first_refusals_by_member {
            *self.first_refusals_by_member.entry(k).or_default() += v;
        }
        self.first_n_config_refusals += other.first_n_config_refusals;
        for (k, v) in other.first_c670_by_branch {
            *self.first_c670_by_branch.entry(k).or_default() += v;
        }
        self.first_c670_in_short_segments += other.first_c670_in_short_segments;
        for (k, v) in other.first_c670_by_step {
            *self.first_c670_by_step.entry(k).or_default() += v;
        }
        self.first_c670_in_torn_states += other.first_c670_in_torn_states;
        self.second_states += other.second_states;
        self.second_n_config_refusals += other.second_n_config_refusals;
        for (k, v) in other.second_c670_by_branch {
            *self.second_c670_by_branch.entry(k).or_default() += v;
        }
        self.abandoning_steps += other.abandoning_steps;
        self.abandoning_steps_that_sent_something += other.abandoning_steps_that_sent_something;
        for e in other.examples {
            if self.examples.len() < 20 {
                self.examples.push(e);
            }
        }
    }
}

struct Options {
    second: bool,
    torn: bool,
    /// 只展开写数不超过它的候选段（命名用例自己的短段域；None 是层 0 全域）。
    segments_up_to: Option<usize>,
    budget: u64,
    /// 二次崩溃只展开挂载写流里写数不超过它的段（取号、轮换、记录、根那几段；单元写那几段不展开）。
    second_segments_up_to: usize,
    /// 只做全局序号 mod n == k 的首次崩溃态（主 agent 并行分片跑全域用）。
    shard: (u64, u64),
}

fn geometry_of(parameters: &MakeFilesystemParameters) -> FixedGeometry {
    FixedGeometry {
        fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
        journal_ring_bytes: parameters.geometry.journal_ring_bytes,
        root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
    }
}

/// 在镜像的副本上起可写挂载，录下挂载发的流。
fn mount_on(
    image: &MemoryPool,
    parameters: &MakeFilesystemParameters,
) -> (Result<(), MountError>, Vec<singlefs_harness::RetainedOperation>) {
    let stream = SharedStream::retaining_contents();
    let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = image
        .devices
        .iter()
        .map(|(identity, sectors)| {
            let mut device = SparseBlockDevice::new(
                image.device_size_in_bytes,
                PhysicalBlockSizeInBytes(parameters.geometry.physical_block_size),
            );
            device.image = sectors.clone();
            (
                *identity,
                RecordingBlockDevice::with_shared_stream(*identity, device, stream.clone()),
            )
        })
        .collect();
    let result = mount_writable_with_space_admission(
        parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
    )
    .map(|_| ());
    (result, stream.retained_operations())
}

fn member_of(error: &MountError) -> String {
    let text = format!("{error:?}");
    let cut = text
        .find(|c: char| c == '(' || c == ' ' || c == '{')
        .unwrap_or(text.len());
    let mut member = text[..cut].to_string();
    if let MountError::NewerStateStillUnreadableAfterOneReread(inner) = error {
        let inner_text = format!("{inner:?}");
        let inner_cut = inner_text
            .find(|c: char| c == '(' || c == ' ' || c == '{')
            .unwrap_or(inner_text.len());
        member.push_str("::");
        member.push_str(&inner_text[..inner_cut]);
        if let Some(at) = inner_text.find("comparison: ") {
            let rest = &inner_text[at + 12..];
            let end = rest
                .find(|c: char| c == '(' || c == ' ' || c == '{' || c == ',')
                .unwrap_or(rest.len());
            member.push_str("::");
            member.push_str(&rest[..end]);
        }
    }
    member
}

/// C670 那一条：拒因是判据 N-配置、新判官（没注入读故障）交回的不满足条件里有见证判据为真那一格，交回那一支。
fn c670_branch(image: &MemoryPool, error: &MountError) -> Option<String> {
    if refusal_reason_on_the_d23_list(error)
        != Some(RefusalReasonOnTheD23List::NewerPublishWitnessedBySystemConfiguration)
    {
        return None;
    }
    let fields = read_recovery_fields(image, ComparedVersion::TheRecoveredVersion);
    match judge_writable_mount_refusal_of_a_crash_state(
        image,
        fields.report.effective_root,
        error,
        DiskAfterTheRefusal::ByteForByteUnchanged,
        ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet,
        ReadFaultsInTheCrashState::NoneInjected,
    ) {
        WritableMountRefusalVerdict::Illegal {
            conditions_not_met, ..
        } => conditions_not_met.iter().find_map(|condition| match condition {
            RefusalConditionNotMet::WitnessCriterionTrueOnACrashImageWithoutReadFaults(branch) => {
                Some(format!("{branch:?}"))
            }
            _ => None,
        }),
        _ => None,
    }
}

/// 段内每个写各取几态的数字串（0 没落、1 撕裂、2 落），去掉全落那一个。
fn landings_of(segment: &[usize], tearable: &TearableInPlaceOverwrites, torn: bool) -> Vec<Vec<u8>> {
    let radix: Vec<u8> = segment
        .iter()
        .map(|w| if torn && tearable.contains(*w) { 3 } else { 2 })
        .collect();
    let mut out = Vec::new();
    let mut digits = vec![0u8; segment.len()];
    loop {
        let all_landed = digits.iter().zip(&radix).all(|(d, r)| *d == r - 1);
        if !all_landed {
            out.push(digits.clone());
        }
        let mut position = 0;
        loop {
            if position == digits.len() {
                return out;
            }
            digits[position] += 1;
            if digits[position] < radix[position] {
                break;
            }
            digits[position] = 0;
            position += 1;
        }
    }
}

fn digit_to_state(radix: u8, digit: u8) -> u8 {
    // 两态的写：0 没落、1 落；三态：0 没落、1 撕裂、2 落。统一成 0 / 1 / 2。
    if radix == 2 && digit == 1 { 2 } else { digit }
}

fn apply_landing(image: &mut MemoryPool, write: &RetainedWrite, state: u8) {
    match state {
        0 => {}
        2 => image.apply_writes(std::slice::from_ref(write)),
        _ => {
            let length = usize::try_from(write.length_in_bytes()).expect("写长");
            let device = image.devices.get_mut(&write.device).expect("有这块盘");
            let old = device.read(write.offset, length);
            let torn = torn_image_of_in_place_overwrite(&old, &write.contents);
            device.write(write.offset, &torn);
        }
    }
}

fn state_count(segments: &[Vec<usize>], candidates: &[usize], tearable: &TearableInPlaceOverwrites, torn: bool) -> u64 {
    candidates
        .iter()
        .map(|s| {
            segments[*s]
                .iter()
                .map(|w| if torn && tearable.contains(*w) { 3u64 } else { 2 })
                .product::<u64>()
                - 1
        })
        .sum()
}

fn run_history(history: &GeneratedHistory, options: &Options, counts: &mut Counts) {
    let parameters = WIDTH.parameters();
    let geometry = geometry_of(&parameters);
    let stream = SharedStream::retaining_contents();
    let mut after_each_step: Vec<(StepPosition, usize)> = Vec::new();
    let run = execute_history_with(
        history,
        HistoryExecution {
            per_step_checker: PerStepChecker::Skipped,
            device_width: WIDTH,
            space_admission: SpaceAdmission::JudgedByTheFormula,
        },
        &stream,
        &mut |observation| after_each_step.push((observation.position, stream.operation_count())),
    );
    counts.histories += 1;
    if matches!(run.ending, HistoryEnding::Completed) {
        counts.histories_completed += 1;
    }
    let after_mkfs = run.operations_written_by_make_filesystem;
    {
        let mut before = 0usize;
        for (position, after) in &after_each_step {
            if let StepPosition::Operation(i) = position {
                if history.operations[*i].kind() == HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot {
                    counts.abandoning_steps += 1;
                    if *after > before {
                        counts.abandoning_steps_that_sent_something += 1;
                    }
                }
            }
            before = *after;
        }
    }
    let after_last = after_each_step.last().map_or(0, |(_, n)| *n);
    let operations = stream.retained_operations();
    let (writes, segments, stream_indexes) =
        writes_and_segments_with_stream_indexes(&operations, &geometry);
    let candidates: Vec<usize> = (0..segments.len())
        .filter(|s| segments[*s].iter().all(|w| (after_mkfs..after_last).contains(&stream_indexes[*w])))
        .filter(|s| options.segments_up_to.is_none_or(|up_to| segments[*s].len() <= up_to))
        .collect();
    let empty = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], WIDTH.device_bytes());
    let tearable = TearableInPlaceOverwrites::of(&empty, &writes);
    let planned = state_count(&segments, &candidates, &tearable, options.torn);
    if std::env::var_os("C670_PROBE_PLAN").is_some() {
        let lengths: Vec<(usize, usize, usize)> = candidates
            .iter()
            .map(|s| (*s, segments[*s].len(), segments[*s].iter().filter(|w| tearable.contains(**w)).count()))
            .collect();
        eprintln!(
            "plan seed={} start={:?} ops={:?} states_torn={} states_untorn={} segments(index,writes,tearable)={lengths:?}",
            history.seed.0,
            history.starting_point,
            history.operations.iter().map(HistoryOperation::kind).collect::<Vec<_>>(),
            state_count(&segments, &candidates, &tearable, true),
            state_count(&segments, &candidates, &tearable, false)
        );
        return;
    }
    if planned > options.budget {
        eprintln!("种子 {:?}：首次崩溃态 {planned} 个超过一次的上限，跳过", history.seed);
        return;
    }
    let step_kinds: Vec<HistoryOperationKind> = history.operations.iter().map(HistoryOperation::kind).collect();
    let step_name = |stream_index: usize| -> String {
        match after_each_step.iter().find(|(_, n)| *n > stream_index).map(|(p, _)| *p) {
            Some(StepPosition::StartingPoint) => "StartingPoint".to_string(),
            Some(StepPosition::Operation(i)) => format!("{:?}", step_kinds[i]),
            None => "after".to_string(),
        }
    };
    let mut base = empty.clone();
    let mut applied = 0usize;
    let mut ordinal: u64 = 0;
    for segment_index in candidates {
        let segment = &segments[segment_index];
        base.apply_writes(&writes[applied..segment[0]]);
        applied = segment[0];
        let radix: Vec<u8> = segment
            .iter()
            .map(|w| if options.torn && tearable.contains(*w) { 3 } else { 2 })
            .collect();
        for digits in landings_of(segment, &tearable, options.torn) {
            ordinal += 1;
            if (ordinal - 1) % options.shard.1 != options.shard.0 {
                continue;
            }
            let mut image = base.clone();
            let mut has_torn = false;
            let mut first_withheld: Option<usize> = None;
            for (k, w) in segment.iter().enumerate() {
                let state = digit_to_state(radix[k], digits[k]);
                if state == 1 {
                    has_torn = true;
                }
                if state != 2 && first_withheld.is_none() {
                    first_withheld = Some(*w);
                }
                apply_landing(&mut image, &writes[*w], state);
            }
            counts.first_states += 1;
            if has_torn {
                counts.first_states_with_a_torn_write += 1;
            }
            let (mounted, mount_operations) = mount_on(&image, &parameters);
            match mounted {
                Err(error) => {
                    *counts.first_refusals_by_member.entry(member_of(&error)).or_default() += 1;
                    if refusal_reason_on_the_d23_list(&error)
                        == Some(RefusalReasonOnTheD23List::NewerPublishWitnessedBySystemConfiguration)
                    {
                        counts.first_n_config_refusals += 1;
                    }
                    if let Some(branch) = c670_branch(&image, &error) {
                        *counts.first_c670_by_branch.entry(branch.clone()).or_default() += 1;
                        if segment.len() <= SHORT_SEGMENT_WRITES {
                            counts.first_c670_in_short_segments += 1;
                        }
                        if has_torn {
                            counts.first_c670_in_torn_states += 1;
                        }
                        let step = step_name(stream_indexes[first_withheld.expect("真子集")]);
                        *counts.first_c670_by_step.entry(step.clone()).or_default() += 1;
                        if counts.examples.iter().filter(|e| e.starts_with("first")).count() < 6 {
                            let kinds: Vec<String> = segment
                                .iter()
                                .zip(&digits)
                                .map(|(w, d)| format!("{:?}@d{}:{}", writes[*w].kind, writes[*w].device.0, d))
                                .collect();
                            counts.examples.push(format!(
                                "first seed={} seg={segment_index} step={step} branch={branch} landing={kinds:?} member={}",
                                history.seed.0,
                                member_of(&error)
                            ));
                        }
                    }
                }
                Ok(()) => {
                    counts.first_mounts_succeeded += 1;
                    if options.second && !has_torn {
                        second_crashes(&image, &mount_operations, &parameters, &geometry, history.seed.0, options.second_segments_up_to, counts);
                    }
                }
            }
        }
    }
}

fn second_crashes(
    crash_image: &MemoryPool,
    mount_operations: &[singlefs_harness::RetainedOperation],
    parameters: &MakeFilesystemParameters,
    geometry: &FixedGeometry,
    seed: u64,
    second_segments_up_to: usize,
    counts: &mut Counts,
) {
    let (writes, segments, _) = writes_and_segments_with_stream_indexes(mount_operations, geometry);
    let none = TearableInPlaceOverwrites::none(writes.len());
    let mut base = crash_image.clone();
    let mut applied = 0usize;
    for (segment_index, segment) in segments.iter().enumerate() {
        base.apply_writes(&writes[applied..segment[0]]);
        applied = segment[0];
        if segment.len() > second_segments_up_to {
            continue;
        }
        for digits in landings_of(segment, &none, false) {
            let mut image = base.clone();
            for (k, w) in segment.iter().enumerate() {
                apply_landing(&mut image, &writes[*w], digit_to_state(2, digits[k]));
            }
            counts.second_states += 1;
            let (mounted, _) = mount_on(&image, parameters);
            if let Err(error) = mounted {
                if refusal_reason_on_the_d23_list(&error)
                    == Some(RefusalReasonOnTheD23List::NewerPublishWitnessedBySystemConfiguration)
                {
                    counts.second_n_config_refusals += 1;
                }
                if let Some(branch) = c670_branch(&image, &error) {
                    *counts.second_c670_by_branch.entry(branch.clone()).or_default() += 1;
                    if counts.examples.iter().filter(|e| e.starts_with("second")).count() < 4 {
                        let kinds: Vec<String> = segment
                            .iter()
                            .zip(&digits)
                            .map(|(w, d)| format!("{:?}@d{}:{}", writes[*w].kind, writes[*w].device.0, d))
                            .collect();
                        counts.examples.push(format!(
                            "second seed={seed} mount-seg={segment_index} branch={branch} landing={kinds:?} member={}",
                            member_of(&error)
                        ));
                    }
                }
            }
        }
    }
}

fn weights_named(name: &str) -> GenerationWeights {
    match name {
        "broad" => GenerationWeights::BROAD,
        "reuse" => GenerationWeights::REUSE_AFTER_RAISING_THE_FLOOR,
        "rollback" => GenerationWeights::ROLLBACK_AFTER_RAISING_THE_FLOOR,
        "allocwall" => GenerationWeights::TOWARD_THE_ALLOCATION_RECORD_WALL,
        "unitwall" => GenerationWeights::TOWARD_THE_UNIT_AREA_WALL,
        "deleting" => GenerationWeights::DELETING_FILES_OF_SEVERAL_LOWER_LEAVES,
        other => panic!("不认得的权重名 {other}"),
    }
}

fn print(counts: &Counts, label: &str) {
    println!("== {label}");
    println!("histories={} completed={}", counts.histories, counts.histories_completed);
    println!(
        "first_states={} with_torn_write={} mounts_succeeded={} n_config_refusals={}",
        counts.first_states, counts.first_states_with_a_torn_write, counts.first_mounts_succeeded, counts.first_n_config_refusals
    );
    for (k, v) in &counts.first_refusals_by_member {
        println!("first_refusal {k} {v}");
    }
    let c670: u64 = counts.first_c670_by_branch.values().sum();
    println!(
        "first_c670={c670} in_short_segments(<={SHORT_SEGMENT_WRITES})={} in_torn_states={}",
        counts.first_c670_in_short_segments, counts.first_c670_in_torn_states
    );
    for (k, v) in &counts.first_c670_by_branch {
        println!("first_c670_branch {k} {v}");
    }
    for (k, v) in &counts.first_c670_by_step {
        println!("first_c670_step {k} {v}");
    }
    let second: u64 = counts.second_c670_by_branch.values().sum();
    println!(
        "second_states={} second_n_config_refusals={} second_c670={second}",
        counts.second_states, counts.second_n_config_refusals
    );
    for (k, v) in &counts.second_c670_by_branch {
        println!("second_c670_branch {k} {v}");
    }
    println!(
        "abandoning_steps={} abandoning_steps_that_sent_something={}",
        counts.abandoning_steps, counts.abandoning_steps_that_sent_something
    );
    for e in &counts.examples {
        println!("example {e}");
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let options = Options {
        second: !args.iter().any(|a| a == "--no-second"),
        torn: !args.iter().any(|a| a == "--no-torn"),
        segments_up_to: args
            .iter()
            .position(|a| a == "--segments-up-to")
            .map(|i| args[i + 1].parse().expect("段长")),
        budget: std::env::var("C670_PROBE_BUDGET").map_or(1_100_000, |v| v.parse().expect("上限")),
        second_segments_up_to: args
            .iter()
            .position(|a| a == "--second-segments-up-to")
            .map_or(4, |i| args[i + 1].parse().expect("段长")),
        shard: args.iter().position(|a| a == "--shard").map_or((0, 1), |i| {
            let (k, n) = args[i + 1].split_once('/').expect("k/n");
            (k.parse().expect("k"), n.parse().expect("n"))
        }),
    };
    let started = std::time::Instant::now();
    let threads: usize = args
        .iter()
        .position(|a| a == "--threads")
        .map_or(1, |i| args[i + 1].parse().expect("线程数"));
    match args.get(1).map(String::as_str) {
        Some("named") => {
            // 16-3 那条崩溃注入用例的历史：起点（取号、暖机、第一个文件）加一次关掉重挂。
            let history = GeneratedHistory {
                seed: HistorySeed(0),
                starting_point: HistoryStartingPoint::AfterFirstFile,
                operations: vec![HistoryOperation::CloseAndMountWritable],
            };
            let mut counts = Counts::default();
            run_history(&history, &options, &mut counts);
            print(&counts, "named AfterFirstFile+CloseAndMountWritable");
            println!("elapsed_seconds={:.1}", started.elapsed().as_secs_f64());
        }
        Some("ops") => {
            // 写死的一段历史：ops <AfterFirstFile|AfterMakeFilesystem> <逗号分隔的操作名>
            let starting_point = match args[2].as_str() {
                "AfterFirstFile" => HistoryStartingPoint::AfterFirstFile,
                "AfterMakeFilesystem" => HistoryStartingPoint::AfterMakeFilesystem,
                other => panic!("起点 {other}"),
            };
            let content = ContentChoice {
                length: ContentLength::InsideOneDataUnit { selector: 0 },
                fill_seed: 1,
            };
            let operations: Vec<HistoryOperation> = args[3]
                .split(',')
                .filter(|name| !name.is_empty())
                .map(|name| match name {
                    "PublishFirstFile" => HistoryOperation::PublishFirstFile(content),
                    "PublishOverwrite" => HistoryOperation::PublishOverwrite(content),
                    "PublishWithoutUnits" => HistoryOperation::PublishWithoutUnits,
                    "CloseAndMountWritable" => HistoryOperation::CloseAndMountWritable,
                    "RollBackWhileMounted" => HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRoot { index_from_newest: 1 }),
                    "RaiseRollbackFloor" => HistoryOperation::RaiseRollbackFloor(FloorTargetChoice { steps_above_current_floor: 1 }),
                    "ColdStartRecover" => HistoryOperation::ColdStartRecover,
                    "CrashRecoveryAbandoningTheNewestRoot" => HistoryOperation::CrashRecoveryAbandoningTheNewestRoot,
                    "Unlink" => HistoryOperation::Unlink,
                    "CreateInode" => HistoryOperation::CreateInode,
                    other => panic!("操作 {other}"),
                })
                .collect();
            let history = GeneratedHistory { seed: HistorySeed(0), starting_point, operations };
            let mut counts = Counts::default();
            run_history(&history, &options, &mut counts);
            print(&counts, &format!("ops {} {}", args[2], args[3]));
            println!("elapsed_seconds={:.1}", started.elapsed().as_secs_f64());
        }
        Some("random") => {
            let weights_name = args[2].clone();
            let steps: usize = args[3].parse().expect("步数");
            let from: u64 = args[4].parse().expect("种子起");
            let to: u64 = args[5].parse().expect("种子止");
            let seeds: Vec<u64> = (from..to).collect();
            let chunk = seeds.len().div_ceil(threads.max(1)).max(1);
            let mut total = Counts::default();
            std::thread::scope(|scope| {
                let handles: Vec<_> = seeds
                    .chunks(chunk)
                    .map(|part| {
                        let part = part.to_vec();
                        let weights_name = weights_name.clone();
                        let options = &options;
                        scope.spawn(move || {
                            let weights = weights_named(&weights_name);
                            let mut counts = Counts::default();
                            for seed in part {
                                let history = generate_history_with_weights(HistorySeed(seed), steps, &weights);
                                run_history(&history, options, &mut counts);
                            }
                            counts
                        })
                    })
                    .collect();
                for handle in handles {
                    total.absorb(handle.join().expect("线程"));
                }
            });
            print(&total, &format!("random {weights_name} steps={steps} seeds={from}..{to}"));
            println!("elapsed_seconds={:.1}", started.elapsed().as_secs_f64());
        }
        _ => panic!("用法：c670-probe named | random <权重名> <步数> <种子起> <种子止> [--no-second] [--no-torn] [--threads N]"),
    }
}

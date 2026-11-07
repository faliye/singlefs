//! 逐写独立判第一轮攻方腿的原型（报告 `research/prompts/per-write-judging-r1-opus-output.md`）。
//!
//! 在两块内存稀疏盘上自造一条短流：mkfs → 取号 → 暖机 → 新池新建文件（A）→ 覆盖写（C1）→ 覆盖写（C2），文件一直在。
//! 三条臂各改一处写表次序（改的是录下来的写表，等于一份把这一处写晚了或写早了的实现）：
//! - `unmodified`：原样；
//! - `late_unit`：C1 的一个被引用单元的两份副本挪进 C2 的单元段（单元晚于引用它的根，「前缀里的根只指向已经落了的单元」不成立）；
//! - `record_before_units`：C2 的记录段挪到 C2 的单元段之前（记录早于它点名的单元）。
//! 每条臂上每个只有单元写的段，把段内全部真子集（2^n − 1 个，含空集）逐个判（`judge_crash_image`：两遍恢复、两份 oracle、
//! 池级 checker、记录核对器），与逐写判的四种代表法逐状态比：
//! - `union`：空集的红项 ∪ 各单写的红项；
//! - `delta`：空集的红项，加上各单写比空集多出的项、减去各单写比空集少掉的项；
//! - `group_delta`：同上，但按单元副本组（同偏移、不同盘的几份）取组内子集的判定；
//! - `baseline_if_singles_equal`：每个单写整份判定都与空集相同时，整段都取空集那一份（不相同时这一行只报不比）。
//! 只读、不写任何文件、不碰判定库；行格式见各 `println!`。退出码：0 跑完；2 流的段形状与预想不同；3 估时超过 40 分钟或状态数超过上限，没跑。

use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

use singlefs_checker::image::InvariantVerdict;
use singlefs_core::address::{DeviceIdentity, InstanceGeneration};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::{allocator_after_make_filesystem, make_filesystem};
use singlefs_core::recovery::{PoolReader, RecoveryOutcome, RecoveryReport};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_overwrite, warm_up, FirstFile, PoolWriter,
    TransactionOutput,
};
use singlefs_format::TEST_IMAGE_DEFAULT_BYTES;
use singlefs_verification::crash::{
    crash_state_count_with_torn_in_place_overwrites, judge_crash_image,
    CrashPointReplaySegmentExpansion, CrashStateJudgement,
};
use singlefs_verification::crash_state_enumeration::crash_point_replay_worker_threads;
use singlefs_verification::memory_pool::{
    writes_and_segments, CrashImage, MemoryPool, PublishedVersion, RetainedWrite, SparseBlockDevice,
};
use singlefs_verification::scenario::{
    e142_parameters, first_file_content, FIXED_WRITE_TIME_SECONDS,
};
use singlefs_verification::segments::{FixedGeometry, StepKind};
use singlefs_verification::{RecordingBlockDevice, SharedStream};

/// 一段写数超过它就不展开这一段（2^19 − 1 = 524,287 态）。
const LARGEST_EXPANDED_SEGMENT_WRITES: usize = 19;
/// 一次跑的状态数上限（三条臂合起来）。
const STATE_CEILING_OF_ONE_RUN: u64 = 1_000_000;
/// 估出来的挂钟超过它就不跑（秒）。
const WALL_CLOCK_CEILING_SECONDS: f64 = 2400.0;
/// 每条臂每种代表法打几个对不上的例子。
const EXAMPLES_PER_REPRESENTATION: usize = 3;

/// 录好的流：mkfs 之后的盘（基线）、之后的全部写与切好的段、三版带文件的内容。
struct RecordedFlow {
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    versions: Vec<PublishedVersion>,
}

fn content_of_the_overwrite(seed: usize) -> Vec<u8> {
    (0..3000)
        .map(|index| u8::try_from((index * 7 + seed) % 251).expect("小于 256"))
        .collect()
}

fn version_of(output: &TransactionOutput, content: &[u8]) -> PublishedVersion {
    PublishedVersion {
        instance: output.root.instance,
        checkpoint_txg: output.root.checkpoint_txg,
        content: content.to_vec(),
    }
}

fn record_the_flow() -> RecordedFlow {
    let parameters = e142_parameters(512, 512);
    let device_bytes = TEST_IMAGE_DEFAULT_BYTES;
    let mut plain_devices: Vec<(DeviceIdentity, SparseBlockDevice)> = (0..2u32)
        .map(|device| {
            (
                DeviceIdentity(device),
                SparseBlockDevice::new(device_bytes, PhysicalBlockSizeInBytes(512)),
            )
        })
        .collect();
    let genesis = make_filesystem(&parameters, &mut plain_devices)
        .unwrap_or_else(|error| panic!("mkfs 做不成：{error:?}"));
    let mut allocator = allocator_after_make_filesystem(&parameters, &plain_devices, &genesis);
    let base = MemoryPool {
        devices: plain_devices
            .iter()
            .map(|(identity, device)| (*identity, device.image.clone()))
            .collect(),
        device_size_in_bytes: device_bytes,
    };
    let stream = SharedStream::retaining_contents();
    let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = plain_devices
        .into_iter()
        .map(|(identity, device)| {
            (
                identity,
                RecordingBlockDevice::with_shared_stream(identity, device, stream.clone()),
            )
        })
        .collect();
    let first_content = first_file_content();
    let first_overwrite_content = content_of_the_overwrite(1);
    let second_overwrite_content = content_of_the_overwrite(2);
    let instance = InstanceGeneration(1);
    let first_file = {
        let mut pool = PoolWriter::new(&parameters, &mut devices);
        let acquired =
            acquire_instance(&mut pool).unwrap_or_else(|error| panic!("取号：{error:?}"));
        assert_eq!(acquired, instance, "新池第一次取号取到实例 1");
        let warm = warm_up(&mut pool, &genesis.root, instance)
            .unwrap_or_else(|failed| panic!("暖机：{:?}", failed.cause));
        publish_first_file(
            &mut pool,
            &mut allocator,
            warm.roots.last().expect("暖机两代根"),
            FirstFile {
                content: &first_content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            instance,
            &warm.last_record_bytes,
        )
        .unwrap_or_else(|error| panic!("新池新建文件：{error:?}"))
    };
    let first_overwrite = publish_overwrite(
        &mut PoolWriter::new(&parameters, &mut devices),
        &mut allocator,
        &first_file,
        FirstFile {
            content: &first_overwrite_content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
        },
        instance,
    )
    .unwrap_or_else(|error| panic!("覆盖写 C1：{error:?}"));
    let second_overwrite = publish_overwrite(
        &mut PoolWriter::new(&parameters, &mut devices),
        &mut allocator,
        &first_overwrite,
        FirstFile {
            content: &second_overwrite_content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 120,
        },
        instance,
    )
    .unwrap_or_else(|error| panic!("覆盖写 C2：{error:?}"));
    let geometry = FixedGeometry {
        fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
        journal_ring_bytes: parameters.geometry.journal_ring_bytes,
        root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
    };
    let (writes, segments) = writes_and_segments(&stream.retained_operations(), &geometry);
    RecordedFlow {
        base,
        writes,
        segments,
        versions: vec![
            version_of(&first_file, &first_content),
            version_of(&first_overwrite, &first_overwrite_content),
            version_of(&second_overwrite, &second_overwrite_content),
        ],
    }
}

/// 一条臂：重排之后的写表与段（段按时间先后），挪了什么，被判的那次根槽写（C2 的根）在新写表里的下标。
struct Arm {
    name: &'static str,
    moved: String,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    judged_root_index: usize,
}

fn every_write_is(writes: &[RetainedWrite], segment: &[usize], kind: StepKind) -> bool {
    !segment.is_empty() && segment.iter().all(|index| writes[*index].kind == kind)
}

fn kinds_of(writes: &[RetainedWrite], segment: &[usize]) -> String {
    let kinds: BTreeSet<StepKind> = segment.iter().map(|index| writes[*index].kind).collect();
    let names: Vec<String> = kinds.iter().map(|kind| format!("{kind:?}")).collect();
    names.join("+")
}

fn describe_write(write: &RetainedWrite) -> String {
    format!(
        "{:?}@dev{}+{}x{}",
        write.kind,
        write.device.0,
        write.offset.0,
        write.length_in_bytes()
    )
}

/// 段号：C1 的单元段、C2 的单元段、C2 的记录段。只有单元写的段里最后两段是 C1、C2 的；C2 的记录段是 C2 单元段之后第一段只有记录写的。
struct LandmarkSegments {
    first_overwrite_units: usize,
    second_overwrite_units: usize,
    second_overwrite_records: usize,
}

fn landmark_segments(flow: &RecordedFlow) -> Result<LandmarkSegments, String> {
    let unit_segments: Vec<usize> = (0..flow.segments.len())
        .filter(|segment| {
            every_write_is(&flow.writes, &flow.segments[*segment], StepKind::UnitWrite)
        })
        .collect();
    if unit_segments.len() < 2 {
        return Err(format!("只有单元写的段不到两段：{unit_segments:?}"));
    }
    let first_overwrite_units = unit_segments[unit_segments.len() - 2];
    let second_overwrite_units = unit_segments[unit_segments.len() - 1];
    let root_between = (first_overwrite_units + 1..second_overwrite_units).any(|segment| {
        flow.segments[segment]
            .iter()
            .any(|index| flow.writes[*index].kind == StepKind::RootRecordFua)
    });
    if !root_between {
        return Err("C1 与 C2 的单元段之间没有根槽写".to_string());
    }
    let second_overwrite_records = (second_overwrite_units + 1..flow.segments.len())
        .find(|segment| {
            every_write_is(
                &flow.writes,
                &flow.segments[*segment],
                StepKind::JournalRecord,
            )
        })
        .ok_or_else(|| "C2 的单元段之后没有只有记录写的段".to_string())?;
    if second_overwrite_records != second_overwrite_units + 1 {
        return Err(format!(
            "C2 的记录段不紧跟它的单元段（单元段 {second_overwrite_units}、记录段 {second_overwrite_records}）"
        ));
    }
    Ok(LandmarkSegments {
        first_overwrite_units,
        second_overwrite_units,
        second_overwrite_records,
    })
}

/// 按新的段表（元素是旧写表的下标，段按时间先后）重排写表：新写表就是各段依次连起来，交回新写表与按新下标写的段表。
fn rearranged(
    flow: &RecordedFlow,
    segments_by_old_index: &[Vec<usize>],
) -> (Vec<RetainedWrite>, Vec<Vec<usize>>) {
    let mut writes = Vec::with_capacity(flow.writes.len());
    let mut segments = Vec::with_capacity(segments_by_old_index.len());
    for segment in segments_by_old_index {
        let mut new_segment = Vec::with_capacity(segment.len());
        for old_index in segment {
            new_segment.push(writes.len());
            writes.push(flow.writes[*old_index].clone());
        }
        segments.push(new_segment);
    }
    assert_eq!(
        writes.len(),
        flow.writes.len(),
        "重排前后写数相同：段表盖住整张写表"
    );
    (writes, segments)
}

fn last_root_write(writes: &[RetainedWrite]) -> usize {
    (0..writes.len())
        .rev()
        .find(|index| writes[*index].kind == StepKind::RootRecordFua)
        .expect("写表里有根槽写")
}

fn unmodified_arm(flow: &RecordedFlow) -> Arm {
    let (writes, segments) = rearranged(flow, &flow.segments);
    let judged_root_index = last_root_write(&writes);
    Arm {
        name: "unmodified",
        moved: "无".to_string(),
        writes,
        segments,
        judged_root_index,
    }
}

/// C1 单元段里一组两份副本（同偏移、不同盘），优先取数据单元（32768 字节）。
/// C1 单元段里成对的副本（同偏移、不同盘），数据单元（32768 字节）排前，取前 `count` 对；不够 `count` 对时 None。
fn copy_pairs_in(flow: &RecordedFlow, segment: &[usize], count: usize) -> Option<Vec<[usize; 2]>> {
    let mut by_offset: BTreeMap<u64, Vec<usize>> = BTreeMap::new();
    for index in segment {
        by_offset
            .entry(flow.writes[*index].offset.0)
            .or_default()
            .push(*index);
    }
    let mut pairs: Vec<[usize; 2]> = by_offset
        .values()
        .filter(|copies| {
            copies.len() == 2 && flow.writes[copies[0]].device != flow.writes[copies[1]].device
        })
        .map(|copies| [copies[0], copies[1]])
        .collect();
    pairs.sort_by_key(|pair| flow.writes[pair[0]].length_in_bytes() != 32768);
    (pairs.len() >= count).then(|| pairs[..count].to_vec())
}

/// C1 的 `pair_count` 对副本挪进 C2 单元段开头：单元晚于引用它的根。
fn late_units_arm(
    flow: &RecordedFlow,
    landmarks: &LandmarkSegments,
    name: &'static str,
    pair_count: usize,
) -> Option<Arm> {
    let pairs = copy_pairs_in(
        flow,
        &flow.segments[landmarks.first_overwrite_units],
        pair_count,
    )?;
    let moved_writes: Vec<usize> = pairs.iter().flatten().copied().collect();
    let mut segments = flow.segments.clone();
    segments[landmarks.first_overwrite_units].retain(|index| !moved_writes.contains(index));
    let mut moved_in: Vec<usize> = moved_writes.clone();
    moved_in.extend(segments[landmarks.second_overwrite_units].iter().copied());
    segments[landmarks.second_overwrite_units] = moved_in;
    let (writes, new_segments) = rearranged(flow, &segments);
    let judged_root_index = last_root_write(&writes);
    let described: Vec<String> = moved_writes
        .iter()
        .map(|index| describe_write(&flow.writes[*index]))
        .collect();
    Some(Arm {
        name,
        moved: format!("C1 单元段的 {} 挪到 C2 单元段开头", described.join("、")),
        writes,
        segments: new_segments,
        judged_root_index,
    })
}

fn record_before_units_arm(flow: &RecordedFlow, landmarks: &LandmarkSegments) -> Arm {
    let mut segments = flow.segments.clone();
    segments.swap(
        landmarks.second_overwrite_units,
        landmarks.second_overwrite_records,
    );
    let moved_records: Vec<String> = flow.segments[landmarks.second_overwrite_records]
        .iter()
        .map(|index| describe_write(&flow.writes[*index]))
        .collect();
    let (writes, new_segments) = rearranged(flow, &segments);
    let judged_root_index = last_root_write(&writes);
    Arm {
        name: "record_before_units",
        moved: format!(
            "C2 的记录段（{}）挪到 C2 单元段之前",
            moved_records.join("、")
        ),
        writes,
        segments: new_segments,
        judged_root_index,
    }
}

/// 判据的可读段表形式：段里有几个写盖到写表里更早的写、几个写盖到基线上的非零字节。
fn overwrite_counts(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segment: &[usize],
) -> (usize, usize) {
    let mut over_an_earlier_write = 0;
    let mut over_nonzero_base = 0;
    for index in segment {
        let write = &writes[*index];
        let start = write.offset.0;
        let end = start + write.length_in_bytes();
        if writes[..*index].iter().any(|earlier| {
            earlier.device == write.device
                && earlier.offset.0 < end
                && start < earlier.offset.0 + earlier.length_in_bytes()
        }) {
            over_an_earlier_write += 1;
        }
        let length = usize::try_from(write.length_in_bytes()).expect("写长装得进 usize");
        if PoolReader::read(base, write.device, write.offset, length)
            .is_some_and(|bytes| bytes.iter().any(|byte| *byte != 0))
        {
            over_nonzero_base += 1;
        }
    }
    (over_an_earlier_write, over_nonzero_base)
}

/// 一个状态的判定：红项（`red_items`）与整份（两遍恢复走到哪、读出什么、改按映射读了几次、每条不变量的四种结局、记录核对器两样）。
#[derive(Clone, Debug, PartialEq, Eq)]
struct StateVerdict {
    red_items: BTreeSet<String>,
    whole: String,
}

fn outcome_name(report: &RecoveryReport) -> String {
    let outcome = match &report.outcome {
        RecoveryOutcome::NoFile { .. } => "no_file",
        RecoveryOutcome::FileRead { .. } => "file_read",
        RecoveryOutcome::Failed { .. } => "failed",
    };
    format!(
        "{outcome}@{:?}/fallbacks={}",
        report.effective_root, report.mapping_fallbacks
    )
}

fn verdict_of(judgement: &CrashStateJudgement) -> StateVerdict {
    let verdict_letters: String = judgement
        .pool_checker
        .iter()
        .map(|(_, verdict)| match verdict {
            InvariantVerdict::Holds => 'H',
            InvariantVerdict::Violated(_) => 'V',
            InvariantVerdict::NotApplicable(_) => 'N',
            InvariantVerdict::PartiallyJudged(_) => 'P',
        })
        .collect();
    StateVerdict {
        red_items: judgement.red_items().into_iter().collect(),
        whole: format!(
            "consulted={} ignored={} checker={} records={}/{}",
            outcome_name(&judgement.consulted),
            outcome_name(&judgement.ignored),
            verdict_letters,
            judgement.records.root_without_record,
            judgement.records.claimed_state_missing_unit
        ),
    }
}

/// 段 `segment` 的状态 `mask`（第 k 位 = 段里第 k 个写落了）：更早的段全落、这一段按 mask、之后的一个都没落。
fn persisted_of(arm: &Arm, segment: usize, mask: u64) -> Vec<bool> {
    let mut persisted = vec![false; arm.writes.len()];
    for earlier in &arm.segments[..segment] {
        for index in earlier {
            persisted[*index] = true;
        }
    }
    for (position, index) in arm.segments[segment].iter().enumerate() {
        if mask >> position & 1 == 1 {
            persisted[*index] = true;
        }
    }
    persisted
}

fn judge_masks(
    flow: &RecordedFlow,
    arm: &Arm,
    segment: usize,
    masks: &[u64],
    threads: usize,
) -> Vec<(u64, StateVerdict)> {
    let chunk = masks.len().div_ceil(threads.max(1)).max(1);
    let mut judged: Vec<(u64, StateVerdict)> = Vec::with_capacity(masks.len());
    std::thread::scope(|scope| {
        let handles: Vec<_> = masks
            .chunks(chunk)
            .map(|part| {
                scope.spawn(move || {
                    part.iter()
                        .map(|mask| {
                            let image = CrashImage {
                                base: &flow.base,
                                writes: &arm.writes,
                                persisted: persisted_of(arm, segment, *mask),
                            };
                            let judgement =
                                judge_crash_image(&image, arm.judged_root_index, &flow.versions);
                            (*mask, verdict_of(&judgement))
                        })
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        for handle in handles {
            judged.extend(handle.join().expect("判定线程没有 panic"));
        }
    });
    judged
}

fn positions_of(mask: u64, writes: usize) -> Vec<usize> {
    (0..writes)
        .filter(|position| mask >> position & 1 == 1)
        .collect()
}

/// 单写或组内子集相对空集的增减：多出的项、少掉的项。
fn delta_against(
    baseline: &StateVerdict,
    verdict: &StateVerdict,
) -> (BTreeSet<String>, BTreeSet<String>) {
    (
        verdict
            .red_items
            .difference(&baseline.red_items)
            .cloned()
            .collect(),
        baseline
            .red_items
            .difference(&verdict.red_items)
            .cloned()
            .collect(),
    )
}

fn apply_deltas(
    baseline: &StateVerdict,
    deltas: &[(BTreeSet<String>, BTreeSet<String>)],
) -> BTreeSet<String> {
    let mut items = baseline.red_items.clone();
    for (_, removed) in deltas {
        for item in removed {
            items.remove(item);
        }
    }
    for (added, _) in deltas {
        items.extend(added.iter().cloned());
    }
    items
}

#[derive(Default)]
struct Mismatches {
    compared: u64,
    red_or_not: u64,
    red_items: u64,
    examples: Vec<String>,
}

impl Mismatches {
    fn note(
        &mut self,
        mask: u64,
        writes: usize,
        actual: &StateVerdict,
        represented: &BTreeSet<String>,
    ) {
        self.compared += 1;
        let red_differs = actual.red_items.is_empty() != represented.is_empty();
        let items_differ = actual.red_items != *represented;
        if red_differs {
            self.red_or_not += 1;
        }
        if items_differ {
            self.red_items += 1;
            if self.examples.len() < EXAMPLES_PER_REPRESENTATION {
                self.examples.push(format!(
                    "state={:?} actual={:?} represented={:?}",
                    positions_of(mask, writes),
                    actual.red_items,
                    represented
                ));
            }
        }
    }
}

/// 一条臂一段：全部真子集逐个判，再与四种代表法逐状态比，打行。
fn compare_one_segment(flow: &RecordedFlow, arm: &Arm, segment: usize, threads: usize) {
    let writes = arm.segments[segment].len();
    let masks: Vec<u64> = (0..(1u64 << writes) - 1).collect();
    let judged: BTreeMap<u64, StateVerdict> = judge_masks(flow, arm, segment, &masks, threads)
        .into_iter()
        .collect();
    let baseline = &judged[&0];
    let single = |position: usize| &judged[&(1u64 << position)];
    let red_states = judged
        .values()
        .filter(|verdict| !verdict.red_items.is_empty())
        .count();
    let singles_equal_baseline = (0..writes).all(|position| single(position) == baseline);
    println!(
        "PROTO_STATES arm={} segment={segment} writes={writes} states={} red={red_states} baseline_red_items={:?} singles_equal_baseline={singles_equal_baseline}",
        arm.name,
        judged.len(),
        baseline.red_items
    );
    for position in 0..writes {
        println!(
            "PROTO_WRITE arm={} segment={segment} position={position} {} single_red_items={:?} single_equals_baseline={}",
            arm.name,
            describe_write(&arm.writes[arm.segments[segment][position]]),
            single(position).red_items,
            single(position) == baseline
        );
    }
    // 单元副本组：同偏移、不同盘的几份，按段内位置记成位掩码
    let mut groups_by_offset: BTreeMap<u64, u64> = BTreeMap::new();
    for (position, index) in arm.segments[segment].iter().enumerate() {
        *groups_by_offset
            .entry(arm.writes[*index].offset.0)
            .or_default() |= 1u64 << position;
    }
    let groups: Vec<u64> = groups_by_offset.into_values().collect();
    let mut union = Mismatches::default();
    let mut delta = Mismatches::default();
    let mut group_delta = Mismatches::default();
    let mut baseline_if_singles_equal = Mismatches::default();
    let mut baseline_whole_differs = 0u64;
    for (mask, actual) in &judged {
        let mut union_items = baseline.red_items.clone();
        let mut single_deltas = Vec::new();
        for position in positions_of(*mask, writes) {
            union_items.extend(single(position).red_items.iter().cloned());
            single_deltas.push(delta_against(baseline, single(position)));
        }
        union.note(*mask, writes, actual, &union_items);
        delta.note(
            *mask,
            writes,
            actual,
            &apply_deltas(baseline, &single_deltas),
        );
        let group_deltas: Option<Vec<_>> = groups
            .iter()
            .filter(|group| mask & **group != 0)
            .map(|group| {
                judged
                    .get(&(mask & group))
                    .map(|verdict| delta_against(baseline, verdict))
            })
            .collect();
        if let Some(group_deltas) = group_deltas {
            group_delta.note(
                *mask,
                writes,
                actual,
                &apply_deltas(baseline, &group_deltas),
            );
        }
        if singles_equal_baseline {
            baseline_if_singles_equal.note(*mask, writes, actual, &baseline.red_items);
            if actual.whole != baseline.whole {
                baseline_whole_differs += 1;
            }
        }
    }
    for (name, mismatches) in [
        ("union", &union),
        ("delta", &delta),
        ("group_delta", &group_delta),
        ("baseline_if_singles_equal", &baseline_if_singles_equal),
    ] {
        println!(
            "PROTO_COMPARE arm={} segment={segment} representation={name} compared={} red_or_not_differs={} red_items_differ={}",
            arm.name, mismatches.compared, mismatches.red_or_not, mismatches.red_items
        );
        for example in &mismatches.examples {
            println!(
                "PROTO_EXAMPLE arm={} segment={segment} representation={name} {example}",
                arm.name
            );
        }
    }
    if singles_equal_baseline {
        println!(
            "PROTO_WHOLE arm={} segment={segment} representation=baseline_if_singles_equal whole_differs={baseline_whole_differs}",
            arm.name
        );
    }
}

fn unit_segments_of(arm: &Arm, from_segment: usize) -> Vec<usize> {
    (from_segment..arm.segments.len())
        .filter(|segment| every_write_is(&arm.writes, &arm.segments[*segment], StepKind::UnitWrite))
        .collect()
}

fn main() {
    let flow = record_the_flow();
    let landmarks = match landmark_segments(&flow) {
        Ok(landmarks) => landmarks,
        Err(reason) => {
            for (segment, indexes) in flow.segments.iter().enumerate() {
                println!(
                    "PROTO_FLOW_SEGMENT segment={segment} writes={} kinds={}",
                    indexes.len(),
                    kinds_of(&flow.writes, indexes)
                );
            }
            println!("PROTO_REFUSED reason=段形状与预想不同：{reason}");
            std::process::exit(2);
        }
    };
    let mut arms = vec![unmodified_arm(&flow)];
    for (name, pair_count) in [("late_unit", 1), ("late_two_units", 2)] {
        match late_units_arm(&flow, &landmarks, name, pair_count) {
            Some(arm) => arms.push(arm),
            None => println!(
                "PROTO_ARM_SKIPPED arm={name} reason=C1 单元段里同偏移、不同盘的成对副本不到 {pair_count} 对"
            ),
        }
    }
    arms.push(record_before_units_arm(&flow, &landmarks));
    let (threads, thread_source) = crash_point_replay_worker_threads();
    println!(
        "PROTO_RUN threads={threads} source={thread_source} flow_writes={} flow_segments={}",
        flow.writes.len(),
        flow.segments.len()
    );
    // 每条臂从 C1 的单元段起往后的段表，与这一次要判的段及状态数（先算全，超上限就一个都不判）
    let mut plan: Vec<(usize, usize)> = Vec::new();
    let mut planned_states = 0u64;
    for (arm_index, arm) in arms.iter().enumerate() {
        println!(
            "PROTO_ARM arm={} moved={} judged_root_index={}",
            arm.name, arm.moved, arm.judged_root_index
        );
        for segment in landmarks.first_overwrite_units..arm.segments.len() {
            let (over_an_earlier_write, over_nonzero_base) =
                overwrite_counts(&flow.base, &arm.writes, &arm.segments[segment]);
            println!(
                "PROTO_SEGMENT arm={} segment={segment} writes={} kinds={} over_an_earlier_write={over_an_earlier_write} over_nonzero_base={over_nonzero_base}",
                arm.name,
                arm.segments[segment].len(),
                kinds_of(&arm.writes, &arm.segments[segment])
            );
        }
        let unit_segments = unit_segments_of(arm, landmarks.first_overwrite_units);
        let expanded: Vec<usize> = unit_segments
            .iter()
            .copied()
            .filter(|segment| {
                let fits = arm.segments[*segment].len() <= LARGEST_EXPANDED_SEGMENT_WRITES;
                if !fits {
                    println!("PROTO_SEGMENT_SKIPPED arm={} segment={segment} writes={} reason=写数超过 {LARGEST_EXPANDED_SEGMENT_WRITES}", arm.name, arm.segments[*segment].len());
                }
                fits
            })
            .collect();
        let segments_to_count: Vec<Vec<usize>> = expanded
            .iter()
            .map(|segment| arm.segments[*segment].clone())
            .collect();
        let closed_form = crash_state_count_with_torn_in_place_overwrites(
            &flow.base,
            &arm.writes,
            &segments_to_count,
            &|_, _| CrashPointReplaySegmentExpansion::EveryProperSubset,
        );
        let enumerated: u64 = expanded
            .iter()
            .map(|segment| (1u64 << arm.segments[*segment].len()) - 1)
            .sum();
        println!("PROTO_STATE_COUNT arm={} expanded_segments={expanded:?} closed_form={closed_form} enumerated={enumerated}", arm.name);
        println!(
            "PROTO_STATE_COUNT_AGREES arm={} agrees={}",
            arm.name,
            closed_form == enumerated + 1
        );
        planned_states += enumerated;
        plan.extend(expanded.into_iter().map(|segment| (arm_index, segment)));
    }
    println!("PROTO_PLANNED_STATES total={planned_states} ceiling={STATE_CEILING_OF_ONE_RUN}");
    if planned_states > STATE_CEILING_OF_ONE_RUN {
        println!("PROTO_REFUSED reason=状态数超过一次跑的上限");
        std::process::exit(3);
    }
    // 估时：先在第一条臂的第一段上判空集与各单写，按每态耗时外推
    if let Some((arm_index, segment)) = plan.first().copied() {
        let arm = &arms[arm_index];
        let sample: Vec<u64> = std::iter::once(0)
            .chain((0..arm.segments[segment].len()).map(|position| 1u64 << position))
            .collect();
        let started = Instant::now();
        let _ = judge_masks(&flow, arm, segment, &sample, 1);
        let per_state = started.elapsed().as_secs_f64() / sample.len() as f64;
        let estimated = per_state * planned_states as f64 / threads.max(1) as f64;
        println!("PROTO_ESTIMATE sample_states={} per_state_seconds={per_state:.4} threads={threads} estimated_seconds={estimated:.0}", sample.len());
        if estimated > WALL_CLOCK_CEILING_SECONDS {
            println!("PROTO_REFUSED reason=估时超过 40 分钟，先缩取样");
            std::process::exit(3);
        }
    }
    for (arm_index, segment) in plan {
        let started = Instant::now();
        compare_one_segment(&flow, &arms[arm_index], segment, threads);
        println!(
            "PROTO_SEGMENT_DONE arm={} segment={segment} seconds={:.1}",
            arms[arm_index].name,
            started.elapsed().as_secs_f64()
        );
    }
    println!("PROTO_FINISHED");
}

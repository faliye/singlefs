//! 逐写独立判第二轮攻方腿的原型（报告 `research/prompts/per-write-judging-r2-opus-output.md`）。
//!
//! 第一轮原型（`research/prompts/per-write-judging-r1-opus-model/`）把写数超过 19 的段整段跳过、一个状态都没判；这一份在写数多的段上
//! 按子集取样真判，另把单元段按副本组切成不超过 12 个写的小段、逐段全量判。流是原型自己调 core 的公开入口造的（两块内存稀疏盘，mkfs 只跑一次）：
//! mkfs → 取号 → 暖机 → 新池新建文件（A）→ 覆盖写（C1）→ 第二次发布（C2：覆盖写、顺序写两个数据单元、建一个 inode 三选一，是用户动作）。
//! 臂改录下来的写表（等于一份把某一处写早了、写晚了、少了一道屏障或多写了几份的实现），每条臂判几段：
//! 每个被判的状态跑 `judge_crash_image`（两遍恢复、两份 oracle、池级 checker、记录核对器），与四种代表法逐状态比：
//! - `per_write_union`（候选甲）：空集的红项 ∪ 状态里每个写「只它一个落了」那一态的红项；
//! - `per_write_delta`（候选甲）：空集的红项，减去各单写比空集少掉的项、加上各单写比空集多出的项；
//! - `copy_group_union`（候选乙）：同偏移的几份算一组，空集的红项 ∪ 状态与每组相交那一部分（组内子态）的红项；
//! - `copy_group_delta`（候选乙）：同上的增减合成。
//! 两层都比：红不红（代表的红项空不空与真判的空不空）、红项（两份集合相等不相等）。取样族与切段见各函数的注释。
//! 只读、不写任何文件、不碰判定库。用法：`per-write-judging-r2-attack <段落>`，段落是 counterexamples、user_actions、unseen_shapes、
//! small_segments_exhaustive、plan_only 之一。退出码：0 跑完；2 用法错或流的段形状与预想不同；3 状态数或估时超上限，没跑。

use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

use singlefs_checker::image::InvariantVerdict;
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::{allocator_after_make_filesystem, make_filesystem};
use singlefs_core::recovery::{PoolReader, RecoveryOutcome, RecoveryReport};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_new_inodes, publish_overwrite,
    publish_sequential_write, warm_up, FirstFile, PoolWriter, TransactionOutput,
};
use singlefs_format::{SLOT_BYTES, TEST_IMAGE_DEFAULT_BYTES};
use singlefs_verification::crash::{
    crash_state_count_with_torn_in_place_overwrites, judge_crash_image,
    torn_image_of_in_place_overwrite, CrashPointReplaySegmentExpansion, CrashStateJudgement,
    TearableInPlaceOverwrites,
};
use singlefs_verification::crash_state_enumeration::crash_point_replay_worker_threads;
use singlefs_verification::memory_pool::{
    writes_and_segments, CrashImage, MemoryPool, PublishedVersion, RetainedWrite,
    SparseBlockDevice, WrittenContents,
};
use singlefs_verification::scenario::{
    e142_parameters, first_file_content, FIXED_WRITE_TIME_SECONDS,
};
use singlefs_verification::segments::{FixedGeometry, StepKind};
use singlefs_verification::{RecordingBlockDevice, SharedStream};

/// 一次跑（一个段落）判的状态数上限。
const STATE_CEILING_OF_ONE_RUN: u64 = 1_000_000;
/// 估出来的挂钟超过它就不跑（秒）。
const WALL_CLOCK_CEILING_SECONDS: f64 = 2400.0;
/// 小段全量那一段落：一个小段至多几个写（按副本组切，组不拆开）。
const LARGEST_EXHAUSTIVE_CHUNK_WRITES: usize = 12;
/// 组格（若干整组全落、其余组一份不落）组数不超过它就全取，超过就随机取 `SAMPLED_GROUP_LATTICE_STATES` 个。
const LARGEST_FULL_GROUP_LATTICE_GROUPS: usize = 13;
const SAMPLED_GROUP_LATTICE_STATES: usize = 8192;
/// 单盘格（只在一块盘上的写里取任意子集）写数不超过它就全取，超过就随机取 `SAMPLED_DEVICE_LATTICE_STATES` 个。
const LARGEST_FULL_DEVICE_LATTICE_WRITES: usize = 12;
const SAMPLED_DEVICE_LATTICE_STATES: usize = 4096;
/// 整个取值域上均匀随机取几个状态。
const RANDOM_STATES: usize = 1000;
/// 每条臂每段每种代表法打几个对不上的例子。
const EXAMPLES_PER_REPRESENTATION: usize = 3;
/// 顺序写那一种 C2 的内容长度：大于一个数据单元，切成两个数据单元、两个事务、两条以上的记录。
const SEQUENTIAL_WRITE_BYTES: usize = 50_000;

/// C2 做哪一种发布（用户动作）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SecondPublish {
    Overwrite,
    SequentialWriteOfTwoDataUnits,
    NewInode,
}

impl SecondPublish {
    fn name(self) -> &'static str {
        match self {
            Self::Overwrite => "overwrite",
            Self::SequentialWriteOfTwoDataUnits => "sequential_write_two_data_units",
            Self::NewInode => "new_inode",
        }
    }
}

/// 录好的流：mkfs 之后的盘（基线）、之后的全部写与切好的段、各版带文件的内容。
struct RecordedFlow {
    second_publish: SecondPublish,
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    versions: Vec<PublishedVersion>,
}

fn content_of_length(seed: usize, length: usize) -> Vec<u8> {
    (0..length)
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

fn record_the_flow(second_publish: SecondPublish) -> RecordedFlow {
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
    let first_overwrite_content = content_of_length(1, 3000);
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
    let (second, second_content) = match second_publish {
        SecondPublish::Overwrite => {
            let content = content_of_length(2, 3000);
            let output = publish_overwrite(
                &mut PoolWriter::new(&parameters, &mut devices),
                &mut allocator,
                &first_overwrite,
                FirstFile {
                    content: &content,
                    write_time_seconds: FIXED_WRITE_TIME_SECONDS + 120,
                },
                instance,
            )
            .unwrap_or_else(|error| panic!("覆盖写 C2：{error:?}"));
            (output, content)
        }
        SecondPublish::SequentialWriteOfTwoDataUnits => {
            let content = content_of_length(3, SEQUENTIAL_WRITE_BYTES);
            let output = publish_sequential_write(
                &mut PoolWriter::new(&parameters, &mut devices),
                &mut allocator,
                &first_overwrite,
                FirstFile {
                    content: &content,
                    write_time_seconds: FIXED_WRITE_TIME_SECONDS + 120,
                },
                instance,
            )
            .unwrap_or_else(|error| panic!("顺序写 C2：{error:?}"));
            (output, content)
        }
        SecondPublish::NewInode => {
            let output = publish_new_inodes(
                &mut PoolWriter::new(&parameters, &mut devices),
                &mut allocator,
                &first_overwrite,
                1,
                FIXED_WRITE_TIME_SECONDS + 120,
                instance,
            )
            .unwrap_or_else(|error| panic!("建 inode C2：{error:?}"));
            (output, first_overwrite_content.clone())
        }
    };
    let geometry = FixedGeometry {
        fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
        journal_ring_bytes: parameters.geometry.journal_ring_bytes,
        root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
    };
    let (writes, segments) = writes_and_segments(&stream.retained_operations(), &geometry);
    RecordedFlow {
        second_publish,
        base,
        writes,
        segments,
        versions: vec![
            version_of(&first_file, &first_content),
            version_of(&first_overwrite, &first_overwrite_content),
            version_of(&second, &second_content),
        ],
    }
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

fn overlaps(first: &RetainedWrite, second: &RetainedWrite) -> bool {
    first.device == second.device
        && first.offset.0 < second.offset.0 + second.length_in_bytes()
        && second.offset.0 < first.offset.0 + first.length_in_bytes()
}

fn last_root_write(writes: &[RetainedWrite]) -> usize {
    (0..writes.len())
        .rev()
        .find(|index| writes[*index].kind == StepKind::RootRecordFua)
        .expect("写表里有根槽写")
}

/// 录制流里的几个段号：C1 的单元段、C2 的单元段、C2 的记录段、C2 的根段、C2 单元段之前那一段（C1 的系统配置槽轮换）。
struct LandmarkSegments {
    first_units: usize,
    second_units: usize,
    second_records: usize,
    second_root: usize,
    rotation_before_second_units: usize,
}

fn landmark_segments(flow: &RecordedFlow) -> Result<LandmarkSegments, String> {
    let is = |segment: usize, kind: StepKind| {
        every_write_is(&flow.writes, &flow.segments[segment], kind)
    };
    let unit_segments: Vec<usize> = (0..flow.segments.len())
        .filter(|segment| is(*segment, StepKind::UnitWrite))
        .collect();
    if unit_segments.len() < 2 {
        return Err(format!("只有单元写的段不到两段：{unit_segments:?}"));
    }
    let first_units = unit_segments[unit_segments.len() - 2];
    let second_units = unit_segments[unit_segments.len() - 1];
    let second_records = second_units + 1;
    let second_root = second_units + 2;
    let rotation_before_second_units = second_units - 1;
    if second_root >= flow.segments.len()
        || !is(second_records, StepKind::JournalRecord)
        || !is(second_root, StepKind::RootRecordFua)
        || !is(
            rotation_before_second_units,
            StepKind::SystemConfigurationSlot,
        )
        || rotation_before_second_units <= first_units
    {
        return Err(format!(
            "C2 单元段 {second_units} 前后不是「系统配置槽 → 单元 → 记录 → 根」"
        ));
    }
    Ok(LandmarkSegments {
        first_units,
        second_units,
        second_records,
        second_root,
        rotation_before_second_units,
    })
}

/// 目标状态：名字、段号、落了的段内位置（都按「整次写落了」）。
struct Target {
    name: String,
    segment: usize,
    positions: Vec<usize>,
}

/// 一条臂：写表（录制的写在前，被判段里原地覆写的撕裂镜像接在后）、段表、被判的根槽写、要判的段、撕裂镜像在哪、目标状态。
struct Arm {
    name: String,
    moved: String,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    judged_root_index: usize,
    judged_segments: Vec<usize>,
    torn_image_of: BTreeMap<usize, usize>,
    targets: Vec<Target>,
    versions: Vec<PublishedVersion>,
}

fn segments_as_writes(flow: &RecordedFlow) -> Vec<Vec<RetainedWrite>> {
    flow.segments
        .iter()
        .map(|segment| {
            segment
                .iter()
                .map(|index| flow.writes[*index].clone())
                .collect()
        })
        .collect()
}

/// 按段（写本身）建一条臂；被判段里的原地覆写（`TearableInPlaceOverwrites` 认的）各接一份撕裂镜像，旧字节照引擎按「这次写之前的写全落了」算。
fn arm_from_segments(
    flow: &RecordedFlow,
    name: String,
    moved: String,
    segments_of_writes: Vec<Vec<RetainedWrite>>,
    judged_segments: Vec<usize>,
) -> Arm {
    let mut writes: Vec<RetainedWrite> = Vec::new();
    let mut segments: Vec<Vec<usize>> = Vec::new();
    for segment in segments_of_writes {
        let mut indexes = Vec::with_capacity(segment.len());
        for write in segment {
            indexes.push(writes.len());
            writes.push(write);
        }
        segments.push(indexes);
    }
    let recorded = writes.len();
    let judged_root_index = last_root_write(&writes);
    let tearable = TearableInPlaceOverwrites::of(&flow.base, &writes);
    let mut torn_image_of = BTreeMap::new();
    let mut torn_writes = Vec::new();
    for segment in &judged_segments {
        for (position, index) in segments[*segment].iter().enumerate() {
            if !tearable.contains(*index) {
                continue;
            }
            let write = &writes[*index];
            assert!(
                !segments[*segment][position + 1..]
                    .iter()
                    .any(|later| overlaps(&writes[*later], write)),
                "臂 {name} 段 {segment}：原地覆写之后同段还有与它重叠的写，原型不接重放"
            );
            let mut before = flow.base.clone();
            before.apply_writes(&writes[..*index]);
            let length = usize::try_from(write.length_in_bytes()).expect("写长装得进 usize");
            let old = PoolReader::read(&before, write.device, write.offset, length)
                .expect("录到的写落在盘内");
            torn_image_of.insert(*index, recorded + torn_writes.len());
            torn_writes.push(RetainedWrite {
                device: write.device,
                kind: write.kind,
                is_force_unit_access: write.is_force_unit_access,
                offset: write.offset,
                contents: WrittenContents::Bytes(torn_image_of_in_place_overwrite(
                    &old,
                    &write.contents,
                )),
            });
        }
    }
    writes.extend(torn_writes);
    Arm {
        name,
        moved,
        writes,
        segments,
        judged_root_index,
        judged_segments,
        torn_image_of,
        targets: Vec::new(),
        versions: flow.versions.clone(),
    }
}

/// 段里按偏移分的副本组（段内位置），按组里第一个写在段里的次序排。
fn copy_groups(writes: &[RetainedWrite], segment: &[usize]) -> Vec<Vec<usize>> {
    let mut order: Vec<u64> = Vec::new();
    let mut by_offset: BTreeMap<u64, Vec<usize>> = BTreeMap::new();
    for (position, index) in segment.iter().enumerate() {
        let offset = writes[*index].offset.0;
        if !by_offset.contains_key(&offset) {
            order.push(offset);
        }
        by_offset.entry(offset).or_default().push(position);
    }
    order
        .iter()
        .map(|offset| by_offset[offset].clone())
        .collect()
}

/// 一段里成对的副本（同偏移、不同盘，录制写表的下标），数据单元（32768 字节）排前，同长的按偏移。
fn copy_pairs_in(flow: &RecordedFlow, segment: &[usize]) -> Vec<[usize; 2]> {
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
    pairs
}

fn writes_of(flow: &RecordedFlow, indexes: &[usize]) -> Vec<RetainedWrite> {
    indexes
        .iter()
        .map(|index| flow.writes[*index].clone())
        .collect()
}

fn described(flow: &RecordedFlow, indexes: &[usize]) -> String {
    let names: Vec<String> = indexes
        .iter()
        .map(|index| describe_write(&flow.writes[*index]))
        .collect();
    names.join("、")
}

fn named(flow: &RecordedFlow, arm: &str) -> String {
    format!("{}/{arm}", flow.second_publish.name())
}

fn unmodified_arm(flow: &RecordedFlow, landmarks: &LandmarkSegments) -> Arm {
    arm_from_segments(
        flow,
        named(flow, "unmodified"),
        "无".to_string(),
        segments_as_writes(flow),
        vec![landmarks.first_units, landmarks.second_units],
    )
}

/// C1 的 `pair_count` 对副本挪进 C2 单元段开头：单元晚于引用它的根（第一轮反例 A；漏项 L1）。
fn late_units_arm(
    flow: &RecordedFlow,
    landmarks: &LandmarkSegments,
    arm: &str,
    pair_count: usize,
) -> Option<Arm> {
    let pairs = copy_pairs_in(flow, &flow.segments[landmarks.first_units]);
    if pairs.len() < pair_count {
        println!(
            "PROTO_ARM_SKIPPED arm={} reason=C1 单元段成对的副本不到 {pair_count} 对",
            named(flow, arm)
        );
        return None;
    }
    let moved: Vec<usize> = pairs[..pair_count].iter().flatten().copied().collect();
    let mut segments = segments_as_writes(flow);
    let kept: Vec<usize> = flow.segments[landmarks.first_units]
        .iter()
        .copied()
        .filter(|index| !moved.contains(index))
        .collect();
    segments[landmarks.first_units] = writes_of(flow, &kept);
    let mut second = writes_of(flow, &moved);
    second.extend(writes_of(flow, &flow.segments[landmarks.second_units]));
    segments[landmarks.second_units] = second;
    let mut built = arm_from_segments(
        flow,
        named(flow, arm),
        format!("C1 单元段的 {} 挪到 C2 单元段开头", described(flow, &moved)),
        segments,
        vec![landmarks.first_units, landmarks.second_units],
    );
    built.targets.push(Target {
        name: "moved_copies_only".to_string(),
        segment: landmarks.second_units,
        positions: (0..moved.len()).collect(),
    });
    Some(built)
}

/// C2 的记录段挪到 C2 的单元段之前（第一轮反例 B；漏项 L2）。目标状态：C2 单元段里只缺一整组。
fn record_before_units_arm(flow: &RecordedFlow, landmarks: &LandmarkSegments) -> Arm {
    let mut segments = segments_as_writes(flow);
    segments.swap(landmarks.second_units, landmarks.second_records);
    let judged_units = landmarks.second_records;
    let mut built = arm_from_segments(
        flow,
        named(flow, "record_before_units"),
        format!(
            "C2 的记录段（{}）挪到 C2 单元段之前",
            described(flow, &flow.segments[landmarks.second_records])
        ),
        segments,
        vec![landmarks.first_units, judged_units],
    );
    let segment = built.segments[judged_units].clone();
    for group in copy_groups(&built.writes, &segment) {
        let positions = (0..segment.len())
            .filter(|position| !group.contains(position))
            .collect();
        let name = format!(
            "every_group_but_{}",
            built.writes[segment[group[0]]].offset.0
        );
        built.targets.push(Target {
            name,
            segment: judged_units,
            positions,
        });
    }
    built
}

/// C1 的系统配置槽轮换与 C2 的单元写之间少一道屏障、并成一段（漏项 L4；正推腿 b8「单元写与系统配置槽同段」、混合种类段）。
fn rotation_with_units_arm(flow: &RecordedFlow, landmarks: &LandmarkSegments) -> Arm {
    let mut segments = segments_as_writes(flow);
    let units = segments.remove(landmarks.second_units);
    segments[landmarks.rotation_before_second_units].extend(units);
    arm_from_segments(
        flow,
        named(flow, "rotation_with_units"),
        "C1 的系统配置槽轮换段与 C2 的单元段之间的屏障去掉、并成一段".to_string(),
        segments,
        vec![
            landmarks.first_units,
            landmarks.rotation_before_second_units,
        ],
    )
}

/// C2 的单元、记录、根三段之间的屏障都去掉、并成一段（正推腿 b6「根与单元同段」、混合种类段）。
fn units_records_root_arm(flow: &RecordedFlow, landmarks: &LandmarkSegments) -> Arm {
    let mut segments = segments_as_writes(flow);
    let root = segments.remove(landmarks.second_root);
    let records = segments.remove(landmarks.second_records);
    segments[landmarks.second_units].extend(records);
    segments[landmarks.second_units].extend(root);
    arm_from_segments(
        flow,
        named(flow, "units_records_root"),
        "C2 的单元段、记录段、根段之间的屏障去掉、并成一段".to_string(),
        segments,
        vec![landmarks.first_units, landmarks.second_units],
    )
}

/// 盖住 C1 一对副本的两次写：同盘同偏移，字节取同段另一对同长副本（`donor`）在同一块盘上的那一份。
fn planted_over(flow: &RecordedFlow, target: [usize; 2], donor: [usize; 2]) -> Vec<RetainedWrite> {
    target
        .iter()
        .map(|copy| {
            let victim = &flow.writes[*copy];
            let source = donor
                .iter()
                .map(|index| &flow.writes[*index])
                .find(|write| write.device == victim.device)
                .expect("供体两盘各一份");
            RetainedWrite {
                device: victim.device,
                kind: StepKind::UnitWrite,
                is_force_unit_access: source.is_force_unit_access,
                offset: victim.offset,
                contents: source.contents.clone(),
            }
        })
        .collect()
}

/// C2 单元段末尾接四次写，盖住 C1 两对副本（数据单元一对、16K 节点一对），选那个盖了就让文件读不出的节点（在文件那条路上、是数据单元的祖先）。
/// 判据本来就不收它（盖旧写），量的是候选乙「组间独立」那一句（正推腿取样点「多组同时被盖、且有引用关系」）。
fn planted_two_groups_arm(
    flow: &RecordedFlow,
    landmarks: &LandmarkSegments,
    threads: usize,
) -> Option<Arm> {
    let pairs = copy_pairs_in(flow, &flow.segments[landmarks.first_units]);
    let length_of = |pair: &[usize; 2]| flow.writes[pair[0]].length_in_bytes();
    let child = pairs
        .iter()
        .copied()
        .find(|pair| length_of(pair) == 32768)?;
    let nodes: Vec<[usize; 2]> = pairs
        .iter()
        .copied()
        .filter(|pair| length_of(pair) == 16384)
        .collect();
    let donor_of = |target: [usize; 2], other: Option<[usize; 2]>| {
        pairs.iter().copied().find(|pair| {
            *pair != target && Some(*pair) != other && length_of(pair) == length_of(&target)
        })
    };
    let child_donor = donor_of(child, None)?;
    let with_planted = |planted: Vec<RetainedWrite>, name: &str| {
        let mut segments = segments_as_writes(flow);
        segments[landmarks.second_units].extend(planted);
        arm_from_segments(
            flow,
            named(flow, name),
            String::new(),
            segments,
            vec![landmarks.second_units],
        )
    };
    let unit_writes = flow.segments[landmarks.second_units].len();
    let mut chosen: Option<([usize; 2], usize, bool)> = None;
    for node in &nodes {
        let Some(donor) = donor_of(*node, Some(child)) else {
            continue;
        };
        let probe = with_planted(planted_over(flow, *node, donor), "planted_probe");
        let state = State {
            landed: 0b11 << unit_writes,
            torn: 0,
        };
        let verdicts = judge_states(
            &flow.base,
            &probe,
            landmarks.second_units,
            &[state],
            threads,
        );
        let verdict = &verdicts[0];
        let unreadable = verdict
            .red_items
            .contains("journal_consulted_oracle:read_failed");
        println!(
            "PROTO_PLANT_CHOICE flow={} node={} file_unreadable={unreadable} red_items={:?}",
            flow.second_publish.name(),
            described(flow, node),
            verdict.red_items
        );
        let better = match chosen {
            None => true,
            Some((_, count, was_unreadable)) => {
                (unreadable && !was_unreadable)
                    || (unreadable == was_unreadable && verdict.red_items.len() > count)
            }
        };
        if better {
            chosen = Some((*node, verdict.red_items.len(), unreadable));
        }
    }
    let (parent, _, _) = chosen?;
    let parent_donor = donor_of(parent, Some(child))?;
    let mut planted = planted_over(flow, child, child_donor);
    planted.extend(planted_over(flow, parent, parent_donor));
    let mut built = with_planted(planted, "planted_two_groups");
    built.moved = format!(
        "C2 单元段末尾接四次写，盖住 C1 的 {}（字节取 {}）与 {}（字节取 {}）",
        described(flow, &child),
        described(flow, &child_donor),
        described(flow, &parent),
        described(flow, &parent_donor)
    );
    for (name, positions) in [
        ("both_planted_groups", unit_writes..unit_writes + 4),
        ("child_group_only", unit_writes..unit_writes + 2),
        ("parent_group_only", unit_writes + 2..unit_writes + 4),
    ] {
        built.targets.push(Target {
            name: name.to_string(),
            segment: landmarks.second_units,
            positions: positions.collect(),
        });
    }
    Some(built)
}

/// 写表里任何单元写都没碰过、基线也是 0 的一个槽：所有单元写的最远末端向上取整到槽，再空四个槽（两块盘同一个偏移）。
fn free_unit_offset(flow: &RecordedFlow) -> u64 {
    let end = flow
        .writes
        .iter()
        .filter(|write| write.kind == StepKind::UnitWrite)
        .map(|write| write.offset.0 + write.length_in_bytes())
        .max()
        .expect("流里有单元写");
    end.div_ceil(SLOT_BYTES) * SLOT_BYTES + 4 * SLOT_BYTES
}

/// 搬迁那一形（漏项 L3、第一轮循环 ② 之二）：C2 单元段开头多两次写，把 C1 一个数据单元的两份原样写到空槽（头不盖戳、birth 不变，
/// 映射与分配记录在前缀里还指源落点，与搬迁批的单元段在根落之前同形）。`drop_one_source` 时 C1 那一对里盘 1 那一份从写表里拿掉
/// （前缀上遍历方向有一份读不出，走 I-1.13 的相等比较那一支）。
fn relocation_copies_arm(
    flow: &RecordedFlow,
    landmarks: &LandmarkSegments,
    drop_one_source: bool,
) -> Option<Arm> {
    let pair = *copy_pairs_in(flow, &flow.segments[landmarks.first_units])
        .iter()
        .find(|pair| flow.writes[pair[0]].length_in_bytes() == 32768)?;
    let destination = free_unit_offset(flow);
    let copies: Vec<RetainedWrite> = pair
        .iter()
        .map(|index| {
            let source = &flow.writes[*index];
            RetainedWrite {
                device: source.device,
                kind: StepKind::UnitWrite,
                is_force_unit_access: source.is_force_unit_access,
                offset: DeviceOffsetInBytes(destination),
                contents: source.contents.clone(),
            }
        })
        .collect();
    let mut segments = segments_as_writes(flow);
    let dropped = flow.writes[pair[1]].clone();
    if drop_one_source {
        let kept: Vec<usize> = flow.segments[landmarks.first_units]
            .iter()
            .copied()
            .filter(|index| *index != pair[1])
            .collect();
        segments[landmarks.first_units] = writes_of(flow, &kept);
    }
    let mut second = copies;
    second.extend(writes_of(flow, &flow.segments[landmarks.second_units]));
    segments[landmarks.second_units] = second;
    let (arm, judged) = if drop_one_source {
        (
            "relocation_copies_over_a_missing_source",
            vec![landmarks.first_units, landmarks.second_units],
        )
    } else {
        ("relocation_copies", vec![landmarks.second_units])
    };
    let mut built = arm_from_segments(
        flow,
        named(flow, arm),
        format!(
            "C2 单元段开头多写 {} 两份的原样拷贝到两盘偏移 {destination}{}",
            described(flow, &pair),
            if drop_one_source {
                format!("；C1 单元段里 {} 拿掉", describe_write(&dropped))
            } else {
                String::new()
            }
        ),
        segments,
        judged,
    );
    for (name, positions) in [
        ("relocation_copies_only", vec![0, 1]),
        ("relocation_copy_on_device_0_only", vec![0]),
    ] {
        built.targets.push(Target {
            name: name.to_string(),
            segment: landmarks.second_units,
            positions,
        });
    }
    Some(built)
}

/// 被判的单元段按副本组切成不超过 `LARGEST_EXHAUSTIVE_CHUNK_WRITES` 个写的小段（组不拆开，次序不变），每个小段都判、全量判。
/// 等于一份在单元写之间多发几道屏障的实现：每个小段的前缀是更早的小段全落。
fn chunked_arm(flow: &RecordedFlow, arm: &Arm) -> Arm {
    assert!(arm.torn_image_of.is_empty(), "切小段只给只有单元写的臂");
    let mut segments: Vec<Vec<RetainedWrite>> = Vec::new();
    let mut judged = Vec::new();
    for (segment_index, segment) in arm.segments.iter().enumerate() {
        let take = |positions: &[usize]| -> Vec<RetainedWrite> {
            positions
                .iter()
                .map(|position| arm.writes[segment[*position]].clone())
                .collect()
        };
        if !arm.judged_segments.contains(&segment_index) {
            segments.push(
                segment
                    .iter()
                    .map(|index| arm.writes[*index].clone())
                    .collect(),
            );
            continue;
        }
        let mut chunk: Vec<RetainedWrite> = Vec::new();
        for group in copy_groups(&arm.writes, segment) {
            if !chunk.is_empty() && chunk.len() + group.len() > LARGEST_EXHAUSTIVE_CHUNK_WRITES {
                judged.push(segments.len());
                segments.push(std::mem::take(&mut chunk));
            }
            chunk.extend(take(group.as_slice()));
        }
        if !chunk.is_empty() {
            judged.push(segments.len());
            segments.push(chunk);
        }
    }
    let mut built = arm_from_segments(
        flow,
        format!("{}/chunked", arm.name),
        format!(
            "{}；被判段按副本组切成不超过 {LARGEST_EXHAUSTIVE_CHUNK_WRITES} 个写的小段",
            arm.moved
        ),
        segments,
        judged,
    );
    built.versions = arm.versions.clone();
    built
}

/// 一段里的一个崩溃状态：`landed` 第 k 位 = 段里第 k 个写整次落了，`torn` 第 k 位 = 第 k 个写（原地覆写）撕裂；两者不相交。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct State {
    landed: u64,
    torn: u64,
}

impl State {
    const EMPTY: State = State { landed: 0, torn: 0 };

    fn is_empty(self) -> bool {
        self.landed | self.torn == 0
    }

    fn restricted_to(self, mask: u64) -> State {
        State {
            landed: self.landed & mask,
            torn: self.torn & mask,
        }
    }

    fn describe(self, writes: usize) -> String {
        let positions: Vec<String> = (0..writes)
            .filter_map(|position| {
                if self.landed >> position & 1 == 1 {
                    Some(position.to_string())
                } else if self.torn >> position & 1 == 1 {
                    Some(format!("{position}撕裂"))
                } else {
                    None
                }
            })
            .collect();
        format!("[{}]", positions.join(","))
    }
}

/// 一个状态的判定：红项与整份（两遍恢复走到哪、改按映射读了几次、每条不变量的四种结局、记录核对器两样）。
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

/// 状态的持久集合：更早的段全落、这一段按 `state`（撕裂的写换成它的撕裂镜像）、之后的一个都没落。
fn persisted_of(arm: &Arm, segment: usize, state: State) -> Vec<bool> {
    let mut persisted = vec![false; arm.writes.len()];
    for earlier in &arm.segments[..segment] {
        for index in earlier {
            persisted[*index] = true;
        }
    }
    for (position, index) in arm.segments[segment].iter().enumerate() {
        if state.landed >> position & 1 == 1 {
            persisted[*index] = true;
        }
        if state.torn >> position & 1 == 1 {
            persisted[arm.torn_image_of[index]] = true;
        }
    }
    persisted
}

/// 逐个判 `states`，`threads` 条线程按小批轮着领；交回的次序与 `states` 相同。
fn judge_states(
    base: &MemoryPool,
    arm: &Arm,
    segment: usize,
    states: &[State],
    threads: usize,
) -> Vec<StateVerdict> {
    use std::sync::atomic::{AtomicUsize, Ordering};
    const BATCH: usize = 8;
    let next = AtomicUsize::new(0);
    let next = &next;
    let mut judged: Vec<(usize, StateVerdict)> = Vec::with_capacity(states.len());
    std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads.max(1))
            .map(|_| {
                scope.spawn(move || {
                    let mut mine = Vec::new();
                    loop {
                        let start = next.fetch_add(BATCH, Ordering::Relaxed);
                        if start >= states.len() {
                            break;
                        }
                        for index in start..(start + BATCH).min(states.len()) {
                            let image = CrashImage {
                                base,
                                writes: &arm.writes,
                                persisted: persisted_of(arm, segment, states[index]),
                            };
                            let judgement =
                                judge_crash_image(&image, arm.judged_root_index, &arm.versions);
                            mine.push((index, verdict_of(&judgement)));
                        }
                    }
                    mine
                })
            })
            .collect();
        for handle in handles {
            judged.extend(handle.join().expect("判定线程没有 panic"));
        }
    });
    judged.sort_by_key(|(index, _)| *index);
    assert_eq!(judged.len(), states.len(), "每个状态判一次");
    judged.into_iter().map(|(_, verdict)| verdict).collect()
}

/// `positions` 上每个写各取它的几态（原地覆写三态、其余两态）的全部组合，含空集。
fn sub_states(positions: &[usize], tearable: u64) -> Vec<State> {
    let mut states = vec![State::EMPTY];
    for position in positions {
        let bit = 1u64 << position;
        let mut extended = Vec::with_capacity(states.len() * 3);
        for state in &states {
            extended.push(*state);
            extended.push(State {
                landed: state.landed | bit,
                torn: state.torn,
            });
            if tearable & bit != 0 {
                extended.push(State {
                    landed: state.landed,
                    torn: state.torn | bit,
                });
            }
        }
        states = extended;
    }
    states
}

/// 定种子的伪随机数（splitmix64），种子取臂名与段号的 FNV-1a。
struct SplitMix(u64);

impl SplitMix {
    fn of(text: &str) -> Self {
        let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
        for byte in text.bytes() {
            hash = (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3);
        }
        SplitMix(hash)
    }

    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut mixed = self.0;
        mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        mixed ^ (mixed >> 31)
    }
}

/// 取样族（一个状态可以同属几族；`support` 是代表法要用的空集、各单写与各组组内子态，族名见 `FAMILY_NAMES`）。
const FAMILY_NAMES: [&str; 9] = [
    "support",
    "single",
    "pair",
    "all_but_one",
    "group_lattice",
    "device_lattice",
    "random",
    "every_proper_subset",
    "target",
];
const SUPPORT: usize = 0;
const SINGLE: usize = 1;
const PAIR: usize = 2;
const ALL_BUT_ONE: usize = 3;
const GROUP_LATTICE: usize = 4;
const DEVICE_LATTICE: usize = 5;
const RANDOM: usize = 6;
const EVERY_PROPER_SUBSET: usize = 7;
const TARGET: usize = 8;

fn tearable_mask(arm: &Arm, segment: usize) -> u64 {
    arm.segments[segment]
        .iter()
        .enumerate()
        .filter(|(_, index)| arm.torn_image_of.contains_key(*index))
        .fold(0, |mask, (position, _)| mask | 1u64 << position)
}

fn mask_of(positions: &[usize]) -> u64 {
    positions
        .iter()
        .fold(0, |mask, position| mask | 1u64 << position)
}

/// 一段要判的状态与各自属于哪几族。`exhaustive` 时全量（段内每个写各取几态的全部组合，去掉整段全落）；否则取样：
/// 单元写（两态）那一部分取空集、单写、成对、只差一个、组格（若干整组全落），各与原地覆写（三态）的全部组合交叉；
/// 只在一块盘的单元写里取的任意子集（单盘格，原地覆写不落）；整个取值域上均匀随机；臂点名的目标状态。
fn sample_of_segment(arm: &Arm, segment: usize, exhaustive: bool) -> BTreeMap<State, u16> {
    let indexes = &arm.segments[segment];
    let writes = indexes.len();
    assert!(writes < 64, "段内写数装得进位掩码");
    let full = (1u64 << writes) - 1;
    let tearable = tearable_mask(arm, segment);
    let groups = copy_groups(&arm.writes, indexes);
    let mut rng = SplitMix::of(&format!("{}#{segment}", arm.name));
    let mut sample: BTreeMap<State, u16> = BTreeMap::new();
    let mut add = |state: State, family: usize| {
        if state.landed == full && state.torn == 0 {
            return;
        }
        *sample.entry(state).or_default() |= 1u16 << family;
    };
    add(State::EMPTY, SUPPORT);
    for group in &groups {
        for state in sub_states(group, tearable) {
            add(state, SUPPORT);
        }
    }
    for position in 0..writes {
        add(
            State {
                landed: 1 << position,
                torn: 0,
            },
            SINGLE,
        );
        if tearable >> position & 1 == 1 {
            add(
                State {
                    landed: 0,
                    torn: 1 << position,
                },
                SINGLE,
            );
        }
    }
    if exhaustive {
        let every: Vec<usize> = (0..writes).collect();
        for state in sub_states(&every, tearable) {
            add(state, EVERY_PROPER_SUBSET);
        }
    } else {
        let unit_positions: Vec<usize> = (0..writes).filter(|p| tearable >> p & 1 == 0).collect();
        let in_place_positions: Vec<usize> =
            (0..writes).filter(|p| tearable >> p & 1 == 1).collect();
        let unit_mask = mask_of(&unit_positions);
        let mut unit_samples: Vec<(u64, usize)> = vec![(0, SUPPORT)];
        for (rank, first) in unit_positions.iter().enumerate() {
            unit_samples.push((1 << first, SINGLE));
            unit_samples.push((unit_mask & !(1 << first), ALL_BUT_ONE));
            for second in &unit_positions[rank + 1..] {
                unit_samples.push((1 << first | 1 << second, PAIR));
            }
        }
        let unit_groups: Vec<u64> = groups
            .iter()
            .map(|group| mask_of(group))
            .filter(|group| group & tearable == 0)
            .collect();
        let lattice_of = |choice: u64| {
            unit_groups
                .iter()
                .enumerate()
                .filter(|(rank, _)| choice >> rank & 1 == 1)
                .fold(0, |mask, (_, group)| mask | group)
        };
        if unit_groups.len() <= LARGEST_FULL_GROUP_LATTICE_GROUPS {
            for choice in 0..(1u64 << unit_groups.len()) {
                unit_samples.push((lattice_of(choice), GROUP_LATTICE));
            }
        } else {
            for _ in 0..SAMPLED_GROUP_LATTICE_STATES {
                let choice = rng.next() & ((1u64 << unit_groups.len()) - 1);
                unit_samples.push((lattice_of(choice), GROUP_LATTICE));
            }
        }
        for (landed, family) in &unit_samples {
            for in_place in sub_states(&in_place_positions, tearable) {
                add(
                    State {
                        landed: landed | in_place.landed,
                        torn: in_place.torn,
                    },
                    *family,
                );
            }
        }
        let devices: BTreeSet<u32> = indexes
            .iter()
            .map(|index| arm.writes[*index].device.0)
            .collect();
        for device in devices {
            let on_device: Vec<usize> = unit_positions
                .iter()
                .copied()
                .filter(|position| arm.writes[indexes[*position]].device.0 == device)
                .collect();
            let subset_of = |choice: u64| {
                on_device
                    .iter()
                    .enumerate()
                    .filter(|(rank, _)| choice >> rank & 1 == 1)
                    .fold(0, |mask, (_, position)| mask | 1u64 << position)
            };
            if on_device.len() <= LARGEST_FULL_DEVICE_LATTICE_WRITES {
                for choice in 0..(1u64 << on_device.len()) {
                    add(
                        State {
                            landed: subset_of(choice),
                            torn: 0,
                        },
                        DEVICE_LATTICE,
                    );
                }
            } else {
                for _ in 0..SAMPLED_DEVICE_LATTICE_STATES {
                    let choice = rng.next() & ((1u64 << on_device.len()) - 1);
                    add(
                        State {
                            landed: subset_of(choice),
                            torn: 0,
                        },
                        DEVICE_LATTICE,
                    );
                }
            }
        }
        for _ in 0..RANDOM_STATES {
            let mut state = State::EMPTY;
            for position in 0..writes {
                let choices = if tearable >> position & 1 == 1 { 3 } else { 2 };
                match rng.next() % choices {
                    0 => {}
                    1 => state.landed |= 1 << position,
                    _ => state.torn |= 1 << position,
                }
            }
            add(state, RANDOM);
        }
    }
    for target in arm
        .targets
        .iter()
        .filter(|target| target.segment == segment)
    {
        add(
            State {
                landed: mask_of(&target.positions),
                torn: 0,
            },
            TARGET,
        );
    }
    sample
}

const REPRESENTATIONS: [&str; 4] = [
    "per_write_union",
    "per_write_delta",
    "copy_group_union",
    "copy_group_delta",
];

/// 一族（或全部）状态的计数：判了几个、几个红、几个整份与空集不同；每种代表法红不红不同、红项不同、凑不出代表（缺要用的组内子态）各几个。
#[derive(Clone, Default)]
struct Tally {
    compared: u64,
    red: u64,
    whole_differs_from_empty: u64,
    red_or_not_differs: [u64; 4],
    red_items_differ: [u64; 4],
    unrepresentable: [u64; 4],
}

fn union_of(baseline: &StateVerdict, parts: &[&StateVerdict]) -> BTreeSet<String> {
    let mut items = baseline.red_items.clone();
    for part in parts {
        items.extend(part.red_items.iter().cloned());
    }
    items
}

fn delta_of(baseline: &StateVerdict, parts: &[&StateVerdict]) -> BTreeSet<String> {
    let mut items = baseline.red_items.clone();
    for part in parts {
        for removed in baseline.red_items.difference(&part.red_items) {
            items.remove(removed);
        }
    }
    for part in parts {
        items.extend(part.red_items.difference(&baseline.red_items).cloned());
    }
    items
}

/// 四种代表法给 `state` 的红项；要用的单写或组内子态没判过的交 None。
fn representations(
    state: State,
    writes: usize,
    group_masks: &[u64],
    judged: &BTreeMap<State, StateVerdict>,
) -> [Option<BTreeSet<String>>; 4] {
    let baseline = &judged[&State::EMPTY];
    let singles: Option<Vec<&StateVerdict>> = (0..writes)
        .filter_map(|position| {
            let bit = 1u64 << position;
            if state.landed & bit != 0 {
                Some(State {
                    landed: bit,
                    torn: 0,
                })
            } else if state.torn & bit != 0 {
                Some(State {
                    landed: 0,
                    torn: bit,
                })
            } else {
                None
            }
        })
        .map(|single| judged.get(&single))
        .collect();
    let group_parts: Option<Vec<&StateVerdict>> = group_masks
        .iter()
        .map(|mask| state.restricted_to(*mask))
        .filter(|part| !part.is_empty())
        .map(|part| judged.get(&part))
        .collect();
    [
        singles.as_ref().map(|parts| union_of(baseline, parts)),
        singles.as_ref().map(|parts| delta_of(baseline, parts)),
        group_parts.as_ref().map(|parts| union_of(baseline, parts)),
        group_parts.as_ref().map(|parts| delta_of(baseline, parts)),
    ]
}

/// 一条臂一段：取样（或全量）、逐个判、与四种代表法逐状态比，打行。交回判了几个状态。
fn compare_one_segment(
    base: &MemoryPool,
    arm: &Arm,
    segment: usize,
    exhaustive: bool,
    threads: usize,
) -> u64 {
    let sample = sample_of_segment(arm, segment, exhaustive);
    let states: Vec<State> = sample.keys().copied().collect();
    let verdicts = judge_states(base, arm, segment, &states, threads);
    let judged: BTreeMap<State, StateVerdict> = states.iter().copied().zip(verdicts).collect();
    let baseline = &judged[&State::EMPTY];
    let indexes = &arm.segments[segment];
    let writes = indexes.len();
    let group_masks: Vec<u64> = copy_groups(&arm.writes, indexes)
        .iter()
        .map(|group| mask_of(group))
        .collect();
    let label = format!("arm={} segment={segment}", arm.name);
    println!(
        "PROTO_BASELINE {label} red_items={:?} whole={}",
        baseline.red_items, baseline.whole
    );
    let mut singles_differing = 0;
    for (position, index) in indexes.iter().enumerate() {
        for single in [
            State {
                landed: 1 << position,
                torn: 0,
            },
            State {
                landed: 0,
                torn: 1 << position,
            },
        ] {
            let Some(verdict) = judged.get(&single) else {
                continue;
            };
            if verdict != baseline {
                singles_differing += 1;
                println!(
                    "PROTO_SINGLE_DIFFERS {label} state={} write={} red_items={:?} red_items_equal_empty={}",
                    single.describe(writes),
                    describe_write(&arm.writes[*index]),
                    verdict.red_items,
                    verdict.red_items == baseline.red_items
                );
            }
        }
    }
    println!("PROTO_SINGLES {label} differing_from_empty={singles_differing}");
    let mut tallies: Vec<Tally> = vec![Tally::default(); FAMILY_NAMES.len() + 1];
    let mut examples: Vec<Vec<String>> = vec![Vec::new(); REPRESENTATIONS.len()];
    for (state, families) in &sample {
        if state.is_empty() {
            continue;
        }
        let actual = &judged[state];
        let represented = representations(*state, writes, &group_masks, &judged);
        let rows = (0..FAMILY_NAMES.len())
            .filter(|family| families >> family & 1 == 1)
            .chain(std::iter::once(FAMILY_NAMES.len()));
        for row in rows {
            let tally = &mut tallies[row];
            tally.compared += 1;
            tally.red += u64::from(!actual.red_items.is_empty());
            tally.whole_differs_from_empty += u64::from(actual.whole != baseline.whole);
            for (rank, items) in represented.iter().enumerate() {
                match items {
                    None => tally.unrepresentable[rank] += 1,
                    Some(items) => {
                        tally.red_or_not_differs[rank] +=
                            u64::from(items.is_empty() != actual.red_items.is_empty());
                        tally.red_items_differ[rank] += u64::from(*items != actual.red_items);
                    }
                }
            }
        }
        for (rank, items) in represented.iter().enumerate() {
            if let Some(items) = items {
                if *items != actual.red_items && examples[rank].len() < EXAMPLES_PER_REPRESENTATION
                {
                    examples[rank].push(format!(
                        "state={} actual={:?} represented={:?}",
                        state.describe(writes),
                        actual.red_items,
                        items
                    ));
                }
            }
        }
    }
    for (row, tally) in tallies.iter().enumerate() {
        if tally.compared == 0 {
            continue;
        }
        let family = FAMILY_NAMES.get(row).copied().unwrap_or("all");
        println!(
            "PROTO_FAMILY {label} family={family} states={} red={} whole_differs_from_empty={}",
            tally.compared, tally.red, tally.whole_differs_from_empty
        );
        for (rank, representation) in REPRESENTATIONS.iter().enumerate() {
            println!(
                "PROTO_COMPARE {label} family={family} representation={representation} compared={} red_or_not_differs={} red_items_differ={} unrepresentable={}",
                tally.compared, tally.red_or_not_differs[rank], tally.red_items_differ[rank], tally.unrepresentable[rank]
            );
        }
    }
    for (rank, representation) in REPRESENTATIONS.iter().enumerate() {
        for example in &examples[rank] {
            println!("PROTO_EXAMPLE {label} representation={representation} {example}");
        }
    }
    for target in arm
        .targets
        .iter()
        .filter(|target| target.segment == segment)
    {
        let state = State {
            landed: mask_of(&target.positions),
            torn: 0,
        };
        let represented = representations(state, writes, &group_masks, &judged);
        println!(
            "PROTO_TARGET {label} target={} state={} actual={:?} whole_equals_empty={} per_write_union={:?} copy_group_union={:?}",
            target.name,
            state.describe(writes),
            judged[&state].red_items,
            judged[&state].whole == baseline.whole,
            represented[0],
            represented[2]
        );
    }
    u64::try_from(states.len()).expect("状态数装得进 u64")
}

/// 判据的可读段表形式（正推腿 D1、D2）：段里有几个写盖到写表里更早的写、几个写盖到基线上的非零字节。
fn overwrite_counts(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segment: &[usize],
) -> (usize, usize) {
    let mut over_an_earlier_write = 0;
    let mut over_nonzero_base = 0;
    for index in segment {
        let write = &writes[*index];
        if writes[..*index]
            .iter()
            .any(|earlier| overlaps(earlier, write))
        {
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

/// 一个段落：判哪几种 C2、哪几条臂、全量还是取样。
fn arms_of_the_part(part: &str, threads: usize) -> Option<(Vec<(RecordedFlow, Vec<Arm>)>, bool)> {
    let kinds: &[SecondPublish] = match part {
        "user_actions" => &[
            SecondPublish::SequentialWriteOfTwoDataUnits,
            SecondPublish::NewInode,
        ],
        "counterexamples" | "unseen_shapes" | "small_segments_exhaustive" | "plan_only" => {
            &[SecondPublish::Overwrite]
        }
        _ => return None,
    };
    let mut flows = Vec::new();
    for kind in kinds {
        let flow = record_the_flow(*kind);
        let landmarks = match landmark_segments(&flow) {
            Ok(landmarks) => landmarks,
            Err(reason) => {
                for (segment, indexes) in flow.segments.iter().enumerate() {
                    println!(
                        "PROTO_FLOW_SEGMENT flow={} segment={segment} writes={} kinds={}",
                        kind.name(),
                        indexes.len(),
                        kinds_of(&flow.writes, indexes)
                    );
                }
                println!("PROTO_REFUSED reason=段形状与预想不同：{reason}");
                std::process::exit(2);
            }
        };
        println!(
            "PROTO_FLOW flow={} writes={} segments={} first_units={} second_units={}",
            kind.name(),
            flow.writes.len(),
            flow.segments.len(),
            landmarks.first_units,
            landmarks.second_units
        );
        let counterexample_arms = |flow: &RecordedFlow, with_two_pairs: bool| {
            let mut arms = vec![unmodified_arm(flow, &landmarks)];
            arms.extend(late_units_arm(flow, &landmarks, "late_unit", 1));
            if with_two_pairs {
                arms.extend(late_units_arm(flow, &landmarks, "late_two_units", 2));
            }
            arms.push(record_before_units_arm(flow, &landmarks));
            arms
        };
        let unseen_arms = |flow: &RecordedFlow| {
            let mut arms = Vec::new();
            arms.extend(planted_two_groups_arm(flow, &landmarks, threads));
            arms.extend(relocation_copies_arm(flow, &landmarks, false));
            arms.extend(relocation_copies_arm(flow, &landmarks, true));
            arms
        };
        let arms: Vec<Arm> = match part {
            "counterexamples" => counterexample_arms(&flow, true),
            "user_actions" => counterexample_arms(&flow, false),
            "unseen_shapes" => {
                let mut arms = vec![
                    rotation_with_units_arm(&flow, &landmarks),
                    units_records_root_arm(&flow, &landmarks),
                ];
                arms.extend(unseen_arms(&flow));
                arms
            }
            "small_segments_exhaustive" => {
                let mut whole = counterexample_arms(&flow, true);
                whole.extend(unseen_arms(&flow));
                whole.iter().map(|arm| chunked_arm(&flow, arm)).collect()
            }
            _ => {
                let mut arms = counterexample_arms(&flow, true);
                arms.push(rotation_with_units_arm(&flow, &landmarks));
                arms.push(units_records_root_arm(&flow, &landmarks));
                let unseen = unseen_arms(&flow);
                arms.extend(unseen.iter().map(|arm| chunked_arm(&flow, arm)));
                arms.extend(unseen);
                arms
            }
        };
        flows.push((flow, arms));
    }
    Some((flows, part == "small_segments_exhaustive"))
}

fn main() {
    let part = std::env::args().nth(1).unwrap_or_default();
    let (threads, thread_source) = crash_point_replay_worker_threads();
    println!("PROTO_RUN part={part} threads={threads} source={thread_source}");
    let Some((flows, exhaustive)) = arms_of_the_part(&part, threads) else {
        eprintln!("用法：per-write-judging-r2-attack <counterexamples|user_actions|unseen_shapes|small_segments_exhaustive|plan_only>");
        std::process::exit(2);
    };
    // 先排全部要判的段、算状态数，超上限就一个都不判
    let mut plan: Vec<(usize, usize, usize)> = Vec::new();
    let mut planned_states = 0u64;
    let mut full_domain_states = 0u64;
    for (flow_index, (flow, arms)) in flows.iter().enumerate() {
        for (arm_index, arm) in arms.iter().enumerate() {
            println!(
                "PROTO_ARM arm={} moved={} judged_segments={:?} judged_root_index={} torn_images={}",
                arm.name,
                arm.moved,
                arm.judged_segments,
                arm.judged_root_index,
                arm.torn_image_of.len()
            );
            let recorded = &arm.writes[..arm.writes.len() - arm.torn_image_of.len()];
            let first_shown = *arm.judged_segments.iter().min().expect("至少判一段");
            for segment in first_shown..arm.segments.len() {
                let indexes = &arm.segments[segment];
                let (over_an_earlier_write, over_nonzero_base) =
                    overwrite_counts(&flow.base, recorded, indexes);
                let accepts = every_write_is(recorded, indexes, StepKind::UnitWrite)
                    && over_an_earlier_write == 0
                    && over_nonzero_base == 0;
                let judged = arm.judged_segments.contains(&segment);
                let mut line = format!(
                    "PROTO_SEGMENT arm={} segment={segment} writes={} kinds={} over_an_earlier_write={over_an_earlier_write} over_nonzero_base={over_nonzero_base} readable_criterion_accepts={accepts} copy_groups={} tearable={} judged={judged}",
                    arm.name,
                    indexes.len(),
                    kinds_of(recorded, indexes),
                    copy_groups(recorded, indexes).len(),
                    tearable_mask(arm, segment).count_ones()
                );
                if judged {
                    let closed_form = crash_state_count_with_torn_in_place_overwrites(
                        &flow.base,
                        recorded,
                        std::slice::from_ref(indexes),
                        &|_, _| CrashPointReplaySegmentExpansion::EveryProperSubset,
                    );
                    let full_domain = closed_form - 1;
                    let sampled = u64::try_from(sample_of_segment(arm, segment, exhaustive).len())
                        .expect("状态数装得进 u64");
                    line.push_str(&format!(
                        " full_domain_states={full_domain} judged_states={sampled} judged_is_full_domain={}",
                        sampled == full_domain
                    ));
                    planned_states += sampled;
                    full_domain_states += full_domain;
                    plan.push((flow_index, arm_index, segment));
                }
                println!("{line}");
            }
        }
    }
    println!(
        "PROTO_PLANNED_STATES part={part} judged={planned_states} full_domain_of_the_judged_segments={full_domain_states} ceiling={STATE_CEILING_OF_ONE_RUN}"
    );
    if part == "plan_only" {
        println!("PROTO_FINISHED part={part}");
        return;
    }
    if planned_states > STATE_CEILING_OF_ONE_RUN {
        println!("PROTO_REFUSED reason=要判的状态数超过一次跑的上限");
        std::process::exit(3);
    }
    // 估时：在第一段上单线程判空集与各单写，按每态耗时外推
    let (flow_index, arm_index, segment) = plan[0];
    let (flow, arms) = &flows[flow_index];
    let sample: Vec<State> = std::iter::once(State::EMPTY)
        .chain(
            (0..arms[arm_index].segments[segment].len()).map(|position| State {
                landed: 1 << position,
                torn: 0,
            }),
        )
        .collect();
    let started = Instant::now();
    let _ = judge_states(&flow.base, &arms[arm_index], segment, &sample, 1);
    let per_state = started.elapsed().as_secs_f64() / sample.len() as f64;
    let estimated = per_state * planned_states as f64 / threads.max(1) as f64;
    println!(
        "PROTO_ESTIMATE part={part} sample_states={} per_state_seconds={per_state:.4} threads={threads} estimated_seconds={estimated:.0}",
        sample.len()
    );
    if estimated > WALL_CLOCK_CEILING_SECONDS {
        println!("PROTO_REFUSED reason=估时超过 40 分钟，先缩取样");
        std::process::exit(3);
    }
    let started_all = Instant::now();
    let mut judged_states = 0u64;
    for (flow_index, arm_index, segment) in plan {
        let (flow, arms) = &flows[flow_index];
        let started = Instant::now();
        judged_states +=
            compare_one_segment(&flow.base, &arms[arm_index], segment, exhaustive, threads);
        println!(
            "PROTO_SEGMENT_DONE arm={} segment={segment} seconds={:.1}",
            arms[arm_index].name,
            started.elapsed().as_secs_f64()
        );
    }
    println!(
        "PROTO_JUDGED part={part} states={judged_states} seconds={:.1}",
        started_all.elapsed().as_secs_f64()
    );
    println!("PROTO_FINISHED part={part}");
}

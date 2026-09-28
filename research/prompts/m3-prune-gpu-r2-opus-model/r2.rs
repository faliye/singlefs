//! m3-prune-gpu-r2 云端攻方腿的原型（只在草稿副本里，不入库）：挂在第一轮攻方原型 attack.rs 下面的子模块，复用它的历史与判定。
//! 每个子命令打一组 `name=r2_*` 行；`must_be_nonzero=` 是这个世界要报非 0 的数（写成 `must_be_nonzero=-` 的行是对照或回归，不要求非 0）。
//! 历史一律在内存稀疏盘上录；每个世界跑前现算状态数，一次不超过约 10⁶（`r2_state_budget` 行）。

use super::*;

use singlefs_checker::walk::ATTACK_SCAN_SUMMARY;
use singlefs_checker_tier::crash::{
    attack_later_write_passes_the_reclaim_predicate, attack_layer0_plan_hash, publish_of_each_segment, Layer0Parallelism, Layer0SliceLength,
    Layer0WorkerThreadsSource,
};
use singlefs_checker_tier::layer0_progress::Layer0ShardOfShards;
use singlefs_checker_tier::verdict_store::{
    BlockStartStateIndex, EncodedVerdictVector, EnumerationPlanHash, InputFingerprint,
    StateOffsetInBlock, StreamOrNodeName, VerdictBlock, VerdictBlockKey, VerdictStore,
    VerdictVectorNumber,
};
use singlefs_harness::memory_pool::{publishes_in, RecordStreamContinuity};
use std::num::{NonZeroU32, NonZeroUsize};
use std::path::PathBuf;

pub(super) fn run(output: &mut ResultLines, arguments: &[String]) -> i32 {
    match arguments.first().map(String::as_str) {
        Some("r2-b1") => world_b1_label_versus_path_hash(output),
        Some("r2-b2-versions") => world_b2_version_table_outside_the_fingerprint(output),
        Some("r2-b2-downstream") => world_b2_node_verdict_depends_on_later_writes(output),
        Some("r2-b3") => world_b3_state_numbering(output),
        Some("r2-b4") => world_b4_p7_key(output),
        Some("r2-b4-messages") => world_b4_p7_key_and_violation_texts(output),
        Some("r2-b5-legal-reuse") => world_b5_record_key_under_legal_reuse(output),
        Some("r2-b5-implementation") => world_b5_recorded_implementation(output),
        Some("r2-f1-shifted") => world_f1_stale_unit_start_under_overlay(output),
        Some("r2-b5") => world_b5_record_ownership_and_key(output),
        Some("r2-b6") => world_b6_collision_pair_under_sha256(output),
        Some("r2-b7") => world_b7_verdict_vector(output, arguments.get(1)),
        Some("r2-b8") => world_b8_definition_version(output),
        Some("r2-d1") => world_d1_two_machines(output),
        Some("r2-kinds") => world_kinds_under_the_overlap_signature(output, arguments.get(1)),
        _ => {
            eprintln!("用法：attack r2-b1|r2-b2-versions|r2-b2-downstream|r2-b3|r2-b4|r2-f1-shifted|r2-b5|r2-b6|r2-b7 [覆盖写次数]|r2-b8|r2-d1|r2-kinds [覆盖写次数]");
            2
        }
    }
}

// ───────────────────────────── 一个状态的判定：整份摘要（与第一轮同一算法）与按项红绿 ─────────────────────────────

/// 按项红绿：两遍 oracle 的违例类、池级 checker 每条不变量的 成立 / 违反 / 不适用、记录核对器两条判据。不带违例正文。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct StatusVector {
    oracle_consult: String,
    oracle_ignore: String,
    checker: Vec<(String, char)>,
    root_without_record: bool,
    claimed_state_missing_unit: bool,
}

#[derive(Clone, Debug)]
struct R2Verdict {
    /// 与第一轮 `judge_state` 同一算法的整份摘要（两遍恢复报告、两遍 oracle 正文、checker 全部判定含正文、记录核对器）。
    full: Digest128,
    status: StatusVector,
    red: Vec<String>,
    consult_root: Option<(InstanceGeneration, CheckpointTxg)>,
    ignore_root: Option<(InstanceGeneration, CheckpointTxg)>,
}

fn verdict_of<Reader: PoolReader + ImageReader>(
    reader: &Reader,
    writes: &[RetainedWrite],
    persisted: &[bool],
    versions: &[PublishedVersion],
) -> R2Verdict {
    let newest = newest_persisted_root(writes, persisted);
    let consulted = recover(reader, JournalPolicy::Consult);
    let ignored = recover(reader, JournalPolicy::Ignore);
    let oracle_consulted = classified_oracle_violation_for_versions(&consulted.outcome, consulted.effective_root, newest, versions);
    let oracle_ignored = classified_oracle_violation_for_versions(&ignored.outcome, ignored.effective_root, newest, versions);
    let verdicts = check_pool_image(reader);
    let records = singlefs_checker_tier::crash::check_records_against(
        reader,
        reader,
        writes,
        persisted,
        RecordStreamContinuity::OneRecording,
        consulted.effective_root,
    );
    let mut red = Vec::new();
    if let Some(violation) = &oracle_consulted {
        red.push(format!("oracle_consult:{}", violation.kind.name()));
    }
    if let Some(violation) = &oracle_ignored {
        red.push(format!("oracle_ignore:{}", violation.kind.name()));
    }
    for (invariant, verdict) in &verdicts {
        if let InvariantVerdict::Violated(_) = verdict {
            red.push(format!("checker:{invariant}"));
        }
    }
    if records.root_without_record {
        red.push("records:root_without_record".to_string());
    }
    if records.claimed_state_missing_unit {
        red.push("records:claimed_state_missing_unit".to_string());
    }
    let status = StatusVector {
        oracle_consult: oracle_consulted.as_ref().map_or("-".to_string(), |violation| violation.kind.name().to_string()),
        oracle_ignore: oracle_ignored.as_ref().map_or("-".to_string(), |violation| violation.kind.name().to_string()),
        checker: verdicts
            .iter()
            .map(|(invariant, verdict)| {
                let letter = match verdict {
                    InvariantVerdict::Holds => 'H',
                    InvariantVerdict::Violated(_) => 'V',
                    InvariantVerdict::NotApplicable(_) => 'N',
                };
                ((*invariant).to_string(), letter)
            })
            .collect(),
        root_without_record: records.root_without_record,
        claimed_state_missing_unit: records.claimed_state_missing_unit,
    };
    let oracle_text = format!("{:?}|{:?}", oracle_consulted.map(|v| v.reason), oracle_ignored.map(|v| v.reason));
    let full = fingerprint_of_debug_text(&(
        format!("{consulted:?}"),
        format!("{ignored:?}"),
        oracle_text,
        format!("{verdicts:?}"),
        format!("{records:?}"),
    ));
    R2Verdict {
        full,
        status,
        red,
        consult_root: consulted.effective_root,
        ignore_root: ignored.effective_root,
    }
}

fn verdict_of_image(image: &CrashImage<'_>, versions: &[PublishedVersion]) -> R2Verdict {
    verdict_of(image, image.writes, &image.persisted, versions)
}

/// 跑前核状态数（同第一轮的 `guarded_state_count`，行名换成 r2）。
fn budget(output: &mut ResultLines, label: &str, history: &History, expanded: &dyn Fn(usize, &[usize]) -> bool) -> u64 {
    let closed = closed_form_state_count(&history.segments);
    let with_torn = history.torn_state_count(expanded);
    output.line(format!(
        "name=r2_state_budget world={label} closed_form_state_count={closed} enumerated_with_torn={with_torn} writes={} segments={}",
        history.writes.len(),
        history.segments.len()
    ));
    assert!(with_torn <= 1_000_000, "一次跑不超过约 10⁶ 个状态");
    with_torn
}

fn small(_: usize, segment: &[usize]) -> bool {
    segment.len() < 16
}

/// 按键分组，数「同键而值与组里第一个不同」的个数；交回（类数，不一致数）。
fn inconsistent_by<Key: Ord + Clone, Value: PartialEq + Clone>(pairs: &[(Key, Value)]) -> (usize, usize) {
    let mut first: BTreeMap<Key, Value> = BTreeMap::new();
    let mut inconsistent = 0;
    for (key, value) in pairs {
        let kept = first.entry(key.clone()).or_insert_with(|| value.clone());
        if kept != value {
            inconsistent += 1;
        }
    }
    (first.len(), inconsistent)
}

fn sha256_hex(bytes: &[u8]) -> String {
    singlefs_harness::sha256::sha256_hexadecimal(bytes)
}

/// 路径写表哈希（第一轮改法 3 的「整条路径写表的哈希」，最强版本：基线的全部已写扇区 + 写表每一项的盘、种类、FUA、偏移、内容，前面带定义版本串）。
fn path_write_table_hash(base: &MemoryPool, writes: &[RetainedWrite]) -> String {
    let mut message: Vec<u8> = b"singlefs r2 path write table 1".to_vec();
    message.extend_from_slice(&base.device_size_in_bytes.to_le_bytes());
    for (device, sparse) in &base.devices {
        message.extend_from_slice(&device.0.to_le_bytes());
        for (sector, bytes) in sparse.written_sectors() {
            message.extend_from_slice(&sector.to_le_bytes());
            message.extend_from_slice(&u64::try_from(bytes.len()).expect("长").to_le_bytes());
            message.extend_from_slice(bytes);
        }
    }
    for write in writes {
        message.extend_from_slice(&write.device.0.to_le_bytes());
        message.extend_from_slice(write.kind.name().as_bytes());
        message.push(u8::from(write.is_force_unit_access));
        message.extend_from_slice(&write.offset.0.to_le_bytes());
        match &write.contents {
            WrittenContents::Bytes(bytes) => {
                message.push(0);
                message.extend_from_slice(&u64::try_from(bytes.len()).expect("长").to_le_bytes());
                message.extend_from_slice(bytes);
            }
            WrittenContents::Zeros { length } => {
                message.push(1);
                message.extend_from_slice(&length.to_le_bytes());
            }
        }
    }
    sha256_hex(&message)
}

/// 一个状态落在哪一段、这一段里各次录制流写的落法（按枚举用写表认）。
fn landing_code(landing: Landing) -> u8 {
    match landing {
        Landing::NotPersisted => 0,
        Landing::Torn => 1,
        Landing::Persisted => 2,
    }
}

fn segment_and_landings(history: &History, table: &TornWriteTable, persisted: &[bool]) -> (usize, Vec<u8>) {
    let segment = segment_of_state(&history.segments, persisted);
    let landings = landings_of(table, history.writes.len(), persisted);
    let local = if segment < history.segments.len() {
        history.segments[segment].iter().map(|index| landing_code(landings[*index])).collect()
    } else {
        Vec::new()
    };
    (segment, local)
}

/// 同一条录制流写表，换一个起点镜像与写表来源时用：显式给 judged_root_index（没有根槽写的写表给 0，只影响计数）。
fn enumerate_with_root_index(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    judged_root_index: usize,
    versions: &[PublishedVersion],
    expanded: &dyn Fn(usize, &[usize]) -> bool,
    observe: &mut dyn FnMut(&CrashImage<'_>, &RecoveryReport),
) -> Layer0Tally {
    enumerate_layer0_selecting_versions_observing_each_state(base, writes, segments, judged_root_index, versions, expanded, observe)
}

// ───────────────────────────── B1：节点身份（种类串 + 覆盖签名）与路径写表哈希的两头 ─────────────────────────────

/// 与第一轮 `build_first_stream_in_memory` 同一串调用，只把新建文件的写入时间换成给的值。
fn first_stream_with_write_time(content: &[u8], write_time_seconds: u64) -> InMemoryPool {
    let parameters = common::parameters();
    let stream = SharedStream::retaining_contents();
    let mut devices: Vec<(DeviceIdentity, MemoryRecorded)> = (0..2u32)
        .map(|number| {
            (
                DeviceIdentity(number),
                RecordingBlockDevice::with_shared_stream(
                    DeviceIdentity(number),
                    SparseBlockDevice::new(common::IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
                    stream.clone(),
                ),
            )
        })
        .collect();
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mkfs_operation_count = stream.operations().len();
    let mut allocator = PoolAllocator::new(vec![
        DeviceFreeMap::new(DeviceIdentity(0), common::IMAGE_BYTES),
        DeviceFreeMap::new(DeviceIdentity(1), common::IMAGE_BYTES),
    ]);
    allocator.mark_format_time_units(
        Placement { slot: INSTANCE_TABLE_SLOT, span: 2 },
        Placement { slot: TREE_TABLE_GENESIS_SLOT, span: 1 },
    );
    let output = {
        let mut pool = PoolWriter::new(&parameters, &mut devices);
        let instance = acquire_instance(&mut pool).expect("取号");
        let warm = warm_up(&mut pool, &genesis.root, instance).expect("暖机");
        publish_first_file(
            &mut pool,
            &mut allocator,
            warm.roots.last().expect("暖机两代根"),
            FirstFile { content, write_time_seconds },
            instance,
            &warm.last_record_bytes,
        )
        .expect("新池新建文件")
    };
    InMemoryPool { devices, stream, allocator, output, mkfs_operation_count }
}

fn kind_letter(kind: StepKind) -> char {
    match kind {
        StepKind::ZeroFill => 'Z',
        StepKind::UnitWrite => 'U',
        StepKind::JournalRecord => 'J',
        StepKind::RootRecordFua => 'R',
        StepKind::SystemConfigurationSlot => 'S',
        StepKind::Barrier => 'B',
    }
}

fn writes_overlap(left: &RetainedWrite, right: &RetainedWrite) -> bool {
    left.device == right.device
        && left.offset.0 < right.offset.0 + right.length_in_bytes()
        && right.offset.0 < left.offset.0 + left.length_in_bytes()
}

/// 最强版本的 B1 覆盖签名：每次写与路径上离它最近的那一次重叠写的（种类，中间隔了几次根槽写）；没有重叠写记 `-`。
/// 只记最近那一次：记全部更早的重叠写时，每轮都写的系统配置槽让签名随发布次数一直变长，一次发布一个身份（本原型第一版就是这样，r2-kinds 量出 33 次发布 33 个身份）。
fn overlap_signature(writes: &[RetainedWrite], index: usize) -> String {
    let write = &writes[index];
    writes[..index]
        .iter()
        .enumerate()
        .rev()
        .find(|(_, earlier)| writes_overlap(earlier, write))
        .map_or_else(
            || "-".to_string(),
            |(earlier_index, earlier)| {
                let roots_between = writes[earlier_index + 1..index]
                    .iter()
                    .filter(|between| between.kind == StepKind::RootRecordFua)
                    .count();
                format!("{}{}", kind_letter(earlier.kind), roots_between)
            },
        )
}

/// 每一段的节点身份两种：B1（沿路径把每段的种类串与覆盖签名串起来求哈希）、路径写表哈希（到这一段为止）。
fn node_identities(history: &History) -> Vec<(String, String)> {
    let mut label_path = String::from("singlefs r2 node label 1");
    let mut identities = Vec::new();
    for segment in &history.segments {
        let text: String = segment
            .iter()
            .map(|index| format!("{}[{}]", kind_letter(history.writes[*index].kind), overlap_signature(&history.writes, *index)))
            .collect::<Vec<_>>()
            .join("");
        label_path.push('|');
        label_path.push_str(&text);
        let last = *segment.iter().max().expect("段里至少一次写");
        identities.push((
            sha256_hex(label_path.as_bytes()),
            path_write_table_hash(&history.base, &history.writes[..=last]),
        ));
    }
    identities
}

fn statuses_by_state(history: &History, expanded: &dyn Fn(usize, &[usize]) -> bool) -> BTreeMap<(usize, Vec<u8>), R2Verdict> {
    let table = TornWriteTable::of(&history.prepared());
    let mut verdicts = BTreeMap::new();
    enumerate_observing(history, expanded, &mut |image, _report| {
        let key = segment_and_landings(history, &table, &image.persisted);
        verdicts.insert(key, verdict_of_image(image, &history.versions));
    });
    verdicts
}

fn world_b1_label_versus_path_hash(output: &mut ResultLines) -> i32 {
    let content = common::file_content();
    let history_x = first_history();
    let history_y = {
        let pool = first_stream_with_write_time(&content, common::FIXED_WRITE_TIME_SECONDS + 1);
        let mut history = History::of_pool(&pool, Vec::new());
        history.versions = first_versions(&pool, &history.writes, &content);
        history
    };
    // Z：改坏的实现把两份数据单元的载荷各写错一个字节（校验和是按正确载荷算的）；写的位置、长度、种类、次序都不变。
    let history_z = {
        let mut history = history_x.clone();
        let mut corrupted = 0;
        for write in &mut history.writes {
            if write.kind == StepKind::UnitWrite && write.length_in_bytes() == LOCAL_DATA_UNIT_BYTES {
                if let WrittenContents::Bytes(bytes) = &mut write.contents {
                    if bytes.windows(64).any(|window| window == &content[..64]) {
                        bytes[20_000] ^= 0x01;
                        corrupted += 1;
                    }
                }
            }
        }
        assert_eq!(corrupted, 2, "两盘各一份数据单元");
        history
    };
    budget(output, "b1_x", &history_x, &small);
    let identities_x = node_identities(&history_x);
    let identities_y = node_identities(&history_y);
    let identities_z = node_identities(&history_z);
    let verdicts_x = statuses_by_state(&history_x, &small);
    let verdicts_y = statuses_by_state(&history_y, &small);
    let verdicts_z = statuses_by_state(&history_z, &small);
    let segments = history_x.segments.len();
    let mut merged_label_equal_status_differs = 0u64;
    let mut merged_segments = BTreeSet::new();
    let mut split_path_differs_status_equal = 0u64;
    let mut split_segments = BTreeSet::new();
    let mut compared = 0u64;
    for ((segment, landings), verdict_x) in &verdicts_x {
        if *segment == segments {
            continue;
        }
        compared += 1;
        let verdict_z = verdicts_z.get(&(*segment, landings.clone())).expect("Z 与 X 同形");
        if identities_x[*segment].0 == identities_z[*segment].0 && verdict_x.status != verdict_z.status {
            merged_label_equal_status_differs += 1;
            merged_segments.insert(*segment);
        }
        let verdict_y = verdicts_y.get(&(*segment, landings.clone())).expect("Y 与 X 同形");
        if identities_x[*segment].1 != identities_y[*segment].1 && verdict_x.status == verdict_y.status {
            split_path_differs_status_equal += 1;
            split_segments.insert(*segment);
        }
    }
    let label_equal_xz = (0..segments).filter(|index| identities_x[*index].0 == identities_z[*index].0).count();
    let label_equal_xy = (0..segments).filter(|index| identities_x[*index].0 == identities_y[*index].0).count();
    let path_equal_xy = (0..segments).filter(|index| identities_x[*index].1 == identities_y[*index].1).count();
    let example_z = verdicts_z
        .iter()
        .find(|((segment, _), verdict)| *segment < segments && !verdict.red.is_empty())
        .map(|((segment, _), verdict)| format!("segment={segment} red={:?}", verdict.red));
    output.line(format!(
        "name=r2_b1_merge world=x_versus_payload_mutant_z segments={segments} segments_with_equal_b1_label={label_equal_xz} compared_states={compared} states_equal_label_status_differs={merged_label_equal_status_differs} in_segments={merged_segments:?} example_z={example_z:?} must_be_nonzero={merged_label_equal_status_differs}"
    ));
    output.line(format!(
        "name=r2_b1_split world=x_versus_write_time_plus_one_y segments={segments} segments_with_equal_b1_label={label_equal_xy} segments_with_equal_path_hash={path_equal_xy} compared_states={compared} states_path_hash_differs_status_equal={split_path_differs_status_equal} in_segments={split_segments:?} must_be_nonzero={split_path_differs_status_equal}"
    ));
    0
}

// ───────────────────────────── B2：输入指纹罩不住的输入 ─────────────────────────────

fn world_b2_version_table_outside_the_fingerprint(output: &mut ResultLines) -> i32 {
    let history = first_history();
    let mut changed_versions = history.versions.clone();
    changed_versions[0].content[0] ^= 0x01;
    budget(output, "b2_versions", &history, &small);
    let path_hash = path_write_table_hash(&history.base, &history.writes);
    let mut states = 0u64;
    let mut differs = 0u64;
    let mut example = None;
    enumerate_observing(&history, &small, &mut |image, _report| {
        states += 1;
        let original = verdict_of_image(image, &history.versions);
        let changed = verdict_of_image(image, &changed_versions);
        if original.status != changed.status {
            differs += 1;
            if example.is_none() {
                example = Some(format!("original_red={:?} changed_red={:?}", original.red, changed.red));
            }
        }
    });
    output.line(format!(
        "name=r2_b2_versions path_write_table_hash_equal=true path_hash={} states={states} states_whose_status_differs={differs} example={example:?} must_be_nonzero={differs}",
        &path_hash[..16]
    ));
    0
}

/// 第一条流 + 一次覆盖写（txg 4），与第一轮 small 世界的路径 b 同一串。
fn first_plus_one_overwrite() -> History {
    let content = common::file_content();
    let path_a = first_history();
    let mut pool = build_first_stream_in_memory(&content);
    let overwrite_content: Vec<u8> = (0..4100).map(|index| u8::try_from((index * 7 + 3) % 253).expect("字节")).collect();
    overwrite_in_memory(&mut pool, &overwrite_content);
    let mut history = History::of_pool(&pool, Vec::new());
    let mut versions = first_versions(&pool, &path_a.writes, &content);
    let (instance, checkpoint_txg) = root_identity_of_root_write(&history.writes[history.judged_root_index()]);
    versions.push(PublishedVersion { instance, checkpoint_txg, content: overwrite_content });
    history.versions = versions;
    history
}

/// 按新的段次序重排一段历史（段内次序不变）：交回新写表与新段表。
fn reorder_segments(history: &History, order: &[usize]) -> History {
    let mut writes = Vec::new();
    let mut segments = Vec::new();
    for segment_index in order {
        let start = writes.len();
        for index in &history.segments[*segment_index] {
            writes.push(history.writes[*index].clone());
        }
        segments.push((start..writes.len()).collect());
    }
    History { base: history.base.clone(), writes, segments, versions: history.versions.clone() }
}

/// 一个节点（段）里每个状态的判定，按「整条路径写表」与「只到这一节点为止的写表」各判一遍逐个比。
fn compare_full_and_prefix(output: &mut ResultLines, label: &str, history: &History) -> u64 {
    budget(output, label, history, &small);
    let full = statuses_by_state(history, &small);
    let mut compared = 0u64;
    let mut status_differs = 0u64;
    let mut full_digest_differs = 0u64;
    let mut differing_segments = BTreeSet::new();
    let mut skipped_segments_without_a_root_in_the_prefix = Vec::new();
    let mut example = None;
    for segment in 0..history.segments.len() {
        if !small(segment, &history.segments[segment]) {
            continue;
        }
        let prefix = history.prefix_of_segments(segment + 1);
        if !prefix.writes.iter().any(|write| write.kind == StepKind::RootRecordFua) {
            skipped_segments_without_a_root_in_the_prefix.push(segment);
            continue;
        }
        let local = statuses_by_state(&prefix, &|index, _| index == segment);
        for ((local_segment, landings), prefix_verdict) in &local {
            if *local_segment != segment {
                continue;
            }
            let full_verdict = full.get(&(segment, landings.clone())).expect("同一段同一落法在整条路径上也枚举到了");
            compared += 1;
            if full_verdict.full != prefix_verdict.full {
                full_digest_differs += 1;
            }
            if full_verdict.status != prefix_verdict.status {
                status_differs += 1;
                differing_segments.insert(segment);
                if example.is_none() {
                    example = Some(format!(
                        "segment={segment} landings={landings:?} full_red={:?} prefix_red={:?} consult_root_full={:?} consult_root_prefix={:?}",
                        full_verdict.red, prefix_verdict.red, full_verdict.consult_root, prefix_verdict.consult_root
                    ));
                }
            }
        }
    }
    output.line(format!(
        "name=r2_b2_downstream world={label} compared_states={compared} states_whose_status_differs_full_versus_prefix={status_differs} states_whose_full_digest_differs={full_digest_differs} in_segments={differing_segments:?} skipped_segments_without_a_root_in_the_prefix={skipped_segments_without_a_root_in_the_prefix:?} example={example:?} must_be_nonzero={status_differs}"
    ));
    status_differs
}

fn world_b2_node_verdict_depends_on_later_writes(output: &mut ResultLines) -> i32 {
    let first = first_history();
    compare_full_and_prefix(output, "healthy_first_stream", &first);
    let path_b = first_plus_one_overwrite();
    compare_full_and_prefix(output, "healthy_first_plus_overwrite", &path_b);
    // 屏障放错：新建文件那次发布的 24 次单元写里，后 4 次与两条 journal 记录之间没有屏障（前 20 次之后才有）。
    {
        let segment_of_units = 7;
        assert_eq!(first.segments[segment_of_units].len(), 24);
        assert!(first.segments[8].iter().all(|index| first.writes[*index].kind == StepKind::JournalRecord));
        let mut segments = first.segments[..segment_of_units].to_vec();
        let units = &first.segments[segment_of_units];
        segments.push(units[..20].to_vec());
        let mut merged = units[20..].to_vec();
        merged.extend(first.segments[8].iter().copied());
        segments.push(merged);
        segments.extend(first.segments[9..].iter().cloned());
        let mutant = History { base: first.base.clone(), writes: first.writes.clone(), segments, versions: first.versions.clone() };
        compare_full_and_prefix(output, "barrier_after_20_of_24_units", &mutant);
    }
    // F2：新建文件那次发布先写根、后写两条记录（第一轮 order 世界），它是最后一次发布。
    {
        let mut order: Vec<usize> = (0..8).collect();
        order.extend([9, 8]);
        order.extend(10..first.segments.len());
        compare_full_and_prefix(output, "root_before_records_last_publish", &reorder_segments(&first, &order));
    }
    // F2 后面还有一次发布：txg 3 的根挪到它自己的两条记录前面。
    {
        let mut order: Vec<usize> = (0..8).collect();
        order.extend([9, 8]);
        order.extend(10..path_b.segments.len());
        compare_full_and_prefix(output, "root_before_records_then_txg4", &reorder_segments(&path_b, &order));
    }
    // 第一轮 reuse 世界：txg 4 之后提前盖掉 txg 3 的数据单元，再接一段无害的写。
    {
        let content = common::file_content();
        let mut history = path_b.clone();
        let old_data_units: Vec<RetainedWrite> = first.segments[7]
            .iter()
            .map(|index| first.writes[*index].clone())
            .filter(|write| write.length_in_bytes() == LOCAL_DATA_UNIT_BYTES && write.bytes().is_some_and(|bytes| bytes.windows(64).any(|window| window == &content[..64])))
            .collect();
        assert_eq!(old_data_units.len(), 2);
        let start = history.writes.len();
        for old in &old_data_units {
            history.writes.push(RetainedWrite { contents: WrittenContents::Bytes(vec![0xEE; 32_768]), ..old.clone() });
        }
        history.segments.push((start..history.writes.len()).collect());
        let harmless = history.writes.len();
        history.writes.push(unit_write(0, 60_000, vec![0; 16_384]));
        history.writes.push(unit_write(1, 60_000, vec![0; 16_384]));
        history.segments.push(vec![harmless, harmless + 1]);
        compare_full_and_prefix(output, "premature_reuse_of_the_txg3_data_unit", &history);
    }
    0
}

// ───────────────────────────── B3：状态身份（节点，段内序号）在全量与单点之间、ignore 开关前后 ─────────────────────────────

fn parallelism_with_threads(worker_threads: usize) -> Layer0Parallelism {
    Layer0Parallelism {
        worker_threads: NonZeroUsize::new(worker_threads).expect("线程数不为 0"),
        worker_threads_source: Layer0WorkerThreadsSource::EnvironmentVariable,
        slice_length: Layer0SliceLength::ScaledToWorkerThreads,
    }
}

fn world_b3_state_numbering(output: &mut ResultLines) -> i32 {
    let first = first_history();
    // 第一部分：第一轮 t4 世界的变体 b（父结束镜像与变体 a 逐字节相同），子节点里「段内序号 → 各次写的落法」按整条路径与按物化的父镜像各排一遍。
    let parent = first.prefix_of_segments(7);
    let record_segment = &first.segments[8];
    let record_offset = first.writes[record_segment[0]].offset;
    let child_writes: Vec<RetainedWrite> = record_segment.iter().map(|index| first.writes[*index].clone()).collect();
    let mut writes = parent.writes.clone();
    let mut segments = parent.segments.clone();
    let extra_start = writes.len();
    writes.push(RetainedWrite {
        device: DeviceIdentity(0),
        kind: StepKind::JournalRecord,
        is_force_unit_access: false,
        offset: record_offset,
        contents: WrittenContents::Bytes(vec![0x5A; 4096]),
    });
    writes.push(RetainedWrite {
        device: DeviceIdentity(0),
        kind: StepKind::ZeroFill,
        is_force_unit_access: false,
        offset: record_offset,
        contents: WrittenContents::Zeros { length: 4096 },
    });
    segments.push((extra_start..writes.len()).collect());
    let parent_end = materialized(&parent.base, &writes);
    let child_start = writes.len();
    writes.extend(child_writes.iter().cloned());
    segments.push((child_start..writes.len()).collect());
    let child_segment = segments.len() - 1;
    let full = History { base: parent.base.clone(), writes, segments, versions: first.versions.clone() };
    let full_table = TornWriteTable::of(&full.prepared());
    let mut full_order: Vec<Vec<u8>> = Vec::new();
    enumerate_observing(&full, &|index, _| index == child_segment, &mut |image, _report| {
        let (segment, landings) = segment_and_landings(&full, &full_table, &image.persisted);
        if segment == child_segment {
            full_order.push(landings);
        }
    });
    let single_point = History {
        base: parent_end.clone(),
        writes: child_writes.clone(),
        segments: vec![(0..child_writes.len()).collect()],
        versions: first.versions.clone(),
    };
    let mut single_prepared = single_point.prepared_without_root();
    single_prepared.base = parent_end.clone();
    let single_table = TornWriteTable::of(&single_prepared);
    let mut single_order: Vec<Vec<u8>> = Vec::new();
    enumerate_with_root_index(&single_point.base, &single_point.writes, &single_point.segments, 0, &single_point.versions, &|_, _| true, &mut |image, _report| {
        let landings: Vec<u8> = landings_of(&single_table, single_point.writes.len(), &image.persisted).into_iter().map(landing_code).collect();
        if segment_of_state(&single_point.segments, &image.persisted) == 0 {
            single_order.push(landings);
        }
    });
    let longest = full_order.len().max(single_order.len());
    let mismatched: Vec<String> = (0..longest)
        .filter(|ordinal| full_order.get(*ordinal) != single_order.get(*ordinal))
        .map(|ordinal| format!("{ordinal}:{:?}/{:?}", full_order.get(ordinal), single_order.get(ordinal)))
        .collect();
    output.line(format!(
        "name=r2_b3_single_point_numbering world=t4_variant_b child_states_full_path={} child_states_from_materialized_parent={} ordinals_mapping_to_different_landings={} detail={mismatched:?} must_be_nonzero={}",
        full_order.len(),
        single_order.len(),
        mismatched.len(),
        mismatched.len()
    ));
    // 第二部分：同一条流，把一个小段从展开改成 ignore（不展开）；其余段的状态在两次跑里的（计划哈希，全流序号）比一遍。
    let toggled = 2usize;
    let expansion_all = |index: usize, segment: &[usize]| small(index, segment);
    let expansion_ignoring = |index: usize, segment: &[usize]| index != toggled && small(index, segment);
    budget(output, "b3_ignore_toggle", &first, &expansion_all);
    let table = TornWriteTable::of(&first.prepared());
    let ordinals = |expansion: &dyn Fn(usize, &[usize]) -> bool| -> BTreeMap<(usize, Vec<u8>), u64> {
        let mut map = BTreeMap::new();
        let mut next = 0u64;
        enumerate_observing(&first, expansion, &mut |image, _report| {
            map.insert(segment_and_landings(&first, &table, &image.persisted), next);
            next += 1;
        });
        map
    };
    let before = ordinals(&expansion_all);
    let after = ordinals(&expansion_ignoring);
    let threads = parallelism_with_threads(4);
    let (hash_before, count_before, _) = attack_layer0_plan_hash(&first.base, &first.writes, &first.segments, first.judged_root_index(), &first.versions, &expansion_all, threads, true, false, None);
    let (hash_after, count_after, _) = attack_layer0_plan_hash(&first.base, &first.writes, &first.segments, first.judged_root_index(), &first.versions, &expansion_ignoring, threads, true, false, None);
    let common_states: Vec<&(usize, Vec<u8>)> = before.keys().filter(|key| after.contains_key(*key)).collect();
    let ordinal_moved = common_states.iter().filter(|key| before[**key] != after[**key]).count();
    let key_changed = common_states.iter().filter(|key| hash_before != hash_after || before[**key] != after[**key]).count();
    let earlier_than_toggled = common_states.iter().filter(|key| key.0 < toggled).count();
    output.line(format!(
        "name=r2_b3_ignore_toggle toggled_segment={toggled} states_before={count_before} states_after={count_after} states_in_both={} ordinal_moved={ordinal_moved} plan_hash_changed={} states_in_both_whose_plan_hash_plus_ordinal_changed={key_changed} of_which_in_segments_before_the_toggled_one={earlier_than_toggled} must_be_nonzero={key_changed}",
        common_states.len(),
        hash_before != hash_after
    ));
    0
}

// ───────────────────────────── B4：恢复与池级 checker 的等价类键（读集键与 P7 键） ─────────────────────────────

/// 池级 checker 跑一遍，扫描方向三处聚合（I-7.8 扫到的最大树 ID、I-7.7 单元区载体的最大实例代号、I-1.8 归并组与定序）的输入摘要。
fn scan_summary(reader: &dyn ImageReader) -> Vec<String> {
    ATTACK_SCAN_SUMMARY.with(|summary| summary.borrow_mut().clear());
    let _ = check_pool_image(reader);
    ATTACK_SCAN_SUMMARY.with(|summary| std::mem::take(&mut *summary.borrow_mut()))
}

#[derive(Default)]
struct KeyTally {
    states: usize,
    full_pairs_read_set: Vec<(Digest128, Digest128)>,
    status_pairs_read_set: Vec<(Digest128, StatusVector)>,
    full_pairs_p7: Vec<(Digest128, Digest128)>,
    status_pairs_p7: Vec<(Digest128, StatusVector)>,
    distinct_full: BTreeSet<Digest128>,
    distinct_status: BTreeSet<StatusVector>,
    red_states: usize,
}

impl KeyTally {
    fn add(&mut self, image: &CrashImage<'_>, versions: &[PublishedVersion]) {
        let verdict = verdict_of_image(image, versions);
        let reads = state_reads(image);
        let summary = scan_summary(image);
        let recovery_key = pair_digest(sequence_key(&reads.consult_calls), sequence_key(&reads.ignore_calls));
        let read_set_key = pair_digest(recovery_key, sequence_key(&reads.normal_calls));
        let p7_key = pair_digest(pair_digest(recovery_key, sequence_key(&reads.walk_calls)), fingerprint_of_debug_text(&summary));
        self.states += 1;
        self.red_states += usize::from(!verdict.red.is_empty());
        self.full_pairs_read_set.push((read_set_key, verdict.full));
        self.status_pairs_read_set.push((read_set_key, verdict.status.clone()));
        self.full_pairs_p7.push((p7_key, verdict.full));
        self.status_pairs_p7.push((p7_key, verdict.status.clone()));
        self.distinct_full.insert(verdict.full);
        self.distinct_status.insert(verdict.status);
    }

    fn line(&self, world: &str) -> String {
        let (read_set_classes, read_set_full_inconsistent) = inconsistent_by(&self.full_pairs_read_set);
        let (_, read_set_status_inconsistent) = inconsistent_by(&self.status_pairs_read_set);
        let (p7_classes, p7_full_inconsistent) = inconsistent_by(&self.full_pairs_p7);
        let (_, p7_status_inconsistent) = inconsistent_by(&self.status_pairs_p7);
        format!(
            "name=r2_b4_keys world={world} states={} red_states={} distinct_full_verdicts={} distinct_status_vectors={} read_set_key_classes={read_set_classes} read_set_key_inconsistent_full={read_set_full_inconsistent} read_set_key_inconsistent_status={read_set_status_inconsistent} p7_key_classes={p7_classes} p7_key_inconsistent_full={p7_full_inconsistent} p7_key_inconsistent_status={p7_status_inconsistent} p7_classes_minus_distinct_status={} must_be_nonzero={}",
            self.states,
            self.red_states,
            self.distinct_full.len(),
            self.distinct_status.len(),
            p7_classes.saturating_sub(self.distinct_status.len()),
            p7_status_inconsistent + p7_full_inconsistent
        )
    }
}

fn world_b4_p7_key(output: &mut ResultLines) -> i32 {
    let full = first_history();
    // p5：对照与改坏（单元段里写出 I-7.8 会数的孤儿、段尾盖掉），从单元段起的每个状态。
    for mutant in [false, true] {
        let label = if mutant { "p5_mutant" } else { "p5_control" };
        let (history, unit_segment) = p5_history(&full, mutant);
        budget(output, label, &history, &|_, _| true);
        let mut tally = KeyTally::default();
        enumerate_observing(&history, &|_, _| true, &mut |image, _report| {
            if segment_of_state(&history.segments, &image.persisted) >= unit_segment {
                tally.add(image, &history.versions);
            }
        });
        output.line(tally.line(label));
    }
    // p6：新建文件那段的前 12 次单元写（录制流前缀），单元段里 4095 个状态。
    {
        let prefix = full.prefix_of_segments(7);
        let mut history = prefix.clone();
        let mut segment = Vec::new();
        for index in full.segments[7].iter().take(12) {
            segment.push(history.writes.len());
            history.writes.push(full.writes[*index].clone());
        }
        history.segments.push(segment);
        let unit_segment = history.segments.len() - 1;
        budget(output, "p6_k12", &history, &|_, _| true);
        let mut tally = KeyTally::default();
        let started = Instant::now();
        enumerate_observing(&history, &|_, _| true, &mut |image, _report| {
            if segment_of_state(&history.segments, &image.persisted) == unit_segment {
                tally.add(image, &history.versions);
            }
        });
        output.line(format!("{} seconds={:.1}", tally.line("p6_k12_unit_segment"), started.elapsed().as_secs_f64()));
    }
    // 第一条流与 txg 4 覆盖写（小段全部）、reuse 与 interior 两个世界的小段全部。
    for (label, history) in [("first_stream_small", full.clone()), ("first_plus_overwrite_small", first_plus_one_overwrite())] {
        budget(output, label, &history, &small);
        let mut tally = KeyTally::default();
        enumerate_observing(&history, &small, &mut |image, _report| tally.add(image, &history.versions));
        output.line(tally.line(label));
    }
    0
}

// ───────────────────────────── F1 在叠加形态下：数据单元的后半槽上曾经有过一次单元写的开头 ─────────────────────────────

/// 候选槽只留「这一槽上最后一次持久了的单元写从这一槽开头」的那些（扫描按当前的单元边界认槽）；别的全交给崩溃镜像。
struct CurrentUnitStartReader<'reader, 'base> {
    image: &'reader CrashImage<'base>,
}

impl PoolReader for CurrentUnitStartReader<'_, '_> {
    fn device_identities(&self) -> Vec<DeviceIdentity> {
        PoolReader::device_identities(self.image)
    }
    fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> {
        PoolReader::device_size_in_bytes(self.image, device)
    }
    fn read(&self, device: DeviceIdentity, offset: DeviceOffsetInBytes, length: usize) -> Option<Vec<u8>> {
        PoolReader::read(self.image, device, offset, length)
    }
    fn journal_record_offsets_hint(&self, device: DeviceIdentity, ring_start: DeviceOffsetInBytes, ring_bytes: u64) -> Option<Vec<DeviceOffsetInBytes>> {
        PoolReader::journal_record_offsets_hint(self.image, device, ring_start, ring_bytes)
    }
}

impl ImageReader for CurrentUnitStartReader<'_, '_> {
    fn devices(&self) -> Vec<u32> {
        ImageReader::devices(self.image)
    }
    fn device_bytes(&self, device: u32) -> Option<u64> {
        ImageReader::device_bytes(self.image, device)
    }
    fn read(&self, device: u32, offset: u64, length: usize) -> Option<Vec<u8>> {
        ImageReader::read(self.image, device, offset, length)
    }
    fn candidate_unit_slots(&self, device: u32) -> Option<Vec<u64>> {
        let slots = ImageReader::candidate_unit_slots(self.image, device)?;
        Some(
            slots
                .into_iter()
                .filter(|slot| {
                    let slot_start = slot * LOCAL_SLOT_BYTES;
                    let last_covering = self
                        .image
                        .writes
                        .iter()
                        .zip(&self.image.persisted)
                        .filter(|(write, persisted)| {
                            **persisted
                                && write.kind == StepKind::UnitWrite
                                && write.device.0 == device
                                && write.offset.0 < slot_start + LOCAL_SLOT_BYTES
                                && slot_start < write.offset.0 + write.length_in_bytes()
                        })
                        .last();
                    last_covering.is_none_or(|(write, _)| write.offset.0 == slot_start)
                })
                .collect(),
        )
    }
    fn candidate_journal_slots(&self, device: u32) -> Option<Vec<u64>> {
        ImageReader::candidate_journal_slots(self.image, device)
    }
}

fn world_f1_stale_unit_start_under_overlay(output: &mut ResultLines) -> i32 {
    let full = first_history();
    let node = full.segments[7]
        .iter()
        .map(|index| &full.writes[*index])
        .find(|write| write.device == DeviceIdentity(0) && write.length_in_bytes() == LOCAL_NODE_BYTES && write.bytes().is_some_and(|bytes| bytes[6] == 2))
        .expect("新建文件那段里有一个码 2 节点")
        .bytes()
        .expect("字节")
        .to_vec();
    let orphan = orphan_node_that_the_i78_scan_counts(&node, 1 << 40);
    let header_end = 86 + 2 * usize::from(orphan[51]);
    // 32K 数据单元的内容：前 16K 不像单元头，后 16K 开头是用户数据里一段像节点头的字节。
    let mut data_unit = vec![0u8; 32_768];
    data_unit[16_384..16_384 + header_end].copy_from_slice(&orphan[..header_end]);
    for with_node_first in [false, true] {
        let label = if with_node_first { "node_at_60001_then_data_unit_at_60000" } else { "data_unit_at_60000_only" };
        let mut history = full.clone();
        if with_node_first {
            let start = history.writes.len();
            history.writes.push(unit_write(0, 60_001, node.clone()));
            history.segments.push(vec![start]);
        }
        let start = history.writes.len();
        history.writes.push(RetainedWrite {
            device: DeviceIdentity(0),
            kind: StepKind::UnitWrite,
            is_force_unit_access: false,
            offset: DeviceOffsetInBytes(60_000 * LOCAL_SLOT_BYTES),
            contents: WrittenContents::Bytes(data_unit.clone()),
        });
        history.writes.push(unit_write(1, 60_010, vec![0; 16_384]));
        history.segments.push(vec![start, start + 1]);
        budget(output, label, &history, &small);
        let first_new_segment = full.segments.len();
        let mut states = 0u64;
        let mut red_overlay = 0u64;
        let mut red_current_unit_start = 0u64;
        let mut red_names: BTreeMap<String, u64> = BTreeMap::new();
        enumerate_observing(&history, &small, &mut |image, _report| {
            if segment_of_state(&history.segments, &image.persisted) < first_new_segment {
                return;
            }
            states += 1;
            let overlay = verdict_of_image(image, &history.versions);
            let fixed_reader = CurrentUnitStartReader { image };
            let fixed = verdict_of(&fixed_reader, image.writes, &image.persisted, &history.versions);
            if !overlay.red.is_empty() {
                red_overlay += 1;
                for name in &overlay.red {
                    *red_names.entry(name.clone()).or_insert(0) += 1;
                }
            }
            red_current_unit_start += u64::from(!fixed.red.is_empty());
        });
        output.line(format!(
            "name=r2_f1_shifted world={label} states_from_the_new_segments={states} red_under_todays_overlay_candidates={red_overlay} red_names={red_names:?} red_under_current_unit_start_candidates={red_current_unit_start} must_be_nonzero={}",
            if with_node_first { red_overlay } else { 0 }
        ));
    }
    0
}

// ───────────────────────────── B5：记录归属（按写表次序 / 按记录自己的 (实例, txg)）与记录核对器的剪枝键 ─────────────────────────────

/// 一条 journal 记录写自己字节里的 (实例代号, checkpoint_txg)：偏移 16 的 u32、偏移 26 的 u64（`journal.rs` 的 `to_bytes`）。
fn record_identity(write: &RetainedWrite) -> Option<(u32, u64)> {
    let bytes = write.bytes()?;
    if bytes.len() < 34 {
        return None;
    }
    Some((
        u32::from_le_bytes(bytes[16..20].try_into().expect("4 字节")),
        u64::from_le_bytes(bytes[26..34].try_into().expect("8 字节")),
    ))
}

/// 录制流里每条根槽写 →（它写的根身份，按写表次序归来的记录，按记录自己的身份归来的记录）。
fn ownership_table(writes: &[RetainedWrite], recorded: usize) -> Vec<(usize, (u32, u64), Vec<usize>, Vec<usize>)> {
    let persisted_all = vec![true; writes.len()];
    let by_order: BTreeMap<usize, Vec<usize>> = publishes_in(&writes[..recorded], &persisted_all[..recorded], RecordStreamContinuity::OneRecording)
        .into_iter()
        .map(|publish| (publish.root, publish.records))
        .collect();
    (0..recorded)
        .filter(|index| writes[*index].kind == StepKind::RootRecordFua)
        .map(|root| {
            let (instance, txg) = root_identity_of_root_write(&writes[root]);
            let identity = (instance.0, txg.0);
            let own: Vec<usize> = (0..recorded)
                .filter(|index| writes[*index].kind == StepKind::JournalRecord && record_identity(&writes[*index]) == Some(identity))
                .collect();
            (root, identity, by_order.get(&root).cloned().unwrap_or_default(), own)
        })
        .collect()
}

/// 第一条判据换成按记录自己的身份归属：一条根在盘上、写表里带它身份的记录至少有一条、而一条都不在盘上。
fn root_without_own_record(image: &CrashImage<'_>, ownership: &[(usize, (u32, u64), Vec<usize>, Vec<usize>)]) -> bool {
    let in_place = |index: usize| {
        let write = &image.writes[index];
        let length = usize::try_from(write.length_in_bytes()).expect("长");
        PoolReader::read(image, write.device, write.offset, length).is_some_and(|bytes| write.contents.still_on_disk(&bytes))
    };
    ownership
        .iter()
        .any(|(root, _, _, own)| !own.is_empty() && in_place(*root) && !own.iter().any(|record| in_place(*record)))
}

/// 记录核对器的剪枝键（最强版本）：看 journal 那一遍恢复落到的根 + 记录核对器自己对崩溃镜像的整串读（位置 + 内容）
/// + 它直接读的持久位（恢复落到的那一版及更早的发布的每份单元副本上，更晚、与它重叠的写落没落）。
fn record_checker_key(image: &CrashImage<'_>, consult_root: Option<(InstanceGeneration, CheckpointTxg)>) -> Digest128 {
    let tables = RefCell::new(ContentTables::default());
    let reader = RecordingReader::new(image, RecordedPass::Recovery, 0, &tables);
    let _ = singlefs_checker_tier::crash::check_records_against(&reader, &reader, image.writes, &image.persisted, RecordStreamContinuity::OneRecording, consult_root);
    let mut persisted_bits: BTreeSet<(usize, bool)> = BTreeSet::new();
    if let Some((_, landed_txg)) = consult_root {
        for publish in publishes_in(image.writes, &image.persisted, RecordStreamContinuity::OneRecording) {
            if publish.checkpoint_txg > landed_txg.0 {
                continue;
            }
            for unit in &publish.units {
                for later in unit + 1..image.writes.len() {
                    if writes_overlap(&image.writes[*unit], &image.writes[later]) {
                        persisted_bits.insert((later, image.persisted[later]));
                    }
                }
            }
        }
    }
    let calls: Vec<LoggedCall> = reader.calls.borrow().clone();
    pair_digest(sequence_key(&calls), fingerprint_of_debug_text(&(consult_root, persisted_bits)))
}

/// 记录核对器的符号键（不读单元副本的内容）：看 journal 那一遍恢复落到的根 + 每条根槽写与 journal 记录写在不在盘上
/// + 恢复落到的那一版及更早的发布的单元副本上、更晚而过不了回收谓词的写落没落。过得了谓词的后写不进键：它落不落都解释得了那个扇区。
fn record_checker_symbolic_key(image: &CrashImage<'_>, consult_root: Option<(InstanceGeneration, CheckpointTxg)>) -> Digest128 {
    let in_place = |index: usize| {
        let write = &image.writes[index];
        let length = usize::try_from(write.length_in_bytes()).expect("长");
        PoolReader::read(image, write.device, write.offset, length).is_some_and(|bytes| write.contents.still_on_disk(&bytes))
    };
    let roots_and_records: Vec<(usize, bool)> = (0..image.writes.len())
        .filter(|index| matches!(image.writes[*index].kind, StepKind::RootRecordFua | StepKind::JournalRecord))
        .map(|index| (index, in_place(index)))
        .collect();
    let mut failing_bits: BTreeSet<(usize, usize, bool)> = BTreeSet::new();
    if let Some((_, landed_txg)) = consult_root {
        for publish in publishes_in(image.writes, &image.persisted, RecordStreamContinuity::OneRecording) {
            if publish.checkpoint_txg > landed_txg.0 {
                continue;
            }
            for unit in &publish.units {
                for later in unit + 1..image.writes.len() {
                    if writes_overlap(&image.writes[*unit], &image.writes[later])
                        && !attack_later_write_passes_the_reclaim_predicate(image.writes, *unit, later)
                    {
                        failing_bits.insert((*unit, later, image.persisted[later]));
                    }
                }
            }
        }
    }
    fingerprint_of_debug_text(&(consult_root, roots_and_records, failing_bits))
}

fn world_b5_record_ownership_and_key(output: &mut ResultLines) -> i32 {
    let first = first_history();
    let path_b = first_plus_one_overwrite();
    let ten_overwrites = {
        let content = common::file_content();
        let mut pool = build_first_stream_in_memory(&content);
        for seed in 0..10usize {
            let next: Vec<u8> = (0..3000).map(|index| u8::try_from((index * 5 + seed) % 251).expect("字节")).collect();
            overwrite_in_memory(&mut pool, &next);
        }
        let mut history = History::of_pool(&pool, Vec::new());
        history.versions = Vec::new();
        history
    };
    let mut order_last: Vec<usize> = (0..8).collect();
    order_last.extend([9, 8]);
    order_last.extend(10..first.segments.len());
    let mut order_next: Vec<usize> = (0..8).collect();
    order_next.extend([9, 8]);
    order_next.extend(10..path_b.segments.len());
    let worlds: Vec<(&str, History, bool)> = vec![
        ("healthy_first_stream", first.clone(), false),
        ("healthy_first_plus_overwrite", path_b.clone(), false),
        ("healthy_ten_overwrites", ten_overwrites, false),
        ("root_before_records_last_publish", reorder_segments(&first, &order_last), true),
        ("root_before_records_then_txg4", reorder_segments(&path_b, &order_next), true),
        ("premature_reuse_of_the_txg3_data_unit", premature_reuse_history(&first, &path_b), false),
    ];
    for (label, history, is_mutant) in &worlds {
        let ownership = ownership_table(&history.writes, history.writes.len());
        let ownership_differs: Vec<String> = ownership
            .iter()
            .filter(|(_, _, by_order, own)| by_order != own)
            .map(|(root, identity, by_order, own)| format!("root{root}{identity:?}:order={by_order:?}/own={own:?}"))
            .collect();
        budget(output, label, history, &small);
        let mut states = 0u64;
        let mut red_today = 0u64;
        let mut red_own = 0u64;
        let mut key_pairs: Vec<(Digest128, (bool, bool))> = Vec::new();
        let mut symbolic_pairs: Vec<(Digest128, (bool, bool))> = Vec::new();
        enumerate_observing(history, &small, &mut |image, _report| {
            states += 1;
            let consult = recover(image, JournalPolicy::Consult);
            let today = singlefs_checker_tier::crash::check_records(image, consult.effective_root);
            red_today += u64::from(today.root_without_record);
            red_own += u64::from(root_without_own_record(image, &ownership));
            key_pairs.push((record_checker_key(image, consult.effective_root), (today.root_without_record, today.claimed_state_missing_unit)));
            symbolic_pairs.push((record_checker_symbolic_key(image, consult.effective_root), (today.root_without_record, today.claimed_state_missing_unit)));
        });
        let (classes, inconsistent) = inconsistent_by(&key_pairs);
        let (symbolic_classes, symbolic_inconsistent) = inconsistent_by(&symbolic_pairs);
        output.line(format!(
            "name=r2_b5_ownership world={label} publishes={} publishes_whose_records_differ_by_ownership_rule={} detail={ownership_differs:?} states={states} root_without_record_today={red_today} root_without_own_record={red_own} record_key_classes={classes} record_key_inconsistent={inconsistent} symbolic_key_classes={symbolic_classes} symbolic_key_inconsistent={symbolic_inconsistent} must_be_nonzero={}",
            ownership.len(),
            ownership_differs.len(),
            if *is_mutant { red_own.saturating_sub(red_today) } else { 0 }
        ));
    }
    // 记录核对器的剪枝键在单元段上：p6（4095 个状态）与 p5 改坏、reuse 改坏。
    {
        let prefix = first.prefix_of_segments(7);
        let mut history = prefix.clone();
        let mut segment = Vec::new();
        for index in first.segments[7].iter().take(12) {
            segment.push(history.writes.len());
            history.writes.push(first.writes[*index].clone());
        }
        history.segments.push(segment);
        let unit_segment = history.segments.len() - 1;
        budget(output, "b5_p6_k12", &history, &|_, _| true);
        let mut key_pairs: Vec<(Digest128, (bool, bool))> = Vec::new();
        let mut symbolic_pairs: Vec<(Digest128, (bool, bool))> = Vec::new();
        enumerate_observing(&history, &|_, _| true, &mut |image, _report| {
            if segment_of_state(&history.segments, &image.persisted) != unit_segment {
                return;
            }
            let consult = recover(image, JournalPolicy::Consult);
            let check = singlefs_checker_tier::crash::check_records(image, consult.effective_root);
            key_pairs.push((record_checker_key(image, consult.effective_root), (check.root_without_record, check.claimed_state_missing_unit)));
            symbolic_pairs.push((record_checker_symbolic_key(image, consult.effective_root), (check.root_without_record, check.claimed_state_missing_unit)));
        });
        let (classes, inconsistent) = inconsistent_by(&key_pairs);
        let (symbolic_classes, symbolic_inconsistent) = inconsistent_by(&symbolic_pairs);
        output.line(format!(
            "name=r2_b5_record_key world=p6_k12_unit_segment states={} record_key_classes={classes} record_key_inconsistent={inconsistent} symbolic_key_classes={symbolic_classes} symbolic_key_inconsistent={symbolic_inconsistent} must_be_nonzero=-",
            key_pairs.len()
        ));
    }
    0
}

// ───────────────────────────── B6：第一轮 collide 那一对在抗碰撞散列下 ─────────────────────────────

fn world_b6_collision_pair_under_sha256(output: &mut ResultLines) -> i32 {
    let length = 16_384usize;
    let block_word = 100usize;
    let filler = |index: usize| -> u64 { (index as u64).wrapping_mul(0x0123_4567_89AB_CDEF) ^ 0xFEED };
    let mut builder = DigestBuilder::new(DIGEST_TAG_CONTENT);
    builder.feed_word(u64::try_from(length).expect("长"));
    for index in 0..block_word {
        builder.feed_word(filler(index));
    }
    let (start_low, start_high) = (builder.low, builder.high);
    let low_after = |word: u64| (start_low ^ word).wrapping_mul(DIGEST_MULTIPLIER_LOW).rotate_left(29);
    let collision_function = |word: u64| -> u64 {
        let low = low_after(word);
        let multiplied = ((start_high ^ word.rotate_left(17)).wrapping_mul(DIGEST_MULTIPLIER_HIGH)).rotate_left(37);
        multiplied ^ low ^ low.rotate_left(17)
    };
    let started = Instant::now();
    let mut found: Option<(u64, u64)> = None;
    let mut seed = 0x1234_5678_9ABC_DEF0u64;
    while found.is_none() {
        let mut power = 1u64;
        let mut cycle_length = 1u64;
        let mut tortoise = seed;
        let mut hare = collision_function(seed);
        while tortoise != hare {
            if power == cycle_length {
                tortoise = hare;
                power *= 2;
                cycle_length = 0;
            }
            hare = collision_function(hare);
            cycle_length += 1;
        }
        let mut tortoise = seed;
        let mut hare = seed;
        for _ in 0..cycle_length {
            hare = collision_function(hare);
        }
        let mut previous_tortoise = 0u64;
        let mut previous_hare = 0u64;
        let mut tail = 0u64;
        while tortoise != hare {
            previous_tortoise = tortoise;
            previous_hare = hare;
            tortoise = collision_function(tortoise);
            hare = collision_function(hare);
            tail += 1;
        }
        if tail > 0 && previous_tortoise != previous_hare {
            found = Some((previous_tortoise, previous_hare));
        } else {
            seed = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
        }
    }
    let (first_word, second_word) = found.expect("撞到了");
    let next_word = 0x0BAD_C0DEu64;
    let second_next_word = next_word ^ low_after(first_word) ^ low_after(second_word);
    let build = |block: [u64; 2]| -> Vec<u8> {
        (0..length / 8)
            .flat_map(|index| {
                let word = match index.checked_sub(block_word) {
                    Some(0) => block[0],
                    Some(1) => block[1],
                    _ => filler(index),
                };
                word.to_le_bytes()
            })
            .collect()
    };
    let first = build([first_word, next_word]);
    let second = build([second_word, second_next_word]);
    let first_sha = sha256_hex(&first);
    let second_sha = sha256_hex(&second);
    output.line(format!(
        "name=r2_b6_collide_regression digest128_equal={} bytes_equal={} sha256_first={} sha256_second={} sha256_equal={} seconds={:.1} must_be_nonzero=-",
        digest_of_bytes(&first) == digest_of_bytes(&second),
        first == second,
        &first_sha[..16],
        &second_sha[..16],
        first_sha == second_sha,
        started.elapsed().as_secs_f64()
    ));
    0
}

// ───────────────────────────── B7：判定向量 ─────────────────────────────

/// 第一条流 + `overwrites` 次覆盖写，每一版都登记进版本表（与第一轮 kinds 世界同一串内容）。
fn many_overwrites(overwrites: usize) -> History {
    let content = common::file_content();
    let mut pool = build_first_stream_in_memory(&content);
    let mut contents = vec![content.clone()];
    for seed in 0..overwrites {
        let next: Vec<u8> = (0..3000).map(|index| u8::try_from((index * 5 + seed) % 251).expect("字节")).collect();
        overwrite_in_memory(&mut pool, &next);
        contents.push(next);
    }
    let mut history = History::of_pool(&pool, Vec::new());
    let roots: Vec<usize> = (0..history.writes.len()).filter(|index| history.writes[*index].kind == StepKind::RootRecordFua).collect();
    assert_eq!(roots.len(), 2 + contents.len(), "暖机两代根 + 每一版一条根");
    history.versions = roots[2..]
        .iter()
        .zip(contents)
        .map(|(root, content)| {
            let (instance, checkpoint_txg) = root_identity_of_root_write(&history.writes[*root]);
            PublishedVersion { instance, checkpoint_txg, content }
        })
        .collect();
    history
}

fn scratch_library(name: &str) -> PathBuf {
    let directory = PathBuf::from(format!("/tmp/claude-1000/m3-prune-gpu-r2-opus/kv/{name}-{}", std::process::id()));
    std::fs::create_dir_all(directory.parent().expect("上级目录")).expect("建 kv 目录");
    if directory.exists() {
        std::fs::remove_dir_all(&directory).expect("删旧库");
    }
    directory
}

fn world_b7_verdict_vector(output: &mut ResultLines, overwrites_argument: Option<&String>) -> i32 {
    // 第一部分：同一个节点（共享前缀的段）在两条路径上归哪次发布不同，计数格子跟着不同。
    let path_a = first_history();
    let path_b = first_plus_one_overwrite();
    let publish_a = publish_of_each_segment(&path_a.writes, &path_a.segments);
    let publish_b = publish_of_each_segment(&path_b.writes, &path_b.segments);
    let mut states_whose_cell_differs = 0u64;
    let mut detail = Vec::new();
    for segment in 0..path_a.segments.len() {
        if publish_a[segment] != publish_b[segment] {
            let states = path_a.torn_state_count(&|index, _| index == segment) - 1;
            states_whose_cell_differs += states;
            detail.push(format!("{segment}:{}->{}:{states}", publish_a[segment].name(), publish_b[segment].name()));
        }
    }
    output.line(format!(
        "name=r2_b7_publish_cell shared_segments={} segments_whose_publish_cell_differs={detail:?} states_whose_tally_cell_is_not_a_function_of_the_node={states_whose_cell_differs} must_be_nonzero={states_whose_cell_differs}",
        path_a.segments.len()
    ));
    // 第二部分：长流上不同判定向量有几种（不带 / 带恢复落到的根），带根时第 257 种在真库上被拒。
    let overwrites: usize = overwrites_argument.map_or(300, |text| text.parse().expect("次数"));
    let started = Instant::now();
    let history = many_overwrites(overwrites);
    let built_seconds = started.elapsed().as_secs_f64();
    budget(output, "b7_many_overwrites", &history, &small);
    let mut without_roots: BTreeSet<StatusVector> = BTreeSet::new();
    let mut with_roots: Vec<String> = Vec::new();
    let mut with_roots_set: BTreeSet<String> = BTreeSet::new();
    let mut states = 0u64;
    let started = Instant::now();
    enumerate_observing(&history, &small, &mut |image, _report| {
        states += 1;
        let verdict = verdict_of_image(image, &history.versions);
        let with_root = format!("{:?}|{:?}|{:?}", verdict.status, verdict.consult_root, verdict.ignore_root);
        if with_roots_set.insert(with_root.clone()) {
            with_roots.push(with_root);
        }
        without_roots.insert(verdict.status);
    });
    let directory = scratch_library("b7");
    let mut store = VerdictStore::create_empty(&directory).expect("建库");
    let mut refused_at = None;
    for (ordinal, vector) in with_roots.iter().enumerate() {
        if let Err(error) = store.register_verdict_vector(&EncodedVerdictVector(vector.as_bytes().to_vec())) {
            refused_at = Some(format!("{}:{error:?}", ordinal + 1));
            break;
        }
    }
    drop(store);
    std::fs::remove_dir_all(&directory).expect("删库");
    output.line(format!(
        "name=r2_b7_distinct_vectors overwrites={overwrites} states={states} distinct_status_vectors={} distinct_vectors_with_landed_roots={} real_store_refused_at={refused_at:?} build_seconds={built_seconds:.1} seconds={:.1} must_be_nonzero={}",
        without_roots.len(),
        with_roots.len(),
        started.elapsed().as_secs_f64(),
        with_roots.len().saturating_sub(256)
    ));
    0
}

// ───────────────────────────── B8：定义版本 ─────────────────────────────

fn premature_reuse_history(first: &History, path_b: &History) -> History {
    let content = common::file_content();
    let mut history = path_b.clone();
    let old_data_units: Vec<RetainedWrite> = first.segments[7]
        .iter()
        .map(|index| first.writes[*index].clone())
        .filter(|write| write.length_in_bytes() == LOCAL_DATA_UNIT_BYTES && write.bytes().is_some_and(|bytes| bytes.windows(64).any(|window| window == &content[..64])))
        .collect();
    assert_eq!(old_data_units.len(), 2);
    let start = history.writes.len();
    for old in &old_data_units {
        history.writes.push(RetainedWrite { contents: WrittenContents::Bytes(vec![0xEE; 32_768]), ..old.clone() });
    }
    history.segments.push((start..history.writes.len()).collect());
    let harmless = history.writes.len();
    history.writes.push(unit_write(0, 60_000, vec![0; 16_384]));
    history.writes.push(unit_write(1, 60_000, vec![0; 16_384]));
    history.segments.push(vec![harmless, harmless + 1]);
    history
}

/// 判定来源：p5 改坏（整条枚举）与 reuse 改坏（小段全部），按枚举次序。
fn source_statuses(output: &mut ResultLines) -> Vec<StatusVector> {
    let first = first_history();
    let path_b = first_plus_one_overwrite();
    let (p5, _) = p5_history(&first, true);
    let reuse = premature_reuse_history(&first, &path_b);
    let mut statuses = Vec::new();
    budget(output, "source_p5_mutant", &p5, &|_, _| true);
    enumerate_observing(&p5, &|_, _| true, &mut |image, _report| statuses.push(verdict_of_image(image, &p5.versions).status));
    budget(output, "source_premature_reuse", &reuse, &small);
    enumerate_observing(&reuse, &small, &mut |image, _report| statuses.push(verdict_of_image(image, &reuse.versions).status));
    statuses
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum VerdictEncoding {
    /// 一字节：位 0 看 journal 的 oracle 红、位 1 不看 journal 的 oracle 红、位 2 池级 checker 任一条红、位 3 记录核对器任一条红。
    Original,
    /// 定义改了、字段名跟着改：位 3 拆成「根在而记录缺」、位 4「声称的单元缺席」。
    RecordsSplitIntoTwoBits,
    /// 定义改了、字段名不变：位 2 不再含 I-7.8（挪去单独计），位 3 只剩「声称的单元缺席」。
    SameFieldNamesNewMeaning,
}

impl VerdictEncoding {
    fn field_names(self) -> &'static [&'static str] {
        match self {
            Self::Original | Self::SameFieldNamesNewMeaning => &["oracle_consult_red", "oracle_ignore_red", "checker_red", "records_red"],
            Self::RecordsSplitIntoTwoBits => &["oracle_consult_red", "oracle_ignore_red", "checker_red", "root_without_record", "claimed_state_missing_unit"],
        }
    }

    fn schema_hash(self) -> String {
        sha256_hex(self.field_names().join(",").as_bytes())[..16].to_string()
    }

    fn encode(self, status: &StatusVector) -> u8 {
        let oracle = u8::from(status.oracle_consult != "-") | (u8::from(status.oracle_ignore != "-") << 1);
        let any_checker = status.checker.iter().any(|(_, letter)| *letter == 'V');
        let checker_without_i78 = status.checker.iter().any(|(name, letter)| *letter == 'V' && name != "I-7.8");
        match self {
            Self::Original => oracle | (u8::from(any_checker) << 2) | (u8::from(status.root_without_record || status.claimed_state_missing_unit) << 3),
            Self::RecordsSplitIntoTwoBits => oracle | (u8::from(any_checker) << 2) | (u8::from(status.root_without_record) << 3) | (u8::from(status.claimed_state_missing_unit) << 4),
            Self::SameFieldNamesNewMeaning => oracle | (u8::from(checker_without_i78) << 2) | (u8::from(status.claimed_state_missing_unit) << 3),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum VersionPlacement {
    /// 今天的库：块键与判定向量表里都没有定义版本。
    NoVersion,
    /// 库级布局版本（一个元数据块，开库时核）：编码改了，库的布局没改，版本不动。
    LibraryLayoutVersion,
    /// 判定向量编码的版本进块键前缀，改定义的人抬了版本。
    DefinitionVersionInKeyBumped,
    /// 同上，改定义的人忘了抬。
    DefinitionVersionInKeyForgotten,
    /// 编码的字段名表的哈希进块键前缀（自动算，不靠人抬）。
    SchemaHashInKey,
    /// 库里存一组金丝雀状态与它们的编码；开库时用读者自己的编码重判这几个状态，逐个比，对不上拒读。金丝雀取每第 8 个状态。
    CanaryStatesRejudgedAtOpen,
    /// 同上，金丝雀取「库里每一种判定向量各一个状态」（第一次出现的那个）。
    CanaryOneStatePerStoredVector,
    /// 同上，每一种判定向量取最后一次出现的那个状态。
    CanaryOneStatePerStoredVectorLastOccurrence,
    /// 不存编码过的位，存逐项的原始判定（每条不变量的名字与 成立 / 违反 / 不适用、两遍 oracle 的违例类名、记录核对器两条），读的一方按自己的定义现算要的位。
    RawStatusesDerivedAtRead,
}

fn serialized_status(status: &StatusVector) -> String {
    let checker: Vec<String> = status.checker.iter().map(|(name, letter)| format!("{name}={letter}")).collect();
    format!(
        "oc={}|oi={}|rwr={}|csm={}|{}",
        status.oracle_consult,
        status.oracle_ignore,
        u8::from(status.root_without_record),
        u8::from(status.claimed_state_missing_unit),
        checker.join(",")
    )
}

fn parsed_status(text: &str) -> StatusVector {
    let fields: Vec<&str> = text.splitn(5, '|').collect();
    let value = |field: &str, prefix: &str| field.strip_prefix(prefix).expect("字段前缀").to_string();
    StatusVector {
        oracle_consult: value(fields[0], "oc="),
        oracle_ignore: value(fields[1], "oi="),
        root_without_record: value(fields[2], "rwr=") == "1",
        claimed_state_missing_unit: value(fields[3], "csm=") == "1",
        checker: fields[4]
            .split(',')
            .filter(|pair| !pair.is_empty())
            .map(|pair| {
                let (name, letter) = pair.rsplit_once('=').expect("名字=字母");
                (name.to_string(), letter.chars().next().expect("字母"))
            })
            .collect(),
    }
}

const B8_BLOCK_STATES: usize = 16;

fn b8_block_key(name: &str, start: u64) -> VerdictBlockKey {
    VerdictBlockKey {
        input_fingerprint: InputFingerprint([0x11; 32]),
        stream_or_node_name: StreamOrNodeName::new(name).expect("名字"),
        enumeration_plan_hash: EnumerationPlanHash([0x22; 32]),
        block_start: BlockStartStateIndex(start),
    }
}

fn store_single_byte_blocks(store: &mut VerdictStore, name: &str, bytes: &[u8]) {
    for (block_index, chunk) in bytes.chunks(B8_BLOCK_STATES).enumerate() {
        let numbers: Vec<VerdictVectorNumber> = chunk
            .iter()
            .map(|byte| store.register_verdict_vector(&EncodedVerdictVector(vec![*byte])).expect("登记"))
            .collect();
        let violating: Vec<StateOffsetInBlock> = chunk
            .iter()
            .enumerate()
            .filter(|(_, byte)| **byte != 0)
            .map(|(offset, _)| StateOffsetInBlock(u64::try_from(offset).expect("序号")))
            .collect();
        let start = u64::try_from(block_index * B8_BLOCK_STATES).expect("起点");
        store
            .store_verdict_block(&b8_block_key(name, start), &VerdictBlock::new(numbers).expect("非空"), &violating)
            .expect("存块");
    }
}

fn read_single_byte_blocks(store: &VerdictStore, name: &str, states: usize) -> Option<Vec<u8>> {
    let mut bytes = Vec::new();
    for block_index in 0..states.div_ceil(B8_BLOCK_STATES) {
        let block = store.read_verdict_block(&b8_block_key(name, u64::try_from(block_index * B8_BLOCK_STATES).expect("起点"))).expect("读块")?;
        for number in block.verdict_vector_number_of_each_state() {
            bytes.push(store.verdict_vector_of_number(*number).expect("登记过").0[0]);
        }
    }
    Some(bytes)
}

fn world_b8_definition_version(output: &mut ResultLines) -> i32 {
    let statuses = source_statuses(output);
    let canary_states: Vec<usize> = (0..statuses.len()).step_by(8).collect();
    let per_vector_canary_states: Vec<usize> = {
        let mut seen = BTreeSet::new();
        (0..statuses.len()).filter(|index| seen.insert(VerdictEncoding::Original.encode(&statuses[*index]))).collect()
    };
    let per_vector_last_canary_states: Vec<usize> = {
        let mut seen = BTreeSet::new();
        let mut last: Vec<usize> = (0..statuses.len()).rev().filter(|index| seen.insert(VerdictEncoding::Original.encode(&statuses[*index]))).collect();
        last.reverse();
        last
    };
    output.line(format!(
        "name=r2_b8_canaries every_eighth_state={} one_state_per_stored_vector={per_vector_canary_states:?} last_occurrence={per_vector_last_canary_states:?} stored_vectors={:?}",
        canary_states.len(),
        per_vector_canary_states.iter().map(|index| VerdictEncoding::Original.encode(&statuses[*index])).collect::<Vec<_>>()
    ));
    for changed in [VerdictEncoding::RecordsSplitIntoTwoBits, VerdictEncoding::SameFieldNamesNewMeaning] {
        for placement in [
            VersionPlacement::NoVersion,
            VersionPlacement::LibraryLayoutVersion,
            VersionPlacement::DefinitionVersionInKeyBumped,
            VersionPlacement::DefinitionVersionInKeyForgotten,
            VersionPlacement::SchemaHashInKey,
            VersionPlacement::CanaryStatesRejudgedAtOpen,
            VersionPlacement::CanaryOneStatePerStoredVector,
            VersionPlacement::CanaryOneStatePerStoredVectorLastOccurrence,
            VersionPlacement::RawStatusesDerivedAtRead,
        ] {
            if placement == VersionPlacement::RawStatusesDerivedAtRead {
                let directory = scratch_library("b8-raw");
                let mut store = VerdictStore::create_empty(&directory).expect("建库");
                let serialized: Vec<Vec<u8>> = statuses.iter().map(|status| serialized_status(status).into_bytes()).collect();
                store_blocks_of(&mut store, &serialized, &(0..serialized.len().div_ceil(D1_BLOCK_STATES)).collect::<Vec<_>>(), D1_BLOCK_STATES, "stream");
                drop(store);
                let (store, _) = VerdictStore::open_existing(&directory).expect("开库");
                let mut misread = 0usize;
                for block_index in 0..serialized.len().div_ceil(D1_BLOCK_STATES) {
                    let first = block_index * D1_BLOCK_STATES;
                    let decoded = decoded_block(&store, &d1_key("stream", u64::try_from(first).expect("起点"))).expect("块在");
                    for (offset, bytes) in decoded.iter().enumerate() {
                        let derived = changed.encode(&parsed_status(std::str::from_utf8(bytes).expect("UTF-8")));
                        misread += usize::from(derived != changed.encode(&statuses[first + offset]));
                    }
                }
                drop(store);
                std::fs::remove_dir_all(&directory).expect("删库");
                output.line(format!(
                    "name=r2_b8_version world={changed:?} placement={placement:?} states={} old_blocks_found=true refused_at_open=false states_read_with_the_wrong_meaning={misread} must_be_nonzero=-",
                    statuses.len()
                ));
                continue;
            }
            let directory = scratch_library("b8");
            // 写的一方：旧定义（Original）。
            {
                let mut store = VerdictStore::create_empty(&directory).expect("建库");
                let original: Vec<u8> = statuses.iter().map(|status| VerdictEncoding::Original.encode(status)).collect();
                let name = match placement {
                    VersionPlacement::NoVersion | VersionPlacement::LibraryLayoutVersion | VersionPlacement::CanaryStatesRejudgedAtOpen | VersionPlacement::CanaryOneStatePerStoredVector | VersionPlacement::CanaryOneStatePerStoredVectorLastOccurrence | VersionPlacement::RawStatusesDerivedAtRead => "stream".to_string(),
                    VersionPlacement::DefinitionVersionInKeyBumped | VersionPlacement::DefinitionVersionInKeyForgotten => "encoding1/stream".to_string(),
                    VersionPlacement::SchemaHashInKey => format!("{}/stream", VerdictEncoding::Original.schema_hash()),
                };
                store_single_byte_blocks(&mut store, &name, &original);
                if placement == VersionPlacement::LibraryLayoutVersion {
                    store_single_byte_blocks(&mut store, "__library_layout", b"1");
                }
                if placement == VersionPlacement::CanaryStatesRejudgedAtOpen {
                    let canary: Vec<u8> = canary_states.iter().map(|index| original[*index]).collect();
                    store_single_byte_blocks(&mut store, "__canary", &canary);
                }
                if placement == VersionPlacement::CanaryOneStatePerStoredVector {
                    let canary: Vec<u8> = per_vector_canary_states.iter().map(|index| original[*index]).collect();
                    store_single_byte_blocks(&mut store, "__canary", &canary);
                }
                if placement == VersionPlacement::CanaryOneStatePerStoredVectorLastOccurrence {
                    let canary: Vec<u8> = per_vector_last_canary_states.iter().map(|index| original[*index]).collect();
                    store_single_byte_blocks(&mut store, "__canary", &canary);
                }
            }
            // 读的一方：新定义。
            let (store, _) = VerdictStore::open_existing(&directory).expect("开库");
            let truth: Vec<u8> = statuses.iter().map(|status| changed.encode(status)).collect();
            let refused_by_library_version = placement == VersionPlacement::LibraryLayoutVersion
                && read_single_byte_blocks(&store, "__library_layout", 1).as_deref() != Some(b"1".as_slice());
            let canary_of_this_placement: &[usize] = match placement {
                VersionPlacement::CanaryStatesRejudgedAtOpen => &canary_states,
                VersionPlacement::CanaryOneStatePerStoredVector => &per_vector_canary_states,
                VersionPlacement::CanaryOneStatePerStoredVectorLastOccurrence => &per_vector_last_canary_states,
                _ => &[],
            };
            let refused_by_canary = !canary_of_this_placement.is_empty() && {
                let stored = read_single_byte_blocks(&store, "__canary", canary_of_this_placement.len()).expect("金丝雀块在");
                canary_of_this_placement.iter().zip(&stored).any(|(index, byte)| truth[*index] != *byte)
            };
            let lookup = match placement {
                VersionPlacement::NoVersion | VersionPlacement::LibraryLayoutVersion | VersionPlacement::CanaryStatesRejudgedAtOpen | VersionPlacement::CanaryOneStatePerStoredVector | VersionPlacement::CanaryOneStatePerStoredVectorLastOccurrence | VersionPlacement::RawStatusesDerivedAtRead => "stream".to_string(),
                VersionPlacement::DefinitionVersionInKeyBumped => "encoding2/stream".to_string(),
                VersionPlacement::DefinitionVersionInKeyForgotten => "encoding1/stream".to_string(),
                VersionPlacement::SchemaHashInKey => format!("{}/stream", changed.schema_hash()),
            };
            let found = if refused_by_library_version || refused_by_canary { None } else { read_single_byte_blocks(&store, &lookup, statuses.len()) };
            let misread = found.as_ref().map_or(0, |stored| stored.iter().zip(&truth).filter(|(stored, truth)| stored != truth).count());
            drop(store);
            std::fs::remove_dir_all(&directory).expect("删库");
            let caught = placement == VersionPlacement::DefinitionVersionInKeyBumped
                || (placement == VersionPlacement::SchemaHashInKey && changed == VerdictEncoding::RecordsSplitIntoTwoBits);
            output.line(format!(
                "name=r2_b8_version world={changed:?} placement={placement:?} states={} old_blocks_found={} refused_at_open={} states_read_with_the_wrong_meaning={misread} must_be_nonzero={}",
                statuses.len(),
                found.is_some(),
                refused_by_library_version || refused_by_canary,
                if caught { "-".to_string() } else { misread.to_string() }
            ));
        }
    }
    0
}

// ───────────────────────────── D1：两台机器录入之后与单机逐项相同 ─────────────────────────────

const D1_BLOCK_STATES: usize = 16;

fn d1_key(name: &str, start: u64) -> VerdictBlockKey {
    b8_block_key(name, start)
}

/// 一台机器把它分到的块按块序存进自己的库（判定向量按它自己第一次见到的次序登记）。
fn store_blocks_of(store: &mut VerdictStore, vectors: &[Vec<u8>], block_indexes: &[usize], block_states: usize, name: &str) {
    for block_index in block_indexes {
        let first = block_index * block_states;
        let chunk = &vectors[first..(first + block_states).min(vectors.len())];
        let numbers: Vec<VerdictVectorNumber> = chunk
            .iter()
            .map(|vector| store.register_verdict_vector(&EncodedVerdictVector(vector.clone())).expect("登记"))
            .collect();
        store
            .store_verdict_block(&d1_key(name, u64::try_from(first).expect("起点")), &VerdictBlock::new(numbers).expect("非空"), &[])
            .expect("存块");
    }
}

fn decoded_block(store: &VerdictStore, key: &VerdictBlockKey) -> Option<Vec<Vec<u8>>> {
    let block = store.read_verdict_block(key).expect("读块")?;
    Some(
        block
            .verdict_vector_number_of_each_state()
            .iter()
            .map(|number| store.verdict_vector_of_number(*number).expect("登记过").0.clone())
            .collect(),
    )
}

fn world_d1_two_machines(output: &mut ResultLines) -> i32 {
    let statuses = source_statuses(output);
    let vectors: Vec<Vec<u8>> = statuses.iter().map(|status| format!("{status:?}").into_bytes()).collect();
    let block_count = vectors.len().div_ceil(D1_BLOCK_STATES);
    let all_blocks: Vec<usize> = (0..block_count).collect();
    let machine_a_blocks: Vec<usize> = all_blocks.iter().copied().filter(|index| index % 2 == 0).collect();
    let machine_b_blocks: Vec<usize> = all_blocks.iter().copied().filter(|index| index % 2 == 1).collect();
    // （一）各写一份、按块导入：原样搬编号 vs 按内容换编号。
    for translate in [false, true] {
        let directory_a = scratch_library("d1-a");
        let directory_b = scratch_library("d1-b");
        let mut store_a = VerdictStore::create_empty(&directory_a).expect("建库");
        let mut store_b = VerdictStore::create_empty(&directory_b).expect("建库");
        store_blocks_of(&mut store_a, &vectors, &machine_a_blocks, D1_BLOCK_STATES, "stream");
        store_blocks_of(&mut store_b, &vectors, &machine_b_blocks, D1_BLOCK_STATES, "stream");
        let mut imported_silently = 0usize;
        let mut refused_loudly = 0usize;
        for block_index in &machine_b_blocks {
            let key = d1_key("stream", u64::try_from(block_index * D1_BLOCK_STATES).expect("起点"));
            let block_in_b = store_b.read_verdict_block(&key).expect("读").expect("B 有这一块");
            let block_for_a = if translate {
                let numbers: Vec<VerdictVectorNumber> = block_in_b
                    .verdict_vector_number_of_each_state()
                    .iter()
                    .map(|number| {
                        let vector = store_b.verdict_vector_of_number(*number).expect("B 登记过").clone();
                        store_a.register_verdict_vector(&vector).expect("A 登记")
                    })
                    .collect();
                VerdictBlock::new(numbers).expect("非空")
            } else {
                block_in_b
            };
            let stored = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| store_a.store_verdict_block(&key, &block_for_a, &[])));
            match stored {
                Ok(Ok(_)) => imported_silently += 1,
                Ok(Err(_)) | Err(_) => refused_loudly += 1,
            }
        }
        let mut states_decoded_differently = 0usize;
        let mut states_missing = 0usize;
        for block_index in &all_blocks {
            let first = block_index * D1_BLOCK_STATES;
            match decoded_block(&store_a, &d1_key("stream", u64::try_from(first).expect("起点"))) {
                Some(decoded) => {
                    states_decoded_differently += decoded.iter().zip(&vectors[first..]).filter(|(left, right)| left != right).count();
                }
                None => states_missing += (first + D1_BLOCK_STATES).min(vectors.len()) - first,
            }
        }
        drop(store_a);
        drop(store_b);
        std::fs::remove_dir_all(&directory_a).expect("删库");
        std::fs::remove_dir_all(&directory_b).expect("删库");
        output.line(format!(
            "name=r2_d1_import mode={} states={} blocks_a={} blocks_b={} imported_silently={imported_silently} refused_loudly={refused_loudly} merged_states_decoded_differently_from_single_machine={states_decoded_differently} merged_states_missing={states_missing} must_be_nonzero={}",
            if translate { "translate_numbers_by_content" } else { "copy_block_bytes_as_they_are" },
            vectors.len(),
            machine_a_blocks.len(),
            machine_b_blocks.len(),
            if translate { "-".to_string() } else { states_decoded_differently.to_string() }
        ));
    }
    // （二）被杀之后换了块长续跑：块长不进键，起点不同的重叠块全收下。
    {
        let directory = scratch_library("d1-overlap");
        let mut store = VerdictStore::create_empty(&directory).expect("建库");
        let killed_after_blocks = 3usize;
        store_blocks_of(&mut store, &vectors, &(0..killed_after_blocks).collect::<Vec<_>>(), D1_BLOCK_STATES, "stream");
        let resumed_from = 40usize;
        let resumed_block_states = 24usize;
        let mut resumed_starts = Vec::new();
        let mut start = resumed_from;
        let mut refused = 0usize;
        while start < vectors.len() {
            let chunk = &vectors[start..(start + resumed_block_states).min(vectors.len())];
            let numbers: Vec<VerdictVectorNumber> = chunk.iter().map(|vector| store.register_verdict_vector(&EncodedVerdictVector(vector.clone())).expect("登记")).collect();
            match store.store_verdict_block(&d1_key("stream", u64::try_from(start).expect("起点")), &VerdictBlock::new(numbers).expect("非空"), &[]) {
                Ok(_) => resumed_starts.push(start),
                Err(_) => refused += 1,
            }
            start += resumed_block_states;
        }
        let mut coverage = vec![0u32; vectors.len()];
        let mut stored_starts: Vec<usize> = (0..killed_after_blocks).map(|index| index * D1_BLOCK_STATES).collect();
        stored_starts.extend(resumed_starts.iter().copied());
        for block_start in &stored_starts {
            let decoded = decoded_block(&store, &d1_key("stream", u64::try_from(*block_start).expect("起点"))).expect("存过");
            for offset in 0..decoded.len() {
                coverage[block_start + offset] += 1;
            }
        }
        let covered_twice = coverage.iter().filter(|count| **count >= 2).count();
        let not_covered = coverage.iter().filter(|count| **count == 0).count();
        let tally_over_blocks: u32 = coverage.iter().sum();
        drop(store);
        std::fs::remove_dir_all(&directory).expect("删库");
        output.line(format!(
            "name=r2_d1_resume_with_another_block_length states={} first_run_blocks_of={D1_BLOCK_STATES}x{killed_after_blocks} resumed_from={resumed_from} resumed_block_states={resumed_block_states} resumed_blocks_accepted={} refused={refused} states_covered_by_two_blocks={covered_twice} states_not_covered={not_covered} states_counted_by_a_tally_over_stored_blocks={tally_over_blocks} must_be_nonzero={covered_twice}",
            vectors.len(),
            resumed_starts.len()
        ));
    }
    // （三）计划哈希（块键里的「枚举计划哈希」若取层 0 今天的那一份）随线程数、续跑与否、分片、观察者变。
    {
        let first = first_history();
        let root = first.judged_root_index();
        let mut hashes: Vec<(String, String, u64, usize)> = Vec::new();
        let shard = |index: u32| Some(Layer0ShardOfShards::new(index, NonZeroU32::new(2).expect("2")).expect("片号"));
        for (label, threads, resumable, observer, shard_of) in [
            ("single_machine_4_threads", 4usize, false, false, None),
            ("single_machine_32_threads", 32, false, false, None),
            ("resumable_4_threads", 4, true, false, None),
            ("resumable_32_threads", 32, true, false, None),
            ("shard_0_of_2", 16, true, false, shard(0)),
            ("shard_1_of_2", 16, true, false, shard(1)),
            ("resumable_with_observer", 4, true, true, None),
        ] {
            let (hash, states, slices) = attack_layer0_plan_hash(&first.base, &first.writes, &first.segments, root, &first.versions, &|_, _| true, parallelism_with_threads(threads), resumable, observer, shard_of);
            hashes.push((label.to_string(), hash[..16].to_string(), states, slices));
        }
        let distinct: BTreeSet<&String> = hashes.iter().map(|(_, hash, _, _)| hash).collect();
        output.line(format!(
            "name=r2_d1_plan_hash runs={} distinct_plan_hashes={} detail={hashes:?} must_be_nonzero={}",
            hashes.len(),
            distinct.len(),
            distinct.len() - 1
        ));
    }
    // （四）续跑的片（块的单位之一）跨节点：两条流全量展开时，跨段的片有几个。
    {
        // 只算第一条流：E161 的 `prepare_second_stream` 经 `common::build_pool` 在盘上建镜像文件，定义 3b 不许。
        let first_prepared = first_history().prepared();
        for (label, stream) in [("first_stream", &first_prepared)] {
            let mut ranges = Vec::new();
            let mut next = 0u64;
            for segment_index in 0..stream.segments.len() {
                let count = layer0_state_count_with_torn_in_place_overwrites(&stream.base, &stream.writes, &stream.segments, &|index, _| {
                    if index == segment_index { Layer0SegmentExpansion::EveryProperSubset } else { Layer0SegmentExpansion::NotExpanded }
                }) - 1;
                ranges.push(next..next + count);
                next += count;
            }
            let total = next + 1;
            let per_slice = total.div_ceil(65_536).max(16);
            let slices = total.div_ceil(per_slice);
            let spanning = (0..slices)
                .filter(|slice| {
                    let start = slice * per_slice;
                    let end = (start + per_slice).min(total);
                    ranges.iter().filter(|range| range.start < end && start < range.end).count() >= 2
                })
                .count();
            let nodes_not_starting_on_a_slice_boundary = ranges.iter().filter(|range| range.start < range.end && range.start % per_slice != 0).count();
            output.line(format!(
                "name=r2_d1_slices_across_nodes stream={label} segments={} states={total} states_per_slice={per_slice} slices={slices} slices_spanning_two_or_more_segments={spanning} segments_not_starting_on_a_slice_boundary={nodes_not_starting_on_a_slice_boundary} must_be_nonzero={spanning}",
                stream.segments.len()
            ));
        }
    }
    0
}

// ───────────────────────────── 第一轮 kinds 世界在最强版本的覆盖签名下 ─────────────────────────────

fn world_kinds_under_the_overlap_signature(output: &mut ResultLines, count_argument: Option<&String>) -> i32 {
    let overwrites: usize = count_argument.map_or(30, |text| text.parse().expect("次数"));
    let history = many_overwrites(overwrites);
    let mut previous_root_segment: Option<usize> = None;
    let mut conditions_by_kind: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut conditions_by_kind_and_signature: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut publishes = 0usize;
    for (segment_index, segment) in history.segments.iter().enumerate() {
        if !segment.iter().any(|index| history.writes[*index].kind == StepKind::RootRecordFua) {
            continue;
        }
        let first = previous_root_segment.map_or(0, |previous| previous + 1);
        let last = (segment_index + 1).min(history.segments.len() - 1);
        let this_publish: Vec<usize> = history.segments[first..=segment_index].iter().flatten().copied().collect();
        let kinds: String = history.segments[first..=last]
            .iter()
            .map(|segment| segment.iter().map(|index| kind_letter(history.writes[*index].kind)).collect::<String>())
            .collect::<Vec<_>>()
            .join("|");
        let signature: String = history.segments[first..=last]
            .iter()
            .flatten()
            .map(|index| overlap_signature(&history.writes, *index))
            .collect::<Vec<_>>()
            .join(";");
        let earlier_end = this_publish.iter().min().copied().unwrap_or(0);
        let earlier = &history.writes[..earlier_end];
        let condition = |kind: StepKind| this_publish.iter().any(|index| {
            let write = &history.writes[*index];
            write.kind == kind && earlier.iter().any(|old| old.kind == kind && writes_overlap(old, write))
        });
        let condition_text = format!(
            "reuse={},root_slot_rewrite={},journal_slot_rewrite={}",
            condition(StepKind::UnitWrite),
            condition(StepKind::RootRecordFua),
            condition(StepKind::JournalRecord)
        );
        conditions_by_kind.entry(kinds.clone()).or_default().insert(condition_text.clone());
        conditions_by_kind_and_signature.entry(format!("{kinds}#{signature}")).or_default().insert(condition_text);
        publishes += 1;
        previous_root_segment = Some(segment_index);
    }
    let merged_by_kind = conditions_by_kind.values().filter(|conditions| conditions.len() > 1).count();
    let merged_by_signature = conditions_by_kind_and_signature.values().filter(|conditions| conditions.len() > 1).count();
    output.line(format!(
        "name=r2_kinds_regression overwrites={overwrites} publishes={publishes} identities_by_kind_string={} kind_strings_merging_conditions={merged_by_kind} identities_by_kind_string_plus_signature={} identities_merging_conditions={merged_by_signature} must_be_nonzero=-",
        conditions_by_kind.len(),
        conditions_by_kind_and_signature.len()
    ));
    0
}


// ───────────────────────────── B4 再攻：P7 键下违例正文不同 ─────────────────────────────

/// 码 2 节点改成写序实例代号 7（大于系统配置里的实例代号），树 ID 与诞生代号不动，头校验和重算：I-7.7 会判违反，I-7.8 不受影响。
fn node_with_write_order_instance(node: &[u8], instance: u32) -> Vec<u8> {
    let mut changed = node.to_vec();
    assert_eq!(&changed[..4], b"SFSU");
    assert_eq!(changed[6], 2);
    let key_count = usize::from(changed[51]);
    let header_end = 86 + 2 * key_count;
    let instance_offset = 68 + 2 * key_count;
    changed[instance_offset..instance_offset + 4].copy_from_slice(&instance.to_le_bytes());
    changed[10..42].fill(0);
    let checksum = singlefs_checker::crc32_castagnoli_bitwise(&changed[..header_end]);
    changed[10..14].copy_from_slice(&checksum.to_le_bytes());
    changed
}

fn world_b4_p7_key_and_violation_texts(output: &mut ResultLines) -> i32 {
    let full = first_history();
    let prefix = full.prefix_of_segments(7);
    let node = full.segments[7]
        .iter()
        .map(|index| &full.writes[*index])
        .find(|write| write.device == DeviceIdentity(0) && write.length_in_bytes() == LOCAL_NODE_BYTES && write.bytes().is_some_and(|bytes| bytes[6] == 2))
        .expect("码 2 节点")
        .bytes()
        .expect("字节")
        .to_vec();
    let changed = node_with_write_order_instance(&node, 7);
    let mut history = prefix.clone();
    let start = history.writes.len();
    history.writes.push(unit_write(0, 60_000, changed.clone()));
    history.writes.push(unit_write(0, 60_002, changed));
    for index in full.segments[7].iter().take(4) {
        history.writes.push(full.writes[*index].clone());
    }
    history.segments.push((start..history.writes.len()).collect());
    let unit_segment = history.segments.len() - 1;
    budget(output, "b4_two_nodes_with_a_high_instance", &history, &|_, _| true);
    let mut tally = KeyTally::default();
    let mut example: Option<String> = None;
    enumerate_observing(&history, &|_, _| true, &mut |image, _report| {
        if segment_of_state(&history.segments, &image.persisted) != unit_segment {
            return;
        }
        let first_only = image.persisted[start] && !image.persisted[start + 1];
        if first_only && example.is_none() {
            example = Some(format!("{:?}", verdict_of_image(image, &history.versions).red));
        }
        tally.add(image, &history.versions);
    });
    output.line(format!("{} example_red_with_only_the_first_node={example:?}", tally.line("two_nodes_with_a_high_instance_in_a_unit_segment")));
    0
}

// ───────────────────────────── B5 再攻：合法复用的段上，记录核对器的剪枝键 ─────────────────────────────

fn world_b5_record_key_under_legal_reuse(output: &mut ResultLines) -> i32 {
    let mut history = many_overwrites(30);
    let recorded = history.writes.len();
    // txg 3（新建文件）那次发布写下的单元副本：按写表次序归到 txg 3 那条根的单元写。
    let persisted_all = vec![true; recorded];
    let txg3 = publishes_in(&history.writes, &persisted_all, RecordStreamContinuity::OneRecording)
        .into_iter()
        .find(|publish| publish.checkpoint_txg == 3)
        .expect("txg 3 那次发布");
    let copies: Vec<usize> = txg3.units.iter().copied().take(12).collect();
    let start = history.writes.len();
    for copy in &copies {
        let old = history.writes[*copy].clone();
        let length = usize::try_from(old.length_in_bytes()).expect("长");
        history.writes.push(RetainedWrite { contents: WrittenContents::Bytes(vec![0x33; length]), ..old });
    }
    history.segments.push((start..history.writes.len()).collect());
    let reuse_segment = history.segments.len() - 1;
    let legal: Vec<bool> = copies.iter().enumerate().map(|(offset, copy)| attack_later_write_passes_the_reclaim_predicate(&history.writes, *copy, start + offset)).collect();
    budget(output, "b5_legal_reuse_segment", &history, &|index, _| index == reuse_segment);
    let mut pairs_all_overlapping: Vec<(Digest128, (bool, bool))> = Vec::new();
    let mut pairs_only_failing_the_predicate: Vec<(Digest128, (bool, bool))> = Vec::new();
    let mut pairs_symbolic: Vec<(Digest128, (bool, bool))> = Vec::new();
    let started = Instant::now();
    enumerate_observing(&history, &|index, _| index == reuse_segment, &mut |image, _report| {
        if segment_of_state(&history.segments, &image.persisted) != reuse_segment {
            return;
        }
        let consult = recover(image, JournalPolicy::Consult);
        let check = singlefs_checker_tier::crash::check_records(image, consult.effective_root);
        let verdict = (check.root_without_record, check.claimed_state_missing_unit);
        let key_all = record_checker_key(image, consult.effective_root);
        let tables = RefCell::new(ContentTables::default());
        let reader = RecordingReader::new(image, RecordedPass::Recovery, 0, &tables);
        let _ = singlefs_checker_tier::crash::check_records_against(&reader, &reader, image.writes, &image.persisted, RecordStreamContinuity::OneRecording, consult.effective_root);
        let calls: Vec<LoggedCall> = reader.calls.borrow().clone();
        let failing: Vec<(usize, bool)> = (0..copies.len()).filter(|offset| !legal[*offset]).map(|offset| (start + offset, image.persisted[start + offset])).collect();
        let key_failing = pair_digest(sequence_key(&calls), fingerprint_of_debug_text(&(consult.effective_root, failing)));
        pairs_all_overlapping.push((key_all, verdict));
        pairs_only_failing_the_predicate.push((key_failing, verdict));
        pairs_symbolic.push((record_checker_symbolic_key(image, consult.effective_root), verdict));
    });
    let (classes_symbolic, inconsistent_symbolic) = inconsistent_by(&pairs_symbolic);
    let (classes_all, inconsistent_all) = inconsistent_by(&pairs_all_overlapping);
    let (classes_failing, inconsistent_failing) = inconsistent_by(&pairs_only_failing_the_predicate);
    let distinct_verdicts: BTreeSet<(bool, bool)> = pairs_all_overlapping.iter().map(|(_, verdict)| *verdict).collect();
    output.line(format!(
        "name=r2_b5_legal_reuse overwrites=30 reused_copies_of_txg3={} later_writes_passing_the_reclaim_predicate={} states={} distinct_record_verdicts={} key_with_every_overlapping_later_write_classes={classes_all} inconsistent={inconsistent_all} key_with_only_writes_failing_the_predicate_classes={classes_failing} inconsistent={inconsistent_failing} symbolic_key_classes={classes_symbolic} symbolic_inconsistent={inconsistent_symbolic} seconds={:.1} must_be_nonzero={}",
        copies.len(),
        legal.iter().filter(|passes| **passes).count(),
        pairs_all_overlapping.len(),
        distinct_verdicts.len(),
        started.elapsed().as_secs_f64(),
        classes_all.saturating_sub(distinct_verdicts.len())
    ));
    0
}


// ───────────────────────────── B5：录下来的真实现（健康编译与「先根后记录」编译各跑一次） ─────────────────────────────

/// 第一条流按今天编进来的真实现录一遍（`build_first_stream_in_memory`），小段全部：数「根在盘上、按记录字节认出的它自己的记录一条都不在盘上」的状态（真洞），
/// 与今天的第一条判据、按自己身份归属的判据、五样判定里任一样红各几个。同一份原型在健康编译与把 `persist_publish_writes` 改成先根后记录的编译上各跑一次。
fn world_b5_recorded_implementation(output: &mut ResultLines) -> i32 {
    let history = first_history();
    let ownership = ownership_table(&history.writes, history.writes.len());
    let shape: Vec<String> = history
        .segments
        .iter()
        .map(|segment| format!("{}x{}", segment.len(), kind_letter(history.writes[segment[0]].kind)))
        .collect();
    let ownership_text: Vec<String> = ownership
        .iter()
        .map(|(root, identity, by_order, own)| format!("root{root}{identity:?}:order={by_order:?}/own={own:?}"))
        .collect();
    budget(output, "b5_recorded_implementation", &history, &small);
    let mut states = 0u64;
    let mut true_holes = 0u64;
    let mut red_today = 0u64;
    let mut red_own = 0u64;
    let mut true_holes_red_by_any_judgment = 0u64;
    let mut red_own_that_is_not_a_true_hole = 0u64;
    enumerate_observing(&history, &small, &mut |image, _report| {
        states += 1;
        let is_true_hole = ownership
            .iter()
            .any(|(root, _, _, own)| !own.is_empty() && image.persisted[*root] && own.iter().all(|record| !image.persisted[*record]));
        let verdict = verdict_of_image(image, &history.versions);
        let own = root_without_own_record(image, &ownership);
        true_holes += u64::from(is_true_hole);
        red_today += u64::from(verdict.status.root_without_record);
        red_own += u64::from(own);
        true_holes_red_by_any_judgment += u64::from(is_true_hole && !verdict.red.is_empty());
        red_own_that_is_not_a_true_hole += u64::from(own && !is_true_hole);
    });
    output.line(format!(
        "name=r2_b5_implementation segments={shape:?} ownership={ownership_text:?} states={states} true_holes_by_persisted_bits={true_holes} of_those_red_by_any_todays_judgment={true_holes_red_by_any_judgment} root_without_record_today={red_today} root_without_own_record={red_own} own_red_that_is_not_a_true_hole={red_own_that_is_not_a_true_hole} must_be_nonzero={}",
        true_holes.saturating_sub(true_holes_red_by_any_judgment)
    ));
    0
}

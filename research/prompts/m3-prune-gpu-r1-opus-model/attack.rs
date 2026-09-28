//! m3-prune-gpu-r1 云端攻方腿的原型（只在草稿副本里，不入库）：造「必须报非 0」的世界。
//! 每个子命令打一组 `name=attack_*` 行；`must_be_nonzero=` 字段是这个世界要报非 0 的那个数。
//! 历史一律在内存稀疏盘（`SparseBlockDevice`）上录，不建镜像文件；起点池建一次、之后拷内存。

use super::*;

use singlefs_checker_tier::crash::{
    classified_oracle_violation_for_versions,
    enumerate_layer0_selecting_versions_observing_each_state,
    layer0_state_count_with_torn_in_place_overwrites, Layer0SegmentExpansion,
    TearableInPlaceOverwrites,
};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::{make_filesystem, INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_overwrite, warm_up, FirstFile, PoolWriter,
    TransactionOutput,
};
use singlefs_harness::memory_pool::{closed_form_state_count, SparseBlockDevice};
use singlefs_harness::{RecordingBlockDevice, SharedStream};

pub(super) fn run(output: &mut ResultLines, arguments: &[String]) -> i32 {
    match arguments.first().map(String::as_str) {
        Some("shape") => shape(output),
        Some("p5") => world_p5_mutant_node_inside_a_unit_only_segment(output),
        Some("p6") => world_p6_leaves_equal_states_on_a_unit_segment(output, arguments.get(1)),
        Some("t4") => world_t4_same_parent_image_different_domain(output),
        Some("torn") => world_torn_convention_two_writes_same_slot(output),
        Some("small") => world_small_domain_g2_p4_u4(output),
        Some("order") => world_u4_materialized_record_stream_misses_root_before_records(output),
        Some("interior") => world_t2_data_unit_interior_scanned_only_when_materialized(output),
        Some("kinds") => world_kind_string_identity_merges_conditions(output, arguments.get(1)),
        Some("collide") => world_digest_collision(output),
        Some("reuse") => world_u4_materialized_record_stream_misses_premature_reuse(output),
        _ => {
            eprintln!("用法：attack shape|p5|p6 [k]|t4|torn|small|order");
            2
        }
    }
}

// ───────────────────────────── 内存池上录历史 ─────────────────────────────

type MemoryRecorded = RecordingBlockDevice<SparseBlockDevice>;

/// 与 `common::build_pool` 同一串调用（mkfs → 取号 → 暖机 → 新池新建文件），盘换成内存稀疏盘。
struct InMemoryPool {
    devices: Vec<(DeviceIdentity, MemoryRecorded)>,
    stream: SharedStream,
    allocator: PoolAllocator,
    output: TransactionOutput,
    mkfs_operation_count: usize,
}

fn build_first_stream_in_memory(content: &[u8]) -> InMemoryPool {
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
        Placement {
            slot: INSTANCE_TABLE_SLOT,
            span: 2,
        },
        Placement {
            slot: TREE_TABLE_GENESIS_SLOT,
            span: 1,
        },
    );
    let output = {
        let mut pool = PoolWriter::new(&parameters, &mut devices);
        let instance = acquire_instance(&mut pool).expect("取号");
        let warm = warm_up(&mut pool, &genesis.root, instance).expect("暖机");
        publish_first_file(
            &mut pool,
            &mut allocator,
            warm.roots.last().expect("暖机两代根"),
            FirstFile {
                content,
                write_time_seconds: common::FIXED_WRITE_TIME_SECONDS,
            },
            instance,
            &warm.last_record_bytes,
        )
        .expect("新池新建文件")
    };
    InMemoryPool {
        devices,
        stream,
        allocator,
        output,
        mkfs_operation_count,
    }
}

fn overwrite_in_memory(pool: &mut InMemoryPool, content: &[u8]) {
    let parameters = common::parameters();
    let mut writer = PoolWriter::new(&parameters, pool.devices.as_mut_slice());
    pool.output = publish_overwrite(
        &mut writer,
        &mut pool.allocator,
        &pool.output,
        FirstFile {
            content,
            write_time_seconds: common::FIXED_WRITE_TIME_SECONDS + 60,
        },
        InstanceGeneration(1),
    )
    .expect("覆盖写");
}

/// 一段历史：mkfs 之后的基线、写表、段、版本表。
#[derive(Clone)]
struct History {
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    versions: Vec<PublishedVersion>,
}

impl History {
    fn of_pool(pool: &InMemoryPool, versions: Vec<PublishedVersion>) -> Self {
        let operations = pool.stream.retained_operations();
        let mut base = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], common::IMAGE_BYTES);
        base.apply(&operations[..pool.mkfs_operation_count]);
        let (writes, segments) = singlefs_harness::memory_pool::writes_and_segments(
            &operations[pool.mkfs_operation_count..],
            &common::geometry(),
        );
        Self {
            base,
            writes,
            segments,
            versions,
        }
    }

    fn judged_root_index(&self) -> usize {
        self.writes
            .iter()
            .rposition(|write| write.kind == StepKind::RootRecordFua)
            .expect("写表里至少一条根槽写")
    }

    fn prepared(&self) -> PreparedStream {
        PreparedStream {
            name: StreamName::First,
            base: self.base.clone(),
            writes: self.writes.clone(),
            segments: self.segments.clone(),
            judged_root_index: self.judged_root_index(),
            versions: self.versions.clone(),
            root_write_count: self
                .writes
                .iter()
                .filter(|write| write.kind == StepKind::RootRecordFua)
                .count(),
        }
    }

    fn torn_state_count(&self, expanded: &dyn Fn(usize, &[usize]) -> bool) -> u64 {
        layer0_state_count_with_torn_in_place_overwrites(
            &self.base,
            &self.writes,
            &self.segments,
            &|index, segment| {
                if expanded(index, segment) {
                    Layer0SegmentExpansion::EveryProperSubset
                } else {
                    Layer0SegmentExpansion::NotExpanded
                }
            },
        )
    }

    /// 前 `segment_count` 段组成的录制流前缀（后面的写整个去掉：录制流截短，是「缩历史长度」）。
    fn prefix_of_segments(&self, segment_count: usize) -> Self {
        let kept: Vec<usize> = self.segments[..segment_count].iter().flatten().copied().collect();
        let last = kept.iter().max().copied().expect("至少一次写");
        assert_eq!(kept.len(), last + 1, "段是按写表次序连着的");
        Self {
            base: self.base.clone(),
            writes: self.writes[..=last].to_vec(),
            segments: self.segments[..segment_count].to_vec(),
            versions: self.versions.clone(),
        }
    }
}

fn first_versions(pool: &InMemoryPool, history_writes: &[RetainedWrite], content: &[u8]) -> Vec<PublishedVersion> {
    let _ = pool;
    let root_index = history_writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("根槽写");
    let (instance, checkpoint_txg) = root_identity_of_root_write(&history_writes[root_index]);
    vec![PublishedVersion {
        instance,
        checkpoint_txg,
        content: content.to_vec(),
    }]
}

fn first_history() -> History {
    let content = common::file_content();
    let pool = build_first_stream_in_memory(&content);
    let mut history = History::of_pool(&pool, Vec::new());
    history.versions = first_versions(&pool, &history.writes, &content);
    history
}

// ───────────────────────────── 一个状态上的全部判定 ─────────────────────────────

/// 一个状态的全部判定（层 0 在每个状态上跑的五样：两遍恢复、两遍 oracle、池级 checker、记录核对器），与哪几样报红。
struct Judgment {
    digest: Digest128,
    red: Vec<String>,
}

fn judge_state<Reader: PoolReader + ImageReader>(
    reader: &Reader,
    writes: &[RetainedWrite],
    persisted: &[bool],
    versions: &[PublishedVersion],
) -> Judgment {
    let newest = newest_persisted_root(writes, persisted);
    let consulted = recover(reader, JournalPolicy::Consult);
    let ignored = recover(reader, JournalPolicy::Ignore);
    let oracle_consulted = classified_oracle_violation_for_versions(
        &consulted.outcome,
        consulted.effective_root,
        newest,
        versions,
    );
    let oracle_ignored = classified_oracle_violation_for_versions(
        &ignored.outcome,
        ignored.effective_root,
        newest,
        versions,
    );
    let verdicts = check_pool_image(reader);
    let records = singlefs_checker_tier::crash::check_records_against(
        reader,
        reader,
        writes,
        persisted,
        singlefs_harness::memory_pool::RecordStreamContinuity::OneRecording,
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
    let oracle_text = format!("{:?}|{:?}", oracle_consulted.map(|v| v.reason), oracle_ignored.map(|v| v.reason));
    let digest = fingerprint_of_debug_text(&(
        format!("{consulted:?}"),
        format!("{ignored:?}"),
        oracle_text,
        format!("{verdicts:?}"),
        format!("{records:?}"),
    ));
    Judgment { digest, red }
}

fn judge_image(image: &CrashImage<'_>, versions: &[PublishedVersion]) -> Judgment {
    judge_state(image, image.writes, &image.persisted, versions)
}

/// 一个状态落在哪一段：第一段里有一次录制流写没持久的那一段；全持久那一个交段数。
fn segment_of_state(segments: &[Vec<usize>], persisted: &[bool]) -> usize {
    segments
        .iter()
        .position(|segment| segment.iter().any(|write_index| !persisted[*write_index]))
        .unwrap_or(segments.len())
}

/// 跑前核状态数：两态闭式与带第三态的实数都不许超过上限（约 10⁶）。
fn guarded_state_count(output: &mut ResultLines, label: &str, history: &History, expanded: &dyn Fn(usize, &[usize]) -> bool) -> u64 {
    let closed = closed_form_state_count(&history.segments);
    let with_torn = history.torn_state_count(expanded);
    output.line(format!(
        "name=attack_state_budget world={label} closed_form_state_count={closed} enumerated_with_torn={with_torn} writes={} segments={}",
        history.writes.len(),
        history.segments.len()
    ));
    assert!(with_torn <= 1_000_000, "一次跑不超过约 10⁶ 个状态");
    with_torn
}

fn enumerate_observing(
    history: &History,
    expanded: &dyn Fn(usize, &[usize]) -> bool,
    observe: &mut dyn FnMut(&CrashImage<'_>, &RecoveryReport),
) -> Layer0Tally {
    enumerate_layer0_selecting_versions_observing_each_state(
        &history.base,
        &history.writes,
        &history.segments,
        history.judged_root_index(),
        &history.versions,
        expanded,
        observe,
    )
}

// ───────────────────────────── 形状 ─────────────────────────────

fn shape(output: &mut ResultLines) -> i32 {
    let history = first_history();
    let prepared = history.prepared();
    let table = TornWriteTable::of(&prepared);
    let lengths: Vec<usize> = history.segments.iter().map(Vec::len).collect();
    output.line(format!(
        "name=attack_shape stream=first_in_memory writes={} segments={} lengths={lengths:?} mask_width={} closed_form={} with_torn={}",
        history.writes.len(),
        history.segments.len(),
        table.mask_width(),
        closed_form_state_count(&history.segments),
        history.torn_state_count(&|_, _| true)
    ));
    0
}

// ───────────────────────────── 读集（位置与调用） ─────────────────────────────

/// 一个状态上恢复两遍与 checker 正常那一遍、走树那一遍各自的调用；按字节读的位置（盘、偏移、长度）去重。
struct StateReads {
    consult_calls: Vec<LoggedCall>,
    ignore_calls: Vec<LoggedCall>,
    normal_calls: Vec<LoggedCall>,
    walk_calls: Vec<LoggedCall>,
    byte_positions: Vec<(u32, u64, usize)>,
    candidate_unit_slots: Vec<Option<Vec<u64>>>,
}

fn state_reads(image: &CrashImage<'_>) -> StateReads {
    let tables = RefCell::new(ContentTables::default());
    let consult = RecordingReader::new(image, RecordedPass::Recovery, 0, &tables);
    let _ = recover(&consult, JournalPolicy::Consult);
    let ignore = RecordingReader::new(image, RecordedPass::Recovery, 0, &tables);
    let _ = recover(&ignore, JournalPolicy::Ignore);
    let normal = RecordingReader::new(image, RecordedPass::CheckerNormal, 0, &tables);
    let _ = check_pool_image(&normal);
    let walk = RecordingReader::new(image, RecordedPass::CheckerWalk, 0, &tables);
    let _ = check_pool_image(&walk);
    let mut positions = BTreeSet::new();
    for reader in [&consult, &ignore, &normal] {
        for read in reader.reads.borrow().iter() {
            positions.insert((read.device, read.offset, read.length));
        }
    }
    let candidate_unit_slots = [0u32, 1]
        .iter()
        .map(|device| ImageReader::candidate_unit_slots(image, *device))
        .collect();
    StateReads {
        consult_calls: consult.calls.into_inner(),
        ignore_calls: ignore.calls.into_inner(),
        normal_calls: normal.calls.into_inner(),
        walk_calls: walk.calls.into_inner(),
        byte_positions: positions.into_iter().collect(),
        candidate_unit_slots,
    }
}

fn distinct<T: Ord>(values: impl Iterator<Item = T>) -> usize {
    values.collect::<BTreeSet<T>>().len()
}

/// 按键分组，数「同键而判定与组里第一个不同」的状态数。
fn inconsistent_under_key(pairs: &[(Digest128, Digest128)]) -> (usize, usize) {
    let mut first_judgment_by_key: BTreeMap<Digest128, Digest128> = BTreeMap::new();
    let mut inconsistent = 0;
    for (key, judgment) in pairs {
        let first = first_judgment_by_key.entry(*key).or_insert(*judgment);
        if first != judgment {
            inconsistent += 1;
        }
    }
    (first_judgment_by_key.len(), inconsistent)
}

// ───────────────────────────── P5：单元段里改坏一个节点 ─────────────────────────────

/// 码 2 节点改成 I-7.8 扫描方向会数进去的孤儿：树 ID 抬到根环水位之上、诞生代号压到 1（不超过盘上最新根的 txg），头校验和重算。
fn orphan_node_that_the_i78_scan_counts(node: &[u8], tree_identifier: u64) -> Vec<u8> {
    let mut bad = node.to_vec();
    assert_eq!(&bad[..4], b"SFSU", "拿来改的是一个单元");
    assert_eq!(bad[6], 2, "拿来改的是码 2 节点");
    let key_count = usize::from(bad[51]);
    let header_end = 86 + 2 * key_count;
    bad[42..50].copy_from_slice(&tree_identifier.to_le_bytes());
    let birth = 52 + 2 * key_count;
    bad[birth..birth + 8].copy_from_slice(&1u64.to_le_bytes());
    bad[10..42].fill(0);
    let checksum = singlefs_checker::crc32_castagnoli_bitwise(&bad[..header_end]);
    bad[10..14].copy_from_slice(&checksum.to_le_bytes());
    bad
}

fn unit_write(device: u32, slot: u64, bytes: Vec<u8>) -> RetainedWrite {
    RetainedWrite {
        device: DeviceIdentity(device),
        kind: StepKind::UnitWrite,
        is_force_unit_access: false,
        offset: DeviceOffsetInBytes(slot * LOCAL_SLOT_BYTES),
        contents: WrittenContents::Bytes(bytes),
    }
}

/// 前 7 段（mkfs 之后到暖机第二代的系统配置轮换）+ 一段只有单元写、没有根的段：`mutant` 时段首写一个 I-7.8 会数的孤儿节点、
/// 段尾同一槽写回全零（整段全落之后盘上看不出它）；不 `mutant` 时两次写换成同一槽上两次全零写（对照）。中间夹新建文件那段的前 4 次单元写。
fn p5_history(full: &History, mutant: bool) -> (History, usize) {
    let prefix = full.prefix_of_segments(7);
    let node = full.segments[7]
        .iter()
        .map(|index| &full.writes[*index])
        .find(|write| {
            write.device == DeviceIdentity(0)
                && write.length_in_bytes() == LOCAL_NODE_BYTES
                && write.bytes().is_some_and(|bytes| bytes[6] == 2)
        })
        .expect("新建文件那段里有一个码 2 节点")
        .bytes()
        .expect("字节")
        .to_vec();
    let free_slot = 60_000u64;
    for write in &full.writes {
        let start = write.offset.0;
        let end = start + write.length_in_bytes();
        assert!(end <= free_slot * LOCAL_SLOT_BYTES || start >= (free_slot + 1) * LOCAL_SLOT_BYTES, "孤儿那一槽没被任何写罩到");
    }
    let first_bytes = if mutant {
        orphan_node_that_the_i78_scan_counts(&node, 1 << 40)
    } else {
        vec![0u8; usize::try_from(LOCAL_NODE_BYTES).expect("16384")]
    };
    let mut writes = prefix.writes.clone();
    let mut segment = Vec::new();
    segment.push(writes.len());
    writes.push(unit_write(0, free_slot, first_bytes));
    for index in full.segments[7].iter().take(4) {
        segment.push(writes.len());
        writes.push(full.writes[*index].clone());
    }
    segment.push(writes.len());
    writes.push(unit_write(0, free_slot, vec![0u8; usize::try_from(LOCAL_NODE_BYTES).expect("16384")]));
    let mut segments = prefix.segments.clone();
    segments.push(segment);
    let segment_index = segments.len() - 1;
    (
        History {
            base: prefix.base.clone(),
            writes,
            segments,
            versions: full.versions.clone(),
        },
        segment_index,
    )
}

fn world_p5_mutant_node_inside_a_unit_only_segment(output: &mut ResultLines) -> i32 {
    let full = first_history();
    for mutant in [false, true] {
        let label = if mutant { "p5_mutant" } else { "p5_control" };
        let (history, unit_segment) = p5_history(&full, mutant);
        guarded_state_count(output, label, &history, &|_, _| true);
        let mut states_in_segment = 0u64;
        let mut red_in_segment = 0u64;
        let mut red_names: BTreeMap<String, u64> = BTreeMap::new();
        let mut first_state_red: Option<bool> = None;
        let mut all_persisted_red = false;
        let mut walk_key_pairs = Vec::new();
        let mut full_key_pairs = Vec::new();
        let mut example_red: Option<Vec<bool>> = None;
        let tally = enumerate_observing(&history, &|_, _| true, &mut |image, _report| {
            let segment = segment_of_state(&history.segments, &image.persisted);
            if segment < unit_segment {
                return;
            }
            let judgment = judge_image(image, &history.versions);
            let is_red = !judgment.red.is_empty();
            if segment == history.segments.len() {
                all_persisted_red = is_red;
                return;
            }
            states_in_segment += 1;
            if first_state_red.is_none() {
                first_state_red = Some(is_red);
            }
            if is_red {
                red_in_segment += 1;
                for name in &judgment.red {
                    *red_names.entry(name.clone()).or_insert(0) += 1;
                }
                if example_red.is_none() {
                    example_red = Some(history.segments[unit_segment].iter().map(|index| image.persisted[*index]).collect());
                }
            }
            let reads = state_reads(image);
            let recovery_key = pair_digest(sequence_key(&reads.consult_calls), sequence_key(&reads.ignore_calls));
            walk_key_pairs.push((pair_digest(recovery_key, sequence_key(&reads.walk_calls)), judgment.digest));
            full_key_pairs.push((pair_digest(recovery_key, sequence_key(&reads.normal_calls)), judgment.digest));
        });
        let (walk_classes, walk_inconsistent) = inconsistent_under_key(&walk_key_pairs);
        let (full_classes, full_inconsistent) = inconsistent_under_key(&full_key_pairs);
        let missed_by_p5 = if first_state_red == Some(false) && !all_persisted_red {
            red_in_segment
        } else {
            0
        };
        output.line(format!(
            "name=attack_p5 world={label} enumerated_states={} unit_segment={unit_segment} states_in_unit_segment={states_in_segment} red_states_in_unit_segment={red_in_segment} red_by_name={red_names:?} segment_first_state_red={:?} all_persisted_state_red={all_persisted_red} example_red_landings_in_segment_order={example_red:?} tally_checker_violated={:?} walk_key_classes={walk_classes} walk_key_inconsistent={walk_inconsistent} full_key_classes={full_classes} full_key_inconsistent={full_inconsistent} must_be_nonzero={}",
            tally.states,
            first_state_red,
            tally.checker_violated_states,
            if mutant { missed_by_p5 } else { 0 }
        ));
    }
    0
}

// ───────────────────────────── P6：单元段上叶子数 = 状态数 ─────────────────────────────

fn world_p6_leaves_equal_states_on_a_unit_segment(output: &mut ResultLines, k_argument: Option<&String>) -> i32 {
    let k: usize = k_argument.map_or(12, |text| text.parse().expect("k"));
    let full = first_history();
    let prefix = full.prefix_of_segments(7);
    let mut history = prefix.clone();
    let mut segment = Vec::new();
    for index in full.segments[7].iter().take(k) {
        segment.push(history.writes.len());
        history.writes.push(full.writes[*index].clone());
    }
    history.segments.push(segment);
    let unit_segment = history.segments.len() - 1;
    guarded_state_count(output, "p6_recording_prefix", &history, &|_, _| true);
    let mut judgments = Vec::new();
    let mut normal_keys = Vec::new();
    let mut walk_keys = Vec::new();
    let mut recovery_keys = Vec::new();
    let mut candidate_answers = Vec::new();
    let mut byte_position_sets = Vec::new();
    let mut red = 0u64;
    let started = Instant::now();
    let tally = enumerate_observing(&history, &|_, _| true, &mut |image, _report| {
        if segment_of_state(&history.segments, &image.persisted) != unit_segment {
            return;
        }
        let judgment = judge_image(image, &history.versions);
        if !judgment.red.is_empty() {
            red += 1;
        }
        let reads = state_reads(image);
        judgments.push(judgment.digest);
        normal_keys.push(sequence_key(&reads.normal_calls));
        walk_keys.push(sequence_key(&reads.walk_calls));
        recovery_keys.push(pair_digest(sequence_key(&reads.consult_calls), sequence_key(&reads.ignore_calls)));
        candidate_answers.push(format!("{:?}", reads.candidate_unit_slots));
        byte_position_sets.push(reads.byte_positions.clone());
    });
    let states = judgments.len();
    let distinct_judgment = distinct(judgments.iter());
    let distinct_normal = distinct(normal_keys.iter());
    let full_pairs: Vec<(Digest128, Digest128)> = normal_keys.iter().zip(&recovery_keys).map(|(n, r)| pair_digest(*n, *r)).zip(judgments.iter().copied()).collect();
    let (full_classes, full_inconsistent) = inconsistent_under_key(&full_pairs);
    let walk_pairs: Vec<(Digest128, Digest128)> = walk_keys.iter().zip(&recovery_keys).map(|(w, r)| pair_digest(*w, *r)).zip(judgments.iter().copied()).collect();
    let (walk_classes, walk_inconsistent) = inconsistent_under_key(&walk_pairs);
    output.line(format!(
        "name=attack_p6 k={k} enumerated_states={} states_in_unit_segment={states} red_states_in_unit_segment={red} distinct_judgment={distinct_judgment} distinct_recovery_keys={} distinct_checker_normal_keys={distinct_normal} distinct_candidate_unit_slot_answers={} distinct_byte_position_sets={} leaves_recovery_plus_checker_normal={full_classes} inconsistent_under_that_key={full_inconsistent} leaves_if_scan_dropped_walk_key={walk_classes} inconsistent_if_scan_dropped={walk_inconsistent} seconds={:.1} must_be_nonzero={}",
        tally.states,
        distinct(recovery_keys.iter()),
        distinct(candidate_answers.iter()),
        distinct(byte_position_sets.iter()),
        started.elapsed().as_secs_f64(),
        full_classes.saturating_sub(distinct_judgment)
    ));
    0
}

// ───────────────────────────── T4 / T2：父结束镜像逐字节相同，下游枚举域不同 ─────────────────────────────

fn materialized(base: &MemoryPool, writes: &[RetainedWrite]) -> MemoryPool {
    let mut pool = base.clone();
    pool.apply_writes(writes);
    pool
}

fn world_t4_same_parent_image_different_domain(output: &mut ResultLines) -> i32 {
    let full = first_history();
    let parent = full.prefix_of_segments(7);
    let record_segment = &full.segments[8];
    let record_offset = full.writes[record_segment[0]].offset;
    assert_eq!(full.writes[record_segment[0]].kind, StepKind::JournalRecord);
    let child_writes: Vec<RetainedWrite> = record_segment.iter().map(|index| full.writes[*index].clone()).collect();
    let mut rows = Vec::new();
    for variant in ["a_plain", "b_bytes_then_zero_fill", "c_zero_bytes"] {
        let mut writes = parent.writes.clone();
        let mut segments = parent.segments.clone();
        let extra: Vec<RetainedWrite> = match variant {
            "a_plain" => Vec::new(),
            "b_bytes_then_zero_fill" => vec![
                RetainedWrite {
                    device: DeviceIdentity(0),
                    kind: StepKind::JournalRecord,
                    is_force_unit_access: false,
                    offset: record_offset,
                    contents: WrittenContents::Bytes(vec![0x5A; 4096]),
                },
                RetainedWrite {
                    device: DeviceIdentity(0),
                    kind: StepKind::ZeroFill,
                    is_force_unit_access: false,
                    offset: record_offset,
                    contents: WrittenContents::Zeros { length: 4096 },
                },
            ],
            _ => vec![RetainedWrite {
                device: DeviceIdentity(0),
                kind: StepKind::JournalRecord,
                is_force_unit_access: false,
                offset: record_offset,
                contents: WrittenContents::Bytes(vec![0; 4096]),
            }],
        };
        if !extra.is_empty() {
            let start = writes.len();
            writes.extend(extra);
            segments.push((start..writes.len()).collect());
        }
        let parent_end = materialized(&parent.base, &writes);
        let child_start = writes.len();
        writes.extend(child_writes.iter().cloned());
        segments.push((child_start..writes.len()).collect());
        let child_segment = segments.len() - 1;
        let history = History {
            base: parent.base.clone(),
            writes,
            segments,
            versions: full.versions.clone(),
        };
        let overlay_child_states = history.torn_state_count(&|index, _| index == child_segment) - 1;
        let tearable = TearableInPlaceOverwrites::of(&history.base, &history.writes);
        let materialized_history = History {
            base: parent_end.clone(),
            writes: child_writes.clone(),
            segments: vec![(0..child_writes.len()).collect()],
            versions: full.versions.clone(),
        };
        let materialized_child_states = materialized_history.torn_state_count(&|_, _| true) - 1;
        rows.push((variant, parent_end, overlay_child_states, materialized_child_states, tearable.contains(child_start)));
    }
    let (_, plain_end, plain_overlay, _, _) = &rows[0];
    for (variant, end, overlay, materialized_count, child_write_tearable) in &rows {
        let sector_maps_equal = end == plain_end;
        let bytes_equal = [0u32, 1].iter().all(|device| {
            let left = end.devices.get(&DeviceIdentity(*device)).expect("盘");
            let right = plain_end.devices.get(&DeviceIdentity(*device)).expect("盘");
            let sectors: BTreeSet<u64> = left.written_sectors().keys().chain(right.written_sectors().keys()).copied().collect();
            sectors.iter().all(|sector| {
                left.read(DeviceOffsetInBytes(sector * 512), 512) == right.read(DeviceOffsetInBytes(sector * 512), 512)
            })
        });
        output.line(format!(
            "name=attack_t4 variant={variant} parent_end_sector_maps_equal_to_a={sector_maps_equal} parent_end_bytes_equal_to_a={bytes_equal} child_writes_identical=true child_first_write_tearable_in_overlay={child_write_tearable} child_states_overlay={overlay} child_states_materialized={materialized_count} overlay_minus_a_plain={} overlay_minus_materialized={} must_be_nonzero={}",
            i128::from(*overlay) - i128::from(*plain_overlay),
            i128::from(*overlay) - i128::from(*materialized_count),
            if *variant == "a_plain" { 0 } else { overlay.abs_diff(*plain_overlay) }
        ));
    }
    0
}

// ───────────────────────────── 撕裂态：同一槽被同段两次原地写罩住 ─────────────────────────────

fn world_torn_convention_two_writes_same_slot(output: &mut ResultLines) -> i32 {
    let full = first_history();
    let slot_bytes = 4096usize;
    // 前 6 段（到暖机第二代根为止，带根槽写）之后接一段：同一个系统配置槽上先写 A（段 6 那次轮换写的字节）、再写回前缀之后那一份 C。
    let prefix = full.prefix_of_segments(6);
    let current_slot = materialized(&prefix.base, &prefix.writes).devices.get(&DeviceIdentity(0)).expect("盘 0").read(DeviceOffsetInBytes(0), slot_bytes);
    let first_write = full.writes[full.segments[6][0]].clone();
    assert_eq!(first_write.kind, StepKind::SystemConfigurationSlot);
    assert_eq!(first_write.offset.0, 0);
    assert_eq!(first_write.device, DeviceIdentity(0));
    assert_ne!(first_write.bytes().expect("字节"), current_slot.as_slice(), "A 与槽里现有的 C 不同");
    let written_back = RetainedWrite {
        contents: WrittenContents::Bytes(current_slot.clone()),
        ..first_write.clone()
    };
    let mut writes = prefix.writes.clone();
    let start = writes.len();
    writes.push(first_write.clone());
    writes.push(written_back.clone());
    let mut segments = prefix.segments.clone();
    segments.push(vec![start, start + 1]);
    let history = History {
        base: full.base.clone(),
        writes,
        segments,
        versions: full.versions.clone(),
    };
    guarded_state_count(output, "torn_two_writes_same_slot", &history, &|_, _| true);
    let prepared = history.prepared();
    let table = TornWriteTable::of(&prepared);
    let mut differing_images = 0u64;
    let mut differing_judgments = 0u64;
    let mut states = 0u64;
    let mut examples = Vec::new();
    let tally = enumerate_observing(&history, &|_, _| true, &mut |image, _report| {
        if segment_of_state(&history.segments, &image.persisted) != history.segments.len() - 1 {
            return;
        }
        states += 1;
        let landing = |index: usize| -> Landing {
            if image.persisted[index] {
                Landing::Persisted
            } else if table.torn_image_by_write.get(&index).is_some_and(|(torn, _)| image.persisted[*torn]) {
                Landing::Torn
            } else {
                Landing::NotPersisted
            }
        };
        let mut physical = history.base.clone();
        for (index, write) in history.writes.iter().enumerate() {
            let device = physical.devices.get_mut(&write.device).expect("盘");
            let landed = if index < start { Landing::Persisted } else { landing(index) };
            match landed {
                Landing::NotPersisted => {}
                Landing::Persisted => write.contents.apply_to(device, write.offset),
                Landing::Torn => {
                    let old = device.read(write.offset, slot_bytes);
                    device.write(write.offset, &torn_bytes(&old, &write.contents));
                }
            }
        }
        let enumerated_slot = PoolReader::read(image, DeviceIdentity(0), DeviceOffsetInBytes(0), slot_bytes).expect("读");
        let physical_slot = physical.devices.get(&DeviceIdentity(0)).expect("盘").read(DeviceOffsetInBytes(0), slot_bytes);
        let physical_persisted = vec![false; image.writes.len()];
        let physical_image = CrashImage {
            base: &physical,
            writes: image.writes,
            persisted: physical_persisted,
        };
        let enumerated_judgment = judge_image(image, &history.versions);
        let physical_judgment = judge_state(&physical_image, image.writes, &image.persisted, &history.versions);
        let images_differ = enumerated_slot != physical_slot;
        let judgments_differ = enumerated_judgment.digest != physical_judgment.digest;
        if images_differ {
            differing_images += 1;
        }
        if judgments_differ {
            differing_judgments += 1;
        }
        examples.push(format!(
            "{:?}/{:?}:images_differ={images_differ}:judgments_differ={judgments_differ}:enumerated_red={:?}:physical_red={:?}",
            landing(start),
            landing(start + 1),
            enumerated_judgment.red,
            physical_judgment.red
        ));
    });
    output.line(format!(
        "name=attack_torn enumerated_states={} states_in_segment={states} states_where_enumerated_slot_differs_from_physical_tearing={differing_images} states_where_judgment_differs={differing_judgments} per_state={examples:?} must_be_nonzero={differing_images}",
        tally.states
    ));
    0
}

// ───────────────────────────── 小域：G2 共模、P4 相邻、U4 同节点两条路径 ─────────────────────────────

/// 一个状态里每次录制流写的落法（持久 / 撕裂 / 没持久），按枚举用的写表认。
fn landings_of(table: &TornWriteTable, recorded_count: usize, persisted: &[bool]) -> Vec<Landing> {
    (0..recorded_count)
        .map(|index| {
            if persisted[index] {
                Landing::Persisted
            } else if table.torn_image_by_write.get(&index).is_some_and(|(torn, _)| persisted[*torn]) {
                Landing::Torn
            } else {
                Landing::NotPersisted
            }
        })
        .collect()
}

/// 与 `LocalPlan::persisted_of` 末段同一条换算：录制流写的落法 → 枚举用写表上的持久集合。
fn persisted_of_landings(table: &TornWriteTable, landings: &[Landing]) -> Vec<bool> {
    let mut persisted = vec![false; table.mask_width()];
    for (index, landing) in landings.iter().enumerate() {
        persisted[index] = *landing == Landing::Persisted;
    }
    for (index, (torn, replays)) in &table.torn_image_by_write {
        if landings[*index] == Landing::Torn {
            persisted[*torn] = true;
            for (replay, replayed) in replays {
                persisted[*replay] = landings[*replayed] == Landing::Persisted;
            }
        }
    }
    persisted
}

/// G2 的「内容编号」：这个位置每个扇区上最后一次持久了的写表项的下标（没有就是基线）。
fn content_numbers(image: &CrashImage<'_>, position: (u32, u64, usize)) -> Vec<u32> {
    let (device, offset, length) = position;
    let length_in_bytes = u64::try_from(length).expect("长");
    (offset / 512..(offset + length_in_bytes).div_ceil(512))
        .map(|sector| {
            let sector_start = sector * 512;
            image
                .writes
                .iter()
                .zip(&image.persisted)
                .enumerate()
                .filter(|(_, (write, persisted))| {
                    **persisted
                        && write.device.0 == device
                        && write.offset.0 < sector_start + 512
                        && sector_start < write.offset.0 + write.length_in_bytes()
                })
                .map(|(index, _)| u32::try_from(index).expect("下标"))
                .last()
                .unwrap_or(u32::MAX)
        })
        .collect()
}

fn world_small_domain_g2_p4_u4(output: &mut ResultLines) -> i32 {
    let content = common::file_content();
    let path_a = first_history();
    let path_b = {
        let mut pool = build_first_stream_in_memory(&content);
        let overwrite_content: Vec<u8> = (0..4100).map(|index| u8::try_from((index * 7 + 3) % 253).expect("字节")).collect();
        overwrite_in_memory(&mut pool, &overwrite_content);
        let mut history = History::of_pool(&pool, Vec::new());
        let mut versions = first_versions(&pool, &path_a.writes, &content);
        let (instance, checkpoint_txg) = root_identity_of_root_write(&history.writes[history.judged_root_index()]);
        versions.push(PublishedVersion {
            instance,
            checkpoint_txg,
            content: overwrite_content,
        });
        history.versions = versions;
        history
    };
    let shared_writes = path_a.writes.len();
    let shared_prefix_identical = path_b.base == path_a.base
        && path_b.writes[..shared_writes] == path_a.writes[..]
        && path_b.segments[..path_a.segments.len()] == path_a.segments[..];
    output.line(format!(
        "name=attack_small_paths path_a_writes={} path_a_segments={} path_b_writes={} path_b_segments={} path_b_lengths={:?} shared_prefix_identical={shared_prefix_identical}",
        path_a.writes.len(),
        path_a.segments.len(),
        path_b.writes.len(),
        path_b.segments.len(),
        path_b.segments.iter().map(Vec::len).collect::<Vec<_>>()
    ));
    let small = |_: usize, segment: &[usize]| segment.len() < 16;
    guarded_state_count(output, "small_domain_path_a", &path_a, &small);
    let table_a = TornWriteTable::of(&path_a.prepared());
    let table_b = TornWriteTable::of(&path_b.prepared());
    let publish_a = singlefs_checker_tier::crash::publish_of_each_segment(&path_a.writes, &path_a.segments);
    let publish_b = singlefs_checker_tier::crash::publish_of_each_segment(&path_b.writes, &path_b.segments);
    // 每段：代表态（段里第一个状态）的位置表，与段里每个状态的数。
    struct SegmentState {
        landings: Vec<Landing>,
        positions: Vec<(u32, u64, usize)>,
        judgment: Digest128,
        gpu_vector: Vec<Vec<u32>>,
        cpu_vector: Vec<Digest128>,
    }
    let mut by_segment: BTreeMap<usize, Vec<SegmentState>> = BTreeMap::new();
    let mut representative_positions: BTreeMap<usize, Vec<(u32, u64, usize)>> = BTreeMap::new();
    let mut u4_states = 0u64;
    let mut u4_judgment_differs = 0u64;
    let mut u4_red_differs = 0u64;
    let tally = enumerate_observing(&path_a, &small, &mut |image, _report| {
        let segment = segment_of_state(&path_a.segments, &image.persisted);
        let judgment = judge_image(image, &path_a.versions);
        let reads = state_reads(image);
        let positions = representative_positions.entry(segment).or_insert_with(|| reads.byte_positions.clone()).clone();
        let gpu_vector = positions.iter().map(|position| content_numbers(image, *position)).collect();
        let cpu_vector = positions
            .iter()
            .map(|(device, offset, length)| {
                PoolReader::read(image, DeviceIdentity(*device), DeviceOffsetInBytes(*offset), *length)
                    .map_or(Digest128(0), |bytes| digest_of_bytes(&bytes))
            })
            .collect();
        let landings = landings_of(&table_a, path_a.writes.len(), &image.persisted);
        if segment < path_a.segments.len() {
            let mut landings_b = landings.clone();
            landings_b.resize(path_b.writes.len(), Landing::NotPersisted);
            let persisted_b = persisted_of_landings(&table_b, &landings_b);
            let image_b = CrashImage {
                base: &path_b.base,
                writes: &table_b.writes,
                persisted: persisted_b,
            };
            let judgment_b = judge_image(&image_b, &path_b.versions);
            u4_states += 1;
            if judgment_b.digest != judgment.digest {
                u4_judgment_differs += 1;
            }
            if judgment_b.red != judge_image(image, &path_a.versions).red {
                u4_red_differs += 1;
            }
        }
        by_segment.entry(segment).or_default().push(SegmentState {
            landings,
            positions: reads.byte_positions,
            judgment: judgment.digest,
            gpu_vector,
            cpu_vector,
        });
    });
    let differing_publish_labels: Vec<String> = (0..path_a.segments.len())
        .filter(|index| publish_a[*index] != publish_b[*index])
        .map(|index| format!("{index}:{}->{}", publish_a[index].name(), publish_b[index].name()))
        .collect();
    output.line(format!(
        "name=attack_u4_shared_node states={u4_states} judgment_digest_differs={u4_judgment_differs} red_list_differs={u4_red_differs} segments_whose_publish_label_differs={differing_publish_labels:?} enumerated_states={}",
        tally.states
    ));
    // G2：按代表态的位置表求内容编号向量分组；CPU 另一份实现读字节求摘要分组。
    let mut g2_states = 0usize;
    let mut g2_inconsistent = 0usize;
    let mut g2_gpu_cpu_partition_mismatch = 0usize;
    let mut g2_classes = 0usize;
    let mut p4_pairs = 0usize;
    let mut p4_pairs_with_new_positions = 0usize;
    let mut p4_example = None;
    for (segment, states) in &by_segment {
        if *segment == path_a.segments.len() {
            continue;
        }
        let mut gpu_first: BTreeMap<&Vec<Vec<u32>>, (usize, Digest128)> = BTreeMap::new();
        let mut cpu_first: BTreeMap<&Vec<Digest128>, usize> = BTreeMap::new();
        for (state_index, state) in states.iter().enumerate() {
            g2_states += 1;
            let (gpu_representative, gpu_judgment) = *gpu_first.entry(&state.gpu_vector).or_insert((state_index, state.judgment));
            let cpu_representative = *cpu_first.entry(&state.cpu_vector).or_insert(state_index);
            if gpu_judgment != state.judgment {
                g2_inconsistent += 1;
            }
            if gpu_representative != cpu_representative {
                g2_gpu_cpu_partition_mismatch += 1;
            }
        }
        g2_classes += gpu_first.len();
        for (left_index, left) in states.iter().enumerate() {
            for right in &states[left_index + 1..] {
                let flipped: Vec<usize> = (0..left.landings.len()).filter(|index| left.landings[*index] != right.landings[*index]).collect();
                if flipped.len() != 1 {
                    continue;
                }
                p4_pairs += 1;
                let write = &path_a.writes[flipped[0]];
                let covered = |position: &(u32, u64, usize)| {
                    position.0 == write.device.0
                        && position.1 < write.offset.0 + write.length_in_bytes()
                        && write.offset.0 < position.1 + u64::try_from(position.2).expect("长")
                };
                let left_set: BTreeSet<_> = left.positions.iter().collect();
                let right_set: BTreeSet<_> = right.positions.iter().collect();
                let new_positions: Vec<_> = left_set.symmetric_difference(&right_set).filter(|position| !covered(position)).collect();
                if !new_positions.is_empty() {
                    p4_pairs_with_new_positions += 1;
                    if p4_example.is_none() {
                        p4_example = Some(format!("segment={segment} flipped_write={} kind={:?} {:?}->{:?} positions_not_covered_by_the_flipped_write={}", flipped[0], write.kind, left.landings[flipped[0]], right.landings[flipped[0]], new_positions.len()));
                    }
                }
            }
        }
    }
    output.line(format!(
        "name=attack_g2 states={g2_states} classes_by_representative_positions={g2_classes} states_whose_judgment_differs_from_their_class={g2_inconsistent} gpu_cpu_partition_mismatch={g2_gpu_cpu_partition_mismatch} must_be_nonzero={g2_inconsistent}"
    ));
    output.line(format!(
        "name=attack_p4 pairs_differing_in_one_write={p4_pairs} pairs_reading_positions_the_flipped_write_does_not_cover={p4_pairs_with_new_positions} example={p4_example:?} must_be_nonzero={p4_pairs_with_new_positions}"
    ));
    0
}

// ───────────────────────────── U4：物化父镜像、只交本节点写表，记录核对器看不见「根在前、记录在后」 ─────────────────────────────

/// 物化那一形的一个状态：基线 = 这一段之前各段全落的镜像；写表 = 本段的写（带撕裂镜像）；持久集合按本段的落法。
struct MaterializedState {
    base: MemoryPool,
    table: TornWriteTable,
}

fn materialized_node(history: &History, segment: usize) -> MaterializedState {
    let earlier: Vec<RetainedWrite> = history.segments[..segment].iter().flatten().map(|index| history.writes[*index].clone()).collect();
    let base = materialized(&history.base, &earlier);
    let local_writes: Vec<RetainedWrite> = history.segments[segment].iter().map(|index| history.writes[*index].clone()).collect();
    let local = History {
        base: base.clone(),
        writes: local_writes.clone(),
        segments: vec![(0..local_writes.len()).collect()],
        versions: history.versions.clone(),
    };
    let mut prepared = local.prepared_without_root();
    prepared.base = base.clone();
    let table = TornWriteTable::of(&prepared);
    MaterializedState { base, table }
}

impl History {
    fn prepared_without_root(&self) -> PreparedStream {
        PreparedStream {
            name: StreamName::First,
            base: self.base.clone(),
            writes: self.writes.clone(),
            segments: self.segments.clone(),
            judged_root_index: 0,
            versions: self.versions.clone(),
            root_write_count: 0,
        }
    }
}

fn compare_materialized(output: &mut ResultLines, label: &str, history: &History, expanded: &dyn Fn(usize, &[usize]) -> bool) {
    guarded_state_count(output, label, history, expanded);
    let table = TornWriteTable::of(&history.prepared());
    let mut nodes: BTreeMap<usize, MaterializedState> = BTreeMap::new();
    let mut rows: BTreeMap<usize, [u64; 8]> = BTreeMap::new();
    let mut red_names: BTreeMap<String, (u64, u64, u64)> = BTreeMap::new();
    let tally = enumerate_observing(history, expanded, &mut |image, _report| {
        let segment = segment_of_state(&history.segments, &image.persisted);
        if segment == history.segments.len() {
            return;
        }
        let node = nodes.entry(segment).or_insert_with(|| materialized_node(history, segment));
        let landings = landings_of(&table, history.writes.len(), &image.persisted);
        let local_landings: Vec<Landing> = history.segments[segment].iter().map(|index| landings[*index]).collect();
        let local_persisted = persisted_of_landings(&node.table, &local_landings);
        let local_image = CrashImage {
            base: &node.base,
            writes: &node.table.writes,
            persisted: local_persisted.clone(),
        };
        let full = judge_image(image, &history.versions);
        let local_only = judge_state(&local_image, &node.table.writes, &local_persisted, &history.versions);
        let local_image_full_records = judge_state(&local_image, image.writes, &image.persisted, &history.versions);
        let reads = state_reads(image);
        let bytes_differ = reads.byte_positions.iter().any(|(device, offset, length)| {
            PoolReader::read(image, DeviceIdentity(*device), DeviceOffsetInBytes(*offset), *length)
                != PoolReader::read(&local_image, DeviceIdentity(*device), DeviceOffsetInBytes(*offset), *length)
        });
        let hints_differ = [0u32, 1].iter().any(|device| {
            ImageReader::candidate_unit_slots(image, *device) != ImageReader::candidate_unit_slots(&local_image, *device)
                || ImageReader::candidate_journal_slots(image, *device) != ImageReader::candidate_journal_slots(&local_image, *device)
                || PoolReader::journal_record_offsets_hint(image, DeviceIdentity(*device), DeviceOffsetInBytes(RING_START_BYTES), LOCAL_JOURNAL_RING_DEFAULT_BYTES)
                    != PoolReader::journal_record_offsets_hint(&local_image, DeviceIdentity(*device), DeviceOffsetInBytes(RING_START_BYTES), LOCAL_JOURNAL_RING_DEFAULT_BYTES)
        });
        let row = rows.entry(segment).or_insert([0; 8]);
        row[0] += 1;
        row[1] += u64::from(!full.red.is_empty());
        row[2] += u64::from(!local_only.red.is_empty());
        row[3] += u64::from(full.digest != local_only.digest);
        row[4] += u64::from(!full.red.is_empty() && local_only.red.is_empty());
        row[5] += u64::from(full.digest != local_image_full_records.digest);
        row[6] += u64::from(bytes_differ);
        row[7] += u64::from(hints_differ);
        for name in &full.red {
            red_names.entry(name.clone()).or_insert((0, 0, 0)).0 += 1;
        }
        for name in &local_only.red {
            red_names.entry(name.clone()).or_insert((0, 0, 0)).1 += 1;
        }
        for name in &local_image_full_records.red {
            red_names.entry(name.clone()).or_insert((0, 0, 0)).2 += 1;
        }
    });
    let mut missed_total = 0;
    let mut materialized_image_differs_total = 0;
    for (segment, row) in &rows {
        missed_total += row[4];
        materialized_image_differs_total += row[5];
        output.line(format!(
            "name=attack_materialized world={label} segment={segment} states={} red_full_stream={} red_materialized_node_local_writes={} judgment_differs_node_local={} red_only_under_full_stream={} judgment_differs_materialized_image_with_full_record_stream={} states_with_read_bytes_differing={} states_with_hint_answers_differing={}",
            row[0], row[1], row[2], row[3], row[4], row[5], row[6], row[7]
        ));
    }
    output.line(format!(
        "name=attack_materialized_summary world={label} enumerated_states={} red_names_full_local_localimage_fullrecords={red_names:?} judgment_differs_materialized_image_with_full_record_stream_total={materialized_image_differs_total} must_be_nonzero={missed_total}",
        tally.states
    ));
}

fn world_u4_materialized_record_stream_misses_root_before_records(output: &mut ResultLines) -> i32 {
    let full = first_history();
    let small = |_: usize, segment: &[usize]| segment.len() < 16;
    compare_materialized(output, "healthy_first_stream", &full, &small);
    // 改坏的实现：新建文件那次发布先写根（FUA）、后写 journal 记录——录制流里段 8（记录）与段 9（根）对调。
    let records = full.segments[8].clone();
    let root = full.segments[9].clone();
    let mut order: Vec<usize> = full.segments[..8].iter().flatten().copied().collect();
    order.extend(root.iter().copied());
    order.extend(records.iter().copied());
    for segment in &full.segments[10..] {
        order.extend(segment.iter().copied());
    }
    let writes: Vec<RetainedWrite> = order.iter().map(|index| full.writes[*index].clone()).collect();
    let mut segments = full.segments[..8].to_vec();
    let mut next = segments.iter().map(Vec::len).sum::<usize>();
    for length in [root.len(), records.len()].into_iter().chain(full.segments[10..].iter().map(Vec::len)) {
        segments.push((next..next + length).collect());
        next += length;
    }
    let mutant = History {
        base: full.base.clone(),
        writes,
        segments,
        versions: full.versions.clone(),
    };
    compare_materialized(output, "root_before_records_mutant", &mutant, &small);
    0
}

// ───────────────────────────── T2：数据单元的后半槽只在物化那一形上被扫 ─────────────────────────────

fn world_t2_data_unit_interior_scanned_only_when_materialized(output: &mut ResultLines) -> i32 {
    let probe_words = 4250u32;
    let probe_content: Vec<u8> = (0..probe_words).flat_map(|index| (index ^ 0x5EED_0000).to_le_bytes()).collect();
    let probe_pool = build_first_stream_in_memory(&probe_content);
    let probe = History::of_pool(&probe_pool, Vec::new());
    let data_unit = probe
        .writes
        .iter()
        .find(|write| {
            write.kind == StepKind::UnitWrite
                && write.length_in_bytes() == LOCAL_DATA_UNIT_BYTES
                && write.bytes().is_some_and(|bytes| bytes.windows(64).any(|window| window == &probe_content[..64]))
        })
        .expect("有一次 32K 单元写带着文件内容");
    let unit_bytes = data_unit.bytes().expect("字节");
    let content_start = unit_bytes.windows(64).position(|window| window == &probe_content[..64]).expect("位置");
    let node = probe
        .writes
        .iter()
        .find(|write| write.length_in_bytes() == LOCAL_NODE_BYTES && write.bytes().is_some_and(|bytes| &bytes[..4] == b"SFSU" && bytes[6] == 2))
        .expect("码 2 节点")
        .bytes()
        .expect("字节")
        .to_vec();
    let orphan = orphan_node_that_the_i78_scan_counts(&node, 1 << 40);
    let header_end = 86 + 2 * usize::from(orphan[51]);
    let slot_bytes = usize::try_from(LOCAL_NODE_BYTES).expect("16384");
    let place = slot_bytes - content_start;
    let mut crafted = probe_content.clone();
    assert!(place + header_end <= crafted.len(), "文件够长，能把一个节点头摆到数据单元的第二个 16K 槽开头");
    crafted[place..place + header_end].copy_from_slice(&orphan[..header_end]);
    let crafted_pool = build_first_stream_in_memory(&crafted);
    let mut history = History::of_pool(&crafted_pool, Vec::new());
    history.versions = first_versions(&crafted_pool, &history.writes, &crafted);
    let landed = history
        .writes
        .iter()
        .filter(|write| write.kind == StepKind::UnitWrite && write.length_in_bytes() == LOCAL_DATA_UNIT_BYTES)
        .filter(|write| write.bytes().is_some_and(|bytes| bytes[slot_bytes..slot_bytes + header_end] == orphan[..header_end]))
        .count();
    output.line(format!(
        "name=attack_t2_interior content_bytes={} content_start_in_data_unit={content_start} node_header_bytes={header_end} data_unit_writes_carrying_the_header_at_the_second_slot={landed} segment_lengths={:?}",
        crafted.len(),
        history.segments.iter().map(Vec::len).collect::<Vec<_>>()
    ));
    let small = |_: usize, segment: &[usize]| segment.len() < 16;
    compare_materialized(output, "data_unit_interior_looks_like_a_node", &history, &small);
    let final_overlay = CrashImage {
        base: &history.base,
        writes: &history.writes,
        persisted: vec![true; history.writes.len()],
    };
    let final_materialized = materialized(&history.base, &history.writes);
    let overlay_judgment = judge_image(&final_overlay, &history.versions);
    let materialized_judgment = judge_state(&final_materialized, &history.writes, &vec![true; history.writes.len()], &history.versions);
    output.line(format!(
        "name=attack_t2_interior_final_state overlay_red={:?} materialized_memory_pool_red={:?} must_be_nonzero={}",
        overlay_judgment.red,
        materialized_judgment.red,
        u64::from(overlay_judgment.red != materialized_judgment.red)
    ));
    0
}

// ───────────────────────────── 节点身份按写种类串：条件不同的发布并成一个节点 ─────────────────────────────

fn world_kind_string_identity_merges_conditions(output: &mut ResultLines, count_argument: Option<&String>) -> i32 {
    let overwrites: usize = count_argument.map_or(30, |text| text.parse().expect("次数"));
    let content = common::file_content();
    let mut pool = build_first_stream_in_memory(&content);
    for seed in 0..overwrites {
        let next: Vec<u8> = (0..3000).map(|index| u8::try_from((index * 5 + seed) % 251).expect("字节")).collect();
        overwrite_in_memory(&mut pool, &next);
    }
    let history = History::of_pool(&pool, Vec::new());
    let kind_letter = |kind: StepKind| match kind {
        StepKind::ZeroFill => 'Z',
        StepKind::UnitWrite => 'U',
        StepKind::JournalRecord => 'J',
        StepKind::RootRecordFua => 'R',
        StepKind::SystemConfigurationSlot => 'S',
        StepKind::Barrier => 'B',
    };
    let segment_text = |segment: &Vec<usize>| -> String {
        segment.iter().map(|index| kind_letter(history.writes[*index].kind)).collect()
    };
    let overlaps = |left: &RetainedWrite, right: &RetainedWrite| {
        left.device == right.device
            && left.offset.0 < right.offset.0 + right.length_in_bytes()
            && right.offset.0 < left.offset.0 + left.length_in_bytes()
    };
    let mut previous_root_segment: Option<usize> = None;
    let mut signatures_by_kind: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut publishes = 0usize;
    for (segment_index, segment) in history.segments.iter().enumerate() {
        if !segment.iter().any(|index| history.writes[*index].kind == StepKind::RootRecordFua) {
            continue;
        }
        let first = previous_root_segment.map_or(0, |previous| previous + 1);
        let last = (segment_index + 1).min(history.segments.len() - 1);
        let kinds: Vec<String> = history.segments[first..=last].iter().map(segment_text).collect();
        let this_publish: Vec<usize> = history.segments[first..=segment_index].iter().flatten().copied().collect();
        let earlier_end = this_publish.iter().min().copied().unwrap_or(0);
        let earlier = &history.writes[..earlier_end];
        let reuses_a_unit_slot = this_publish.iter().any(|index| {
            let write = &history.writes[*index];
            write.kind == StepKind::UnitWrite && earlier.iter().any(|old| old.kind == StepKind::UnitWrite && overlaps(old, write))
        });
        let rewrites_a_root_slot = this_publish.iter().any(|index| {
            let write = &history.writes[*index];
            write.kind == StepKind::RootRecordFua && earlier.iter().any(|old| old.kind == StepKind::RootRecordFua && overlaps(old, write))
        });
        let rewrites_a_journal_slot = this_publish.iter().any(|index| {
            let write = &history.writes[*index];
            write.kind == StepKind::JournalRecord && earlier.iter().any(|old| old.kind == StepKind::JournalRecord && overlaps(old, write))
        });
        let signature = format!("reuse={reuses_a_unit_slot},root_slot_rewrite={rewrites_a_root_slot},journal_slot_rewrite={rewrites_a_journal_slot}");
        signatures_by_kind.entry(kinds.join("|")).or_default().insert(signature);
        publishes += 1;
        previous_root_segment = Some(segment_index);
    }
    let merged: Vec<String> = signatures_by_kind
        .iter()
        .filter(|(_, signatures)| signatures.len() > 1)
        .map(|(kinds, signatures)| format!("{kinds}=>{signatures:?}"))
        .collect();
    output.line(format!(
        "name=attack_kind_string overwrites={overwrites} publishes={publishes} distinct_kind_strings={} kind_strings_with_several_condition_signatures={} detail={merged:?} must_be_nonzero={}",
        signatures_by_kind.len(),
        merged.len(),
        merged.len()
    ));
    0
}

// ───────────────────────────── 键用的 128 位摘要：用户数据可控时造得出碰撞 ─────────────────────────────

fn world_digest_collision(output: &mut ResultLines) -> i32 {
    // 两份 16384 字节（一个 16K 节点那么长）的内容，只在第 100、101 两个 8 字节字上不同；前缀与后缀逐字节相同。
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
    let mut attempts = 0u64;
    let mut found: Option<(u64, u64)> = None;
    let mut seed = 0x1234_5678_9ABC_DEF0u64;
    while found.is_none() {
        attempts += 1;
        // Brent：先求环长 λ，再求尾长 μ，环入口前一步的两个不同原像撞在同一个值上。
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
    let differing_bytes = first.iter().zip(&second).filter(|(left, right)| left != right).count();
    let first_digest = digest_of_bytes(&first);
    let second_digest = digest_of_bytes(&second);
    output.line(format!(
        "name=attack_digest_collision length={length} differing_bytes={differing_bytes} first_digest={:032x} second_digest={:032x} digests_equal={} rho_attempts={attempts} seconds={:.1} crc32c_first={:08x} crc32c_second={:08x} must_be_nonzero={}",
        first_digest.0,
        second_digest.0,
        first_digest == second_digest,
        started.elapsed().as_secs_f64(),
        singlefs_checker::crc32_castagnoli_table(&first),
        singlefs_checker::crc32_castagnoli_table(&second),
        u64::from(first_digest == second_digest && first != second)
    ));
    0
}

/// 改坏的分配器：txg 4 那次覆盖写发布之后，另起一段把 txg 3 那一版的数据单元（两盘两份）提前盖掉（回收谓词不许：释放代 4 > 环里最旧根的 txg）；
/// 再接一段无害的单元写（空槽写全零），让「两份都盖掉」那一态落在一段的中间、被枚举到。
fn world_u4_materialized_record_stream_misses_premature_reuse(output: &mut ResultLines) -> i32 {
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
    let old_data_units: Vec<RetainedWrite> = path_a.segments[7]
        .iter()
        .map(|index| path_a.writes[*index].clone())
        .filter(|write| write.length_in_bytes() == LOCAL_DATA_UNIT_BYTES && write.bytes().is_some_and(|bytes| bytes.windows(64).any(|window| window == &content[..64])))
        .collect();
    assert_eq!(old_data_units.len(), 2, "txg 3 的数据单元两盘各一份");
    let start = history.writes.len();
    for old in &old_data_units {
        history.writes.push(RetainedWrite { contents: WrittenContents::Bytes(vec![0xEE; 32_768]), ..old.clone() });
    }
    history.segments.push((start..history.writes.len()).collect());
    let harmless = history.writes.len();
    history.writes.push(unit_write(0, 60_000, vec![0; 16_384]));
    history.writes.push(unit_write(1, 60_000, vec![0; 16_384]));
    history.segments.push(vec![harmless, harmless + 1]);
    output.line(format!(
        "name=attack_reuse_mutant old_data_unit_offset={} segment_lengths={:?}",
        old_data_units[0].offset.0,
        history.segments.iter().map(Vec::len).collect::<Vec<_>>()
    ));
    let small = |_: usize, segment: &[usize]| segment.len() < 16;
    compare_materialized(output, "premature_reuse_of_the_txg3_data_unit", &history, &small);
    0
}

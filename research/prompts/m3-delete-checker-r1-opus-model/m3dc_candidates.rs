//! 攻方腿原型（m3-delete-checker-r1-opus，副本专用）：删除线 checker 五条不变量的各个读法候选，逐版判在合法历史与改坏的镜像上。
//! 每段历史每一步之后的镜像判根环里每一条读得出的根；每一步写出的那一段按层 0 的枚举域（前几段全持久、当前段任意整写子集）
//! 逐个崩溃状态判，每个状态另跑一次池级 checker。批大小由 `M3DC_BATCH`、改坏由 `M3DC_MUT`、计划由 `M3DC_PLAN` 选。
#![allow(clippy::too_many_lines, reason = "原型")]

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceOffsetInBytes};
use singlefs_core::bytes::ByteReader;
use singlefs_core::checksum::crc32_castagnoli;
use singlefs_core::deleted_inodes::inode_numbers_of_the_deleted_inodes_root_entries;
use singlefs_core::extent_tree::{read_extent_tree, ExtentTreeHeaderJudgement, ExtentTreeReading};
use singlefs_core::pointer::NodePointer;
use singlefs_core::records::{
    parse_inode_internal_entry, AccountingEntry, InodeRecord, TombstoneRecord, TreeTableEntry,
    STATISTIC_PENDING_DELETE_BYTES,
    TREE_KIND_DELETED_INODES, TREE_KIND_LOGGED_OPERATIONS, TREE_KIND_TOMBSTONE,
};
use singlefs_core::recovery::{
    choose_system_configuration, readable_roots, recover, JournalPolicy, PoolReader,
    RecoveryFailure,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::unit::{
    data_unit_payload_capacity, parse_index_node, parse_packed_unit, unit_filesystem_identifier,
};
use singlefs_format::SLOT_BYTES;
use singlefs_harness::history::{
    execute_history_with, ContentChoice, ContentLength, GeneratedHistory, HistoryDeviceWidth,
    HistoryExecution, HistoryOperation, HistorySeed, HistoryStartingPoint, PerStepChecker,
    RollbackTargetChoice, StepPosition,
};
use singlefs_harness::memory_pool::{
    closed_form_state_count, writes_and_segments, CrashImage, MemoryPool,
};
use singlefs_harness::SharedStream;

const TREE_KIND_EXTENT: u16 = 1;
const TREE_KIND_INODE: u16 = 2;
const TREE_KIND_ACCOUNTING: u16 = 4;

#[derive(Clone, Debug, PartialEq, Eq)]
enum Tag4 {
    NoAccountingTree,
    Missing,
    Value(u64),
}

/// 一版里候选要用的事实，全由原型自己按字节读（不经实现的重建与它的判定：重建会拒改坏的几种镜像）。
#[derive(Clone, Debug)]
struct VersionFacts {
    txg: u64,
    rollback_floor: u64,
    /// inode → (nlink, size, 对象出生代)
    records: BTreeMap<u64, (u32, u64, u64)>,
    queue: Vec<u64>,
    queue_entry_width: Option<usize>,
    units: BTreeMap<u64, u64>,
    tag4: Tag4,
    logged_ops_root_is_zero: bool,
    tombstones: Vec<(u64, u64)>,
    has_trees: bool,
    read_failure: Option<String>,
}

fn read_unit(reader: &dyn PoolReader, pointer: &NodePointer, bytes: usize) -> Option<Vec<u8>> {
    for location in &pointer.locations {
        if let Some(read) =
            reader.read(location.device, DeviceOffsetInBytes(location.slot.0 * SLOT_BYTES), bytes)
        {
            if crc32_castagnoli(&read) == location.unit_checksum {
                return Some(read);
            }
        }
    }
    None
}

fn node_pointer_at(bytes: &[u8], offset: usize) -> NodePointer {
    NodePointer::read_from(&mut ByteReader::at(bytes, offset))
}

/// 码 2 根之下：类型段 0 的孩子是码 2、类型段非 0 的孩子是码 3 容器（inode 树、墓碑树同形：条目 = 分隔 key 8 + 身份引用 26 + 子指针 86）。
fn packed_records_below(
    reader: &dyn PoolReader,
    node: &NodePointer,
    depth: usize,
    out: &mut Vec<Vec<u8>>,
) -> Result<(), String> {
    if depth > 8 {
        return Err("码 2 树太深".into());
    }
    let bytes = read_unit(reader, node, 16384).ok_or("码 2 节点读不出")?;
    let header = parse_index_node(&bytes).map_err(|error| format!("码 2 节点解不开 {error:?}"))?;
    for entry in &header.entries {
        let (_, identity, child) =
            parse_inode_internal_entry(entry).ok_or_else(|| format!("条目解不开，宽 {}", entry.len()))?;
        if identity.record_type != 0 {
            let unit = read_unit(reader, &child, 32768).ok_or("码 3 容器读不出")?;
            let packed =
                parse_packed_unit(&unit).map_err(|error| format!("码 3 容器解不开 {error:?}"))?;
            out.extend(packed.records);
        } else {
            packed_records_below(reader, &child, depth + 1, out)?;
        }
    }
    Ok(())
}

fn accounting_rows_below(
    reader: &dyn PoolReader,
    node: &NodePointer,
    depth: usize,
    out: &mut Vec<AccountingEntry>,
) -> Result<(), String> {
    if depth > 8 {
        return Err("记账树太深".into());
    }
    let bytes = read_unit(reader, node, 16384).ok_or("记账节点读不出")?;
    let header = parse_index_node(&bytes).map_err(|error| format!("记账节点解不开 {error:?}"))?;
    for entry in &header.entries {
        if header.level == 0 {
            out.push(AccountingEntry::parse(entry).ok_or("记账行解不开")?);
        } else {
            accounting_rows_below(reader, &node_pointer_at(entry, 22), depth + 1, out)?;
        }
    }
    Ok(())
}

fn facts_of_root(reader: &dyn PoolReader, root: &RootRecord) -> VersionFacts {
    let mut facts = VersionFacts {
        txg: root.checkpoint_txg.0,
        rollback_floor: root.rollback_floor.0,
        records: BTreeMap::new(),
        queue: Vec::new(),
        queue_entry_width: None,
        units: BTreeMap::new(),
        tag4: Tag4::NoAccountingTree,
        logged_ops_root_is_zero: true,
        tombstones: Vec::new(),
        has_trees: false,
        read_failure: None,
    };
    if let Err(failure) = fill_facts(reader, root, &mut facts) {
        facts.read_failure = Some(failure);
    }
    facts
}

fn fill_facts(
    reader: &dyn PoolReader,
    root: &RootRecord,
    facts: &mut VersionFacts,
) -> Result<(), String> {
    let table = read_unit(reader, &root.tree_table, 16384).ok_or("树表读不出")?;
    let header = parse_index_node(&table).map_err(|error| format!("树表解不开 {error:?}"))?;
    if header.entries.is_empty() {
        return Ok(());
    }
    facts.has_trees = true;
    for entry_bytes in &header.entries {
        let entry = TreeTableEntry::parse(entry_bytes).ok_or("树表条目解不开")?;
        let zero = entry.root == NodePointer::empty_root();
        match entry.kind {
            TREE_KIND_INODE if !zero => {
                let mut records = Vec::new();
                packed_records_below(reader, &entry.root, 0, &mut records)?;
                for bytes in records {
                    let record = InodeRecord::parse(&bytes).ok_or("inode 记录解不开")?;
                    facts.records.insert(
                        record.inode,
                        (record.nlink, record.size, record.object_birth.0),
                    );
                }
            }
            TREE_KIND_DELETED_INODES if !zero => {
                let bytes = read_unit(reader, &entry.root, 16384).ok_or("队列根读不出")?;
                let node =
                    parse_index_node(&bytes).map_err(|error| format!("队列根解不开 {error:?}"))?;
                facts.queue_entry_width = Some(node.entry_width);
                facts.queue = inode_numbers_of_the_deleted_inodes_root_entries(&node.entries)
                    .into_iter()
                    .map(|number| number.0)
                    .collect();
            }
            TREE_KIND_EXTENT if !zero => {
                let reading = ExtentTreeReading {
                    tree: entry.tree,
                    judgement: ExtentTreeHeaderJudgement::OnlyWhatThePositionsNeed,
                    root,
                    expected_filesystem_identifier: unit_filesystem_identifier(
                        &root.filesystem_identifier,
                    ),
                };
                let tree = read_extent_tree(&reading, &entry.root, &mut |pointer| {
                    read_unit(reader, pointer, 16384).ok_or(RecoveryFailure::UnitMalformed {
                        what: "m3dc 读 extent 节点",
                    })
                })
                .map_err(|failure| format!("extent 树读不下去 {failure:?}"))?;
                for (inode, extents) in &tree.files {
                    let count = u64::try_from(extents.data_pointers().len()).expect("单元数");
                    facts.units.insert(*inode, count);
                }
            }
            TREE_KIND_ACCOUNTING if !zero => {
                let mut rows = Vec::new();
                accounting_rows_below(reader, &entry.root, 0, &mut rows)?;
                facts.tag4 = rows
                    .iter()
                    .find(|row| row.statistic == STATISTIC_PENDING_DELETE_BYTES)
                    .map_or(Tag4::Missing, |row| Tag4::Value(row.value));
            }
            TREE_KIND_LOGGED_OPERATIONS => facts.logged_ops_root_is_zero = zero,
            TREE_KIND_TOMBSTONE if !zero => {
                let mut records = Vec::new();
                packed_records_below(reader, &entry.root, 0, &mut records)?;
                for bytes in records {
                    let tombstone = TombstoneRecord::parse(&bytes).ok_or("墓碑记录解不开")?;
                    facts.tombstones.push((tombstone.object.0, tombstone.object_birth.0));
                }
            }
            _ => {}
        }
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Verdict {
    Holds,
    Red(String),
    NotApplicable,
}

fn judge_all(rule: impl Iterator<Item = (u64, bool)>) -> Verdict {
    let mut seen = false;
    for (inode, holds) in rule {
        seen = true;
        if !holds {
            return Verdict::Red(format!("inode {inode}"));
        }
    }
    if seen {
        Verdict::Holds
    } else {
        Verdict::NotApplicable
    }
}

/// 每个候选在这一版上的判定。名字里带「零轮」的是本腿自己提的读法。
fn candidate_verdicts(facts: &VersionFacts) -> Vec<(&'static str, Verdict)> {
    let capacity = u64::try_from(data_unit_payload_capacity()).expect("32634");
    let queue: BTreeSet<u64> = facts.queue.iter().copied().collect();
    let units = |inode: u64| facts.units.get(&inode).copied().unwrap_or(0);
    let nlink_zero: Vec<u64> = facts
        .records
        .iter()
        .filter(|(_, (nlink, _, _))| *nlink == 0)
        .map(|(inode, _)| *inode)
        .collect();
    let with_units: Vec<u64> = facts
        .units
        .iter()
        .filter(|(_, count)| **count > 0)
        .map(|(inode, _)| *inode)
        .collect();
    let live = |inode: u64| matches!(facts.records.get(&inode), Some((nlink, _, _)) if *nlink >= 1);
    let size_fits = |inode: u64, lenient: bool| {
        let (_, size, _) = facts.records[&inode];
        let count = units(inode);
        count == size.div_ceil(capacity) || (lenient && size == 0 && count <= 1)
    };
    let b1: Vec<u64> = with_units.iter().copied().filter(|inode| !live(*inode)).collect();
    let b2: Vec<u64> = with_units
        .iter()
        .copied()
        .filter(|inode| !facts.records.contains_key(inode))
        .collect();
    let mut c: BTreeSet<u64> = nlink_zero.iter().copied().collect();
    c.extend(b1.iter().copied());
    let in_queue_or_released = |inode: u64| queue.contains(&inode) || units(inode) == 0;
    let mut verdicts = vec![
        (
            "I-9.5",
            judge_all(facts.queue.iter().map(|inode| {
                (*inode, matches!(facts.records.get(inode), Some((0, _, _))))
            })),
        ),
        ("I-4.7/A", judge_all(nlink_zero.iter().map(|i| (*i, in_queue_or_released(*i))))),
        ("I-4.7/B1", judge_all(b1.iter().map(|i| (*i, in_queue_or_released(*i))))),
        ("I-4.7/B2", judge_all(b2.iter().map(|i| (*i, in_queue_or_released(*i))))),
        ("I-4.7/丙", judge_all(c.iter().map(|i| (*i, in_queue_or_released(*i))))),
        ("I-4.7/丁零轮", judge_all(nlink_zero.iter().map(|i| (*i, queue.contains(i))))),
        ("I-4.6/乙1", judge_all(nlink_zero.iter().map(|i| (*i, size_fits(*i, false))))),
        ("I-4.6/乙2", judge_all(facts.records.keys().map(|i| (*i, size_fits(*i, false))))),
        ("I-4.6/乙1z零轮", judge_all(nlink_zero.iter().map(|i| (*i, size_fits(*i, true))))),
        ("I-4.6/乙2z零轮", judge_all(facts.records.keys().map(|i| (*i, size_fits(*i, true))))),
        ("I-9.24", judge_all(std::iter::once((0, queue.len() <= 1)))),
        (
            "I-9.18近似",
            judge_all(facts.queue.iter().map(|inode| {
                let birth = facts.records.get(inode).map(|(_, _, birth)| *birth);
                (*inode, birth.is_some_and(|birth| facts.tombstones.contains(&(*inode, birth))))
            })),
        ),
        (
            "I-9.19近似",
            judge_all(facts.tombstones.iter().map(|(inode, birth)| {
                let holds = match facts.records.get(inode) {
                    Some((nlink, _, record_birth)) if record_birth == birth => *nlink == 0,
                    _ => true,
                };
                (*inode, holds)
            })),
        ),
    ];
    let has = facts.has_trees;
    verdicts.push((
        "I-4.4/甲",
        if !has {
            Verdict::NotApplicable
        } else if facts.queue_entry_width.is_some_and(|width| width != 8) {
            Verdict::Red(format!("队列条目宽 {:?}", facts.queue_entry_width))
        } else if !facts.logged_ops_root_is_zero {
            Verdict::Red("logged_ops 根非零".into())
        } else {
            Verdict::Holds
        },
    ));
    let tag4 = |missing_is_red: bool| match &facts.tag4 {
        Tag4::NoAccountingTree => Verdict::NotApplicable,
        Tag4::Missing if missing_is_red => Verdict::Red("标签 4 那一行缺".into()),
        Tag4::Missing => Verdict::NotApplicable,
        Tag4::Value(0) => Verdict::Holds,
        Tag4::Value(value) => Verdict::Red(format!("标签 4 = {value}")),
    };
    verdicts.push(("I-3.4/缺行判红", tag4(true)));
    verdicts.push(("I-3.4/缺行不适用", tag4(false)));
    // 这一版有一棵树读不全：每个候选都报判不了（F4 c 的两种报法在这里同形），不拿半版的事实判红。
    if facts.read_failure.is_some() {
        for (_, verdict) in &mut verdicts {
            *verdict = Verdict::NotApplicable;
        }
        verdicts.push(("读不全的版", Verdict::Red("读不全".into())));
    }
    verdicts
}

/// 一版的事实摘要：一行。
fn facts_line(facts: &VersionFacts) -> String {
    let mut line = format!(
        "txg={} F={} trees={} rec={:?} q={:?} qw={:?} units={:?} tag4={:?} lo0={} tomb={}",
        facts.txg,
        facts.rollback_floor,
        facts.has_trees,
        facts.records,
        facts.queue,
        facts.queue_entry_width,
        facts.units,
        facts.tag4,
        facts.logged_ops_root_is_zero,
        facts.tombstones.len()
    );
    if let Some(failure) = &facts.read_failure {
        let _ = write!(line, " READFAIL={failure}");
    }
    line
}

fn readable_roots_of(reader: &dyn PoolReader) -> Vec<RootRecord> {
    let Ok(system_configuration) = choose_system_configuration(reader) else {
        return Vec::new();
    };
    let mut roots = readable_roots(
        reader,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    );
    roots.sort_by_key(|root| (root.checkpoint_txg.0, root.instance.0));
    roots.dedup_by_key(|root| (root.checkpoint_txg.0, root.instance.0));
    roots
}

fn checker_reds(reader: &dyn singlefs_checker::image::ImageReader) -> Vec<&'static str> {
    check_pool_image(reader)
        .into_iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(name, _)| name)
        .collect()
}

/// 判定计数：(候选, 场合) → [成立, 红, 不适用]，与每格第一个红的例子。
#[derive(Default)]
struct Tally {
    counts: BTreeMap<(String, String), [u64; 3]>,
    first_red: BTreeMap<(String, String), String>,
    checker_reds: BTreeMap<String, u64>,
    states: u64,
    record_applied_states: u64,
    lines: Vec<String>,
}

impl Tally {
    fn note(&mut self, place: &str, facts: &VersionFacts, context: &str) {
        let verdicts = candidate_verdicts(facts);
        let mut reds = Vec::new();
        for (name, verdict) in verdicts {
            let key = (name.to_string(), place.to_string());
            let slot = self.counts.entry(key.clone()).or_default();
            match verdict {
                Verdict::Holds => slot[0] += 1,
                Verdict::Red(detail) => {
                    slot[1] += 1;
                    reds.push(name);
                    self.first_red
                        .entry(key)
                        .or_insert_with(|| format!("{context} {detail} | {}", facts_line(facts)));
                }
                Verdict::NotApplicable => slot[2] += 1,
            }
        }
        if place.starts_with("step") {
            self.lines.push(format!("{context} {place} RED={reds:?} | {}", facts_line(facts)));
        }
    }
    fn absorb(&mut self, other: Tally) {
        for (key, value) in other.counts {
            let slot = self.counts.entry(key).or_default();
            for index in 0..3 {
                slot[index] += value[index];
            }
        }
        for (key, value) in other.first_red {
            self.first_red.entry(key).or_insert(value);
        }
        for (key, value) in other.checker_reds {
            *self.checker_reds.entry(key).or_default() += value;
        }
        self.states += other.states;
        self.record_applied_states += other.record_applied_states;
        self.lines.extend(other.lines);
    }
}

fn role_of(root: &RootRecord, newest: &RootRecord) -> &'static str {
    if (root.checkpoint_txg, root.instance) == (newest.checkpoint_txg, newest.instance) {
        "newest"
    } else if root.checkpoint_txg.0 >= newest.rollback_floor.0 {
        "older>=F"
    } else {
        "older<F"
    }
}

fn judge_image(tally: &mut Tally, reader: &dyn PoolReader, prefix: &str, only_txg_above: u64, context: &str) {
    let roots = readable_roots_of(reader);
    let Some(newest) = roots.last().cloned() else {
        return;
    };
    for root in &roots {
        let role = role_of(root, &newest);
        if role != "newest" && root.checkpoint_txg.0 <= only_txg_above {
            continue;
        }
        let facts = facts_of_root(reader, root);
        tally.note(&format!("{prefix}-{role}"), &facts, context);
    }
}

fn run_history(label: &str, operations: Vec<HistoryOperation>, enumerate_crash_states: bool) -> Tally {
    let mut tally = Tally::default();
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations,
    };
    let execution = HistoryExecution {
        per_step_checker: PerStepChecker::Skipped,
        device_width: HistoryDeviceWidth::FourGibibytes,
        space_admission: singlefs_core::admission::SpaceAdmission::JudgedByTheFormula,
    };
    let stream = SharedStream::retaining_contents();
    let mut snapshots: Vec<(String, MemoryPool, usize)> = Vec::new();
    let run = execute_history_with(&history, execution, &stream, &mut |observation| {
        let position = match observation.position {
            StepPosition::StartingPoint => "start".to_string(),
            StepPosition::Operation(index) => format!(
                "op{index}:{:?}->{:?}",
                observation.operation.map(HistoryOperation::kind),
                observation.outcome.map(|outcome| format!("{outcome:?}").chars().take(60).collect::<String>())
            ),
        };
        snapshots.push((position, observation.image.clone(), stream.operation_count()));
    });
    tally.lines.push(format!("{label} ENDING {:?}", format!("{:?}", run.ending).chars().take(300).collect::<String>()));
    let geometry = HistoryDeviceWidth::FourGibibytes.fixed_geometry();
    let retained = stream.retained_operations();
    for index in 0..snapshots.len() {
        let (position, image, count) = &snapshots[index];
        let context = format!("{label} {position}");
        for name in checker_reds(image) {
            *tally.checker_reds.entry(format!("step:{name}")).or_default() += 1;
            tally.lines.push(format!("{context} CHECKER-RED {name}"));
        }
        judge_image(&mut tally, image, "step", 0, &context);
        if !enumerate_crash_states || index == 0 {
            continue;
        }
        let (_, base, base_count) = &snapshots[index - 1];
        if base_count == count {
            continue;
        }
        let base_newest = readable_roots_of(base).last().map_or(0, |root| root.checkpoint_txg.0);
        let (writes, segments) = writes_and_segments(&retained[*base_count..*count], &geometry);
        let expected = closed_form_state_count(&segments);
        let segment_lengths: Vec<usize> = segments.iter().map(Vec::len).collect();
        eprintln!("M3DC_PROGRESS {context} writes={} segments={segment_lengths:?} closed_form={expected}", writes.len());
        if std::env::var("M3DC_COUNT_ONLY").is_ok_and(|value| value == "1") {
            tally.states += expected;
            continue;
        }
        let mut states = 0u64;
        for segment_index in 0..segments.len() {
            let mut persisted = vec![false; writes.len()];
            for earlier in &segments[..segment_index] {
                for write in earlier {
                    persisted[*write] = true;
                }
            }
            let segment = &segments[segment_index];
            let first_mask = if segment_index == 0 { 0u64 } else { 1 };
            for mask in first_mask..(1u64 << segment.len()) {
                let mut state = persisted.clone();
                for (bit, write) in segment.iter().enumerate() {
                    state[*write] = mask & (1 << bit) != 0;
                }
                states += 1;
                let crash = CrashImage { base, writes: &writes, persisted: state };
                let state_context = format!("{context} crash seg{segment_index} mask{mask:#x}");
                for name in checker_reds(&crash) {
                    *tally.checker_reds.entry(format!("crash:{name}")).or_default() += 1;
                    tally.lines.push(format!("{state_context} CHECKER-RED {name}"));
                }
                let report = recover(&crash, JournalPolicy::Consult);
                let roots = readable_roots_of(&crash);
                if let Some((_, txg)) = report.effective_root {
                    if !roots.iter().any(|root| root.checkpoint_txg == CheckpointTxg(txg.0)) {
                        tally.record_applied_states += 1;
                    }
                }
                judge_image(&mut tally, &crash, "crash", base_newest, &state_context);
            }
        }
        assert_eq!(states, expected, "层 0 枚举的状态数等于闭式");
        tally.states += states;
    }
    tally
}

#[derive(Clone, Copy, Debug)]
enum Content {
    Empty,
    Small,
    Exact,
    Span(u64),
}

fn content(choice: Content) -> ContentChoice {
    let length = match choice {
        Content::Empty => ContentLength::Empty,
        Content::Small => ContentLength::InsideOneDataUnit { selector: 2999 },
        Content::Exact => ContentLength::ExactlyDataUnitPayloadCapacity,
        Content::Span(data_units) => ContentLength::SpanningDataUnitsThroughTheSequentialWrite {
            data_units,
            selector: data_units,
        },
    };
    ContentChoice { length, fill_seed: 7 }
}

fn contents_for_the_batch() -> Vec<Content> {
    if singlefs_core::m3dc::product_batch() == 144 {
        vec![Content::Empty, Content::Small, Content::Exact, Content::Span(145), Content::Span(290)]
    } else {
        vec![
            Content::Empty,
            Content::Small,
            Content::Exact,
            Content::Span(2),
            Content::Span(3),
            Content::Span(4),
            Content::Span(7),
        ]
    }
}

/// 合法历史：第一个文件写成给定内容 → unlink → 尾巴。尾巴放开用户动作：冷启动、挂载（续做）、建 inode 之后回退到环里第 k 新的根，
/// 再挂载续做 / 再删 / 覆盖写 / 建 inode / 冷启动再挂载 / 挂载再删。
fn legal_plan() -> Vec<(String, Vec<HistoryOperation>)> {
    use HistoryOperation as Op;
    let small = content(Content::Small);
    let mut tails: Vec<(String, Vec<Op>)> = vec![
        ("none".into(), vec![]),
        ("cold".into(), vec![Op::ColdStartRecover]),
        ("mount".into(), vec![Op::CloseAndMountWritable]),
        (
            "inode-mount-inode".into(),
            vec![Op::CreateInode, Op::CloseAndMountWritable, Op::CreateInode],
        ),
    ];
    let afters: Vec<(&str, Vec<Op>)> = vec![
        ("", vec![]),
        ("mount", vec![Op::CloseAndMountWritable]),
        ("unlink", vec![Op::Unlink]),
        ("overwrite", vec![Op::PublishOverwrite(small)]),
        ("inode", vec![Op::CreateInode]),
        ("cold-mount", vec![Op::ColdStartRecover, Op::CloseAndMountWritable]),
        ("mount-unlink", vec![Op::CloseAndMountWritable, Op::Unlink]),
    ];
    for k in 1..=12u64 {
        for (name, after) in &afters {
            let mut tail = vec![
                Op::CreateInode,
                Op::RollBackWhileMounted(RollbackTargetChoice::RingRoot { index_from_newest: k }),
            ];
            tail.extend(after.iter().cloned());
            tails.push((format!("inode-rb{k}-{name}"), tail));
        }
    }
    let mut plan = Vec::new();
    for choice in contents_for_the_batch() {
        for (tail_name, tail) in &tails {
            let mut operations = vec![Op::PublishOverwrite(content(choice)), Op::Unlink];
            operations.extend(tail.iter().cloned());
            plan.push((format!("{choice:?}/{tail_name}"), operations));
        }
    }
    plan
}

/// 改坏的镜像：写成给定内容之后 unlink 一次（改坏的那一步模型对不上、历史停在那里，镜像照样判）。
fn mutation_plan() -> Vec<(String, Vec<HistoryOperation>)> {
    contents_for_the_batch()
        .into_iter()
        .map(|choice| {
            (
                format!("{choice:?}/unlink"),
                vec![HistoryOperation::PublishOverwrite(content(choice)), HistoryOperation::Unlink],
            )
        })
        .collect()
}

/// 随机历史：快档那组比重（`GenerationWeights::BROAD`），种子从 `M3DC_SEED_BASE` 起 `M3DC_SEEDS` 个、每段 30 步，只留带 unlink 的。
fn random_plan() -> Vec<(String, Vec<HistoryOperation>)> {
    use singlefs_harness::history::{generate_history_with_weights, GenerationWeights, HistoryOperationKind};
    let base: u64 = std::env::var("M3DC_SEED_BASE").ok().and_then(|v| v.parse().ok()).unwrap_or(1_000);
    let seeds: u64 = std::env::var("M3DC_SEEDS").ok().and_then(|v| v.parse().ok()).unwrap_or(200);
    (base..base + seeds)
        .map(|seed| (seed, generate_history_with_weights(HistorySeed(seed), 30, &GenerationWeights::BROAD)))
        .filter(|(_, history)| {
            history.operations.iter().any(|operation| operation.kind() == HistoryOperationKind::Unlink)
                && history.starting_point == HistoryStartingPoint::AfterFirstFile
        })
        .map(|(seed, history)| (format!("seed{seed}"), history.operations))
        .collect()
}

#[test]
fn m3dc_candidates_on_histories() {
    let plan_name = std::env::var("M3DC_PLAN").unwrap_or_else(|_| "smoke".into());
    let crash = std::env::var("M3DC_CRASH").map_or(true, |value| value != "0");
    let threads: usize = std::env::var("M3DC_THREADS").ok().and_then(|v| v.parse().ok()).unwrap_or(1);
    let limit: usize = std::env::var("M3DC_LIMIT").ok().and_then(|v| v.parse().ok()).unwrap_or(usize::MAX);
    let skip: usize = std::env::var("M3DC_SKIP").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
    let mut plan = match plan_name.as_str() {
        "legal" => legal_plan(),
        "mut" => mutation_plan(),
        "random" => random_plan(),
        _ => legal_plan().into_iter().take(1).collect(),
    };
    plan = plan.into_iter().skip(skip).take(limit).collect();
    let started = std::time::Instant::now();
    let next = std::sync::atomic::AtomicUsize::new(0);
    let total = std::sync::Mutex::new(Tally::default());
    std::thread::scope(|scope| {
        for _ in 0..threads {
            scope.spawn(|| loop {
                let index = next.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                let Some((label, operations)) = plan.get(index).cloned() else {
                    break;
                };
                let tally = run_history(&label, operations, crash);
                total.lock().expect("锁").absorb(tally);
            });
        }
    });
    let total = total.into_inner().expect("锁");
    let mutation = std::env::var("M3DC_MUT").unwrap_or_default();
    println!(
        "M3DC_SUMMARY plan={plan_name} batch={} mutation={mutation:?} histories={} crash={crash} states={} record_applied_states={} elapsed_seconds={:.1}",
        singlefs_core::m3dc::product_batch(),
        plan.len(),
        total.states,
        total.record_applied_states,
        started.elapsed().as_secs_f64()
    );
    for ((name, place), [holds, red, not_applicable]) in &total.counts {
        println!("M3DC_COUNT {name}\t{place}\tholds={holds}\tred={red}\tna={not_applicable}");
    }
    for ((name, place), example) in &total.first_red {
        println!("M3DC_FIRST_RED {name}\t{place}\t{example}");
    }
    for (name, count) in &total.checker_reds {
        println!("M3DC_CHECKER_RED {name}\t{count}");
    }
    if std::env::var("M3DC_LINES").is_ok_and(|value| value == "1") {
        for line in &total.lines {
            println!("M3DC_LINE {line}");
        }
    }
}

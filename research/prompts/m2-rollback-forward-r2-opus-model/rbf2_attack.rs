//! m2-rollback-forward-r2 云端攻方腿的原型用例（冻结副本的拷贝上，不入库）。
//! 原型：`singlefs_core::mount::prototype_rollback_in_mount`（挂着的时候向前发布的回退）、`prototype_unmount_b1`（B1）、
//! `recovery::PROTOTYPE_FLOOR_RULE_MAX`（C419 取最大值）、`rollback_witness::PROTOTYPE_WITNESS_AND_TRUNCATION_DELETED`（删见证与截断）。
//! 每条用例打一行 `RBF2 ...` 给报告抄。

mod common;

use common::{build_pool, file_content, parameters, FIXED_WRITE_TIME_SECONDS};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::allocator::PoolAllocator;
use singlefs_core::mount::{
    mount_rollback, mount_writable, prototype_raise_rollback_floor, prototype_rollback_in_mount,
    prototype_unmount_b1, Mounted, Rbf2Arm, Rbf2Counts, RollbackTarget,
    ShadowLedger,
};
use singlefs_core::recovery::PROTOTYPE_HOLD_UNTIL_COVERED;
use singlefs_core::recovery::{
    choose_system_configuration, effective_rollback_floor, instance_table_chain_of_root,
    readable_roots, rebuild_version, recover, scan_journal, walk_to_file, JournalPolicy,
    RebuiltVersion, RecoveryOutcome, PROTOTYPE_FLOOR_RULE_MAX,
};
use singlefs_core::rollback_witness::PROTOTYPE_WITNESS_AND_TRUNCATION_DELETED;
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::transaction::{
    publish_new_inodes, publish_overwrite, publish_sequential_write, FirstFile,
    PoolWriter, TransactionOutput,
};
use singlefs_harness::crash::{MemoryPool, SparseBlockDevice};
use singlefs_harness::{RecordingBlockDevice, SharedStream};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::Ordering;
use std::sync::OnceLock;

type Dev = RecordingBlockDevice<SparseBlockDevice>;

static BASE: OnceLock<(MemoryPool, PoolAllocator, TransactionOutput)> = OnceLock::new();

/// 起点池只建一次（mkfs、暖机、A = 实例 1 第 3 代），之后每段历史在内存里拷。
fn base() -> &'static (MemoryPool, PoolAllocator, TransactionOutput) {
    BASE.get_or_init(|| {
        let pool = build_pool("rbf2");
        (pool.memory_pool(), pool.allocator.clone(), pool.output.clone())
    })
}

fn set_rule(max: bool) {
    PROTOTYPE_FLOOR_RULE_MAX.store(max, Ordering::SeqCst);
}
fn set_deleted(deleted: bool) {
    PROTOTYPE_WITNESS_AND_TRUNCATION_DELETED.store(deleted, Ordering::SeqCst);
}

pub fn content_of(seed: usize) -> Vec<u8> {
    (0..2600 + seed)
        .map(|index| u8::try_from((index * 7 + seed * 11) % 239).expect("小于 256"))
        .collect()
}
/// 三个数据单元的内容（顺序写）。
pub fn long_content_of(seed: usize) -> Vec<u8> {
    (0..70000 + seed)
        .map(|index| u8::try_from((index * 13 + seed * 5) % 241).expect("小于 256"))
        .collect()
}

pub struct Session {
    pub devices: Vec<(DeviceIdentity, Dev)>,
    pub stream: SharedStream,
    pub contents: BTreeMap<(u32, u64), Vec<u8>>,
    pub seed: usize,
}

pub struct Open {
    pub allocator: PoolAllocator,
    pub current: TransactionOutput,
    pub instance: InstanceGeneration,
}

impl Session {
    pub fn new() -> (Session, Open) {
        let (image, allocator, output) = base();
        let stream = SharedStream::retaining_contents();
        let devices = common::crash_state_devices(image, &[], &[], &stream);
        let mut contents = BTreeMap::new();
        contents.insert((1, 3), file_content());
        (
            Session { devices, stream, contents, seed: 100 },
            Open { allocator: allocator.clone(), current: output.clone(), instance: InstanceGeneration(1) },
        )
    }

    pub fn from_image(image: &MemoryPool, contents: &BTreeMap<(u32, u64), Vec<u8>>) -> Session {
        let stream = SharedStream::retaining_contents();
        Session {
            devices: common::crash_state_devices(image, &[], &[], &stream),
            stream,
            contents: contents.clone(),
            seed: 500,
        }
    }

    pub fn image(&self) -> MemoryPool {
        common::memory_pool_of_sparse_devices(&self.devices)
    }

    /// 录制流从这里重新起（崩溃枚举的基线）。
    pub fn restart_stream(&mut self) -> MemoryPool {
        let base = self.image();
        self.stream = SharedStream::retaining_contents();
        self.devices = common::crash_state_devices(&base, &[], &[], &self.stream);
        base
    }

    fn content_of_version(&self, instance: u32, txg: u64) -> Vec<u8> {
        self.contents.get(&(instance, txg)).cloned().unwrap_or_else(|| {
            let root = roots_of(&self.image()).into_iter().find(|r| (r.instance.0, r.checkpoint_txg.0) == (instance, txg)).expect("根在环里");
            walk_to_file(&self.image(), &root, &mut 0).ok().flatten().expect("读得出")
        })
    }

    fn note(&mut self, root: &RootRecord, content: Vec<u8>) {
        self.contents.insert((root.instance.0, root.checkpoint_txg.0), content);
    }

    pub fn try_mount(&mut self) -> Result<Open, String> {
        let mounted = mount_writable(&parameters(), &mut self.devices).map_err(|e| format!("{e:?}"))?;
        Ok(self.adopt(mounted))
    }
    pub fn mount(&mut self) -> Open {
        self.try_mount().expect("可写挂载")
    }
    fn adopt(&mut self, mounted: Mounted) -> Open {
        let effective = mounted.output.effective_root;
        let content = self.content_of_version(effective.instance.0, effective.checkpoint_txg.0);
        for published in std::iter::once(&mounted.output.row_publish).chain(&mounted.output.warm_up_publishes) {
            self.note(published.root(), content.clone());
        }
        let instance = mounted.output.instance;
        Open { allocator: mounted.allocator, current: mounted.current.into_file_version().expect("带文件"), instance }
    }

    pub fn try_overwrite(&mut self, open: &mut Open) -> Result<u64, String> {
        self.seed += 1;
        let content = content_of(self.seed);
        let parameters = parameters();
        let mut writer = PoolWriter::new(&parameters, self.devices.as_mut_slice());
        let output = publish_overwrite(&mut writer, &mut open.allocator, &open.current,
            FirstFile { content: &content, write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60 }, open.instance)
            .map_err(|error| format!("{error:?}"))?;
        self.note(&output.root, content);
        open.current = output;
        Ok(open.current.root.checkpoint_txg.0)
    }
    pub fn try_seq_write(&mut self, open: &mut Open) -> Result<u64, String> {
        self.seed += 1;
        let content = long_content_of(self.seed);
        let parameters = parameters();
        let mut writer = PoolWriter::new(&parameters, self.devices.as_mut_slice());
        let output = publish_sequential_write(&mut writer, &mut open.allocator, &open.current,
            FirstFile { content: &content, write_time_seconds: FIXED_WRITE_TIME_SECONDS + 70 }, open.instance)
            .map_err(|error| format!("{error:?}"))?;
        self.note(&output.root, content);
        open.current = output;
        Ok(open.current.root.checkpoint_txg.0)
    }
    pub fn try_new_inodes(&mut self, open: &mut Open, count: u64) -> Result<u64, String> {
        let content = self.content_of_version(open.current.root.instance.0, open.current.root.checkpoint_txg.0);
        let parameters = parameters();
        let mut writer = PoolWriter::new(&parameters, self.devices.as_mut_slice());
        let output = publish_new_inodes(&mut writer, &mut open.allocator, &open.current, count,
            FIXED_WRITE_TIME_SECONDS + 90, open.instance).map_err(|error| format!("{error:?}"))?;
        self.note(&output.root, content);
        open.current = output;
        Ok(open.current.root.checkpoint_txg.0)
    }

    /// 挂着的时候向前回退到 `target`。
    pub fn rollback(&mut self, open: &mut Open, target: (u32, u64), arm: Rbf2Arm) -> Result<Rbf2Counts, String> {
        let parameters = parameters();
        let counts = prototype_rollback_in_mount(
            &parameters,
            &mut self.devices,
            &mut open.allocator,
            &mut open.current,
            RollbackTarget { instance: InstanceGeneration(target.0), checkpoint_txg: CheckpointTxg(target.1) },
            arm,
        )
        .map_err(|error| format!("{error:?}"))?;
        let content = self.content_of_version(target.0, target.1);
        let root = open.current.root;
        self.note(&root, content);
        Ok(counts)
    }

    /// B1 卸载（`literal` 按字面上限判）。交回这一串的 txg。
    pub fn unmount_b1(&mut self, open: &mut Open, literal: bool) -> Result<Vec<u64>, String> {
        let content = self.content_of_version(open.current.root.instance.0, open.current.root.checkpoint_txg.0);
        let raised = prototype_unmount_b1(&parameters(), &mut self.devices, &mut open.allocator, &mut open.current, literal)
            .map_err(|error| format!("{error:?}"))?;
        for published in &raised.publishes {
            self.note(&published.root, content.clone());
        }
        Ok(raised.publishes.iter().map(|p| p.root.checkpoint_txg.0).collect())
    }

    /// 同一个函数当「准入抬 F」用：抬到 `floor`，`bypass` 不判上限。
    pub fn raise(&mut self, open: &mut Open, floor: u64, bypass: bool) -> Result<Vec<u64>, String> {
        let content = self.content_of_version(open.current.root.instance.0, open.current.root.checkpoint_txg.0);
        let raised = prototype_raise_rollback_floor(&parameters(), &mut self.devices, &mut open.allocator,
            &mut open.current, CheckpointTxg(floor), ShadowLedger::On, bypass)
            .map_err(|error| format!("{error:?}"))?;
        for published in &raised.publishes {
            self.note(&published.root, content.clone());
        }
        Ok(raised.publishes.iter().map(|p| p.root.checkpoint_txg.0).collect())
    }

    /// 今天的挂载时回退（造旧镜像用）。
    pub fn old_mount_rollback(&mut self, target: (u32, u64)) -> Result<Open, String> {
        let mounted = mount_rollback(&parameters(), &mut self.devices,
            RollbackTarget { instance: InstanceGeneration(target.0), checkpoint_txg: CheckpointTxg(target.1) }, ShadowLedger::On)
            .map_err(|e| format!("{e:?}"))?;
        let content = self.content_of_version(target.0, target.1);
        for published in std::iter::once(&mounted.output.row_publish).chain(&mounted.output.warm_up_publishes) {
            self.note(published.root(), content.clone());
        }
        let instance = mounted.output.instance;
        Ok(Open { allocator: mounted.allocator, current: mounted.current.into_file_version().expect("带文件"), instance })
    }
}

pub fn roots_of(image: &MemoryPool) -> Vec<RootRecord> {
    let system_configuration = choose_system_configuration(image).expect("系统配置");
    let mut roots = readable_roots(image, &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes, &system_configuration.immutable.filesystem_identifier);
    roots.sort_by_key(|root| (root.checkpoint_txg, root.instance));
    roots
}

pub fn device_of_txg(txg: u64) -> DeviceIdentity {
    let target = target_for_publish(CheckpointTxg(txg), parameters().geometry.root_ring_slots_per_region);
    parameters().region_devices[usize::try_from(target.region).expect("区域")]
}

pub fn corrupt_roots(image: &mut MemoryPool, txgs: &[u64]) {
    let slots_per_region = parameters().geometry.root_ring_slots_per_region;
    for txg in txgs {
        let target = target_for_publish(CheckpointTxg(*txg), slots_per_region);
        image.flip_byte(device_of_txg(*txg), slot_offset(target, 4096), 100);
    }
}

pub fn floor_of(image: &MemoryPool) -> u64 {
    let system_configuration = choose_system_configuration(image).expect("系统配置");
    effective_rollback_floor(image, &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes, &system_configuration.immutable.filesystem_identifier).0
}

pub fn checker_reds(image: &MemoryPool) -> Vec<String> {
    check_pool_image(image)
        .into_iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(detail) => Some(format!("{invariant}:{}", detail.chars().take(120).collect::<String>())),
            _ => None,
        })
        .collect()
}

pub fn checker_red_names(image: &MemoryPool) -> Vec<String> {
    checker_reds(image).into_iter().map(|red| red.split(':').next().unwrap_or("").to_string()).collect()
}

pub fn recover_and_judge(image: &MemoryPool, contents: &BTreeMap<(u32, u64), Vec<u8>>) -> Result<(u32, u64), String> {
    let report = recover(image, JournalPolicy::Consult);
    let effective = report.effective_root;
    match report.outcome {
        RecoveryOutcome::FileRead { root, content } => {
            let root = effective.unwrap_or(root);
            match contents.get(&(root.0 .0, root.1 .0)) {
                Some(expected) if *expected == content => Ok((root.0 .0, root.1 .0)),
                Some(_) => Err(format!("落在 ({}, {})，读回的内容不是那一版的", root.0 .0, root.1 .0)),
                None => Err(format!("落在 ({}, {})，没有这一版的记录", root.0 .0, root.1 .0)),
            }
        }
        RecoveryOutcome::NoFile { root } => {
            if root.1 .0 <= 2 && !contents.contains_key(&(root.0 .0, root.1 .0)) {
                Ok((root.0 .0, root.1 .0))
            } else {
                Err(format!("落在 ({}, {})，报没有文件", root.0 .0, root.1 .0))
            }
        }
        RecoveryOutcome::Failed { root, failure } => Err(format!("落在 {:?}，走读失败：{}",
            root.map(|(i, t)| (i.0, t.0)), format!("{failure:?}").chars().take(160).collect::<String>())),
    }
}

/// 对环里每条根 X，把比它新的根全改坏，恢复落在哪、对不对：交回坏格。
pub fn landing_bad(image: &MemoryPool, contents: &BTreeMap<(u32, u64), Vec<u8>>) -> (usize, Vec<String>) {
    let roots = roots_of(image);
    let mut bad = Vec::new();
    for index in (0..roots.len()).rev() {
        let newer: Vec<u64> = roots[index + 1..].iter().map(|r| r.checkpoint_txg.0).collect();
        let mut damaged = image.clone();
        corrupt_roots(&mut damaged, &newer);
        if let Err(reason) = recover_and_judge(&damaged, contents) {
            bad.push(format!("k={} {reason}", newer.len()));
        }
    }
    (roots.len(), bad)
}

/// 候选集：环里、按最新根的实例表有效、txg ≥ F_生效（盘上）。
pub fn candidates(image: &MemoryPool) -> Vec<RootRecord> {
    let roots = roots_of(image);
    let Some(newest) = roots.last().copied() else { return Vec::new() };
    let table = instance_table_chain_of_root(image, &newest).map(|c| c.records.rows).unwrap_or_default();
    let floor = floor_of(image);
    roots.into_iter()
        .filter(|root| root.checkpoint_txg.0 >= floor)
        .filter(|root| !table.iter().any(|row| row.instance == root.instance && root.checkpoint_txg > row.selected_root_txg))
        .collect()
}

/// 共用问句：环里每条根沿它走到文件、重建整版，读得对不对。交回 (候选里坏的, 非候选里坏的)。
pub fn ring_audit(image: &MemoryPool, contents: &BTreeMap<(u32, u64), Vec<u8>>) -> (Vec<String>, Vec<String>) {
    let candidate_keys: BTreeSet<(u32, u64)> = candidates(image).iter().map(|r| (r.instance.0, r.checkpoint_txg.0)).collect();
    let system_configuration = choose_system_configuration(image).expect("系统配置");
    let records = scan_journal(image, &system_configuration);
    let mut bad_candidates = Vec::new();
    let mut bad_others = Vec::new();
    for root in roots_of(image) {
        let key = (root.instance.0, root.checkpoint_txg.0);
        let walked = walk_to_file(image, &root, &mut 0);
        let verdict = match (&walked, contents.get(&key)) {
            (Ok(Some(content)), Some(expected)) if content == expected => None,
            (Ok(None), None) if key.1 <= 2 => None,
            (Ok(Some(_)), Some(_)) => Some("内容不对".to_string()),
            (Ok(Some(_)), None) => None, // 没记过内容的根（外来）：只看读不读得出
            (Ok(None), _) => Some("报没有文件".to_string()),
            (Err(failure), _) => Some(format!("走读失败 {}", format!("{failure:?}").chars().take(80).collect::<String>())),
        };
        let record = records.values().filter(|r| r.instance == root.instance && r.checkpoint_txg == root.checkpoint_txg)
            .max_by_key(|r| r.counter).or_else(|| records.values().max_by_key(|r| r.counter)).cloned();
        let rebuilt = match rebuild_version(image, &root, record) {
            Ok(RebuiltVersion::WithFile(_)) | Ok(RebuiltVersion::WithoutFile) => None,
            Err(failure) => Some(format!("重建失败 {}", format!("{failure:?}").chars().take(80).collect::<String>())),
        };
        if let Some(reason) = verdict.or(rebuilt) {
            let line = format!("({},{}) {reason}", key.0, key.1);
            if candidate_keys.contains(&key) { bad_candidates.push(line) } else { bad_others.push(line) }
        }
    }
    (bad_candidates, bad_others)
}

/// G2：内存里的现行版本与分配器，和盘上重建出来的那一版逐项比。交回不同的项。
pub fn memory_vs_disk(image: &MemoryPool, open: &Open) -> Vec<String> {
    let mut diffs = Vec::new();
    let system_configuration = choose_system_configuration(image).expect("系统配置");
    let records = scan_journal(image, &system_configuration);
    let root = open.current.root;
    let record = records.values().filter(|r| r.instance == root.instance && r.checkpoint_txg == root.checkpoint_txg)
        .max_by_key(|r| r.counter).cloned();
    let rebuilt = match rebuild_version(image, &root, record) {
        Ok(RebuiltVersion::WithFile(output)) => output,
        other => return vec![format!("盘上重建不出：{other:?}").chars().take(200).collect()],
    };
    let key_units = |output: &TransactionOutput| {
        let mut units: Vec<(String, u64, Vec<u8>)> = output.units.iter().map(|u| (format!("{:?}", u.identity), u.slot.0, u.bytes.clone())).collect();
        units.sort();
        units
    };
    if key_units(&rebuilt) != key_units(&open.current) { diffs.push("units".into()); }
    if rebuilt.allocation_records != open.current.allocation_records { diffs.push("allocation_records(版本)".into()); }
    let mut in_memory = open.allocator.records().to_vec();
    in_memory.sort_by_key(|record| (record.device, record.slot));
    if rebuilt.allocation_records != in_memory {
        let first = rebuilt.allocation_records.iter().zip(&in_memory).find(|(a, b)| a != b).map(|(a, b)| format!("{a:?} vs {b:?}"));
        diffs.push(format!("allocation_records(分配器) disk={} mem={} first={first:?}", rebuilt.allocation_records.len(), in_memory.len()));
    }
    if rebuilt.accounting_entries != open.current.accounting_entries { diffs.push("accounting_entries".into()); }
    if rebuilt.data_pointers != open.current.data_pointers { diffs.push("data_pointers".into()); }
    if rebuilt.inode_leaf_container_contents() != open.current.inode_leaf_container_contents() { diffs.push("inode_leaf_containers".into()); }
    if rebuilt.inode_record != open.current.inode_record { diffs.push("inode_record".into()); }
    if rebuilt.tree_table_entries != open.current.tree_table_entries { diffs.push("tree_table_entries".into()); }
    diffs
}

fn summary(session: &Session, open: &Open) -> String {
    let image = session.image();
    let (bad_candidates, bad_others) = ring_audit(&image, &session.contents);
    let (roots, landing) = landing_bad(&image, &session.contents);
    format!(
        "newest=({},{}) F={} ring_roots={roots} cand_bad={:?} other_bad={} landing_bad={:?} mem_vs_disk={:?} checker={:?}",
        open.current.root.instance.0, open.current.root.checkpoint_txg.0, floor_of(&image),
        bad_candidates, bad_others.len(), landing, memory_vs_disk(&image, open), checker_red_names(&image)
    )
}

/// 冒烟：实例 2 里 B C D，挂着回退到 A、写、回退到 D、写、回退到第一次回退的根。
#[test]
fn g1_smoke() {
    set_rule(false);
    set_deleted(false);
    for arm in [Rbf2Arm::Full, Rbf2Arm::WithoutProtectionCheck, Rbf2Arm::LiteralWholeTreeDiff, Rbf2Arm::WatermarkOfNewestOnly] {
        let (mut session, open0) = Session::new();
        drop(open0);
        let mut open = session.mount();
        let b = session.try_overwrite(&mut open).expect("B");
        let _c = session.try_overwrite(&mut open).expect("C");
        let d = session.try_overwrite(&mut open).expect("D");
        let first = session.rollback(&mut open, (1, 3), arm);
        let rb1 = open.current.root.checkpoint_txg.0;
        println!("RBF2 g1_smoke arm={arm:?} B={b} D={d} rollback_to_A={first:?} :: {}", summary(&session, &open));
        let e = session.try_overwrite(&mut open);
        println!("RBF2 g1_smoke arm={arm:?} write_after={e:?} :: {}", summary(&session, &open));
        let second = session.rollback(&mut open, (2, d), arm);
        println!("RBF2 g1_smoke arm={arm:?} rollback_to_D={second:?} :: {}", summary(&session, &open));
        let f = session.try_overwrite(&mut open);
        let third = session.rollback(&mut open, (2, rb1), arm);
        println!("RBF2 g1_smoke arm={arm:?} write={f:?} rollback_to_rb1={third:?} :: {}", summary(&session, &open));
        for _ in 0..4 { let _ = session.try_overwrite(&mut open); }
        println!("RBF2 g1_smoke arm={arm:?} after_4_writes :: {}", summary(&session, &open));
    }
}

/// 候选集里每条根的 inode 树：同一个 inode 号在两条候选根下对象出生代不同 ⇒ 号被重发。
pub fn inode_number_conflicts(image: &MemoryPool) -> Vec<String> {
    let system_configuration = choose_system_configuration(image).expect("系统配置");
    let records = scan_journal(image, &system_configuration);
    let mut births: BTreeMap<u64, BTreeSet<u64>> = BTreeMap::new();
    for root in candidates(image) {
        let record = records.values().filter(|r| r.instance == root.instance && r.checkpoint_txg == root.checkpoint_txg)
            .max_by_key(|r| r.counter).or_else(|| records.values().max_by_key(|r| r.counter)).cloned();
        if let Ok(RebuiltVersion::WithFile(output)) = rebuild_version(image, &root, record) {
            for container in output.inode_leaf_container_contents() {
                for inode in container.records {
                    births.entry(inode.inode).or_default().insert(inode.object_birth.0);
                }
            }
        }
    }
    births.into_iter().filter(|(_, b)| b.len() > 1).map(|(i, b)| format!("inode {i} births {b:?}")).collect()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Step {
    Overwrite,
    NewInodes,
    SeqWrite,
    /// 回退到候选里 txg 第二新的那一条（上一个状态）。
    RollbackPrevious,
    /// 回退到候选里带文件的最旧那一条。
    RollbackOldest,
    /// 回退到候选里居中的那一条。
    RollbackMiddle,
    /// 关掉、重开（新实例）。
    Remount,
}

const STEPS: [Step; 7] = [Step::Overwrite, Step::NewInodes, Step::SeqWrite, Step::RollbackPrevious,
    Step::RollbackOldest, Step::RollbackMiddle, Step::Remount];

fn pick_target(session: &Session, open: &Open, step: Step) -> Option<(u32, u64)> {
    let image = session.image();
    let current = (open.current.root.instance.0, open.current.root.checkpoint_txg.0);
    let with_file: Vec<(u32, u64)> = candidates(&image).iter()
        .map(|r| (r.instance.0, r.checkpoint_txg.0))
        .filter(|key| key.1 >= 3 && *key != current)
        .collect();
    if with_file.is_empty() { return None; }
    match step {
        Step::RollbackPrevious => with_file.last().copied(),
        Step::RollbackOldest => with_file.first().copied(),
        _ => with_file.get(with_file.len() / 2).copied(),
    }
}

/// 执行一步；回退那一步交回 (目标, 结果, 回退之后内存与盘上的差)。
fn do_step(session: &mut Session, open: &mut Open, step: Step, arm: Rbf2Arm) -> String {
    match step {
        Step::Overwrite => format!("O{:?}", session.try_overwrite(open).map_err(|e| e.chars().take(80).collect::<String>())),
        Step::NewInodes => format!("I{:?}", session.try_new_inodes(open, 3).map_err(|e| e.chars().take(80).collect::<String>())),
        Step::SeqWrite => format!("S{:?}", session.try_seq_write(open).map_err(|e| e.chars().take(80).collect::<String>())),
        Step::Remount => {
            let reopened = session.try_mount();
            match reopened {
                Ok(new_open) => { *open = new_open; format!("M(inst {})", open.instance.0) }
                Err(e) => format!("M(err {e})"),
            }
        }
        _ => {
            let Some(target) = pick_target(session, open, step) else { return "R(none)".into() };
            let result = session.rollback(open, target, arm);
            let mem = if result.is_ok() { memory_vs_disk(&session.image(), open) } else { Vec::new() };
            match result {
                Ok(counts) => format!("R{target:?}->{} rel={} rev={} skip={} lit={} mem={mem:?}", counts.txg,
                    counts.released_placements, counts.revived_placements, counts.revive_skipped,
                    counts.whole_tree_diff_referenced_by_the_new_root),
                Err(e) => format!("R{target:?} refused {}", e.chars().take(100).collect::<String>()),
            }
        }
    }
}

fn sequences(max_len: usize) -> Vec<Vec<Step>> {
    let mut all = vec![Vec::new()];
    let mut frontier = vec![Vec::new()];
    for _ in 0..max_len {
        let mut next = Vec::new();
        for prefix in &frontier {
            for step in STEPS {
                let mut s: Vec<Step> = prefix.clone();
                s.push(step);
                next.push(s);
            }
        }
        all.extend(next.iter().cloned());
        frontier = next;
    }
    all
}

/// G1 / G2：前缀两种 × 第一次回退三种目标 × 之后用户动作放开扫（长度 0..=3，7 种动作）。每段历史末尾：环里每条根读得对不对、
/// 逐条改坏更新的根之后恢复落在哪、候选根之间 inode 号冲突、checker；每次回退之后内存与盘上逐项比。
#[test]
fn g1_sweep_user_steps() {
    set_rule(false);
    set_deleted(false);
    let max_len: usize = std::env::var("RBF2_SWEEP_LEN").ok().and_then(|v| v.parse().ok()).unwrap_or(3);
    let arms: Vec<Rbf2Arm> = match std::env::var("RBF2_SWEEP_ARM").as_deref() {
        Ok("noprotect") => vec![Rbf2Arm::WithoutProtectionCheck],
        _ => vec![Rbf2Arm::Full],
    };
    let mut totals: BTreeMap<String, u64> = BTreeMap::new();
    for arm in arms {
        for prefix in ["overwrites", "mixed"] {
            for first in [Step::RollbackPrevious, Step::RollbackOldest, Step::RollbackMiddle] {
                for suffix in sequences(max_len) {
                    let (mut session, open0) = Session::new();
                    drop(open0);
                    let mut open = session.mount();
                    let mut log = Vec::new();
                    let prefix_steps: &[Step] = if prefix == "overwrites" {
                        &[Step::Overwrite, Step::Overwrite, Step::Overwrite]
                    } else {
                        &[Step::SeqWrite, Step::NewInodes, Step::Overwrite]
                    };
                    for step in prefix_steps.iter().copied().chain(std::iter::once(first)).chain(suffix.iter().copied()) {
                        log.push(do_step(&mut session, &mut open, step, arm));
                    }
                    let image = session.image();
                    let (bad_candidates, bad_others) = ring_audit(&image, &session.contents);
                    let (_, landing) = landing_bad(&image, &session.contents);
                    let conflicts = inode_number_conflicts(&image);
                    let reds = checker_red_names(&image);
                    let mem_bad = log.iter().any(|l| l.contains("mem=[\"") );
                    let refused = log.iter().filter(|l| l.contains("refused")).count();
                    for l in &log {
                        if l.starts_with("R(none)") { *totals.entry("rollback_none".into()).or_default() += 1; }
                        if l.starts_with('R') && l.contains("->") {
                            *totals.entry("rollback_done".into()).or_default() += 1;
                            let field = |key: &str| l.split(key).nth(1).and_then(|rest| rest.split(' ').next()).and_then(|v| v.parse::<u64>().ok()).unwrap_or(0);
                            *totals.entry("revived_sum".into()).or_default() += field(" rev=");
                            *totals.entry("released_sum".into()).or_default() += field(" rel=");
                            if field(" rev=") > 0 { *totals.entry("rollbacks_with_revive".into()).or_default() += 1; }
                            if l.starts_with("R(1,") || l.contains("R((1,") { *totals.entry("rollback_to_instance_1".into()).or_default() += 1; }
                        }
                        if l.starts_with("M(inst") { *totals.entry("remounts".into()).or_default() += 1; }
                    }
                    *totals.entry("histories".into()).or_default() += 1;
                    for (name, hit) in [("cand_bad", !bad_candidates.is_empty()), ("landing_bad", !landing.is_empty()),
                        ("inode_conflict", !conflicts.is_empty()), ("mem_vs_disk", mem_bad), ("checker_red", !reds.is_empty()),
                        ("rollback_refused", refused > 0)] {
                        if hit { *totals.entry(name.into()).or_default() += 1; }
                    }
                    *totals.entry("other_bad_roots".into()).or_default() += bad_others.len() as u64;
                    if !bad_candidates.is_empty() || !landing.is_empty() || !conflicts.is_empty() || mem_bad || !reds.is_empty() || refused > 0 {
                        println!("RBF2 g1_sweep arm={arm:?} prefix={prefix} first={first:?} suffix={suffix:?} log={log:?} cand_bad={bad_candidates:?} landing_bad={landing:?} conflicts={conflicts:?} checker={reds:?}");
                    }
                }
            }
        }
    }
    println!("RBF2 g1_sweep totals={totals:?}");
}

use singlefs_harness::crash::{
    enumerate_layer0_selecting_versions_observing_each_state, writes_and_segments, CrashImage,
    PublishedVersion, RetainedWrite,
};
use singlefs_harness::segments::StepKind;

fn versions_of(contents: &BTreeMap<(u32, u64), Vec<u8>>) -> Vec<PublishedVersion> {
    contents.iter().map(|((instance, txg), content)| PublishedVersion {
        instance: InstanceGeneration(*instance), checkpoint_txg: CheckpointTxg(*txg), content: content.clone(),
    }).collect()
}

/// 对 `base` 之后录下的流逐点穷举崩溃状态（每一段都展开），oracle 判结束状态；每个状态（每 `RBF2_PROBE_EVERY` 个取一个）
/// 另把最新的 1、2、3 条可读根改坏再恢复，落在哪、读得对不对；另对每个状态跑环里每条候选根的读（`RBF2_AUDIT=1`）。
fn crash_enumerate(name: &str, base: &MemoryPool, session: &Session) {
    let operations = session.stream.retained_operations();
    let (writes, segments) = writes_and_segments(&operations, &common::geometry());
    let judged_root_index = writes.iter().rposition(|write: &RetainedWrite| write.kind == StepKind::RootRecordFua).expect("流里有根槽写");
    let versions = versions_of(&session.contents);
    let expand_max: usize = std::env::var("RBF2_EXPAND_MAX").ok().and_then(|v| v.parse().ok()).unwrap_or(usize::MAX);
    let expand = |_index: usize, segment: &[usize]| segment.len() <= expand_max;
    println!("RBF2 crash name={name} closed_form={} segment_lengths={:?} expand_max={expand_max}",
        singlefs_harness::crash::closed_form_state_count(&segments), segments.iter().map(Vec::len).collect::<Vec<_>>());
    if std::env::var("RBF2_COUNT_ONLY").is_ok() { return; }
    let every: u64 = std::env::var("RBF2_PROBE_EVERY").ok().and_then(|v| v.parse().ok()).unwrap_or(1);
    let audit = std::env::var("RBF2_AUDIT").is_ok();
    let mut state_index = 0u64;
    let mut probe_states = 0u64;
    let mut probe_bad = [0u64; 3];
    let mut probe_first_bad: [Option<String>; 3] = [None, None, None];
    let mut audit_bad = 0u64;
    let mut audit_first: Option<String> = None;
    let tally = enumerate_layer0_selecting_versions_observing_each_state(
        base, &writes, &segments, judged_root_index, &versions, &expand,
        &mut |image: &CrashImage<'_>, _report| {
            state_index += 1;
            if (state_index - 1) % every != 0 { return; }
            let mut materialized = image.base.clone();
            let persisted: Vec<RetainedWrite> = image.writes.iter().zip(&image.persisted)
                .filter(|(_, p)| **p).map(|(w, _)| w.clone()).collect();
            materialized.apply_writes(&persisted);
            probe_states += 1;
            let roots = roots_of(&materialized);
            for killed in 1..=3usize {
                if roots.len() <= killed { continue; }
                let newest: Vec<u64> = roots[roots.len() - killed..].iter().map(|r| r.checkpoint_txg.0).collect();
                let mut damaged = materialized.clone();
                corrupt_roots(&mut damaged, &newest);
                if let Err(reason) = recover_and_judge(&damaged, &session.contents) {
                    probe_bad[killed - 1] += 1;
                    if probe_first_bad[killed - 1].is_none() {
                        probe_first_bad[killed - 1] = Some(format!("killed={newest:?} {reason}"));
                    }
                }
            }
            if audit {
                let (bad_candidates, _) = ring_audit(&materialized, &session.contents);
                if !bad_candidates.is_empty() {
                    audit_bad += 1;
                    if audit_first.is_none() { audit_first = Some(format!("{bad_candidates:?}")); }
                }
            }
        },
    );
    println!("RBF2 crash name={name} writes={} states={} violations={} failed={} first_violation={} probe_states={probe_states} probe_bad(kill1,kill2,kill3)={probe_bad:?} probe_first_bad={probe_first_bad:?} audit_bad={audit_bad} audit_first={audit_first:?}",
        writes.len(), tally.states, tally.violations, tally.failed_states, tally.first_violation.as_deref().unwrap_or("none"));
    let reds: Vec<String> = tally.checker_violated_states.iter().map(|(invariant, count)| format!("{invariant}={count}:{}",
        tally.checker_first_violation.get(invariant).map(|s| s.chars().take(140).collect::<String>()).unwrap_or_default())).collect();
    println!("RBF2 crash name={name} checker_violated={reds:?}");
}

fn prefix_three_overwrites() -> (Session, Open) {
    let (mut session, open0) = Session::new();
    drop(open0);
    let mut open = session.mount();
    for _ in 0..3 { session.try_overwrite(&mut open).expect("写"); }
    (session, open)
}

/// G1：挂着的时候回退那一次发布的每一个崩溃点（流只含回退那一次）。目标取实例 1 的 A 与上一个状态。
#[test]
fn g1_crash_rollback_publish() {
    set_rule(false);
    set_deleted(false);
    let kinds: &[&str] = if std::env::var("RBF2_ONLY_A").is_ok() { &["A"] } else { &["A", "previous"] };
    for target_kind in kinds.iter().copied() {
        let (mut session, mut open) = prefix_three_overwrites();
        let target = if target_kind == "A" { (1, 3) } else { (open.instance.0, open.current.root.checkpoint_txg.0 - 1) };
        let base = session.restart_stream();
        let counts = session.rollback(&mut open, target, Rbf2Arm::Full).expect("回退");
        println!("RBF2 g1_crash target={target:?} counts={counts:?}");
        crash_enumerate(&format!("rollback-publish-to-{target_kind}"), &base, &session);
    }
}

/// G1：回退、写、再回退、写，整条流（四次发布）逐点崩溃。
#[test]
fn g1_crash_rollback_write_rollback_write() {
    set_rule(false);
    set_deleted(false);
    let (mut session, mut open) = prefix_three_overwrites();
    let newest_before = (open.instance.0, open.current.root.checkpoint_txg.0);
    let base = session.restart_stream();
    session.rollback(&mut open, (1, 3), Rbf2Arm::Full).expect("回退 1");
    session.try_overwrite(&mut open).expect("写 1");
    session.rollback(&mut open, newest_before, Rbf2Arm::Full).expect("回退 2");
    session.try_overwrite(&mut open).expect("写 2");
    crash_enumerate("rollback-write-rollback-write", &base, &session);
}

/// G1：多单元文件、多 inode 叶之后回退到它们之前（回退那次要复活 / 释放的单元多）。
#[test]
fn g1_crash_rollback_over_seq_write_and_inodes() {
    set_rule(false);
    set_deleted(false);
    let (mut session, open0) = Session::new();
    drop(open0);
    let mut open = session.mount();
    session.try_overwrite(&mut open).expect("B");
    let b = (open.instance.0, open.current.root.checkpoint_txg.0);
    session.try_seq_write(&mut open).expect("S");
    session.try_new_inodes(&mut open, 300).expect("I");
    let base = session.restart_stream();
    let counts = session.rollback(&mut open, b, Rbf2Arm::Full).expect("回退");
    println!("RBF2 g1_crash_big counts={counts:?}");
    crash_enumerate("rollback-over-seq-and-inodes", &base, &session);
}

/// G2：回退那次发布与一次普通写排不排得开。(a) 一个写在回退之前按旧的现行版本（N）备好、回退发布之后才交；
/// (b) 回退按旧的现行版本（N）算、一个普通写已经把 N + 1 发出去之后才交。分配器共用一份（第一版串行提交，`&mut`）。
#[test]
fn g2_stale_previous_around_the_rollback() {
    set_rule(false);
    set_deleted(false);
    // (a)
    {
        let (mut session, mut open) = prefix_three_overwrites();
        let stale = open.current.clone();
        session.rollback(&mut open, (1, 3), Rbf2Arm::Full).expect("回退");
        let parameters = parameters();
        let content = content_of(9001);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut writer = PoolWriter::new(&parameters, session.devices.as_mut_slice());
            publish_overwrite(&mut writer, &mut open.allocator, &stale,
                FirstFile { content: &content, write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60 }, open.instance)
                .map(|o| o.root.checkpoint_txg.0).map_err(|e| format!("{e:?}"))
        }));
        let outcome = match &result {
            Ok(inner) => format!("returned {inner:?}"),
            Err(panic) => format!("panicked {}", panic.downcast_ref::<String>().cloned().or_else(|| panic.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default().chars().take(200).collect::<String>()),
        };
        let image = session.image();
        println!("RBF2 g2a stale_write_after_rollback outcome={outcome} recover={:?} newest_roots={:?} cand_bad={:?}",
            recover_and_judge(&image, &session.contents),
            roots_of(&image).iter().rev().take(3).map(|r| (r.instance.0, r.checkpoint_txg.0)).collect::<Vec<_>>(),
            ring_audit(&image, &session.contents).0);
    }
    // (b)
    {
        let (mut session, mut open) = prefix_three_overwrites();
        let mut stale = open.current.clone();
        session.try_overwrite(&mut open).expect("普通写");
        let parameters = parameters();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            prototype_rollback_in_mount(&parameters, &mut session.devices, &mut open.allocator, &mut stale,
                RollbackTarget { instance: InstanceGeneration(1), checkpoint_txg: CheckpointTxg(3) }, Rbf2Arm::Full)
                .map(|c| c.txg).map_err(|e| format!("{e:?}"))
        }));
        let outcome = match &result {
            Ok(inner) => format!("returned {inner:?}"),
            Err(panic) => format!("panicked {}", panic.downcast_ref::<String>().cloned().or_else(|| panic.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default().chars().take(200).collect::<String>()),
        };
        let image = session.image();
        println!("RBF2 g2b stale_rollback_after_write outcome={outcome} recover={:?} newest_roots={:?} cand_bad={:?}",
            recover_and_judge(&image, &session.contents),
            roots_of(&image).iter().rev().take(3).map(|r| (r.instance.0, r.checkpoint_txg.0)).collect::<Vec<_>>(),
            ring_audit(&image, &session.contents).0);
    }
}

/// G2：回退之后接着建 inode，号从哪里发；回退跨过建 inode 的几代时，候选根之间 inode 号冲不冲突（两臂：水位取环里 max / 只取最新那一版）。
#[test]
fn g2_inode_numbers_after_rollback() {
    set_rule(false);
    set_deleted(false);
    for arm in [Rbf2Arm::Full, Rbf2Arm::WatermarkOfNewestOnly] {
        let (mut session, open0) = Session::new();
        drop(open0);
        let mut open = session.mount();
        session.try_overwrite(&mut open).expect("B");
        let b = (open.instance.0, open.current.root.checkpoint_txg.0);
        session.try_new_inodes(&mut open, 5).expect("I");
        let wm_before = open.current.inode_number_watermark();
        let counts = session.rollback(&mut open, b, arm).expect("回退");
        let wm_after_rollback = open.current.inode_number_watermark();
        session.try_new_inodes(&mut open, 3).expect("I2");
        let image = session.image();
        println!("RBF2 g2_inode arm={arm:?} wm_before={wm_before} ring_wm={} wm_after_rollback={wm_after_rollback} wm_after_new={} conflicts={:?} checker={:?}",
            counts.inode_watermark_of_the_ring, open.current.inode_number_watermark(), inode_number_conflicts(&image), checker_red_names(&image));
    }
}

/// G1：两棵账整棵求差的字面，回退之后接着写到环转过一圈，新根引用的那几个固定点槽被复用没有、池还挂不挂得上。
#[test]
fn g1_literal_whole_tree_diff_after_a_ring_turn() {
    set_rule(false);
    set_deleted(false);
    for arm in [Rbf2Arm::Full, Rbf2Arm::LiteralWholeTreeDiff] {
        let (mut session, mut open) = prefix_three_overwrites();
        let counts = session.rollback(&mut open, (1, 3), arm).expect("回退");
        let mut first_bad: Option<String> = None;
        let mut writes = 0;
        for index in 0..40 {
            match session.try_overwrite(&mut open) {
                Ok(_) => writes += 1,
                Err(e) => { first_bad.get_or_insert(format!("write {index}: {}", e.chars().take(160).collect::<String>())); break; }
            }
        }
        let image = session.image();
        let (cand_bad, _) = ring_audit(&image, &session.contents);
        let mut copy = Session::from_image(&image, &session.contents);
        let remount = copy.try_mount().map(|o| o.instance.0).map_err(|e| e.chars().take(160).collect::<String>());
        println!("RBF2 g1_literal arm={arm:?} lit={} writes={writes} first_bad={first_bad:?} recover={:?} cand_bad={cand_bad:?} remount={remount:?} checker={:?}",
            counts.whole_tree_diff_referenced_by_the_new_root, recover_and_judge(&image, &session.contents), checker_red_names(&image));
    }
}

fn rule_name(max: bool) -> &'static str {
    match (max, PROTOTYPE_HOLD_UNTIL_COVERED.load(Ordering::SeqCst)) {
        (false, _) => "min_of_max",
        (true, false) => "max",
        (true, true) => "max+hold",
    }
}
fn set_hold(hold: bool) {
    PROTOTYPE_HOLD_UNTIL_COVERED.store(hold, Ordering::SeqCst);
}
const MODES: [(bool, bool); 3] = [(false, false), (true, false), (true, true)];

/// 带 F > 0 的根里落在某块盘上的那几条的 txg。
fn floor_roots_on(image: &MemoryPool, device: u32) -> Vec<u64> {
    roots_of(image).iter().filter(|r| r.rollback_floor.0 > 0 && device_of_txg(r.checkpoint_txg.0) == DeviceIdentity(device))
        .map(|r| r.checkpoint_txg.0).collect()
}

/// 在一份镜像上另开一次可写挂载、挂着回退到 `target`：交回 (接受与否, 新根下读得对不对, 候选里坏的根, checker)。
fn try_rollback_on_copy(image: &MemoryPool, contents: &BTreeMap<(u32, u64), Vec<u8>>, target: (u32, u64), arm: Rbf2Arm) -> String {
    let mut session = Session::from_image(image, contents);
    let mut open = match session.try_mount() {
        Ok(open) => open,
        Err(e) => return format!("mount_err {}", e.chars().take(120).collect::<String>()),
    };
    match session.rollback(&mut open, target, arm) {
        Ok(counts) => {
            let after = session.image();
            let read = walk_to_file(&after, &open.current.root, &mut 0).map(|c| c == contents.get(&target).cloned());
            let (cand_bad, _) = ring_audit(&after, &session.contents);
            format!("accepted F={} read_ok={:?} cand_bad={cand_bad:?} checker={:?}", counts.floor_used,
                read.map_err(|e| format!("{e:?}").chars().take(80).collect::<String>()), checker_red_names(&after))
        }
        Err(e) => format!("refused {}", e.chars().take(120).collect::<String>()),
    }
}

/// G3：第一轮 H1 那 15 格（B 之后盘 1 上带新 F 的根全坏，F 回落，再回退到 F 之下的根）在两种生效规则下还中不中。
/// 前缀：实例 1 A(3) B(4) C(5)，B1 卸载抬 F 到 5，重开写 0..4 次、关掉，改坏盘 1 上带 F 的根，重开、挂着回退到 A / B / C。
#[test]
fn g3_h1_cells_under_both_rules() {
    set_deleted(false);
    for (max, hold) in MODES {
        set_rule(max);
        set_hold(hold);
        let mut hits = 0;
        let mut accepted = 0;
        for writes in 0..=4 {
            for target in [(1u32, 3u64), (1, 4), (1, 5)] {
                for arm in [Rbf2Arm::Full, Rbf2Arm::WithoutProtectionCheck] {
                    let (mut session, mut open) = Session::new();
                    session.try_overwrite(&mut open).expect("B");
                    session.try_overwrite(&mut open).expect("C");
                    let chain = session.unmount_b1(&mut open, false).expect("B1");
                    drop(open);
                    let mut reopened = session.mount();
                    for _ in 0..writes { session.try_overwrite(&mut reopened).expect("写"); }
                    drop(reopened);
                    let mut damaged = session.image();
                    let on_one = floor_roots_on(&damaged, 1);
                    corrupt_roots(&mut damaged, &on_one);
                    let verdict = try_rollback_on_copy(&damaged, &session.contents, target, arm);
                    let hit = verdict.starts_with("accepted") && (verdict.contains("read_ok=Ok(false)") || verdict.contains("read_ok=Err") || !verdict.contains("cand_bad=[]") || verdict.contains("I-7.4"));
                    if verdict.starts_with("accepted") { accepted += 1; }
                    if hit { hits += 1; }
                    println!("RBF2 g3_h1 rule={} writes={writes} target={target:?} arm={arm:?} chain={chain:?} killed_on_dev1={on_one:?} F_after={} recover={:?} rollback={verdict} hit={hit}",
                        rule_name(max), floor_of(&damaged), recover_and_judge(&damaged, &session.contents));
                }
            }
        }
        println!("RBF2 g3_h1-summary rule={} cells=30 accepted={accepted} hits={hits}", rule_name(max));
    }
    set_rule(false);
    set_hold(false);
}

/// 前缀：实例 2 里写到下一个 txg 落盘 0，B1 卸载（这一串先落盘 0、后落盘 1），然后把这一串落在盘 1 上的根改坏 = 崩在盘 1 那一次之前。
/// 交回 (崩了之后的镜像, 内容表, 这一串的 txg, 卸载前的现行 txg)。
fn chain_crashed_before_device_one() -> (MemoryPool, BTreeMap<(u32, u64), Vec<u8>>, Vec<u64>, u64) {
    let (mut session, open0) = Session::new();
    drop(open0);
    let mut open = session.mount();
    session.try_overwrite(&mut open).expect("B");
    while device_of_txg(open.current.root.checkpoint_txg.0 + 1) != DeviceIdentity(0) {
        session.try_overwrite(&mut open).expect("写到下一个 txg 落盘 0");
    }
    let newest = open.current.root.checkpoint_txg.0;
    let chain = session.unmount_b1(&mut open, false).expect("B1");
    let mut image = session.image();
    let on_one: Vec<u64> = chain.iter().copied().filter(|t| device_of_txg(*t) == DeviceIdentity(1)).collect();
    corrupt_roots(&mut image, &on_one);
    (image, session.contents.clone(), chain, newest)
}

/// G3：抬 F 那一串崩在盘 1 那一次之前。两种规则 × 重开之后写 0..2 次 × 之后的故障（无 / 改坏盘 0 上带 F 的根 / 改坏全部带 F 的根 /
/// 改坏最新 1、2 条根）：恢复落在哪、读得对不对、候选根坏没坏、挂着回退到环里每一条带文件的根接受与否、接受了读得对不对。
#[test]
fn g3_chain_crashed_before_device_one() {
    set_deleted(false);
    let (crashed, contents, chain, newest) = chain_crashed_before_device_one();
    for (max, hold) in MODES {
        set_rule(max);
        set_hold(hold);
        println!("RBF2 g3_crash rule={} chain={chain:?} newest_before_chain={newest} F_eff={} recover={:?} roots={:?}",
            rule_name(max), floor_of(&crashed), recover_and_judge(&crashed, &contents),
            roots_of(&crashed).iter().map(|r| (r.checkpoint_txg.0, r.rollback_floor.0)).collect::<Vec<_>>());
        let mut summary: BTreeMap<String, u64> = BTreeMap::new();
        for writes in 0..=2 {
            for fault in ["none", "kill_dev0_floor_roots", "kill_all_floor_roots", "kill_newest_1", "kill_newest_2"] {
                let mut session = Session::from_image(&crashed, &contents);
                let mut open = match session.try_mount() {
                    Ok(open) => open,
                    Err(e) => { println!("RBF2 g3_crash rule={} writes={writes} mount_err={e}", rule_name(max)); continue; }
                };
                for _ in 0..writes { session.try_overwrite(&mut open).expect("写"); }
                drop(open);
                let mut image = session.image();
                let killed: Vec<u64> = match fault {
                    "kill_dev0_floor_roots" => floor_roots_on(&image, 0),
                    "kill_all_floor_roots" => roots_of(&image).iter().filter(|r| r.rollback_floor.0 > 0).map(|r| r.checkpoint_txg.0).collect(),
                    "kill_newest_1" => roots_of(&image).iter().rev().take(1).map(|r| r.checkpoint_txg.0).collect(),
                    "kill_newest_2" => roots_of(&image).iter().rev().take(2).map(|r| r.checkpoint_txg.0).collect(),
                    _ => Vec::new(),
                };
                corrupt_roots(&mut image, &killed);
                let recovered = recover_and_judge(&image, &session.contents);
                let (cand_bad, _) = ring_audit(&image, &session.contents);
                let mut rollbacks = Vec::new();
                for root in roots_of(&image).iter().filter(|r| r.checkpoint_txg.0 >= 3) {
                    let target = (root.instance.0, root.checkpoint_txg.0);
                    let verdict = try_rollback_on_copy(&image, &session.contents, target, Rbf2Arm::Full);
                    let hit = verdict.starts_with("accepted") && (!verdict.contains("read_ok=Ok(true)") || !verdict.contains("cand_bad=[]"));
                    *summary.entry(if verdict.starts_with("accepted") { "accepted" } else { "refused" }.to_string()).or_default() += 1;
                    if hit { *summary.entry("rollback_hit".into()).or_default() += 1; }
                    rollbacks.push(format!("{target:?}:{}", verdict.chars().take(140).collect::<String>()));
                }
                if recovered.is_err() { *summary.entry("recover_bad".into()).or_default() += 1; }
                if !cand_bad.is_empty() { *summary.entry("cand_bad".into()).or_default() += 1; }
                println!("RBF2 g3_crash rule={} writes={writes} fault={fault} killed={killed:?} F_after={} recover={recovered:?} cand_bad={cand_bad:?} rollbacks={rollbacks:?}",
                    rule_name(max), floor_of(&image));
            }
        }
        println!("RBF2 g3_crash-summary rule={} {summary:?}", rule_name(max));
    }
    set_rule(false);
    set_hold(false);
}

/// G3：同一个崩溃状态之后，重开那次挂载的写行发布落了单元、根没落（第二次崩），再改坏盘 0 上带 F 的根：
/// 挂着回退到 F 之下的根，接受与否、读得对不对。写行那次发布写进了哪些「按新 F 回收出来」的槽也报出。
#[test]
fn g3_chain_crash_then_row_publish_crash() {
    set_deleted(false);
    let (crashed, contents, chain, newest) = chain_crashed_before_device_one();
    for (max, hold) in MODES {
        set_rule(max);
        set_hold(hold);
        let mut session = Session::from_image(&crashed, &contents);
        let base = session.restart_stream();
        let open = session.mount();
        let row_units: Vec<u64> = open.current.units.iter().map(|u| u.slot.0).collect();
        drop(open);
        // N 那一版账里释放代 ≤ newest 的落点（按新 F 回收的那一批）。
        let system_configuration = choose_system_configuration(&crashed).expect("系统配置");
        let records = scan_journal(&crashed, &system_configuration);
        let n_root = roots_of(&crashed).into_iter().find(|r| r.checkpoint_txg.0 == newest).expect("N 在环里");
        let n_record = records.values().filter(|r| r.instance == n_root.instance && r.checkpoint_txg == n_root.checkpoint_txg).max_by_key(|r| r.counter).cloned();
        let reclaimable: BTreeSet<u64> = match rebuild_version(&crashed, &n_root, n_record) {
            Ok(RebuiltVersion::WithFile(output)) => output.allocation_records.iter().filter(|r| r.is_released && r.generation.0 <= newest).map(|r| r.slot.0).collect(),
            _ => BTreeSet::new(),
        };
        let operations = session.stream.retained_operations();
        let (writes, _segments) = writes_and_segments(&operations, &common::geometry());
        let first_root = writes.iter().position(|w| w.kind == StepKind::RootRecordFua).expect("写行那次有根槽写");
        let unit_slots_written: BTreeSet<u64> = writes[..first_root].iter()
            .filter(|w| w.kind == StepKind::UnitWrite)
            .map(|w| w.offset.0 / singlefs_format::SLOT_BYTES).collect();
        let persisted: Vec<RetainedWrite> = writes[..first_root].to_vec();
        let mut state = base.clone();
        state.apply_writes(&persisted);
        let dev0 = floor_roots_on(&state, 0);
        corrupt_roots(&mut state, &dev0);
        let into_reclaimed: Vec<u64> = unit_slots_written.iter().copied().filter(|s| reclaimable.contains(s)).collect();
        println!("RBF2 g3_row_crash rule={} chain={chain:?} N={newest} F_eff_at_mount={} row_publish_unit_slots={:?} reclaimable_by_new_F={} row_units_into_reclaimed={into_reclaimed:?} killed_dev0={dev0:?} F_after={} recover={:?}",
            rule_name(max), floor_of(&crashed), unit_slots_written.len(), reclaimable.len(), floor_of(&state), recover_and_judge(&state, &contents));
        let _ = row_units;
        for root in roots_of(&state).iter().filter(|r| r.checkpoint_txg.0 >= 3) {
            let target = (root.instance.0, root.checkpoint_txg.0);
            println!("RBF2 g3_row_crash rule={} target={target:?} rollback={}", rule_name(max), try_rollback_on_copy(&state, &contents, target, Rbf2Arm::Full));
        }
    }
    set_rule(false);
    set_hold(false);
}

/// G3：只有盘 0 上有带新 F 的根，盘 1 整块读不出（从池里拿掉）：F_生效 各是多少、恢复落在哪、可写挂载怎样；盘 1 回来之后又怎样。
#[test]
fn g3_device_one_gone() {
    set_deleted(false);
    let (crashed, contents, chain, _newest) = chain_crashed_before_device_one();
    for (max, hold) in MODES {
        set_rule(max);
        set_hold(hold);
        let mut gone = crashed.clone();
        gone.devices.remove(&DeviceIdentity(1));
        let mut session = Session::from_image(&gone, &contents);
        let mount = session.try_mount().map(|o| o.instance.0).map_err(|e| e.chars().take(160).collect::<String>());
        println!("RBF2 g3_gone rule={} chain={chain:?} F_eff_one_device={} recover={:?} writable_mount={mount:?} F_eff_both_devices_back={}",
            rule_name(max), floor_of(&gone), recover_and_judge(&gone, &contents), floor_of(&crashed));
    }
    set_rule(false);
    set_hold(false);
}

/// G4：B1 的上限按字面（每块盘上最新的持久有效根，各盘取小）判「抬到最新根」：前缀是可写挂载之后 0..6 次非空发布
/// （两种负载），每一格报 (最新根, 字面上限, 今天的上限, 字面判的结果)。
#[test]
fn g4_b1_literal_ceiling() {
    set_rule(true);
    set_deleted(false);
    let mut refused = 0;
    let mut total = 0;
    for load in ["overwrite", "new_inodes"] {
        for count in 0..=6 {
            let (mut session, open0) = Session::new();
            drop(open0);
            let mut open = session.mount();
            for _ in 0..count {
                if load == "overwrite" { session.try_overwrite(&mut open).expect("写"); } else { session.try_new_inodes(&mut open, 2).expect("建"); }
            }
            let newest = open.current.root.checkpoint_txg.0;
            let literal = singlefs_core::mount::prototype_b1_ceiling(&session.devices, &open.current).map(|c| c.0);
            let system_configuration = choose_system_configuration(&session.image()).expect("系统配置");
            let table = instance_table_chain_of_root(&session.devices, &open.current.root).expect("表").records;
            let today = singlefs_core::mount::rollback_floor_ceiling(&session.devices, &system_configuration, open.current.root.rollback_floor, &table).map(|c| c.0);
            let result = session.unmount_b1(&mut open, true);
            total += 1;
            if result.is_err() { refused += 1; }
            println!("RBF2 g4_literal load={load} nonempty_after_mount={count} newest={newest} literal_ceiling={literal:?} today_ceiling={today:?} literal_b1={:?}",
                result.map_err(|e| e.chars().take(120).collect::<String>()));
        }
    }
    println!("RBF2 g4_literal-summary cells={total} refused={refused}");
    set_rule(false);
    set_hold(false);
}

/// G4：B1（不判上限，抬到最新根）那一串逐点崩溃，两种起点（这一串 2 次 / 3 次），生效规则取最大值。
#[test]
fn g4_crash_points_of_the_b1_chain() {
    set_rule(true);
    set_deleted(false);
    for extra in 0..3 {
        let (mut session, open0) = Session::new();
        drop(open0);
        let mut open = session.mount();
        for _ in 0..(1 + extra) { session.try_overwrite(&mut open).expect("写"); }
        let base = session.restart_stream();
        let chain = session.unmount_b1(&mut open, false).expect("B1");
        println!("RBF2 g4_crash start={} chain={chain:?}", open.current.root.checkpoint_txg.0 - chain.len() as u64);
        crash_enumerate(&format!("b1-chain-{}", chain.len()), &base, &session);
    }
    set_rule(false);
    set_hold(false);
}

/// G4：B1 卸载之后再挂载、写 0 / 1 / 3 次、关掉，再把最新的 j 条根改坏（j 从 1 到比 F 新或等于 F 的根全坏再多一条）：
/// 恢复落在哪、报什么，可写挂载挂不挂得上、报什么。两种生效规则。
#[test]
fn g4_remount_after_b1_then_newest_roots_damaged() {
    set_deleted(false);
    for (max, hold) in MODES {
        set_rule(max);
        set_hold(hold);
        for writes in [0usize, 1, 3] {
            let (mut session, open0) = Session::new();
            drop(open0);
            let mut open = session.mount();
            for _ in 0..3 { session.try_overwrite(&mut open).expect("写"); }
            let f = open.current.root.checkpoint_txg.0;
            let chain = session.unmount_b1(&mut open, false).expect("B1");
            drop(open);
            let mut reopened = session.mount();
            for _ in 0..writes { session.try_overwrite(&mut reopened).expect("写"); }
            drop(reopened);
            let image = session.image();
            let roots = roots_of(&image);
            let at_or_above = roots.iter().filter(|r| r.checkpoint_txg.0 >= f).count();
            for j in 1..=(at_or_above + 1).min(roots.len() - 1) {
                let killed: Vec<u64> = roots.iter().rev().take(j).map(|r| r.checkpoint_txg.0).collect();
                let mut damaged = image.clone();
                corrupt_roots(&mut damaged, &killed);
                let mut copy = Session::from_image(&damaged, &session.contents);
                let mount = copy.try_mount().map(|o| (o.instance.0, o.current.root.checkpoint_txg.0)).map_err(|e| e.chars().take(140).collect::<String>());
                println!("RBF2 g4_remount rule={} F={f} chain={chain:?} writes_after={writes} killed_newest={j} F_after={} recover={:?} writable_mount={mount:?}",
                    rule_name(max), floor_of(&damaged), recover_and_judge(&damaged, &session.contents));
            }
        }
    }
    set_rule(false);
    set_hold(false);
}

/// G4：同一个起点，「卸载那一串」与「准入抬 F 抬到同一个值」写出来的盘逐字节比；checker 的 I-7.9 各判什么。
#[test]
fn g4_unmount_chain_and_admission_raise_are_the_same_bytes() {
    set_rule(true);
    set_deleted(false);
    let (mut session, open0) = Session::new();
    drop(open0);
    let mut open = session.mount();
    for _ in 0..2 { session.try_overwrite(&mut open).expect("写"); }
    let image = session.image();
    let newest = open.current.root.checkpoint_txg.0;
    let mut unmount = Session::from_image(&image, &session.contents);
    let mut unmount_open = Open { allocator: open.allocator.clone(), current: open.current.clone(), instance: open.instance };
    let chain_u = unmount.unmount_b1(&mut unmount_open, false).expect("B1");
    let mut admission = Session::from_image(&image, &session.contents);
    let mut admission_open = Open { allocator: open.allocator.clone(), current: open.current.clone(), instance: open.instance };
    let chain_a = admission.raise(&mut admission_open, newest, true).expect("同一个函数当准入抬 F");
    let a = unmount.image();
    let b = admission.image();
    let same = a.devices == b.devices;
    let i79 = |image: &MemoryPool| checker_reds(image).into_iter().filter(|r| r.starts_with("I-7.9")).collect::<Vec<_>>();
    println!("RBF2 g4_same chain_unmount={chain_u:?} chain_admission={chain_a:?} whole_devices_identical={same} roots_identical={} I-7.9_unmount={:?} I-7.9_admission={:?}",
        roots_of(&a) == roots_of(&b), i79(&a), i79(&b));
    set_rule(false);
    set_hold(false);
}

/// 今天的代码造一份旧镜像：实例 2 写 B C，关掉，挂载时回退到 B（实例 3，回退行 + 见证条目），写 D E，关掉。
fn old_image_with_rollback_row_and_witness() -> (MemoryPool, BTreeMap<(u32, u64), Vec<u8>>, (u32, u64), (u32, u64), Vec<(u32, u64)>) {
    set_deleted(false);
    let (mut session, open0) = Session::new();
    drop(open0);
    let mut open = session.mount();
    session.try_overwrite(&mut open).expect("B");
    let b = (open.instance.0, open.current.root.checkpoint_txg.0);
    session.try_overwrite(&mut open).expect("C");
    let c = (open.instance.0, open.current.root.checkpoint_txg.0);
    drop(open);
    let mut rolled = session.old_mount_rollback(b).expect("今天的挂载时回退");
    session.try_overwrite(&mut rolled).expect("D");
    session.try_overwrite(&mut rolled).expect("E");
    let instance3 = rolled.instance.0;
    drop(rolled);
    let image = session.image();
    let instance3_roots: Vec<(u32, u64)> = roots_of(&image).iter().filter(|r| r.instance.0 == instance3).map(|r| (r.instance.0, r.checkpoint_txg.0)).collect();
    (image, session.contents.clone(), b, c, instance3_roots)
}

/// G5：旧镜像（带回退行、见证表非空）交给「删掉见证与截断」之后的代码：挂不挂得上、逐条改坏更新的根之后落在哪、
/// 实例 3 的根全坏（C332 那一形）之后落在哪；之后再挂、再写、实例 3 的根又读得出时，被抛弃时间线引用的块有没有被复用。
#[test]
fn g5_old_image_under_the_deleting_code() {
    let (image, contents, b, c, instance3_roots) = old_image_with_rollback_row_and_witness();
    let witness_entries = singlefs_core::recovery::rollback_witness_of_the_pool(&image, &choose_system_configuration(&image).expect("系统配置")).entries().len();
    println!("RBF2 g5 B={b:?} C={c:?} instance3_roots={instance3_roots:?} witness_entries={witness_entries} roots={:?}",
        roots_of(&image).iter().map(|r| (r.instance.0, r.checkpoint_txg.0)).collect::<Vec<_>>());
    set_deleted(false);
    let high_waters: Vec<((u32, u64), Option<u64>)> = roots_of(&image).iter()
        .map(|r| ((r.instance.0, r.checkpoint_txg.0), singlefs_core::recovery::rollback_high_water_of_root(&image, r))).collect();
    let rows_of_newest = instance_table_chain_of_root(&image, roots_of(&image).last().expect("有根")).map(|c| c.records.rows.iter()
        .map(|row| (row.instance.0, row.selected_root_txg.0, row.applied_transaction_high_water, row.is_rollback)).collect::<Vec<_>>());
    println!("RBF2 g5 rollback_high_water_of_every_root={high_waters:?} rows_of_newest_table={rows_of_newest:?}");
    for deleted in [false, true] {
        set_deleted(deleted);
        let label = if deleted { "deleted" } else { "today" };
        let mut copy = Session::from_image(&image, &contents);
        let mounted = copy.try_mount().map(|o| (o.instance.0, o.current.root.checkpoint_txg.0)).map_err(|e| e.chars().take(140).collect::<String>());
        let (cand_bad, other_bad) = ring_audit(&image, &contents);
        println!("RBF2 g5 code={label} writable_mount={mounted:?} cand_bad={cand_bad:?} other_bad={other_bad:?} checker={:?}", checker_red_names(&image));
        let roots = roots_of(&image);
        for index in (0..roots.len()).rev() {
            let newer: Vec<u64> = roots[index + 1..].iter().map(|r| r.checkpoint_txg.0).collect();
            let mut damaged = image.clone();
            corrupt_roots(&mut damaged, &newer);
            let report = recover(&damaged, JournalPolicy::Consult);
            println!("RBF2 g5-row code={label} killed_newest={} landed={:?} judged={:?}", newer.len(),
                report.effective_root.map(|(i, t)| (i.0, t.0)), recover_and_judge(&damaged, &contents));
        }
        // C332：实例 3 的根全坏。
        let mut damaged = image.clone();
        let killed: Vec<u64> = instance3_roots.iter().map(|(_, t)| *t).collect();
        corrupt_roots(&mut damaged, &killed);
        let report = recover(&damaged, JournalPolicy::Consult);
        let mut next = Session::from_image(&damaged, &contents);
        let reopened = next.try_mount();
        let mut after_writes = String::new();
        if let Ok(mut open) = reopened {
            let landed_on = (open.current.root.instance.0, open.current.root.checkpoint_txg.0);
            for _ in 0..3 { let _ = next.try_overwrite(&mut open); }
            drop(open);
            // 实例 3 的根又读得出（瞬时故障撤掉：再翻一次同一个字节）。
            let mut restored = next.image();
            corrupt_roots(&mut restored, &killed);
            let (cand_bad, other_bad) = ring_audit(&restored, &next.contents);
            after_writes = format!("mounted_on={landed_on:?} then_3_writes restored_instance3 recover={:?} cand_bad={cand_bad:?} other_bad={other_bad:?}",
                recover_and_judge(&restored, &next.contents));
        } else if let Err(e) = reopened {
            after_writes = format!("mount_err {}", e.chars().take(140).collect::<String>());
        }
        println!("RBF2 g5-c332 code={label} killed={killed:?} landed={:?} judged={:?} :: {after_writes}",
            report.effective_root.map(|(i, t)| (i.0, t.0)), recover_and_judge(&damaged, &contents));
    }
    set_deleted(false);
}

/// G2 (c)：回退到用户可见树与现行那一版相同的根（写行根与它的暖机根同树，回退不释放、不复活），一个按旧现行版本备好的写之后才交：
/// 两次发布同取 txg N + 1。
#[test]
fn g2_stale_write_after_a_rollback_that_releases_nothing() {
    set_rule(false);
    set_deleted(false);
    let (mut session, open0) = Session::new();
    drop(open0);
    let mut open = session.mount();
    let current = (open.instance.0, open.current.root.checkpoint_txg.0);
    let same_trees = roots_of(&session.image()).iter().rev()
        .map(|r| (r.instance.0, r.checkpoint_txg.0))
        .find(|key| key.0 == current.0 && key.1 < current.1).expect("同实例更早的根");
    let stale = open.current.clone();
    let counts = session.rollback(&mut open, same_trees, Rbf2Arm::Full);
    let parameters = parameters();
    let content = content_of(9002);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut writer = PoolWriter::new(&parameters, session.devices.as_mut_slice());
        publish_overwrite(&mut writer, &mut open.allocator, &stale,
            FirstFile { content: &content, write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60 }, open.instance)
            .map(|o| o.root.checkpoint_txg.0).map_err(|e| format!("{e:?}"))
    }));
    let outcome = match &result {
        Ok(inner) => format!("returned {inner:?}"),
        Err(panic) => format!("panicked {}", panic.downcast_ref::<String>().cloned().or_else(|| panic.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default().chars().take(160).collect::<String>()),
    };
    let image = session.image();
    println!("RBF2 g2c current={current:?} rollback_to={same_trees:?} counts={counts:?} stale_write={outcome} recover={:?} newest_roots={:?} cand_bad={:?}",
        recover_and_judge(&image, &session.contents),
        roots_of(&image).iter().rev().take(3).map(|r| (r.instance.0, r.checkpoint_txg.0)).collect::<Vec<_>>(),
        ring_audit(&image, &session.contents).0);
}

/// G3 对照：这一串完整落盘（两块盘都有带新 F 的根），重开写 0..2 次，改坏全部带 F 的根：F 回落之后候选根坏没坏。
#[test]
fn g3_complete_chain_control() {
    set_deleted(false);
    let (mut session, open0) = Session::new();
    drop(open0);
    let mut open = session.mount();
    session.try_overwrite(&mut open).expect("B");
    while device_of_txg(open.current.root.checkpoint_txg.0 + 1) != DeviceIdentity(0) {
        session.try_overwrite(&mut open).expect("写");
    }
    let chain = session.unmount_b1(&mut open, false).expect("B1");
    drop(open);
    let complete = session.image();
    let contents = session.contents.clone();
    for (max, hold) in MODES {
        set_rule(max);
        set_hold(hold);
        for writes in 0..=2 {
            let mut s = Session::from_image(&complete, &contents);
            let mut o = s.mount();
            for _ in 0..writes { s.try_overwrite(&mut o).expect("写"); }
            drop(o);
            let mut image = s.image();
            let carriers: Vec<u64> = roots_of(&image).iter().filter(|r| r.rollback_floor.0 > 0).map(|r| r.checkpoint_txg.0).collect();
            let on_one = floor_roots_on(&image, 1);
            let mut one = image.clone();
            corrupt_roots(&mut one, &on_one);
            corrupt_roots(&mut image, &carriers);
            println!("RBF2 g3_control rule={} chain={chain:?} writes={writes} kill_dev1_carriers={on_one:?} F_after={} cand_bad={:?} || kill_all_carriers={carriers:?} F_after={} cand_bad={:?}",
                rule_name(max), floor_of(&one), ring_audit(&one, &s.contents).0, floor_of(&image), ring_audit(&image, &s.contents).0);
        }
    }
    set_rule(false);
    set_hold(false);
}

/// 逐点崩溃（按写的先后，每一个前缀是一个崩溃点，共 写数 + 1 个）：每点判恢复落点与内容（盘上最新持久的根不许被越过）、
/// 另把最新 1..3 条可读根改坏再判、环里候选根逐条读、checker。整段子集穷举太大的流用它。
fn crash_points_prefix(name: &str, base: &MemoryPool, session: &Session) {
    let operations = session.stream.retained_operations();
    let (writes, _segments) = writes_and_segments(&operations, &common::geometry());
    let mut bad = 0u64;
    let mut first_bad: Option<String> = None;
    let mut probe_bad = [0u64; 3];
    let mut probe_first: [Option<String>; 3] = [None, None, None];
    let mut audit_bad = 0u64;
    let mut audit_first: Option<String> = None;
    let mut reds: BTreeMap<String, u64> = BTreeMap::new();
    for point in 0..=writes.len() {
        let mut state = base.clone();
        state.apply_writes(&writes[..point]);
        let newest = roots_of(&state).last().map(|r| (r.checkpoint_txg.0, r.instance.0));
        let verdict = recover_and_judge(&state, &session.contents);
        let regressed = match (&verdict, newest) {
            (Ok((instance, txg)), Some((newest_txg, newest_instance))) => (*txg, *instance) < (newest_txg, newest_instance),
            _ => false,
        };
        if verdict.is_err() || regressed {
            bad += 1;
            first_bad.get_or_insert(format!("point={point} {verdict:?} newest={newest:?}"));
        }
        let roots = roots_of(&state);
        for killed in 1..=3usize {
            if roots.len() <= killed { continue; }
            let newest_txgs: Vec<u64> = roots[roots.len() - killed..].iter().map(|r| r.checkpoint_txg.0).collect();
            let mut damaged = state.clone();
            corrupt_roots(&mut damaged, &newest_txgs);
            if let Err(reason) = recover_and_judge(&damaged, &session.contents) {
                probe_bad[killed - 1] += 1;
                probe_first[killed - 1].get_or_insert(format!("point={point} killed={newest_txgs:?} {reason}"));
            }
        }
        let (cand_bad, _) = ring_audit(&state, &session.contents);
        if !cand_bad.is_empty() {
            audit_bad += 1;
            audit_first.get_or_insert(format!("point={point} {cand_bad:?}"));
        }
        for red in checker_red_names(&state) {
            *reds.entry(red).or_default() += 1;
        }
    }
    println!("RBF2 prefix name={name} points={} bad={bad} first_bad={first_bad:?} probe_bad(kill1,kill2,kill3)={probe_bad:?} probe_first={probe_first:?} audit_bad={audit_bad} audit_first={audit_first:?} checker_reds_by_invariant={reds:?}",
        writes.len() + 1);
}

/// 逐点崩溃，全部流：回退到 A / 上一个状态、回退-写-回退-写、跨多单元文件与 300 个 inode 的回退、B1 那一串三种长度（取最大值规则）。
#[test]
fn g1_g4_crash_points_prefix() {
    set_deleted(false);
    set_rule(false);
    for target_kind in ["A", "previous"] {
        let (mut session, mut open) = prefix_three_overwrites();
        let target = if target_kind == "A" { (1, 3) } else { (open.instance.0, open.current.root.checkpoint_txg.0 - 1) };
        let base = session.restart_stream();
        session.rollback(&mut open, target, Rbf2Arm::Full).expect("回退");
        crash_points_prefix(&format!("rollback-to-{target_kind}"), &base, &session);
    }
    {
        let (mut session, mut open) = prefix_three_overwrites();
        let newest_before = (open.instance.0, open.current.root.checkpoint_txg.0);
        let base = session.restart_stream();
        session.rollback(&mut open, (1, 3), Rbf2Arm::Full).expect("回退 1");
        session.try_overwrite(&mut open).expect("写 1");
        session.rollback(&mut open, newest_before, Rbf2Arm::Full).expect("回退 2");
        session.try_overwrite(&mut open).expect("写 2");
        crash_points_prefix("rollback-write-rollback-write", &base, &session);
    }
    {
        let (mut session, open0) = Session::new();
        drop(open0);
        let mut open = session.mount();
        session.try_overwrite(&mut open).expect("B");
        let b = (open.instance.0, open.current.root.checkpoint_txg.0);
        session.try_seq_write(&mut open).expect("S");
        session.try_new_inodes(&mut open, 300).expect("I");
        let base = session.restart_stream();
        session.rollback(&mut open, b, Rbf2Arm::Full).expect("回退");
        session.try_new_inodes(&mut open, 3).expect("回退之后建 inode");
        crash_points_prefix("rollback-over-seq-and-inodes-then-inodes", &base, &session);
    }
    for (max, hold) in MODES {
        set_rule(max);
        set_hold(hold);
        for extra in 0..3 {
            let (mut session, open0) = Session::new();
            drop(open0);
            let mut open = session.mount();
            for _ in 0..(1 + extra) { session.try_overwrite(&mut open).expect("写"); }
            let base = session.restart_stream();
            let chain = session.unmount_b1(&mut open, false).expect("B1");
            drop(open);
            let mut reopened = session.mount();
            session.try_overwrite(&mut reopened).expect("重开之后写");
            crash_points_prefix(&format!("b1-chain{chain:?}-remount-write rule={}", rule_name(max)), &base, &session);
        }
    }
    set_rule(false);
    set_hold(false);
}

/// G1：来回回退之后准入抬 F（今天的上限：第 4 新的非空根）：候选集里还剩几个「可退到的不同状态」（按内容数）。
/// 回退根按「非空」的认法（树表里 inode / extent 根指针与前一条有效根不同）算非空，而它的内容与更早的某一版相同。
#[test]
fn g1_distinct_states_after_ping_pong_rollbacks_and_a_raise() {
    set_rule(true);
    set_deleted(false);
    for pattern in ["none", "A-D", "A-D-A", "A-D-A-D", "A-D-A-D-A-D"] {
        let (mut session, open0) = Session::new();
        drop(open0);
        let mut open = session.mount();
        for _ in 0..3 { session.try_overwrite(&mut open).expect("写"); }
        let d = (open.instance.0, open.current.root.checkpoint_txg.0);
        if pattern != "none" {
            for step in pattern.split('-') {
                let target = if step == "A" { (1, 3) } else { d };
                session.rollback(&mut open, target, Rbf2Arm::Full).expect("回退");
            }
        }
        let system_configuration = choose_system_configuration(&session.image()).expect("系统配置");
        let table = instance_table_chain_of_root(&session.devices, &open.current.root).expect("表").records;
        let ceiling = singlefs_core::mount::rollback_floor_ceiling(&session.devices, &system_configuration, open.current.root.rollback_floor, &table).expect("上限").0;
        let raised = session.raise(&mut open, ceiling, false);
        let image = session.image();
        let cands = candidates(&image);
        let distinct: BTreeSet<Vec<u8>> = cands.iter().filter_map(|r| session.contents.get(&(r.instance.0, r.checkpoint_txg.0)).cloned()).collect();
        let with_file = cands.iter().filter(|r| r.checkpoint_txg.0 >= 3).count();
        println!("RBF2 g1_distinct pattern={pattern} ceiling={ceiling} raise={:?} F={} candidates={:?} candidate_roots_with_file={with_file} distinct_states={}",
            raised.map(|c| c.len()), floor_of(&image), cands.iter().map(|r| (r.instance.0, r.checkpoint_txg.0)).collect::<Vec<_>>(), distinct.len());
    }
    set_rule(false);
}

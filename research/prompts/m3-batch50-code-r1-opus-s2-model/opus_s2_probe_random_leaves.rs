//! s2 攻方原型 K1 / K3 / K4 / K6：多片 inode 叶上的随机历史（建一批空 inode、删任意号、成串删一片叶里的号、挂着回退到环里任一版、
//! 进程崩溃再可写挂载、覆盖写第一个文件、抬 F 到准入上限），每一步之后：池级 checker 在整份镜像上判（合法历史上任何一条 Violated 都是误报）；
//! 我自己的叶结构判据（与实现、checker 都不共用代码）；每次删除类发布的退役完备（上一版叶身份 − 这一版 = 死于这一版的退役记录）；
//! 退役树只追加；inode 号不在同一条时间线上重发（每个号记第一次见到的出生代，同一版里同号出生代变了就记）。只在攻方副本上跑。
//! 参数：S2_SEED_FROM / S2_SEEDS（种子区间）、S2_STEPS（每段步数）。一行一段：`name=hist seed=… steps=… checker_red=… oracle_red=…`，
//! 红的另起 `name=red …` 行带步序与原文。
mod common_admission_pool;

use std::collections::{BTreeMap, BTreeSet};

use common_admission_pool::{
    checker_violations_on, parameters_of_the_384_slot_small_devices, plain_device, PoolUnderTest,
};
use singlefs_core::address::{CheckpointTxg, InodeNumber, InstanceGeneration};
use singlefs_core::allocator::UnitAreaStart;
use singlefs_core::mount::{
    raise_rollback_floor_to_the_admission_ceiling, roll_back_by_a_forward_publish, RollbackTarget,
};
use singlefs_core::mounted_session::UserChange;
use singlefs_core::transaction::{PoolVersion, TransactionOutput};
use singlefs_core::unit::PackedIdentity as RawIdentity;

/// 身份四段做成可排序的键（PackedIdentity 没有 Ord）。
type PackedIdentity = (u64, u16, u64, u64);
fn id(raw: &RawIdentity) -> PackedIdentity {
    (raw.birth_tree.0, raw.record_type, raw.container, raw.container_birth.0)
}
use singlefs_format::SLOT_BYTES;
use singlefs_harness::memory_pool::SparseBlockDevice;

type Pool = PoolUnderTest<SparseBlockDevice>;

fn env_u64(name: &str, default: u64) -> u64 {
    std::env::var(name).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}

/// S2_KNOBS 设了就装只供测试的容量开关：退役容器 1 条一片、容器树码 2 节点 2 条（退役树、墓碑树、谱系树根满长高走得快）；批大小与墓碑、谱系容器照产品。
fn install_knobs_if_asked(pool: &mut Pool) {
    if std::env::var("S2_KNOBS").is_err() {
        return;
    }
    let switches = singlefs_core::unlink::DeletionSwitches {
        retirement_container_capacity: singlefs_core::retirement_tree::RetirementContainerCapacity::CappedForTests(
            singlefs_core::retirement_tree::RetirementRecordsPerContainerCappedForTests::new(1).expect("1 条"),
        ),
        container_tree_node_entries: singlefs_core::container_tree::ContainerTreeNodeEntries::CappedForTests(
            singlefs_core::container_tree::ContainerTreeNodeEntriesCappedForTests::new(2).expect("2 条"),
        ),
        ..singlefs_core::unlink::DeletionSwitches::PRODUCT
    };
    if let Some(session) = pool.session.as_mut() {
        session.allocator.set_deletion_switches(switches);
    }
}

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        // xorshift64*
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    fn below(&mut self, n: u64) -> u64 {
        if n == 0 { 0 } else { self.next() % n }
    }
}

/// 两块盘：单元区起点（6 MiB 环末尾的下一个槽）+ 环窗口补足 + 60 000 槽。
fn device_bytes() -> u64 {
    let start = UnitAreaStart::following_the_journal_ring(6 << 20).expect("1408").slot().0;
    (start + 1007 + 60_000) * SLOT_BYTES
}

fn leaf_identities(v: &TransactionOutput) -> BTreeSet<PackedIdentity> {
    v.inode_leaf_containers.iter().map(|c| id(&c.contents.identity)).collect()
}

fn retirement_records(v: &TransactionOutput) -> Vec<(PackedIdentity, u64)> {
    v.retirement_containers
        .iter()
        .flat_map(|c| c.contents.records.iter().map(|r| (id(&r.retired), r.death_txg.0)))
        .collect()
}

/// 叶结构判据（自己写的）：每片 1..=233 条、按号严格升序、号 ≥ 容器号与分隔 key；相邻两片：左片最大号 < 右片分隔 key ≤ 右片最小号；
/// 分隔 key 严格升序；身份两两不同。
fn leaf_structure_errors(v: &TransactionOutput) -> Vec<String> {
    let mut errors = Vec::new();
    let mut previous_max: Option<u64> = None;
    let mut previous_separator: Option<u64> = None;
    let mut seen = BTreeSet::new();
    for (i, c) in v.inode_leaf_containers.iter().enumerate() {
        let leaf = &c.contents;
        let n = leaf.records.len();
        if n == 0 || n > 233 {
            errors.push(format!("leaf {i} has {n} records"));
        }
        if !seen.insert(id(&leaf.identity)) {
            errors.push(format!("leaf {i} identity repeated"));
        }
        let sep = leaf.separator_key_in_the_parent_entry;
        for w in leaf.records.windows(2) {
            if w[0].inode >= w[1].inode {
                errors.push(format!("leaf {i} not ascending"));
            }
        }
        if let Some(first) = leaf.records.first() {
            if first.inode < sep || first.inode < leaf.identity.container {
                errors.push(format!("leaf {i} min {} < sep {sep} or container {}", first.inode, leaf.identity.container));
            }
        }
        if let Some(pm) = previous_max {
            if !(pm < sep) {
                errors.push(format!("leaf {i} sep {sep} <= left max {pm}"));
            }
        }
        if let Some(ps) = previous_separator {
            if ps >= sep {
                errors.push(format!("leaf {i} sep {sep} not above left sep {ps}"));
            }
        }
        previous_max = leaf.records.last().map(|r| r.inode).or(previous_max);
        previous_separator = Some(sep);
    }
    errors
}

/// 一次删除类发布（prev → next）的退役完备与只追加：prev 的叶身份 − next 的 = next 里死亡 txg 等于 next 的 txg 的退役记录（集合、条数都比）；
/// next 的退役记录以 prev 的为前缀。
fn retirement_errors(prev: &TransactionOutput, next: &TransactionOutput) -> Vec<String> {
    let mut errors = Vec::new();
    let gone: BTreeSet<PackedIdentity> = leaf_identities(prev).difference(&leaf_identities(next)).copied().collect();
    let txg = next.root.checkpoint_txg.0;
    let before = retirement_records(prev);
    let after = retirement_records(next);
    if after.len() < before.len() || after[..before.len()] != before[..] {
        errors.push(format!("retirement records not append-only at txg {txg}"));
    }
    let new_ones: Vec<(PackedIdentity, u64)> = after[before.len().min(after.len())..].to_vec();
    let new_ids: BTreeSet<PackedIdentity> = new_ones.iter().map(|(id, _)| *id).collect();
    if new_ones.iter().any(|(_, d)| *d != txg) {
        errors.push(format!("new retirement record death txg != {txg}: {new_ones:?}"));
    }
    if new_ids != gone || new_ones.len() != gone.len() {
        errors.push(format!("txg {txg}: gone {} vs recorded {} ({:?} vs {:?})", gone.len(), new_ones.len(), gone, new_ids));
    }
    errors
}

fn live_inodes(v: &TransactionOutput) -> Vec<u64> {
    v.inode_leaf_containers
        .iter()
        .flat_map(|c| c.contents.records.iter())
        .filter(|r| r.inode != 1 && r.nlink >= 1)
        .map(|r| r.inode)
        .collect()
}

struct History {
    pool: Pool,
    rng: Rng,
    published: Vec<(InstanceGeneration, CheckpointTxg)>,
    births: BTreeMap<u64, BTreeSet<u64>>,
    checker_red: u64,
    oracle_red: u64,
    deletion_publishes: u64,
    retirements_seen: u64,
    borrows_seen: u64,
    max_leaves: usize,
    max_gone_in_one_publish: usize,
    rollbacks_done: u64,
    rollbacks_refused: u64,
    seed: u64,
}

impl History {
    fn current(&self) -> TransactionOutput {
        self.pool.current_file_version().clone()
    }

    fn note(&mut self, step: usize, what: &str, versions: &[TransactionOutput], prev: Option<&TransactionOutput>, deletion: bool) {
        let mut previous = prev.cloned();
        for v in versions {
            self.published.push((v.root.instance, v.root.checkpoint_txg));
            let mut errs = leaf_structure_errors(v);
            if deletion {
                if let Some(p) = &previous {
                    let e = retirement_errors(p, v);
                    if !e.is_empty() {
                        errs.extend(e);
                    }
                    self.deletion_publishes += 1;
                    let gone = leaf_identities(p).difference(&leaf_identities(v)).count();
                    self.retirements_seen += gone as u64;
                    self.max_gone_in_one_publish = self.max_gone_in_one_publish.max(gone);
                    // 借：两片身份都在、一片分隔 key 变了
                    let seps_before: BTreeMap<PackedIdentity, u64> = p.inode_leaf_containers.iter().map(|c| (id(&c.contents.identity), c.contents.separator_key_in_the_parent_entry)).collect();
                    if v.inode_leaf_containers.iter().any(|c| seps_before.get(&id(&c.contents.identity)).is_some_and(|s| *s != c.contents.separator_key_in_the_parent_entry)) {
                        self.borrows_seen += 1;
                    }
                }
            }
            self.max_leaves = self.max_leaves.max(v.inode_leaf_containers.len());
            for e in errs {
                self.oracle_red += 1;
                println!("name=red kind=oracle seed={} step={step} op={what} txg={} {e}", self.seed, v.root.checkpoint_txg.0);
            }
            previous = Some(v.clone());
        }
    }

    fn check_image(&mut self, step: usize, what: &str) {
        let violations = checker_violations_on(&self.pool.image());
        for v in &violations {
            self.checker_red += 1;
            println!("name=red kind=checker seed={} step={step} op={what} {}", self.seed, v.chars().take(if std::env::var("S2_TRACE").is_ok() { 4000 } else { 400 }).collect::<String>());
        }
        // 号不重发（同一条时间线上的一版里）：一个号在现行这一版里的出生代，与历史上见过的同号出生代不同，
        // 且那个旧出生代那一版没被崩溃恢复抛弃（这里不造抛弃，所以都算）——记下，交人看。
        let v = self.current();
        for c in &v.inode_leaf_containers {
            for r in &c.contents.records {
                let set = self.births.entry(r.inode).or_default();
                set.insert(r.object_birth.0);
                if set.len() > 1 {
                    self.oracle_red += 1;
                    println!("name=red kind=reissue seed={} step={step} op={what} inode {} births {:?}", self.seed, r.inode, set);
                }
            }
        }
    }

    fn create(&mut self, step: usize, count: u64) {
        let prev = self.current();
        self.pool.write_time_seconds += 1;
        let t = self.pool.write_time_seconds;
        match self.pool.publish(UserChange::NewInodes { count, write_time_seconds: t }, None) {
            Ok(p) => self.note(step, "create", &p.versions_published, Some(&prev), false),
            Err(e) => println!("name=note seed={} step={step} create refused {:?}", self.seed, format!("{e:?}").chars().take(80).collect::<String>()),
        }
    }

    fn unlink(&mut self, step: usize, inode: u64) {
        let prev = self.current();
        self.pool.write_time_seconds += 1;
        let t = self.pool.write_time_seconds;
        match self.pool.publish(UserChange::Unlink { inode: InodeNumber(inode), write_time_seconds: t }, None) {
            Ok(p) => self.note(step, "unlink", &p.versions_published, Some(&prev), true),
            Err(e) => {
                self.oracle_red += 1;
                println!("name=red kind=unlink-refused seed={} step={step} inode={inode} {}", self.seed, format!("{e:?}").chars().take(300).collect::<String>());
            }
        }
    }

    fn rollback(&mut self, step: usize) {
        if self.published.is_empty() {
            return;
        }
        let window = self.published.len().min(30) as u64;
        let pick = self.published.len() - 1 - self.rng.below(window) as usize;
        let (instance, checkpoint_txg) = self.published[pick];
        let mut session = self.pool.session.take().expect("会话");
        let PoolVersion::WithFile(current) = &mut session.current else { self.pool.session = Some(session); return; };
        let result = roll_back_by_a_forward_publish(
            &self.pool.parameters,
            &mut self.pool.devices,
            &mut session.allocator,
            current,
            RollbackTarget { instance, checkpoint_txg },
        );
        let after = current.clone();
        self.pool.session = Some(session);
        match result {
            Ok(_) => {
                self.rollbacks_done += 1;
                self.note(step, "rollback", std::slice::from_ref(&after), None, false);
            }
            Err(_) => self.rollbacks_refused += 1,
        }
    }

    fn raise(&mut self) {
        let mut session = self.pool.session.take().expect("会话");
        if let PoolVersion::WithFile(current) = &mut session.current {
            let _ = raise_rollback_floor_to_the_admission_ceiling(&self.pool.parameters, &mut self.pool.devices, &mut session.allocator, current, session.shadow_ledger);
        }
        self.pool.session = Some(session);
    }

    fn crash_and_mount(&mut self, step: usize) {
        let mounted = self.pool.crash_and_mount_writable();
        install_knobs_if_asked(&mut self.pool);
        match mounted {
            Ok(out) => {
                let v = self.current();
                self.published.push((v.root.instance, v.root.checkpoint_txg));
                let _ = out;
            }
            Err(e) => {
                self.oracle_red += 1;
                println!("name=red kind=mount-refused seed={} step={step} {}", self.seed, format!("{e:?}").chars().take(300).collect::<String>());
            }
        }
    }
}

#[test]
fn random_histories_over_several_inode_leaves() {
    let seed_from = env_u64("S2_SEED_FROM", 1);
    let seeds = env_u64("S2_SEEDS", 4);
    let steps = env_u64("S2_STEPS", 30) as usize;
    // 迭代上界：S2_SEEDS 段，每段 S2_STEPS 步。
    for seed in seed_from..seed_from + seeds {
        let pool: Pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
            parameters_of_the_384_slot_small_devices(),
            device_bytes(),
            plain_device,
        );
        let mut pool = pool;
        install_knobs_if_asked(&mut pool);
        let mut h = History {
            pool,
            rng: Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1),
            published: Vec::new(),
            births: BTreeMap::new(),
            checker_red: 0,
            oracle_red: 0,
            deletion_publishes: 0,
            retirements_seen: 0,
            borrows_seen: 0,
            max_leaves: 0,
            max_gone_in_one_publish: 0,
            rollbacks_done: 0,
            rollbacks_refused: 0,
            seed,
        };
        let first = h.current();
        h.published.push((first.root.instance, first.root.checkpoint_txg));
        for step in 0..steps {
            let roll = h.rng.below(100);
            let op = if std::env::var("S2_CHURN").is_ok() && roll % 4 == 0 {
                // 成片生灭：建 233 个、再从最大的号起全删掉（每一轮至少一片叶出生、一片叶死亡）。
                let watermark = h.current().inode_number_watermark();
                h.create(step, 233);
                for inode in (watermark..watermark + 233).rev() {
                    h.unlink(step, inode);
                }
                "churn"
            } else if roll < 22 {
                let counts = [1, 3, 60, 116, 117, 118, 232, 233, 234, 300];
                let count = counts[h.rng.below(counts.len() as u64) as usize];
                h.create(step, count);
                "create"
            } else if roll < 45 {
                let live = live_inodes(&h.current());
                if !live.is_empty() {
                    let inode = live[h.rng.below(live.len() as u64) as usize];
                    h.unlink(step, inode);
                }
                "unlink"
            } else if roll < 65 {
                // 成串删一片叶里的号：挑一片叶，从它的最小号或最大号起删 r 个。
                let v = h.current();
                let leaves = v.inode_leaf_containers.len() as u64;
                let leaf = &v.inode_leaf_containers[h.rng.below(leaves) as usize].contents;
                let mut numbers: Vec<u64> = leaf.records.iter().filter(|r| r.inode != 1 && r.nlink >= 1).map(|r| r.inode).collect();
                if h.rng.below(2) == 0 { numbers.reverse(); }
                let r = 1 + h.rng.below(numbers.len().max(1) as u64) as usize;
                for inode in numbers.into_iter().take(r.min(130)) {
                    h.unlink(step, inode);
                }
                "unlink-run"
            } else if roll < 77 {
                h.rollback(step);
                "rollback"
            } else if roll < 85 {
                h.crash_and_mount(step);
                "crash-mount"
            } else if roll < 93 {
                h.raise();
                "raise"
            } else {
                let _ = h.pool.overwrite(2999);
                "overwrite"
            };
            if std::env::var("S2_TRACE").is_ok() {
                let txg = h.pool.session.as_ref().map(|s| s.current.root().checkpoint_txg.0);
                println!("name=trace seed={seed} step={step} op={op} txg={txg:?}");
            }
            if h.pool.session.as_ref().is_some_and(|s| s.current.file_version().is_some()) {
                h.check_image(step, op);
            }
        }
        if h.pool.session.as_ref().is_none_or(|s| s.current.file_version().is_none()) {
            println!("name=hist seed={seed} stopped-without-file-version checker_red={} oracle_red={}", h.checker_red, h.oracle_red);
            continue;
        }
        let retirement_records = h.current().retirement_containers.iter().map(|c| c.contents.records.len()).sum::<usize>();
        let retirement_containers = h.current().retirement_containers.len();
        let retirement_levels = h.current().retirement_tree_internal_levels();
        println!("name=retirement-tree seed={seed} records={retirement_records} containers={retirement_containers} internal_levels={retirement_levels}");
        println!(
            "name=hist knobs={} seed={seed} steps={steps} checker_red={} oracle_red={} deletion_publishes={} retirements={} borrows={} max_leaves={} max_gone_in_one_publish={} rollbacks={}/{} final_txg={}",
            std::env::var("S2_KNOBS").is_ok(), h.checker_red, h.oracle_red, h.deletion_publishes, h.retirements_seen, h.borrows_seen, h.max_leaves, h.max_gone_in_one_publish, h.rollbacks_done, h.rollbacks_done + h.rollbacks_refused, h.current().root.checkpoint_txg.0
        );
    }
}

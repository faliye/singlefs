//! m3-w3-r3 正推腿（Sonnet）的抽象时间线模型：在 m3-w3-r2 辩方腿（Sonnet）的模型（它又改自 m3-w3-r1 攻方腿的模型）上并进 m3-w3-r2 攻方腿（Opus）的
//! 候选（C2aL、C3rL 的最强形与 I-9.30 草案的几种读法、六种新变异），为的是三个形态放在同一组历史、同一套口径上并排；再加 Y1 两种取法的两个臂
//! 与一个控制臂（见 Arm）。老候选的列与 r2 辩方腿的模型逐数相同（`rerun.sh compare-r2` 核）；新候选追加在后面。
//! m3-w3-r2 辩方腿（Sonnet）在 m3-w3-r1 攻方腿的抽象时间线模型上加了：墓碑（跟现行那一版走、不回收）、谱系树 (T_rb, T_old)、
//! 被第一轮判出局的候选的最强形态（C1b、C2i、C4g 等）、「现有检查」（I-9.29 含谱系上的墓碑、I-9.6 含墓碑那一半）口径的并集计数。
//! 原文件头：m3-w3-r1 攻方腿（Opus）的抽象时间线模型。只照条文走，不读 crates/：
//! - D8 已定项 6 射程：「时间线」= 所选根的祖先链，含管理员回退抛弃的那一段、不含崩溃恢复抛弃的那一段；
//! - D23 已定项 14：管理员回退是挂着时的一次向前发布，水位取 max(cur, 环里读得出的根)（V1 臂改成取目标那一版的）；
//!   候选集 = 环里按 cur 的实例表有效 ∧ txg ≥ F ∧ 带文件；实例切换重新读盘择根、写行、重发在飞的 checkpoint；
//! - D18 已定项 11：行只在恢复与切换时写，(i, T_pub, W)；按行判抛弃；行在根环里没有该实例的根、行数超过阈值时被清扫删掉。
//! 每一步之后对「镜像」判各候选：只读最新那条根的记录（根下的候选）、它的实例表、盘上的定长发号区（C3f 那几种）。
//! 由用户定的步（什么时候崩、回退到哪、建几个、什么时候挂载、阈值）全部放开穷举，只固定起点（新池新建文件之后）。
use std::collections::BTreeMap;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Arm {
    Product,
    RollbackTakesTheTargetsWatermark,
    PublishDoesNotAddTheNewInodes,
    /// 辩方腿加的第三个阳性对照臂 V3：恢复（挂载、崩溃恢复、实例切换）时新实例的水位比所选根的少 1（不低于 2），即「恢复时水位取错」，与回退无关。
    RecoveryLowersTheWatermark,
    /// 第三轮 Y1 取法二：管理员回退的水位 = max(cur 内存里的, 环里读得出且按 cur 的实例表判没被抛弃的根的)。
    ProductWithoutAbandonedRoots,
    /// 第三轮控制臂：管理员回退的水位只取环里读得出的根，不与 cur 内存里的取 max（C143 第三轮判决里出局的老写法，
    /// `crates/mutations.tsv` 第 717 行那条变异的模型版）；只在 rollback-blind 那一步上看得出它。
    RollbackRingOnly,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mutation {
    None,
    /// 回退那次，根下的发号历史（C2*、C3r）与谱系里那份「回退前水位」都取 R_old 那一版的（V1 的同源错）。
    HistoryFromTheTarget,
    /// 历史只照抄这一版自己的值，不从父版本带（C2c 峰值 = 水位、C2a 只留这一代的行、C3r 只留这一版的树……）。
    HistoryCopiesThisVersion,
    /// 历史一样都不写（C1 的 M 写 0）。
    HistoryNotWritten,
    /// 第三轮新增（m3-w3-r2 攻方腿的 H）：谱系那一对 (H_cur, count) 取 R_old 那一版的，历史本身照对。
    LineagePairFromTheTarget,
    /// 第三轮新增（Z）：谱系那一对写 0。
    LineagePairZero,
    /// 第三轮新增（R）：回退那次跳升的台阶行不写。
    RollbackRowNotWritten,
    /// 第三轮新增（D）：台阶行的 H 记成这次发布之前的水位。
    RowsRecordTheWatermarkBefore,
    /// 第三轮新增（Mh）：崩溃恢复抛弃最新那条根时，新实例的历史从被抛弃的那条根带。
    MountHistoryFromTheAbandonedRoot,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Op {
    Create,
    Delete,
    Rollback(u8),
    RaiseFloor,
    Empty,
    Mount,
    CrashHidingTheNewestRoot,
    CrashInTheMiddleOfACreate,
    SwitchRedoingTheCreate,
    SwitchFailingTheCreate,
    /// 第三轮新增（只有 W3_OPS 点名 rollback-blind 才进字母表）：回退 1，但算水位那一遍读根环时 cur 自己的根槽瞬时读不出
    /// （C143 第三轮判决 K1 那一格：判候选两遍读得出、算水位那一遍读不出）。
    RollbackBlind,
}

impl Op {
    const ALL: [Op; 13] = [
        Op::Create,
        Op::Delete,
        Op::Rollback(1),
        Op::Rollback(2),
        Op::Rollback(3),
        Op::RaiseFloor,
        Op::Empty,
        Op::Mount,
        Op::CrashHidingTheNewestRoot,
        Op::CrashInTheMiddleOfACreate,
        Op::SwitchRedoingTheCreate,
        Op::SwitchFailingTheCreate,
        Op::RollbackBlind,
    ];

    fn code(self) -> String {
        match self {
            Op::Create => "create".into(),
            Op::Delete => "delete".into(),
            Op::Rollback(k) => format!("rollback{k}"),
            Op::RaiseFloor => "raiseF".into(),
            Op::Empty => "empty".into(),
            Op::Mount => "mount".into(),
            Op::CrashHidingTheNewestRoot => "crash-hide".into(),
            Op::CrashInTheMiddleOfACreate => "crash-mid-create".into(),
            Op::SwitchRedoingTheCreate => "switch-redo".into(),
            Op::SwitchFailingTheCreate => "switch-error".into(),
            Op::RollbackBlind => "rollback-blind".into(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Config {
    arm: Arm,
    mutation: Mutation,
    ring_slots: usize,
    /// 行数超过它就在挂载末尾清扫；u64::MAX 表示从不清扫（产品占位 369 在短历史里够不着）。
    sweep_threshold: u64,
    longest: usize,
}

/// 实例表一行：(实例, T_pub, M)；M 有两份读法：盘上所选那一版的水位（C1），与切换时内存里的水位（C1m）。
#[derive(Clone, PartialEq, Eq, Debug)]
struct Row {
    instance: u32,
    selected_txg: u64,
    m_disk: u64,
    m_memory: u64,
    m_true: u64,
    /// C1b：写这一行那一刻，根环里读得出的、该实例且 txg ≤ T 的根的水位最大值（Sonnet 正推的 C1b）。
    m_ring: u64,
    m_ring_true: u64,
}

/// 根下的候选记录。
#[derive(Clone, PartialEq, Eq, Debug, Default)]
struct Records {
    /// C2a：历代 (代, 水位)。
    generation_rows: Vec<(u64, u64)>,
    /// C2b：每个实例最后一代的水位。
    per_instance_last: Vec<(u32, u64)>,
    /// C2c：峰值。
    peak: u64,
    /// C2d：(峰值, 第一次到达它的代)。
    peak_pair: (u64, u64),
    /// C3r：根下只追加的 (号, 出生代)。
    log: Vec<(u64, u64)>,
    /// C4：谱系记录带的回退前水位里最大的那个。
    lineage_max: u64,
    /// 这条时间线上写过的每一行 (实例, T_pub, M)，清扫删掉的也留着：只在 true_records 里维护，给 C1k 用，不是盘上的东西。
    rows_ever: Vec<(u32, u64, u64)>,
    /// C2ay：谱系记录多带的 (cur 的 txg, 回退前水位)。谱系树随现行那一版，所以只在 true_records 里带；值按变异写（攻方腿提的改法，零轮）。
    lineage_entries: Vec<(u64, u64)>,
    /// C2i：每个实例的 (实例, 最高水位, 第一次到它的代)。
    peak_by_instance: Vec<(u32, u64, u64)>,
    /// C4g：谱系记录多带 (T_rb, 回退前 cur 的水位)，判时 A 半句水位 < W、B 半句 (号, 出生代) 满足号 < W 且出生代 > T_rb。随现行那一版（谱系树的载体），回退那次追加一条。
    lin: Vec<(u64, u64)>,
    /// C1bk：清扫删掉的行也留着的 (实例, T, m_ring)，不是盘上的东西（真相侧）。
    rows_ever_ring: Vec<(u32, u64, u64)>,
    /// 第三轮（C2aL、C2a-all）：台阶行 (代, 水位)，每次水位变了的发布留一行（建 inode 的发布、回退跳升那次）。
    rows_step: Vec<(u64, u64)>,
    /// 第三轮（I-9.30 草案字面的 C2a-create）：只在建 inode 的发布留台阶行。
    rows_create: Vec<(u64, u64)>,
    /// 第三轮：谱系记录的完整一份（m3-w3-r2 攻方腿的六元组）。随现行那一版，回退那次追加一条。
    lineage6: Vec<Lin>,
}

/// 谱系记录的六元组：(T_rb, T_old, T_cur, H_cur, count, H_new)。T_cur = 回退之前现行那一版的 txg，H_cur = 它的水位，
/// count = 它那一版发号记录（C3r 的 log）的条数，H_new = 回退那次发布之后的水位。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Lin {
    t_rb: u64,
    t_old: u64,
    t_cur: u64,
    h_cur: u64,
    count: u64,
    h_new: u64,
}

#[derive(Clone, Debug)]
struct Version {
    instance: u32,
    txg: u64,
    watermark: u64,
    tree: Vec<(u64, u64)>,
    floor: u64,
    table: Vec<Row>,
    records: Records,
    true_records: Records,
    /// 真相：这条时间线上发过的 (号, 出生代)。
    issued: Vec<(u64, u64)>,
    /// 墓碑树：(号, 出生代, 死亡 txg)。随现行那一版、第一版不回收（D18 已定项 24 第 4、5 条）。
    tombs: Vec<(u64, u64, u64)>,
    /// 谱系树：(T_rb, T_old)，随现行那一版、只增不删（D18 已定项 24 第 5 条）。
    lineage_tree: Vec<(u64, u64)>,
}

#[derive(Clone, Debug)]
struct Slot {
    version: Version,
    readable: bool,
}

/// 盘上定长发号区的一条（C3f 三种读法共用）：(实例, txg, 号, 出生代)，写在那次发布的根之前、单独持久。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct FixedEntry {
    instance: u32,
    txg: u64,
    inode: u64,
    birth: u64,
}

fn union_sorted(base: &[(u64, u64)], extra: &[(u64, u64)]) -> Vec<(u64, u64)> {
    let mut out: Vec<(u64, u64)> = base.iter().chain(extra.iter()).copied().collect();
    out.sort_unstable();
    out.dedup();
    out
}

fn upsert<K: PartialEq + Copy, V: Copy>(list: &mut Vec<(K, V)>, key: K, value: V) {
    if let Some(entry) = list.iter_mut().find(|(k, _)| *k == key) {
        entry.1 = value;
    } else {
        list.push((key, value));
    }
}

/// 照写规则从父版本的记录带出这一版的记录（正确写法）。
#[allow(clippy::too_many_arguments)]
fn extend_records(
    base: &Records,
    instance: u32,
    txg: u64,
    watermark: u64,
    tree: &[(u64, u64)],
    watermark_before: u64,
    created: bool,
    row_value: u64,
) -> Records {
    let mut records = base.clone();
    if created {
        records.rows_create.push((txg, row_value));
    }
    if watermark != watermark_before {
        records.rows_step.push((txg, row_value));
    }
    upsert(&mut records.generation_rows, txg, watermark);
    upsert(&mut records.per_instance_last, instance, watermark);
    if watermark > records.peak {
        records.peak = watermark;
        records.peak_pair = (watermark, txg);
    }
    match records.peak_by_instance.iter_mut().find(|(i, _, _)| *i == instance) {
        Some(entry) => {
            if watermark > entry.1 {
                entry.1 = watermark;
                entry.2 = txg;
            }
        }
        None => records.peak_by_instance.push((instance, watermark, txg)),
    }
    records.log = union_sorted(&records.log, tree);
    records
}

#[allow(clippy::too_many_arguments)]
fn mutated_records(
    mutation: Mutation,
    base: &Records,
    instance: u32,
    txg: u64,
    watermark: u64,
    tree: &[(u64, u64)],
    watermark_before: u64,
    created: bool,
) -> Records {
    match mutation {
        Mutation::None
        | Mutation::HistoryFromTheTarget
        | Mutation::LineagePairFromTheTarget
        | Mutation::LineagePairZero
        | Mutation::RollbackRowNotWritten
        | Mutation::MountHistoryFromTheAbandonedRoot => {
            extend_records(base, instance, txg, watermark, tree, watermark_before, created, watermark)
        }
        Mutation::RowsRecordTheWatermarkBefore => {
            extend_records(base, instance, txg, watermark, tree, watermark_before, created, watermark_before)
        }
        Mutation::HistoryCopiesThisVersion => Records {
            generation_rows: vec![(txg, watermark)],
            per_instance_last: vec![(instance, watermark)],
            peak: watermark,
            peak_pair: (watermark, txg),
            log: tree.to_vec(),
            lineage_max: base.lineage_max,
            rows_ever: Vec::new(),
            lineage_entries: Vec::new(),
            peak_by_instance: vec![(instance, watermark, txg)],
            lin: Vec::new(),
            rows_ever_ring: Vec::new(),
            rows_step: if watermark != watermark_before { vec![(txg, watermark)] } else { Vec::new() },
            rows_create: if created { vec![(txg, watermark)] } else { Vec::new() },
            lineage6: Vec::new(),
        },
        Mutation::HistoryNotWritten => Records::default(),
    }
}

#[derive(Clone)]
struct State {
    ring: Vec<Option<Slot>>,
    newest_txg_written: u64,
    newest_instance: u32,
    current: Version,
    fixed: Vec<FixedEntry>,
    fixed_erased_by_the_sweep: Vec<FixedEntry>,
    /// 自最近一次管理员回退起，有没有发生过恢复类的步（挂载、崩溃恢复、实例切换）。回退时清零。
    mounted_since_rollback: bool,
    /// 这一段历史里有过回退把水位从「只有被抛弃的根才有的高水位」抬上去的跳升（第三轮：误报归因用）。
    jumped_from_abandoned: bool,
    /// 这一段历史里有过清扫、实例切换或回到先前 cur 的回退（A → B → A）。
    sweep_switch_or_aba: bool,
}

/// 一步做完之后要判的东西：这一步发出去的 (号, 出生代)（有就是一次发号），这一步之后的镜像就是 State 本身。
#[derive(Default)]
struct StepResult {
    issued_now: Vec<(u64, u64)>,
    /// 回退这一步：环里读得出的根里有比 cur 水位高的，而且全是按 cur 的实例表判被抛弃的根。
    jump_from_abandoned_only: bool,
    /// 回退这一步：环里有没被抛弃的根比 cur 的水位高。
    jump_from_a_root_on_the_timeline: bool,
    swept: bool,
    switched: bool,
    rollback_back_to_a_former_cur: bool,
}

impl State {
    fn initial(ring_slots: usize) -> State {
        // 新池新建文件之后：实例 1，txg 3，inode 1 出生代 3，水位 2（D8 已定项 6「新池新建文件写」）。
        let tree = vec![(1, 3)];
        let records = extend_records(&Records::default(), 1, 3, 2, &tree, 0, true, 2);
        let version = Version {
            instance: 1,
            txg: 3,
            watermark: 2,
            tree: tree.clone(),
            floor: 0,
            table: Vec::new(),
            records: records.clone(),
            true_records: records,
            issued: tree,
            tombs: Vec::new(),
            lineage_tree: Vec::new(),
        };
        let mut state = State {
            ring: vec![None; ring_slots],
            newest_txg_written: 0,
            newest_instance: 1,
            current: version.clone(),
            fixed: vec![FixedEntry { instance: 1, txg: 3, inode: 1, birth: 3 }],
            fixed_erased_by_the_sweep: vec![FixedEntry { instance: 1, txg: 3, inode: 1, birth: 3 }],
            mounted_since_rollback: false,
            jumped_from_abandoned: false,
            sweep_switch_or_aba: false,
        };
        state.publish(version);
        state
    }

    fn publish(&mut self, version: Version) {
        let slots = self.ring.len();
        let slot = usize::try_from(version.txg).expect("txg 很小") % slots;
        self.newest_txg_written = self.newest_txg_written.max(version.txg);
        self.ring[slot] = Some(Slot { version: version.clone(), readable: true });
        self.current = version;
    }

    fn readable_versions(&self) -> Vec<&Version> {
        let mut out: Vec<&Version> = self.ring.iter().flatten().filter(|slot| slot.readable).map(|slot| &slot.version).collect();
        out.sort_by_key(|version| (version.txg, version.instance));
        out
    }

    fn newest_readable(&self) -> &Version {
        self.readable_versions().into_iter().last().expect("环里至少一条可读的根")
    }

    fn append_fixed(&mut self, entry: FixedEntry) {
        self.fixed.push(entry);
        self.fixed_erased_by_the_sweep.push(entry);
    }
}

fn abandoned_by(table: &[Row], instance: u32, txg: u64) -> bool {
    table.iter().any(|row| row.instance == instance && txg > row.selected_txg)
}

impl State {
    /// 在 `base`（内容来源）之上建一版：树、水位、F、实例表给定，记录从 `record_base` 带。
    #[allow(clippy::too_many_arguments)]
    fn next_version(
        &self,
        config: &Config,
        carry: &Version,
        instance: u32,
        txg: u64,
        watermark: u64,
        tree: Vec<(u64, u64)>,
        floor: u64,
        table: Vec<Row>,
        record_base: &Records,
        true_base: &Records,
        issued: Vec<(u64, u64)>,
    ) -> Version {
        let created = tree.iter().any(|&(_, birth)| birth == txg);
        let records =
            mutated_records(config.mutation, record_base, instance, txg, watermark, &tree, carry.watermark, created);
        let mut true_records =
            extend_records(true_base, instance, txg, watermark, &tree, carry.watermark, created, watermark);
        for row in &table {
            let key = (row.instance, row.selected_txg, row.m_true);
            if !true_records.rows_ever.contains(&key) {
                true_records.rows_ever.push(key);
            }
            let key_ring = (row.instance, row.selected_txg, row.m_ring_true);
            if !true_records.rows_ever_ring.contains(&key_ring) {
                true_records.rows_ever_ring.push(key_ring);
            }
        }
        Version {
            instance, txg, watermark, tree, floor, table, records, true_records, issued,
            tombs: carry.tombs.clone(),
            lineage_tree: carry.lineage_tree.clone(),
        }
    }

    fn create_in(&self, config: &Config, base: &Version, txg: u64, instance: u32, table: Vec<Row>) -> (Version, (u64, u64)) {
        let inode = base.watermark;
        let mut tree: Vec<(u64, u64)> = base.tree.iter().copied().filter(|(number, _)| *number != inode).collect();
        tree.push((inode, txg));
        tree.sort_unstable();
        let watermark = match config.arm {
            Arm::PublishDoesNotAddTheNewInodes => inode,
            _ => inode + 1,
        };
        let issued = union_sorted(&base.issued, &[(inode, txg)]);
        let version = self.next_version(
            config, base, instance, txg, watermark, tree, base.floor, table, &base.records, &base.true_records, issued,
        );
        (version, (inode, txg))
    }

    /// 恢复或切换：按 `selected` 写行（[max(所选根的实例, 1), 新实例)），发新实例的第一次发布（`txg`）。
    /// `memory_watermark` 是切换时内存里的水位（C1m 那一读法写进 M），恢复时就是所选那一版的水位。
    fn rows_for_a_new_instance(&self, config: &Config, selected: &Version, new_instance: u32, memory_watermark: u64) -> Vec<Row> {
        let mut table: Vec<Row> = selected.table.clone();
        let ring_maximum = self
            .readable_versions()
            .iter()
            .filter(|version| version.instance == selected.instance && version.txg <= selected.txg)
            .map(|version| version.watermark)
            .max()
            .unwrap_or(0)
            .max(selected.watermark);
        for instance in selected.instance.max(1)..new_instance {
            table.retain(|row| row.instance != instance);
            let row = if instance == selected.instance {
                Row {
                    instance,
                    selected_txg: selected.txg,
                    m_disk: if config.mutation == Mutation::HistoryNotWritten { 0 } else { selected.watermark },
                    m_memory: memory_watermark,
                    m_true: selected.watermark,
                    m_ring: if config.mutation == Mutation::HistoryNotWritten { 0 } else { ring_maximum },
                    m_ring_true: ring_maximum,
                }
            } else {
                Row { instance, selected_txg: 0, m_disk: 0, m_memory: 0, m_true: 0, m_ring: 0, m_ring_true: 0 }
            };
            table.push(row);
        }
        table.sort_by_key(|row| row.instance);
        table
    }

    fn sweep(&mut self, config: &Config) -> bool {
        let table = self.current.table.clone();
        if u64::try_from(table.len()).expect("小") <= config.sweep_threshold {
            return false;
        }
        let in_ring: Vec<u32> = self.ring.iter().flatten().map(|slot| slot.version.instance).collect();
        let deleted: Vec<Row> = table.iter().filter(|row| !in_ring.contains(&row.instance)).cloned().collect();
        if deleted.is_empty() {
            return false;
        }
        let kept: Vec<Row> = table.iter().filter(|row| in_ring.contains(&row.instance)).cloned().collect();
        let base = self.current.clone();
        let txg = self.newest_txg_written + 1;
        let version = self.next_version(
            config, &base, base.instance, txg, base.watermark, base.tree.clone(), base.floor, kept, &base.records,
            &base.true_records, base.issued.clone(),
        );
        self.fixed_erased_by_the_sweep.retain(|entry| {
            !deleted.iter().any(|row| row.instance == entry.instance && entry.txg > row.selected_txg)
        });
        self.publish(version);
        true
    }

    fn recover_onto(
        &mut self,
        config: &Config,
        selected: Version,
        memory_watermark: u64,
        txg: u64,
        record_base_override: Option<Records>,
    ) -> Vec<Row> {
        let new_instance = self.newest_instance + 1;
        self.newest_instance = new_instance;
        let table = self.rows_for_a_new_instance(config, &selected, new_instance, memory_watermark);
        let recovered_watermark = match config.arm {
            Arm::RecoveryLowersTheWatermark => selected.watermark.saturating_sub(1).max(2),
            _ => selected.watermark,
        };
        let record_base = record_base_override.unwrap_or_else(|| selected.records.clone());
        let version = self.next_version(
            config, &selected, new_instance, txg, recovered_watermark, selected.tree.clone(), selected.floor, table.clone(),
            &record_base, &selected.true_records, selected.issued.clone(),
        );
        self.publish(version);
        table
    }

    fn candidate_set(&self, current: &Version) -> Vec<Version> {
        let mut out: Vec<Version> = self
            .readable_versions()
            .into_iter()
            .filter(|version| version.txg != current.txg || version.instance != current.instance)
            .filter(|version| !abandoned_by(&current.table, version.instance, version.txg))
            .filter(|version| version.txg >= current.floor)
            .cloned()
            .collect();
        out.sort_by_key(|version| std::cmp::Reverse((version.txg, version.instance)));
        out
    }
}

impl State {
    /// 做一步；前提不满足交回 None（这条历史在这一步截掉，不往下扫）。
    fn apply(&mut self, config: &Config, op: Op) -> Option<StepResult> {
        let current = self.current.clone();
        let next_txg = self.newest_txg_written + 1;
        match op {
            Op::Create => {
                self.append_fixed(FixedEntry { instance: current.instance, txg: next_txg, inode: current.watermark, birth: next_txg });
                let (version, issued) = self.create_in(config, &current, next_txg, current.instance, current.table.clone());
                self.publish(version);
                Some(StepResult { issued_now: vec![issued], ..StepResult::default() })
            }
            Op::Delete => {
                let victim = current.tree.iter().copied().filter(|(number, _)| *number >= 2).max()?;
                let tree: Vec<(u64, u64)> = current.tree.iter().copied().filter(|pair| *pair != victim).collect();
                let mut version = self.next_version(
                    config, &current, current.instance, next_txg, current.watermark, tree, current.floor, current.table.clone(),
                    &current.records, &current.true_records, current.issued.clone(),
                );
                version.tombs.push((victim.0, victim.1, next_txg));
                self.publish(version);
                Some(StepResult::default())
            }
            Op::Rollback(_) | Op::RollbackBlind => {
                let (k, blind) = match op {
                    Op::Rollback(k) => (k, false),
                    _ => (1, true),
                };
                let candidates = self.candidate_set(&current);
                let target = candidates.get(usize::from(k) - 1)?.clone();
                // 算水位那一遍读根环：blind 时 cur 自己的根槽瞬时读不出（判候选两遍读得出）。
                let scan: Vec<&Version> = self
                    .readable_versions()
                    .into_iter()
                    .filter(|version| !(blind && version.txg == current.txg && version.instance == current.instance))
                    .collect();
                let ring_all = scan.iter().map(|version| version.watermark).max();
                let ring_on_the_timeline = scan
                    .iter()
                    .filter(|version| !abandoned_by(&current.table, version.instance, version.txg))
                    .map(|version| version.watermark)
                    .max();
                let ring_highest = ring_all.unwrap_or(0);
                let watermark = match config.arm {
                    Arm::RollbackTakesTheTargetsWatermark => target.watermark,
                    Arm::Product | Arm::PublishDoesNotAddTheNewInodes | Arm::RecoveryLowersTheWatermark => current.watermark.max(ring_highest),
                    Arm::ProductWithoutAbandonedRoots => current.watermark.max(ring_on_the_timeline.unwrap_or(0)),
                    Arm::RollbackRingOnly => ring_all.unwrap_or(current.watermark),
                };
                let jump_from_abandoned_only = ring_highest > current.watermark && ring_on_the_timeline.unwrap_or(0) <= current.watermark;
                let jump_from_a_root_on_the_timeline = ring_on_the_timeline.unwrap_or(0) > current.watermark;
                let rollback_back_to_a_former_cur = current.true_records.lineage6.iter().any(|lin| lin.t_cur == target.txg);
                let (record_base, watermark_before) = match config.mutation {
                    Mutation::HistoryFromTheTarget => (target.records.clone(), target.watermark),
                    Mutation::LineagePairFromTheTarget => (current.records.clone(), target.watermark),
                    Mutation::LineagePairZero => (current.records.clone(), 0),
                    _ => (current.records.clone(), current.watermark),
                };
                let issued = union_sorted(&current.issued, &target.tree);
                let mut version = self.next_version(
                    config, &current, current.instance, next_txg, watermark, target.tree.clone(), current.floor,
                    current.table.clone(), &record_base, &current.true_records, issued,
                );
                if config.mutation == Mutation::RollbackRowNotWritten && watermark != current.watermark {
                    version.records.rows_step.pop();
                }
                // 变异 D「台阶行记成发布之前的水位」只改建 inode 的发布留的行；回退跳升那一行照写真值（与 m3-w3-r2 攻方腿同一个口径）。
                if config.mutation == Mutation::RowsRecordTheWatermarkBefore && watermark != current.watermark {
                    if let Some(last) = version.records.rows_step.last_mut() {
                        *last = (next_txg, watermark);
                    }
                }
                if config.mutation != Mutation::HistoryNotWritten {
                    version.records.lineage_max = current.records.lineage_max.max(watermark_before);
                    version.records.lin.push((next_txg, watermark_before));
                    // 谱系六元组：H_cur / count 按变异写（正确写法取 cur 的；A 与 H 取 R_old 的；Z 写 0）。
                    let cur_count = u64::try_from(current.records.log.len()).expect("小");
                    let target_count = u64::try_from(target.records.log.len()).expect("小");
                    let lin = match config.mutation {
                        Mutation::HistoryFromTheTarget | Mutation::LineagePairFromTheTarget => Lin {
                            t_rb: next_txg, t_old: target.txg, t_cur: current.txg, h_cur: target.watermark, count: target_count, h_new: watermark,
                        },
                        Mutation::LineagePairZero => Lin { t_rb: next_txg, t_old: target.txg, t_cur: 0, h_cur: 0, count: 0, h_new: watermark },
                        _ => Lin {
                            t_rb: next_txg, t_old: target.txg, t_cur: current.txg, h_cur: current.watermark, count: cur_count, h_new: watermark,
                        },
                    };
                    version.records.lineage6.push(lin);
                }
                version.lineage_tree.push((next_txg, target.txg));
                self.mounted_since_rollback = false;
                version.true_records.lin.push((next_txg, current.watermark));
                version.true_records.lineage_max = current.true_records.lineage_max.max(current.watermark);
                version.true_records.lineage6.push(Lin {
                    t_rb: next_txg,
                    t_old: target.txg,
                    t_cur: current.txg,
                    h_cur: current.watermark,
                    count: u64::try_from(current.true_records.log.len()).expect("小"),
                    h_new: watermark,
                });
                let entry = match config.mutation {
                    Mutation::HistoryNotWritten => (0, 0),
                    Mutation::HistoryFromTheTarget => (current.txg, target.watermark),
                    _ => (current.txg, current.watermark),
                };
                version.true_records.lineage_entries.push(entry);
                self.publish(version);
                if jump_from_abandoned_only {
                    self.jumped_from_abandoned = true;
                }
                if rollback_back_to_a_former_cur {
                    self.sweep_switch_or_aba = true;
                }
                Some(StepResult { jump_from_abandoned_only, jump_from_a_root_on_the_timeline, rollback_back_to_a_former_cur, ..StepResult::default() })
            }
            Op::RaiseFloor => {
                if current.floor >= current.txg {
                    return None;
                }
                let version = self.next_version(
                    config, &current, current.instance, next_txg, current.watermark, current.tree.clone(), current.txg,
                    current.table.clone(), &current.records, &current.true_records, current.issued.clone(),
                );
                self.publish(version);
                Some(StepResult::default())
            }
            Op::Empty => {
                let version = self.next_version(
                    config, &current, current.instance, next_txg, current.watermark, current.tree.clone(), current.floor,
                    current.table.clone(), &current.records, &current.true_records, current.issued.clone(),
                );
                self.publish(version);
                Some(StepResult::default())
            }
            Op::Mount => {
                self.mounted_since_rollback = true;
                let selected = self.newest_readable().clone();
                self.recover_onto(config, selected.clone(), selected.watermark, next_txg, None);
                let swept = self.sweep(config);
                self.sweep_switch_or_aba |= swept;
                Some(StepResult { swept, ..StepResult::default() })
            }
            Op::CrashHidingTheNewestRoot => {
                // 只有最新那条根、且系统配置没见证它（崩在它的根槽 FUA 之后、系统配置轮换之前）才藏得住：C554 乙罩不到的那一形。
                let newest = self.newest_readable().clone();
                if newest.txg != current.txg || newest.instance != current.instance {
                    return None;
                }
                self.mounted_since_rollback = true;
                let slot = usize::try_from(newest.txg).expect("小") % self.ring.len();
                self.ring[slot].as_mut().expect("最新那条根在环里").readable = false;
                if self.readable_versions().is_empty() {
                    return None;
                }
                let selected = self.newest_readable().clone();
                let override_base = (config.mutation == Mutation::MountHistoryFromTheAbandonedRoot).then(|| newest.records.clone());
                self.recover_onto(config, selected.clone(), selected.watermark, next_txg, override_base);
                if let Some(slot) = self.ring[slot].as_mut() {
                    slot.readable = true;
                }
                let swept = self.sweep(config);
                self.sweep_switch_or_aba |= swept;
                Some(StepResult { swept, ..StepResult::default() })
            }
            Op::CrashInTheMiddleOfACreate => {
                // 发号区那一条已经持久、根与记录都没持久；恢复的新实例从环里最大 + 1 起，正好就是那次没成的 txg。
                self.mounted_since_rollback = true;
                self.append_fixed(FixedEntry { instance: current.instance, txg: next_txg, inode: current.watermark, birth: next_txg });
                let selected = self.newest_readable().clone();
                self.recover_onto(config, selected.clone(), selected.watermark, next_txg, None);
                let swept = self.sweep(config);
                self.sweep_switch_or_aba |= swept;
                Some(StepResult { swept, ..StepResult::default() })
            }
            Op::SwitchRedoingTheCreate | Op::SwitchFailingTheCreate => {
                // 挂着时一次建 inode 的发布写失败（发号区那一条已持久）→ 实例切换：重新读盘择根（就是 cur）、写行、
                // 重发在飞的 checkpoint（同一个 txg，新实例）；redo 把那次建重做，error 向调用者报错、不建。
                self.mounted_since_rollback = true;
                self.append_fixed(FixedEntry { instance: current.instance, txg: next_txg, inode: current.watermark, birth: next_txg });
                let memory_watermark = match config.arm {
                    Arm::PublishDoesNotAddTheNewInodes => current.watermark,
                    _ => current.watermark + 1,
                };
                self.sweep_switch_or_aba = true;
                let selected = self.newest_readable().clone();
                let new_instance = self.newest_instance + 1;
                self.newest_instance = new_instance;
                let table = self.rows_for_a_new_instance(config, &selected, new_instance, memory_watermark);
                if op == Op::SwitchRedoingTheCreate {
                    self.append_fixed(FixedEntry { instance: new_instance, txg: next_txg, inode: selected.watermark, birth: next_txg });
                    let (version, issued) = self.create_in(config, &selected, next_txg, new_instance, table);
                    self.publish(version);
                    Some(StepResult { issued_now: vec![issued], switched: true, ..StepResult::default() })
                } else {
                    let version = self.next_version(
                        config, &selected, new_instance, next_txg, match config.arm { Arm::RecoveryLowersTheWatermark => selected.watermark.saturating_sub(1).max(2), _ => selected.watermark }, selected.tree.clone(), selected.floor, table,
                        &selected.records, &selected.true_records, selected.issued.clone(),
                    );
                    self.publish(version);
                    Some(StepResult { switched: true, ..StepResult::default() })
                }
            }
        }
    }
}

const N: usize = 40;
const CANDIDATES: [&str; N] = [
    "I-9.29", "I-9.6", "I-9.29t", "I-9.6t", "C1", "C1b", "C1bk", "C1k", "C2a", "C2b", "C2c", "C2d", "C2i", "C3r", "C4", "C4g",
    "C2ax", "C3rx", "C2ay", "C2dg", "C3rg", "C2ag", "C2ig", "C1bg", "C4gk",
    // 第三轮追加（m3-w3-r2 攻方腿的候选，名字去掉了冒号与括号）：
    "I930-C2a-create", "I930-C2a-all", "C2aL-pair-eq", "C2aL", "C2aL-pair-cover", "C2aL-create-pair-cover", "C2aL-pair-TcurHnew",
    "C3rs", "C3r-I930-3", "C3rL-count-cover", "C3rL-count-eq", "C3rL",
    // 第三轮追加（Y1 取法一的「容许这一跳」那一句：③ 的覆盖判放过「环里有被抛弃的根带这个水位」）：
    "I930-C2a-create-tol", "I930-C2a-all-tol", "C2aL-tol",
];
const IDX_I929: usize = 0;
const IDX_I929T: usize = 2;
const IDX_I96T: usize = 3;
const IDX_C1: usize = 4;
const IDX_C1K: usize = 7;

/// 死亡 txg `death` 在不在这一版的谱系父链上：从这一版的 txg 起往回走，有谱系记录 (T_rb, T_old) 的那个 txg 的父是 T_old，其余是 txg − 1（I-9.18 的定义）。
fn on_the_lineage(version: &Version, death: u64) -> bool {
    let mut txg = version.txg;
    loop {
        if txg == death {
            return true;
        }
        if txg < death || txg <= 1 {
            return false;
        }
        txg = match version.lineage_tree.iter().find(|(rollback, _)| *rollback == txg) {
            Some((_, old)) => *old,
            None => txg - 1,
        };
    }
}

fn pairs_with_tombstones_on_the_lineage(version: &Version) -> Vec<(u64, u64)> {
    let mut out = version.tree.clone();
    out.extend(version.tombs.iter().filter(|(_, _, death)| on_the_lineage(version, *death)).map(|(n, b, _)| (*n, *b)));
    out
}

fn two_births(pairs: &[(u64, u64)]) -> bool {
    pairs.windows(2).any(|pair| pair[0].0 == pair[1].0 && pair[0].1 != pair[1].1)
}

fn fixed_entry_published(entry: &FixedEntry, newest: &Version) -> bool {
    if entry.instance == newest.instance {
        return entry.txg <= newest.txg;
    }
    match newest.table.iter().find(|row| row.instance == entry.instance) {
        Some(row) => entry.txg <= row.selected_txg,
        None => entry.instance < newest.instance,
    }
}

fn fixed_verdict(entries: &[FixedEntry], newest: &Version, filter: &dyn Fn(&FixedEntry) -> bool) -> bool {
    let mut pairs: Vec<(u64, u64)> = entries.iter().filter(|entry| filter(entry)).map(|entry| (entry.inode, entry.birth)).collect();
    pairs.sort_unstable();
    pairs.dedup();
    two_births(&pairs) || pairs.iter().any(|(inode, _)| *inode >= newest.watermark)
}

fn floor_red(rows: &[(u64, u64)], watermark: u64) -> bool {
    rows.iter().any(|&(_, h)| h > watermark)
}

fn cut_red(rows: &[(u64, u64)], tree: &[(u64, u64)]) -> bool {
    tree.iter().any(|&(number, birth)| rows.iter().any(|&(generation, h)| h > number && birth > generation))
}

/// ③ 历史盖住最新根：水位大于初值而没有一行 H ≥ 水位且 g ≤ 最新根的 txg。`tolerated_ring_watermark` 非 0 时是「容许跳升」的读法：
/// 环里有按实例表判被抛弃的根带着不小于这个水位的水位就放过。
fn cover_red(rows: &[(u64, u64)], watermark: u64, txg: u64, tolerated_ring_watermark: u64) -> bool {
    watermark > 2 && !rows.iter().any(|&(generation, h)| h >= watermark && generation <= txg) && tolerated_ring_watermark < watermark
}

fn max_h_upto(rows: &[(u64, u64)], t: u64) -> u64 {
    rows.iter().filter(|(generation, _)| *generation <= t).map(|(_, h)| *h).max().unwrap_or(0)
}

fn log_upto(log: &[(u64, u64)], t: u64) -> u64 {
    u64::try_from(log.iter().filter(|(_, birth)| *birth <= t).count()).expect("小")
}

/// 单镜像上各候选的判定（true = 红）。只读：最新那条根、它的实例表与它下面的记录、环里读得出的根。
fn verdicts(state: &State) -> [bool; N] {
    let newest = state.newest_readable();
    let records = &newest.records;
    let watermark = newest.watermark;
    // 现在的 I-9.29 / I-9.6（Opus 口径，只看树）：候选集（按最新那条根的实例表滤、txg ≥ F）各版的 inode 树合起来。
    let mut candidate_pairs: Vec<(u64, u64)> = newest.tree.clone();
    let mut candidate_pairs_t: Vec<(u64, u64)> = pairs_with_tombstones_on_the_lineage(newest);
    for version in state.readable_versions() {
        if !abandoned_by(&newest.table, version.instance, version.txg) && version.txg >= newest.floor {
            candidate_pairs.extend(version.tree.iter().copied());
            candidate_pairs_t.extend(pairs_with_tombstones_on_the_lineage(version));
        }
    }
    candidate_pairs.sort_unstable();
    candidate_pairs.dedup();
    candidate_pairs_t.sort_unstable();
    candidate_pairs_t.dedup();
    let i9_29 = two_births(&candidate_pairs);
    let i9_6 = candidate_pairs.iter().any(|(inode, _)| *inode >= watermark);
    let i9_29t = two_births(&candidate_pairs_t);
    let i9_6t = i9_6 || newest.tombs.iter().any(|(inode, _, _)| *inode >= watermark);
    let c1 = newest.table.iter().any(|row| row.m_disk > watermark);
    let readable = state.readable_versions();
    // C1b：A 半句水位 < M_ring；B 半句 (号, 出生代) 满足号 < M_ring 且出生代 > T（Sonnet 正推的判法 A、B）。
    let c1b_row = |instance_row: &Row, m: u64| -> bool {
        instance_row.selected_txg > 0
            && (m > watermark || newest.tree.iter().any(|&(inode, birth)| inode < m && birth > instance_row.selected_txg))
    };
    let c1b = newest.table.iter().any(|row| c1b_row(row, row.m_ring));
    let c1bk = c1b
        || newest.true_records.rows_ever_ring.iter().any(|&(_, t_pub, m)| {
            t_pub > 0 && (m > watermark || newest.tree.iter().any(|&(inode, birth)| inode < m && birth > t_pub))
        });
    let rows_max = records.generation_rows.iter().map(|(_, value)| *value).max().unwrap_or(0);
    let generation_reissue = newest.tree.iter().any(|&(inode, birth)| {
        records.generation_rows.iter().filter(|(generation, _)| *generation < birth).map(|(_, value)| *value).max().is_some_and(|before| inode < before)
    });
    let c2a = rows_max > watermark || generation_reissue;
    let c2b = records.per_instance_last.iter().any(|(_, value)| *value > watermark);
    let c2c = records.peak > watermark;
    let (peak, peak_generation) = records.peak_pair;
    let c2d = peak > watermark || newest.tree.iter().any(|&(inode, birth)| inode < peak && birth > peak_generation);
    let c2i = records.peak_by_instance.iter().any(|&(_, high, generation)| {
        high > watermark || newest.tree.iter().any(|&(inode, birth)| inode < high && birth > generation)
    });
    let c3r = two_births(&records.log) || records.log.iter().any(|(inode, _)| *inode >= watermark);
    let c4 = records.lineage_max > watermark;
    let c4g = records.lin.iter().any(|&(t_rb, w)| {
        w > watermark || newest.tree.iter().any(|&(inode, birth)| inode < w && birth > t_rb)
    });
    // C1k：清扫删掉的行也留着（看「行回收丢了 M」那一格用）。
    let c1k = newest.true_records.rows_ever.iter().any(|(_, _, m)| *m > watermark);
    // C2ax / C3rx：再拿环里读得出、按实例表不被抛弃的每一条根交叉核。
    let on_the_timeline: Vec<&Version> =
        readable.iter().copied().filter(|version| !abandoned_by(&newest.table, version.instance, version.txg)).collect();
    let c2as = c2a || !records.generation_rows.contains(&(newest.txg, watermark));
    let c3rs = c3r || newest.tree.iter().any(|pair| !records.log.contains(pair));
    let c2ax = c2as || on_the_timeline.iter().any(|version| !records.generation_rows.contains(&(version.txg, version.watermark)));
    let c3rx = c3rs || on_the_timeline.iter().any(|version| version.tree.iter().any(|pair| !records.log.contains(pair)));
    let c2ay = c2ax || newest.true_records.lineage_entries.iter().any(|entry| !records.generation_rows.contains(entry));
    // 组合：两处各自存的历史，红一处即红（OR）。
    let c2dg = c2d || c4g;
    let c3rg = c3r || c4g;
    let c2ag = c2a || c4g;
    let c2ig = c2i || c4g;
    let c1bg = c1b || c4g;
    let c4gk = c4g || c1bk;
    // ---- 第三轮追加：I-9.30 草案的几种读法与 L 形（照 m3-w3-r2 攻方腿的 verdicts 写，字段名换成这个模型的）
    let (txg, tree) = (newest.txg, &newest.tree);
    let rows_all = &records.rows_step;
    let rows_create = &records.rows_create;
    let abandoned_ring_watermark = readable
        .iter()
        .filter(|version| abandoned_by(&newest.table, version.instance, version.txg))
        .map(|version| version.watermark)
        .max()
        .unwrap_or(0);
    let i930 = |rows: &[(u64, u64)], tolerated: u64| floor_red(rows, watermark) || cut_red(rows, tree) || cover_red(rows, watermark, txg, tolerated);
    let i930_create = i930(rows_create, 0);
    let i930_all = i930(rows_all, 0);
    let pair_cover_all = records.lineage6.iter().any(|lin| max_h_upto(rows_all, lin.t_cur) < lin.h_cur);
    let pair_eq_all = records.lineage6.iter().any(|lin| lin.h_cur == 0 || max_h_upto(rows_all, lin.t_cur) != lin.h_cur);
    let pair_cover_create = records.lineage6.iter().any(|lin| max_h_upto(rows_create, lin.t_cur) < lin.h_cur);
    let pair_hnew = records.lineage6.iter().any(|lin| max_h_upto(rows_all, lin.t_cur) < lin.h_new);
    let log_max = records.log.iter().map(|(number, _)| *number).max().unwrap_or(0);
    let c3r_cover = watermark > 2 && log_max + 1 < watermark;
    let count_cover = records.lineage6.iter().any(|lin| log_upto(&records.log, lin.t_cur) < lin.count);
    let count_eq = records.lineage6.iter().any(|lin| lin.count == 0 || log_upto(&records.log, lin.t_cur) != lin.count);
    let c2al = i930_all || pair_eq_all;
    let c3rl = c3rs || count_eq;
    let i930_create_tol = i930(rows_create, abandoned_ring_watermark);
    let i930_all_tol = i930(rows_all, abandoned_ring_watermark);
    let c2al_tol = i930_all_tol || pair_eq_all;
    [
        i9_29, i9_6, i9_29t, i9_6t, c1, c1b, c1bk, c1k, c2a, c2b, c2c, c2d, c2i, c3r, c4, c4g, c2ax, c3rx, c2ay, c2dg, c3rg, c2ag, c2ig,
        c1bg, c4gk, i930_create, i930_all, pair_eq_all, c2al, pair_cover_all, pair_cover_create, pair_hnew, c3rs, c3r_cover, count_cover,
        count_eq, c3rl, i930_create_tol, i930_all_tol, c2al_tol,
    ]
}

/// 每个候选的记录这一格被变异改过没有（与正确写法比）。
fn mutated_components(state: &State) -> [bool; N] {
    let newest = state.newest_readable();
    let (records, truth) = (&newest.records, &newest.true_records);
    let rows = newest.table.iter().any(|row| row.m_disk != row.m_true);
    let rows_ring = newest.table.iter().any(|row| row.m_ring != row.m_ring_true);
    let generation = records.generation_rows != truth.generation_rows;
    let per_instance = records.per_instance_last != truth.per_instance_last;
    let peak = records.peak != truth.peak;
    let pair = records.peak_pair != truth.peak_pair;
    let by_instance = records.peak_by_instance != truth.peak_by_instance;
    let log = records.log != truth.log;
    let lineage = records.lineage_max != truth.lineage_max;
    let lin = records.lin != truth.lin;
    let rows_step = records.rows_step != truth.rows_step;
    let rows_create = records.rows_create != truth.rows_create;
    // 谱系六元组按每个候选读的字段比：C2aL 读 (T_rb, T_cur, H_cur)，(T_cur, H_new) 那一读法读 H_new，C3rL 读 (T_rb, T_cur, count)。
    let project = |f: &dyn Fn(&Lin) -> (u64, u64, u64)| -> bool {
        records.lineage6.iter().map(f).collect::<Vec<_>>() != truth.lineage6.iter().map(f).collect::<Vec<_>>()
    };
    let ln_h = project(&|lin: &Lin| (lin.t_rb, lin.t_cur, lin.h_cur));
    let ln_new = project(&|lin: &Lin| (lin.t_rb, lin.t_cur, lin.h_new));
    let ln_count = project(&|lin: &Lin| (lin.t_rb, lin.t_cur, lin.count));
    [
        false, false, false, false, rows, rows_ring, rows_ring, rows, generation, per_instance, peak, pair, by_instance, log, lineage,
        lin, generation, log, generation, pair || lin, log || lin, generation || lin, by_instance || lin, rows_ring || lin,
        lin || rows_ring,
        // 第三轮追加，与 CANDIDATES 同序
        rows_create, rows_step, rows_step || ln_h, rows_step || ln_h, rows_step || ln_h, rows_create || ln_h, rows_step || ln_new,
        log, log, log || ln_count, log || ln_count, log || ln_count, rows_create, rows_step, rows_step || ln_h,
    ]
}

/// 真相：这条时间线上有没有同号两个出生代、最新那一版的树里有没有被重发的活对象、水位退没退回去。
struct Truth {
    reissue_on_the_timeline: bool,
    reissued_object_alive: bool,
    watermark_regressed: bool,
}

fn truth_of(state: &State) -> Truth {
    let newest = state.newest_readable();
    let issued = &newest.issued;
    // 活着的是后发的那一个（同号更早的出生代在这条时间线上发过）：被重发的对象还在最新那一版里。
    let reissued_object_alive = newest.tree.iter().any(|&(inode, birth)| issued.iter().any(|&(other, other_birth)| other == inode && other_birth < birth));
    Truth {
        reissue_on_the_timeline: two_births(issued),
        reissued_object_alive,
        watermark_regressed: issued.iter().any(|(inode, _)| *inode >= newest.watermark),
    }
}

const KINDS: [&str; 12] = [
    "false_positive",
    "event_missed",
    "event_missed_where_I-9.29_holds",
    "residual_missed",
    "mutated_history_undetected",
    "event_missed_only_because_the_sweep_deleted_the_row",
    "event_missed_where_existing_checks_silent",
    "residual_missed_where_existing_checks_silent",
    "event_missed_where_existing_silent_and_mounted_since_the_last_rollback",
    "event_missed_where_existing_silent_and_not_mounted_since_the_last_rollback",
    "false_positive_after_a_jump_from_abandoned_roots",
    "false_positive_after_a_sweep_or_switch_or_aba",
];

#[derive(Default, Clone)]
struct Tally {
    images: u64,
    legal_images: u64,
    events: u64,
    events_where_i9_29_holds: u64,
    events_where_existing_silent: u64,
    residual_images: u64,
    residual_where_existing_silent: u64,
    /// 合法的跨被抛弃时间线重发：这一步发的号，在环里一条按最新那条根的实例表判被抛弃的根上有另一个出生代（Q3b 那一形，K2 要它出现）。
    reissues_across_abandoned_roots: u64,
    mutated_images: Vec<u64>,
    counts: Vec<[u64; 12]>,
    events_existing_silent_mounted: u64,
    events_existing_silent_not_mounted: u64,
    examples: Vec<Vec<Option<Vec<u8>>>>,
    /// 第三轮：Y1 与 C143 那一格要的总数（单独一行 W3EXTRA 打印，不动 W3MODEL 那一行的字段）。
    jumps_from_abandoned_only: u64,
    jumps_from_a_root_on_the_timeline: u64,
    sweeps: u64,
    switches: u64,
    aba_rollbacks: u64,
    timeline_reissue_images: u64,
    watermark_regressed_images: u64,
    images_after_a_jump_from_abandoned_roots: u64,
}

fn better(candidate: &[u8], kept: &Option<Vec<u8>>) -> bool {
    match kept {
        None => true,
        Some(kept) => (candidate.len(), candidate) < (kept.len(), kept.as_slice()),
    }
}

impl Tally {
    fn new() -> Tally {
        Tally {
            examples: vec![vec![None; KINDS.len()]; CANDIDATES.len()],
            mutated_images: vec![0; N],
            counts: vec![[0; 12]; N],
            ..Tally::default()
        }
    }

    fn hit(&mut self, candidate: usize, kind: usize, sequence: &[u8]) {
        self.counts[candidate][kind] += 1;
        if better(sequence, &self.examples[candidate][kind]) {
            self.examples[candidate][kind] = Some(sequence.to_vec());
        }
    }

    fn judge(&mut self, config: &Config, state: &State, step: &StepResult, sequence: &[u8]) {
        self.images += 1;
        self.jumps_from_abandoned_only += u64::from(step.jump_from_abandoned_only);
        self.jumps_from_a_root_on_the_timeline += u64::from(step.jump_from_a_root_on_the_timeline);
        self.sweeps += u64::from(step.swept);
        self.switches += u64::from(step.switched);
        self.aba_rollbacks += u64::from(step.rollback_back_to_a_former_cur);
        self.images_after_a_jump_from_abandoned_roots += u64::from(state.jumped_from_abandoned);
        let red = verdicts(state);
        let truth = truth_of(state);
        let mutated = mutated_components(state);
        let newest = state.newest_readable();
        let event = step.issued_now.iter().any(|&(inode, birth)| newest.issued.iter().any(|&(other, b)| other == inode && b != birth));
        let legal = config.mutation == Mutation::None && !truth.reissue_on_the_timeline && !truth.watermark_regressed;
        if legal {
            self.legal_images += 1;
        }
        self.timeline_reissue_images += u64::from(truth.reissue_on_the_timeline);
        self.watermark_regressed_images += u64::from(truth.watermark_regressed);
        let existing_silent = !red[IDX_I929T] && !red[IDX_I96T];
        if event {
            self.events += 1;
            if !red[IDX_I929] {
                self.events_where_i9_29_holds += 1;
            }
            if existing_silent {
                self.events_where_existing_silent += 1;
                if state.mounted_since_rollback {
                    self.events_existing_silent_mounted += 1;
                } else {
                    self.events_existing_silent_not_mounted += 1;
                }
            }
        }
        let across_abandoned = step.issued_now.iter().any(|&(inode, birth)| {
            state.readable_versions().iter().any(|version| {
                abandoned_by(&newest.table, version.instance, version.txg)
                    && version.tree.iter().any(|&(other, b)| other == inode && b != birth)
            })
        });
        if across_abandoned && !event {
            self.reissues_across_abandoned_roots += 1;
        }
        let residual = !event && truth.reissued_object_alive;
        if residual {
            self.residual_images += 1;
            if existing_silent {
                self.residual_where_existing_silent += 1;
            }
        }
        for candidate in 0..CANDIDATES.len() {
            if legal && red[candidate] {
                self.hit(candidate, 0, sequence);
                if state.jumped_from_abandoned {
                    self.hit(candidate, 10, sequence);
                }
                if state.sweep_switch_or_aba {
                    self.hit(candidate, 11, sequence);
                }
            }
            if event && !red[candidate] {
                self.hit(candidate, 1, sequence);
                if !red[IDX_I929] {
                    self.hit(candidate, 2, sequence);
                }
                if existing_silent {
                    self.hit(candidate, 6, sequence);
                    self.hit(candidate, if state.mounted_since_rollback { 8 } else { 9 }, sequence);
                }
            }
            if residual && !red[candidate] {
                self.hit(candidate, 3, sequence);
                if existing_silent {
                    self.hit(candidate, 7, sequence);
                }
            }
            if event && !red[candidate] && candidate == IDX_C1 && red[IDX_C1K] {
                self.hit(candidate, 5, sequence);
            }
            if mutated[candidate] {
                self.mutated_images[candidate] += 1;
                if !red[candidate] {
                    self.hit(candidate, 4, sequence);
                }
            }
        }
    }
}

fn explore(config: &Config, state: &State, sequence: &mut Vec<u8>, tally: &mut Tally, histories: &mut u64) {
    if sequence.len() == config.longest {
        *histories += 1;
        return;
    }
    for (index, op) in Op::ALL.iter().enumerate() {
        if !op_allowed(*op) {
            continue;
        }
        let mut next = state.clone();
        let Some(step) = next.apply(config, *op) else { continue };
        sequence.push(u8::try_from(index).expect("13 个"));
        tally.judge(config, &next, &step, sequence);
        explore(config, &next, sequence, tally, histories);
        sequence.pop();
    }
}

fn render_version(version: &Version) -> String {
    let tree: Vec<String> = version.tree.iter().map(|(inode, birth)| format!("{inode}@{birth}")).collect();
    let rows: Vec<String> = version
        .table
        .iter()
        .map(|row| format!("{}:{}:M{}:Mm{}", row.instance, row.selected_txg, row.m_disk, row.m_memory))
        .collect();
    let log: Vec<String> = version.records.log.iter().map(|(inode, birth)| format!("{inode}@{birth}")).collect();
    format!(
        "newest={}:{} wm={} F={} tree=[{}] rows=[{}] peak={} peak_pair={}@{} lineage={} lin=[{}] tombs=[{}] c3r_log=[{}]",
        version.instance,
        version.txg,
        version.watermark,
        version.floor,
        tree.join(","),
        rows.join(","),
        version.records.peak,
        version.records.peak_pair.0,
        version.records.peak_pair.1,
        version.records.lineage_max,
        version.records.lin.iter().map(|(t, w)| format!("{t}:{w}")).collect::<Vec<_>>().join(","),
        version.tombs.iter().map(|(n, b, d)| format!("{n}@{b}d{d}")).collect::<Vec<_>>().join(","),
        log.join(","),
    )
}

fn render_trace(config: &Config, sequence: &[u8]) -> Vec<String> {
    let mut state = State::initial(config.ring_slots);
    let mut lines = vec![format!("step=start {}", render_version(state.newest_readable()))];
    for (step, index) in sequence.iter().enumerate() {
        let op = Op::ALL[usize::from(*index)];
        state.apply(config, op).expect("复放的历史每一步都做得成");
        let red = verdicts(&state);
        let red_names: Vec<&str> = CANDIDATES.iter().zip(red).filter(|(_, red)| *red).map(|(name, _)| *name).collect();
        let ring: Vec<String> = state
            .ring
            .iter()
            .flatten()
            .map(|slot| format!("{}:{}{}", slot.version.instance, slot.version.txg, if slot.readable { "" } else { "(hidden)" }))
            .collect();
        let fixed: Vec<String> = state.fixed.iter().map(|entry| format!("{}:{}:{}@{}", entry.instance, entry.txg, entry.inode, entry.birth)).collect();
        let truth = truth_of(&state);
        lines.push(format!(
            "step={step} op={} {} ring=[{}] fixed=[{}] truth_reissue={} truth_regressed={} red=[{}]",
            op.code(),
            render_version(state.newest_readable()),
            ring.join(","),
            fixed.join(","),
            u8::from(truth.reissue_on_the_timeline),
            u8::from(truth.watermark_regressed),
            red_names.join(","),
        ));
    }
    lines
}

fn config_label(config: &Config) -> String {
    let threshold = if config.sweep_threshold == u64::MAX { "never".to_string() } else { config.sweep_threshold.to_string() };
    let alphabet = std::env::var("W3_OPS").map_or_else(|_| "all12".to_string(), |list| list.replace(',', "+"));
    format!(
        "arm={:?} mutation={:?} ring={} sweep_threshold={} longest={} alphabet={}",
        config.arm, config.mutation, config.ring_slots, threshold, config.longest, alphabet
    )
}

fn op_allowed(op: Op) -> bool {
    static ALLOWED: std::sync::OnceLock<Option<Vec<String>>> = std::sync::OnceLock::new();
    let allowed = ALLOWED.get_or_init(|| {
        std::env::var("W3_OPS").ok().map(|list| list.split(',').map(|name| name.trim().to_string()).collect())
    });
    match allowed {
        // 没设 W3_OPS：字母表是原来的 12 个，rollback-blind 不进（保持与前两轮同一个历史空间）。
        None => op != Op::RollbackBlind,
        Some(names) => names.contains(&op.code()),
    }
}

fn run(mut config: Config) {
    if let Ok(value) = std::env::var("W3_LONGEST") {
        config.longest = value.parse().expect("W3_LONGEST 是整数");
    }
    if let Ok(value) = std::env::var("W3_RING") {
        config.ring_slots = value.parse().expect("W3_RING 是整数");
    }
    let mut tally = Tally::new();
    let mut histories = 0_u64;
    let start = State::initial(config.ring_slots);
    explore(&config, &start, &mut Vec::new(), &mut tally, &mut histories);
    let label = config_label(&config);
    println!(
        "W3MODEL {label} full_length_histories={histories} images={} legal_images={} events={} events_where_I-9.29_holds={} events_where_existing_checks_silent={} events_existing_silent_mounted_since_the_last_rollback={} events_existing_silent_not_mounted_since_the_last_rollback={} residual_images={} residual_where_existing_checks_silent={} reissues_across_abandoned_roots={}",
        tally.images, tally.legal_images, tally.events, tally.events_where_i9_29_holds, tally.events_where_existing_silent,
        tally.events_existing_silent_mounted, tally.events_existing_silent_not_mounted, tally.residual_images, tally.residual_where_existing_silent, tally.reissues_across_abandoned_roots
    );
    println!(
        "W3EXTRA {label} rollback_jumps_from_abandoned_roots_only={} rollback_jumps_from_a_root_on_the_timeline={} sweeps={} switches={} rollbacks_back_to_a_former_cur={} timeline_reissue_images={} watermark_regressed_images={} images_after_a_jump_from_abandoned_roots={}",
        tally.jumps_from_abandoned_only, tally.jumps_from_a_root_on_the_timeline, tally.sweeps, tally.switches, tally.aba_rollbacks,
        tally.timeline_reissue_images, tally.watermark_regressed_images, tally.images_after_a_jump_from_abandoned_roots
    );
    for (candidate, name) in CANDIDATES.iter().enumerate() {
        let counts: Vec<String> = KINDS.iter().enumerate().map(|(kind, kind_name)| format!("{kind_name}={}", tally.counts[candidate][kind])).collect();
        println!("W3COUNT {label} candidate={name} {} mutated_images={}", counts.join(" "), tally.mutated_images[candidate]);
    }
    for (candidate, name) in CANDIDATES.iter().enumerate() {
        for (kind, kind_name) in KINDS.iter().enumerate() {
            if let Some(sequence) = &tally.examples[candidate][kind] {
                let ops: Vec<String> = sequence.iter().map(|index| Op::ALL[usize::from(*index)].code()).collect();
                println!("W3EXAMPLE {label} candidate={name} kind={kind_name} ops=[{}]", ops.join(","));
                for line in render_trace(&config, sequence) {
                    println!("W3TRACE {label} candidate={name} kind={kind_name} {line}");
                }
            }
        }
    }
}

fn replay(arguments: &[String]) {
    let arm = match arguments[1].as_str() {
        "P" => Arm::Product,
        "V1" => Arm::RollbackTakesTheTargetsWatermark,
        "V2" => Arm::PublishDoesNotAddTheNewInodes,
        "V3" => Arm::RecoveryLowersTheWatermark,
        "P2" => Arm::ProductWithoutAbandonedRoots,
        "R0" => Arm::RollbackRingOnly,
        other => panic!("不认的臂 {other}"),
    };
    let ring_slots: usize = arguments[2].parse().expect("环长是整数");
    let names: Vec<&str> = arguments[3].split(',').collect();
    let config = Config { arm, mutation: Mutation::None, ring_slots, sweep_threshold: u64::MAX, longest: names.len() };
    let sequence: Vec<u8> = names
        .iter()
        .map(|name| {
            let index = Op::ALL.iter().position(|op| op.code() == *name).unwrap_or_else(|| panic!("不认的操作 {name}"));
            u8::try_from(index).expect("13 个")
        })
        .collect();
    println!("W3REPLAY {} ops={}", config_label(&config), arguments[3]);
    for line in render_trace(&config, &sequence) {
        // 只留判红名单，不重复打印整条根环（环长 24 时太长）：把 ring=[…] fixed=[…] 两段截掉。
        let cut = line.find(" ring=[").map_or(line.len(), |at| at);
        let red_at = line.find(" red=[");
        let mut shown = line[..cut].to_string();
        if let Some(at) = red_at {
            shown.push_str(&line[at..]);
        }
        println!("W3REPLAYSTEP {shown}");
    }
}

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    if arguments.first().map(String::as_str) == Some("replay") {
        replay(&arguments);
        return;
    }
    let selection = arguments.first().map_or("all", String::as_str).to_string();
    let only_index: Option<usize> = arguments.get(1).map(|index| index.parse().expect("第二个参数是组内序号"));
    let mut configs: Vec<(&str, Config)> = Vec::new();
    let base = Config { arm: Arm::Product, mutation: Mutation::None, ring_slots: 4, sweep_threshold: u64::MAX, longest: 7 };
    for (arm, arm_name) in [(Arm::Product, "P"), (Arm::RollbackTakesTheTargetsWatermark, "V1"), (Arm::PublishDoesNotAddTheNewInodes, "V2")] {
        for (threshold, threshold_name) in [(u64::MAX, "never"), (0, "any")] {
            configs.push(("main", Config { arm, sweep_threshold: threshold, ..base }));
            let _ = (arm_name, threshold_name);
        }
    }
    for arm in [Arm::Product, Arm::RollbackTakesTheTargetsWatermark] {
        for mutation in [Mutation::HistoryFromTheTarget, Mutation::HistoryCopiesThisVersion, Mutation::HistoryNotWritten] {
            configs.push(("mutation", Config { arm, mutation, sweep_threshold: 0, longest: 6, ..base }));
        }
        configs.push(("ring6", Config { arm, ring_slots: 6, sweep_threshold: 0, longest: 6, ..base }));
    }
    // 辩方组（defense）：V1 在环 4 / 6 / 8 / 12 上（看 C1b 挂载之后的漏随环长怎么变）、V3、P 在环 8 上（误报）。
    for ring_slots in [4_usize, 6, 8, 12] {
        configs.push(("defense", Config { arm: Arm::RollbackTakesTheTargetsWatermark, ring_slots, sweep_threshold: u64::MAX, ..base }));
    }
    configs.push(("defense", Config { arm: Arm::RecoveryLowersTheWatermark, ring_slots: 4, sweep_threshold: u64::MAX, ..base }));
    configs.push(("defense", Config { arm: Arm::Product, ring_slots: 8, sweep_threshold: 0, ..base }));
    // 第三轮：m3-w3-r2 攻方腿的六种新变异里没有并进老三种的五种（H、Z、R、D、Mh；A + H 就是老的 A），P 与 V1 两臂，口径同 mutation 组。
    for arm in [Arm::Product, Arm::RollbackTakesTheTargetsWatermark] {
        for mutation in [
            Mutation::LineagePairFromTheTarget,
            Mutation::LineagePairZero,
            Mutation::RollbackRowNotWritten,
            Mutation::RowsRecordTheWatermarkBefore,
            Mutation::MountHistoryFromTheAbandonedRoot,
        ] {
            configs.push(("mutation2", Config { arm, mutation, sweep_threshold: 0, longest: 6, ..base }));
        }
    }
    // 第三轮 Y1：取法二（回退水位只取没被抛弃的根）在 P 的同一组历史上（对照 main 组的 0、1 与 defense 组的 5）。
    configs.push(("k2", Config { arm: Arm::ProductWithoutAbandonedRoots, sweep_threshold: u64::MAX, ..base }));
    configs.push(("k2", Config { arm: Arm::ProductWithoutAbandonedRoots, sweep_threshold: 0, ..base }));
    configs.push(("k2", Config { arm: Arm::ProductWithoutAbandonedRoots, ring_slots: 8, sweep_threshold: 0, ..base }));
    // 第三轮 C143 那一格：P、取法二、控制臂（只取环）三臂在含 rollback-blind 的字母表上（W3_OPS 点名，W3_LONGEST 定深）。
    for arm in [Arm::Product, Arm::ProductWithoutAbandonedRoots, Arm::RollbackRingOnly] {
        configs.push(("blind", Config { arm, sweep_threshold: u64::MAX, ..base }));
    }
    let mut index_in_group: BTreeMap<&str, usize> = BTreeMap::new();
    for (group, config) in configs {
        let index = index_in_group.entry(group).or_insert(0);
        let this_index = *index;
        *index += 1;
        if (selection == "all" || selection == group) && only_index.is_none_or(|wanted| wanted == this_index) {
            run(config);
        }
    }
}

//! E16：journal 的角色是 WAL 还是意图日志。
//!
//! 起因是 [decisions.md] D23 未定项 1，它自陈「**必须先定角色，才能定大小**」：
//!   - **WAL**：每个操作提交时同步写 journal ⇒ 操作即持久，fsync 只是把尾块刷下去；
//!   - **意图日志**：只记批的游标，持久性靠 checkpoint ⇒ **fsync = 提前触发一次完整 checkpoint**。
//! D16 原文两处指向意图日志，而「所有结构、所有操作先进同一个 journal」读起来像 WAL。
//!
//! ## 口径：本实验量的是**设备写块数**，不是时间
//!
//! 没有设备、没有虚机、没有事务层。所以：
//!   - 「fsync 代价」的单位是**块**，不是微秒。p99 是块数分布的 p99。
//!   - 真实的 fsync 延迟还含 FUA 往返与队列排队，本实验**够不着**，
//!     那一半要等 E12 harness 与事务层（E16 注册时已写明这条前置）。
//! **拿本实验的数去谈延迟是误用。**
//!
//! ## 三条臂，不是两条 —— 这是本实验最先得到的东西
//!
//! D23 已定「**分配先于 journal**：journal 记录是**已经写到盘上的东西的发布指令**」。
//! ⇒ **「WAL 的 fsync = 追加一条记录」在本工程不可实现**：记录里点名的块必须已经在盘上。
//! 于是 WAL 这一侧裂成两条本质不同的臂，它们的差别不在快慢，在**重放要不要分配器**：
//!
//! | 臂 | fsync 时写什么 | 重放要不要分配器 | 与 D23 已定项 |
//! |---|---|---|---|
//! | `intent` 意图日志 | 脏叶 + 全部祖先 + 根槽 + 一条游标记录 | 不要 | 相容 |
//! | `wal_full` WAL / 记录点名到根下 | 脏叶 + 全部祖先 + 一条记录（**不发根**） | 不要 | 相容 |
//! | `wal_leaf` WAL / 记录只点名叶 | **只写脏叶** + 一条记录；祖先延到 checkpoint | **要**（祖先在重放时才生成，要分配） | **冲突** |
//!
//! ⚠️ **`wal_leaf` 是唯一真正买到东西的那条**，而它踩的正是 D23 用来否掉「逻辑意图日志」
//! 的那条判据。本实验的任务因此变成：**量出它买到多少**，好判断那条判据值不值这个价。
//!
//! ## 三条臂共用同一份操作流
//!
//! **这是本实验的非平凡性保证**：臂若吃到不同的操作流，比的就是不同的工作量。
//! 操作流生成一次，三臂共享同一个切片，且各自算出的流指纹必须相同——不同即整轮作废。
//!
//! ## 阳性对照（对**每一条**臂都跑）
//!
//! 1. **`fsync_every=0` 时三条臂的非 journal 写块数必须逐格相等。** 不 fsync 就没有
//!    「fsync 怎么落地」这个差异，checkpoint 的调度对三臂完全相同 ⇒ COW 写必须一模一样。
//!    不等 ⇒ 模型串了，**整轮作废**。
//!    ⚠️ **这条对每一条臂都跑，不是只跑第一条**——E18 的教训：阳性对照只跑一条臂时，
//!    产出结论的往往正是没跑的那条。
//! 2. **判别力**：整个网格上臂间总写块数差异全部 < 5% ⇒ 说明 checkpoint 间隔把差异吃掉了，
//!    **整轮作废**（E16 注册时写下的失败条款，原样执行）。
//!
//! ⚠️ 这两条都是**对照**，不是结论：它们判的是「这套测量有没有判别力」，
//! 不判「哪种角色更好」。后者是本实验要产出的东西，测出来什么就是什么。

use e7_index_bench::Emitter;
use std::collections::HashSet;

// ── 几何：一棵完全 F 叉树，叶就是用户数据块 ──────────────────────────────
const FANOUT: u64 = 128; // 与 D19 那张扇出表同量级（40 B 指针 / 16 KiB 单元 ≈ 291，取保守值）
const HEIGHT: u32 = 4; // 内部层数（不含叶）；叶数 = FANOUT^HEIGHT
const BLOCK: u64 = 4096;

/// 根环的槽数（D22：发根 = 覆写根环的一个槽，不是追加）。
/// 每次 checkpoint 覆写一个槽 ⇒ 每次 checkpoint +1 块。
const ROOT_SLOT_BLOCKS: u64 = 1;

/// 一条记录里一项的字节数（落点 + 代号等）。
const ENTRY_BYTES: u64 = 24;
/// 校验和字节数。D4 已定校验和内联进**父指针**。
const CHECKSUM_BYTES: u64 = 32;

/// WAL 臂一条记录的字节数：固定头 + 每项。
/// D23 已定「记录承载指针层的目标态」= 哪些子树根换到了哪个块 + 新根。
///
/// ⚠️ **`parent_on_disk == false` 时每一项必须自带校验和。**
/// D4 把校验和内联进父指针，而 fsync 记录点名的东西**其父此刻还没写到盘上**——
/// `wal_full` 点名子树根，它们的父是根，根没发；`wal_leaf` 点名叶，父是内部节点，也没写。
/// ⇒ **记录必须临时充当父**，否则重放时那些块没有任何完整性凭据。
/// checkpoint 记录不需要：根在同一次 checkpoint 里发出，校验和住进根里了。
/// **这是 WAL 逼进格式里的一项额外开销，此前没有任何地方写过。**
fn wal_record_bytes(named: usize, parent_on_disk: bool) -> u64 {
    let per = if parent_on_disk { ENTRY_BYTES } else { ENTRY_BYTES + CHECKSUM_BYTES };
    48 + per * named as u64
}

/// 意图日志臂一条记录的字节数：**只记批的游标**，与批多大无关。
const INTENT_RECORD_BYTES: u64 = 48;

// ── 操作流 ──────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Workload {
    /// 顺序写：叶号单调推进，一次操作弄脏 8 个相邻叶
    Seq,
    /// 随机小写：叶号全域随机，一次操作弄脏 1 个叶
    Rand,
    /// 元数据密集：叶号在一小段热区内随机，一次操作弄脏 2 个叶
    MetaHeavy,
    /// 多流：`STREAMS` 条流轮流来，每条在**自己的子树**里顺序追加，一次操作弄脏 1 个叶。
    /// 它模拟的是「组提交完全可用、但各流之间几乎不共享脊柱」——
    /// rand 太散（全域随机）、seq 太集中（单条脊柱），两者都不是这个形态。
    MultiStream,
}

/// 多流负载的默认流数。各流的子树在**根的子节点**那一层就分开，脊柱只在根共享。
/// ⚠️ 主网格恒用这个默认值 —— 扫描模式改它，主网格不许受影响。
const STREAMS: u64 = 16;

/// 本次运行的流数。扫描模式下由命令行覆盖。
static STREAMS_NOW: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(STREAMS);
fn streams() -> u64 { STREAMS_NOW.load(std::sync::atomic::Ordering::Relaxed) }

impl Workload {
    fn name(self) -> &'static str {
        match self {
            // 没有 `_ =>` —— 新增一种负载不补这里就编译不过
            Workload::Seq => "seq",
            Workload::Rand => "rand",
            Workload::MetaHeavy => "metaheavy",
            Workload::MultiStream => "multistream",
        }
    }
}

/// 一个操作弄脏的叶集合。
type Op = Vec<u64>;

fn gen_ops(n: usize, wl: Workload, seed: u64) -> Vec<Op> {
    let mut s = seed | 1;
    let mut r = move || {
        s ^= s >> 12;
        s ^= s << 25;
        s ^= s >> 27;
        s.wrapping_mul(0x2545_F491_4F6C_DD1D)
    };
    let leaves = FANOUT.pow(HEIGHT);
    let hot = leaves / 1024; // 元数据热区
    let mut cursor = 0u64;
    (0..n)
        .map(|_| match wl {
            Workload::Seq => {
                let base = cursor;
                cursor = (cursor + 8) % leaves;
                (0..8).map(|i| (base + i) % leaves).collect()
            }
            Workload::Rand => vec![r() % leaves],
            Workload::MetaHeavy => vec![r() % hot, r() % hot],
            Workload::MultiStream => {
                // 第 i 个操作归第 (i % STREAMS) 条流；每条流在自己的子树里顺序推进。
                let s = streams();
                let sid = cursor % s;
                let per_stream = leaves / s;
                let off = (cursor / s) % per_stream;
                cursor += 1;
                vec![sid * per_stream + off]
            }
        })
        .collect()
}

/// 操作流指纹。两臂各自算一遍，不同即整轮作废。
fn stream_fingerprint(ops: &[Op]) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for op in ops {
        for &l in op {
            h ^= l;
            h = h.wrapping_mul(0x100_0000_01b3);
        }
        h ^= 0xffff_ffff_ffff_ffff;
        h = h.wrapping_mul(0x100_0000_01b3);
    }
    h
}

// ── checkpoint 的代价：COW 集 = 脏叶 ∪ 全部祖先 ──────────────────────────

/// 脏叶集合对应的 COW 写块数：脏叶自己 + 去重后的全部祖先（含根）。
/// **祖先必须去重**——同一个内部节点被多个脏叶共享时只 COW 一次，
/// 不去重会把随机负载的代价高估到接近 `脏叶数 × 树高`。
fn cow_blocks(dirty: &HashSet<u64>) -> u64 {
    let mut anc: HashSet<(u32, u64)> = HashSet::new();
    for &leaf in dirty {
        let mut idx = leaf;
        for lvl in 0..HEIGHT {
            idx /= FANOUT;
            anc.insert((lvl, idx));
        }
    }
    dirty.len() as u64 + anc.len() as u64
}

/// 一条记录点名的**子树根**条数 = 脏叶在「根的子节点」那一层的去重祖先数。
///
/// ⚠️ **记录按子树根计费，不按脏叶计费。** D23 已定记录承载「哪些**子树根**换到了哪个块 + 新根」；
/// 祖先已经写下去之后，记录只需点名根的哪几个子槽位变了，条数因此被那一层的节点数封顶
/// （FANOUT=128 ⇒ 一条记录最多 128 项 ≈ 3.1 KB，**永远不越块**）。
/// 本实验第一版按脏叶计费，把 `wal_full` 的环峰值虚报到 192048 字节（47 块）——
/// 而那正是本实验要产出的那个数。是反推腿抓出来的，不是跑出来的。
fn named_subtree_roots(dirty: &HashSet<u64>) -> usize {
    debug_assert!(HEIGHT >= 2, "树高不足 2 时「根的子节点」这一层不存在");
    let mut top: HashSet<u64> = HashSet::new();
    for &leaf in dirty {
        top.insert(leaf / FANOUT.pow(HEIGHT - 1));
    }
    top.len()
}

/// 一个操作贡献的用户写次数。写放大的分母走这里，**算错会静默污染所有格**。
fn user_write_count(op: &Op) -> u64 {
    op.len() as u64
}

// ── 三条臂 ──────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Arm {
    /// 意图日志：只记批的游标，fsync ⇒ 提前触发一次**完整 checkpoint**（含发根）。
    Intent,
    /// WAL，记录点名到根下一层：fsync 要把脏叶与**全部祖先**都落盘，但不发根。
    /// 重放 = 按记录把子树根接上去，**不需要分配**。
    WalFull,
    /// WAL，记录只点名叶：fsync 只落脏叶。祖先延到 checkpoint。
    /// ⚠️ 重放时祖先还不存在，**必须现场分配**——与 D23「重放不要分配器」冲突。
    WalLeaf,
}

impl Arm {
    fn name(self) -> &'static str {
        match self {
            // 没有 `_ =>` —— 新增一条臂不补这里就编译不过
            Arm::Intent => "intent",
            Arm::WalFull => "wal_full",
            Arm::WalLeaf => "wal_leaf",
        }
    }
    /// 该臂的 fsync 会不会把祖先一起写下去。
    fn fsync_writes_ancestors(self) -> bool {
        match self {
            Arm::Intent => true,
            Arm::WalFull => true,
            Arm::WalLeaf => false,
        }
    }
    /// 该臂的 fsync 会不会发根。
    fn fsync_publishes_root(self) -> bool {
        match self {
            Arm::Intent => true,
            Arm::WalFull => false,
            Arm::WalLeaf => false,
        }
    }
    /// 记录大小：意图日志是定长游标；两条 WAL 臂按**点名条数**增长。
    /// 点名的是什么由调用点决定——祖先已写就点名子树根，祖先没写就只能点名叶。
    fn record_bytes(self, named: usize, parent_on_disk: bool) -> u64 {
        match self {
            Arm::Intent => INTENT_RECORD_BYTES,
            Arm::WalFull => wal_record_bytes(named, parent_on_disk),
            Arm::WalLeaf => wal_record_bytes(named, parent_on_disk),
        }
    }
    /// **重放要不要分配器。** 这不是性能，是与 D23 已定项相容与否。
    fn replay_needs_allocator(self) -> bool {
        match self {
            Arm::Intent => false,
            Arm::WalFull => false,
            Arm::WalLeaf => true,
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
struct Out {
    journal_blocks: u64,
    ckpt_blocks: u64,
    root_blocks: u64,
    journal_bytes: u64,
    ring_peak_bytes: u64,
    checkpoints: u64,
    fsyncs: u64,
    user_writes: u64,
    /// 每次 fsync 当场付出的块数，用来出分布
    fsync_cost: Vec<u64>,
    fingerprint: u64,
}

impl Out {
    fn total_blocks(&self) -> u64 {
        self.journal_blocks + self.ckpt_blocks + self.root_blocks
    }
    fn write_amp(&self) -> f64 {
        if self.user_writes == 0 {
            return f64::NAN;
        }
        self.total_blocks() as f64 / self.user_writes as f64
    }
}

fn pct(v: &mut Vec<u64>, p: f64) -> u64 {
    if v.is_empty() {
        return 0;
    }
    v.sort_unstable();
    let i = (((v.len() - 1) as f64) * p).round() as usize;
    v[i]
}

/// 一条臂跑完整条流。
///
/// `inject_skip_journal` 是故障注入开关：置真则不写任何 journal 记录。
/// 它存在的唯一目的是**证明守恒检查会红**。
fn run(ops: &[Op], arm: Arm, fsync_every: usize, ckpt_interval: usize, inject_skip_journal: bool) -> Out {
    let mut o = Out { fingerprint: stream_fingerprint(ops), ..Default::default() };
    // 自上次 checkpoint 以来被改过的叶
    let mut dirty: HashSet<u64> = HashSet::new();
    // 其中已经在某次 fsync 里落过盘、且此后没再被改的那些（只有 WalLeaf 用得上）
    let mut persisted: HashSet<u64> = HashSet::new();
    // journal 环占用：本 checkpoint 窗口内累计写进 journal 的字节
    let mut ring = 0u64;

    let append = |o: &mut Out, ring: &mut u64, bytes: u64| {
        if inject_skip_journal {
            return 0;
        }
        o.journal_bytes += bytes;
        *ring += bytes;
        o.ring_peak_bytes = o.ring_peak_bytes.max(*ring);
        // 一条记录至少占一个块：本工程的记录是**同步发布指令**，不与后续记录合并成一块。
        // ⚠️ 这是保守取法，真实实现可以合并；合并只会让两条 WAL 臂更便宜，不改变结论方向。
        let blks = bytes.div_ceil(BLOCK).max(1);
        o.journal_blocks += blks;
        blks
    };

    for (i, op) in ops.iter().enumerate() {
        o.user_writes += user_write_count(op);
        for &l in op {
            dirty.insert(l);
            persisted.remove(&l); // 又被改了，之前落的那份不算数
        }

        if fsync_every > 0 && (i + 1) % fsync_every == 0 {
            o.fsyncs += 1;
            let mut cost = 0u64;
            if arm.fsync_writes_ancestors() {
                cost += cow_blocks(&dirty);
                cost += append(&mut o, &mut ring, arm.record_bytes(named_subtree_roots(&dirty), false));
                o.ckpt_blocks += cow_blocks(&dirty);
                if arm.fsync_publishes_root() {
                    o.root_blocks += ROOT_SLOT_BLOCKS;
                    cost += ROOT_SLOT_BLOCKS;
                    o.checkpoints += 1;
                    // ⚠️ **只有真发了根才许清环。** 不发根时持久态是「根 ⊕ journal 前缀」，
                    // 丢掉前缀等于丢掉已经应答过的 fsync。第一版在这里无条件清零，
                    // 把 wal_full 的环峰值虚报低了 4–5 个数量级。
                    ring = 0;
                }
                dirty.clear();
                persisted.clear();
            } else {
                // 只落还没落过的脏叶
                let fresh: Vec<u64> = dirty.difference(&persisted).copied().collect();
                o.ckpt_blocks += fresh.len() as u64;
                cost += fresh.len() as u64;
                cost += append(&mut o, &mut ring, arm.record_bytes(fresh.len(), false));
                persisted = dirty.clone();
            }
            o.fsync_cost.push(cost);
        }

        // ⚠️ 触发条件里 `ring > 0` 那一半不能少：`wal_full` 的 fsync 会清空 dirty
        // 却留下未发布的 journal 前缀，只看 dirty 的话它**一次根都发不出来**——
        // 第一版正是如此，20 万次操作里 `root=0 ckpts=0`，而守恒检查看不见。
        if (i + 1) % ckpt_interval == 0 && (!dirty.is_empty() || ring > 0) {
            // checkpoint：把还没落盘的叶补齐 + 全部祖先 + 一条记录 + 发根
            let fresh = dirty.difference(&persisted).count() as u64;
            let anc = cow_blocks(&dirty) - dirty.len() as u64;
            o.ckpt_blocks += fresh + anc;
            append(&mut o, &mut ring, arm.record_bytes(named_subtree_roots(&dirty), true));
            o.root_blocks += ROOT_SLOT_BLOCKS;
            o.checkpoints += 1;
            dirty.clear();
            persisted.clear();
            ring = 0;
        }
    }
    o
}

// ── 守恒检查（独立于臂的计数器重算一遍）──────────────────────────────────

/// 守恒：三项之和对得上、fsync 次数对得上、根槽次数与 checkpoint 次数对得上、
/// journal 不为空。返回 Err(说明) 表示这一格作废。
fn conserve(o: &Out, n_ops: usize, fsync_every: usize, ckpt_interval: usize) -> Result<(), String> {
    if o.total_blocks() != o.journal_blocks + o.ckpt_blocks + o.root_blocks {
        return Err("三项之和对不上总数".into());
    }
    let expect_fsync = if fsync_every == 0 { 0 } else { n_ops / fsync_every };
    if o.fsyncs != expect_fsync as u64 {
        return Err(format!("fsync 次数 {} ≠ 预期 {}", o.fsyncs, expect_fsync));
    }
    if o.root_blocks != o.checkpoints * ROOT_SLOT_BLOCKS {
        return Err("根槽写次数与 checkpoint 次数对不上".into());
    }
    if o.journal_bytes == 0 && n_ops > 0 {
        return Err("一个字节的 journal 都没写".into());
    }
    if o.fsync_cost.len() as u64 != o.fsyncs {
        return Err("fsync 代价样本数与 fsync 次数对不上".into());
    }
    // **每一条臂都必须至少每 ckpt_interval 次操作发一次根。**
    // 不发根 ⇒ journal 前缀永远截断不了、环无界增长，而且这个故障
    // 对「三臂互比」和「三项之和」两种检查都是隐形的——
    // 第一版 wal_full 在 20 万次操作里一次根都没发，没有任何东西报警。
    if n_ops >= ckpt_interval && o.checkpoints < (n_ops / ckpt_interval) as u64 {
        return Err(format!(
            "发根次数 {} 少于「每 {ckpt_interval} 次操作至少一次」要求的 {}",
            o.checkpoints, n_ops / ckpt_interval
        ));
    }
    // 不 fsync 时 checkpoint 完全由区间决定，次数是**绝对可预测**的。
    // ⚠️ 只让三条臂互相比是不够的：区间算错时三条臂会一起错，比出来仍然相等。
    // 这条是变异测试（把区间乘 2）逼出来的——那次破坏没有任何检查看见。
    if fsync_every == 0 && o.checkpoints != (n_ops / ckpt_interval) as u64 {
        return Err(format!("checkpoint 次数 {} ≠ 区间推出的 {}", o.checkpoints, n_ops / ckpt_interval));
    }
    Ok(())
}

/// 扫描模式：`e16-journal sweep` —— 流数 × 批大小的二维扫描。
///
/// 它要证伪的是一条**先算后测**的预测：甲每批 ≈ `批 + min(流数,批)×树高 + 2`，
/// 乙 ≈ `批 + 1` ⇒ **比值随批增长到「批 ≈ 流数」时见顶，再往下掉**。
/// 若扫出来是这条非单调曲线，那 multistream 那条结论是结构性的；
/// 若效应只在某一档流数上冒出来，它就是建模伪影。
fn sweep() {
    // ⚠️ checkpoint 间隔必须**有界且远小于 n**：乙臂的 fsync 不清空脏集合，
    // 间隔比 n 还大的话脏集合无界增长，`cow_blocks` 每次 fsync 都扫全集 ⇒ O(n²·树高)。
    // 第一版正是如此，跑不完。间隔取最大批的 10 倍，既不让 checkpoint 主导，也把脏集合封住。
    let n = 20_000usize;
    let ckpt = 2_000usize;
    let mut em = Emitter::new();
    let mut out = String::new();
    println!("E7RESULT name=sweep_config ops={n} ckpt_interval={ckpt} height={HEIGHT} fanout={FANOUT}");
    for s in [2u64, 4, 8, 16, 32, 64, 128] {
        STREAMS_NOW.store(s, std::sync::atomic::Ordering::Relaxed);
        for b in [1usize, 2, 5, 10, 20, 50, 100, 200] {
            let ops = gen_ops(n, Workload::MultiStream, 0xBEEF ^ s ^ (b as u64) << 8);
            // ckpt 间隔取得远大于 n，使区间 checkpoint 不介入，只看 fsync 那一侧
            let a = run(&ops, Arm::Intent, b, ckpt, false);
            let l = run(&ops, Arm::WalLeaf, b, ckpt, false);
            let (ab, lb) = (a.fsync_cost.iter().sum::<u64>() as f64 / a.fsyncs as f64,
                            l.fsync_cost.iter().sum::<u64>() as f64 / l.fsyncs as f64);
            // 先算的预测值
            let pred_a = b as f64 + (s.min(b as u64) as f64) * HEIGHT as f64 + 2.0;
            let pred_l = b as f64 + 1.0;
            out.push_str(&em.emit_raw(&format!(
                "name=sweep streams={s} batch={b} a_blocks={ab:.2} l_blocks={lb:.2}                  ratio={:.3} pred_ratio={:.3}", ab / lb, pred_a / pred_l)));
            out.push('\n');
        }
    }
    out.push_str(&em.finish());
    print!("{out}");
}

fn main() {
    if std::env::args().nth(1).as_deref() == Some("sweep") { sweep(); return; }
    let n: usize = std::env::args().nth(1).and_then(|x| x.parse().ok()).unwrap_or(200_000);
    let mut em = Emitter::new();
    let mut out = String::new();
    let mut say = |s: String| {
        out.push_str(&s);
        out.push('\n');
    };
    let die = |out: &mut String, em: &mut Emitter, msg: &str| -> ! {
        out.push_str(&em.finish());
        out.push('\n');
        print!("{out}");
        eprintln!("E16: {msg}");
        std::process::exit(4);
    };

    say(em.emit_raw(&format!(
        "name=config ops={n} fanout={FANOUT} height={HEIGHT} leaves={} block={BLOCK} intent_rec={INTENT_RECORD_BYTES}",
        FANOUT.pow(HEIGHT)
    )));

    const ARMS: [Arm; 3] = [Arm::Intent, Arm::WalFull, Arm::WalLeaf];

    // ── 故障注入自检：对**每一条**臂都证明守恒检查会红 ──
    {
        let ops = gen_ops(1000, Workload::Rand, 3);
        let mut all_ok = true;
        for arm in ARMS {
            let good = conserve(&run(&ops, arm, 10, 100, false), 1000, 10, 100).is_ok();
            let caught = conserve(&run(&ops, arm, 10, 100, true), 1000, 10, 100).is_err();
            say(em.emit_raw(&format!(
                "name=faultinject arm={} good_passes={good} injected_is_caught={caught}",
                arm.name()
            )));
            all_ok &= good && caught;
        }
        if !all_ok {
            die(&mut out, &mut em, "守恒检查没有判别力：注入「不写 journal」它没红");
        }
    }

    let workloads = [Workload::Seq, Workload::Rand, Workload::MetaHeavy, Workload::MultiStream];
    let fsyncs = [0usize, 10, 1];
    let intervals = [100usize, 1000];
    let mut max_ratio = 0.0f64;
    let mut zero_fsync_cells = 0u32;

    for wl in workloads {
        for &fe in &fsyncs {
            for &ci in &intervals {
                let ops = gen_ops(n, wl, 0x5eed ^ (wl as u64) << 8 ^ (fe as u64) << 16 ^ ci as u64);
                let rs: Vec<Out> = ARMS.iter().map(|&a| run(&ops, a, fe, ci, false)).collect();

                for (a, r) in ARMS.iter().zip(&rs) {
                    if r.fingerprint != rs[0].fingerprint {
                        die(&mut out, &mut em, "臂吃到的操作流不同 —— 比的是不同的工作量");
                    }
                    if let Err(e) = conserve(r, n, fe, ci) {
                        die(&mut out, &mut em, &format!("{} 臂守恒失败：{e}", a.name()));
                    }
                }

                // 阳性对照 1：不 fsync 时**三条臂**的 COW 写与根槽必须逐格相等
                if fe == 0 {
                    zero_fsync_cells += 1;
                    for (a, r) in ARMS.iter().zip(&rs) {
                        if r.ckpt_blocks != rs[0].ckpt_blocks || r.root_blocks != rs[0].root_blocks {
                            die(&mut out, &mut em, &format!(
                                "阳性对照 1 失败：不 fsync 时 {} 臂的 COW 写与 {} 臂不等（{} vs {}）",
                                a.name(), ARMS[0].name(), r.ckpt_blocks, rs[0].ckpt_blocks));
                        }
                    }
                }

                let tot: Vec<u64> = rs.iter().map(|r| r.total_blocks()).collect();
                let ratio = *tot.iter().max().unwrap() as f64 / *tot.iter().min().unwrap() as f64;
                max_ratio = max_ratio.max(ratio);

                for (a, r) in ARMS.iter().zip(&rs) {
                    let mut c = r.fsync_cost.clone();
                    say(em.emit_raw(&format!(
                        "name=cell wl={} fsync_every={fe} ckpt_interval={ci} arm={} \
                         total={} journal={} ckpt={} root={} ring_peak={} wa={:.4} \
                         fsync_p50={} fsync_p99={} ckpts={} replay_alloc={}",
                        wl.name(), a.name(),
                        r.total_blocks(), r.journal_blocks, r.ckpt_blocks, r.root_blocks,
                        r.ring_peak_bytes, r.write_amp(),
                        pct(&mut c, 0.50), pct(&mut c, 0.99), r.checkpoints,
                        a.replay_needs_allocator()
                    )));
                }
            }
        }
    }

    if zero_fsync_cells == 0 {
        die(&mut out, &mut em, "阳性对照 1 一格都没跑到");
    }
    say(em.emit_raw(&format!("name=discrimination max_ratio={max_ratio:.4} zero_fsync_cells={zero_fsync_cells}")));
    if max_ratio < 1.05 {
        die(&mut out, &mut em, "三条臂在整个网格上差异 < 5%：checkpoint 间隔把差异吃掉了，整轮作废");
    }

    say(em.finish());
    print!("{out}");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 祖先必须去重。8 个相邻叶共享同一条祖先链 ⇒ COW 集远小于「叶数 × 树高」。
    #[test]
    fn ancestors_are_deduplicated() {
        let d: HashSet<u64> = (0..8).collect();
        assert_eq!(cow_blocks(&d), 8 + HEIGHT as u64);
    }

    /// 单个叶的 COW 集 = 它自己 + 树高。
    #[test]
    fn single_leaf_costs_height_plus_one() {
        let d: HashSet<u64> = [12345].into_iter().collect();
        assert_eq!(cow_blocks(&d), 1 + HEIGHT as u64);
    }

    /// 不 fsync 时三条臂的 COW 写必须完全相同 —— 阳性对照 1 的可执行形式，
    /// **对每一条臂都断言**。
    #[test]
    fn without_fsync_all_arms_do_the_same_cow_work() {
        let ops = gen_ops(5_000, Workload::Rand, 9);
        let base = run(&ops, Arm::Intent, 0, 100, false);
        for arm in [Arm::WalFull, Arm::WalLeaf] {
            let r = run(&ops, arm, 0, 100, false);
            assert_eq!(r.ckpt_blocks, base.ckpt_blocks, "{} 臂", arm.name());
            assert_eq!(r.root_blocks, base.root_blocks, "{} 臂", arm.name());
            assert_eq!(r.checkpoints, base.checkpoints, "{} 臂", arm.name());
        }
    }

    /// 意图日志臂每次 fsync 至少要付「一个叶 + 树高 + 一条记录 + 根槽」。
    /// 这条钉住「fsync = 提前触发一次完整 checkpoint」这个语义没被实现丢掉。
    #[test]
    fn intent_fsync_pays_a_full_checkpoint() {
        let r = run(&gen_ops(200, Workload::Rand, 11), Arm::Intent, 1, 1000, false);
        assert_eq!(r.fsync_cost.len(), 200);
        for c in &r.fsync_cost {
            assert!(*c >= 1 + HEIGHT as u64 + 1 + ROOT_SLOT_BLOCKS, "fsync 代价 {c} 太小");
        }
    }

    /// wal_leaf 的 fsync **不许**写祖先：随机负载下每次 fsync 恰好是「一个叶 + 一条记录」。
    /// 它比 intent 便宜的那一截就是本实验要量的东西，写漏了就量不出来。
    #[test]
    fn wal_leaf_fsync_does_not_write_ancestors() {
        let r = run(&gen_ops(200, Workload::Rand, 11), Arm::WalLeaf, 1, 1000, false);
        assert!(r.fsync_cost.iter().all(|&c| c == 2), "wal_leaf 的 fsync 代价不该含祖先：{:?}", &r.fsync_cost[..5]);
    }

    /// wal_full 的 fsync 要写祖先但**不发根**：它与 intent 的差恰好是根槽。
    #[test]
    fn wal_full_differs_from_intent_by_exactly_the_root_slot() {
        let ops = gen_ops(2_000, Workload::Rand, 29);
        let i = run(&ops, Arm::Intent, 1, 100_000, false);
        let w = run(&ops, Arm::WalFull, 1, 100_000, false);
        assert_eq!(i.ckpt_blocks, w.ckpt_blocks);
        assert_eq!(i.total_blocks() - w.total_blocks(), i.root_blocks - w.root_blocks);
        assert!(w.root_blocks < i.root_blocks, "wal_full 不该每次 fsync 都发根");
    }

    /// 重放要不要分配器：这是三条臂里唯一的结构性差异，**不许被后来的改动抹平**。
    #[test]
    fn only_wal_leaf_needs_an_allocator_on_replay() {
        assert!(!Arm::Intent.replay_needs_allocator());
        assert!(!Arm::WalFull.replay_needs_allocator());
        assert!(Arm::WalLeaf.replay_needs_allocator());
    }

    /// 故障注入必须被守恒检查抓到 —— 对每一条臂都验一遍。
    #[test]
    fn conservation_catches_missing_journal_on_every_arm() {
        let ops = gen_ops(1_000, Workload::Rand, 17);
        for arm in [Arm::Intent, Arm::WalFull, Arm::WalLeaf] {
            assert!(conserve(&run(&ops, arm, 10, 100, false), 1000, 10, 100).is_ok(), "{}", arm.name());
            assert!(conserve(&run(&ops, arm, 10, 100, true), 1000, 10, 100).is_err(), "{}", arm.name());
        }
    }

    /// 三条臂必须吃到同一条流。
    #[test]
    fn all_arms_see_the_same_stream() {
        let ops = gen_ops(3_000, Workload::MetaHeavy, 19);
        let f = run(&ops, Arm::Intent, 10, 100, false).fingerprint;
        assert_eq!(run(&ops, Arm::WalFull, 10, 100, false).fingerprint, f);
        assert_eq!(run(&ops, Arm::WalLeaf, 10, 100, false).fingerprint, f);
    }

    /// checkpoint 次数必须钉在**绝对值**上，不能只让三条臂互相比。
    /// 区间算错时三条臂会一起错，比出来仍然相等——这条是变异测试逼出来的。
    #[test]
    fn checkpoint_count_is_pinned_to_the_interval() {
        let ops = gen_ops(5_000, Workload::Rand, 31);
        for arm in [Arm::Intent, Arm::WalFull, Arm::WalLeaf] {
            let r = run(&ops, arm, 0, 100, false);
            assert_eq!(r.checkpoints, 50, "{} 臂", arm.name());
            assert!(conserve(&r, 5_000, 0, 100).is_ok(), "{} 臂", arm.name());
        }
    }

    /// 守恒检查自己也要被验：喂一个 fsync 样本数对不上的结果，它必须报错。
    #[test]
    fn conserve_rejects_mismatched_fsync_sample_count() {
        let ops = gen_ops(1_000, Workload::Rand, 37);
        let mut r = run(&ops, Arm::Intent, 10, 100, false);
        assert!(conserve(&r, 1_000, 10, 100).is_ok());
        r.fsync_cost.pop(); // 少一个样本
        assert!(conserve(&r, 1_000, 10, 100).is_err(), "样本数对不上没被守恒检查抓到");
    }

    /// 记录点名的是**子树根**，不是脏叶。这条把「记录大小被那一层节点数封顶」钉死——
    /// 第一版按脏叶计费，把 wal_full 的环峰值虚报了 47 倍。
    #[test]
    fn record_names_subtree_roots_not_leaves() {
        let d: HashSet<u64> = (0..8_000).collect();
        let named = named_subtree_roots(&d);
        assert!(named <= FANOUT as usize, "点名条数 {named} 超过了根的子节点数 {FANOUT}");
        assert!(
            wal_record_bytes(named, true) <= BLOCK,
            "checkpoint 记录（父在盘上）不该越块：{} 字节",
            wal_record_bytes(named, true)
        );
        // fsync 记录每项多带 32 字节校验和，仍然不该越块 —— 越了就说明扇出取错了
        assert!(
            wal_record_bytes(named, false) <= BLOCK,
            "fsync 记录（父不在盘上）越块了：{} 字节",
            wal_record_bytes(named, false)
        );
    }

    /// 由上一条推出的可观测后果：不 fsync 时，两条 WAL 臂每次 checkpoint 恰好写一个 journal 块。
    /// **这条钉的是绝对值**——只让三条臂互相比，三条一起错时比出来仍然相等。
    #[test]
    fn wal_arms_spend_exactly_one_journal_block_per_checkpoint() {
        let ops = gen_ops(20_000, Workload::Seq, 41);
        for arm in [Arm::WalFull, Arm::WalLeaf] {
            let r = run(&ops, arm, 0, 1000, false);
            assert_eq!(r.checkpoints, 20);
            assert_eq!(r.journal_blocks, r.checkpoints, "{} 臂", arm.name());
        }
    }

    /// **每条臂都必须真的发根。** `wal_full` 的 fsync 会清空 dirty，
    /// 若 checkpoint 的触发条件只看 dirty，它一次根都发不出来——第一版正是如此。
    #[test]
    fn every_arm_actually_publishes_roots() {
        let ops = gen_ops(20_000, Workload::Rand, 43);
        for arm in [Arm::Intent, Arm::WalFull, Arm::WalLeaf] {
            let r = run(&ops, arm, 1, 1000, false);
            assert!(r.checkpoints >= 20, "{} 臂只发了 {} 次根", arm.name(), r.checkpoints);
            assert_eq!(r.root_blocks, r.checkpoints * ROOT_SLOT_BLOCKS, "{} 臂", arm.name());
            assert!(conserve(&r, 20_000, 1, 1000).is_ok(), "{} 臂", arm.name());
        }
    }

    /// **环只能被「发根」截断。** 不发根的 fsync 之后 journal 前缀仍然承重，
    /// 所以 `wal_full` 的环峰值必须随 checkpoint 间隔增长；不增长说明环被偷偷清零了。
    #[test]
    fn ring_is_only_truncated_by_a_root_publish() {
        let ops = gen_ops(20_000, Workload::Rand, 47);
        let short = run(&ops, Arm::WalFull, 1, 100, false);
        let long = run(&ops, Arm::WalFull, 1, 1000, false);
        assert!(
            long.ring_peak_bytes > short.ring_peak_bytes * 5,
            "环峰值没随 checkpoint 间隔增长：ci=100 {} B，ci=1000 {} B",
            short.ring_peak_bytes, long.ring_peak_bytes
        );
    }

    /// 记录**在 `run()` 里也确实按子树根计费**。上一条只验了算条数的那个函数，
    /// 没验调用点用没用它——变异测试把这个盲区直接指了出来。
    #[test]
    fn wal_full_records_are_capped_by_the_subtree_root_count() {
        // 每次 fsync 攒 1000 个操作 × 8 叶 = 8000 个脏叶，远多于 FANOUT 个子树根槽位
        let ops = gen_ops(20_000, Workload::Seq, 53);
        let r = run(&ops, Arm::WalFull, 1000, 1000, false);
        let cap = r.fsyncs * wal_record_bytes(FANOUT as usize, false)
            + r.checkpoints * wal_record_bytes(FANOUT as usize, true);
        assert!(
            r.journal_bytes <= cap,
            "记录没按子树根计费：journal_bytes={} 超过上限 {cap}",
            r.journal_bytes
        );
    }

    /// fsync 记录每项要多带一份校验和（父此刻不在盘上），checkpoint 记录不用。
    /// 这个差价是 WAL 逼进格式的开销，抹掉它等于把 WAL 的代价算少了。
    #[test]
    fn fsync_records_carry_checksums_and_checkpoint_records_do_not() {
        for n in [1usize, 7, 128] {
            assert_eq!(
                wal_record_bytes(n, false) - wal_record_bytes(n, true),
                CHECKSUM_BYTES * n as u64,
                "n={n} 时校验和差价不对"
            );
        }
    }

    /// 多流负载的各流必须**真的不共享脊柱**：一个批次里的脏叶要落在
    /// 尽可能多的「根的子节点」上。否则它只是 seq 的马甲，测不出「组提交在场但脊柱不共享」。
    #[test]
    fn multistream_streams_do_not_share_a_spine() {
        let ops = gen_ops(1_024, Workload::MultiStream, 59);
        // 取前 STREAMS 个操作：按构造它们应当分属 STREAMS 条不同的流
        let head: HashSet<u64> = ops[..streams() as usize].iter().flatten().copied().collect();
        assert_eq!(head.len(), streams() as usize, "前 {} 个操作没有落在同样多条流上", streams());
        assert_eq!(
            named_subtree_roots(&head), streams() as usize,
            "各流没有在「根的子节点」那一层分开——那它们仍共享脊柱，本负载没有判别力"
        );
        // 对照：同样多的 seq 操作只占 1 个子树根
        let seq: HashSet<u64> = gen_ops(STREAMS as usize, Workload::Seq, 59).iter().flatten().copied().collect();
        assert_eq!(named_subtree_roots(&seq), 1, "seq 应当只占一个子树根，否则对照没意义");
    }

    /// 用户写次数与操作流一致 —— 写放大的分母算错会静默污染所有结论。
    #[test]
    fn user_writes_match_the_stream() {
        let ops = gen_ops(1_000, Workload::Seq, 23);
        let expect: u64 = ops.iter().map(|o| o.len() as u64).sum();
        assert_eq!(run(&ops, Arm::Intent, 0, 100, false).user_writes, expect);
    }
}

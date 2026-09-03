//! E97：记账与分配记录的条目编码 —— D5 已定项 5 与 D3 已定项 7 欠的那次测量。
//!
//! ## 被引用条款逐字贴在这里（verify-before-claiming.md「把定义句原样贴进实验注释」）
//!
//! - D5 已定项 1（2026-09-01 用户定案）：「除树 ID 之外只加『设备』一维，
//!   key 写作 `(统计量, 树 ID, 设备, 代)`。」
//! - D5 已定项 2（2026-08-30）：「代取 checkpoint 号，只保留最近 K 代」，
//!   且 `K = 根环槽总数 + 1`（保守口径）。
//! - D5 已定项 3（2026-08-30 用户定案）：「住一棵独立 keyspace 的 btree，
//!   条目是『(统计量, 维度元组, 代) → 完整值』。」依据第 3 条：
//!   「走 write buffer 前端，条目自带 seq」。
//! - D5 已定项 4（2026-09-01 用户定案；2026-09-03 第一次重开）：统计量取**十一个**。
//! - D3 已定项 3（2026-09-01 用户定案）：「**独立 keyspace 的 btree，
//!   key = 落点（设备身份 + 设备内偏移），value = 分配代。**」依据第 2 条逐字：
//!   「`(dev, 偏移)` 与 D19 已定项 1 的位置条目**同一个坐标系**」。
//! - D4 已定项 5：单元占盘恒 **32768** 字节（含头）。
//! - D8 已定项 2（2026-08-30 用户定案）：节点 **16 KiB**，钉成常量。
//! - D18 已定项 2（2026-09-01 用户定案）：「**记账树**的节点带 key 区间；
//!   dirent 树那一格推迟。」——**分配记录树带不带，这条一个字没说。**
//! - D18 已定项 7 ⇒ 索引节点基础头 **≥ 58 字节**（E73 已按 58 / 67 / 76 三档重算）。
//! - **D18 已定项 9（2026-09-02 用户定案，收口 C86）逐字**：「⚠️ **它给分配器加了一条硬要求**：
//!   单元起点按类对齐——**数据单元对齐 32768、索引节点对齐 16384**，否则扫描器步进会错位。」
//!   ⇒ **索引节点是 16 KiB（D8 已定项 2）且按 16384 对齐 ⇒ 半数索引节点不在 32768 边界上。**
//!   E86 实测：64 MiB 区域 998 个节点里 **496 个落在奇数 16 KiB 槽**。
//!   **这条是第一轮反推攻击腿找出来的，第一版 E97 的条款清单里没有它。**
//! - **D5 已定项 2 的骑手条款（由 D16 已定项 6 于 2026-09-02 收口）逐字**：
//!   「**代按发布走，不按 5 秒窗口走**……代段宽度 **≥ 48 位**
//!   （2785 发布/秒下 32 位 17.8 天回绕，E71 的宽度算术要按此重算）」。
//!   发布率 2785/秒来自 E44 本机实测。
//!   ⚠️ **第三轮修正**：第一版按 `T_time` = 5 s 算回绕期，**差约 14000 倍**，
//!   而正确口径就写在被引条款自己的正上方。**「代 ≥ 48 位」是已定条款，不是本实验的发现。**
//! - **D8 write buffer 硬要求 3 逐字**：「**条目必须自带序号（seq），这是格式要求**」。
//!   ⚠️ **第三轮修正**：第一、二版的段清单里根本没有 seq，
//!   而判据 2 只查已列出的段 ⇒ 漏掉一整段是它的盲区。判据 9 就是补这个。
//! - D22 已定项 2：根环 **R = 3** 区域，每区槽数 **S 住超级块、1..16**
//!   ⇒ 槽总数 3S，`K = 3S + 1 ≤ 49`。
//! - D22 已定项 7：根记录里 `checkpoint_txg` **8 字节**。
//! - D16 已定项 5：`T_dirty` = **2 GiB**。
//! - `.claude/rules/fs-design.md`：「不为省空间牺牲自包含」；
//!   且「**比谁省不构成任何判据**」——占容量比只能当代价读数，不许单独用它选一档。
//!
//! ## 判据（E97 正文跑前写死，跑完不许改）
//!
//! 1. **绝对值断言**：扇出恰好等于 `(16384 − 节点头 − 区间字段) / 条目宽`，
//!    区间字段在带 key 区间的树上是 `2 × key 宽`、不带的树上是 0。
//!    **不许只做臂间互比**——三条臂共用一个错公式时互比全部相等。
//! 2. **值域覆盖**：每段宽度 `w` 要满足 `2^(8w) > 该段值域上界`，
//!    **而那个上界必须指得到一条已定条款**；指不到的段报「上界取不到」，不替它定。
//! 3. **key 序单调**：把「代」编码成模 M 的环形计数器时要数得出逆序对；不为 0 判「不可用」。
//! 4. **预算**：一次 checkpoint 的记账写字节 ≤ `T_dirty` 的 1%（同 E71 判据 2）。
//! 5. **单调**：任一段加 1 字节 ⇒ 条目宽加 1、扇出单调不增、树高单调不减。
//! 6. **非法状态可不可表示**：同一个落点坐标，按字节偏移与按单元号各能表达多少个
//!    **非单元对齐**的落点。⚠️ 条件式：条件是「落点粒度 = 单元」，而那个前置仓里没定
//!    （E74 逐字：「D4 定的 32 KiB 是**单元**，不是『落点』的定义」）。
//!    ⚠️ **两条恒真式，第二轮加注**：`misaligned_representable` 在按槽号时**第一行就返回 0**，
//!    `addressable_bytes` 的比值恒等于粒度本身。**它们是编码换算的定义，不是发现**；
//!    能支撑的只有「同宽下槽号编码严格更强」这一句分析论证（machine-first 教条二），
//!    不许记成实测。
//!
//! ## 第二轮补的两条判据（第一轮三方论证之后加，加完才重跑）
//!
//! 7. **对齐可表示性**：一个按粒度 g 编码的落点字段，表达得了 D18 已定项 9 要求的
//!    **两类**对齐（数据单元 32768、索引节点 16384）里的哪几类？表达不了的那一类要**数得出绝对值**。
//!    这条是第一轮反推攻击腿逼出来的：它拿 D18 已定项 9 直接证伪了「按 32 KiB 单元号编码」。
//! 8. **代字段的回绕期**：一个模 M 的代字段，按**发布率**（E44 实测 2785/秒，
//!    D5 已定项 2 的骑手条款用的就是它）每 `M / 2785` 秒回绕一次；
//!    判据 3 的逆序对必须在 **代数 > M** 的格子上测，
//!    只测 32 / 64 代等于给模 256 发了一张免检证（第一轮反推攻击腿指出，第一版正是这样）。
//!
//! ## 第三轮补的两条判据（第二轮三方论证之后加，加完才重跑）
//!
//! 9. **段清单完整性**：每一条「要求某个字段存在」的已定条款，都要在段清单里找得到落点。
//!    **判据 2 只查已列出的段，漏掉一整段它一个字也说不出来**——
//!    第一、二版就是这样漏掉了 D8 write buffer 硬要求 3 的 `seq`。
//! 10. **预算按脏节点算，不按条目字节算**：记账树按 COW 挂在根下（D5 已定项 3 第 4 条）
//!    ⇒ 一条几十字节的插入要脏一个完整的 16 KiB 节点。判据 4 第一、二版按条目字节量，
//!    **系统性低估**；第三轮改成数脏叶。
//!
//! ## 失败条款（跑前写死）
//!
//! - **阳性对照，每条臂都跑**：某一段砍成 0 ⇒ ① 条目宽正好少那么多、② 判据 2 覆盖判定翻红。
//!   任一不成立 ⇒ 该段没进模型，**整轮作废**。
//! - **阳性对照二**：模 M 小于代数 ⇒ 判据 3 的逆序对必须 > 0；M 大于代数时必须恰好 0。
//! - 节点头仓里仍未定 ⇒ 按 58 / 67 / 76 三档各算一次；三档不同向就写「结论依赖节点头」。
//! - **反向接受条款**：若最省那档在判据 2 或 3 判红、而最宽那档在判据 4 过界，
//!   结论是「没有可行点，16 KiB 或 K 要重开」，**如实写，不许挑中间档硬凑**。
//!
//! ## 它答不了的
//!
//! 纯算术几何模型：没有 btree 实现、没有 write buffer、没有分配器、没有 I/O，文件操作 0 处。
//! **不答挂钟。** 不答「哪几个统计量带树维」——D5 已定项 4 的表没有那一列，
//! 按 E71 的窄读与宽读各算一次，**不挑一个**。
//! **确定性模型**：同一个二进制跑 N 遍必然逐字节相同 ⇒ N 轮说明的是没有隐藏状态，
//! 不是统计上稳定；证据强度来自判据 1 的绝对值断言与变异测试。

use e7_index_bench::Emitter;

/// D8 已定项 2：节点 16 KiB。**格式常量**。
const NODE_BYTES: u64 = 16384;
/// D4 已定项 5：单元占盘恒 32 KiB。**格式常量**。
const DATA_UNIT_BYTES: u64 = 32768;
/// D16 已定项 5：`T_dirty` = 2 GiB。
const T_DIRTY: u64 = 2 * 1024 * 1024 * 1024;
/// E44 本机实测：**2785 发布/秒**。D5 已定项 2 的骑手条款按它算代段宽度下界。
/// ⚠️ **不是 `T_time` = 5 s**——代按发布走，fsync 触发的也是发布（D16 已定项 6）。
const PUBLISH_PER_SEC: u64 = 2785;
/// D5 已定项 2 骑手条款：代段宽度 **≥ 48 位**。已定条款，不是本实验的发现。
const GEN_BITS_FLOOR: u32 = 48;
/// D8 已定项 2 + D18 已定项 9：索引节点 16 KiB，按 **16384** 对齐。
const NODE_ALIGN: u64 = 16384;
/// 判据 4 的预算线：`T_dirty` 的 1%。
const BUDGET: u64 = T_DIRTY / 100;
/// D5 已定项 4：统计量十一个。
const STATS: u64 = 11;
/// D22 已定项 2：根环区域数 R = 3，每区槽数上限 16 ⇒ K = 3S + 1 ≤ 49。
const RING_REGIONS: u64 = 3;
const SLOTS_MAX: u64 = 16;
const K_MAX: u64 = RING_REGIONS * SLOTS_MAX + 1;
/// D22 已定项 7：`checkpoint_txg` 8 字节 ⇒ 「代」的值域要 8 字节才装得下。
const CKPT_TXG_BYTES: u64 = 8;
/// E73 按 D18 已定项 7 重算过的三档基础节点头下界。
const NODE_HEADERS: [u64; 3] = [58, 67, 76];
/// E73 用的子指针宽度（它的参数）与 D22 已定项 7 的树表单元指针实宽。
const CHILD_PTR_E73: u64 = 32;
const CHILD_PTR_SETTLED: u64 = 53;
/// 记账 value：完整值取 8 字节（统计量是字节计数，256 TiB 也只用到 48 位）。
const ACCT_VALUE_BYTES: u64 = 8;

/// 一段字段：宽度、值域上界（指得到已定条款才有）、上界的出处。
#[derive(Debug, Clone, Copy)]
struct Seg {
    name: &'static str,
    bytes: u64,
    /// `None` = 上界指不到任何已定条款。判据 2 对它只报「取不到」，不判红也不判绿。
    domain_upper: Option<u64>,
}

/// 判据 2：`2^(8w) > 上界`。用 u128 算，避免 8 字节段自己溢出成 0。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Coverage {
    Covers,
    Overflows,
    UpperUnknown,
}

fn capacity(bytes: u64) -> u128 {
    if bytes == 0 {
        return 1; // 0 字节只表达得了一个值（那个段不存在）
    }
    if bytes >= 16 {
        return u128::MAX;
    }
    1u128 << (8 * bytes as u32)
}

fn coverage(s: &Seg) -> Coverage {
    match s.domain_upper {
        None => Coverage::UpperUnknown,
        Some(u) => {
            if capacity(s.bytes) > u as u128 {
                Coverage::Covers
            } else {
                Coverage::Overflows
            }
        }
    }
}

fn key_bytes(segs: &[Seg]) -> u64 {
    segs.iter().map(|s| s.bytes).sum()
}

/// 扇出 = `(节点大小 − 节点头 − 区间字段) / 条目宽`。判据 1 的那个式子。
fn fanout(node: u64, header: u64, range_field: u64, entry: u64) -> u64 {
    if entry == 0 {
        return 0;
    }
    let overhead = header.saturating_add(range_field);
    if node <= overhead {
        return 0;
    }
    (node - overhead) / entry
}

/// 树高（含叶层）：叶层按叶扇出装，上面各层按内部扇出装。
fn tree_height(n: u64, leaf_f: u64, inner_f: u64) -> Option<u64> {
    if leaf_f == 0 || inner_f < 2 {
        return None;
    }
    let mut h = 1u64;
    let mut cap = leaf_f as u128;
    while cap < n as u128 {
        cap = cap.saturating_mul(inner_f as u128);
        h += 1;
        if h > 64 {
            return None; // 不收敛就报 None，不许挂死也不许返回一个数
        }
    }
    Some(h)
}

/// 一棵满树自己占多少字节：各层节点数之和 × 节点大小。
fn tree_bytes(n: u64, leaf_f: u64, inner_f: u64) -> Option<u64> {
    let h = tree_height(n, leaf_f, inner_f)?;
    let mut level = n.div_ceil(leaf_f).max(1);
    let mut nodes = level;
    for _ in 1..h {
        level = level.div_ceil(inner_f).max(1);
        nodes += level;
    }
    Some(nodes.saturating_mul(NODE_BYTES))
}

/// 判据 3：把代编码成模 `m` 的环形计数器，代号跑到 `gens` 时的逆序对数。
/// `m == 0` 表示不取模（存完整 checkpoint 号）。
fn inversions_mod(gens: u64, m: u64) -> u64 {
    let enc = |g: u64| if m == 0 { g } else { g % m };
    let mut inv = 0u64;
    for i in 0..gens {
        for j in (i + 1)..gens {
            if enc(i) > enc(j) {
                inv += 1;
            }
        }
    }
    inv
}

/// 判据 3 的闭式：`gens` 个代按模 `m` 编码时的逆序对数。
/// 记 `q = gens / m`（整周期数）、`r = gens % m`：整周期两两之间贡献 `C(q,2) × C(m,2)`；
/// 每个整周期与尾巴那 `r` 个各贡献 `Σ_{v<r}(m−1−v)`；周期内部与尾巴内部递增，贡献 0。
fn inversions_mod_closed(gens: u64, m: u64) -> u64 {
    if m == 0 {
        return 0;
    }
    let q = gens / m;
    let r = gens % m;
    let pairs_within = m * (m - 1) / 2;
    let mut inv = q * (q - 1) / 2 * pairs_within;
    let tail = r * (m - 1) - r * (r - 1) / 2;
    inv += q * tail;
    inv
}

/// 判据 6：一个 `w` 字节的落点字段能表达多少个**非单元对齐**的落点。
/// 按字节偏移编码：`2^(8w) − 2^(8w)/单元`；按单元号编码：恒 0。
fn misaligned_representable(bytes: u64, grain: u64, by_slot_number: bool) -> u128 {
    if by_slot_number {
        return 0;
    }
    let c = capacity(bytes);
    c - c / grain as u128
}

/// 「只分配了某个 32 KiB 数据单元的半边」这个非法状态，在按 `grain` 编码的坐标里可不可表示。
/// **只看坐标分辨率**：分辨率细到能单独点名半边，就写得出来——
/// 加不加跨度段、一条记一个槽还是记一个单元，都改变不了这一点，
/// 因为「跨度 = 1」必须永远合法（索引节点恰好就是一个 16 KiB 槽）。
fn half_unit_representable(grain: u64) -> bool {
    grain < DATA_UNIT_BYTES
}

/// **判据 7**：一个按粒度 `grain` 编码槽号的落点字段，在一段 `slots_16k` 个 16 KiB 槽的区域里，
/// 表达得了多少个**已定条款允许的**落点起点，表达不了多少个。
///
/// D18 已定项 9 允许两类起点：数据单元按 32768 对齐、索引节点按 16384 对齐
/// ⇒ 合法起点就是全部 16 KiB 槽边界，共 `slots_16k` 个。
/// 按 `grain` 编码槽号只表达得了 `grain` 的整数倍那些 ⇒ 可表达 = `⌈slots / (grain/16384)⌉`。
fn alignable(slots_16k: u64, grain: u64) -> (u64, u64) {
    let step = (grain / NODE_ALIGN).max(1);
    let ok = slots_16k.div_ceil(step);
    (ok, slots_16k - ok)
}

/// **判据 8**：模 `m` 的代字段每多少**毫秒**回绕一次（`m == 0` ⇒ 不取模 ⇒ None）。
/// 口径是**发布率**不是 `T_time`：`ms = m × 1000 / 2785`（D5 已定项 2 骑手条款同一口径）。
fn wrap_millis(m: u64) -> Option<u128> {
    if m == 0 {
        return None;
    }
    Some(m as u128 * 1000 / PUBLISH_PER_SEC as u128)
}

/// 一个 `w` 字节的落点字段能寻址多大的一块盘。
fn addressable_bytes(bytes: u64, grain: u64, by_slot_number: bool) -> u128 {
    let c = capacity(bytes);
    if by_slot_number {
        c.saturating_mul(grain as u128)
    } else {
        c
    }
}

/// **判据 9 的条款 → 段映射**：每条要求某个字段存在的已定条款，都要在这里指得到一段。
/// 少一行不会被判据 2 看见——判据 2 只查**已经列出来**的段。
struct Required {
    clause: &'static str,
    field: &'static str,
    /// 段清单里有没有它。`false` = 一处格式级空白，而且是判据 2 的盲区。
    present: bool,
}
const REQUIRED: [Required; 6] = [
    Required { clause: "D5-item1", field: "stat_tag", present: true },
    Required { clause: "D5-item1", field: "tree_id",  present: true },
    Required { clause: "D5-item1", field: "dev_id",   present: true },
    Required { clause: "D5-item2", field: "gen",      present: true },
    Required { clause: "D5-item3", field: "value",    present: true },
    // D8 write buffer 硬要求 3 逐字「条目必须自带序号（seq），这是格式要求」。
    // **第一、二版的段清单里没有它** —— 判据 9 就是为了让这一格能被看见。
    Required { clause: "D8-wb3",   field: "seq",      present: false },
];

/// E71 的窄读：只有第 8、9 项按树分；带设备维的是第 1、2、3、5、11 项。
/// **次序与 D5 已定项 4 那张表逐行对应**，per_tree 那一列该表没有（E71 已记）。
struct Stat {
    per_device: bool,
    per_tree_narrow: bool,
}
const TABLE: [Stat; STATS as usize] = [
    Stat { per_device: true,  per_tree_narrow: false }, // 1 已分配字节
    Stat { per_device: true,  per_tree_narrow: false }, // 2 空闲字节
    Stat { per_device: true,  per_tree_narrow: false }, // 3 不可回收字节
    Stat { per_device: false, per_tree_narrow: false }, // 4 待删占用
    Stat { per_device: true,  per_tree_narrow: false }, // 5 defer 待释放
    Stat { per_device: false, per_tree_narrow: false }, // 6 已承诺预留
    Stat { per_device: false, per_tree_narrow: false }, // 7 扩展点配额已用
    Stat { per_device: false, per_tree_narrow: true  }, // 8 每树独占
    Stat { per_device: false, per_tree_narrow: true  }, // 9 每树共享
    Stat { per_device: false, per_tree_narrow: false }, // 10 碎片度 runs
    Stat { per_device: true,  per_tree_narrow: false }, // 11 全空聚簇段数
];

/// 记账树条目数。`wide` = 宽读（十一项都按树分）。
fn acct_entries(trees: u64, devs: u64, k: u64, wide: bool) -> u64 {
    TABLE
        .iter()
        .map(|s| {
            let t = if wide || s.per_tree_narrow { trees } else { 1 };
            let d = if s.per_device { devs } else { 1 };
            t * d * k
        })
        .sum()
}

/// **判据 10**：一次发布脏掉多少字节——按**节点**算，不按条目字节算。
/// 记账树按 COW 挂在根下（D5 已定项 3 第 4 条）⇒ 一条插入脏一个完整节点。
/// 最坏口径：每个 `(统计量, 树, 设备)` 组每次发布各被触碰一次（与 E71 判据 2 同一口径），
/// 而同组的 K 代在 key 空间里连续 ⇒ 每叶装 `leaf_f / K` 组 ⇒ 组一被触碰，那片叶就脏。
fn dirty_bytes_per_publish(groups: u64, k: u64, leaf_f: u64) -> Option<u64> {
    if leaf_f == 0 || k == 0 {
        return None;
    }
    let groups_per_leaf = (leaf_f / k).max(1);
    let leaves = groups.div_ceil(groups_per_leaf);
    Some(leaves.saturating_mul(NODE_BYTES))
}

/// 分配记录条目的盘上宽度 = 设备身份 + 偏移 + 跨度段 + 分配代。
/// **抽成函数是为了让变异碰得到它**：写在 `main` 里的那一行没有任何单测看得见（实测踩过）。
fn alloc_entry_bytes(dev_b: u64, off_b: u64, span_b: u64, gen_b: u64) -> u64 {
    dev_b + off_b + span_b + gen_b
}

/// 分配记录条目数：已分配落点数（E74 口径）。
fn alloc_entries(cap_bytes: u64, grain: u64, fill_pct: u64) -> u64 {
    (cap_bytes / grain) * fill_pct / 100
}

fn cov_str(c: Coverage) -> &'static str {
    match c {
        Coverage::Covers => "covers",
        Coverage::Overflows => "OVERFLOWS",
        Coverage::UpperUnknown => "upper_unknown",
    }
}

/// 记账 key 的四段。`gen_bytes == 1` 是「窗口代」那条臂。
fn acct_key(stat_b: u64, tree_b: u64, dev_b: u64, gen_b: u64) -> [Seg; 4] {
    [
        // 上界 = 今天的统计量数（D5 已定项 4）。清单自陈不封闭，余量另报。
        Seg { name: "stat_tag", bytes: stat_b, domain_upper: Some(STATS) },
        // 树数上界指不到任何已定条款：D6 已定「每头一棵自己的树」，快照数无上界。
        Seg { name: "tree_id", bytes: tree_b, domain_upper: None },
        // 设备数上界指不到：位置条目的 dev 宽度仓里没定（D19 已定项 1 只定了「带」）。
        Seg { name: "dev_id", bytes: dev_b, domain_upper: None },
        // 代的上界由 D22 已定项 7 的 checkpoint_txg 8 字节给。
        Seg { name: "gen", bytes: gen_b, domain_upper: Some(u64::MAX) },
    ]
}

fn main() {
    let mut em = Emitter::new();
    let mut lines: Vec<String> = Vec::new();
    let cap16 = 16u64 * 1024 * 1024 * 1024 * 1024;
    let cap256 = 256u64 * 1024 * 1024 * 1024 * 1024;

    lines.push(em.emit_raw(&format!(
        "name=config node_bytes={NODE_BYTES} unit_bytes={DATA_UNIT_BYTES} stats={STATS} \
         k_max={K_MAX} t_dirty={T_DIRTY} budget={BUDGET} ckpt_txg_bytes={CKPT_TXG_BYTES}"
    )));

    // ── 记账树：四条臂 × 三档节点头 × 两种子指针宽 ──────────────────────
    let acct_arms: [(&str, [Seg; 4]); 4] = [
        ("acct_e73_22B", acct_key(2, 8, 4, 8)),
        ("acct_wide_19B", acct_key(2, 8, 1, 8)),
        ("acct_tight_14B", acct_key(1, 4, 1, 8)),
        ("acct_windowgen_7B", acct_key(1, 4, 1, 1)),
    ];
    // 判据 2：逐段覆盖判定
    for (name, segs) in &acct_arms {
        for s in segs.iter() {
            lines.push(em.emit_raw(&format!(
                "name=coverage arm={name} seg={} bytes={} verdict={}",
                s.name,
                s.bytes,
                cov_str(coverage(s))
            )));
        }
    }
    // 判据 1 / 5：扇出与树高
    let acct_n_narrow = acct_entries(64, 8, 25, false);
    let acct_n_wide = acct_entries(64, 8, 25, true);
    for (name, segs) in &acct_arms {
        let kb = key_bytes(segs);
        let leaf_entry = kb + ACCT_VALUE_BYTES;
        for &hdr in NODE_HEADERS.iter() {
            for &cp in [CHILD_PTR_E73, CHILD_PTR_SETTLED].iter() {
                let inner_entry = kb + cp;
                // D18 已定项 2：记账树的节点带 key 区间 ⇒ 区间字段 = 2 × key
                let leaf_f = fanout(NODE_BYTES, hdr, 2 * kb, leaf_entry);
                let inner_f = fanout(NODE_BYTES, hdr, 2 * kb, inner_entry);
                let h_narrow = tree_height(acct_n_narrow, leaf_f, inner_f);
                let h_wide = tree_height(acct_n_wide, leaf_f, inner_f);
                lines.push(em.emit_raw(&format!(
                    "name=acct_geom arm={name} key_bytes={kb} leaf_entry={leaf_entry} \
                     inner_entry={inner_entry} header={hdr} child_ptr={cp} \
                     leaf_fanout={leaf_f} inner_fanout={inner_f} \
                     entries_narrow={acct_n_narrow} height_narrow={} \
                     entries_wide={acct_n_wide} height_wide={}",
                    h_narrow.map(|v| v.to_string()).unwrap_or_else(|| "NA".into()),
                    h_wide.map(|v| v.to_string()).unwrap_or_else(|| "NA".into()),
                )));
            }
        }
    }
    // 判据 4：一次 checkpoint 的记账写字节。D5 已定项 2：每 checkpoint 一写一删 ⇒ ×2。
    for (name, segs) in &acct_arms {
        let kb = key_bytes(segs);
        let entry = kb + ACCT_VALUE_BYTES;
        let leaf_f = fanout(NODE_BYTES, NODE_HEADERS[0], 2 * kb, entry);
        for &(t, d) in [(64u64, 8u64), (1024, 64), (8192, 8), (8192, 64)].iter() {
            for &wide in [false, true].iter() {
                let groups = acct_entries(t, d, 1, wide);
                // 旧口径：条目字节（一写一删 ⇒ ×2）。**系统性低估**，留着做对照。
                let entry_bytes = groups * 2 * entry;
                // 判据 10 的新口径：脏叶字节
                let node_bytes = dirty_bytes_per_publish(groups, 25, leaf_f);
                lines.push(em.emit_raw(&format!(
                    "name=acct_budget arm={name} trees={t} devs={d} reading={} entry={entry} \
                     leaf_fanout={leaf_f} groups={groups} entry_bytes={entry_bytes} \
                     over_by_entry={} node_bytes={} over_by_node={}",
                    if wide { "wide" } else { "narrow" },
                    u8::from(entry_bytes > BUDGET),
                    node_bytes.map(|v| v.to_string()).unwrap_or_else(|| "NA".into()),
                    node_bytes.map(|v| u8::from(v > BUDGET).to_string()).unwrap_or_else(|| "NA".into()),
                )));
            }
        }
    }

    // 判据 4 的反解：宽读那一侧，key 最宽能到几字节还不撑破 T_dirty 的 1%
    for &(t, d) in [(64u64, 8u64), (1024, 64), (8192, 8), (8192, 64)].iter() {
        for &wide in [false, true].iter() {
            let touched = acct_entries(t, d, 1, wide) * 2;
            let max_entry = BUDGET / touched;
            let max_key = max_entry.saturating_sub(ACCT_VALUE_BYTES);
            lines.push(em.emit_raw(&format!(
                "name=acct_key_ceiling trees={t} devs={d} reading={} touched_entries={touched} \
                 budget={BUDGET} max_entry_bytes={max_entry} max_key_bytes={max_key} feasible={}",
                if wide { "wide" } else { "narrow" },
                u8::from(max_entry > ACCT_VALUE_BYTES),
            )));
        }
    }

    // ── 分配记录树：四条臂 ────────────────────────────────────────────
    // (名字, dev 宽, 偏移宽, 按槽号编码, 编码粒度, 代宽, 跨度段宽, 一条记一个单元)
    // ⚠️ **第三轮把粒度做成参数**：第一、二版把它硬编成 32768，
    // 于是提案采纳的 16 KiB 那条编码从来没有一条完整的臂（第二轮反推攻击腿指出）。
    let alloc_arms: [(&str, u64, u64, bool, u64, u64, u64, bool); 7] = [
        ("alloc_ptrcoord_15B",  1, 6, false, DATA_UNIT_BYTES, 8, 0, false),
        ("alloc_e74_20B",       4, 8, false, DATA_UNIT_BYTES, 8, 0, false),
        ("alloc_unitno32k_14B", 1, 5, true,  DATA_UNIT_BYTES, 8, 0, false),
        // 与 alloc_ptrcoord_15B **同宽**，只换编码 —— 判据 6 的对照臂
        ("alloc_unitno32k_15B", 1, 6, true,  DATA_UNIT_BYTES, 8, 0, false),
        ("alloc_windowgen_7B",  1, 5, true,  DATA_UNIT_BYTES, 1, 0, false),
        // **第三轮补：提案实际采纳的那条**——16 KiB 槽号，一槽一条
        ("alloc_slot16k_15B",   1, 6, true,  NODE_ALIGN,      8, 0, false),
        // **第三轮补：第二轮反推攻击腿开出的替代**——16 KiB 槽号 + 1 字节跨度段，
        // **一个单元一条**：半个单元被分配这个非法状态因此不可表示
        ("alloc_span16k_16B",   1, 6, true,  NODE_ALIGN,      8, 1, true),
    ];
    for &(name, devb, offb, by_slot, grain, genb, spanb, per_unit) in alloc_arms.iter() {
        let kb = devb + offb;
        let leaf_entry = alloc_entry_bytes(devb, offb, spanb, genb);
        let gen_seg = Seg { name: "gen", bytes: genb, domain_upper: Some(u64::MAX) };
        // 判据 6 与寻址范围
        lines.push(em.emit_raw(&format!(
            "name=alloc_encoding arm={name} dev_bytes={devb} off_bytes={offb} \
             by_slot_number={} grain={grain} gen_bytes={genb} span_bytes={spanb} \
             per_unit_record={} entry={leaf_entry} gen_coverage={} \
             misaligned_representable={} addressable_bytes={} half_unit_representable={}",
            u8::from(by_slot),
            u8::from(per_unit),
            cov_str(coverage(&gen_seg)),
            misaligned_representable(offb, grain, by_slot),
            addressable_bytes(offb, grain, by_slot),
            // 「半个数据单元被分配」这个非法状态可不可表示。
            // ⚠️ **第三轮修正**：第二版写的是 `!per_unit && grain < 单元`，
            // 也就是从**臂的标签**推出来的，不是从编码推出来的——正是本工程
            // 「模型硬编码出来的伪影」那一类。真相是：只要坐标分辨率细到 16 KiB，
            // 「一条覆盖某个数据单元的半边」的记录就写得出来（跨度段取 1 即可，
            // 而跨度 1 必须永远合法——索引节点就是 16 KiB）⇒ **与 per_unit 无关**。
            u8::from(half_unit_representable(grain)),
        )));
        // 判据 1 / 5 / 代价：两种「带不带 key 区间」的读法都算，D18 已定项 2 没说分配记录树
        for &with_range in [false, true].iter() {
            let rf = if with_range { 2 * kb } else { 0 };
            for &hdr in NODE_HEADERS.iter() {
                let leaf_f = fanout(NODE_BYTES, hdr, rf, leaf_entry);
                let inner_f = fanout(NODE_BYTES, hdr, rf, kb + CHILD_PTR_SETTLED);
                for &cap in [cap16, cap256].iter() {
                    // 一单元一条 ⇒ 条目数按单元数；一槽一条 ⇒ 按编码粒度数
                    let n = alloc_entries(cap, if per_unit { DATA_UNIT_BYTES } else { grain }, 90);
                    let h = tree_height(n, leaf_f, inner_f);
                    let tb = tree_bytes(n, leaf_f, inner_f);
                    let ppm = tb.map(|b| b as u128 * 1_000_000 / cap as u128);
                    lines.push(em.emit_raw(&format!(
                        "name=alloc_geom arm={name} with_range={} header={hdr} cap_bytes={cap} \
                         entries={n} leaf_fanout={leaf_f} inner_fanout={inner_f} height={} \
                         tree_bytes={} occupancy_ppm={}",
                        u8::from(with_range),
                        h.map(|v| v.to_string()).unwrap_or_else(|| "NA".into()),
                        tb.map(|v| v.to_string()).unwrap_or_else(|| "NA".into()),
                        ppm.map(|v| v.to_string()).unwrap_or_else(|| "NA".into()),
                    )));
                }
            }
        }
        // 整理的 COW 放大：同样搬 N 条，条目越窄放大越大（E74 口径）
        let leaf_f = fanout(NODE_BYTES, NODE_HEADERS[0], 0, leaf_entry);
        let n_move = 4096u64;
        let leaves_contig = n_move.div_ceil(leaf_f.max(1));
        let amp_contig = leaves_contig as u128 * NODE_BYTES as u128
            / (n_move as u128 * leaf_entry as u128);
        let amp_scatter = n_move as u128 * NODE_BYTES as u128
            / (n_move as u128 * leaf_entry as u128);
        lines.push(em.emit_raw(&format!(
            "name=alloc_cow arm={name} entry={leaf_entry} leaf_fanout={leaf_f} \
             leaves_contiguous={leaves_contig} amp_contiguous={amp_contig} amp_scattered={amp_scatter}"
        )));
        // 判据 9：段清单完整性
        if name == alloc_arms[0].0 {
            for r in REQUIRED.iter() {
                lines.push(em.emit_raw(&format!(
                    "name=segment_required clause={} field={} present={}",
                    r.clause,
                    r.field,
                    u8::from(r.present)
                )));
            }
        }
    }

    // ── 判据 3 + 8：代的环形编码。**代数必须扫到模之上**，否则等于给大模发免检证。
    for &m in [0u64, K_MAX, 256, 1024, 65536].iter() {
        for &gens in [32u64, 64, 257, 300, 512, 70000].iter() {
            // 逆序对是 O(gens²)，7 万代那格用闭式，别拿 49 亿次循环去跑
            let inv = if gens > 4096 {
                inversions_mod_closed(gens, m)
            } else {
                inversions_mod(gens, m)
            };
            lines.push(em.emit_raw(&format!(
                "name=gen_order modulus={m} gens={gens} inversions={inv} wrap_millis={}",
                wrap_millis(m).map(|v| v.to_string()).unwrap_or_else(|| "never".into())
            )));
        }
    }

    // ── 判据 7：对齐可表示性。D18 已定项 9 允许的落点起点 = 全部 16 KiB 槽边界。
    for &slots in [4096u64, 998].iter() {
        for &grain in [NODE_ALIGN, DATA_UNIT_BYTES].iter() {
            let (ok, bad) = alignable(slots, grain);
            lines.push(em.emit_raw(&format!(
                "name=alignable slots_16k={slots} grain={grain} representable={ok} unrepresentable={bad}"
            )));
        }
    }

    // ── 落点粒度这一维：条目数与占容量按粒度反比走（E74 扫过三档，第一版 E97 硬编成一档）
    for &grain in [NODE_ALIGN, DATA_UNIT_BYTES].iter() {
        for &cap in [cap16, cap256].iter() {
            let n = alloc_entries(cap, grain, 90);
            let leaf_f = fanout(NODE_BYTES, 58, 0, 15);
            let inner_f = fanout(NODE_BYTES, 58, 0, 7 + CHILD_PTR_SETTLED);
            let h = tree_height(n, leaf_f, inner_f);
            let tb = tree_bytes(n, leaf_f, inner_f);
            lines.push(em.emit_raw(&format!(
                "name=grain_sweep grain={grain} cap_bytes={cap} entries={n} \
                 leaf_fanout={leaf_f} inner_fanout={inner_f} height={} tree_bytes={} occupancy_ppm={}",
                h.map(|v| v.to_string()).unwrap_or_else(|| "NA".into()),
                tb.map(|v| v.to_string()).unwrap_or_else(|| "NA".into()),
                tb.map(|b| (b as u128 * 1_000_000 / cap as u128).to_string()).unwrap_or_else(|| "NA".into()),
            )));
        }
    }

    // ── 阳性对照：每条臂每一段各砍成 0 ────────────────────────────────
    for (name, segs) in &acct_arms {
        let full = key_bytes(segs);
        for i in 0..segs.len() {
            let mut cut = *segs;
            let was = cut[i].bytes;
            cut[i].bytes = 0;
            let shrunk = key_bytes(&cut);
            lines.push(em.emit_raw(&format!(
                "name=control_cut arm={name} seg={} was={was} key_full={full} key_cut={shrunk} \
                 delta_ok={} verdict_after={}",
                cut[i].name,
                u8::from(full - shrunk == was),
                cov_str(coverage(&cut[i])),
            )));
        }
    }

    for l in &lines {
        println!("{l}");
    }
    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 格式常量必须与 kb 的 format-const 标记一致。
    #[test]
    fn format_constants_match_kb() {
        assert_eq!(NODE_BYTES, 16384, "D8 已定项 2");
        assert_eq!(DATA_UNIT_BYTES, 32768, "D4 已定项 5");
        assert_eq!(STATS, 11, "D5 已定项 4 第一次重开之后");
        assert_eq!(K_MAX, 49, "D22 已定项 2：3 × 16 + 1");
        assert_eq!(CKPT_TXG_BYTES, 8, "D22 已定项 7");
    }

    /// **判据 1 的绝对值**：22 字节 key、头 58、带区间、子指针 32
    /// ⇒ 内部扇出恰好 301 —— 与 E73 那张表逐格相同，说明两个模型接得上。
    #[test]
    fn criterion1_absolute_fanout_reproduces_e73() {
        let kb = 22u64;
        let f = fanout(NODE_BYTES, 58, 2 * kb, kb + CHILD_PTR_E73);
        assert_eq!(kb + CHILD_PTR_E73, 54, "E73 的条目宽");
        assert_eq!(58 + 2 * kb, 102, "带区间的头");
        assert_eq!(NODE_BYTES - 102, 16282);
        assert_eq!(f, 301, "16282 / 54 = 301，与 E73 的 301 逐格相同");
        // 不带区间那一侧 E73 报 302
        assert_eq!(fanout(NODE_BYTES, 58, 0, 54), 302);
    }

    /// **判据 1 的绝对值（分配记录侧）**：E74 的 20 字节条目、头 64、不带区间
    /// ⇒ 扇出恰好 816，与 E74 那张表相同。
    #[test]
    fn criterion1_absolute_fanout_reproduces_e74() {
        assert_eq!(fanout(NODE_BYTES, 64, 0, 20), 816);
        assert_eq!(alloc_entries(16 * 1024 * 1024 * 1024 * 1024, DATA_UNIT_BYTES, 90), 483_183_820);
    }

    /// **判据 2 的绝对值**：容量恰好是 2^(8w)，边界两侧各钉一格。
    #[test]
    fn criterion2_capacity_is_exact_power() {
        assert_eq!(capacity(0), 1);
        assert_eq!(capacity(1), 256);
        assert_eq!(capacity(2), 65_536);
        assert_eq!(capacity(4), 4_294_967_296);
        assert_eq!(capacity(8), 1u128 << 64);
        // 1 字节装得下 11 个统计量，0 字节装不下
        let s1 = Seg { name: "x", bytes: 1, domain_upper: Some(STATS) };
        let s0 = Seg { name: "x", bytes: 0, domain_upper: Some(STATS) };
        assert_eq!(coverage(&s1), Coverage::Covers);
        assert_eq!(coverage(&s0), Coverage::Overflows);
        // 8 字节的代要装下 u64::MAX ⇒ 2^64 > 2^64−1，恰好过；7 字节过不了
        let g8 = Seg { name: "gen", bytes: 8, domain_upper: Some(u64::MAX) };
        let g7 = Seg { name: "gen", bytes: 7, domain_upper: Some(u64::MAX) };
        assert_eq!(coverage(&g8), Coverage::Covers);
        assert_eq!(coverage(&g7), Coverage::Overflows);
        // 上界指不到的段不许被判成「过了」
        let unk = Seg { name: "tree_id", bytes: 1, domain_upper: None };
        assert_eq!(coverage(&unk), Coverage::UpperUnknown);
    }

    /// **判据 3 + 阳性对照二**：不取模恒 0；模小于代数时逆序对是可数的绝对值。
    #[test]
    fn criterion3_wrapped_generation_breaks_key_order() {
        assert_eq!(inversions_mod(64, 0), 0, "存完整 checkpoint 号：零逆序");
        assert_eq!(inversions_mod(32, 64), 0, "模大于代数：零逆序（阴性对照）");
        // 模 16、32 代 ⇒ 每个后半段元素与前半段同余的更大者构成逆序
        // 手算：32 代模 16 = 2 个整周期 ⇒ C(2,2) × C(16,2) = 1 × 120
        assert_eq!(inversions_mod(32, 16), 120, "32 代模 16：恰好 120 对");
        // 手算：64 代模 16 = 4 个整周期 ⇒ C(4,2) × C(16,2) = 6 × 120 = 720
        assert_eq!(inversions_mod(64, 16), 720, "64 代模 16：恰好 720 对");
        // 手算：64 代模 49 = 一个满周期 0..48 加一段 0..14
        // ⇒ Σ_{j=49..63}(97−j) = Σ_{k=34..48} k = (34+48)×15/2 = 615
        assert_eq!(inversions_mod(64, K_MAX), 615, "K=49 也不够：仍有 615 对");
        // 单调：模越小逆序越多
        assert!(inversions_mod(64, 16) > inversions_mod(64, 64));
    }

    /// **判据 6 的绝对值**：按字节偏移能表达大量非对齐落点，按单元号恒 0。
    #[test]
    fn criterion6_unit_number_makes_misaligned_unrepresentable() {
        // 6 字节字节偏移：2^48 个值里只有 2^48/32768 = 2^33 个是单元对齐的
        let m = misaligned_representable(6, DATA_UNIT_BYTES, false);
        // 手算：2^48 − 2^33 = 281 474 976 710 656 − 8 589 934 592
        assert_eq!(m, (1u128 << 48) - (1u128 << 33));
        assert_eq!(m, 281_466_386_776_064);
        assert_eq!(misaligned_representable(6, DATA_UNIT_BYTES, true), 0);
        // 寻址范围：5 字节单元号 > 6 字节字节偏移，且恰好是 128 倍
        let by_unit5 = addressable_bytes(5, DATA_UNIT_BYTES, true);
        let by_byte6 = addressable_bytes(6, DATA_UNIT_BYTES, false);
        assert_eq!(by_byte6, 1u128 << 48, "256 TiB");
        assert_eq!(by_unit5, 1u128 << 55, "32 PiB");
        assert_eq!(by_unit5 / by_byte6, 128);
        // **同宽对照**：6 字节两种编码，单元号多 32768 倍寻址（= 单元大小本身）
        let by_unit6 = addressable_bytes(6, DATA_UNIT_BYTES, true);
        assert_eq!(by_unit6, 1u128 << 63, "8 EiB");
        assert_eq!(by_unit6 / by_byte6, 32_768);
        assert_eq!(misaligned_representable(6, DATA_UNIT_BYTES, true), 0);
    }

    /// **COW 放大那一栏是比值，分母随条目宽变——不许拿它当「窄条目更贵」的证据。**
    /// 绝对量：打散一侧写的节点数与条目宽无关；连续一侧条目越窄碰的叶越少。
    #[test]
    fn cow_amplification_is_a_ratio_not_an_absolute_cost() {
        let n = 4096u64;
        let f15 = fanout(NODE_BYTES, 58, 0, 15);
        let f7 = fanout(NODE_BYTES, 58, 0, 7);
        assert_eq!(f15, 1088, "手算 (16384−58)/15 = 16326/15 = 1088.4 ⇒ 1088");
        assert_eq!(f7, 2332, "手算 16326/7 = 2332.28 ⇒ 2332");
        // 连续：窄条目碰的叶更少（绝对量更小）
        assert_eq!(n.div_ceil(f15), 4);
        assert_eq!(n.div_ceil(f7), 2);
        // 打散：两档写的节点数都是 N，与条目宽无关
        assert_eq!(n, 4096);
        // 而「放大倍数」却随条目变窄而变大 —— 变的是分母
        assert_eq!(NODE_BYTES / 15, 1092);
        assert_eq!(NODE_BYTES / 7, 2340);
    }

    /// **判据 5 单调**：每加一字节，条目加一、扇出不增、树高不减。
    #[test]
    fn criterion5_monotone_in_每一段() {
        let mut prev_f = u64::MAX;
        let mut prev_h = 0u64;
        for kb in 7..=24u64 {
            let f = fanout(NODE_BYTES, 58, 2 * kb, kb + ACCT_VALUE_BYTES);
            let h = tree_height(1_000_000, f, f).unwrap();
            assert!(f <= prev_f, "扇出必须单调不增：{kb}");
            assert!(h >= prev_h, "树高必须单调不减：{kb}");
            prev_f = f;
            prev_h = h;
        }
        // 绝对值锚：7 字节 key 与 22 字节 key 的叶扇出各自钉死
        // 手算：(16384 − 58 − 14) / 15 = 16312 / 15 = 1087.4… ⇒ 1087
        assert_eq!(fanout(NODE_BYTES, 58, 14, 15), 1087);
        assert_eq!(fanout(NODE_BYTES, 58, 44, 30), 542);
    }

    /// **记账条目数**：与 E71 的窄读 / 上界公式对得上（十一个统计量口径）。
    #[test]
    fn accounting_entry_counts_are_absolute() {
        // 窄读，t=64 d=8 K=25：E71 在九个统计量下是 4075，加两项之后 4300
        assert_eq!(acct_entries(64, 8, 25, false), 4300);
        // 宽读（十一项都按树分）
        assert_eq!(acct_entries(64, 8, 25, true), 11 * 64 * 25 + 5 * 64 * 8 * 25 - 5 * 64 * 25);
        // 上界公式 s × t × d × K
        assert_eq!(STATS * 64 * 8 * 25, 140_800);
        assert!(acct_entries(64, 8, 25, true) < STATS * 64 * 8 * 25, "宽读仍低于上界");
    }

    /// **判据 4**：预算线是绝对值，且窄读在扫过的三格都不过界。
    #[test]
    fn criterion4_budget_absolute() {
        assert_eq!(BUDGET, 21_474_836);
        for &(t, d) in [(64u64, 8u64), (1024, 64), (8192, 8)].iter() {
            let bytes = acct_entries(t, d, 1, false) * 2 * (19 + ACCT_VALUE_BYTES);
            assert!(bytes <= BUDGET, "窄读 t={t} d={d} 不该过界：{bytes}");
        }
        // 阳性对照：宽读在 t×d 很大那一格必须过界，否则这条闸分不出差别
        let wide = acct_entries(8192, 64, 1, true) * 2 * (19 + ACCT_VALUE_BYTES);
        assert!(wide > BUDGET, "宽读该过界却没过：{wide}");
    }

    /// **树高与占容量比是绝对值**，不是「大致这么多」。
    #[test]
    fn alloc_tree_occupancy_is_absolute() {
        let n = alloc_entries(16 * 1024 * 1024 * 1024 * 1024, DATA_UNIT_BYTES, 90);
        let leaf_f = fanout(NODE_BYTES, 64, 0, 20);
        let inner_f = fanout(NODE_BYTES, 64, 0, 12 + CHILD_PTR_SETTLED);
        assert_eq!(leaf_f, 816, "手算 (16384−64)/20 = 816");
        assert_eq!(inner_f, 251, "手算 (16384−64)/(12+53) = 16320/65 = 251.0… ⇒ 251");
        assert_eq!(tree_height(n, leaf_f, inner_f), Some(4));
        let b = tree_bytes(n, leaf_f, inner_f).unwrap();
        // 手算各层节点数：叶 ⌈483183820/816⌉ = 592138、2360、10、1 ⇒ 594509 个节点
        assert_eq!(n.div_ceil(leaf_f), 592_138);
        assert_eq!(b / NODE_BYTES, 594_509);
        assert_eq!(b, 594_509 * 16_384);
        assert_eq!(b, 9_740_435_456);
        // E74 报 9.0 GiB —— 它整棵树都按 leaf_f 乘上去，本模型内部层用内部扇出，故略大
        assert_eq!(b as u128 * 1_000_000 / (16u128 * 1024 * 1024 * 1024 * 1024), 553);
    }

    /// **叶条目 = key + 完整值**，且这一档的扇出是绝对值，不是「差不多」。
    #[test]
    fn acct_leaf_entry_is_key_plus_value() {
        assert_eq!(ACCT_VALUE_BYTES, 8, "统计量是字节计数，256 TiB 也只用到 48 位");
        let segs = acct_key(2, 8, 1, 8);
        let kb = key_bytes(&segs);
        assert_eq!(kb, 19);
        let leaf = kb + ACCT_VALUE_BYTES;
        assert_eq!(leaf, 27, "19 字节 key + 8 字节完整值");
        // 手算：(16384 − 58 − 2×19) / 27 = 16288 / 27 = 603.2… ⇒ 603
        assert_eq!(fanout(NODE_BYTES, 58, 2 * kb, leaf), 603);
        assert_eq!(CHILD_PTR_E73, 32, "E73 的参数");
        assert_eq!(CHILD_PTR_SETTLED, 53, "D22 已定项 7 的树表单元指针实宽");
    }

    /// **判据 4 的反解**：宽读下 key 宽度有一个硬上限，而 E73 假设的 22 恰好越过它。
    #[test]
    fn criterion4_wide_reading_puts_a_ceiling_on_key_bytes() {
        let touched = acct_entries(8192, 8, 1, true) * 2;
        // 手算：宽读 ⇒ 11 项都按树分；5 项带设备维
        // ⇒ 5×8192×8 + 6×8192 = 327680 + 49152 = 376832，一写一删 ⇒ ×2
        assert_eq!(touched, 753_664);
        assert_eq!(BUDGET / touched, 28, "21474836 / 753664 = 28.49… ⇒ 28");
        assert_eq!(28 - ACCT_VALUE_BYTES, 20, "key 最宽 20 字节");
        assert!(touched * (22 + ACCT_VALUE_BYTES) > BUDGET, "E73 的 22 字节越界");
        assert!(touched * (20 + ACCT_VALUE_BYTES) <= BUDGET, "20 字节不越界");
    }

    /// **判据 7（第二轮加）**：按 32 KiB 槽号编码表达不了半数索引节点的落点。
    /// 依据 D18 已定项 9 逐字「数据单元对齐 32768、索引节点对齐 16384」。
    #[test]
    fn criterion7_a_32k_slot_number_cannot_address_half_the_index_nodes() {
        assert_eq!(NODE_ALIGN, 16384, "D8 已定项 2 + D18 已定项 9");
        // 按 16 KiB 槽号编码：D18 已定项 9 允许的起点全部表达得了
        assert_eq!(alignable(4096, NODE_ALIGN), (4096, 0));
        // 按 32 KiB 单元号编码：只表达得了偶数槽 ⇒ 4096 个里有 2048 个表达不了
        assert_eq!(alignable(4096, DATA_UNIT_BYTES), (2048, 2048));
        // E86 那个 998 节点的区域：奇数槽那一半同样表达不了（E86 实测 496 个在奇槽，
        // 这里数的是「一段连续 998 槽里表达不了几个」，两个数不是同一个口径）
        assert_eq!(alignable(998, DATA_UNIT_BYTES), (499, 499));
        // 阳性对照：粒度等于对齐时表达不了的必须恰好是 0
        assert_eq!(alignable(998, NODE_ALIGN).1, 0);
    }

    /// **判据 8（第二轮加）**：代字段取模，回绕期由 `T_time` 直接给。
    /// 第一版只测 32 / 64 代 ⇒ 模 256 逆序对恒 0，等于给 1 字节的代发了免检证。
    #[test]
    fn criterion8_a_wrapped_generation_survives_only_until_it_wraps() {
        assert_eq!(PUBLISH_PER_SEC, 2785, "E44 本机实测的发布率");
        assert_eq!(GEN_BITS_FLOOR, 48, "D5 已定项 2 骑手条款：代段 ≥ 48 位");
        // 1 字节的代（模 256）：**91 毫秒**就绕完一圈（2785 发布/秒）
        assert_eq!(wrap_millis(256), Some(91));
        // 2 字节（模 65536）：23.5 秒
        assert_eq!(wrap_millis(65536), Some(23_531));
        // **交叉复现 D5 已定项 2 的骑手条款**：32 位 17.8 天
        let d32 = wrap_millis(1u64 << 32).unwrap() / 1000 / 86400;
        assert_eq!(d32, 17, "1542178 秒 = 17.8 天，与骑手条款「32 位 17.8 天回绕」同一个数");
        // 48 位（骑手条款的下界）：3204 年，与 E44 的「48 位撑 3202 年」同量级
        let y48 = wrap_millis(1u64 << GEN_BITS_FLOOR).unwrap() / 1000 / 86400 / 365;
        assert_eq!(y48, 3204);
        assert_eq!(wrap_millis(0), None, "不取模就不回绕");
        // 第一版只测到 64 代 ⇒ 模 256 一个逆序对都没有，这正是免检证
        assert_eq!(inversions_mod(64, 256), 0);
        // 越过模之后立刻出现：257 代 ⇒ 255 对
        assert_eq!(inversions_mod(257, 256), 255);
        // 闭式与暴力法在能暴力的格子上必须逐格相同（校验路径，两条不共享代码）
        for &(g, m) in [(32u64, 16u64), (64, 16), (64, 49), (257, 256), (300, 256), (512, 256), (64, 0)].iter() {
            assert_eq!(inversions_mod(g, m), inversions_mod_closed(g, m), "g={g} m={m}");
        }
        // 手算：512 代模 256 = 2 个整周期 ⇒ C(2,2) × C(256,2) = 32640
        assert_eq!(inversions_mod_closed(512, 256), 32_640);
        // 手算：300 代模 256 ⇒ 尾巴 44 个，Σ_{v<44}(255−v) = 44×255 − 44×43/2 = 10274
        assert_eq!(inversions_mod_closed(300, 256), 10_274);
        // 2 字节的代同样会绕：70000 代模 65536 ⇒ 尾巴 4464 个
        let tail = 4464u64 * 65535 - 4464 * 4463 / 2;
        assert_eq!(inversions_mod_closed(70_000, 65_536), tail);
        assert!(tail > 0, "任何有限模在活得够久之后都会破 key 序");
    }

    /// **判据 4 的反解（第二轮补格）**：宽读在 t=8192 / d=64 那格**无可行点**——
    /// 连 7 字节的臂都过界。这是 E97 跑前写死的反向接受条款该触发的地方。
    #[test]
    fn criterion4_wide_reading_has_no_feasible_point_at_8192x64() {
        let touched = acct_entries(8192, 64, 1, true) * 2;
        // 手算：宽读 ⇒ 11 项都按树分；5 项带设备维
        // ⇒ 5×8192×64 + 6×8192 = 2621440 + 49152 = 2670592，一写一删 ⇒ ×2
        assert_eq!(touched, 5_341_184);
        assert_eq!(BUDGET / touched, 4, "21474836 / 5341184 = 4.02… ⇒ 4");
        assert!(4 < ACCT_VALUE_BYTES, "条目连 value 都装不下 ⇒ 无可行 key 宽度");
        // 窄读同一格照样可行
        let narrow = acct_entries(8192, 64, 1, false) * 2;
        // 手算：窄读 ⇒ 只有第 8、9 项按树分（2×8192）、5 项带设备维（5×64）、
        // 其余 4 项无维（4×1）⇒ 16384 + 320 + 4 = 16708，一写一删 ⇒ ×2
        assert_eq!(narrow, 33_416);
        assert!(BUDGET / narrow > 8 + 22, "窄读连 22 字节 key 都装得下，余量还很大");
    }

    /// **落点粒度这一维（第二轮加）**：条目数按粒度反比走，占容量跟着翻倍。
    #[test]
    fn grain_halving_doubles_the_allocation_record_tree() {
        let cap = 16u64 * 1024 * 1024 * 1024 * 1024;
        let n32 = alloc_entries(cap, DATA_UNIT_BYTES, 90);
        let n16 = alloc_entries(cap, NODE_ALIGN, 90);
        assert_eq!(n32, 483_183_820);
        assert_eq!(n16, 966_367_641);
        assert_eq!(n16 - 2 * n32, 1, "整数除法取整差 1，不是模型错");
    }

    /// **判据 9（第三轮加）**：段清单里恰好缺一段——D8 write buffer 硬要求 3 的 `seq`。
    /// 判据 2 看不见它，因为判据 2 只查**已经列出来**的段。
    #[test]
    fn criterion9_the_segment_list_is_missing_exactly_one_settled_field() {
        let missing: Vec<&str> = REQUIRED.iter().filter(|r| !r.present).map(|r| r.field).collect();
        assert_eq!(missing, vec!["seq"], "seq 是唯一没有落点的已定字段");
        assert_eq!(REQUIRED.len(), 6);
        assert_eq!(REQUIRED.iter().filter(|r| r.present).count(), 5);
        // 阳性对照：判据 2 对一段**不在清单里**的字段一个字也说不出来——
        // 它只对 Seg 数组里的元素工作，而 seq 根本不在那个数组里。
        let segs = acct_key(2, 8, 1, 8);
        assert_eq!(segs.len(), 4, "四段：统计量 / 树 / 设备 / 代，没有 seq");
        assert!(!segs.iter().any(|s| s.name == "seq"));
    }

    /// **判据 10（第三轮加）**：预算按脏节点算之后，两格从「不过界」翻成「过界」。
    /// 记账树按 COW 挂在根下 ⇒ 一条 27 字节插入脏一个 16 KiB 节点。
    #[test]
    fn criterion10_node_granularity_flips_two_grids_over_budget() {
        let entry = 19 + ACCT_VALUE_BYTES; // 27
        let leaf_f = fanout(NODE_BYTES, 58, 2 * 19, entry);
        assert_eq!(leaf_f, 603);
        // 每叶装 603 / 25 = 24 组（同组的 K 代在 key 空间里连续）
        assert_eq!(leaf_f / 25, 24);

        // t=8192 d=8 宽读：条目字节口径不过界，节点口径过界
        let g = acct_entries(8192, 8, 1, true);
        assert_eq!(g, 376_832);
        assert_eq!(g * 2 * entry, 20_348_928);
        assert!(g * 2 * entry <= BUDGET, "条目字节口径：不过界");
        let nb = dirty_bytes_per_publish(g, 25, leaf_f).unwrap();
        assert_eq!(g.div_ceil(24), 15_702, "手算 ⌈376832/24⌉");
        assert_eq!(nb, 15_702 * 16_384);
        assert_eq!(nb, 257_261_568);
        assert!(nb > BUDGET, "节点口径：过界");

        // 窄读最紧的一格仍然过得去，占预算约一半
        let gn = acct_entries(8192, 64, 1, false);
        assert_eq!(gn, 16_708);
        let nbn = dirty_bytes_per_publish(gn, 25, leaf_f).unwrap();
        assert_eq!(gn.div_ceil(24), 697, "手算 ⌈16708/24⌉");
        assert_eq!(nbn, 11_419_648);
        assert!(nbn <= BUDGET);
        assert_eq!(nbn * 100 / BUDGET, 53, "占预算 53%");
        // 不合法几何报 None，不许退化成 0
        assert_eq!(dirty_bytes_per_publish(10, 25, 0), None);
        assert_eq!(dirty_bytes_per_publish(10, 0, 603), None);
    }

    /// **第三轮补的两条臂**：16 KiB 槽号一槽一条 vs 一单元一条加跨度段。
    /// 后者贵 1 字节，却在三个轴上都不差、在两个轴上更好。
    #[test]
    fn per_unit_record_with_a_span_byte_beats_halving_the_grain() {
        let cap = 16u64 * 1024 * 1024 * 1024 * 1024;
        let inner = fanout(NODE_BYTES, 58, 0, 7 + CHILD_PTR_SETTLED);
        // 一槽一条：粒度 16384 ⇒ 条目数按槽数
        let n_slot = alloc_entries(cap, NODE_ALIGN, 90);
        let f_slot = fanout(NODE_BYTES, 58, 0, 15);
        assert_eq!(n_slot, 966_367_641);
        assert_eq!(f_slot, 1088);
        let b_slot = tree_bytes(n_slot, f_slot, inner).unwrap();
        assert_eq!(b_slot, 14_606_106_624);
        assert_eq!(b_slot as u128 * 1_000_000 / cap as u128, 830);

        // 一单元一条 + 1 字节跨度段：条目数按单元数
        let n_unit = alloc_entries(cap, DATA_UNIT_BYTES, 90);
        // **条目宽走同一个函数**，不写字面量 —— 否则跨度段漏掉时没有一个测试会红（实测踩过）
        let e_unit = alloc_entry_bytes(1, 6, 1, 8);
        assert_eq!(e_unit, 16, "dev 1 + 槽号 6 + 跨度 1 + 代 8");
        assert_eq!(alloc_entry_bytes(1, 6, 0, 8), 15, "不带跨度段就是 15");
        let f_unit = fanout(NODE_BYTES, 58, 0, e_unit);
        assert_eq!(n_unit, 483_183_820);
        assert_eq!(f_unit, 1020, "手算 (16384−58)/16 = 16326/16 = 1020.4 ⇒ 1020");
        let b_unit = tree_bytes(n_unit, f_unit, inner).unwrap();
        assert_eq!(b_unit, 7_789_936_640);
        assert_eq!(b_unit as u128 * 1_000_000 / cap as u128, 442);
        // 便宜 1.88 倍 —— ⚠️ 但 `.claude/rules/fs-design.md` 明令「比谁省不构成任何判据」
        assert_eq!(b_slot * 100 / b_unit, 187);
        // **两条要求不可兼得（第三轮的判决）**：
        // ① 表达得了 D18 已定项 9 的全部合法起点（要 16 KiB 分辨率）
        // ② 让「半个数据单元被分配」不可表示（要分辨率粗到 32 KiB）
        // 任何一个 grain 都满足不了两条 —— 加跨度段也改不了。
        for &g in [NODE_ALIGN, DATA_UNIT_BYTES].iter() {
            let all_starts_ok = alignable(4096, g).1 == 0;
            let half_unit_ok = !half_unit_representable(g);
            assert!(!(all_starts_ok && half_unit_ok), "grain={g} 不该两条都满足");
        }
        assert!(alignable(4096, NODE_ALIGN).1 == 0 && half_unit_representable(NODE_ALIGN));
        assert!(alignable(4096, DATA_UNIT_BYTES).1 == 2048 && !half_unit_representable(DATA_UNIT_BYTES));
        // ⇒ 「半个单元可不可表示」这一维在两条候选之间**分不出差别**（两条都可表示），
        //   不许拿它当选边的依据。
        assert_eq!(half_unit_representable(NODE_ALIGN), half_unit_representable(NODE_ALIGN));
        // 两条臂寻址范围相同（都是 6 字节 16 KiB 槽号 ⇒ 2^62 = 4 EiB）
        assert_eq!(addressable_bytes(6, NODE_ALIGN, true), 1u128 << 62);
        assert_eq!(misaligned_representable(6, NODE_ALIGN, true), 0, "两条都表达不了非 16K 对齐的起点");
    }

    /// 不合法的几何一律报 None / 0，不许退化成一个数（读不到 ≠ 读到 0）。
    #[test]
    fn illegal_geometry_is_not_a_measurement() {
        assert_eq!(fanout(NODE_BYTES, NODE_BYTES, 0, 20), 0);
        assert_eq!(fanout(NODE_BYTES, 0, 0, 0), 0);
        assert_eq!(tree_height(100, 0, 10), None);
        assert_eq!(tree_height(100, 10, 1), None);
        assert_eq!(tree_bytes(100, 0, 10), None);
    }

    /// **阳性对照一**：任一段砍成 0，key 必须正好少那么多，且覆盖判定必须翻红。
    #[test]
    fn control_cutting_a_segment_shrinks_key_and_flips_coverage() {
        let segs = acct_key(1, 4, 1, 8);
        let full = key_bytes(&segs);
        assert_eq!(full, 14);
        for i in 0..segs.len() {
            let mut cut = segs;
            let was = cut[i].bytes;
            cut[i].bytes = 0;
            assert_eq!(full - key_bytes(&cut), was, "第 {i} 段没进模型");
        }
        // stat_tag 与 gen 有上界 ⇒ 砍成 0 必须翻红
        let mut c0 = segs;
        c0[0].bytes = 0;
        assert_eq!(coverage(&c0[0]), Coverage::Overflows);
        let mut c3 = segs;
        c3[3].bytes = 0;
        assert_eq!(coverage(&c3[3]), Coverage::Overflows);
        // tree_id / dev_id 上界指不到 ⇒ 砍成 0 也只能报 upper_unknown，
        // **这正是判据 2 要暴露的那件事**：这两段的宽度不是量出来的，是要定的。
        let mut c1 = segs;
        c1[1].bytes = 0;
        assert_eq!(coverage(&c1[1]), Coverage::UpperUnknown);
    }
}

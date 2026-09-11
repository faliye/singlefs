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
const DIRTY_BYTES_THRESHOLD: u64 = 2 * 1024 * 1024 * 1024;
/// E44 本机实测：**2785 发布/秒**。D5 已定项 2 的骑手条款按它算代段宽度下界。
/// ⚠️ **不是 `T_time` = 5 s**——代按发布走，fsync 触发的也是发布（D16 已定项 6）。
const PUBLISHES_PER_SECOND: u64 = 2785;
/// D5 已定项 2 骑手条款：代段宽度 **≥ 48 位**。已定条款，不是本实验的发现。
const GENERATION_BITS_FLOOR: u32 = 48;
/// D8 已定项 2 + D18 已定项 9：索引节点 16 KiB，按 **16384** 对齐。
const NODE_ALIGNMENT_BYTES: u64 = 16384;
/// 判据 4 的预算线：`T_dirty` 的 1%。
const ACCOUNTING_WRITE_BUDGET_BYTES: u64 = DIRTY_BYTES_THRESHOLD / 100;
/// D5 已定项 4：统计量十一个。
const STATISTIC_COUNT: u64 = 11;
/// D22 已定项 2：根环区域数 R = 3，每区槽数上限 16 ⇒ K = 3S + 1 ≤ 49。
const RING_REGIONS: u64 = 3;
const SLOTS_PER_REGION_MAXIMUM: u64 = 16;
const KEPT_GENERATIONS_MAXIMUM: u64 = RING_REGIONS * SLOTS_PER_REGION_MAXIMUM + 1;
/// D22 已定项 7：`checkpoint_txg` 8 字节 ⇒ 「代」的值域要 8 字节才装得下。
const CHECKPOINT_TRANSACTION_GROUP_BYTES: u64 = 8;
/// E73 按 D18 已定项 7 重算过的三档基础节点头下界。
const NODE_HEADERS: [u64; 3] = [58, 67, 76];
/// D18 已定项 7 的两个预留位合计（nonce 代号 12 + MAC 16）。E117 的 `resv12` 臂同一个数。
const RESERVED_HEADER_BYTES: u64 = 12 + 16;
/// 写序（C113 定案 P1）。D18 已定项 7 补注：三档下界各加它。
const WRITE_ORDER: u64 = 10;
/// 今天成立的三档基础节点头。上面那组 58 / 67 / 76 是 E73 跑那天的下界，
/// 此后两笔加宽从来没落到本实验的源码里，2026-09-07 一次补上（C89 ④ 与 C193）：
/// 写序 10 + 预留位 28。两组都留着并逐格跑——删掉旧那组就看不出这次补账改了什么。
const NODE_HEADERS_TODAY: [u64; 3] = [
    NODE_HEADERS[0] + WRITE_ORDER + RESERVED_HEADER_BYTES,
    NODE_HEADERS[1] + WRITE_ORDER + RESERVED_HEADER_BYTES,
    NODE_HEADERS[2] + WRITE_ORDER + RESERVED_HEADER_BYTES,
];
/// E73 用的子指针宽度（它的参数）与 D22 已定项 7 的树表单元指针实宽。
const CHILD_POINTER_BYTES_E73: u64 = 32;
const CHILD_POINTER_BYTES_SETTLED: u64 = 53;
/// 记账 value：完整值取 8 字节（统计量是字节计数，256 TiB 也只用到 48 位）。
const ACCOUNTING_VALUE_BYTES: u64 = 8;

/// 一段字段：宽度、值域上界（指得到已定条款才有）、上界的出处。
#[derive(Debug, Clone, Copy)]
struct Segment {
    name: &'static str,
    bytes: u64,
    /// `None` = 上界指不到任何已定条款。判据 2 对它只报「取不到」，不判红也不判绿。
    domain_upper_bound: Option<u64>,
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

fn coverage(segment: &Segment) -> Coverage {
    match segment.domain_upper_bound {
        None => Coverage::UpperUnknown,
        Some(upper_bound) => {
            if capacity(segment.bytes) > upper_bound as u128 {
                Coverage::Covers
            } else {
                Coverage::Overflows
            }
        }
    }
}

fn key_bytes(segments: &[Segment]) -> u64 {
    segments.iter().map(|segment| segment.bytes).sum()
}

/// 扇出 = `(节点大小 − 节点头 − 区间字段) / 条目宽`。判据 1 的那个式子。
fn fanout(node_size_bytes: u64, header_bytes: u64, range_field_bytes: u64, entry_width_bytes: u64) -> u64 {
    if entry_width_bytes == 0 {
        return 0;
    }
    let overhead = header_bytes.saturating_add(range_field_bytes);
    if node_size_bytes <= overhead {
        return 0;
    }
    (node_size_bytes - overhead) / entry_width_bytes
}

/// 树高（含叶层）：叶层按叶扇出装，上面各层按内部扇出装。
fn tree_height(entry_count: u64, leaf_fanout: u64, inner_fanout: u64) -> Option<u64> {
    if leaf_fanout == 0 || inner_fanout < 2 {
        return None;
    }
    let mut height = 1u64;
    let mut covered_entries = leaf_fanout as u128;
    while covered_entries < entry_count as u128 {
        covered_entries = covered_entries.saturating_mul(inner_fanout as u128);
        height += 1;
        if height > 64 {
            return None; // 不收敛就报 None，不许挂死也不许返回一个数
        }
    }
    Some(height)
}

/// 一棵满树自己占多少字节：各层节点数之和 × 节点大小。
fn tree_bytes(entry_count: u64, leaf_fanout: u64, inner_fanout: u64) -> Option<u64> {
    let height = tree_height(entry_count, leaf_fanout, inner_fanout)?;
    let mut level_node_count = entry_count.div_ceil(leaf_fanout).max(1);
    let mut total_node_count = level_node_count;
    for _ in 1..height {
        level_node_count = level_node_count.div_ceil(inner_fanout).max(1);
        total_node_count += level_node_count;
    }
    Some(total_node_count.saturating_mul(NODE_BYTES))
}

/// 判据 3：把代编码成模 `modulus` 的环形计数器，代号跑到 `generation_count` 时的逆序对数。
/// `modulus == 0` 表示不取模（存完整 checkpoint 号）。
fn inversions_modulo(generation_count: u64, modulus: u64) -> u64 {
    let encode = |generation: u64| if modulus == 0 { generation } else { generation % modulus };
    let mut inversion_count = 0u64;
    for earlier_generation in 0..generation_count {
        for later_generation in (earlier_generation + 1)..generation_count {
            if encode(earlier_generation) > encode(later_generation) {
                inversion_count += 1;
            }
        }
    }
    inversion_count
}

/// 判据 3 的闭式：`generation_count` 个代按模 `modulus` 编码时的逆序对数。
/// 记 `full_periods = generation_count / modulus`（整周期数）、`tail_length = generation_count % modulus`：整周期两两之间贡献 `C(full_periods,2) × C(modulus,2)`；
/// 每个整周期与尾巴那 `tail_length` 个各贡献 `Σ_{v<tail_length}(modulus−1−v)`；周期内部与尾巴内部递增，贡献 0。
fn inversions_modulo_closed_form(generation_count: u64, modulus: u64) -> u64 {
    if modulus == 0 {
        return 0;
    }
    let full_periods = generation_count / modulus;
    let tail_length = generation_count % modulus;
    let pairs_within = modulus * (modulus - 1) / 2;
    let mut inversion_count = full_periods * (full_periods - 1) / 2 * pairs_within;
    // ⚠️ `tail_length * (tail_length - 1)` 在 r = 0 时先算 `tail_length - 1` 就下溢：release 下静默回绕
    // （0 × u64::MAX = 0，答案照样对），debug 下 panic。
    // 门禁跑的是 `cargo test --release`，所以这条 2026-09-07 之前一直没人看见。
    let tail = tail_length * (modulus - 1) - tail_length * tail_length.saturating_sub(1) / 2;
    inversion_count += full_periods * tail;
    inversion_count
}

/// 判据 6：一个 `w` 字节的落点字段能表达多少个**非单元对齐**的落点。
/// 按字节偏移编码：`2^(8w) − 2^(8w)/单元`；按单元号编码：恒 0。
fn misaligned_representable(bytes: u64, grain: u64, by_slot_number: bool) -> u128 {
    if by_slot_number {
        return 0;
    }
    let capacity_values = capacity(bytes);
    capacity_values - capacity_values / grain as u128
}

/// 「只分配了某个 32 KiB 数据单元的半边」这个非法状态，在按 `grain` 编码的坐标里可不可表示。
/// **只看坐标分辨率**：分辨率细到能单独点名半边，就写得出来——
/// 加不加跨度段、一条记一个槽还是记一个单元，都改变不了这一点，
/// 因为「跨度 = 1」必须永远合法（索引节点恰好就是一个 16 KiB 槽）。
fn half_unit_representable(grain: u64) -> bool {
    grain < DATA_UNIT_BYTES
}

/// **判据 7**：一个按粒度 `grain` 编码槽号的落点字段，在一段 `slots_of_16_kibibytes` 个 16 KiB 槽的区域里，
/// 表达得了多少个**已定条款允许的**落点起点，表达不了多少个。
///
/// D18 已定项 9 允许两类起点：数据单元按 32768 对齐、索引节点按 16384 对齐
/// ⇒ 合法起点就是全部 16 KiB 槽边界，共 `slots_of_16_kibibytes` 个。
/// 按 `grain` 编码槽号只表达得了 `grain` 的整数倍那些 ⇒ 可表达 = `⌈slots / (grain/16384)⌉`。
fn alignable(slots_of_16_kibibytes: u64, grain: u64) -> (u64, u64) {
    let step = (grain / NODE_ALIGNMENT_BYTES).max(1);
    let representable_count = slots_of_16_kibibytes.div_ceil(step);
    (representable_count, slots_of_16_kibibytes - representable_count)
}

/// **判据 8**：模 `modulus` 的代字段每多少**毫秒**回绕一次（`modulus == 0` ⇒ 不取模 ⇒ None）。
/// 口径是**发布率**不是 `T_time`：`ms = modulus × 1000 / 2785`（D5 已定项 2 骑手条款同一口径）。
fn wrap_period_milliseconds(modulus: u64) -> Option<u128> {
    if modulus == 0 {
        return None;
    }
    Some(modulus as u128 * 1000 / PUBLISHES_PER_SECOND as u128)
}

/// 一个 `w` 字节的落点字段能寻址多大的一块盘。
fn addressable_bytes(bytes: u64, grain: u64, by_slot_number: bool) -> u128 {
    let capacity_values = capacity(bytes);
    if by_slot_number {
        capacity_values.saturating_mul(grain as u128)
    } else {
        capacity_values
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
struct Statistic {
    per_device: bool,
    per_tree_narrow: bool,
}
const STATISTICS_TABLE: [Statistic; STATISTIC_COUNT as usize] = [
    Statistic { per_device: true,  per_tree_narrow: false }, // 1 已分配字节
    Statistic { per_device: true,  per_tree_narrow: false }, // 2 空闲字节
    Statistic { per_device: true,  per_tree_narrow: false }, // 3 不可回收字节
    Statistic { per_device: false, per_tree_narrow: false }, // 4 待删占用
    Statistic { per_device: true,  per_tree_narrow: false }, // 5 defer 待释放
    Statistic { per_device: false, per_tree_narrow: false }, // 6 已承诺预留
    Statistic { per_device: false, per_tree_narrow: false }, // 7 扩展点配额已用
    Statistic { per_device: false, per_tree_narrow: true  }, // 8 每树独占
    Statistic { per_device: false, per_tree_narrow: true  }, // 9 每树共享
    Statistic { per_device: false, per_tree_narrow: false }, // 10 碎片度 runs
    Statistic { per_device: true,  per_tree_narrow: false }, // 11 全空聚簇段数
];

/// 记账树条目数。`wide` = 宽读（十一项都按树分）。
fn accounting_entries(tree_count: u64, device_count: u64, kept_generations: u64, wide: bool) -> u64 {
    STATISTICS_TABLE
        .iter()
        .map(|statistic| {
            let tree_factor = if wide || statistic.per_tree_narrow { tree_count } else { 1 };
            let device_factor = if statistic.per_device { device_count } else { 1 };
            tree_factor * device_factor * kept_generations
        })
        .sum()
}

/// **判据 10**：一次发布脏掉多少字节——按**节点**算，不按条目字节算。
/// 记账树按 COW 挂在根下（D5 已定项 3 第 4 条）⇒ 一条插入脏一个完整节点。
/// 最坏口径：每个 `(统计量, 树, 设备)` 组每次发布各被触碰一次（与 E71 判据 2 同一口径），
/// 而同组的 K 代在 key 空间里连续 ⇒ 每叶装 `leaf_fanout / K` 组 ⇒ 组一被触碰，那片叶就脏。
fn dirty_bytes_per_publish(groups: u64, kept_generations: u64, leaf_fanout: u64) -> Option<u64> {
    if leaf_fanout == 0 || kept_generations == 0 {
        return None;
    }
    let groups_per_leaf = (leaf_fanout / kept_generations).max(1);
    let leaves = groups.div_ceil(groups_per_leaf);
    Some(leaves.saturating_mul(NODE_BYTES))
}

/// 分配记录条目的盘上宽度 = 设备身份 + 偏移 + 跨度段 + 分配代。
/// **抽成函数是为了让变异碰得到它**：写在 `main` 里的那一行没有任何单测看得见（实测踩过）。
fn allocation_entry_bytes(device_bytes: u64, offset_bytes: u64, span_bytes: u64, generation_bytes: u64) -> u64 {
    device_bytes + offset_bytes + span_bytes + generation_bytes
}

/// 分配记录条目数：已分配落点数（E74 口径）。
fn allocation_entries(capacity_bytes: u64, grain: u64, fill_percent: u64) -> u64 {
    (capacity_bytes / grain) * fill_percent / 100
}

fn coverage_label(coverage_verdict: Coverage) -> &'static str {
    match coverage_verdict {
        Coverage::Covers => "covers",
        Coverage::Overflows => "OVERFLOWS",
        Coverage::UpperUnknown => "upper_unknown",
    }
}

/// 记账 key 的四段。`gen_bytes == 1` 是「窗口代」那条臂。
fn accounting_key(statistic_bytes: u64, tree_identifier_bytes: u64, device_bytes: u64, generation_bytes: u64) -> [Segment; 4] {
    [
        // 上界 = 今天的统计量数（D5 已定项 4）。清单自陈不封闭，余量另报。
        Segment { name: "stat_tag", bytes: statistic_bytes, domain_upper_bound: Some(STATISTIC_COUNT) },
        // 树数上界指不到任何已定条款：D6 已定「每头一棵自己的树」，快照数无上界。
        Segment { name: "tree_id", bytes: tree_identifier_bytes, domain_upper_bound: None },
        // 设备数上界指不到：位置条目的 dev 宽度仓里没定（D19 已定项 1 只定了「带」）。
        Segment { name: "dev_id", bytes: device_bytes, domain_upper_bound: None },
        // 代的上界由 D22 已定项 7 的 checkpoint_txg 8 字节给。
        Segment { name: "gen", bytes: generation_bytes, domain_upper_bound: Some(u64::MAX) },
    ]
}

fn main() {
    let mut emitter = Emitter::new();
    let mut output_lines: Vec<String> = Vec::new();
    let capacity_16_tebibytes = 16u64 * 1024 * 1024 * 1024 * 1024;
    let capacity_256_tebibytes = 256u64 * 1024 * 1024 * 1024 * 1024;

    output_lines.push(emitter.emit_raw(&format!(
        "name=config node_bytes={NODE_BYTES} unit_bytes={DATA_UNIT_BYTES} stats={STATISTIC_COUNT} \
         k_max={KEPT_GENERATIONS_MAXIMUM} t_dirty={DIRTY_BYTES_THRESHOLD} budget={ACCOUNTING_WRITE_BUDGET_BYTES} ckpt_txg_bytes={CHECKPOINT_TRANSACTION_GROUP_BYTES}"
    )));

    // ── 记账树：四条臂 × 三档节点头 × 两种子指针宽 ──────────────────────
    let accounting_arms: [(&str, [Segment; 4]); 4] = [
        ("acct_e73_22B", accounting_key(2, 8, 4, 8)),
        ("acct_wide_19B", accounting_key(2, 8, 1, 8)),
        ("acct_tight_14B", accounting_key(1, 4, 1, 8)),
        ("acct_windowgen_7B", accounting_key(1, 4, 1, 1)),
    ];
    // 判据 2：逐段覆盖判定
    for (name, segments) in &accounting_arms {
        for segment in segments.iter() {
            output_lines.push(emitter.emit_raw(&format!(
                "name=coverage arm={name} seg={} bytes={} verdict={}",
                segment.name,
                segment.bytes,
                coverage_label(coverage(segment))
            )));
        }
    }
    // 判据 1 / 5：扇出与树高
    let accounting_entries_narrow = accounting_entries(64, 8, 25, false);
    let accounting_entries_wide = accounting_entries(64, 8, 25, true);
    for (name, segments) in &accounting_arms {
        let key_byte_count = key_bytes(segments);
        let leaf_entry = key_byte_count + ACCOUNTING_VALUE_BYTES;
        for &header_bytes in NODE_HEADERS.iter().chain(NODE_HEADERS_TODAY.iter()) {
            for &child_pointer_bytes in [CHILD_POINTER_BYTES_E73, CHILD_POINTER_BYTES_SETTLED].iter() {
                let inner_entry = key_byte_count + child_pointer_bytes;
                // D18 已定项 2：记账树的节点带 key 区间 ⇒ 区间字段 = 2 × key
                let leaf_fanout = fanout(NODE_BYTES, header_bytes, 2 * key_byte_count, leaf_entry);
                let inner_fanout = fanout(NODE_BYTES, header_bytes, 2 * key_byte_count, inner_entry);
                let height_narrow = tree_height(accounting_entries_narrow, leaf_fanout, inner_fanout);
                let height_wide = tree_height(accounting_entries_wide, leaf_fanout, inner_fanout);
                output_lines.push(emitter.emit_raw(&format!(
                    "name=acct_geom arm={name} key_bytes={key_byte_count} leaf_entry={leaf_entry} \
                     inner_entry={inner_entry} header={header_bytes} child_ptr={child_pointer_bytes} \
                     leaf_fanout={leaf_fanout} inner_fanout={inner_fanout} \
                     entries_narrow={accounting_entries_narrow} height_narrow={} \
                     entries_wide={accounting_entries_wide} height_wide={}",
                    height_narrow.map(|height| height.to_string()).unwrap_or_else(|| "NA".into()),
                    height_wide.map(|height| height.to_string()).unwrap_or_else(|| "NA".into()),
                )));
            }
        }
    }
    // 判据 4：一次 checkpoint 的记账写字节。D5 已定项 2：每 checkpoint 一写一删 ⇒ ×2。
    for (name, segments) in &accounting_arms {
        let key_byte_count = key_bytes(segments);
        let accounting_entry_bytes = key_byte_count + ACCOUNTING_VALUE_BYTES;
        let leaf_fanout = fanout(NODE_BYTES, NODE_HEADERS[0], 2 * key_byte_count, accounting_entry_bytes);
        for &(tree_count, device_count) in [(64u64, 8u64), (1024, 64), (8192, 8), (8192, 64)].iter() {
            for &wide in [false, true].iter() {
                let groups = accounting_entries(tree_count, device_count, 1, wide);
                // 旧口径：条目字节（一写一删 ⇒ ×2）。**系统性低估**，留着做对照。
                let entry_bytes = groups * 2 * accounting_entry_bytes;
                // 判据 10 的新口径：脏叶字节
                let node_bytes = dirty_bytes_per_publish(groups, 25, leaf_fanout);
                output_lines.push(emitter.emit_raw(&format!(
                    "name=acct_budget arm={name} trees={tree_count} devs={device_count} reading={} entry={accounting_entry_bytes} \
                     leaf_fanout={leaf_fanout} groups={groups} entry_bytes={entry_bytes} \
                     over_by_entry={} node_bytes={} over_by_node={}",
                    if wide { "wide" } else { "narrow" },
                    u8::from(entry_bytes > ACCOUNTING_WRITE_BUDGET_BYTES),
                    node_bytes.map(|dirty_bytes| dirty_bytes.to_string()).unwrap_or_else(|| "NA".into()),
                    node_bytes.map(|dirty_bytes| u8::from(dirty_bytes > ACCOUNTING_WRITE_BUDGET_BYTES).to_string()).unwrap_or_else(|| "NA".into()),
                )));
            }
        }
    }

    // 判据 4 的反解：宽读那一侧，key 最宽能到几字节还不撑破 T_dirty 的 1%
    for &(tree_count, device_count) in [(64u64, 8u64), (1024, 64), (8192, 8), (8192, 64)].iter() {
        for &wide in [false, true].iter() {
            let touched = accounting_entries(tree_count, device_count, 1, wide) * 2;
            let maximum_entry_bytes = ACCOUNTING_WRITE_BUDGET_BYTES / touched;
            let maximum_key_bytes = maximum_entry_bytes.saturating_sub(ACCOUNTING_VALUE_BYTES);
            output_lines.push(emitter.emit_raw(&format!(
                "name=acct_key_ceiling trees={tree_count} devs={device_count} reading={} touched_entries={touched} \
                 budget={ACCOUNTING_WRITE_BUDGET_BYTES} max_entry_bytes={maximum_entry_bytes} max_key_bytes={maximum_key_bytes} feasible={}",
                if wide { "wide" } else { "narrow" },
                u8::from(maximum_entry_bytes > ACCOUNTING_VALUE_BYTES),
            )));
        }
    }

    // ── 分配记录树：四条臂 ────────────────────────────────────────────
    // (名字, dev 宽, 偏移宽, 按槽号编码, 编码粒度, 代宽, 跨度段宽, 一条记一个单元)
    // ⚠️ **第三轮把粒度做成参数**：第一、二版把它硬编成 32768，
    // 于是提案采纳的 16 KiB 那条编码从来没有一条完整的臂（第二轮反推攻击腿指出）。
    let allocation_arms: [(&str, u64, u64, bool, u64, u64, u64, bool); 7] = [
        ("alloc_ptrcoord_15B",  1, 6, false, DATA_UNIT_BYTES, 8, 0, false),
        ("alloc_e74_20B",       4, 8, false, DATA_UNIT_BYTES, 8, 0, false),
        ("alloc_unitno32k_14B", 1, 5, true,  DATA_UNIT_BYTES, 8, 0, false),
        // 与 alloc_ptrcoord_15B **同宽**，只换编码 —— 判据 6 的对照臂
        ("alloc_unitno32k_15B", 1, 6, true,  DATA_UNIT_BYTES, 8, 0, false),
        ("alloc_windowgen_7B",  1, 5, true,  DATA_UNIT_BYTES, 1, 0, false),
        // **第三轮补：提案实际采纳的那条**——16 KiB 槽号，一槽一条
        ("alloc_slot16k_15B",   1, 6, true,  NODE_ALIGNMENT_BYTES,      8, 0, false),
        // **第三轮补：第二轮反推攻击腿开出的替代**——16 KiB 槽号 + 1 字节跨度段，
        // **一个单元一条**：半个单元被分配这个非法状态因此不可表示
        ("alloc_span16k_16B",   1, 6, true,  NODE_ALIGNMENT_BYTES,      8, 1, true),
    ];
    for &(name, device_bytes, offset_bytes, by_slot_number, grain, generation_bytes, span_bytes, per_unit) in allocation_arms.iter() {
        let key_byte_count = device_bytes + offset_bytes;
        let leaf_entry = allocation_entry_bytes(device_bytes, offset_bytes, span_bytes, generation_bytes);
        let generation_segment = Segment { name: "gen", bytes: generation_bytes, domain_upper_bound: Some(u64::MAX) };
        // 判据 6 与寻址范围
        output_lines.push(emitter.emit_raw(&format!(
            "name=alloc_encoding arm={name} dev_bytes={device_bytes} off_bytes={offset_bytes} \
             by_slot_number={} grain={grain} gen_bytes={generation_bytes} span_bytes={span_bytes} \
             per_unit_record={} entry={leaf_entry} gen_coverage={} \
             misaligned_representable={} addressable_bytes={} half_unit_representable={}",
            u8::from(by_slot_number),
            u8::from(per_unit),
            coverage_label(coverage(&generation_segment)),
            misaligned_representable(offset_bytes, grain, by_slot_number),
            addressable_bytes(offset_bytes, grain, by_slot_number),
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
            let range_field_bytes = if with_range { 2 * key_byte_count } else { 0 };
            for &header_bytes in NODE_HEADERS.iter() {
                let leaf_fanout = fanout(NODE_BYTES, header_bytes, range_field_bytes, leaf_entry);
                let inner_fanout = fanout(NODE_BYTES, header_bytes, range_field_bytes, key_byte_count + CHILD_POINTER_BYTES_SETTLED);
                for &capacity_bytes in [capacity_16_tebibytes, capacity_256_tebibytes].iter() {
                    // 一单元一条 ⇒ 条目数按单元数；一槽一条 ⇒ 按编码粒度数
                    let entry_count = allocation_entries(capacity_bytes, if per_unit { DATA_UNIT_BYTES } else { grain }, 90);
                    let maybe_height = tree_height(entry_count, leaf_fanout, inner_fanout);
                    let maybe_tree_bytes = tree_bytes(entry_count, leaf_fanout, inner_fanout);
                    let occupancy_parts_per_million = maybe_tree_bytes.map(|tree_byte_count| tree_byte_count as u128 * 1_000_000 / capacity_bytes as u128);
                    output_lines.push(emitter.emit_raw(&format!(
                        "name=alloc_geom arm={name} with_range={} header={header_bytes} cap_bytes={capacity_bytes} \
                         entries={entry_count} leaf_fanout={leaf_fanout} inner_fanout={inner_fanout} height={} \
                         tree_bytes={} occupancy_ppm={}",
                        u8::from(with_range),
                        maybe_height.map(|height| height.to_string()).unwrap_or_else(|| "NA".into()),
                        maybe_tree_bytes.map(|tree_byte_count| tree_byte_count.to_string()).unwrap_or_else(|| "NA".into()),
                        occupancy_parts_per_million.map(|parts_per_million| parts_per_million.to_string()).unwrap_or_else(|| "NA".into()),
                    )));
                }
            }
        }
        // 整理的 COW 放大：同样搬 N 条，条目越窄放大越大（E74 口径）
        let leaf_fanout = fanout(NODE_BYTES, NODE_HEADERS[0], 0, leaf_entry);
        let moved_entry_count = 4096u64;
        let leaves_contiguous = moved_entry_count.div_ceil(leaf_fanout.max(1));
        let amplification_contiguous = leaves_contiguous as u128 * NODE_BYTES as u128
            / (moved_entry_count as u128 * leaf_entry as u128);
        let amplification_scattered = moved_entry_count as u128 * NODE_BYTES as u128
            / (moved_entry_count as u128 * leaf_entry as u128);
        output_lines.push(emitter.emit_raw(&format!(
            "name=alloc_cow arm={name} entry={leaf_entry} leaf_fanout={leaf_fanout} \
             leaves_contiguous={leaves_contiguous} amp_contiguous={amplification_contiguous} amp_scattered={amplification_scattered}"
        )));
        // 判据 9：段清单完整性
        if name == allocation_arms[0].0 {
            for requirement in REQUIRED.iter() {
                output_lines.push(emitter.emit_raw(&format!(
                    "name=segment_required clause={} field={} present={}",
                    requirement.clause,
                    requirement.field,
                    u8::from(requirement.present)
                )));
            }
        }
    }

    // ── 判据 3 + 8：代的环形编码。**代数必须扫到模之上**，否则等于给大模发免检证。
    for &modulus in [0u64, KEPT_GENERATIONS_MAXIMUM, 256, 1024, 65536].iter() {
        for &generation_count in [32u64, 64, 257, 300, 512, 70000].iter() {
            // 逆序对是 O(gens²)，7 万代那格用闭式，别拿 49 亿次循环去跑
            let inversion_count = if generation_count > 4096 {
                inversions_modulo_closed_form(generation_count, modulus)
            } else {
                inversions_modulo(generation_count, modulus)
            };
            output_lines.push(emitter.emit_raw(&format!(
                "name=gen_order modulus={modulus} gens={generation_count} inversions={inversion_count} wrap_millis={}",
                wrap_period_milliseconds(modulus).map(|milliseconds| milliseconds.to_string()).unwrap_or_else(|| "never".into())
            )));
        }
    }

    // ── 判据 7：对齐可表示性。D18 已定项 9 允许的落点起点 = 全部 16 KiB 槽边界。
    for &slot_count_of_16_kibibytes in [4096u64, 998].iter() {
        for &grain in [NODE_ALIGNMENT_BYTES, DATA_UNIT_BYTES].iter() {
            let (representable_count, unrepresentable_count) = alignable(slot_count_of_16_kibibytes, grain);
            output_lines.push(emitter.emit_raw(&format!(
                "name=alignable slots_16k={slot_count_of_16_kibibytes} grain={grain} representable={representable_count} unrepresentable={unrepresentable_count}"
            )));
        }
    }

    // ── 落点粒度这一维：条目数与占容量按粒度反比走（E74 扫过三档，第一版 E97 硬编成一档）
    for &grain in [NODE_ALIGNMENT_BYTES, DATA_UNIT_BYTES].iter() {
        for &capacity_bytes in [capacity_16_tebibytes, capacity_256_tebibytes].iter() {
            let entry_count = allocation_entries(capacity_bytes, grain, 90);
            let leaf_fanout = fanout(NODE_BYTES, 58, 0, 15);
            let inner_fanout = fanout(NODE_BYTES, 58, 0, 7 + CHILD_POINTER_BYTES_SETTLED);
            let maybe_height = tree_height(entry_count, leaf_fanout, inner_fanout);
            let maybe_tree_bytes = tree_bytes(entry_count, leaf_fanout, inner_fanout);
            output_lines.push(emitter.emit_raw(&format!(
                "name=grain_sweep grain={grain} cap_bytes={capacity_bytes} entries={entry_count} \
                 leaf_fanout={leaf_fanout} inner_fanout={inner_fanout} height={} tree_bytes={} occupancy_ppm={}",
                maybe_height.map(|height| height.to_string()).unwrap_or_else(|| "NA".into()),
                maybe_tree_bytes.map(|tree_byte_count| tree_byte_count.to_string()).unwrap_or_else(|| "NA".into()),
                maybe_tree_bytes.map(|tree_byte_count| (tree_byte_count as u128 * 1_000_000 / capacity_bytes as u128).to_string()).unwrap_or_else(|| "NA".into()),
            )));
        }
    }

    // ── 阳性对照：每条臂每一段各砍成 0 ────────────────────────────────
    for (name, segments) in &accounting_arms {
        let full_key_bytes = key_bytes(segments);
        for segment_index in 0..segments.len() {
            let mut cut_segments = *segments;
            let was_bytes = cut_segments[segment_index].bytes;
            cut_segments[segment_index].bytes = 0;
            let shrunk_key_bytes = key_bytes(&cut_segments);
            output_lines.push(emitter.emit_raw(&format!(
                "name=control_cut arm={name} seg={} was={was_bytes} key_full={full_key_bytes} key_cut={shrunk_key_bytes} \
                 delta_ok={} verdict_after={}",
                cut_segments[segment_index].name,
                u8::from(full_key_bytes - shrunk_key_bytes == was_bytes),
                coverage_label(coverage(&cut_segments[segment_index])),
            )));
        }
    }

    for output_line in &output_lines {
        println!("{output_line}");
    }
    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 格式常量必须与 kb 的 format-const 标记一致。
    #[test]
    fn format_constants_match_knowledge_base() {
        assert_eq!(NODE_BYTES, 16384, "D8 已定项 2");
        assert_eq!(DATA_UNIT_BYTES, 32768, "D4 已定项 5");
        assert_eq!(STATISTIC_COUNT, 11, "D5 已定项 4 第一次重开之后");
        assert_eq!(KEPT_GENERATIONS_MAXIMUM, 49, "D22 已定项 2：3 × 16 + 1");
        assert_eq!(CHECKPOINT_TRANSACTION_GROUP_BYTES, 8, "D22 已定项 7");
    }

    /// **判据 1 的绝对值**：22 字节 key、头 58、带区间、子指针 32
    /// ⇒ 内部扇出恰好 301 —— 与 E73 那张表逐格相同，说明两个模型接得上。
    #[test]
    fn criterion1_absolute_fanout_reproduces_e73() {
        let key_byte_count = 22u64;
        let inner_fanout_value = fanout(NODE_BYTES, 58, 2 * key_byte_count, key_byte_count + CHILD_POINTER_BYTES_E73);
        assert_eq!(key_byte_count + CHILD_POINTER_BYTES_E73, 54, "E73 的条目宽");
        assert_eq!(58 + 2 * key_byte_count, 102, "带区间的头");
        assert_eq!(NODE_BYTES - 102, 16282);
        assert_eq!(inner_fanout_value, 301, "16282 / 54 = 301，与 E73 的 301 逐格相同");
        // 不带区间那一侧 E73 报 302
        assert_eq!(fanout(NODE_BYTES, 58, 0, 54), 302);
    }

    /// **判据 1 的绝对值（分配记录侧）**：E74 的 20 字节条目、头 64、不带区间
    /// ⇒ 扇出恰好 816，与 E74 那张表相同。
    #[test]
    fn criterion1_absolute_fanout_reproduces_e74() {
        assert_eq!(fanout(NODE_BYTES, 64, 0, 20), 816);
        assert_eq!(allocation_entries(16 * 1024 * 1024 * 1024 * 1024, DATA_UNIT_BYTES, 90), 483_183_820);
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
        let one_byte_segment = Segment { name: "x", bytes: 1, domain_upper_bound: Some(STATISTIC_COUNT) };
        let zero_byte_segment = Segment { name: "x", bytes: 0, domain_upper_bound: Some(STATISTIC_COUNT) };
        assert_eq!(coverage(&one_byte_segment), Coverage::Covers);
        assert_eq!(coverage(&zero_byte_segment), Coverage::Overflows);
        // 8 字节的代要装下 u64::MAX ⇒ 2^64 > 2^64−1，恰好过；7 字节过不了
        let eight_byte_generation_segment = Segment { name: "gen", bytes: 8, domain_upper_bound: Some(u64::MAX) };
        let seven_byte_generation_segment = Segment { name: "gen", bytes: 7, domain_upper_bound: Some(u64::MAX) };
        assert_eq!(coverage(&eight_byte_generation_segment), Coverage::Covers);
        assert_eq!(coverage(&seven_byte_generation_segment), Coverage::Overflows);
        // 上界指不到的段不许被判成「过了」
        let unknown_upper_segment = Segment { name: "tree_id", bytes: 1, domain_upper_bound: None };
        assert_eq!(coverage(&unknown_upper_segment), Coverage::UpperUnknown);
    }

    /// **判据 3 + 阳性对照二**：不取模恒 0；模小于代数时逆序对是可数的绝对值。
    #[test]
    fn criterion3_wrapped_generation_breaks_key_order() {
        assert_eq!(inversions_modulo(64, 0), 0, "存完整 checkpoint 号：零逆序");
        assert_eq!(inversions_modulo(32, 64), 0, "模大于代数：零逆序（阴性对照）");
        // 模 16、32 代 ⇒ 每个后半段元素与前半段同余的更大者构成逆序
        // 手算：32 代模 16 = 2 个整周期 ⇒ C(2,2) × C(16,2) = 1 × 120
        assert_eq!(inversions_modulo(32, 16), 120, "32 代模 16：恰好 120 对");
        // 手算：64 代模 16 = 4 个整周期 ⇒ C(4,2) × C(16,2) = 6 × 120 = 720
        assert_eq!(inversions_modulo(64, 16), 720, "64 代模 16：恰好 720 对");
        // 手算：64 代模 49 = 一个满周期 0..48 加一段 0..14
        // ⇒ Σ_{j=49..63}(97−j) = Σ_{k=34..48} k = (34+48)×15/2 = 615
        assert_eq!(inversions_modulo(64, KEPT_GENERATIONS_MAXIMUM), 615, "K=49 也不够：仍有 615 对");
        // 单调：模越小逆序越多
        assert!(inversions_modulo(64, 16) > inversions_modulo(64, 64));
    }

    /// **判据 6 的绝对值**：按字节偏移能表达大量非对齐落点，按单元号恒 0。
    #[test]
    fn criterion6_unit_number_makes_misaligned_unrepresentable() {
        // 6 字节字节偏移：2^48 个值里只有 2^48/32768 = 2^33 个是单元对齐的
        let misaligned_count = misaligned_representable(6, DATA_UNIT_BYTES, false);
        // 手算：2^48 − 2^33 = 281 474 976 710 656 − 8 589 934 592
        assert_eq!(misaligned_count, (1u128 << 48) - (1u128 << 33));
        assert_eq!(misaligned_count, 281_466_386_776_064);
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
    fn copy_on_write_amplification_is_a_ratio_not_an_absolute_cost() {
        let moved_entry_count = 4096u64;
        let fanout_for_15_byte_entry = fanout(NODE_BYTES, 58, 0, 15);
        let fanout_for_7_byte_entry = fanout(NODE_BYTES, 58, 0, 7);
        assert_eq!(fanout_for_15_byte_entry, 1088, "手算 (16384−58)/15 = 16326/15 = 1088.4 ⇒ 1088");
        assert_eq!(fanout_for_7_byte_entry, 2332, "手算 16326/7 = 2332.28 ⇒ 2332");
        // 连续：窄条目碰的叶更少（绝对量更小）
        assert_eq!(moved_entry_count.div_ceil(fanout_for_15_byte_entry), 4);
        assert_eq!(moved_entry_count.div_ceil(fanout_for_7_byte_entry), 2);
        // 打散：两档写的节点数都是 N，与条目宽无关
        assert_eq!(moved_entry_count, 4096);
        // 而「放大倍数」却随条目变窄而变大 —— 变的是分母
        assert_eq!(NODE_BYTES / 15, 1092);
        assert_eq!(NODE_BYTES / 7, 2340);
    }

    /// **判据 5 单调**：每加一字节，条目加一、扇出不增、树高不减。
    #[test]
    fn criterion5_is_monotone_in_every_segment() {
        let mut previous_fanout = u64::MAX;
        let mut previous_height = 0u64;
        for key_byte_count in 7..=24u64 {
            let leaf_fanout_value = fanout(NODE_BYTES, 58, 2 * key_byte_count, key_byte_count + ACCOUNTING_VALUE_BYTES);
            let height = tree_height(1_000_000, leaf_fanout_value, leaf_fanout_value).unwrap();
            assert!(leaf_fanout_value <= previous_fanout, "扇出必须单调不增：{key_byte_count}");
            assert!(height >= previous_height, "树高必须单调不减：{key_byte_count}");
            previous_fanout = leaf_fanout_value;
            previous_height = height;
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
        assert_eq!(accounting_entries(64, 8, 25, false), 4300);
        // 宽读（十一项都按树分）
        assert_eq!(accounting_entries(64, 8, 25, true), 11 * 64 * 25 + 5 * 64 * 8 * 25 - 5 * 64 * 25);
        // 上界公式 s × t × d × K
        assert_eq!(STATISTIC_COUNT * 64 * 8 * 25, 140_800);
        assert!(accounting_entries(64, 8, 25, true) < STATISTIC_COUNT * 64 * 8 * 25, "宽读仍低于上界");
    }

    /// **判据 4**：预算线是绝对值，且窄读在扫过的三格都不过界。
    #[test]
    fn criterion4_budget_absolute() {
        assert_eq!(ACCOUNTING_WRITE_BUDGET_BYTES, 21_474_836);
        for &(tree_count, device_count) in [(64u64, 8u64), (1024, 64), (8192, 8)].iter() {
            let bytes = accounting_entries(tree_count, device_count, 1, false) * 2 * (19 + ACCOUNTING_VALUE_BYTES);
            assert!(bytes <= ACCOUNTING_WRITE_BUDGET_BYTES, "窄读 t={tree_count} d={device_count} 不该过界：{bytes}");
        }
        // 阳性对照：宽读在 t×d 很大那一格必须过界，否则这条闸分不出差别
        let wide = accounting_entries(8192, 64, 1, true) * 2 * (19 + ACCOUNTING_VALUE_BYTES);
        assert!(wide > ACCOUNTING_WRITE_BUDGET_BYTES, "宽读该过界却没过：{wide}");
    }

    /// **树高与占容量比是绝对值**，不是「大致这么多」。
    #[test]
    fn allocation_tree_occupancy_is_absolute() {
        let entry_count = allocation_entries(16 * 1024 * 1024 * 1024 * 1024, DATA_UNIT_BYTES, 90);
        let leaf_fanout = fanout(NODE_BYTES, 64, 0, 20);
        let inner_fanout = fanout(NODE_BYTES, 64, 0, 12 + CHILD_POINTER_BYTES_SETTLED);
        assert_eq!(leaf_fanout, 816, "手算 (16384−64)/20 = 816");
        assert_eq!(inner_fanout, 251, "手算 (16384−64)/(12+53) = 16320/65 = 251.0… ⇒ 251");
        assert_eq!(tree_height(entry_count, leaf_fanout, inner_fanout), Some(4));
        let tree_byte_count_value = tree_bytes(entry_count, leaf_fanout, inner_fanout).unwrap();
        // 手算各层节点数：叶 ⌈483183820/816⌉ = 592138、2360、10、1 ⇒ 594509 个节点
        assert_eq!(entry_count.div_ceil(leaf_fanout), 592_138);
        assert_eq!(tree_byte_count_value / NODE_BYTES, 594_509);
        assert_eq!(tree_byte_count_value, 594_509 * 16_384);
        assert_eq!(tree_byte_count_value, 9_740_435_456);
        // E74 报 9.0 GiB —— 它整棵树都按 leaf_fanout 乘上去，本模型内部层用内部扇出，故略大
        assert_eq!(tree_byte_count_value as u128 * 1_000_000 / (16u128 * 1024 * 1024 * 1024 * 1024), 553);
    }

    /// **叶条目 = key + 完整值**，且这一档的扇出是绝对值，不是「差不多」。
    #[test]
    fn accounting_leaf_entry_is_key_plus_value() {
        assert_eq!(ACCOUNTING_VALUE_BYTES, 8, "统计量是字节计数，256 TiB 也只用到 48 位");
        let segments = accounting_key(2, 8, 1, 8);
        let key_byte_count = key_bytes(&segments);
        assert_eq!(key_byte_count, 19);
        let leaf = key_byte_count + ACCOUNTING_VALUE_BYTES;
        assert_eq!(leaf, 27, "19 字节 key + 8 字节完整值");
        // 手算：(16384 − 58 − 2×19) / 27 = 16288 / 27 = 603.2… ⇒ 603
        assert_eq!(fanout(NODE_BYTES, 58, 2 * key_byte_count, leaf), 603);
        assert_eq!(CHILD_POINTER_BYTES_E73, 32, "E73 的参数");
        assert_eq!(CHILD_POINTER_BYTES_SETTLED, 53, "D22 已定项 7 的树表单元指针实宽");
    }

    /// **判据 4 的反解**：宽读下 key 宽度有一个硬上限，而 E73 假设的 22 恰好越过它。
    #[test]
    fn criterion4_wide_reading_puts_a_ceiling_on_key_bytes() {
        let touched = accounting_entries(8192, 8, 1, true) * 2;
        // 手算：宽读 ⇒ 11 项都按树分；5 项带设备维
        // ⇒ 5×8192×8 + 6×8192 = 327680 + 49152 = 376832，一写一删 ⇒ ×2
        assert_eq!(touched, 753_664);
        assert_eq!(ACCOUNTING_WRITE_BUDGET_BYTES / touched, 28, "21474836 / 753664 = 28.49… ⇒ 28");
        assert_eq!(28 - ACCOUNTING_VALUE_BYTES, 20, "key 最宽 20 字节");
        assert!(touched * (22 + ACCOUNTING_VALUE_BYTES) > ACCOUNTING_WRITE_BUDGET_BYTES, "E73 的 22 字节越界");
        assert!(touched * (20 + ACCOUNTING_VALUE_BYTES) <= ACCOUNTING_WRITE_BUDGET_BYTES, "20 字节不越界");
    }

    /// **判据 7（第二轮加）**：按 32 KiB 槽号编码表达不了半数索引节点的落点。
    /// 依据 D18 已定项 9 逐字「数据单元对齐 32768、索引节点对齐 16384」。
    #[test]
    fn criterion7_a_32k_slot_number_cannot_address_half_the_index_nodes() {
        assert_eq!(NODE_ALIGNMENT_BYTES, 16384, "D8 已定项 2 + D18 已定项 9");
        // 按 16 KiB 槽号编码：D18 已定项 9 允许的起点全部表达得了
        assert_eq!(alignable(4096, NODE_ALIGNMENT_BYTES), (4096, 0));
        // 按 32 KiB 单元号编码：只表达得了偶数槽 ⇒ 4096 个里有 2048 个表达不了
        assert_eq!(alignable(4096, DATA_UNIT_BYTES), (2048, 2048));
        // E86 那个 998 节点的区域：奇数槽那一半同样表达不了（E86 实测 496 个在奇槽，
        // 这里数的是「一段连续 998 槽里表达不了几个」，两个数不是同一个口径）
        assert_eq!(alignable(998, DATA_UNIT_BYTES), (499, 499));
        // 阳性对照：粒度等于对齐时表达不了的必须恰好是 0
        assert_eq!(alignable(998, NODE_ALIGNMENT_BYTES).1, 0);
    }

    /// **判据 8（第二轮加）**：代字段取模，回绕期由 `T_time` 直接给。
    /// 第一版只测 32 / 64 代 ⇒ 模 256 逆序对恒 0，等于给 1 字节的代发了免检证。
    #[test]
    fn criterion8_a_wrapped_generation_survives_only_until_it_wraps() {
        assert_eq!(PUBLISHES_PER_SECOND, 2785, "E44 本机实测的发布率");
        assert_eq!(GENERATION_BITS_FLOOR, 48, "D5 已定项 2 骑手条款：代段 ≥ 48 位");
        // 1 字节的代（模 256）：**91 毫秒**就绕完一圈（2785 发布/秒）
        assert_eq!(wrap_period_milliseconds(256), Some(91));
        // 2 字节（模 65536）：23.5 秒
        assert_eq!(wrap_period_milliseconds(65536), Some(23_531));
        // **交叉复现 D5 已定项 2 的骑手条款**：32 位 17.8 天
        let wrap_days_32_bit = wrap_period_milliseconds(1u64 << 32).unwrap() / 1000 / 86400;
        assert_eq!(wrap_days_32_bit, 17, "1542178 秒 = 17.8 天，与骑手条款「32 位 17.8 天回绕」同一个数");
        // 48 位（骑手条款的下界）：3204 年，与 E44 的「48 位撑 3202 年」同量级
        let wrap_years_48_bit = wrap_period_milliseconds(1u64 << GENERATION_BITS_FLOOR).unwrap() / 1000 / 86400 / 365;
        assert_eq!(wrap_years_48_bit, 3204);
        assert_eq!(wrap_period_milliseconds(0), None, "不取模就不回绕");
        // 第一版只测到 64 代 ⇒ 模 256 一个逆序对都没有，这正是免检证
        assert_eq!(inversions_modulo(64, 256), 0);
        // 越过模之后立刻出现：257 代 ⇒ 255 对
        assert_eq!(inversions_modulo(257, 256), 255);
        // 闭式与暴力法在能暴力的格子上必须逐格相同（校验路径，两条不共享代码）
        for &(generation_count, modulus) in [(32u64, 16u64), (64, 16), (64, 49), (257, 256), (300, 256), (512, 256), (64, 0)].iter() {
            assert_eq!(inversions_modulo(generation_count, modulus), inversions_modulo_closed_form(generation_count, modulus), "g={generation_count} m={modulus}");
        }
        // 手算：512 代模 256 = 2 个整周期 ⇒ C(2,2) × C(256,2) = 32640
        assert_eq!(inversions_modulo_closed_form(512, 256), 32_640);
        // 手算：300 代模 256 ⇒ 尾巴 44 个，Σ_{v<44}(255−v) = 44×255 − 44×43/2 = 10274
        assert_eq!(inversions_modulo_closed_form(300, 256), 10_274);
        // 2 字节的代同样会绕：70000 代模 65536 ⇒ 尾巴 4464 个
        let tail = 4464u64 * 65535 - 4464 * 4463 / 2;
        assert_eq!(inversions_modulo_closed_form(70_000, 65_536), tail);
        assert!(tail > 0, "任何有限模在活得够久之后都会破 key 序");
    }

    /// **判据 4 的反解（第二轮补格）**：宽读在 t=8192 / d=64 那格**无可行点**——
    /// 连 7 字节的臂都过界。这是 E97 跑前写死的反向接受条款该触发的地方。
    #[test]
    fn criterion4_wide_reading_has_no_feasible_point_at_8192x64() {
        let touched = accounting_entries(8192, 64, 1, true) * 2;
        // 手算：宽读 ⇒ 11 项都按树分；5 项带设备维
        // ⇒ 5×8192×64 + 6×8192 = 2621440 + 49152 = 2670592，一写一删 ⇒ ×2
        assert_eq!(touched, 5_341_184);
        assert_eq!(ACCOUNTING_WRITE_BUDGET_BYTES / touched, 4, "21474836 / 5341184 = 4.02… ⇒ 4");
        assert!(4 < ACCOUNTING_VALUE_BYTES, "条目连 value 都装不下 ⇒ 无可行 key 宽度");
        // 窄读同一格照样可行
        let narrow = accounting_entries(8192, 64, 1, false) * 2;
        // 手算：窄读 ⇒ 只有第 8、9 项按树分（2×8192）、5 项带设备维（5×64）、
        // 其余 4 项无维（4×1）⇒ 16384 + 320 + 4 = 16708，一写一删 ⇒ ×2
        assert_eq!(narrow, 33_416);
        assert!(ACCOUNTING_WRITE_BUDGET_BYTES / narrow > 8 + 22, "窄读连 22 字节 key 都装得下，余量还很大");
    }

    /// **落点粒度这一维（第二轮加）**：条目数按粒度反比走，占容量跟着翻倍。
    #[test]
    fn grain_halving_doubles_the_allocation_record_tree() {
        let capacity_bytes = 16u64 * 1024 * 1024 * 1024 * 1024;
        let entries_at_32_kibibyte_grain = allocation_entries(capacity_bytes, DATA_UNIT_BYTES, 90);
        let entries_at_16_kibibyte_grain = allocation_entries(capacity_bytes, NODE_ALIGNMENT_BYTES, 90);
        assert_eq!(entries_at_32_kibibyte_grain, 483_183_820);
        assert_eq!(entries_at_16_kibibyte_grain, 966_367_641);
        assert_eq!(entries_at_16_kibibyte_grain - 2 * entries_at_32_kibibyte_grain, 1, "整数除法取整差 1，不是模型错");
    }

    /// **判据 9（第三轮加）**：段清单里恰好缺一段——D8 write buffer 硬要求 3 的 `seq`。
    /// 判据 2 看不见它，因为判据 2 只查**已经列出来**的段。
    #[test]
    fn criterion9_the_segment_list_is_missing_exactly_one_settled_field() {
        let missing: Vec<&str> = REQUIRED.iter().filter(|requirement| !requirement.present).map(|requirement| requirement.field).collect();
        assert_eq!(missing, vec!["seq"], "seq 是唯一没有落点的已定字段");
        assert_eq!(REQUIRED.len(), 6);
        assert_eq!(REQUIRED.iter().filter(|requirement| requirement.present).count(), 5);
        // 阳性对照：判据 2 对一段**不在清单里**的字段一个字也说不出来——
        // 它只对 Seg 数组里的元素工作，而 seq 根本不在那个数组里。
        let segments = accounting_key(2, 8, 1, 8);
        assert_eq!(segments.len(), 4, "四段：统计量 / 树 / 设备 / 代，没有 seq");
        assert!(!segments.iter().any(|segment| segment.name == "seq"));
    }

    /// **判据 10（第三轮加）**：预算按脏节点算之后，两格从「不过界」翻成「过界」。
    /// 记账树按 COW 挂在根下 ⇒ 一条 27 字节插入脏一个 16 KiB 节点。
    #[test]
    fn criterion10_node_granularity_flips_two_grids_over_budget() {
        let entry = 19 + ACCOUNTING_VALUE_BYTES; // 27
        let leaf_fanout = fanout(NODE_BYTES, 58, 2 * 19, entry);
        assert_eq!(leaf_fanout, 603);
        // 每叶装 603 / 25 = 24 组（同组的 K 代在 key 空间里连续）
        assert_eq!(leaf_fanout / 25, 24);

        // t=8192 d=8 宽读：条目字节口径不过界，节点口径过界
        let wide_groups = accounting_entries(8192, 8, 1, true);
        assert_eq!(wide_groups, 376_832);
        assert_eq!(wide_groups * 2 * entry, 20_348_928);
        assert!(wide_groups * 2 * entry <= ACCOUNTING_WRITE_BUDGET_BYTES, "条目字节口径：不过界");
        let dirty_node_bytes = dirty_bytes_per_publish(wide_groups, 25, leaf_fanout).unwrap();
        assert_eq!(wide_groups.div_ceil(24), 15_702, "手算 ⌈376832/24⌉");
        assert_eq!(dirty_node_bytes, 15_702 * 16_384);
        assert_eq!(dirty_node_bytes, 257_261_568);
        assert!(dirty_node_bytes > ACCOUNTING_WRITE_BUDGET_BYTES, "节点口径：过界");

        // 窄读最紧的一格仍然过得去，占预算约一半
        let narrow_groups = accounting_entries(8192, 64, 1, false);
        assert_eq!(narrow_groups, 16_708);
        let narrow_dirty_node_bytes = dirty_bytes_per_publish(narrow_groups, 25, leaf_fanout).unwrap();
        assert_eq!(narrow_groups.div_ceil(24), 697, "手算 ⌈16708/24⌉");
        assert_eq!(narrow_dirty_node_bytes, 11_419_648);
        assert!(narrow_dirty_node_bytes <= ACCOUNTING_WRITE_BUDGET_BYTES);
        assert_eq!(narrow_dirty_node_bytes * 100 / ACCOUNTING_WRITE_BUDGET_BYTES, 53, "占预算 53%");
        // 不合法几何报 None，不许退化成 0
        assert_eq!(dirty_bytes_per_publish(10, 25, 0), None);
        assert_eq!(dirty_bytes_per_publish(10, 0, 603), None);
    }

    /// **第三轮补的两条臂**：16 KiB 槽号一槽一条 vs 一单元一条加跨度段。
    /// 后者贵 1 字节，却在三个轴上都不差、在两个轴上更好。
    #[test]
    fn per_unit_record_with_a_span_byte_beats_halving_the_grain() {
        let capacity_bytes = 16u64 * 1024 * 1024 * 1024 * 1024;
        let inner_fanout_value = fanout(NODE_BYTES, 58, 0, 7 + CHILD_POINTER_BYTES_SETTLED);
        // 一槽一条：粒度 16384 ⇒ 条目数按槽数
        let slot_entry_count = allocation_entries(capacity_bytes, NODE_ALIGNMENT_BYTES, 90);
        let slot_fanout = fanout(NODE_BYTES, 58, 0, 15);
        assert_eq!(slot_entry_count, 966_367_641);
        assert_eq!(slot_fanout, 1088);
        let slot_tree_bytes = tree_bytes(slot_entry_count, slot_fanout, inner_fanout_value).unwrap();
        assert_eq!(slot_tree_bytes, 14_606_106_624);
        assert_eq!(slot_tree_bytes as u128 * 1_000_000 / capacity_bytes as u128, 830);

        // 一单元一条 + 1 字节跨度段：条目数按单元数
        let unit_entry_count = allocation_entries(capacity_bytes, DATA_UNIT_BYTES, 90);
        // **条目宽走同一个函数**，不写字面量 —— 否则跨度段漏掉时没有一个测试会红（实测踩过）
        let unit_entry_bytes = allocation_entry_bytes(1, 6, 1, 8);
        assert_eq!(unit_entry_bytes, 16, "dev 1 + 槽号 6 + 跨度 1 + 代 8");
        assert_eq!(allocation_entry_bytes(1, 6, 0, 8), 15, "不带跨度段就是 15");
        let unit_fanout = fanout(NODE_BYTES, 58, 0, unit_entry_bytes);
        assert_eq!(unit_entry_count, 483_183_820);
        assert_eq!(unit_fanout, 1020, "手算 (16384−58)/16 = 16326/16 = 1020.4 ⇒ 1020");
        let unit_tree_bytes = tree_bytes(unit_entry_count, unit_fanout, inner_fanout_value).unwrap();
        assert_eq!(unit_tree_bytes, 7_789_936_640);
        assert_eq!(unit_tree_bytes as u128 * 1_000_000 / capacity_bytes as u128, 442);
        // 便宜 1.88 倍 —— ⚠️ 但 `.claude/rules/fs-design.md` 明令「比谁省不构成任何判据」
        assert_eq!(slot_tree_bytes * 100 / unit_tree_bytes, 187);
        // **两条要求不可兼得（第三轮的判决）**：
        // ① 表达得了 D18 已定项 9 的全部合法起点（要 16 KiB 分辨率）
        // ② 让「半个数据单元被分配」不可表示（要分辨率粗到 32 KiB）
        // 任何一个 grain 都满足不了两条 —— 加跨度段也改不了。
        for &grain in [NODE_ALIGNMENT_BYTES, DATA_UNIT_BYTES].iter() {
            let all_starts_ok = alignable(4096, grain).1 == 0;
            let half_unit_ok = !half_unit_representable(grain);
            assert!(!(all_starts_ok && half_unit_ok), "grain={grain} 不该两条都满足");
        }
        assert!(alignable(4096, NODE_ALIGNMENT_BYTES).1 == 0 && half_unit_representable(NODE_ALIGNMENT_BYTES));
        assert!(alignable(4096, DATA_UNIT_BYTES).1 == 2048 && !half_unit_representable(DATA_UNIT_BYTES));
        // ⇒ 「半个单元可不可表示」这一维在两条候选之间**分不出差别**（两条都可表示），
        //   不许拿它当选边的依据。
        assert_eq!(half_unit_representable(NODE_ALIGNMENT_BYTES), half_unit_representable(NODE_ALIGNMENT_BYTES));
        // 两条臂寻址范围相同（都是 6 字节 16 KiB 槽号 ⇒ 2^62 = 4 EiB）
        assert_eq!(addressable_bytes(6, NODE_ALIGNMENT_BYTES, true), 1u128 << 62);
        assert_eq!(misaligned_representable(6, NODE_ALIGNMENT_BYTES, true), 0, "两条都表达不了非 16K 对齐的起点");
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
        let segments = accounting_key(1, 4, 1, 8);
        let full_key_bytes = key_bytes(&segments);
        assert_eq!(full_key_bytes, 14);
        for segment_index in 0..segments.len() {
            let mut cut_segments = segments;
            let was_bytes = cut_segments[segment_index].bytes;
            cut_segments[segment_index].bytes = 0;
            assert_eq!(full_key_bytes - key_bytes(&cut_segments), was_bytes, "第 {segment_index} 段没进模型");
        }
        // stat_tag 与 gen 有上界 ⇒ 砍成 0 必须翻红
        let mut cut_statistic_segment = segments;
        cut_statistic_segment[0].bytes = 0;
        assert_eq!(coverage(&cut_statistic_segment[0]), Coverage::Overflows);
        let mut cut_generation_segment = segments;
        cut_generation_segment[3].bytes = 0;
        assert_eq!(coverage(&cut_generation_segment[3]), Coverage::Overflows);
        // tree_id / dev_id 上界指不到 ⇒ 砍成 0 也只能报 upper_unknown，
        // **这正是判据 2 要暴露的那件事**：这两段的宽度不是量出来的，是要定的。
        let mut cut_tree_segment = segments;
        cut_tree_segment[1].bytes = 0;
        assert_eq!(coverage(&cut_tree_segment[1]), Coverage::UpperUnknown);
    }
}

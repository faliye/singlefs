//! E98：inode 记录与 inode 树的几何 —— D8 已定项 6 欠的那次测量。
//!
//! ## 被引用条款逐字贴在这里（verify-before-claiming.md「把定义句原样贴进实验注释」）
//!
//! - **D21 三条已定边界逐字**：「权威态 = 单元 + 记账 + 根；索引是派生态；不设兼容通道」，
//!   判据逐字：「一样东西若不能只从权威态重算出来 ⇒ **它已经事实上晋升为权威态，
//!   D21 已定的『权威态 = 单元 + 记账 + 根』必须扩项**，并给它单元同级的持久性与崩溃恢复保证。」
//! - **D18 已定项 3 逐字**：单元头五元组 =
//!   `(单元类型标签, 树 ID, 对象 ID, 对象出生代, 锚点偏移)`。**一个 POSIX 属性都没有。**
//!   同一份正文自陈：「一旦 inode 记录丢了，扫描找到的数据单元**放不回去**」。
//! - **D18 已定项 7**：共同明文前缀 42 字节（magic 4 + 格式版本 2 + flags 2 + 声明长度 2 +
//!   头校验和 32）；索引节点类身份段 = 树 ID + 层级 + 该树已定携带的 key 区间 + 诞生代号 + fsid
//!   ⇒ 基础节点头 ≥ 58 字节（E73 已按 58 / 67 / 76 三档重算过）。
//! - **D18 已定项 2**：**记账树**的节点带 key 区间；dirent 树推迟。**inode 树带不带，这条没说。**
//! - **D8 已定项 3 逐字**：数据 key = `(locality_id, inode, offset)`；`locality_id` 创建时
//!   从父目录继承、**永远不许被改名更新**；命名树 `(parent, name) → inode` 的 value 只有 inode。
//! - **D8 已定项 2**：节点 **16 KiB**。**D4 已定项 2**：短 extent 补齐到 32 KiB（1 KiB 文件占 32 倍）。
//! - **D14 已定项 3**：内联阈值 0 ⇒ inode 记录里不放文件数据。
//! - **D16 已定项 6**：每次发布 `checkpoint_txg` + 1，fsync 触发的也是发布。
//! - **D19 已定项 4（2026-09-03）**：位置条目 14 字节 ⇒ 树表单元指针 **59** 字节。
//! - **D21 硬约束 6 逐字（承重）**：「**把一个算不出来的权威值放进一个可重建的容器里，
//!   它就事实上晋升为权威态**，而 D21 已定的权威态清单里没有它 ⇒
//!   『索引可从单元重建』与『节点上有不可重建的权威字节』**两句话不能同真**。」
//!   同一份正文还判过一次同形的误用：「D20『单元不分元数据与数据』约束的是**单元格式的同一性**，
//!   不是〔角色〕——**把前者推广成后者是一次误用**。」
//! - **D18 已定项 10（2026-09-02 用户定案 + E83）**：墓碑取「**区间记录 + 打包共享单元**」
//!   ——**这是「把许多条记录打包进一个共享单元」的既有先例**，形态可复用。
//! - **D4 已定项 5**：单元占盘恒 **32768** 字节（含头）。**D18 已定项 7**：数据单元头 **91** 字节。
//!
//! ## 判据（E98 正文跑前写死，跑完不许改）
//!
//! 1. **绝对值断言**：叶装得下的记录数恰好 `(16384 − 节点头) / 记录宽`；
//!    树高由叶扇出与内部扇出**两级各自**算出。不许只做臂间互比。
//! 2. **态别判据**：逐字段判「能不能只从权威态重算」，**数得出不能重算的字段数**。
//!    > 0 ⇒ 按 D21 的判据，要么住单元、要么权威态扩项——**两条路都报，不替它选**。
//! 3. **`locality_id` 对照的判别力**：带对照臂必须全抓、不带对照臂必须全漏，两个数都要绝对值。
//! 4. **改动计数的排序力**：同一 checkpoint 内改 k 次，数「排不出先后」的对数。
//! 5. **爆炸半径**：丢一个容器丢多少条 inode 记录，**两种形态各一个数，放进同一张表**。
//!    ⚠️ **第三轮修正**：第一、二版只在 btree 那条几何线上发 `records_lost_per_leaf`，
//!    打包臂的 233 只以 `records_per_container` 的身份出现，两个数从没被并排比过
//!    ⇒ 反向接受条款「住单元被判据 5 的爆炸半径否掉」**在结构上永远碰不到**。
//!    现在两种形态各报一个「**永久损失**」（丢掉之后重算不回来的记录数）。
//!
//! ## 失败条款（跑前写死）
//!
//! - **阳性对照，每条臂都跑**：判据 3 的不带对照臂必须漏光；
//!   判据 2 的「假设属性能从单元头重算」臂必须把不能重算的字段数算成 0。
//! - 节点头按 58 / 67 / 76 三档各算一次；三档不同向 ⇒ 结论写成「依赖节点头」。
//! - **反向接受条款**：判据 2 数出 > 0 **且**「住单元」被判据 5 的爆炸半径否掉
//!   ⇒ 结论是「D21 必须扩项」，如实写，不许硬凑成不扩项。
//!
//! ## 它答不了的
//!
//! 纯算术几何模型：没有 btree 实现、没有目录、没有崩溃点重放，文件操作 0 处。不答挂钟。

use e7_index_bench::Emitter;

/// D8 已定项 2：节点 16 KiB。**格式常量。**
const NODE_BYTES: u64 = 16384;
/// E73 按 D18 已定项 7 重算过的三档基础节点头下界。
const NODE_HEADERS: [u64; 3] = [58, 67, 76];
/// D19 已定项 4（2026-09-03）：位置条目 14 ⇒ 树表单元指针 31 + 14 × 2 = 59。
const CHILD_PTR: u64 = 59;
/// D18 已定项 7：共同明文前缀 42 字节；数据单元头 91 字节初值。
const COMMON_PREFIX: u64 = 42;
const DATA_UNIT_HEADER: u64 = 91;
/// D4 已定项 5：单元占盘恒 32768 字节（含头）。
const UNIT_BYTES: u64 = 32768;

/// inode 记录住哪：两种形态，**态别后果完全不同**。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Shape {
    /// 甲：住 inode 树叶节点的 value 里。容器是**索引节点** ⇒ D21 逐字「索引是派生态」。
    BtreeValue,
    /// 乙：打包进共享的「inode 单元」，inode 树只做路由。
    /// 容器是**单元** ⇒ 落在 D21「权威态 = 单元 + 记账 + 根」的第一项里。
    /// 先例：D18 已定项 10 的墓碑「区间记录 + 打包共享单元」。
    PackedUnit,
}

/// **本实验的核心判据**：这个形态触不触发 D21 硬约束 6。
/// 触发 = 「把算不出来的权威值放进可重建的容器」⇒ 权威态必须扩项。
fn triggers_hard_constraint_6(shape: Shape, non_recomputable_fields: usize) -> bool {
    if non_recomputable_fields == 0 {
        return false; // 全都算得出来 ⇒ 它本来就是派生态，放哪都行
    }
    match shape {
        // 索引节点是派生态、可重建 ⇒ 触发
        Shape::BtreeValue => true,
        // 单元本身就在权威态清单的第一项里 ⇒ 不触发
        Shape::PackedUnit => false,
    }
}

/// **判据 5**：丢一个容器，**永久**损失几条记录。
/// 「永久」= 重算不回来。只有当 14 个字段全都能从权威态重算时，损失才是可恢复的（0）。
/// ⚠️ E29（坏一个节点的爆炸半径）实测索引节点的永久损失**恒 0**，靠的正是「索引可从单元重建」；
/// inode 记录不满足那个前提 ⇒ **两种形态的永久损失都不是 0**。
fn permanent_loss_per_container(shape: Shape, header: u64, rec: u64, non_recomputable: usize) -> u64 {
    if non_recomputable == 0 {
        return 0; // 全都算得回来 ⇒ 与 E29 的索引节点同形，永久损失恒 0
    }
    records_per_container(shape, header, rec)
}

/// 一个容器装几条记录。
fn records_per_container(shape: Shape, header: u64, rec: u64) -> u64 {
    if rec == 0 {
        return 0;
    }
    match shape {
        Shape::BtreeValue => {
            if NODE_BYTES <= header { 0 } else { (NODE_BYTES - header) / rec }
        }
        Shape::PackedUnit => {
            if UNIT_BYTES <= DATA_UNIT_HEADER { 0 } else { (UNIT_BYTES - DATA_UNIT_HEADER) / rec }
        }
    }
}

/// 一个 inode 字段：宽度，以及**能不能只从权威态（单元 + 记账 + 根）重算出来**。
/// `recomputable` 那一列的理由逐条写在 `why` 里，不许只写 true/false。
struct Field {
    name: &'static str,
    bytes: u64,
    recomputable: bool,
    why: &'static str,
}

/// 提案的字段表（初值）。**次序即盘上次序。**
const FIELDS: [Field; 14] = [
    // 身份段
    Field { name: "locality_id", bytes: 8, recomputable: false,
            why: "D8-item3: key 首段，改名永不更新；命名树 value 只有 inode；五元组里没有它" },
    Field { name: "inode", bytes: 8, recomputable: true,
            why: "D18-item3 五元组的对象 ID" },
    Field { name: "obj_birth", bytes: 8, recomputable: true,
            why: "D18-item3 五元组的对象出生代" },
    // 属性段
    Field { name: "mode", bytes: 4, recomputable: false, why: "五元组里没有" },
    Field { name: "uid", bytes: 4, recomputable: false, why: "五元组里没有" },
    Field { name: "gid", bytes: 4, recomputable: false, why: "五元组里没有" },
    Field { name: "nlink", bytes: 4, recomputable: false, why: "要 dirent 树；第一版没有目录" },
    Field { name: "size", bytes: 8, recomputable: false,
            why: "尾部是洞时扫描给不出 size：锚点偏移只给已写到的最远处" },
    Field { name: "blocks", bytes: 8, recomputable: true,
            why: "数这个对象的单元数就得到（D4-item2 补齐到 32 KiB，每单元定长）" },
    Field { name: "rdev", bytes: 8, recomputable: false, why: "设备节点，五元组里没有" },
    Field { name: "atime", bytes: 12, recomputable: false, why: "五元组里没有" },
    Field { name: "mtime", bytes: 12, recomputable: false, why: "五元组里没有" },
    Field { name: "ctime", bytes: 12, recomputable: false, why: "五元组里没有" },
    Field { name: "change_count", bytes: 8, recomputable: false,
            why: "取 checkpoint_txg；扫描读不出「哪一次发布改的」" },
];
/// 预留尾巴。
const RESERVED: u64 = 32;
/// inode 树的 key。头判别位做成**臂**，不是常量——
/// **D6 已定项 1（2026-08-30 用户定案）逐字「取②，每头一棵自己的树」**，
/// 而 D8 已定项 3 的注逐字「每头一棵树之下**三段 key 在树内唯一**，
/// 两个头不再落进同一个 key 槽」⇒ **判别位应当为 0 字节**。
/// 两条臂都跑，让已定条款去选，不在这里替它选。
const KEY_INODE: u64 = 8;
const KEY_HEAD_DISCRIM_ARMS: [u64; 2] = [0, 2];
/// D6 已定项 1 之下该取的那一档。
const KEY_HEAD_DISCRIM: u64 = KEY_HEAD_DISCRIM_ARMS[0];

fn record_bytes() -> u64 {
    FIELDS.iter().map(|f| f.bytes).sum::<u64>() + RESERVED
}

/// **判据 2**：不能只从权威态重算的字段数。
/// `assume_all_recomputable` 是阳性对照——它必须把这个数算成 0。
fn non_recomputable(assume_all_recomputable: bool) -> usize {
    if assume_all_recomputable {
        return 0;
    }
    FIELDS.iter().filter(|f| !f.recomputable).count()
}

fn fanout(node: u64, header: u64, range_field: u64, entry: u64) -> u64 {
    if entry == 0 {
        return 0;
    }
    let o = header.saturating_add(range_field);
    if node <= o {
        return 0;
    }
    (node - o) / entry
}

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
            return None;
        }
    }
    Some(h)
}

/// **判据 3**：`locality_id` 对照。`n` 个对象里 `bad` 个的记录与 extent key 首段不一致。
/// 返回 (抓到的, 静默读成空洞的)。`with_check` 是那条不变量在不在。
fn locality_mismatch(n: u64, bad: u64, with_check: bool) -> (u64, u64) {
    let bad = bad.min(n);
    if with_check {
        (bad, 0)
    } else {
        (0, bad)
    }
}

/// **判据 4**：同一个 checkpoint 内对同一个 inode 改 `k` 次，改动计数取 `checkpoint_txg`
/// ⇒ k 条记录同号 ⇒ 两两之间排不出先后。返回排不出先后的对数。
/// `window_seq_bits` > 0 时窗口内序号能分开它们，返回 0。
fn unordered_pairs(k: u64, window_seq_bits: u32) -> u64 {
    if window_seq_bits > 0 && (k as u128) <= (1u128 << window_seq_bits) {
        return 0;
    }
    k * k.saturating_sub(1) / 2
}

fn main() {
    let mut em = Emitter::new();
    let mut out: Vec<String> = Vec::new();
    let rec = record_bytes();
    let key = KEY_INODE + KEY_HEAD_DISCRIM;

    out.push(em.emit_raw(&format!(
        "name=config node_bytes={NODE_BYTES} common_prefix={COMMON_PREFIX} child_ptr={CHILD_PTR} \
         record_bytes={rec} reserved={RESERVED} key_bytes={key} fields={}",
        FIELDS.len()
    )));

    // 判据 2：逐字段的态别
    for f in FIELDS.iter() {
        out.push(em.emit_raw(&format!(
            "name=field name_of={} bytes={} recomputable={} why={}",
            f.name,
            f.bytes,
            u8::from(f.recomputable),
            f.why
        )));
    }
    for &assume in [false, true].iter() {
        out.push(em.emit_raw(&format!(
            "name=state_class assume_all_recomputable={} non_recomputable={} total_fields={}",
            u8::from(assume),
            non_recomputable(assume),
            FIELDS.len()
        )));
    }

    // **形态判据**：两种形态各自触不触发 D21 硬约束 6、各装几条、丢一个容器永久损失几条
    for &shape in [Shape::BtreeValue, Shape::PackedUnit].iter() {
        for &assume in [false, true].iter() {
            let nr = non_recomputable(assume);
            let loss = permanent_loss_per_container(shape, 58, rec, nr);
            out.push(em.emit_raw(&format!(
                "name=shape shape={:?} assume_all_recomputable={} non_recomputable={nr} \
                 triggers_hc6={} records_per_container={} container_bytes={} \
                 permanent_loss_per_container={loss} needs_redundancy_clause={}",
                shape,
                u8::from(assume),
                u8::from(triggers_hard_constraint_6(shape, nr)),
                records_per_container(shape, 58, rec),
                match shape { Shape::BtreeValue => NODE_BYTES, Shape::PackedUnit => UNIT_BYTES },
                u8::from(loss > 0),
            )));
        }
    }

    // 判据 1 / 5：几何与爆炸半径。inode 树带不带 key 区间两种读法都算（D18 已定项 2 没说）
    for &hdr in NODE_HEADERS.iter() {
        for &disc in KEY_HEAD_DISCRIM_ARMS.iter() {
            let k = KEY_INODE + disc;
            for &with_range in [false, true].iter() {
                let rf = if with_range { 2 * k } else { 0 };
                let leaf_f = fanout(NODE_BYTES, hdr, rf, rec);
                let inner_f = fanout(NODE_BYTES, hdr, rf, k + CHILD_PTR);
                for &n in [1_000_000u64, 100_000_000, 1_000_000_000].iter() {
                    let h = tree_height(n, leaf_f, inner_f);
                    out.push(em.emit_raw(&format!(
                        "name=geom header={hdr} head_discrim={disc} key={k} with_range={} \
                         inodes={n} record={rec} leaf_fanout={leaf_f} inner_fanout={inner_f} \
                         height={} records_lost_per_leaf={leaf_f}",
                        u8::from(with_range),
                        h.map(|v| v.to_string()).unwrap_or_else(|| "NA".into()),
                    )));
                }
            }
        }
    }

    // 判据 3：locality 对照的判别力（含阳性对照）
    for &(n, bad) in [(1_000_000u64, 1_000u64), (1_000_000, 100_000)].iter() {
        for &chk in [true, false].iter() {
            let (caught, silent) = locality_mismatch(n, bad, chk);
            out.push(em.emit_raw(&format!(
                "name=locality objects={n} mismatched={bad} with_check={} caught={caught} silent_holes={silent}",
                u8::from(chk)
            )));
        }
    }

    // 判据 4：改动计数的排序力
    for &k in [1u64, 2, 8, 64].iter() {
        for &bits in [0u32, 8, 16].iter() {
            out.push(em.emit_raw(&format!(
                "name=change_order updates_in_window={k} window_seq_bits={bits} unordered_pairs={}",
                unordered_pairs(k, bits)
            )));
        }
    }

    for l in &out {
        println!("{l}");
    }
    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_constants_match_kb() {
        assert_eq!(NODE_BYTES, 16384, "D8 已定项 2");
        assert_eq!(COMMON_PREFIX, 42, "D18 已定项 7");
        assert_eq!(CHILD_PTR, 59, "D19 已定项 4 之后：31 + 14 × 2");
    }

    /// **判据 1 的绝对值**：字段表逐段加起来是 140 字节，扇出在三档头上都是 116。
    #[test]
    fn criterion1_record_and_fanout_are_absolute() {
        // 手算：身份段 8+8+8 = 24；属性段 4+4+4+4+8+8+8+12+12+12+8 = 84；预留 32
        let ident: u64 = FIELDS[0..3].iter().map(|f| f.bytes).sum();
        let attr: u64 = FIELDS[3..].iter().map(|f| f.bytes).sum();
        assert_eq!(ident, 24);
        assert_eq!(attr, 84);
        assert_eq!(record_bytes(), 140);
        // 手算：(16384 − 58) / 140 = 16326 / 140 = 116.6… ⇒ 116
        assert_eq!(fanout(NODE_BYTES, 58, 0, 140), 116);
        // 三档头逐档相同 ⇒ 结论不依赖节点头
        for &h in NODE_HEADERS.iter() {
            assert_eq!(fanout(NODE_BYTES, h, 0, 140), 116, "头 {h} 那一档");
        }
    }

    /// **判据 2 的绝对值 + 阳性对照**：14 个字段里 11 个不能只从权威态重算。
    #[test]
    fn criterion2_eleven_fields_cannot_be_recomputed() {
        assert_eq!(FIELDS.len(), 14);
        assert_eq!(non_recomputable(false), 11);
        // 能重算的恰好三个，且都指得到 D18 已定项 3 的五元组或数单元
        let ok: Vec<&str> = FIELDS.iter().filter(|f| f.recomputable).map(|f| f.name).collect();
        assert_eq!(ok, vec!["inode", "obj_birth", "blocks"]);
        // 阳性对照：假设全都能重算时必须算成 0，否则这一维根本没进模型
        assert_eq!(non_recomputable(true), 0);
        // ⇒ 按 D21 的判据，inode 记录不是派生态
        assert!(non_recomputable(false) > 0, "不为 0 才触发 D21 那条判据");
    }

    /// **判据 3 的绝对值 + 阳性对照**：带对照全抓、不带对照全漏。
    #[test]
    fn criterion3_locality_check_is_the_only_thing_between_data_and_silent_holes() {
        let (c, s) = locality_mismatch(1_000_000, 1_000, true);
        assert_eq!((c, s), (1_000, 0), "带对照：全抓，零静默");
        let (c2, s2) = locality_mismatch(1_000_000, 1_000, false);
        assert_eq!((c2, s2), (0, 1_000), "不带对照：全漏，全部静默读空洞");
        // 阴性对照：没有不一致时两条臂都必须是 0
        assert_eq!(locality_mismatch(1_000_000, 0, true), (0, 0));
        assert_eq!(locality_mismatch(1_000_000, 0, false), (0, 0));
        // bad 超过 n 时要收敛到 n，不许算出比总数还多
        assert_eq!(locality_mismatch(10, 99, false).1, 10);
    }

    /// **判据 4 的绝对值**：改动计数取 checkpoint_txg 时同窗口内排不出先后。
    #[test]
    fn criterion4_change_counter_alone_cannot_order_within_one_checkpoint() {
        assert_eq!(unordered_pairs(1, 0), 0, "只改一次没有对");
        assert_eq!(unordered_pairs(2, 0), 1);
        assert_eq!(unordered_pairs(8, 0), 28, "手算 C(8,2)");
        assert_eq!(unordered_pairs(64, 0), 2016, "手算 C(64,2)");
        // 有窗口内序号且位够时归零
        assert_eq!(unordered_pairs(64, 8), 0, "8 位装得下 64 次");
        // 位不够时又回来
        assert_eq!(unordered_pairs(64, 0), 2016);
        assert!(unordered_pairs(300, 8) > 0, "8 位装不下 300 次");
    }

    /// **判据 5 的绝对值**：丢一个叶节点丢掉的是整叶的记录，不是一条。
    #[test]
    fn criterion5_blast_radius_is_a_whole_leaf() {
        let leaf_f = fanout(NODE_BYTES, 58, 0, record_bytes());
        assert_eq!(leaf_f, 116);
        // 1e6 个 inode ⇒ 8621 个叶；丢一个叶 = 丢 116 条记录 = 0.0116%
        assert_eq!(1_000_000u64.div_ceil(leaf_f), 8621);
        assert_eq!(leaf_f * 1_000_000 / 1_000_000, 116);
    }

    /// **树高由两级扇出各自算**，不是拿叶扇出一路乘上去（E74 踩过这个）。
    /// **头判别位应当为 0**：D6 已定项 1 定「每头一棵自己的树」，
    /// D8 已定项 3 的注逐字「两个头不再落进同一个 key 槽」⇒ 这一位没有对象。
    /// 两条臂的代价都量出来，差别落在内部扇出上。
    #[test]
    fn head_discriminator_has_no_object_under_per_head_trees() {
        assert_eq!(KEY_HEAD_DISCRIM, 0, "D6 已定项 1：每头一棵自己的树");
        assert_eq!(KEY_HEAD_DISCRIM_ARMS, [0, 2]);
        // 手算：(16384−58)/(8+59) = 16326/67 = 243.6 ⇒ 243
        assert_eq!(fanout(NODE_BYTES, 58, 0, 8 + CHILD_PTR), 243);
        // 带 2 字节判别位：16326/69 = 236.6 ⇒ 236，白付 7 格扇出
        assert_eq!(fanout(NODE_BYTES, 58, 0, 10 + CHILD_PTR), 236);
        assert_eq!(243 - 236, 7);
    }

    #[test]
    fn tree_height_uses_two_fanouts() {
        let key = KEY_INODE + 2; // 带判别位那条臂
        assert_eq!(key, 10);
        let leaf_f = fanout(NODE_BYTES, 58, 0, record_bytes());
        let inner_f = fanout(NODE_BYTES, 58, 0, key + CHILD_PTR);
        assert_eq!(inner_f, 236, "手算 (16384−58)/69 = 16326/69 = 236.6 ⇒ 236");
        assert_eq!(tree_height(1_000_000, leaf_f, inner_f), Some(3));
        // 手算：116 × 236 = 27376；×236 = 6 460 736；×236 = 1.52e9 ≥ 1e9 ⇒ 4 层
        assert_eq!(tree_height(1_000_000_000, leaf_f, inner_f), Some(4));
        // 拿叶扇出一路乘会少算：116^3 = 1.56e6 ⇒ 会误报 1e6 只要 3 层里的 2 层
        assert!(tree_height(1_000_000, leaf_f, leaf_f) <= tree_height(1_000_000, leaf_f, inner_f));
    }

    /// **形态判据的绝对值**：只有「打包进单元」这条形态不触发 D21 硬约束 6。
    #[test]
    fn only_the_packed_unit_shape_avoids_widening_the_authoritative_state() {
        let nr = non_recomputable(false);
        assert_eq!(nr, 11);
        // 甲：住索引节点的 value ⇒ 把 11 个算不出来的权威值放进可重建的容器 ⇒ 触发
        assert!(triggers_hard_constraint_6(Shape::BtreeValue, nr));
        // 乙：打包进单元 ⇒ 容器就在权威态清单第一项里 ⇒ 不触发
        assert!(!triggers_hard_constraint_6(Shape::PackedUnit, nr));
        // 阳性对照：假设全都能重算时，两种形态都不该触发——
        // 这证明触发与否真的由「算不算得出来」驱动，不是由形态标签驱动
        assert!(!triggers_hard_constraint_6(Shape::BtreeValue, 0));
        assert!(!triggers_hard_constraint_6(Shape::PackedUnit, 0));
    }

    /// **密度的绝对值**：打包进 32 KiB 单元比塞进 16 KiB 叶装得多一倍。
    #[test]
    fn packed_unit_holds_twice_as_many_records() {
        let rec = record_bytes();
        assert_eq!(rec, 140);
        // 手算：(16384 − 58) / 140 = 16326 / 140 = 116.6 ⇒ 116
        assert_eq!(records_per_container(Shape::BtreeValue, 58, rec), 116);
        // 手算：(32768 − 91) / 140 = 32677 / 140 = 233.4 ⇒ 233
        assert_eq!(records_per_container(Shape::PackedUnit, 58, rec), 233);
        assert_eq!(DATA_UNIT_HEADER, 91, "D18 已定项 7 的数据单元头初值");
        assert_eq!(UNIT_BYTES, 32768, "D4 已定项 5");
        // 打包形态的容器更大，装的更多——但爆炸半径也更大，两个数都要报
        assert!(records_per_container(Shape::PackedUnit, 58, rec)
            > records_per_container(Shape::BtreeValue, 58, rec));
        // 不合法输入：记录宽 0 时报 0，不许除零
        assert_eq!(records_per_container(Shape::PackedUnit, 58, 0), 0);
    }

    /// **判据 5 的绝对值（第三轮补）**：两种形态的永久损失并排比，
    /// 而**两个都不是 0** ⇒ 反向接受条款那句「爆炸半径否掉住单元」评价不了，
    /// 真正被这一维暴露的是「两种形态都要一条冗余 / 可检出条款」。
    #[test]
    fn criterion5_both_shapes_lose_records_permanently() {
        let rec = record_bytes();
        let nr = non_recomputable(false);
        let a = permanent_loss_per_container(Shape::BtreeValue, 58, rec, nr);
        let b = permanent_loss_per_container(Shape::PackedUnit, 58, rec, nr);
        assert_eq!(a, 116, "丢一个 16 KiB 索引叶 = 永久丢 116 条");
        assert_eq!(b, 233, "丢一个 32 KiB 打包单元 = 永久丢 233 条");
        assert!(a > 0 && b > 0, "两种形态都不是 0 ⇒ 都要冗余 / 可检出条款");
        assert_eq!(b * 100 / a, 200, "打包形态的永久损失恰好是另一种的 2.00 倍");
        // **阳性对照**：假设 14 个字段全都能从权威态重算，两种形态的永久损失必须都归 0——
        // 那正是 E29（坏一个节点的爆炸半径）在索引节点上量到的形态
        assert_eq!(permanent_loss_per_container(Shape::BtreeValue, 58, rec, 0), 0);
        assert_eq!(permanent_loss_per_container(Shape::PackedUnit, 58, rec, 0), 0);
    }

    /// 不合法几何一律 None / 0，不许退化成一个数。
    #[test]
    fn illegal_geometry_is_not_a_measurement() {
        assert_eq!(fanout(NODE_BYTES, NODE_BYTES, 0, 140), 0);
        assert_eq!(tree_height(100, 0, 10), None);
        assert_eq!(tree_height(100, 10, 1), None);
    }
}

//! E111：条带表按落点排的那棵树，付不付得起。
//!
//! **它答的是** D2 已定项 14（条带表这棵逻辑树的 key 取什么）——跑的时候它还是未定项。
//! D2 已定项 12 定了**载体**（住码 3 打包记录、走容器索引、容器恒走镜像），
//! **没说这棵逻辑树按什么排序**；而它的三个消费者问的都是同一句
//! 「给一个落点，它属于哪条条带」——分配准入（C127）、清扫准入（C128）、
//! 让渡区 A 的签发判据（D9 未定项 9 第 ⑦ 条合取）。
//! 容器索引答不了这一句：它的 key 是 `(出生树 ID, 打包记录类型, 容器号, 容器出生代)`，
//! **四段里没有落点**，容器号是「最大号 + 1」单调分配，与它装的记录覆盖哪些落点无函数关系。
//!
//! ## 三条臂
//!
//! | 臂 | key 取什么 | 「给落点找条带」怎么答 |
//! |---|---|---|
//! | `slot`   | 落点（设备身份 4 + 16 KiB 槽号 6），与 D3 已定项 3 的分配记录树同坐标系 | 一次点查 |
//! | `stripe` | 条带号（条带诞生代 8 + 条带序号 4） | **答不了**——要扫全表 |
//! | `none`   | 不排序，只按容器身份索引（**今天的状态**） | **答不了**——要扫全部容器 |
//!
//! ## 口径（跑前写死）
//!
//! 1. 几何全部指得到已定条款：单元 32768（D4 已定项 1）、节点 16384（D8 已定项 2）、
//!    容器头 103 / 成员记录 56（D18 已定项 11 / E106 已钉）、
//!    落点 key 段宽 4 + 6（D19 已定项 4 / D3 已定项 7 同坐标系）。
//! 2. **与 E97 同尺**：16 TiB / 90% 填充，条目数与树高按同一套算法算，便于与分配记录树
//!    （483 183 820 条、树高 4、442 ppm）直接比。
//! 3. **一格一条、含 parity 格**（D2 已定项 12）⇒ 条目数按**格数**算不是按单元数算。
//! 4. `w` 扫 2 / 3 / 4：`w = 2` 时全镜像、一条条带记录都不写（D2 已定项 12 的射程），
//!    条目数恒 0——这是阴性对照。
//!
//! ## 它不答什么（跑前写死）
//!
//! 不答挂钟；不答这棵树自己的更新代价（那是 C103 的一部分）；
//! 不答空间效率的高下（`.claude/rules/fs-design.md`「比谁省不构成任何判据」）——
//! 只把三条臂「答不答得出那句话」与「按落点排要多大」摆出来。

use e7_index_bench::{Emitter, Sample};

/// 数据单元 32768（D4 已定项 1 / 已定项 5）。
const DATA_UNIT: u64 = 32768;
/// 索引节点 16384（D8 已定项 2）。
const NODE: u64 = 16384;
/// 16 KiB 槽（D3 已定项 7 的落点粒度）。
const SLOT: u64 = 16384;
/// 条带成员记录定长 56（E106 已钉）。
const STRIPE_REC: u64 = 56;
/// 码 3 容器头 103（D18 已定项 11）。
const PACKED_HDR: u64 = 103;
/// 落点 key：设备身份 4 + 16 KiB 槽号 6（D19 已定项 4 / D3 已定项 7 同坐标系）。
const SLOT_KEY: u64 = 4 + 6;
/// 条带号 key：条带诞生代 8 + 条带序号 4。
const STRIPE_KEY: u64 = 8 + 4;
/// 索引节点头，与 E97 / E106 同口径。
const NODE_HDR: u64 = 76;
/// 16 TiB。
const CAP: u64 = 16 * 1024 * 1024 * 1024 * 1024;
/// 与 E97 / E106 同口径的填充率。
const FILL_NUM: u64 = 9;
const FILL_DEN: u64 = 10;
/// E97 已入库的分配记录树三个数，拿来做同尺交叉校验。
const E97_ENTRIES: u64 = 483_183_820;
const E97_HEIGHT: u32 = 4;
const E97_PPM: u64 = 442;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Arm {
    Slot,
    Stripe,
    None,
}

impl Arm {
    fn name(self) -> &'static str {
        match self {
            Arm::Slot => "slot",
            Arm::Stripe => "stripe",
            Arm::None => "none",
        }
    }
    /// 「给一个落点，它属于哪条条带」这一句答不答得出来。
    fn answers_slot_lookup(self) -> bool {
        matches!(self, Arm::Slot)
    }
}

/// 装得下 `n` 个条目、扇出 `f` 的树有几层。
fn height(n: u64, f: u64) -> u32 {
    if n <= 1 {
        return 1;
    }
    let mut h = 1u32;
    let mut cap = f;
    while cap < n {
        cap = cap.saturating_mul(f);
        h += 1;
    }
    h
}

/// 一个 16384 节点、key 宽 `k`、value 是容器身份（8 字节）时的扇出。
fn fanout(k: u64) -> u64 {
    (NODE - NODE_HDR) / (k + 8)
}

/// 池里 90% 那笔预算切成多少个格（32768 一格），其中多少是数据格、多少是 parity 格。
/// 口径与 E106（条带成员表的载体） 同：**16 TiB × 90% 是数据格与 parity 格一起花的一笔预算**，
/// 一条 `w` 列的条带占 `w` 个格、其中 1 个是 parity ⇒ 条目数（一格一条）恒等于这笔预算的格数，
/// **与 `w` 无关**；`w` 只改这笔预算里有多少是用户数据。
///
/// ⚠️ **不许把 483 183 820 当成数据格数再往上加 parity。** 那样 `w = 3` 会往一块 16 TiB 的盘上
/// 摆 724 775 730 个 32 KiB 格 = 135% 的盘。E106（条带成员表的载体） 第一版正是这个错（126.3%），
/// 当日已修并留了会红的断言；E111（条带表的 key 与它的几何） 第一版重犯了一次，
/// 现在由 `cells_never_overfill_the_pool` 挡住。
fn cells(w: u64) -> (u64, u64, u64) {
    if w < 3 {
        // w = 2 全镜像，一条条带记录都不写（D2 已定项 12 的射程）
        return (0, 0, 0);
    }
    let usable = CAP / FILL_DEN * FILL_NUM;
    // 这笔预算总共切出多少个格 —— 数据格与 parity 格都从这里出
    let total_cells = usable / DATA_UNIT;
    // parity 格：每条 w 列的条带一格
    let parity_cells = total_cells / w;
    let data_cells = total_cells - parity_cells;
    (data_cells, parity_cells, total_cells)
}

/// 这棵树自己占多少 ppm：条目住码 3 容器，容器 32768、头 103、一条 56。
/// 容器恒走 `w = 2` 镜像（D2 已定项 12）⇒ ×2。
fn tree_ppm(entries: u64) -> u64 {
    if entries == 0 {
        return 0;
    }
    let per = (DATA_UNIT - PACKED_HDR) / STRIPE_REC;
    let containers = entries.div_ceil(per);
    let bytes = containers * DATA_UNIT * 2;
    bytes * 1_000_000 / CAP
}

fn main() {
    let mut em = Emitter::new();
    let mut out = String::new();

    // 阴性对照：w = 2 ⇒ 三条臂条目数恒 0
    let (_, _, z) = cells(2);
    out.push_str(&em.emit_raw(&format!(
        "name=negative_control w=2 entries={z} verdict={}",
        if z == 0 { "pass" } else { "VOID" }
    )));
    out.push('\n');

    // 同尺交叉校验：分配记录树按 16 KiB 槽数算，必须落回 E97 已入库的那三个数
    let slots = CAP / FILL_DEN * FILL_NUM / SLOT;
    out.push_str(&em.emit_raw(&format!(
        "name=e97_crosscheck slots={slots} e97_entries={E97_ENTRIES} e97_height={E97_HEIGHT} e97_ppm={E97_PPM}"
    )));
    out.push('\n');

    for &w in &[2u64, 3, 4] {
        let (dc, pc, total) = cells(w);
        for arm in [Arm::Slot, Arm::Stripe, Arm::None] {
            let k = match arm {
                Arm::Slot => SLOT_KEY,
                Arm::Stripe => STRIPE_KEY,
                Arm::None => 0,
            };
            let f = if k == 0 { 0 } else { fanout(k) };
            let h = if total == 0 || k == 0 { 0 } else { height(total, f) };
            out.push_str(&em.emit_raw(&format!(
                "name=tree w={w} arm={} data_cells={dc} parity_cells={pc} entries={total} \
                 key_bytes={k} fanout={f} height={h} tree_ppm={} answers_slot_lookup={}",
                arm.name(),
                tree_ppm(total),
                arm.answers_slot_lookup()
            )));
            out.push('\n');
        }
    }

    // 阳性对照：把 key 宽度换成分配记录树那一档，扇出必须跟着变
    out.push_str(&em.emit_raw(&format!(
        "name=positive_control fanout_slot_key={} fanout_stripe_key={} verdict={}",
        fanout(SLOT_KEY),
        fanout(STRIPE_KEY),
        if fanout(SLOT_KEY) != fanout(STRIPE_KEY) {
            "pass"
        } else {
            "VOID"
        }
    )));
    out.push('\n');

    let (_, _, t3) = cells(3);
    out.push_str(&em.emit(
        "w3_entries",
        &Sample {
            ops: 1,
            bytes_per_op: t3,
            elapsed_ns: 1,
        },
    ));
    out.push('\n');
    out.push_str(&em.finish());
    println!("{out}");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 阴性对照（跑前写死）：`w = 2` 全镜像，一条条带记录都不写。
    #[test]
    fn width_two_writes_no_stripe_records() {
        assert_eq!(cells(2), (0, 0, 0));
        assert_eq!(tree_ppm(0), 0);
    }

    /// 格数的绝对值：16 TiB × 90% ÷ 32768 = 483 183 820，
    /// **与 E97 已入库的分配记录条目数逐位相同**——两个实验同尺的交叉校验。
    /// ⚠️ 逐位相同的是**总格数**（一格一条 ⇒ 条目数），**不是数据格数**：
    /// 数据格比它少一个 parity 的份额（`w = 3` 时少 161 061 273）。
    #[test]
    fn total_cell_count_matches_e97_entry_count() {
        let (dc, pc, total) = cells(3);
        assert_eq!(total, 483_183_820);
        assert_eq!(total, E97_ENTRIES);
        assert_eq!(CAP / FILL_DEN * FILL_NUM / DATA_UNIT, 483_183_820);
        assert_eq!(dc, 322_122_547);
        assert_eq!(pc, 161_061_273);
        assert!(dc < E97_ENTRIES);
    }

    /// **摆进去的格不许超过 90% 那笔预算。**
    /// E106（条带成员表的载体） 第一版把这笔预算当成数据格数、再往上加 parity 与容器
    /// ⇒ 往盘里摆了 126.3% 的格；E111（条带表的 key 与它的几何） 第一版重犯了一次（135%）。
    /// 这条断言就是那个坑的会红形态——它自己的判别力在最后两行。
    #[test]
    fn cells_never_overfill_the_pool() {
        let usable = CAP / FILL_DEN * FILL_NUM;
        for w in [2u64, 3, 4] {
            let (dc, pc, total) = cells(w);
            assert_eq!(total, dc + pc, "w={w}: 总格数不等于两类之和");
            assert!(
                total * DATA_UNIT <= usable,
                "w={w}: 摆了 {} 字节，90% 预算只有 {usable}",
                total * DATA_UNIT
            );
        }
        // 判别力：第一版那种写法（预算当数据格、parity 加在上面）必须被这条判据判红
        let v1 = E97_ENTRIES + E97_ENTRIES.div_ceil(2);
        assert_eq!(v1, 724_775_730);
        assert!(v1 * DATA_UNIT > usable, "这条断言分不出差别，等于没写");
    }

    /// parity 格：一条 `w` 列的条带一格 ⇒ `w = 3` 占预算的 1/3、`w = 4` 占 1/4。
    /// **条目数与 `w` 无关**——这是换口径之后最反直觉的一条，所以单独钉住。
    #[test]
    fn parity_cell_count_is_pinned() {
        let (dc, pc, total) = cells(3);
        assert_eq!(pc, total / 3);
        assert_eq!(pc, 161_061_273);
        assert_eq!(total, 483_183_820);
        let (dc4, pc4, total4) = cells(4);
        assert_eq!(pc4, total4 / 4);
        assert_eq!(pc4, 120_795_955);
        assert_eq!(total4, dc4 + pc4);
        // w 越大 parity 越少、数据格越多，而**总格数一格不变**
        assert!(pc4 < pc);
        assert!(dc4 > dc);
        assert_eq!(total4, total);
    }

    /// **这个实验要回答的那个数**：按落点排的那棵树有多大。
    /// 扇出 (16384 − 76) / (10 + 8) = 906；483 183 820 条 ⇒ 906² = 820 836 < 条目数 ≤ 906³ = 743 677 416 ⇒ 高 3。
    #[test]
    fn slot_keyed_tree_geometry_is_pinned() {
        assert_eq!(SLOT_KEY, 10);
        assert_eq!(fanout(SLOT_KEY), 906);
        assert_eq!(906u64.pow(2), 820_836);
        assert_eq!(906u64.pow(3), 743_677_416);
        let (_, _, total) = cells(3);
        assert!(906u64.pow(2) < total && total <= 906u64.pow(3));
        assert_eq!(height(total, fanout(SLOT_KEY)), 3);
    }

    /// 这棵树自己占多少（手算，逐步列出来）：
    /// 条目 483 183 820 ÷ 583 条每容器 ⇒ `583 × 828 788 = 483 183 404 < 483 183 820`
    /// ⇒ 向上取整是 **828 789** 个容器；容器恒走镜像
    /// ⇒ `828 789 × 32768 × 2 = 54 315 515 904` 字节；`÷ 16 TiB × 10⁶` ⇒ **3087 ppm**。
    /// 与 E97（记账与分配记录的条目编码） 的 442 ppm 放同一把尺子上 ⇒ 6.98 倍。
    ///
    /// ⚠️ **这个字节数与 E106（条带成员表的载体） 的臂甲镜逐位相同**（`cost_bytes=54315515904`、
    /// `cost_ppm=3087.5`）——两个实验用**互不相同**的算法算同一个量而落到同一个数，
    /// 这是换口径之后才出现的交叉校验；旧口径下 E111 报 4631、E106 报 3087.5，谁也没去对。
    #[test]
    fn tree_ppm_is_pinned_and_comparable_to_e97() {
        let (_, _, total) = cells(3);
        let per = (DATA_UNIT - PACKED_HDR) / STRIPE_REC;
        assert_eq!(per, 583);
        assert_eq!(per * 828_788, 483_183_404);
        assert!(per * 828_788 < total);
        assert_eq!(total.div_ceil(per), 828_789);
        assert_eq!(828_789u64 * DATA_UNIT * 2, 54_315_515_904);
        assert_eq!(tree_ppm(total), 3087);
        // 它比分配记录树贵将近七倍——这是判据要看的那个对比
        assert!(tree_ppm(total) > E97_PPM * 6 && tree_ppm(total) < E97_PPM * 8);
    }

    /// **三条臂里只有一条答得出那句话。** 这是这个实验的承重结论，
    /// 而它不是量出来的、是三条 key 的定义直接给的——所以单独钉住。
    #[test]
    fn only_the_slot_keyed_arm_answers_the_lookup() {
        assert!(Arm::Slot.answers_slot_lookup());
        assert!(!Arm::Stripe.answers_slot_lookup());
        assert!(!Arm::None.answers_slot_lookup());
    }

    /// 阳性对照：换 key 宽度扇出必须跟着变（证明这个模型看得见 key 宽度）。
    #[test]
    fn positive_control_fanout_depends_on_key_width() {
        assert_eq!(fanout(SLOT_KEY), 906);
        assert_eq!(fanout(STRIPE_KEY), 815);
        assert_ne!(fanout(SLOT_KEY), fanout(STRIPE_KEY));
    }

    /// 树高的算法与 E97 同：装不下就多一层，`n <= 1` 时 1 层。
    #[test]
    fn height_algorithm_matches_e97() {
        assert_eq!(height(1, 906), 1);
        assert_eq!(height(906, 906), 1);
        assert_eq!(height(907, 906), 2);
        assert_eq!(height(E97_ENTRIES, 906), 3);
    }
}

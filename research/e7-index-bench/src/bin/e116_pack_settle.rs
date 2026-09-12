//! E116：小数据打包容器的账 —— 把 E114 漏掉的元数据写补进来，并把口径拆开。
//!
//! ## 它问什么
//!
//! E114（小数据打包容器的总账） 的写账只数数据单元，`meta_bytes` 是常驻占用不是写；
//! 而第二轮论证把「永久腾出的占用」直接减掉「一次性写出的字节」得出「净收益为负」，
//! 两个量单位不同、中间没有兑换率。E116 拆开三本账，各报各的：
//!
//! | 账 | 单位 | 问什么 |
//! |---|---|---|
//! | 稳态占用 | 字节·常驻 | 同一批活对象在盘上常驻多少 |
//! | 一次性搬迁写 | 字节·一次 | 把它们搬进容器要写多少（容器 + 中央映射 + 分配记录 + journal）|
//! | 回本比 | 无量纲 | 一次性搬迁写 ÷ 永久腾出的占用。**< 1 表示写得比省得少** |
//!
//! 回本比是唯一能把两个量放在同一把尺子上的数：它回答「为了永久腾出 1 字节，要写掉几字节」。
//!
//! ## 被引用条款逐字贴在这里
//!
//! - **D4 已定项 5**：单元恒 32768 含头。**已定项 1**：元数据侧 16 KiB、数据侧 32 KiB。
//! - **D8 已定项 2**：索引节点 16384（`format-const NODE_BYTES`）。
//! - **D18 已定项 7**：单元头三档 68 / 77 / 86。**已定项 11**：打包记录单元头 103。
//! - **D18 已定项 3**：逻辑身份五元组 33 字节。**D19 已定项 4**：位置条目 14 字节。
//! - **D19 已定项 5**：中央映射是解引用唯一入口，value 是 w 份位置条目。
//! - **D3 已定项 7**：分配记录 key =(设备 4, 16 KiB 槽号 6)，value = 分配代 8 ⇒ 条目 18；粒度 16384。
//! - **D23（journal 的角色与格式）**：journal 记录头 78，登记名 JOURNAL_HEADER_BYTES。
//! - **D2 已定项 9**：第一版 2 盘恒 w = 2。
//!
//! ## 三个假设（不是条款，标出来免得当成常量用 —— C187）
//!
//! - `SLOT_TABLE_ENTRY = 4`：槽表每条几字节，写这份装置时**全仓无出处**、D27 第 3 项还没定（今天是已定项 3）。
//! - `LIVE_COUNT_ENTRY = 16`：每容器活槽计数条目，**写这份装置时 D27 第 5 项还没定**（今天是已定项 5：不设活槽计数）。
//! - `JOURNAL_PER_MOVE`：搬一个对象记不记 journal，**写这份装置时 D27 第 6 项还没定**（今天是已定项 6）；两种都跑。
//!
//! ## 四条臂（失败条款逐条点名，覆盖每一条 —— C186）
//!
//! | 臂 | 整理策略 |
//! |---|---|
//! | `pad` | 不打包（现行）|
//! | `bg_key` | 后台整理，沿中央映射 key 序走 ⇒ 映射叶顺序命中 |
//! | `bg_fill` | 后台整理，沿容器凑满序走，批量 `b` ⇒ 映射叶随机散布 |
//! | `bg_ideal` | 后台整理，元数据写记 0（**收益上界对照，不是候选**）|
//!
//! ## 跑前写死的判据与失败条款（跑完不许改）
//!
//! 1. **主判据**：`bg_key` / `bg_fill` / `bg_ideal` 三条臂各自的回本比，在（对象大小 × 批量 b）各格上
//!    报绝对值与符号。**三条臂逐条点名**。
//! 2. **稳态占用**：死槽比例 `d` 从 0 扫到 1，报打包形态的占用**何时超过** `pad`。
//!    两种死亡模型都跑：均匀独立死亡、整容器一起死。
//! 3. **搬迁读**单列，不并进写账。
//! 4. **爆炸半径**：一个单元不可读，`pad` 丢 1 个，三条打包臂各丢几个。
//!
//! **阳性对照，逐臂跑**：容器容量强制为 1 ⇒ `bg_key` / `bg_fill` / `bg_ideal` 三条臂的
//! 稳态占用必须与 `pad` 逐格字节相同。任一臂不同 ⇒ 整轮作废。
//! **判别力对照**：`pad` 在 512 B 与 16384 B 两档占用相同（都占一个单元）。不同 ⇒ 整轮作废。
//! **阴性对照**：N = 0 时四条臂的所有字节恰好 0。
//!
//! ## 反向接受条款（跑前写死，逐臂点名）
//!
//! - 若 **`bg_key` 的回本比 ≥ 1**（沿映射 key 序走仍然写得比省得多）⇒ 结论写
//!   「后台整理形态在最有利的策略下也不省，D27 已定项 1 该退回未定」，不许回头改口径。
//! - 若 **`bg_fill` 在 b 取到实现可达上界时回本比仍 ≥ 1** ⇒ 结论写「凑满序不可用，整理必须按映射 key 序」。
//! - 若 **`bg_ideal` 的回本比 ≥ 1**（连元数据写记 0 都回不了本）⇒ 结论写「数据侧本身不成立，提案放弃」。
//! - 若**打包形态的稳态占用在任何 `d` 上超过 `pad`** ⇒ 结论写「空间收益要等回收定案才成立」。
//! - 若**爆炸半径倍数 ≥ 占用收益倍数** ⇒ 结论写「买到的和赔上的是同一个数」。
//!
//! ## 它答不了的
//!
//! 纯算术账本：没有实现、没有 I/O、没有崩溃点重放。不答挂钟、不答缓存、不答并发。
//! **回收延迟不建模**：死槽扫描假定全死的容器当即被释放，「全死但还没回收」那段时间的占用没算。
//! 不答「这个收益够不够格进格式」——那一问由 `.claude/rules/fs-design.md` 的格式分支判据答，不由数答。
//! 负载是 E116 自定的，不在 D25（目标负载优先级） 的定义域内。

use e7_index_bench::Emitter;

// ── 有出处的格式常量 ────────────────────────────────────────────────
const UNIT: u64 = 32768; // D4 已定项 5
const NODE: u64 = 16384; // D8 已定项 2（format-const NODE_BYTES）
const PACK_HDR: u64 = 103; // D18 已定项 11（format-const PACKED_UNIT_HEADER_BYTES）
const NODE_HDR: u64 = 86; // D18 已定项 7 三档头最宽那档
const W: u64 = 2; // D2 已定项 9
const MAP_KEY: u64 = 33; // D18 已定项 3
const LOC_ENTRY: u64 = 14; // D19 已定项 4
const ALLOC_ENTRY: u64 = 18; // D3 已定项 7
const GRAIN: u64 = 16384; // D3 已定项 7 落点粒度
const JOURNAL_HEADER_BYTES: u64 = 78; // D23（journal 的角色与格式），登记名 JOURNAL_HEADER_BYTES

// ── 假设，不是条款（C187）──────────────────────────────────────────
const SLOT_TABLE_ENTRY: u64 = 4; // 假设：写这份装置时 D27 第 3 项还没定（今天是已定项 3）
const WRITE_SEQ: u64 = 10; // D18 已定项 7 写序
const SLOT_EXTRA: u64 = MAP_KEY + WRITE_SEQ + SLOT_TABLE_ENTRY; // 47

// ── 派生量 ────────────────────────────────────────────────────────
const MAP_ENTRY: u64 = MAP_KEY + LOC_ENTRY * W; // 61
const N: u64 = 100_000;
const SIZES: [u64; 5] = [512, 1024, 4096, 8192, 16384];
const BATCHES: [u64; 6] = [1, 8, 64, 512, 4096, N];

fn capacity(size: u64, slot_extra: u64) -> u64 { (UNIT - PACK_HDR) / (size + slot_extra) }
fn map_leaf_cap() -> u64 { (NODE - NODE_HDR) / MAP_ENTRY }
fn alloc_leaf_cap() -> u64 { (NODE - NODE_HDR) / ALLOC_ENTRY }
fn grains(bytes: u64) -> u64 { bytes.div_ceil(GRAIN) }

/// 批量 b 下，改 n 条散布在 total_leaves 片叶上的条目，一共要 COW 几片叶。
/// b = 1 ⇒ n 片（每条各刷一次）；b = n ⇒ 收敛到 total_leaves。
fn leaves_touched_scattered(n: u64, total_leaves: u64, b: u64) -> u64 {
    if n == 0 || total_leaves == 0 || b == 0 { return 0; }
    let l = total_leaves as f64;
    let per_batch = l * (1.0 - (1.0 - 1.0 / l).powi(b.min(n) as i32));
    let batches = (n as f64 / b.min(n) as f64).ceil();
    (batches * per_batch).round() as u64
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Ledger {
    occupancy: u64,   // 稳态占用（d = 0，全活）
    move_write: u64,  // 一次性搬迁写
    move_read: u64,   // 一次性搬迁读
    blast: u64,       // 一个单元不可读丢几个对象
}

fn pad_ledger(n: u64) -> Ledger {
    Ledger { occupancy: n * UNIT, move_write: 0, move_read: 0, blast: if n == 0 { 0 } else { 1 } }
}

/// policy: 0 = 沿映射 key 序（叶顺序命中）；1 = 凑满序批量 b；2 = 元数据写记 0（上界对照）
fn pack_ledger(n: u64, size: u64, slot_extra: u64, policy: u8, b: u64, journal: bool) -> Ledger {
    let cap = capacity(size, slot_extra);
    if cap == 0 || n == 0 {
        return Ledger { occupancy: n * UNIT, move_write: 0, move_read: 0,
                        blast: if n == 0 { 0 } else { 1 } };
    }
    let containers = n.div_ceil(cap);
    let occupancy = containers * UNIT;

    // 搬迁写：① 容器数据
    let data_w = containers * UNIT * W;

    // ② 中央映射：n 条条目的 value 改了
    let map_leaves_total = n.div_ceil(map_leaf_cap());
    let map_leaves = match policy {
        0 => map_leaves_total,                                   // key 序：每片叶只 COW 一次
        1 => leaves_touched_scattered(n, map_leaves_total, b),   // 凑满序
        _ => 0,
    };
    let map_w = map_leaves * NODE * W;

    // ③ 分配记录：D3 已定项 7 逐字「一条记一个单元、另带跨度段」⇒ 一个单元一条，不按落点算
    let alloc_changed = n + containers;
    let alloc_leaves_total = alloc_changed.div_ceil(alloc_leaf_cap());
    let alloc_leaves = match policy {
        0 => alloc_leaves_total,
        1 => leaves_touched_scattered(alloc_changed, alloc_leaves_total, b),
        _ => 0,
    };
    let alloc_w = alloc_leaves * NODE * W;

    // ④ journal：每搬一个对象一条记录（假设；policy 2 记 0）
    let journal_w = if journal && policy != 2 { n * (JOURNAL_HEADER_BYTES + MAP_ENTRY) } else { 0 };

    Ledger { occupancy, move_write: data_w + map_w + alloc_w + journal_w,
             move_read: n * UNIT, blast: cap }
}

/// 回本比 = 一次性搬迁写 ÷ 永久腾出的占用。< 1 表示写得比省得少。
fn payback(pad: &Ledger, pk: &Ledger) -> f64 {
    let saved = pad.occupancy.saturating_sub(pk.occupancy);
    if saved == 0 { return f64::INFINITY; }
    pk.move_write as f64 / saved as f64
}

/// 死槽比例 d 下的稳态占用。correlated=false 时槽独立死亡（容器全死的概率 d^cap），
/// correlated=true 时整容器一起死（容器存活比例 = 1 − d）。
fn occupancy_at_death(n: u64, size: u64, slot_extra: u64, d: f64, correlated: bool) -> (u64, u64) {
    let pad_live = ((n as f64) * (1.0 - d)).round() as u64 * UNIT;
    let cap = capacity(size, slot_extra);
    if cap == 0 { return (pad_live, pad_live); }
    let containers = n.div_ceil(cap);
    let alive_frac = if correlated { 1.0 - d } else { 1.0 - d.powi(cap as i32) };
    let pack_live = ((containers as f64) * alive_frac).round() as u64 * UNIT;
    (pad_live, pack_live)
}

fn main() {
    let mut em = Emitter::new();
    println!("{}", em.emit_raw(&format!(
        "name=config unit={UNIT} node={NODE} pack_hdr={PACK_HDR} node_hdr={NODE_HDR} w={W} \
         map_entry={MAP_ENTRY} map_leaf_cap={} alloc_entry={ALLOC_ENTRY} alloc_leaf_cap={} \
         journal_hdr={JOURNAL_HEADER_BYTES} slot_extra={SLOT_EXTRA} n={N} sizes={SIZES:?} batches={BATCHES:?}",
        map_leaf_cap(), alloc_leaf_cap())));

    // 阳性对照：容器容量强制为 1 ⇒ 三条打包臂的稳态占用与 pad 逐格相同
    for &size in SIZES.iter() {
        let cap1 = UNIT - PACK_HDR - size; // 让 size + slot_extra 恰好 > (UNIT-HDR)/2
        let pad = pad_ledger(N);
        for (name, policy) in [("bg_key", 0u8), ("bg_fill", 1u8), ("bg_ideal", 2u8)] {
            let pk = pack_ledger(N, size, cap1, policy, 64, true);
            println!("{}", em.emit_raw(&format!(
                "name=positive_cap1 size={size} arm={name} pad_occ={} pack_occ={} same={}",
                pad.occupancy, pk.occupancy, pad.occupancy == pk.occupancy)));
        }
    }

    // 判别力对照
    let p512 = pad_ledger(N); let p16k = pad_ledger(N);
    println!("{}", em.emit_raw(&format!(
        "name=discrimination pad512={} pad16k={} same={}",
        p512.occupancy, p16k.occupancy, p512.occupancy == p16k.occupancy)));

    // 阴性对照
    for (name, policy) in [("bg_key", 0u8), ("bg_fill", 1u8), ("bg_ideal", 2u8)] {
        let z = pack_ledger(0, 512, SLOT_EXTRA, policy, 64, true);
        println!("{}", em.emit_raw(&format!(
            "name=negative arm={name} occ={} mw={} mr={} blast={}",
            z.occupancy, z.move_write, z.move_read, z.blast)));
    }

    // 主判据：回本比
    let pad = pad_ledger(N);
    for &size in SIZES.iter() {
        let cap = capacity(size, SLOT_EXTRA);
        for &journal in [true, false].iter() {
            let key = pack_ledger(N, size, SLOT_EXTRA, 0, 1, journal);
            let ideal = pack_ledger(N, size, SLOT_EXTRA, 2, 1, journal);
            println!("{}", em.emit_raw(&format!(
                "name=payback_key size={size} cap={cap} journal={journal} \
                 pad_occ={} pack_occ={} saved={} move_write={} move_read={} payback={:.6} \
                 ideal_write={} ideal_payback={:.6}",
                pad.occupancy, key.occupancy, pad.occupancy - key.occupancy,
                key.move_write, key.move_read, payback(&pad, &key),
                ideal.move_write, payback(&pad, &ideal))));
            for &b in BATCHES.iter() {
                let fill = pack_ledger(N, size, SLOT_EXTRA, 1, b, journal);
                println!("{}", em.emit_raw(&format!(
                    "name=payback_fill size={size} cap={cap} b={b} journal={journal} \
                     move_write={} payback={:.6}",
                    fill.move_write, payback(&pad, &fill))));
            }
        }
    }

    // 稳态占用 vs 死槽比例
    for &size in SIZES.iter() {
        for &correlated in [false, true].iter() {
            for step in 0..=20u32 {
                let d = step as f64 / 20.0;
                let (pad_o, pack_o) = occupancy_at_death(N, size, SLOT_EXTRA, d, correlated);
                println!("{}", em.emit_raw(&format!(
                    "name=death size={size} correlated={correlated} d={d:.2} \
                     pad_occ={pad_o} pack_occ={pack_o} pack_worse={}",
                    pack_o > pad_o)));
            }
        }
    }

    // 爆炸半径 vs 占用收益倍数
    for &size in SIZES.iter() {
        let pk = pack_ledger(N, size, SLOT_EXTRA, 0, 1, true);
        let gain = pad.occupancy as f64 / pk.occupancy as f64;
        println!("{}", em.emit_raw(&format!(
            "name=blast size={size} gain={gain:.4} blast={} pad_blast=1 ratio={:.4}",
            pk.blast, gain / pk.blast as f64)));
    }

    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 阳性对照：容器只装得下 1 个对象时，打包与不打包的稳态占用必须逐字节相同。
    #[test]
    fn positive_control_cap1_matches_pad() {
        for &size in SIZES.iter() {
            let slot_extra = UNIT - PACK_HDR - size; // cap = 1
            assert_eq!(capacity(size, slot_extra), 1, "size={size} 构造的不是 cap=1");
            for policy in [0u8, 1, 2] {
                let pk = pack_ledger(N, size, slot_extra, policy, 64, true);
                assert_eq!(pk.occupancy, pad_ledger(N).occupancy,
                           "size={size} policy={policy}：cap=1 时占用该与 pad 相同");
            }
        }
    }

    /// 判别力：pad 在最小与最大对象上占用相同（补齐被建模了）。
    #[test]
    fn discrimination_pad_pads() {
        assert_eq!(pad_ledger(N).occupancy, N * UNIT);
        assert_eq!(N * UNIT, 3_276_800_000);
    }

    /// 阴性：N = 0 时全部为 0。
    #[test]
    fn negative_control_zero() {
        for policy in [0u8, 1, 2] {
            let z = pack_ledger(0, 512, SLOT_EXTRA, policy, 64, true);
            assert_eq!((z.occupancy, z.move_write, z.move_read, z.blast), (0, 0, 0, 0));
        }
    }

    /// 叶容量钉绝对值 —— 两条都不许靠互比。
    #[test]
    fn leaf_capacity_absolute() {
        assert_eq!(MAP_ENTRY, 61);
        assert_eq!(NODE, NODE_HDR + 16298);
        assert_eq!(map_leaf_cap(), 267);
        assert_eq!(alloc_leaf_cap(), 905);
    }

    /// 容器容量钉绝对值（跨整数边界的取样点，防取样点不敏感 —— rules/mutation-sampling.md）。
    #[test]
    fn capacity_absolute() {
        assert_eq!(capacity(512, SLOT_EXTRA), 58);
        assert_eq!(capacity(1024, SLOT_EXTRA), 30);
        assert_eq!(capacity(4096, SLOT_EXTRA), 7);
        assert_eq!(capacity(8192, SLOT_EXTRA), 3);
        assert_eq!(capacity(16384, SLOT_EXTRA), 1);
        // 头宽敏感的取样点：头 103 时 1，头 0 时 2
        assert_eq!(capacity(16286, SLOT_EXTRA), 1);
        assert_eq!((UNIT - 0) / (16286 + SLOT_EXTRA), 2);
    }

    /// bg_key 的搬迁写钉绝对值：512 B 档，四项分别算出来再加。
    #[test]
    fn bg_key_move_write_absolute() {
        let cap = capacity(512, SLOT_EXTRA);
        let containers = N.div_ceil(cap);
        assert_eq!(containers, 1725);
        let data_w = containers * UNIT * W;
        assert_eq!(data_w, 113_049_600);
        let map_leaves = N.div_ceil(map_leaf_cap());
        assert_eq!(map_leaves, 375);
        assert_eq!(map_leaves * NODE * W, 12_288_000);
        let alloc_changed = N + containers; // D3 已定项 7：一条记一个单元
        assert_eq!(alloc_changed, 101_725);
        let alloc_leaves = alloc_changed.div_ceil(alloc_leaf_cap());
        assert_eq!(alloc_leaves, 113);
        assert_eq!(alloc_leaves * NODE * W, 3_702_784);
        let journal_w = N * (JOURNAL_HEADER_BYTES + MAP_ENTRY);
        assert_eq!(journal_w, 13_900_000);
        let l = pack_ledger(N, 512, SLOT_EXTRA, 0, 1, true);
        assert_eq!(l.move_write, 113_049_600 + 12_288_000 + 3_702_784 + 13_900_000);
        assert_eq!(l.move_write, 142_940_384);
    }

    /// 反向接受条款要判的那个符号：bg_key 的回本比，逐档钉住。
    #[test]
    fn payback_key_absolute() {
        let pad = pad_ledger(N);
        let l = pack_ledger(N, 512, SLOT_EXTRA, 0, 1, true);
        let saved = pad.occupancy - l.occupancy;
        assert_eq!(saved, 3_220_275_200);
        let p = payback(&pad, &l);
        assert!((p - 0.044388).abs() < 1e-6, "512 B 档回本比实测 {p}");
        // 16 KiB 档 cap = 1 ⇒ 一点不省 ⇒ 回本比无穷
        let l16 = pack_ledger(N, 16384, SLOT_EXTRA, 0, 1, true);
        assert!(payback(&pad, &l16).is_infinite());
    }

    /// bg_fill 在 b = 1 时的回本比 —— 第二轮那个「负」的正确形态。
    #[test]
    fn payback_fill_b1_absolute() {
        let pad = pad_ledger(N);
        let l = pack_ledger(N, 512, SLOT_EXTRA, 1, 1, true);
        // b=1 ⇒ 映射每条各刷一片叶
        assert_eq!(leaves_touched_scattered(N, N.div_ceil(map_leaf_cap()), 1), N);
        let p = payback(&pad, &l);
        assert!(p > 1.0, "b=1 时回本比该 > 1，实测 {p}");
        // ⚠️ 跑前登记的值是 1.058893，**判否**：那个数来自第二轮只算中央映射的账，
        // 而 b=1 的凑满序把分配记录也打散了（101 725 条改动散在 113 片叶上），再加 journal。
        // 三个数都留在这里，不许回头改前两个：跑前登记值 1.058893；
        // 第一版实测 3.127186（分配记录按落点算，一个单元记 2 条 —— 建模错）；
        // 改成 D3 已定项 7 逐字「一条记一个单元」之后 2.092080。
        assert!((p - 2.092080).abs() < 1e-5, "实测 {p}");
    }

    /// bg_ideal 是上界：它的回本比必须严格小于另外两条臂。
    #[test]
    fn ideal_is_upper_bound() {
        let pad = pad_ledger(N);
        for &size in SIZES.iter() {
            if capacity(size, SLOT_EXTRA) <= 1 { continue; }
            let ideal = payback(&pad, &pack_ledger(N, size, SLOT_EXTRA, 2, 1, true));
            let key = payback(&pad, &pack_ledger(N, size, SLOT_EXTRA, 0, 1, true));
            let fill = payback(&pad, &pack_ledger(N, size, SLOT_EXTRA, 1, 1, true));
            assert!(ideal < key && key < fill, "size={size}: ideal={ideal} key={key} fill={fill}");
        }
        // 上界本身钉绝对值：512 B 档只剩容器数据写
        let ideal = pack_ledger(N, 512, SLOT_EXTRA, 2, 1, true);
        assert_eq!(ideal.move_write, 113_049_600);
    }

    /// 回本比的结构性下界是 w/(cap−1)，与实现无关 —— 元数据写记 0 那条臂正好落在它上面。
    /// ⇒ D27 回得了本当且仅当 cap > w + 1。w = 2 时要 cap ≥ 4：
    /// 4 KiB 那档 cap = 7 过得去，8 KiB 那档 cap = 3 恰好等于 1、一个字节都不赚。
    #[test]
    fn ideal_payback_is_w_over_cap_minus_one() {
        let pad = pad_ledger(N);
        for &size in SIZES.iter() {
            let cap = capacity(size, SLOT_EXTRA);
            if cap <= 1 { continue; }
            let got = payback(&pad, &pack_ledger(N, size, SLOT_EXTRA, 2, 1, true));
            let closed = W as f64 / (cap - 1) as f64;
            assert!((got - closed).abs() < 1e-4,
                    "size={size} cap={cap}: 实测 {got}，闭式 {closed}");
        }
        // 钉绝对值，不许只靠与闭式互比
        assert!((W as f64 / (58.0 - 1.0) - 0.035088).abs() < 1e-6);
        assert!((W as f64 / (3.0 - 1.0) - 1.0).abs() < 1e-12);
        // cap = w + 1 正是不赚不赔那一点
        assert_eq!(capacity(8192, SLOT_EXTRA), W + 1);
    }

    /// 批量单调：凑满序的搬迁写随 b 增大单调不增，且 b = N 时收敛到 key 序。
    #[test]
    fn fill_converges_to_key_order() {
        let mut prev = u64::MAX;
        for &b in BATCHES.iter() {
            let w = pack_ledger(N, 512, SLOT_EXTRA, 1, b, true).move_write;
            assert!(w <= prev, "b={b} 的搬迁写该不增，前一格 {prev} 本格 {w}");
            prev = w;
        }
        // ⚠️ 跑前登记的「b = N 收敛到 key 序」**判否**，而且判得有道理：
        // 中央映射那棵树确实收敛（改 100 000 条、b = 100 000 ⇒ 一批，命中全部 375 片叶），
        // 但分配记录改 101 725 条 > b = 100 000 ⇒ 仍要 2 批、每批命中全部 113 片叶。
        // 「一次攒完」对两棵树不是同一个 b。三条断言都钉绝对值：
        assert_eq!(leaves_touched_scattered(N, N.div_ceil(map_leaf_cap()), N), 375);
        let alloc_changed = N + N.div_ceil(capacity(512, SLOT_EXTRA));
        assert_eq!(alloc_changed, 101_725);
        assert_eq!(leaves_touched_scattered(alloc_changed, alloc_changed.div_ceil(alloc_leaf_cap()), N), 226);
        let key = pack_ledger(N, 512, SLOT_EXTRA, 0, 1, true).move_write;
        let fill_n = pack_ledger(N, 512, SLOT_EXTRA, 1, N, true).move_write;
        assert_eq!(key, 142_940_384);
        assert_eq!(fill_n, 146_643_168);
    }

    /// 稳态占用：均匀独立死亡下，打包形态在任何 d 上都不比 pad 差。
    #[test]
    fn death_uniform_never_worse() {
        for &size in SIZES.iter() {
            for step in 0..=20u32 {
                let d = step as f64 / 20.0;
                let (pad_o, pack_o) = occupancy_at_death(N, size, SLOT_EXTRA, d, false);
                assert!(pack_o <= pad_o, "size={size} d={d}: pack={pack_o} pad={pad_o}");
            }
        }
    }

    /// 均匀独立死亡的绝对值 —— `death_uniform_never_worse` 是互比断言，
    /// 单靠它分不出「容器全死的概率是 d^cap 还是 d」（变异 M14 实测一个测试都没红，
    /// 按 `.claude/rules/mutation-sampling.md` 三分是**真盲区**，不是等价也不是取样点不敏感）。
    #[test]
    fn death_uniform_absolute() {
        // 512 B、cap = 58、d = 0.5：58 个槽同时死的概率是 0.5^58 ≈ 3.5e-18 ⇒ 容器几乎全活
        let (pad_o, pack_o) = occupancy_at_death(N, 512, SLOT_EXTRA, 0.5, false);
        assert_eq!(pad_o, 50_000 * UNIT);
        assert_eq!(pack_o, 1725 * UNIT);
        // d = 0.95 时 0.95^58 ≈ 0.0512 ⇒ 仍有 95% 的容器活着
        let (_, pack95) = occupancy_at_death(N, 512, SLOT_EXTRA, 0.95, false);
        assert_eq!(pack95, 1637 * UNIT);
        // cap = 3 那档对 d 敏感得多：0.5^3 = 0.125 ⇒ 87.5% 的容器活着
        let (_, pack8k) = occupancy_at_death(N, 8192, SLOT_EXTRA, 0.5, false);
        assert_eq!(pack8k, 29_167 * UNIT);
    }

    /// 整容器一起死时也不差 —— 但这一条是钉绝对值的，不是互比。
    #[test]
    fn death_correlated_absolute() {
        let (pad_o, pack_o) = occupancy_at_death(N, 512, SLOT_EXTRA, 0.5, true);
        assert_eq!(pad_o, 50_000 * UNIT);
        assert_eq!(pack_o, 863 * UNIT); // round(1725 × 0.5)
        assert!(pack_o < pad_o);
    }

    /// 爆炸半径与占用收益倍数是同一个数。
    #[test]
    fn blast_equals_gain() {
        let pad = pad_ledger(N);
        for &(size, cap) in [(512u64, 58u64), (1024, 30), (4096, 7), (8192, 3)].iter() {
            let pk = pack_ledger(N, size, SLOT_EXTRA, 0, 1, true);
            assert_eq!(pk.blast, cap);
            let gain = pad.occupancy as f64 / pk.occupancy as f64;
            assert!(gain <= cap as f64 && gain > cap as f64 * 0.88,
                    "size={size}: gain={gain} cap={cap}");
        }
    }

    /// 搬迁读钉绝对值：读的是每个对象所在的整单元。
    #[test]
    fn move_read_absolute() {
        let l = pack_ledger(N, 512, SLOT_EXTRA, 0, 1, true);
        assert_eq!(l.move_read, 3_276_800_000);
    }

    /// journal 那一项是假设：关掉它，512 B 档的搬迁写正好少 13 900 000。
    #[test]
    fn journal_is_an_assumption() {
        let with = pack_ledger(N, 512, SLOT_EXTRA, 0, 1, true).move_write;
        let without = pack_ledger(N, 512, SLOT_EXTRA, 0, 1, false).move_write;
        assert_eq!(with, without + 13_900_000);
    }
}

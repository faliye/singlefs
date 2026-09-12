//! E114：小数据打包容器（A 形态）的总账 —— 三轮九腿之后唯一没算过的那个数。
//!
//! ## 它问什么
//!
//! 2026-09-07 第二轮辩方复核腿的总判逐字：「它取决于这一个数：在一个写下来的小文件负载上,
//! 『数据侧省下的字节 − 容器索引与多一层间接付出的字节』是不是正的，以及它在交错负载下会不会翻负。」
//!
//! ## 被引用条款逐字贴在这里
//!
//! - **D4 已定项 5**：单元恒 32768 含头。**已定项 2**：短 extent 补齐到整单元，
//!   「格式里没有『变长单元』这条分支」。**已定项 3**：凑不满一个单元的写走读—改—写。
//! - **D18 已定项 7**：数据单元头 105。**已定项 11**：打包记录单元头 107；
//!   别的树按（出生树 8 + 打包记录类型 2 + 容器号 8 + 容器出生代 8）= 26 字节引用容器；
//!   **四条装载纪律②「一容器一个代际」** ⇒ 容器不跨发布累积，这是本实验的主自变量的由来。
//! - **D19 已定项 4**：位置条目 14 字节（设备 4 + 物理偏移 6，编码为 16 KiB 槽号 + 密文校验和 4）。
//!   **已定项 5**：中央映射是解引用唯一入口，key = 逻辑身份五元组 33 字节，value = w 份位置条目。
//! - **D3 已定项 7**：分配记录 key =(设备身份 4, 16 KiB 槽号 6)，value = 分配代 8 ⇒ 条目 18；落点粒度 16384。
//! - **D2 已定项 9**：第一版 2 盘恒 w = 2。**D1 已定项 5**：自包不可变数据带唯一一条指向（五元组 33）。
//! - **D18 已定项 7** 写序 10。⇒ 每槽全额自描述 = 五元组 33 + 写序 10 + 槽表条目 4 = 47。
//!
//! ## 三条臂
//!
//! | 臂 | 小对象住哪 | 每槽额外 |
//! |---|---|---|
//! | `pad` | 独立数据 extent，补齐到 32768（现行） | — |
//! | `pack` | 共享容器，**每槽带全额自描述** | 47 |
//! | `packmin` | 共享容器，只带槽表条目（**买不到身份，是收益上界对照，不是候选**） | 4 |
//!
//! ## 判据（跑前写死，跑完不许改）
//!
//! 1. **主判据**：总账 = (pad 数据 + pad 元数据) − (pack 数据 + pack 元数据)，
//!    在（对象大小 × m × 覆写比例 f）各格上报**符号与绝对值**；并报出 pack 不再省的 m 临界值。
//!    元数据两臂都算：中央映射条目、分配记录条目、容器索引条目、活槽记账条目。
//! 2. **收益关系**：验证「数据侧收益 = min(m, 容器容量)」这条解析式在各格上成立（相对误差 < 1%）。
//! 3. **覆写衰减**：f 从 0 扫到 1，报 pack 总占用追平 pad 的那个 f。
//! 4. **整理代价**：回收死槽要读+写受影响容器，报「整理一次的字节 ÷ 该负载填充期省下的字节」。
//! 5. **故障域**：一个 32 KiB 单元不可读，两臂各丢多少对象。
//!
//! ## 失败条款（跑前写死）
//!
//! - **阳性对照，逐臂跑**：把容器容量强制设为 1，`pack` 与 `packmin` 的**数据侧**必须与 `pad` 逐格字节相同。
//!   不同 ⇒ 两条臂不在同一把尺子上，整轮作废。
//! - **判别力对照**：`pad` 在 512 B 与 16384 B 两档上数据侧占用必须完全相同（都占一个单元）。
//!   不同 ⇒ 补齐没被建模，整轮作废。
//! - **阴性对照**：N = 0 时三条臂的所有字节必须恰好 0。
//! - **读不到 ≠ 读到 0**：任何一格算出 0 而它不该是 0，整轮作废（由绝对值断言兜住）。
//!
//! ## 反向接受条款（跑前写死，这一条是本实验能不能推翻提案的全部重量）
//!
//! - 若 **m ≥ 8** 的格上总账为负（pack 更费字节）⇒ 结论写「A 形态不省，提案放弃」，**不许回头改口径**。
//! - 若总账为正但**收益 < 1.94 倍** ⇒ 结论写「A 不如 B 形态（B 已算出 1.88–1.94 倍且不碰身份/AAD/反向索引），
//!   提案改走 B」。1.94 是 2026-09-07 对 B 形态算出的 512 B 档收益，写死在这里当判据。
//! - 若**整理一次的字节 ≥ 填充期省下的字节** ⇒ 结论写「收益被整理吃掉」。
//!
//! ## 它答不了的
//!
//! 纯算术账本：没有实现、没有 I/O、没有崩溃点重放。不答挂钟、不答缓存、不答并发。
//! 负载是**本实验自定**的——D25 目标负载里「小文件」出现 0 次，这一格不在它的定义域内。
//! 整理触发策略未定（D26 未定项 4），本实验只算「整理一次」的字节，不算它多久跑一次。

use e7_index_bench::Emitter;

// ── 格式常量，全部有出处 ────────────────────────────────────────────────
const UNIT: u64 = 32768; // D4 已定项 5
const DATA_UNIT_HEADER_BYTES: u64 = 105; // D18 已定项 7
const PACK_HDR: u64 = 107; // D18 已定项 11
const GRAIN: u64 = 16384; // D3 已定项 7 落点粒度
const W: u64 = 2; // D2 已定项 9
const MAP_KEY: u64 = 33; // D19 已定项 5，逻辑身份五元组
const LOC_ENTRY: u64 = 14; // D19 已定项 4
const ALLOC_ENTRY: u64 = 18; // D3 已定项 7：key 10 + value 8
const CONTAINER_REF: u64 = 26; // D18 已定项 11：出生树 8 + 类型 2 + 容器号 8 + 出生代 8
const SLOT_FULL: u64 = 47; // 五元组 33 + 写序 10 + 槽表 4
const SLOT_MIN: u64 = 4; // 只有槽表条目
const LIVE_COUNT_ENTRY: u64 = 16; // 活槽记账：容器号 8 + 计数 8

const SIZES: [u64; 5] = [512, 1024, 4096, 8192, 16384];
const MS: [u64; 6] = [1, 2, 8, 30, 100, 1000];
const FS: [f64; 5] = [0.0, 0.1, 0.3, 0.5, 0.9];
const N: u64 = 100_000;

#[derive(Debug, Clone, Copy, PartialEq)]
struct Ledger {
    data_bytes: u64,
    meta_bytes: u64,
    write_bytes: u64,
    units: u64,
    objects_lost_per_unit: u64,
}

impl Ledger {
    fn total(&self) -> u64 { self.data_bytes + self.meta_bytes }
}

/// 一个容器装得下几个对象。`slot_extra` 是每槽的额外开销。
fn capacity(size: u64, slot_extra: u64) -> u64 {
    let avail = UNIT - PACK_HDR;
    let per = size + slot_extra;
    if per == 0 { return 0; }
    avail / per
}

/// 装载纪律②「一容器一个代际」下，n 个对象按每次发布 m 个落地，一共产生几个容器。
/// 末次发布只有 n % m 个对象，单独算——按满算会凭空多出容器（阳性对照抓到过）。
fn containers_of(n: u64, m: u64, cap: u64) -> u64 {
    if m == 0 || cap == 0 { return 0; }
    let full = n / m;
    let rem = n % m;
    full * m.div_ceil(cap) + rem.div_ceil(cap)
}

/// 一个对象占几个 16 KiB 落点（分配记录按落点计）。
fn grains(bytes: u64) -> u64 { bytes.div_ceil(GRAIN) }

/// `pad` 臂：每个对象一个补齐到整单元的 extent。
fn pad_ledger(n: u64, _size: u64, m: u64, f: f64) -> Ledger {
    let units = n;
    let data_bytes = units * UNIT;
    // 元数据：中央映射每对象一条；分配记录按落点。
    let map = n * (MAP_KEY + LOC_ENTRY * W);
    let alloc = units * grains(UNIT) * ALLOC_ENTRY;
    // 写：首次每对象一个整单元 × w；覆写走 RMW（读不计入写字节，写仍是一个整单元）。
    let overwritten = (n as f64 * f).round() as u64;
    let write_bytes = (n + overwritten) * UNIT * W;
    let _ = m; // pad 的写与发布节奏无关：每个对象自己一个单元
    Ledger { data_bytes, meta_bytes: map + alloc, write_bytes, units,
             objects_lost_per_unit: 1 }
}

/// `pack` 臂：装载纪律②「一容器一个代际」⇒ 一次发布落 m 个对象就产生 ⌈m/cap⌉ 个容器。
fn pack_ledger(n: u64, size: u64, m: u64, f: f64, slot_extra: u64) -> Ledger {
    let cap = capacity(size, slot_extra);
    if cap == 0 { return pad_ledger(n, size, m, f); }
    // ⚠️ 末次发布不满 m 个对象，不许按满算（2026-09-07 阳性对照抓到的建模 bug）
    let containers = containers_of(n, m, cap);
    // 覆写：搬出成独立单元，旧槽死、容器不释放（不整理）
    let overwritten = (n as f64 * f).round() as u64;
    let units = containers + overwritten;
    let data_bytes = units * UNIT;
    // 元数据：中央映射每对象一条（两臂同）+ 分配记录按落点 + 容器索引每容器一条 + 活槽记账每容器一条
    let map = n * (MAP_KEY + LOC_ENTRY * W);
    let alloc = units * grains(UNIT) * ALLOC_ENTRY;
    let cidx = containers * (CONTAINER_REF + LOC_ENTRY * W);
    let live = containers * LIVE_COUNT_ENTRY;
    // 写：开放容器每次发布被 COW 重写一次（C105）；覆写搬出各写一个整单元
    let write_bytes = (containers + overwritten) * UNIT * W;
    Ledger { data_bytes, meta_bytes: map + alloc + cidx + live, write_bytes, units,
             objects_lost_per_unit: cap.min(m) }
}

/// `packbg` 臂（2026-09-07 第二次推进时加）：**写路径一个字节都不改**——每个对象先照现行
/// 规则独立成单元；由 D26 的常驻整理在对象写完之后把它搬进容器。
///
/// 关键差别：容器数 = ⌈n/cap⌉，**与发布节奏 m 无关**。m 由负载变量变成整理批量，
/// 而整理批量是实现能调的 ⇒ 收益不再挂在「用户多久 fsync 一次」上。
/// 顺带解掉硬要求 5：判据从「正在写的这个文件多大」变成「这个已经写完的对象多大」,
/// 分支变量在操作期间不存在。
///
/// 代价：小对象要落盘两次（先独立单元、后容器），且**整理跑完之前一点空间都没省**。
fn pack_bg_ledger(n: u64, size: u64, m: u64, f: f64, slot_extra: u64) -> Ledger {
    let cap = capacity(size, slot_extra);
    if cap == 0 { return pad_ledger(n, size, m, f); }
    let overwritten = (n as f64 * f).round() as u64;
    let containers = n.div_ceil(cap); // 与 m 无关
    let units = containers + overwritten;
    let data_bytes = units * UNIT;
    let map = n * (MAP_KEY + LOC_ENTRY * W);
    let alloc = units * grains(UNIT) * ALLOC_ENTRY;
    let cidx = containers * (CONTAINER_REF + LOC_ENTRY * W);
    let live = containers * LIVE_COUNT_ENTRY;
    // 写两次：首次每对象一个独立单元（与 pad 逐字节相同）+ 整理每容器写一次
    let write_bytes = (n + containers + overwritten) * UNIT * W;
    Ledger { data_bytes, meta_bytes: map + alloc + cidx + live, write_bytes, units,
             objects_lost_per_unit: cap }
}

/// 整理一次：把死槽回收，需要读+写所有含死槽的容器。返回写字节（读同量）。
fn compaction_bytes(n: u64, size: u64, m: u64, f: f64, slot_extra: u64) -> u64 {
    let cap = capacity(size, slot_extra);
    if cap == 0 { return 0; }
    let containers = containers_of(n, m, cap);
    let overwritten = (n as f64 * f).round() as u64;
    // 被碰到的容器数：overwritten 个死槽均匀散在 containers 个容器上
    let touched = if containers == 0 { 0 } else {
        let p = 1.0 - (1.0 - 1.0 / containers as f64).powi(overwritten as i32);
        (containers as f64 * p).round() as u64
    };
    touched * UNIT * W
}

fn main() {
    let mut em = Emitter::new();
    println!("{}", em.emit_raw(&format!(
        "name=config unit={UNIT} unit_hdr={DATA_UNIT_HEADER_BYTES} pack_hdr={PACK_HDR} grain={GRAIN} w={W} \
         map_key={MAP_KEY} loc={LOC_ENTRY} alloc={ALLOC_ENTRY} cref={CONTAINER_REF} \
         slot_full={SLOT_FULL} slot_min={SLOT_MIN} live={LIVE_COUNT_ENTRY} n={N} \
         sizes={SIZES:?} ms={MS:?} fs={FS:?}")));

    // 阳性对照：容器容量强制为 1 ⇒ 数据侧必须与 pad 逐格相同
    for &size in SIZES.iter() {
        for &m in MS.iter() {
            let p = pad_ledger(N, size, m, 0.0);
            let forced = {
                let cap1_extra = UNIT - PACK_HDR - size; // 逼 capacity() 恰好给 1
                pack_ledger(N, size, m, 0.0, cap1_extra)
            };
            let forced_bg = {
                let cap1_extra = UNIT - PACK_HDR - size;
                pack_bg_ledger(N, size, m, 0.0, cap1_extra)
            };
            println!("{}", em.emit_raw(&format!(
                "name=positive_cap1 size={size} m={m} pad_data={} pack_data={} packbg_data={} \
                 equal={} equal_bg={}",
                p.data_bytes, forced.data_bytes, forced_bg.data_bytes,
                p.data_bytes == forced.data_bytes, p.data_bytes == forced_bg.data_bytes)));
        }
    }

    // 判别力对照：pad 在 512 与 16384 上数据侧必须相同
    let d0 = pad_ledger(N, 512, 8, 0.0).data_bytes;
    let d1 = pad_ledger(N, 16384, 8, 0.0).data_bytes;
    println!("{}", em.emit_raw(&format!(
        "name=discrimination pad_512={d0} pad_16384={d1} equal={}", d0 == d1)));

    // 阴性对照
    let z = pack_ledger(0, 1024, 8, 0.0, SLOT_FULL);
    println!("{}", em.emit_raw(&format!(
        "name=negative n0_data={} n0_meta={} n0_write={}", z.data_bytes, z.meta_bytes, z.write_bytes)));

    // 主判据：总账
    for &size in SIZES.iter() {
        for &m in MS.iter() {
            for &f in FS.iter() {
                let p = pad_ledger(N, size, m, f);
                let k = pack_ledger(N, size, m, f, SLOT_FULL);
                let kmin = pack_ledger(N, size, m, f, SLOT_MIN);
                let ledger = p.total() as i64 - k.total() as i64;
                let ratio = p.total() as f64 / k.total() as f64;
                let data_ratio = p.data_bytes as f64 / k.data_bytes as f64;
                let bg = pack_bg_ledger(N, size, m, f, SLOT_FULL);
                let comp = compaction_bytes(N, size, m, f, SLOT_FULL);
                println!("{}", em.emit_raw(&format!(
                    "name=ledger size={size} m={m} f={f} \
                     pad_total={} pack_total={} packmin_total={} net={ledger} ratio={ratio:.4} \
                     data_ratio={data_ratio:.4} pad_write={} pack_write={} compaction={comp} \
                     packbg_total={} packbg_write={} packbg_peak={} \
                     cap={} lost_pad={} lost_pack={}",
                    p.total(), k.total(), kmin.total(), p.write_bytes, k.write_bytes,
                    bg.total(), bg.write_bytes, N * UNIT,
                    capacity(size, SLOT_FULL), p.objects_lost_per_unit, k.objects_lost_per_unit)));
            }
        }
    }

    // 收益关系：数据侧收益 ?= min(m, cap)
    for &size in SIZES.iter() {
        for &m in MS.iter() {
            let cap = capacity(size, SLOT_FULL);
            let p = pad_ledger(N, size, m, 0.0);
            let k = pack_ledger(N, size, m, 0.0, SLOT_FULL);
            let measured = p.data_bytes as f64 / k.data_bytes as f64;
            let pre = m.min(cap).max(1) as f64;
            let found = m as f64 / m.div_ceil(cap) as f64;
            let rel = ((measured - found) / found).abs();
            println!("{}", em.emit_raw(&format!(
                "name=benefit_law size={size} m={m} cap={cap} measured={measured:.4} \
                 preregistered={pre:.4} found={found:.4} rel_err={rel:.4}")));
        }
    }

    // 覆写衰减：找追平点
    for &size in SIZES.iter() {
        for m in [8u64, 30, 100] {
            let mut breakeven = -1.0f64;
            let mut fx = 0.0f64;
            while fx <= 40.0 {
                let p = pad_ledger(N, size, m, 0.0);
                let k = pack_ledger(N, size, m, fx, SLOT_FULL);
                if k.total() >= p.total() { breakeven = fx; break; }
                fx += 0.05;
            }
            println!("{}", em.emit_raw(&format!(
                "name=overwrite_breakeven size={size} m={m} f_breakeven={breakeven:.2}")));
        }
    }
    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 绝对值：pad 在 100k 个对象上恰好 3 276 800 000 字节数据侧。
    #[test]
    fn pad_absolute() {
        assert_eq!(pad_ledger(N, 1024, 8, 0.0).data_bytes, 3_276_800_000);
        assert_eq!(pad_ledger(N, 1024, 8, 0.0).units, 100_000);
        // 元数据绝对值：映射 100000×(33+28) + 分配 100000×2×18
        assert_eq!(pad_ledger(N, 1024, 8, 0.0).meta_bytes, 100_000 * 61 + 100_000 * 2 * 18);
    }

    /// 绝对值：容器容量按 (32768−107)/(size+47) 定，四档逐个钉死。
    #[test]
    fn capacity_absolute() {
        assert_eq!(capacity(512, SLOT_FULL), 32661 / 559);
        assert_eq!(capacity(512, SLOT_FULL), 58);
        assert_eq!(capacity(1024, SLOT_FULL), 30);
        assert_eq!(capacity(4096, SLOT_FULL), 7);
        assert_eq!(capacity(16384, SLOT_FULL), 1);
        // 上界形态
        assert_eq!(capacity(1024, SLOT_MIN), 31);
        // ⚠️ 容器头必须真的被扣掉。SIZES 那四档对头不敏感——107 改成 0 算出的容量逐档不变
        // （2026-09-07 变异 M1 一个测试都没红，暴露的是取样点不敏感，不是等价变异）。
        // 16286 + 47 = 16333 这一格跨着整数边界：头 107 时 32661/16333 = 1，头 0 时 32768/16333 = 2。
        assert_eq!(capacity(16286, SLOT_FULL), 1);
    }

    /// 阳性对照：容量逼成 1 时数据侧与 pad 逐格相同。
    #[test]
    fn positive_control_cap1_matches_pad() {
        for &size in SIZES.iter() {
            for &m in MS.iter() {
                let extra = UNIT - PACK_HDR - size;
                assert_eq!(capacity(size, extra), 1, "size={size} 容量应恰为 1");
                assert_eq!(pack_ledger(N, size, m, 0.0, extra).data_bytes,
                           pad_ledger(N, size, m, 0.0).data_bytes,
                           "size={size} m={m} 容量为 1 时数据侧应与 pad 相同");
            }
        }
    }

    /// 判别力：pad 在 512 与 16384 上数据侧相同（补齐被建模）。
    #[test]
    fn discrimination_pad_pads() {
        assert_eq!(pad_ledger(N, 512, 8, 0.0).data_bytes, pad_ledger(N, 16384, 8, 0.0).data_bytes);
        assert_eq!(pad_ledger(N, 512, 8, 0.0).data_bytes, N * UNIT);
    }

    /// 阴性对照：N=0 时全 0。
    #[test]
    fn negative_control_zero() {
        let z = pack_ledger(0, 1024, 8, 0.0, SLOT_FULL);
        assert_eq!(z.data_bytes, 0);
        assert_eq!(z.meta_bytes, 0);
        assert_eq!(z.write_bytes, 0);
        assert_eq!(z.units, 0);
    }

    /// 收益关系：数据侧收益 = min(m, cap)，四档 × 六个 m 全格。
    #[test]
    fn benefit_law_holds() {
        for &size in SIZES.iter() {
            for &m in MS.iter() {
                let cap = capacity(size, SLOT_FULL);
                let p = pad_ledger(N, size, m, 0.0);
                let k = pack_ledger(N, size, m, 0.0, SLOT_FULL);
                let measured = p.data_bytes as f64 / k.data_bytes as f64;
                // 跑前写死的解析式 min(m, cap) —— 2026-09-07 实测**不成立**（m=100/cap=58 那格
                // 实测 50、预测 58）。判据 2 判否，如实留在这里，不许事后改成对的那个。
                let predicted_preregistered = m.min(cap).max(1) as f64;
                // 实验发现的正确形式：一次发布产生 ⌈m/cap⌉ 个容器 ⇒ 收益 = m / ⌈m/cap⌉。
                let predicted_found = m as f64 / m.div_ceil(cap) as f64;
                let rel = ((measured - predicted_found) / predicted_found).abs();
                assert!(rel < 0.01,
                        "size={size} m={m} 实测 {measured} 发现式 {predicted_found}（跑前式 {predicted_preregistered}）");
            }
        }
    }

    /// m = 1 时收益必须恰好为 1（一容器一个代际 ⇒ 每次发布只落一个对象就填不满）。
    #[test]
    fn m1_gives_no_benefit() {
        for &size in SIZES.iter() {
            let p = pad_ledger(N, size, 1, 0.0);
            let k = pack_ledger(N, size, 1, 0.0, SLOT_FULL);
            assert_eq!(p.data_bytes, k.data_bytes, "size={size} m=1 时不该有收益");
        }
    }

    /// 写字节绝对值：pack 在 m=30、1 KiB、f=0 时恰为 publishes×1×32768×2。
    #[test]
    fn write_bytes_absolute() {
        let k = pack_ledger(N, 1024, 30, 0.0, SLOT_FULL);
        let publishes = 100_000u64.div_ceil(30);
        assert_eq!(publishes, 3334);
        assert_eq!(k.write_bytes, publishes * UNIT * W);
        assert_eq!(k.write_bytes, 3334 * 65536);
        // pad 同格
        assert_eq!(pad_ledger(N, 1024, 30, 0.0).write_bytes, 100_000 * 65536);
    }

    /// 故障域：一个单元丢多少对象，两臂各一个绝对值。
    #[test]
    fn blast_radius_absolute() {
        assert_eq!(pad_ledger(N, 1024, 30, 0.0).objects_lost_per_unit, 1);
        assert_eq!(pack_ledger(N, 1024, 30, 0.0, SLOT_FULL).objects_lost_per_unit, 30);
        assert_eq!(pack_ledger(N, 1024, 8, 0.0, SLOT_FULL).objects_lost_per_unit, 8);
        assert_eq!(pack_ledger(N, 512, 100, 0.0, SLOT_FULL).objects_lost_per_unit, 58);
    }

    /// packbg 的容器数与发布节奏无关 —— 这是这条臂存在的全部理由，必须有断言守着。
    #[test]
    fn packbg_is_independent_of_publish_cadence() {
        for &size in SIZES.iter() {
            let base = pack_bg_ledger(N, size, 1, 0.0, SLOT_FULL).data_bytes;
            for &m in MS.iter() {
                assert_eq!(pack_bg_ledger(N, size, m, 0.0, SLOT_FULL).data_bytes, base,
                           "size={size} m={m}：packbg 的占用不该随 m 变");
            }
            // 而在线形态 pack 是随 m 变的（否则两条臂没区别，这条断言就是判别力）
            if capacity(size, SLOT_FULL) > 1 {
                assert_ne!(pack_ledger(N, size, 1, 0.0, SLOT_FULL).data_bytes,
                           pack_ledger(N, size, 30, 0.0, SLOT_FULL).data_bytes,
                           "size={size}：在线形态本该随 m 变");
            }
        }
    }

    /// packbg 绝对值：1 KiB、cap=30 ⇒ 容器 3334；写 = (100000+3334) 个整单元 × w。
    #[test]
    fn packbg_absolute() {
        let bg = pack_bg_ledger(N, 1024, 1, 0.0, SLOT_FULL);
        assert_eq!(bg.data_bytes, 3334 * UNIT);
        assert_eq!(bg.write_bytes, (100_000 + 3334) * UNIT * W);
        assert_eq!(bg.write_bytes, 103_334 * 65536);
        // 写字节相对现行只多 1/cap
        let pad = pad_ledger(N, 1024, 1, 0.0);
        let ratio = bg.write_bytes as f64 / pad.write_bytes as f64;
        assert!((ratio - 1.03334).abs() < 1e-5, "写字节比应为 1.03334，实得 {ratio}");
        // 元数据钉绝对值（2026-09-12 补：M13 / M14 此前一个测试都没红，是真盲区）：
        // 映射 100000×(33+28) + 分配 3334×2×18 + 容器索引 3334×(26+28) + 活槽记账 3334×16
        assert_eq!(bg.meta_bytes, 100_000 * 61 + 3334 * 2 * 18 + 3334 * 54 + 3334 * 16);
        assert_eq!(bg.meta_bytes, 6_453_404);
        // 覆写取样点（2026-09-12 补：f = 0 时 M15 与原式同值，是取样点不敏感）：f=0.3 ⇒ 30000 个对象搬出，各占一个整单元
        let bg_overwrite = pack_bg_ledger(N, 1024, 1, 0.3, SLOT_FULL);
        assert_eq!(bg_overwrite.units, 3334 + 30_000);
        assert_eq!(bg_overwrite.data_bytes, (3334 + 30_000) * UNIT);
    }

    /// 覆写路径绝对值：f=0.3 ⇒ 30000 个对象搬出，各占一个整单元，旧容器不释放。
    #[test]
    fn overwrite_absolute() {
        let containers = containers_of(N, 30, capacity(1024, SLOT_FULL));
        assert_eq!(containers, 3334);
        let k = pack_ledger(N, 1024, 30, 0.3, SLOT_FULL);
        assert_eq!(k.units, containers + 30_000);
        assert_eq!(k.data_bytes, (3334 + 30_000) * UNIT);
        assert_eq!(k.write_bytes, (3334 + 30_000) * UNIT * W);
    }

    /// 整理绝对值：f=0.3、m=30 时触及率饱和 ⇒ 全部 3334 个容器被读写一遍。
    #[test]
    fn compaction_absolute() {
        assert_eq!(compaction_bytes(N, 1024, 30, 0.3, SLOT_FULL), 3334 * UNIT * W);
        assert_eq!(compaction_bytes(N, 1024, 30, 0.3, SLOT_FULL), 218_497_024);
        // f = 0 时没有死槽，整理为 0
        assert_eq!(compaction_bytes(N, 1024, 30, 0.0, SLOT_FULL), 0);
    }

    /// 元数据侧确实多付：容器索引 + 活槽记账只有 pack 有，且数额可核。
    #[test]
    fn pack_pays_more_metadata() {
        let size = 1024;
        let m = 30;
        let p = pad_ledger(N, size, m, 0.0);
        let k = pack_ledger(N, size, m, 0.0, SLOT_FULL);
        let containers = 100_000u64.div_ceil(m) * m.div_ceil(capacity(size, SLOT_FULL));
        assert_eq!(containers, 3334);
        let extra = containers * (CONTAINER_REF + LOC_ENTRY * W) + containers * LIVE_COUNT_ENTRY;
        assert_eq!(extra, 3334 * (26 + 28) + 3334 * 16);
        // ⚠️ 这里不断言方向：谁的元数据多是本实验要测的东西之一，
        // 把它写成 assert 就是「先有结论再建模型」。只钉住两边各自可核的绝对值。
        assert_eq!(k.meta_bytes,
                   N * (MAP_KEY + LOC_ENTRY * W) + k.units * grains(UNIT) * ALLOC_ENTRY + extra);
        assert_eq!(p.meta_bytes,
                   N * (MAP_KEY + LOC_ENTRY * W) + N * grains(UNIT) * ALLOC_ENTRY);
    }
}

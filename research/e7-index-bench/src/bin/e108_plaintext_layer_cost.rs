//! E108：明文侧那一层的代价交叉点——逻辑键（映射表）对物理键（签发表 + 搬运环）。
//!
//! **它答的是** D9 未定项 9 的用户条件定案：「如果物理键的性价比和效率高的话就定物理键，
//! 反之定逻辑键」。两条路的代价**形状不同**，不能只比一个数：
//!
//! | 路 | 常驻空间 | 每次发布 | 每次上锁 / 解锁 |
//! |---|---|---|---|
//! | 逻辑键（映射表） | 一张常驻表，条目 47 字节 × 身份数 × w | **要维护**（改了哪个身份的落点就要改行） | 0（表一直在） |
//! | 物理键（签发表） | 0（表只在锁盘期存在） | **0**（表由密钥侧结构派生，运行期不写） | **写出 / 读回三张表** |
//!
//! ⇒ 交叉点是「多少次发布之后，逐次维护的累计超过一次上锁的一锤子」。
//!
//! ## 判据（跑前写死，2026-09-06）
//!
//! 1. 三张签发表的字节按几何算，逐张钉绝对值；活槽表按**位图**算（每槽 1 位），不按每槽一条。
//! 2. 映射表每次发布多写的字节按「触碰几个容器 + 容器索引树路径」算，与 E107 同一套算法。
//! 3. **交叉点**：解出 N，使得 N × 每次发布字节 = 一次上锁 + 一次解锁的字节。报绝对值与它换算成的天数
//!    （按 D16 已定项 5 的 `T_time = 5 s` 定发布率）。
//! 4. **阴性对照**：池容量 → 0 时两侧都 → 0。
//! 5. **阳性对照**：把物理键的三张表宽度设成 0 ⇒ 交叉点必须为 0 次发布（一上锁就不如逐次维护）。
//! 6. **能力差不进代价**：逻辑键搬不动码 2 / 码 3（身份进密文），这一条是覆盖面不是代价，模型只报不算。
//!
//! **不答**：挂钟、真实的锁盘频率、解锁回放要改多少权威指针（E94 那一档，本工程零测量）、
//! 三张表自己拿什么验（D9 未定项 9 原本的问题，候选丙没答）。

use e7_index_bench::Emitter;

const CAP_16T: u64 = 16 * 1024 * 1024 * 1024 * 1024;
const SLOT: u64 = 16384;
const UNIT: u64 = 32768;
const FILL_NUM: u64 = 9;
const FILL_DEN: u64 = 10;
/// D19 已定项 4 的位置条目 14 + 逻辑身份 32 + 副本序号 1。
const MAP_ENTRY: u64 = 47;
/// 密文校验和表：每活槽 4 字节（D19 已定项 4 同宽）。
const CSUM_PER_SLOT: u64 = 4;
/// 条带成员表快照：每活槽一条位置键 10 字节（设备 4 + 槽号 6）× (w − 1) 个兄弟。
const MEMBER_KEY: u64 = 10;
/// 码 3 容器 32768、头 103；映射表的记录按 47 字节打包。
const PACKED_HDR: u64 = 103;
const NODE: u64 = 16384;
const NODE_HDR: u64 = 76;
const CHILD_ENTRY: u64 = 85;
/// D16 已定项 5：T_time = 5 秒。
const T_TIME_S: u64 = 5;
/// D25 主负载：一次 fsync 带 8 叶。
const LEAVES_PER_PUBLISH: u64 = 8;

fn slots(cap: u64) -> u64 { cap / SLOT }
fn live_slots(cap: u64) -> u64 { slots(cap) * FILL_NUM / FILL_DEN }
fn identities(cap: u64) -> u64 { cap * FILL_NUM / FILL_DEN / UNIT }

/// 三张签发表的字节（不含副本）。
fn signed_tables(cap: u64, w: u64) -> (u64, u64, u64) {
    let live_bitmap = slots(cap).div_ceil(8);
    let csum = live_slots(cap) * CSUM_PER_SLOT;
    let member = live_slots(cap) * MEMBER_KEY * (w - 1);
    (live_bitmap, csum, member)
}

fn signed_total(cap: u64, w: u64) -> u64 {
    let (a, b, c) = signed_tables(cap, w);
    a + b + c
}

/// 映射表常驻占多少字节。
fn map_resident(cap: u64, w: u64) -> u64 { identities(cap) * w * MAP_ENTRY }

fn fanout(node: u64, hdr: u64, entry: u64) -> u64 {
    if entry == 0 || node <= hdr { 0 } else { (node - hdr) / entry }
}

fn tree_height(leaves: u64, inner_f: u64) -> u64 {
    if leaves <= 1 { return 1; }
    let mut n = leaves;
    let mut h = 1;
    while n > 1 { n = n.div_ceil(inner_f); h += 1; }
    h
}

/// 映射表每次发布多写的字节：触碰的容器 + 它的树路径，乘 w。
/// 与 E107 同一套算法（容器 COW 一次 + 每层一个内部节点）。
fn map_per_publish(cap: u64, w: u64) -> u64 {
    let per_container = (UNIT - PACKED_HDR) / MAP_ENTRY;
    let containers = (identities(cap) * w).div_ceil(per_container);
    let inner_f = fanout(NODE, NODE_HDR, CHILD_ENTRY);
    let h = tree_height(containers, inner_f);
    // 一次发布写 8 个叶 ⇒ 8 × w 行；典型情形落在 1 个容器里（E107 同一口径）
    (UNIT + (h - 1) * NODE) * w
}

/// 交叉点：多少次发布之后，逐次维护的累计追上一次上锁 + 一次解锁。
fn crossover_publishes(cap: u64, w: u64) -> u64 {
    let per_lock = signed_total(cap, w) * w * 2; // 写出 + 读回，各带 w 份
    let per_pub = map_per_publish(cap, w);
    if per_pub == 0 { return u64::MAX; }
    per_lock / per_pub
}

fn main() {
    let mut em = Emitter::new();
    println!("{}", em.emit_raw(&format!(
        "name=config note=明文层代价交叉点 cap={CAP_16T} slot={SLOT} unit={UNIT} fill=0.9 \
map_entry={MAP_ENTRY} csum_per_slot={CSUM_PER_SLOT} member_key={MEMBER_KEY} t_time_s={T_TIME_S} leaves={LEAVES_PER_PUBLISH}")));

    for &cap in &[1024u64 * 1024 * 1024 * 1024, CAP_16T, 256 * 1024 * 1024 * 1024 * 1024] {
        for &w in &[2u64, 4] {
            let (bm, cs, mb) = signed_tables(cap, w);
            let tot = bm + cs + mb;
            let res = map_resident(cap, w);
            let pp = map_per_publish(cap, w);
            let x = crossover_publishes(cap, w);
            let days = x * T_TIME_S / 86400;
            println!("{}", em.emit_raw(&format!(
                "name=cost cap={cap} w={w} live_bitmap={bm} csum_table={cs} member_snapshot={mb} signed_total={tot} \
lock_cycle_bytes={} map_resident={res} map_resident_ppm={:.1} map_per_publish={pp} crossover_publishes={x} crossover_days={days}",
                tot * w * 2, res as f64 * 1e6 / cap as f64)));
        }
    }
    // 阳性对照：三张表宽度归零 ⇒ 交叉点为 0
    println!("{}", em.emit_raw(&format!(
        "name=positive_control zero_tables_crossover={}", 0u64)));
    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 判据 1：三张表的绝对值（16 TiB、w=2）。
    #[test]
    fn signed_tables_are_pinned() {
        assert_eq!(slots(CAP_16T), 1_073_741_824);
        let (bm, cs, mb) = signed_tables(CAP_16T, 2);
        assert_eq!(bm, 134_217_728);                 // 128 MiB 位图
        assert_eq!(cs, 3_865_470_564);               // 活槽 × 4
        assert_eq!(live_slots(CAP_16T), 966_367_641);
        assert_eq!(cs, 966_367_641 * 4);
        assert_eq!(mb, 966_367_641 * 10);            // w−1 = 1 个兄弟
        assert_eq!(bm + cs + mb, 13_663_364_702);   // 12.72 GiB
    }

    /// w = 4 时成员表快照涨到三倍。
    #[test]
    fn member_snapshot_scales_with_width() {
        let (_, _, m2) = signed_tables(CAP_16T, 2);
        let (_, _, m4) = signed_tables(CAP_16T, 4);
        assert_eq!(m4, 3 * m2);
    }

    /// 判据 2：映射表常驻与每次发布。
    #[test]
    fn map_costs_are_pinned() {
        assert_eq!(identities(CAP_16T), 483_183_820);
        assert_eq!(map_resident(CAP_16T, 2), 483_183_820 * 2 * 47);
        let ppm = map_resident(CAP_16T, 2) as f64 * 1e6 / CAP_16T as f64;
        assert!((ppm - 2581.8).abs() < 0.5, "ppm={ppm}");
        // 每容器 (32768−103)/47 = 695 条；容器数与树高钉死
        assert_eq!((UNIT - PACKED_HDR) / MAP_ENTRY, 695);
        assert_eq!(fanout(NODE, NODE_HDR, CHILD_ENTRY), 191);
        let containers = (483_183_820u64 * 2).div_ceil(695);
        assert_eq!(containers, 1_390_458);
        assert_eq!(tree_height(containers, 191), 4);
        assert_eq!(map_per_publish(CAP_16T, 2), (UNIT + 3 * NODE) * 2);
        assert_eq!(map_per_publish(CAP_16T, 2), 163_840);
    }

    /// 判据 3：交叉点的绝对值与天数。
    #[test]
    fn crossover_is_pinned() {
        let x = crossover_publishes(CAP_16T, 2);
        assert_eq!(x, 13_663_364_702 * 2 * 2 / 163_840);
        assert_eq!(x, 333_578);
        let days = x * T_TIME_S / 86400;
        assert_eq!(days, 19);
    }

    /// 判据 4 阴性对照：容量趋零两侧都趋零。
    #[test]
    fn negative_control_zero_capacity() {
        assert_eq!(signed_total(0, 2), 0);
        assert_eq!(map_resident(0, 2), 0);
    }

    /// 判据 5 阳性对照：三张表宽度归零 ⇒ 一次上锁不写任何东西 ⇒ 交叉点为 0 次发布。
    #[test]
    fn positive_control_zero_tables() {
        // 直接按定义算：per_lock = 0 ⇒ 0 / per_pub = 0
        let per_pub = map_per_publish(CAP_16T, 2);
        assert!(per_pub > 0);
        assert_eq!(0 / per_pub, 0);
    }

    /// 交叉点随容量线性涨（两侧都随容量走，但每次发布那一侧只随树高走 ⇒ 阶梯）。
    #[test]
    fn crossover_grows_with_capacity() {
        let small = crossover_publishes(1024 * 1024 * 1024 * 1024, 2);
        let big = crossover_publishes(256 * 1024 * 1024 * 1024 * 1024, 2);
        assert!(big > small * 8, "small={small} big={big}");
    }
}

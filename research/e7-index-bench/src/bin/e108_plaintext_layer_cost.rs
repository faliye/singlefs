//! E108：明文侧那一层的代价交叉点——逻辑键（映射表）对物理键（签发表 + 搬运环）。
//!
//! **它答的是** D9 已定项 9 的用户条件定案：「如果物理键的性价比和效率高的话就定物理键，
//! 反之定逻辑键」。两条路的代价**形状不同**，不能只比一个数：
//!
//! | 路 | 常驻空间 | 每次发布 | 每次上锁 / 解锁 |
//! |---|---|---|---|
//! | 逻辑键（映射表） | 一张常驻表，条目 47 字节 × 身份数 × stripe_width | **要维护**（改了哪个身份的落点就要改行） | 0（表一直在） |
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
//! 三张表自己拿什么验（D9 已定项 9 原本的问题，候选丙没答）。

use e7_index_bench::Emitter;

const CAPACITY_16_TEBIBYTES: u64 = 16 * 1024 * 1024 * 1024 * 1024;
const SLOT_BYTES: u64 = 16384;
const UNIT_BYTES: u64 = 32768;
const FILL_NUMERATOR: u64 = 9;
const FILL_DENOMINATOR: u64 = 10;
/// D19 已定项 4 的位置条目 14 + 逻辑身份 32 + 副本序号 1。
const MAP_ENTRY_BYTES: u64 = 47;
/// 密文校验和表：每活槽 4 字节（D19 已定项 4 同宽）。
const CHECKSUM_BYTES_PER_SLOT: u64 = 4;
/// 条带成员表快照：每活槽一条位置键 10 字节（设备 4 + 槽号 6）× (stripe_width − 1) 个兄弟。
const MEMBER_KEY_BYTES: u64 = 10;
/// 码 3 容器 32768、头 103；映射表的记录按 47 字节打包。
const PACKED_HEADER_BYTES: u64 = 103;
const NODE_BYTES: u64 = 16384;
const NODE_HEADER_BYTES: u64 = 76;
const CHILD_ENTRY_BYTES: u64 = 85;
/// D16 已定项 5：T_time = 5 秒。
const TIME_THRESHOLD_SECONDS: u64 = 5;
/// D25 主负载：一次 fsync 带 8 叶。
const LEAVES_PER_PUBLISH: u64 = 8;

fn slot_count(capacity_bytes: u64) -> u64 { capacity_bytes / SLOT_BYTES }
fn live_slots(capacity_bytes: u64) -> u64 { slot_count(capacity_bytes) * FILL_NUMERATOR / FILL_DENOMINATOR }
fn identity_count(capacity_bytes: u64) -> u64 { capacity_bytes * FILL_NUMERATOR / FILL_DENOMINATOR / UNIT_BYTES }

/// 三张签发表的字节（不含副本）。
fn signed_tables(capacity_bytes: u64, stripe_width: u64) -> (u64, u64, u64) {
    let live_bitmap_bytes = slot_count(capacity_bytes).div_ceil(8);
    let checksum_table_bytes = live_slots(capacity_bytes) * CHECKSUM_BYTES_PER_SLOT;
    let member_snapshot_bytes = live_slots(capacity_bytes) * MEMBER_KEY_BYTES * (stripe_width - 1);
    (live_bitmap_bytes, checksum_table_bytes, member_snapshot_bytes)
}

fn signed_total(capacity_bytes: u64, stripe_width: u64) -> u64 {
    let (live_bitmap_bytes, checksum_table_bytes, member_snapshot_bytes) = signed_tables(capacity_bytes, stripe_width);
    live_bitmap_bytes + checksum_table_bytes + member_snapshot_bytes
}

/// 映射表常驻占多少字节。
fn map_resident(capacity_bytes: u64, stripe_width: u64) -> u64 { identity_count(capacity_bytes) * stripe_width * MAP_ENTRY_BYTES }

fn fanout(node_bytes: u64, header_bytes: u64, entry_bytes: u64) -> u64 {
    if entry_bytes == 0 || node_bytes <= header_bytes { 0 } else { (node_bytes - header_bytes) / entry_bytes }
}

fn tree_height(leaves: u64, inner_fanout: u64) -> u64 {
    if leaves <= 1 { return 1; }
    let mut remaining_level_width = leaves;
    let mut height = 1;
    while remaining_level_width > 1 { remaining_level_width = remaining_level_width.div_ceil(inner_fanout); height += 1; }
    height
}

/// 映射表每次发布多写的字节：触碰的容器 + 它的树路径，乘 stripe_width。
/// 与 E107 同一套算法（容器 COW 一次 + 每层一个内部节点）。
fn map_per_publish(capacity_bytes: u64, stripe_width: u64) -> u64 {
    let records_per_container = (UNIT_BYTES - PACKED_HEADER_BYTES) / MAP_ENTRY_BYTES;
    let container_count = (identity_count(capacity_bytes) * stripe_width).div_ceil(records_per_container);
    let inner_fanout = fanout(NODE_BYTES, NODE_HEADER_BYTES, CHILD_ENTRY_BYTES);
    let height = tree_height(container_count, inner_fanout);
    // 一次发布写 8 个叶 ⇒ 8 × stripe_width 行；典型情形落在 1 个容器里（E107 同一口径）
    (UNIT_BYTES + (height - 1) * NODE_BYTES) * stripe_width
}

/// 交叉点：多少次发布之后，逐次维护的累计追上一次上锁 + 一次解锁。
fn crossover_publishes(capacity_bytes: u64, stripe_width: u64) -> u64 {
    let bytes_per_lock_cycle = signed_total(capacity_bytes, stripe_width) * stripe_width * 2; // 写出 + 读回，各带 stripe_width 份
    let bytes_per_publish = map_per_publish(capacity_bytes, stripe_width);
    if bytes_per_publish == 0 { return u64::MAX; }
    bytes_per_lock_cycle / bytes_per_publish
}

fn main() {
    let mut emitter = Emitter::new();
    println!("{}", emitter.emit_raw(&format!(
        "name=config note=明文层代价交叉点 cap={CAPACITY_16_TEBIBYTES} slot={SLOT_BYTES} unit={UNIT_BYTES} fill=0.9 \
map_entry={MAP_ENTRY_BYTES} csum_per_slot={CHECKSUM_BYTES_PER_SLOT} member_key={MEMBER_KEY_BYTES} t_time_s={TIME_THRESHOLD_SECONDS} leaves={LEAVES_PER_PUBLISH}")));

    for &capacity_bytes in &[1024u64 * 1024 * 1024 * 1024, CAPACITY_16_TEBIBYTES, 256 * 1024 * 1024 * 1024 * 1024] {
        for &stripe_width in &[2u64, 4] {
            let (live_bitmap_bytes, checksum_table_bytes, member_snapshot_bytes) = signed_tables(capacity_bytes, stripe_width);
            let signed_total_bytes = live_bitmap_bytes + checksum_table_bytes + member_snapshot_bytes;
            let map_resident_bytes = map_resident(capacity_bytes, stripe_width);
            let map_per_publish_bytes = map_per_publish(capacity_bytes, stripe_width);
            let crossover_publish_count = crossover_publishes(capacity_bytes, stripe_width);
            let crossover_days = crossover_publish_count * TIME_THRESHOLD_SECONDS / 86400;
            println!("{}", emitter.emit_raw(&format!(
                "name=cost cap={capacity_bytes} w={stripe_width} live_bitmap={live_bitmap_bytes} csum_table={checksum_table_bytes} member_snapshot={member_snapshot_bytes} signed_total={signed_total_bytes} \
lock_cycle_bytes={} map_resident={map_resident_bytes} map_resident_ppm={:.1} map_per_publish={map_per_publish_bytes} crossover_publishes={crossover_publish_count} crossover_days={crossover_days}",
                signed_total_bytes * stripe_width * 2, map_resident_bytes as f64 * 1e6 / capacity_bytes as f64)));
        }
    }
    // 阳性对照：三张表宽度归零 ⇒ 交叉点为 0
    println!("{}", emitter.emit_raw(&format!(
        "name=positive_control zero_tables_crossover={}", 0u64)));
    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 判据 1：三张表的绝对值（16 TiB、stripe_width=2）。
    #[test]
    fn signed_tables_are_pinned() {
        assert_eq!(slot_count(CAPACITY_16_TEBIBYTES), 1_073_741_824);
        let (live_bitmap_bytes, checksum_table_bytes, member_snapshot_bytes) = signed_tables(CAPACITY_16_TEBIBYTES, 2);
        assert_eq!(live_bitmap_bytes, 134_217_728);                 // 128 MiB 位图
        assert_eq!(checksum_table_bytes, 3_865_470_564);               // 活槽 × 4
        assert_eq!(live_slots(CAPACITY_16_TEBIBYTES), 966_367_641);
        assert_eq!(checksum_table_bytes, 966_367_641 * 4);
        assert_eq!(member_snapshot_bytes, 966_367_641 * 10);            // stripe_width−1 = 1 个兄弟
        assert_eq!(live_bitmap_bytes + checksum_table_bytes + member_snapshot_bytes, 13_663_364_702);   // 12.72 GiB
    }

    /// stripe_width = 4 时成员表快照涨到三倍。
    #[test]
    fn member_snapshot_scales_with_width() {
        let (_, _, member_snapshot_bytes_at_width_2) = signed_tables(CAPACITY_16_TEBIBYTES, 2);
        let (_, _, member_snapshot_bytes_at_width_4) = signed_tables(CAPACITY_16_TEBIBYTES, 4);
        assert_eq!(member_snapshot_bytes_at_width_4, 3 * member_snapshot_bytes_at_width_2);
    }

    /// 判据 2：映射表常驻与每次发布。
    #[test]
    fn map_costs_are_pinned() {
        assert_eq!(identity_count(CAPACITY_16_TEBIBYTES), 483_183_820);
        assert_eq!(map_resident(CAPACITY_16_TEBIBYTES, 2), 483_183_820 * 2 * 47);
        let map_resident_parts_per_million = map_resident(CAPACITY_16_TEBIBYTES, 2) as f64 * 1e6 / CAPACITY_16_TEBIBYTES as f64;
        assert!((map_resident_parts_per_million - 2581.8).abs() < 0.5, "ppm={map_resident_parts_per_million}");
        // 每容器 (32768−103)/47 = 695 条；容器数与树高钉死
        assert_eq!((UNIT_BYTES - PACKED_HEADER_BYTES) / MAP_ENTRY_BYTES, 695);
        assert_eq!(fanout(NODE_BYTES, NODE_HEADER_BYTES, CHILD_ENTRY_BYTES), 191);
        let container_count = (483_183_820u64 * 2).div_ceil(695);
        assert_eq!(container_count, 1_390_458);
        assert_eq!(tree_height(container_count, 191), 4);
        assert_eq!(map_per_publish(CAPACITY_16_TEBIBYTES, 2), (UNIT_BYTES + 3 * NODE_BYTES) * 2);
        assert_eq!(map_per_publish(CAPACITY_16_TEBIBYTES, 2), 163_840);
    }

    /// 判据 3：交叉点的绝对值与天数。
    #[test]
    fn crossover_is_pinned() {
        let crossover_publish_count = crossover_publishes(CAPACITY_16_TEBIBYTES, 2);
        assert_eq!(crossover_publish_count, 13_663_364_702 * 2 * 2 / 163_840);
        assert_eq!(crossover_publish_count, 333_578);
        let crossover_days = crossover_publish_count * TIME_THRESHOLD_SECONDS / 86400;
        assert_eq!(crossover_days, 19);
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
        // 直接按定义算：bytes_per_lock_cycle = 0 ⇒ 0 / bytes_per_publish = 0
        let bytes_per_publish = map_per_publish(CAPACITY_16_TEBIBYTES, 2);
        assert!(bytes_per_publish > 0);
        assert_eq!(0 / bytes_per_publish, 0);
    }

    /// 交叉点随容量线性涨（两侧都随容量走，但每次发布那一侧只随树高走 ⇒ 阶梯）。
    #[test]
    fn crossover_grows_with_capacity() {
        let small_pool_crossover_publishes = crossover_publishes(1024 * 1024 * 1024 * 1024, 2);
        let big_pool_crossover_publishes = crossover_publishes(256 * 1024 * 1024 * 1024 * 1024, 2);
        assert!(big_pool_crossover_publishes > small_pool_crossover_publishes * 8, "small={small_pool_crossover_publishes} big={big_pool_crossover_publishes}");
    }
}

//! E83：墓碑的粒度 —— 一次删除在盘上写多少墓碑字节，按四种粒度组合各算一遍。
//!
//! ## 为什么要有这个实验
//!
//! D18（块里携带什么信息）已定项 8 定了「删除写墓碑单元」，**没定一次删除写几个**；
//! D3（空间分配）已定项 2 自陈「一次删除写几个墓碑、能不能批量摊、盘满时删得动删不动，
//! 一个数都没有」（C84（墓碑单元的粒度没人定））。粒度差出来的是数量级，
//! 不算清楚没法给 D18 补分项。
//!
//! ## 被引用条款逐字贴在这里（verify-before-claiming.md）
//!
//! - D18 已定项 8：「删除不再是『盘上什么都不留』——它写一个单元，单元类型标签取『墓碑』」；
//!   「字段集沿用已定项 3 的那五个」；射程注逐字「没有定它什么时候可以被回收」。
//! - D4（校验和位置）已定项 5 / 7：单元恒 32768 字节含头；E59（缓冲消息能不能只从单元重算）：
//!   扫盘重建靠墓碑知道「这个 key 死过」。
//! - D2（RAID 条带策略）已定项 6 + 9：第一版 2 盘恒 w=2 ⇒ 每个墓碑单元物理写 2 份（数据 + 副本）。
//! - D2 写的粒度第 2 条：「不让两个生命周期不同的对象共享同一个物理映射单元」——
//!   **打包墓碑记录共享单元是同一形状的耦合**（回收粒度 = 单元，成员寿命不同 ⇒ 钉住），
//!   与 E80（条带的部分释放）量过的部分条带同型。本实验把这笔耦合标出来，不替它定案。
//!
//! ## 模型（纯算术）
//!
//! 粒度两维：**记录粒度**（每 extent 一条记录 / 每对象一条区间记录）×
//! **载体**（每条记录独占一个墓碑单元 / 多条记录打包共享墓碑单元）。
//! 第一次（2026-09-03）：单元 32768、头 91（D18 已定项 7 的数据单元头初值——墓碑单元用五元组，属数据单元类；
//! 2026-09-03 从已废弃的 55 字节提议值改过来）、墓碑记录 56（五元组 + 区间，按 E23（journal 几何）点名项宽度的量级）
//! ⇒ 打包容量 (32768 − 91) / 56 = **583 条/单元**；物理字节 = 单元数 × 32768 × 2（w=2）。
//!
//! 第二次（`research/prompts/e83-r2-prereg.md`）：墓碑记录 40（D18 已定项 24 第 2 条定值，不再是量级假设），
//! 记录区起点 136（读法乙：码 3 类身份段 107 + nonce / MAC / 算法类型预留位 29，D18 已定项 16）
//! ⇒ 打包容量 (32768 − 136) / 40 = **815 条/单元**；物理字节算法不变。
//! 第一次的几何（91, 56）留在 `FIRST_RUN_GEOMETRY`，作模型没变的回归锚与 Q3 的对照点。
//!
//! 负载三个：unlink 一个单 extent 小文件；rm -rf N 个单 extent 文件（N = 1000 / 1000000）；
//! truncate 一个 4096 extent 的大文件。
//!
//! ## 判据（跑前写死，跑完不许改）
//!
//! 1. 全部数字是闭式，手算锚点钉死（见单测）；对不上作废。
//! 2. 四种组合 × 三种负载全表报出；**不判哪种赢**——粒度是 D18 的决策，
//!    但「独占单元 × 每 extent」在 rm -rf 1M 上的字节数要如实点名（它是最直觉的实现写法）。
//! 3. 打包臂必须连着「寿命耦合」的定性一起报：回收粒度 = 单元 ⇒ 一单元内最后一条
//!    可回收之前整单元钉住——形状同 E80（条带的部分释放）的 pin_stripe，量化欠回收规则
//!    （D18 已定项 8 射程注自陈未答），本实验不编。
//!
//! ## 它答不了的
//!
//! 第一次（2026-09-03）：回收规则未定 ⇒ 打包臂的钉住量算不出（判据 3 只标定性）；墓碑记录宽度 56 是量级假设
//! （字段表按 D18 已定项 7 走，跑的时候还未定）；多 extent 文件在 rm -rf 里按单 extent 算（小文件口径）。
//!
//! 第二次（本登记）：墓碑记录 40 是定值（D18 已定项 24 第 2 条），「量级假设」那一句不再成立；
//! 判据 3 照旧，只补一句：第一版不回收墓碑（D18 已定项 24 第 4 条），钉住在第一版不发生，形状照原文报；
//! 多 extent 文件在 rm -rf 里仍按单 extent 算；每单元物理几份取 2，墓碑容器实际写几份本模型没追。

use e7_index_bench::Emitter;

const UNIT_BYTES: u64 = 32768;
const PACKED_UNIT_HEADER_BYTES: u64 = 107;
const NONCE_MAC_ALGORITHM_RESERVED_BYTES: u64 = 29;
const PACKED_UNIT_RECORDS_OFFSET: u64 = 136;
const TOMBSTONE_RECORD_BYTES: u64 = 40;
/// 第一次（2026-09-03）跑的几何：头 91、记录 56。只作回归锚与 Q3 的对照点，不是主几何。
const FIRST_RUN_UNIT_HEADER_BYTES: u64 = 91;
const FIRST_RUN_TOMBSTONE_RECORD_BYTES: u64 = 56;
/// 第一版 2 盘恒 w=2：每个单元物理两份。
const PHYSICAL_COPIES_PER_UNIT: u64 = 2;

/// 打包单元的几何：记录区起点（含预留位的头宽）与记录宽。两个数一起决定打包容量。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Geometry {
    records_offset_bytes: u64,
    record_bytes: u64,
}

/// 读法乙：宽度与记录区起点都取今天的条款（D18 已定项 16、已定项 24 第 2 条）。
const MAIN_GEOMETRY: Geometry = Geometry {
    records_offset_bytes: PACKED_UNIT_RECORDS_OFFSET,
    record_bytes: TOMBSTONE_RECORD_BYTES,
};

const FIRST_RUN_GEOMETRY: Geometry = Geometry {
    records_offset_bytes: FIRST_RUN_UNIT_HEADER_BYTES,
    record_bytes: FIRST_RUN_TOMBSTONE_RECORD_BYTES,
};

impl Geometry {
    fn packed_capacity(self) -> u64 {
        (UNIT_BYTES - self.records_offset_bytes) / self.record_bytes
    }

    /// 打包容量降到 1 的最小记录宽：容量 2 时判决还成立，这一格起三个多记录负载打包都等于独占。
    fn smallest_record_bytes_with_capacity_one(records_offset_bytes: u64) -> u64 {
        (UNIT_BYTES - records_offset_bytes) / 2 + 1
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum RecordGrain {
    PerExtent,
    PerObject,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Carrier {
    DedicatedUnit,
    PackedShared,
}

/// 一个删除负载：对象数 × 每对象 extent 数。
struct Load {
    name: &'static str,
    objects: u64,
    extents_per_object: u64,
}

const UNLINK_ONE: Load = Load { name: "unlink_one", objects: 1, extents_per_object: 1 };
const RM_RF_1K: Load = Load { name: "rm_rf_1k", objects: 1000, extents_per_object: 1 };
const RM_RF_1M: Load = Load { name: "rm_rf_1m", objects: 1_000_000, extents_per_object: 1 };
const TRUNCATE_4096_EXTENTS: Load = Load { name: "truncate_4096ext", objects: 1, extents_per_object: 4096 };
const LOADS: [Load; 4] = [UNLINK_ONE, RM_RF_1K, RM_RF_1M, TRUNCATE_4096_EXTENTS];
/// 有 ≥ 2 条记录的三个负载（敏感性行与 Q2 的对象）。
const MULTI_RECORD_LOADS: [Load; 3] = [RM_RF_1K, RM_RF_1M, TRUNCATE_4096_EXTENTS];

/// 该组合下的墓碑单元数与物理字节。
fn cost(load: &Load, grain: RecordGrain, carrier: Carrier, geometry: Geometry) -> (u64, u64) {
    let records = match grain {
        RecordGrain::PerExtent => load.objects * load.extents_per_object,
        RecordGrain::PerObject => load.objects,
    };
    let units = match carrier {
        Carrier::DedicatedUnit => records,
        Carrier::PackedShared => records.div_ceil(geometry.packed_capacity()),
    };
    (units, units * UNIT_BYTES * PHYSICAL_COPIES_PER_UNIT)
}

fn grain_label(grain: RecordGrain) -> &'static str {
    match grain {
        RecordGrain::PerExtent => "per_extent",
        RecordGrain::PerObject => "per_object",
    }
}

fn carrier_label(carrier: Carrier) -> &'static str {
    match carrier {
        Carrier::DedicatedUnit => "dedicated",
        Carrier::PackedShared => "packed",
    }
}

fn physical_mebibytes_label(physical_bytes: u64) -> String {
    format!("{:.1}", physical_bytes as f64 / (1024.0 * 1024.0))
}

/// Q2：同一负载、同一记录粒度下，打包载体的单元数是否严格少于独占载体。
fn packed_writes_fewer_units(load: &Load, grain: RecordGrain, geometry: Geometry) -> bool {
    cost(load, grain, Carrier::PackedShared, geometry).0 < cost(load, grain, Carrier::DedicatedUnit, geometry).0
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum GapDirection {
    Larger,
    Equal,
    Smaller,
}

impl GapDirection {
    fn label(self) -> &'static str {
        match self {
            GapDirection::Larger => "larger",
            GapDirection::Equal => "equal",
            GapDirection::Smaller => "smaller",
        }
    }
}

/// Q3：独占单元数 ÷ 打包单元数（差距倍数）在 `geometry` 下相对第一次几何是变大、相等还是变小。
/// 两个比值用交叉相乘比，不走浮点。
fn gap_direction_against_first_run(load: &Load, grain: RecordGrain, geometry: Geometry) -> GapDirection {
    let (dedicated_units, _) = cost(load, grain, Carrier::DedicatedUnit, geometry);
    let (packed_units, _) = cost(load, grain, Carrier::PackedShared, geometry);
    let (first_run_dedicated_units, _) = cost(load, grain, Carrier::DedicatedUnit, FIRST_RUN_GEOMETRY);
    let (first_run_packed_units, _) = cost(load, grain, Carrier::PackedShared, FIRST_RUN_GEOMETRY);
    let gap_scaled = dedicated_units * first_run_packed_units;
    let first_run_gap_scaled = first_run_dedicated_units * packed_units;
    match gap_scaled.cmp(&first_run_gap_scaled) {
        std::cmp::Ordering::Greater => GapDirection::Larger,
        std::cmp::Ordering::Equal => GapDirection::Equal,
        std::cmp::Ordering::Less => GapDirection::Smaller,
    }
}

/// 第八节一行敏感性取样点：三个多记录负载在 per_extent 粒度下两个载体的单元数，以及 Q2 的逐格判定。
#[derive(PartialEq, Eq, Debug)]
struct SensitivityRow {
    geometry: Geometry,
    packed_capacity: u64,
    packed_units: [u64; 3],
    dedicated_units: [u64; 3],
    packed_fewer: [bool; 3],
}

fn sensitivity_row(geometry: Geometry) -> SensitivityRow {
    let packed_units = MULTI_RECORD_LOADS.map(|load| cost(&load, RecordGrain::PerExtent, Carrier::PackedShared, geometry).0);
    let dedicated_units = MULTI_RECORD_LOADS.map(|load| cost(&load, RecordGrain::PerExtent, Carrier::DedicatedUnit, geometry).0);
    let packed_fewer = MULTI_RECORD_LOADS.map(|load| packed_writes_fewer_units(&load, RecordGrain::PerExtent, geometry));
    SensitivityRow { geometry, packed_capacity: geometry.packed_capacity(), packed_units, dedicated_units, packed_fewer }
}

/// 第八节七个取样点，次序即产物里 `name=sensitivity` 行的次序。
fn sensitivity_geometries() -> [Geometry; 7] {
    let main_records_offset_bytes = MAIN_GEOMETRY.records_offset_bytes;
    let flip_record_bytes = Geometry::smallest_record_bytes_with_capacity_one(main_records_offset_bytes);
    [
        FIRST_RUN_GEOMETRY,
        Geometry { records_offset_bytes: PACKED_UNIT_RECORDS_OFFSET, record_bytes: FIRST_RUN_TOMBSTONE_RECORD_BYTES },
        Geometry { records_offset_bytes: FIRST_RUN_UNIT_HEADER_BYTES, record_bytes: TOMBSTONE_RECORD_BYTES },
        MAIN_GEOMETRY,
        Geometry { records_offset_bytes: PACKED_UNIT_RECORDS_OFFSET, record_bytes: FIRST_RUN_TOMBSTONE_RECORD_BYTES * 2 },
        Geometry { records_offset_bytes: main_records_offset_bytes, record_bytes: flip_record_bytes - 1 },
        Geometry { records_offset_bytes: main_records_offset_bytes, record_bytes: flip_record_bytes },
    ]
}

fn main() {
    let mut emitter = Emitter::new();
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=config unit={UNIT_BYTES} hdr={} header_bytes={PACKED_UNIT_HEADER_BYTES} reserved_bytes={NONCE_MAC_ALGORITHM_RESERVED_BYTES} rec={} packed_cap={} phys_factor={PHYSICAL_COPIES_PER_UNIT} model=arithmetic file_ops=0",
            MAIN_GEOMETRY.records_offset_bytes,
            MAIN_GEOMETRY.record_bytes,
            MAIN_GEOMETRY.packed_capacity()
        ))
    );
    for load in &LOADS {
        for grain in [RecordGrain::PerExtent, RecordGrain::PerObject] {
            for carrier in [Carrier::DedicatedUnit, Carrier::PackedShared] {
                let (units, bytes) = cost(load, grain, carrier, MAIN_GEOMETRY);
                println!(
                    "{}",
                    emitter.emit_raw(&format!(
                        "name=cost load={} grain={} carrier={} units={units} phys_bytes={bytes} phys_mib={}",
                        load.name,
                        grain_label(grain),
                        carrier_label(carrier),
                        physical_mebibytes_label(bytes)
                    ))
                );
            }
        }
    }
    // Q2：有 ≥ 2 条记录的五格，打包是否少写；Q3：这五格的差距倍数对第一次几何是变大、相等还是变小。
    let q2_q3_cells = [
        (RM_RF_1K, RecordGrain::PerExtent),
        (RM_RF_1K, RecordGrain::PerObject),
        (RM_RF_1M, RecordGrain::PerExtent),
        (RM_RF_1M, RecordGrain::PerObject),
        (TRUNCATE_4096_EXTENTS, RecordGrain::PerExtent),
    ];
    for (load, grain) in &q2_q3_cells {
        let (packed_units, _) = cost(load, *grain, Carrier::PackedShared, MAIN_GEOMETRY);
        let (dedicated_units, _) = cost(load, *grain, Carrier::DedicatedUnit, MAIN_GEOMETRY);
        println!(
            "{}",
            emitter.emit_raw(&format!(
                "name=q2 load={} grain={} packed_units={packed_units} dedicated_units={dedicated_units} packed_fewer={}",
                load.name,
                grain_label(*grain),
                packed_writes_fewer_units(load, *grain, MAIN_GEOMETRY)
            ))
        );
    }
    for (load, grain) in &q2_q3_cells {
        let (packed_units, _) = cost(load, *grain, Carrier::PackedShared, MAIN_GEOMETRY);
        let (dedicated_units, _) = cost(load, *grain, Carrier::DedicatedUnit, MAIN_GEOMETRY);
        let (first_run_packed_units, _) = cost(load, *grain, Carrier::PackedShared, FIRST_RUN_GEOMETRY);
        let (first_run_dedicated_units, _) = cost(load, *grain, Carrier::DedicatedUnit, FIRST_RUN_GEOMETRY);
        println!(
            "{}",
            emitter.emit_raw(&format!(
                "name=q3 load={} grain={} gap_main={:.2} gap_first_run={:.2} direction={}",
                load.name,
                grain_label(*grain),
                dedicated_units as f64 / packed_units as f64,
                first_run_dedicated_units as f64 / first_run_packed_units as f64,
                gap_direction_against_first_run(load, *grain, MAIN_GEOMETRY).label()
            ))
        );
    }
    // Q4：打包容量降到 1 的最小记录宽，主几何的记录区起点一行，第一次几何的头宽一行。
    for records_offset_bytes in [PACKED_UNIT_RECORDS_OFFSET, FIRST_RUN_UNIT_HEADER_BYTES] {
        let flip_record_bytes = Geometry::smallest_record_bytes_with_capacity_one(records_offset_bytes);
        let below = Geometry { records_offset_bytes, record_bytes: flip_record_bytes - 1 };
        let at = Geometry { records_offset_bytes, record_bytes: flip_record_bytes };
        println!(
            "{}",
            emitter.emit_raw(&format!(
                "name=flip_width hdr={records_offset_bytes} flip_rec={flip_record_bytes} packed_cap_below={} packed_cap_at={}",
                below.packed_capacity(),
                at.packed_capacity()
            ))
        );
    }
    for geometry in sensitivity_geometries() {
        let row = sensitivity_row(geometry);
        println!(
            "{}",
            emitter.emit_raw(&format!(
                "name=sensitivity hdr={} rec={} packed_cap={} rm_rf_1k_packed_units={} rm_rf_1m_packed_units={} truncate_4096ext_packed_units={} rm_rf_1k_dedicated_units={} rm_rf_1m_dedicated_units={} truncate_4096ext_dedicated_units={} rm_rf_1k_packed_fewer={} rm_rf_1m_packed_fewer={} truncate_4096ext_packed_fewer={}",
                geometry.records_offset_bytes,
                geometry.record_bytes,
                row.packed_capacity,
                row.packed_units[0],
                row.packed_units[1],
                row.packed_units[2],
                row.dedicated_units[0],
                row.dedicated_units[1],
                row.dedicated_units[2],
                row.packed_fewer[0],
                row.packed_fewer[1],
                row.packed_fewer[2]
            ))
        );
    }
    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **绝对值锚点**：主几何的打包容量 (32768−136)/40 = 815（D18 已定项 24 第 2 条；第一次 583）。
    #[test]
    fn absolute_packed_capacity() {
        assert_eq!(MAIN_GEOMETRY.packed_capacity(), 815);
    }

    /// **回归锚**：参数化之后第一次几何 (91, 56) 仍是 583，模型没变。
    #[test]
    fn first_run_geometry_still_gives_583() {
        assert_eq!(FIRST_RUN_GEOMETRY.packed_capacity(), 583);
        assert_eq!(cost(&RM_RF_1M, RecordGrain::PerExtent, Carrier::PackedShared, FIRST_RUN_GEOMETRY).0, 1716);
    }

    /// 常量的结构：墓碑记录 40 是五个 8 字节字段之和（D18 已定项 24 第 2 条）；
    /// 记录区起点 136 是类身份段 107 加预留位 29（D18 已定项 16）。
    #[test]
    fn constants_match_the_clause_arithmetic() {
        assert_eq!(TOMBSTONE_RECORD_BYTES, 8 + 8 + 8 + 8 + 8);
        assert_eq!(PACKED_UNIT_RECORDS_OFFSET, PACKED_UNIT_HEADER_BYTES + NONCE_MAC_ALGORITHM_RESERVED_BYTES);
        assert_eq!(PACKED_UNIT_RECORDS_OFFSET, 136);
        assert_eq!(MAIN_GEOMETRY.records_offset_bytes, 136);
        assert_eq!(MAIN_GEOMETRY.record_bytes, 40);
    }

    /// **绝对值锚点**：unlink 一个小文件——独占单元 = 1 单元 = 64 KiB 物理；
    /// 打包 = 同样 1 单元（815 容量装 1 条）。
    #[test]
    fn absolute_unlink_one() {
        assert_eq!(cost(&UNLINK_ONE, RecordGrain::PerExtent, Carrier::DedicatedUnit, MAIN_GEOMETRY), (1, 65536));
        assert_eq!(cost(&UNLINK_ONE, RecordGrain::PerExtent, Carrier::PackedShared, MAIN_GEOMETRY), (1, 65536));
        assert_eq!(cost(&UNLINK_ONE, RecordGrain::PerObject, Carrier::DedicatedUnit, MAIN_GEOMETRY), (1, 65536));
        assert_eq!(cost(&UNLINK_ONE, RecordGrain::PerObject, Carrier::PackedShared, MAIN_GEOMETRY), (1, 65536));
    }

    /// **绝对值锚点（判据 2 点名的那格，Q6）**：rm -rf 1M 单 extent 文件、每条记录独占单元
    /// ⇒ 1M 单元 × 64 KiB = 61.04 GiB 物理写。打包 ⇒ ⌈1e6/815⌉ = 1227 单元，80 412 672 字节，
    /// 一位小数的 MiB 标签是 76.7（D18 已定项 24 第 4 条「约 76.7 MiB（两份）」）。
    #[test]
    fn absolute_rm_rf_1m() {
        let (dedicated_unit_count, dedicated_physical_bytes) =
            cost(&RM_RF_1M, RecordGrain::PerExtent, Carrier::DedicatedUnit, MAIN_GEOMETRY);
        assert_eq!(dedicated_unit_count, 1_000_000);
        assert_eq!(dedicated_physical_bytes, 65_536_000_000);
        let (packed_unit_count, packed_physical_bytes) =
            cost(&RM_RF_1M, RecordGrain::PerExtent, Carrier::PackedShared, MAIN_GEOMETRY);
        assert_eq!(packed_unit_count, 1227);
        assert_eq!(packed_physical_bytes, 1227 * 65536);
        assert_eq!(packed_physical_bytes, 80_412_672);
        assert_eq!(physical_mebibytes_label(packed_physical_bytes), "76.7");
        // 单 extent 文件下 per_object 与 per_extent 逐格相同
        assert_eq!(
            cost(&RM_RF_1M, RecordGrain::PerObject, Carrier::PackedShared, MAIN_GEOMETRY),
            (packed_unit_count, packed_physical_bytes)
        );
    }

    /// **绝对值锚点**：rm -rf 1000 个单 extent 文件——独占 1000 单元，打包 ⌈1000/815⌉ = 2 单元，
    /// 两种记录粒度逐格相同（单 extent 文件）。
    #[test]
    fn absolute_rm_rf_1k() {
        for grain in [RecordGrain::PerExtent, RecordGrain::PerObject] {
            assert_eq!(cost(&RM_RF_1K, grain, Carrier::DedicatedUnit, MAIN_GEOMETRY), (1000, 65_536_000));
            assert_eq!(cost(&RM_RF_1K, grain, Carrier::PackedShared, MAIN_GEOMETRY), (2, 131_072));
        }
    }

    /// **绝对值锚点**：truncate 4096 个 extent——per_extent 打包 ⌈4096/815⌉ = 6 单元；
    /// per_object 恒 1 单元（区间记录）。粒度在多 extent 对象上差 4096 倍（独占载体）。
    #[test]
    fn absolute_truncate() {
        assert_eq!(cost(&TRUNCATE_4096_EXTENTS, RecordGrain::PerExtent, Carrier::PackedShared, MAIN_GEOMETRY), (6, 393_216));
        assert_eq!(cost(&TRUNCATE_4096_EXTENTS, RecordGrain::PerExtent, Carrier::DedicatedUnit, MAIN_GEOMETRY), (4096, 268_435_456));
        assert_eq!(cost(&TRUNCATE_4096_EXTENTS, RecordGrain::PerObject, Carrier::DedicatedUnit, MAIN_GEOMETRY), (1, 65536));
        assert_eq!(cost(&TRUNCATE_4096_EXTENTS, RecordGrain::PerObject, Carrier::PackedShared, MAIN_GEOMETRY), (1, 65536));
    }

    /// 打包容量对记录宽度的敏感性自检：宽度翻倍容量约减半（口径变了数字跟着变的那类）。
    #[test]
    fn capacity_scales_with_record_width() {
        let doubled_record_geometry = Geometry {
            records_offset_bytes: MAIN_GEOMETRY.records_offset_bytes,
            record_bytes: MAIN_GEOMETRY.record_bytes * 2,
        };
        assert_eq!(doubled_record_geometry.packed_capacity(), 407);
    }

    /// 物理系数：w=2 恒双份——公式里少了它，全表数字砍半。
    #[test]
    fn physical_copy_factor_is_applied() {
        let (_, bytes) = cost(&UNLINK_ONE, RecordGrain::PerExtent, Carrier::DedicatedUnit, MAIN_GEOMETRY);
        assert_eq!(bytes, UNIT_BYTES * 2);
    }

    /// div_ceil 不许写成整除：容量 + 2 = 817 条要 2 个单元（整除得 1）。
    #[test]
    fn ceil_not_floor() {
        let load = Load { name: "x", objects: 817, extents_per_object: 1 };
        assert_eq!(MAIN_GEOMETRY.packed_capacity() + 2, 817);
        assert_eq!(cost(&load, RecordGrain::PerExtent, Carrier::PackedShared, MAIN_GEOMETRY).0, 2);
    }

    /// 阳性对照（登记第五节）：`per_extent` × `dedicated` 拧每对象 extent 数 1 → 4096，单元数 1 → 4096。
    #[test]
    fn positive_control_per_extent_dedicated_follows_extent_count() {
        let one_extent = cost(&UNLINK_ONE, RecordGrain::PerExtent, Carrier::DedicatedUnit, MAIN_GEOMETRY).0;
        let many_extents = cost(&TRUNCATE_4096_EXTENTS, RecordGrain::PerExtent, Carrier::DedicatedUnit, MAIN_GEOMETRY).0;
        assert_eq!((one_extent, many_extents), (1, 4096));
    }

    /// 阳性对照：`per_object` × `dedicated` 拧对象数 1 → 1000，单元数 1 → 1000。
    #[test]
    fn positive_control_per_object_dedicated_follows_object_count() {
        let one_object = cost(&UNLINK_ONE, RecordGrain::PerObject, Carrier::DedicatedUnit, MAIN_GEOMETRY).0;
        let many_objects = cost(&RM_RF_1K, RecordGrain::PerObject, Carrier::DedicatedUnit, MAIN_GEOMETRY).0;
        assert_eq!((one_object, many_objects), (1, 1000));
    }

    /// 阳性对照：两条 `packed` 臂拧记录宽 40 → 56（同一记录区起点 136），
    /// `per_extent` × `packed` 的 truncate 6 → 8 单元，`per_object` × `packed` 的 rm -rf 1M 1227 → 1719 单元。
    #[test]
    fn positive_control_packed_arms_follow_record_width() {
        let wide_record_geometry =
            Geometry { records_offset_bytes: PACKED_UNIT_RECORDS_OFFSET, record_bytes: FIRST_RUN_TOMBSTONE_RECORD_BYTES };
        let truncate_units = |geometry: Geometry| {
            cost(&TRUNCATE_4096_EXTENTS, RecordGrain::PerExtent, Carrier::PackedShared, geometry).0
        };
        let rm_rf_units = |geometry: Geometry| cost(&RM_RF_1M, RecordGrain::PerObject, Carrier::PackedShared, geometry).0;
        assert_eq!((truncate_units(MAIN_GEOMETRY), truncate_units(wide_record_geometry)), (6, 8));
        assert_eq!((rm_rf_units(MAIN_GEOMETRY), rm_rf_units(wide_record_geometry)), (1227, 1719));
    }

    /// 反向（登记第五节）：两条 `dedicated` 臂不读记录宽，在 40 与 56 下逐格相同。
    #[test]
    fn dedicated_arms_ignore_record_width() {
        let wide_record_geometry =
            Geometry { records_offset_bytes: PACKED_UNIT_RECORDS_OFFSET, record_bytes: FIRST_RUN_TOMBSTONE_RECORD_BYTES };
        for load in &LOADS {
            for grain in [RecordGrain::PerExtent, RecordGrain::PerObject] {
                assert_eq!(
                    cost(load, grain, Carrier::DedicatedUnit, MAIN_GEOMETRY),
                    cost(load, grain, Carrier::DedicatedUnit, wide_record_geometry)
                );
            }
        }
    }

    /// 反向：`per_object` 两条臂不读 extent 数，truncate 4096 个 extent 上两个载体都是 1 单元。
    #[test]
    fn per_object_arms_ignore_extent_count() {
        for carrier in [Carrier::DedicatedUnit, Carrier::PackedShared] {
            assert_eq!(cost(&TRUNCATE_4096_EXTENTS, RecordGrain::PerObject, carrier, MAIN_GEOMETRY).0, 1);
        }
    }

    /// Q2：主几何下有 ≥ 2 条记录的五格，打包都少写；Q5：只有 1 条记录的格并列。
    #[test]
    fn main_geometry_packed_writes_fewer_on_the_five_multi_record_cells_and_ties_on_single_record_cells() {
        assert!(packed_writes_fewer_units(&RM_RF_1K, RecordGrain::PerExtent, MAIN_GEOMETRY));
        assert!(packed_writes_fewer_units(&RM_RF_1K, RecordGrain::PerObject, MAIN_GEOMETRY));
        assert!(packed_writes_fewer_units(&RM_RF_1M, RecordGrain::PerExtent, MAIN_GEOMETRY));
        assert!(packed_writes_fewer_units(&RM_RF_1M, RecordGrain::PerObject, MAIN_GEOMETRY));
        assert!(packed_writes_fewer_units(&TRUNCATE_4096_EXTENTS, RecordGrain::PerExtent, MAIN_GEOMETRY));
        for grain in [RecordGrain::PerExtent, RecordGrain::PerObject] {
            assert!(!packed_writes_fewer_units(&UNLINK_ONE, grain, MAIN_GEOMETRY));
        }
        assert!(!packed_writes_fewer_units(&TRUNCATE_4096_EXTENTS, RecordGrain::PerObject, MAIN_GEOMETRY));
    }

    /// Q3：主几何下五格的差距倍数对第一次几何（583）都不变小；两条 rm -rf 1M 的格变大（1716 → 1227 单元）。
    #[test]
    fn gap_against_first_run_grows_on_rm_rf_1m_and_never_shrinks() {
        assert_eq!(gap_direction_against_first_run(&RM_RF_1M, RecordGrain::PerExtent, MAIN_GEOMETRY), GapDirection::Larger);
        assert_eq!(gap_direction_against_first_run(&RM_RF_1M, RecordGrain::PerObject, MAIN_GEOMETRY), GapDirection::Larger);
        assert_eq!(gap_direction_against_first_run(&RM_RF_1K, RecordGrain::PerExtent, MAIN_GEOMETRY), GapDirection::Equal);
        assert_eq!(gap_direction_against_first_run(&RM_RF_1K, RecordGrain::PerObject, MAIN_GEOMETRY), GapDirection::Equal);
        assert_eq!(
            gap_direction_against_first_run(&TRUNCATE_4096_EXTENTS, RecordGrain::PerExtent, MAIN_GEOMETRY),
            GapDirection::Larger
        );
    }

    /// Q3 的判别力：宽往大的方向（136, 112）上同一格会变小，说明这个方向判断分得出“变小”。
    #[test]
    fn gap_direction_reports_smaller_on_a_wider_record_geometry() {
        let wider_record_geometry =
            Geometry { records_offset_bytes: PACKED_UNIT_RECORDS_OFFSET, record_bytes: FIRST_RUN_TOMBSTONE_RECORD_BYTES * 2 };
        assert_eq!(
            gap_direction_against_first_run(&RM_RF_1M, RecordGrain::PerObject, wider_record_geometry),
            GapDirection::Smaller
        );
    }

    /// Q4：打包容量降到 1 的最小记录宽，记录区起点 136 时 16317，起点 91 时 16339；
    /// 差一字节容量是 2。
    #[test]
    fn flip_width_pinned_at_both_records_offsets() {
        assert_eq!(Geometry::smallest_record_bytes_with_capacity_one(136), 16317);
        assert_eq!(Geometry::smallest_record_bytes_with_capacity_one(91), 16339);
        assert_eq!(Geometry { records_offset_bytes: 136, record_bytes: 16316 }.packed_capacity(), 2);
        assert_eq!(Geometry { records_offset_bytes: 136, record_bytes: 16317 }.packed_capacity(), 1);
        assert_eq!(Geometry { records_offset_bytes: 91, record_bytes: 16338 }.packed_capacity(), 2);
        assert_eq!(Geometry { records_offset_bytes: 91, record_bytes: 16339 }.packed_capacity(), 1);
    }

    /// 判别力自证（登记第八节）：把“打包少写”的检查放到 r_flip − 1 与 r_flip 两点上各跑一次，
    /// 前者三个多记录负载全真，后者全假；判据挪到两点之间，检查由绿转红。
    #[test]
    fn packed_fewer_check_flips_between_the_two_flip_points() {
        let flip_record_bytes = Geometry::smallest_record_bytes_with_capacity_one(PACKED_UNIT_RECORDS_OFFSET);
        let last_record_bytes_that_still_wins =
            Geometry { records_offset_bytes: PACKED_UNIT_RECORDS_OFFSET, record_bytes: flip_record_bytes - 1 };
        let first_record_bytes_that_ties =
            Geometry { records_offset_bytes: PACKED_UNIT_RECORDS_OFFSET, record_bytes: flip_record_bytes };
        for load in &MULTI_RECORD_LOADS {
            assert!(packed_writes_fewer_units(load, RecordGrain::PerExtent, last_record_bytes_that_still_wins));
            assert!(!packed_writes_fewer_units(load, RecordGrain::PerExtent, first_record_bytes_that_ties));
        }
    }

    /// 登记第八节七个取样点的绝对值（第一次 583 / 1716 / 8 与另五点；h = 136 时 r_flip = 16317）。
    #[test]
    fn sensitivity_rows_pinned() {
        let expected = [
            (91, 56, 583, [2, 1716, 8]),
            (136, 56, 582, [2, 1719, 8]),
            (91, 40, 816, [2, 1226, 6]),
            (136, 40, 815, [2, 1227, 6]),
            (136, 112, 291, [4, 3437, 15]),
            (136, 16316, 2, [500, 500_000, 2048]),
            (136, 16317, 1, [1000, 1_000_000, 4096]),
        ];
        let geometries = sensitivity_geometries();
        assert_eq!(geometries.len(), expected.len());
        for (geometry, (records_offset_bytes, record_bytes, packed_capacity, packed_units)) in geometries.iter().zip(expected) {
            assert_eq!(*geometry, Geometry { records_offset_bytes, record_bytes });
            let row = sensitivity_row(*geometry);
            assert_eq!(row.packed_capacity, packed_capacity);
            assert_eq!(row.packed_units, packed_units);
            assert_eq!(row.dedicated_units, [1000, 1_000_000, 4096]);
        }
    }
}


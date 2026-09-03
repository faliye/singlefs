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
//! 单元 32768、头 91（D18 已定项 7 的数据单元头初值——墓碑单元用五元组，属数据单元类；
//! 2026-09-03 从已废弃的 55 字节提议值改过来）、墓碑记录 56（五元组 + 区间，按 E23（journal 几何）点名项宽度的量级）
//! ⇒ 打包容量 (32768 − 91) / 56 = **583 条/单元**；物理字节 = 单元数 × 32768 × 2（w=2）。
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
//! 回收规则未定 ⇒ 打包臂的钉住量算不出（判据 3 只标定性）；墓碑记录宽度 56 是量级假设
//! （字段表按 D18 已定项 7 走，跑的时候还未定）；多 extent 文件在 rm -rf 里按单 extent 算（小文件口径）。

use e7_index_bench::Emitter;

const UNIT: u64 = 32768;
const UNIT_HDR: u64 = 91;
const TOMB_REC: u64 = 56;
/// 第一版 2 盘恒 w=2：每个单元物理两份。
const PHYS_FACTOR: u64 = 2;

fn packed_capacity() -> u64 {
    (UNIT - UNIT_HDR) / TOMB_REC
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
    extents_per_obj: u64,
}

/// 该组合下的墓碑单元数与物理字节。
fn cost(load: &Load, grain: RecordGrain, carrier: Carrier) -> (u64, u64) {
    let records = match grain {
        RecordGrain::PerExtent => load.objects * load.extents_per_obj,
        RecordGrain::PerObject => load.objects,
    };
    let units = match carrier {
        Carrier::DedicatedUnit => records,
        Carrier::PackedShared => records.div_ceil(packed_capacity()),
    };
    (units, units * UNIT * PHYS_FACTOR)
}

fn main() {
    let mut em = Emitter::new();
    println!(
        "{}",
        em.emit_raw(&format!(
            "name=config unit={UNIT} hdr={UNIT_HDR} rec={TOMB_REC} packed_cap={} phys_factor={PHYS_FACTOR} model=arithmetic file_ops=0",
            packed_capacity()
        ))
    );
    let loads = [
        Load { name: "unlink_one", objects: 1, extents_per_obj: 1 },
        Load { name: "rm_rf_1k", objects: 1000, extents_per_obj: 1 },
        Load { name: "rm_rf_1m", objects: 1_000_000, extents_per_obj: 1 },
        Load { name: "truncate_4096ext", objects: 1, extents_per_obj: 4096 },
    ];
    for load in &loads {
        for grain in [RecordGrain::PerExtent, RecordGrain::PerObject] {
            for carrier in [Carrier::DedicatedUnit, Carrier::PackedShared] {
                let (units, bytes) = cost(load, grain, carrier);
                let g = match grain {
                    RecordGrain::PerExtent => "per_extent",
                    RecordGrain::PerObject => "per_object",
                };
                let c = match carrier {
                    Carrier::DedicatedUnit => "dedicated",
                    Carrier::PackedShared => "packed",
                };
                println!(
                    "{}",
                    em.emit_raw(&format!(
                        "name=cost load={} grain={g} carrier={c} units={units} phys_bytes={bytes} phys_mib={:.1}",
                        load.name,
                        bytes as f64 / (1024.0 * 1024.0)
                    ))
                );
            }
        }
    }
    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **绝对值锚点**：打包容量 (32768−91)/56 = 583。
    #[test]
    fn absolute_packed_capacity() {
        assert_eq!(packed_capacity(), 583);
    }

    /// **绝对值锚点**：unlink 一个小文件——独占单元 = 1 单元 = 64 KiB 物理；
    /// 打包 = 同样 1 单元（583 容量装 1 条）。
    #[test]
    fn absolute_unlink_one() {
        let l = Load { name: "u", objects: 1, extents_per_obj: 1 };
        assert_eq!(cost(&l, RecordGrain::PerExtent, Carrier::DedicatedUnit), (1, 65536));
        assert_eq!(cost(&l, RecordGrain::PerExtent, Carrier::PackedShared), (1, 65536));
    }

    /// **绝对值锚点（判据 2 点名的那格）**：rm -rf 1M 单 extent 文件、每条记录独占单元
    /// ⇒ 1M 单元 × 64 KiB = 61.04 GiB 物理写。打包 ⇒ ⌈1e6/583⌉ = 1716 单元 ≈ 107.25 MiB。
    #[test]
    fn absolute_rm_rf_1m() {
        let l = Load { name: "m", objects: 1_000_000, extents_per_obj: 1 };
        let (u_ded, b_ded) = cost(&l, RecordGrain::PerExtent, Carrier::DedicatedUnit);
        assert_eq!(u_ded, 1_000_000);
        assert_eq!(b_ded, 65_536_000_000);
        let (u_pack, b_pack) = cost(&l, RecordGrain::PerExtent, Carrier::PackedShared);
        assert_eq!(u_pack, 1716);
        assert_eq!(b_pack, 1716 * 65536);
        // 单 extent 文件下 per_object 与 per_extent 逐格相同
        assert_eq!(
            cost(&l, RecordGrain::PerObject, Carrier::PackedShared),
            (u_pack, b_pack)
        );
    }

    /// **绝对值锚点**：truncate 4096 个 extent——per_extent 打包 ⌈4096/583⌉ = 8 单元；
    /// per_object 恒 1 单元（区间记录）。粒度在多 extent 对象上差 4096 倍（独占载体）。
    #[test]
    fn absolute_truncate() {
        let l = Load { name: "t", objects: 1, extents_per_obj: 4096 };
        assert_eq!(cost(&l, RecordGrain::PerExtent, Carrier::PackedShared).0, 8);
        assert_eq!(cost(&l, RecordGrain::PerObject, Carrier::DedicatedUnit).0, 1);
        assert_eq!(cost(&l, RecordGrain::PerExtent, Carrier::DedicatedUnit).0, 4096);
    }

    /// 打包容量对记录宽度的敏感性自检：宽度翻倍容量约减半（口径变了数字跟着变的那类）。
    #[test]
    fn capacity_scales_with_record_width() {
        assert_eq!((UNIT - UNIT_HDR) / (TOMB_REC * 2), 291);
    }

    /// 物理系数：w=2 恒双份——公式里少了它，全表数字砍半。
    #[test]
    fn phys_factor_is_applied() {
        let l = Load { name: "u", objects: 1, extents_per_obj: 1 };
        let (_, bytes) = cost(&l, RecordGrain::PerExtent, Carrier::DedicatedUnit);
        assert_eq!(bytes, UNIT * 2);
    }

    /// div_ceil 不许写成整除：585 条要 2 个单元。
    #[test]
    fn ceil_not_floor() {
        let l = Load { name: "x", objects: 585, extents_per_obj: 1 };
        assert_eq!(cost(&l, RecordGrain::PerExtent, Carrier::PackedShared).0, 2);
    }
}

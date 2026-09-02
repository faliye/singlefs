//! E34：根环槽几何 —— 槽宽与设备映射单元（`io_min`）的关系。
//!
//! ## 被引用条款逐字贴在这里（verify-before-claiming.md「把定义句原样贴进实验注释」）
//!
//! - D20（承重面：单元的原子性与自包含）已定：自证单元的原子宽度「**等于运行时探测到的 `physical_block_size`**，不许硬编码」。
//! - D2 已定硬要求 1：「**不发出小于设备物理映射单元的写**。在 Linux 上这个量是 `io_min`，
//!   不是 `physical_block_size`」。硬要求 2：「**不让两个生命周期不同的对象共享同一个物理映射单元**」。
//! - Linux 稳定 ABI（`Documentation/ABI/stable/sysfs-block`，2026-08-29 现查）：
//!   `minimum_io_size`「For RAID arrays it is often the **stripe chunk size**」；
//!   `physical_block_size` 是「the smallest unit a physical storage device can write **atomically**」。
//!   ⇒ 两个量管的不是一件事，内核里 `physical_block_size ≤ io_min` 恒成立。
//! - **D22 已定项 2（2026-08-31 用户定案，E34 建档之后两天才定）**：
//!   「**R = 3 个区域；区域按素数步长放置**（区域 r 落在 `r × P × chunk`，P 素数且 `P > devs`）**、
//!   槽序跨区轮转**（区域 = `txg mod R`）**；槽宽 = 挂载时探测的 `physical_block_size`；不留保留槽。**」
//!   同处逐字记着：「**候选形态 2 区 × 4 槽 × 256 字节整个被换掉**」。
//! - E53 真设备实测：4 盘逐块抹一次，**恒挂得上、最坏回退 1 代**。
//!
//! ## ⚠️ E34 的原始前提已经过期，这里只换前提，不换判据
//!
//! E34 正文写的是「D22 已定项 2 **提出的那个候选**根环几何（2 区域 × 4 槽 × 256 字节）」。
//! 那个候选**已于 2026-08-31 被整个换掉**（槽宽 256 由 E41 否掉，R=4 由 E47 否掉）。
//! ⇒ **主张 1 的那个具体参数点已由 E41 答过**；这里把 E34 的判据**原样**跑在
//! **已定的**几何上——判据本来就是按 `(physical_block_size, io_min, 槽宽, 区域间距)` 参数化写的，
//! 没有一个字绑在 256 上。
//!
//! ## 判据（E34 正文跑前写死，跑完不许改）
//!
//! - **测什么**：给定 `(physical_block_size, io_min, 槽宽, 区域间距)`，
//!   量「一次写打掉的槽数」与「掉电后仍可读出的最新根的代数」。
//! - **判据**：存在任一参数组合使**一次写打掉 ≥ 2 个槽** ⇒ 主张 1 成立；
//!   存在任一组合使**一次写打掉全部槽** ⇒ 主张 2 成立。
//! - **阳性对照**：槽宽设成 ≥ `io_min` 且每槽独占一个 `io_min` 单元，
//!   上面两个量必须都降到 1 —— 否则「打掉多个槽」分不清是几何的错还是模型没在按单元算。
//! - **失败条款**：若在全部现实参数组合上一次写都只打掉 1 个槽，主张不成立，**如实记录**。
//! - ⚠️ **必须带绝对值断言**：「一次写打掉几个槽」由 `(槽宽, 单元宽, 起始偏移)` 直接算出，
//!   不许从被测代码读回来。
//!
//! ## 它答不了的
//!
//! **纯几何算术，不碰设备**，文件操作 0 处。
//! 它算的是「**几个槽落在同一个 `io_min` 单元里**」，
//! **不是**「掉电时那个单元真的会被设备内部 RMW 打掉」——后者要能设 `io_min` 的真设备 +
//! 掉电注入，本机 `io_min == physical_block_size == 512` 取不到那种几何。
//! ⇒ 「一次写打掉 N 个槽」在这里的意思是**暴露面 N**，不是观测到的损失。

use e7_index_bench::Emitter;

/// D22 已定项 2：根环区域数。
const RING_REGIONS: u64 = 3;

/// 槽宽怎么取。**没有 `_ =>` 通配臂**：加第三种取法时这里编译不过。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum SlotWidth {
    /// D22 已定项 2 已定的取法：等于探测到的 `physical_block_size`。
    SettledPbs,
    /// E34 建议的取法：`max(physical_block_size, io_min)`，每槽独占一个 `io_min` 单元。
    /// **同时是判据要的阳性对照。**
    SuggestedIoMin,
}

impl SlotWidth {
    fn bytes(self, pbs: u64, io_min: u64) -> u64 {
        match self {
            SlotWidth::SettledPbs => pbs,
            SlotWidth::SuggestedIoMin => pbs.max(io_min),
        }
    }
    fn tag(self) -> &'static str {
        match self {
            SlotWidth::SettledPbs => "settled_pbs",
            SlotWidth::SuggestedIoMin => "suggested_iomin",
        }
    }
}

/// 区域怎么放。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Placement {
    /// D22 已定项 2：区域 r 落在 `r × P × chunk`，P 素数且 `P > devs`，chunk 取 `io_min`。
    PrimeStride { p: u64 },
    /// 已被换掉的候选形态：整个根环连续摆放（2048 字节那种）。**对照臂**，
    /// 没有它就分不出「主张 2 不成立」是几何挡住的、还是模型根本没在算跨区。
    Contiguous,
}

/// 区域 r 的起始字节偏移。
fn region_offset(pl: Placement, r: u64, chunk: u64, slots: u64, slot_w: u64) -> u64 {
    match pl {
        Placement::PrimeStride { p } => r * p * chunk,
        Placement::Contiguous => r * slots * slot_w,
    }
}

/// **绝对值算术**：一个 `io_min` 单元里落得下几个槽。
/// 由 `(槽宽, 单元宽)` 直接算，不查任何被测结构。
fn slots_per_unit(slot_w: u64, io_min: u64) -> u64 {
    if slot_w >= io_min {
        1
    } else {
        io_min / slot_w
    }
}

/// 一次写（打掉一个 `io_min` 单元）最多打掉几个槽。
/// 区域内槽是连续的 ⇒ 上限是该区域的槽数。
fn slots_hit_by_one_write(slot_w: u64, io_min: u64, slots_per_region: u64) -> u64 {
    slots_per_unit(slot_w, io_min).min(slots_per_region)
}

/// 一个 `io_min` 单元能不能同时盖住两个区域。
/// 盖得住 ⇒ 主张 2（一次写打掉全部槽）在这个几何上成立。
fn one_unit_spans_two_regions(
    pl: Placement,
    io_min: u64,
    chunk: u64,
    slots: u64,
    slot_w: u64,
) -> bool {
    // 相邻两个区域的起点间距；间距 < io_min 就可能挤进同一个单元。
    let a = region_offset(pl, 0, chunk, slots, slot_w);
    let b = region_offset(pl, 1, chunk, slots, slot_w);
    b - a < io_min
}

/// 一次写打掉的槽数占全环槽数的比例够不够「全部」。
fn wipes_whole_ring(pl: Placement, io_min: u64, chunk: u64, slots: u64, slot_w: u64) -> bool {
    // 整环连续摆放时，环总字节 = R × slots × slot_w；单元盖得住它就等于全打掉。
    match pl {
        Placement::Contiguous => RING_REGIONS * slots * slot_w <= io_min,
        Placement::PrimeStride { .. } => {
            // 素数步长下相邻区域间距 = P × chunk ≥ 3 × chunk；盖不住两个区域就盖不住全环。
            one_unit_spans_two_regions(pl, io_min, chunk, slots, slot_w)
                && RING_REGIONS * slots * slot_w <= io_min
        }
    }
}

/// 抹掉一个区域之后，最新还读得出来的根要回退几代。
/// 槽序跨区轮转：`txg` 落在区域 `txg mod R`。抹掉区域 `w` ⇒ 所有 `txg ≡ w (mod R)` 没了。
/// 从最新的 `txg` 往回找第一个幸存的，取最坏的那个起点。
fn rollback_after_losing_region(regions: u64, wiped: u64) -> u64 {
    (0..regions)
        .map(|newest| {
            let mut back = 0;
            // 往回走，直到找到一个不在被抹区域里的 txg
            while (newest + regions - back % regions) % regions == wiped % regions {
                back += 1;
                if back > regions {
                    break;
                }
            }
            back
        })
        .max()
        .unwrap_or(0)
}

fn main() {
    let mut em = Emitter::new();
    println!(
        "{}",
        em.emit_raw(&format!(
            "name=config ring_regions={RING_REGIONS} model=geometry file_ops=0 \
             host_pbs=512 host_io_min=512"
        ))
    );

    // ── 主扫：两种槽宽取法 × 两种放置 × (pbs, io_min, S) ──────────────────
    for pbs in [512u64, 4096] {
        for io_min in [512u64, 4096, 65536] {
            if io_min < pbs {
                continue; // 内核里 physical_block_size ≤ io_min 恒成立
            }
            for s in [1u64, 4, 8, 16] {
                for sw in [SlotWidth::SettledPbs, SlotWidth::SuggestedIoMin] {
                    let slot_w = sw.bytes(pbs, io_min);
                    for (pl_tag, pl) in [
                        ("prime_stride", Placement::PrimeStride { p: 11 }),
                        ("contiguous", Placement::Contiguous),
                    ] {
                        let hit = slots_hit_by_one_write(slot_w, io_min, s);
                        let whole = wipes_whole_ring(pl, io_min, io_min, s, slot_w);
                        println!(
                            "{}",
                            em.emit_raw(&format!(
                                "name=exposure pbs={pbs} io_min={io_min} slots_per_region={s} \
                                 slot_width_rule={} slot_width={slot_w} placement={pl_tag} \
                                 slots_hit={hit} claim1_ge2={} claim2_whole_ring={}",
                                sw.tag(),
                                u8::from(hit >= 2),
                                u8::from(whole)
                            ))
                        );
                    }
                }
            }
        }
    }

    // ── 抹掉一个区域之后的回退深度（跨区轮转） ────────────────────────────
    for wiped in 0..RING_REGIONS {
        println!(
            "{}",
            em.emit_raw(&format!(
                "name=rollback regions={RING_REGIONS} wiped_region={wiped} \
                 worst_rollback_generations={}",
                rollback_after_losing_region(RING_REGIONS, wiped)
            ))
        );
    }

    // ── 阳性对照：槽宽 ≥ io_min 且每槽独占一个单元 ⇒ 两个量都必须降到 1 ────
    for io_min in [512u64, 4096, 65536] {
        let slot_w = SlotWidth::SuggestedIoMin.bytes(512, io_min);
        let hit = slots_hit_by_one_write(slot_w, io_min, 16);
        let whole = wipes_whole_ring(Placement::PrimeStride { p: 11 }, io_min, io_min, 16, slot_w);
        println!(
            "{}",
            em.emit_raw(&format!(
                "name=positive_control io_min={io_min} slot_width={slot_w} slots_hit={hit} \
                 whole_ring={} both_are_one={}",
                u8::from(whole),
                u8::from(hit == 1 && !whole)
            ))
        );
    }

    // ── 已被换掉的候选形态，留一格做对照：2 区 × 4 槽 × 256 字节 = 2048 B ──
    let dead_hit = slots_hit_by_one_write(256, 65536, 4);
    let dead_whole = 2 * 4 * 256 <= 65536;
    println!(
        "{}",
        em.emit_raw(&format!(
            "name=dead_candidate slot_width=256 io_min=65536 regions=2 slots=4 \
             slots_hit={dead_hit} whole_ring={} note=replaced_by_d22_item2",
            u8::from(dead_whole)
        ))
    );

    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **绝对值断言**：一个单元里落得下几个槽，由 `(槽宽, 单元宽)` 直接算。
    #[test]
    fn absolute_slots_per_unit() {
        assert_eq!(slots_per_unit(512, 65536), 128); // 65536 / 512
        assert_eq!(slots_per_unit(512, 4096), 8);
        assert_eq!(slots_per_unit(512, 512), 1);
        assert_eq!(slots_per_unit(4096, 4096), 1);
        assert_eq!(slots_per_unit(65536, 65536), 1);
        // 槽比单元还宽 ⇒ 一个单元只碰得到一个槽。
        assert_eq!(slots_per_unit(65536, 512), 1);
    }

    /// **判据的主张 1**：已定的槽宽取法（= `physical_block_size`）在阵列上
    /// 一次写打掉 **≥ 2 个槽**。S=16、pbs=512、io_min=64 KiB ⇒ 打掉全部 16 个。
    #[test]
    fn claim1_settled_slot_width_exposes_multiple_slots() {
        let slot_w = SlotWidth::SettledPbs.bytes(512, 65536);
        assert_eq!(slot_w, 512);
        assert_eq!(slots_hit_by_one_write(slot_w, 65536, 16), 16, "整个区域");
        assert_eq!(slots_hit_by_one_write(slot_w, 4096, 16), 8);
        assert!(slots_hit_by_one_write(slot_w, 65536, 16) >= 2, "主张 1 成立");
    }

    /// **本机上主张 1 不成立**——`io_min == physical_block_size == 512`
    /// ⇒ 一次写只打掉 1 个槽。这正是 E34 正文说的「本机无法证伪」。
    #[test]
    fn claim1_does_not_hold_on_this_host() {
        let slot_w = SlotWidth::SettledPbs.bytes(512, 512);
        assert_eq!(slots_hit_by_one_write(slot_w, 512, 16), 1);
    }

    /// **判据的主张 2**：素数步长放置下，一个单元盖不住两个区域 ⇒ 打不掉全环。
    /// 而已被换掉的连续候选上它成立。
    #[test]
    fn claim2_holds_for_the_dead_candidate_but_not_for_the_settled_placement() {
        // 已定几何：区域间距 = 11 × 65536 = 720896 ≫ 65536
        assert!(!one_unit_spans_two_regions(
            Placement::PrimeStride { p: 11 }, 65536, 65536, 16, 512
        ));
        assert!(!wipes_whole_ring(Placement::PrimeStride { p: 11 }, 65536, 65536, 16, 512));
        // 已被换掉的候选：2 区 × 4 槽 × 256 = 2048 B，整个装得进一个 64 KiB 单元
        assert!(2 * 4 * 256 <= 65536);
        assert!(wipes_whole_ring(Placement::Contiguous, 65536, 65536, 4, 256));
        assert_eq!(slots_hit_by_one_write(256, 65536, 4), 4, "候选的 4 个槽全打掉");
    }

    /// **阳性对照**：槽宽 = `max(pbs, io_min)` ⇒ 两个量都降到 1。
    /// 降不到 1 说明模型没在按单元算，整轮作废。
    #[test]
    fn positive_control_iomin_slot_width_drops_both_to_one() {
        for io_min in [512u64, 4096, 65536] {
            let slot_w = SlotWidth::SuggestedIoMin.bytes(512, io_min);
            assert_eq!(slot_w, io_min.max(512));
            assert_eq!(slots_hit_by_one_write(slot_w, io_min, 16), 1, "io_min={io_min}");
            assert!(!wipes_whole_ring(
                Placement::PrimeStride { p: 11 }, io_min, io_min, 16, slot_w
            ));
        }
    }

    /// **绝对值断言**：跨区轮转下丢掉一整个区域，最坏回退 **1 代**。
    /// 与 E53 真设备实测的「最坏回退 1 代」逐格相等。
    #[test]
    fn absolute_rollback_is_one_generation() {
        for wiped in 0..RING_REGIONS {
            assert_eq!(rollback_after_losing_region(RING_REGIONS, wiped), 1);
        }
        // R=2 时同样是 1；R=1 时无处可退（整个环就一个区域）。
        assert_eq!(rollback_after_losing_region(2, 0), 1);
    }

    /// 区域偏移由放置规则直接算出，不从别处读。
    #[test]
    fn absolute_region_offsets() {
        assert_eq!(region_offset(Placement::PrimeStride { p: 11 }, 0, 65536, 16, 512), 0);
        assert_eq!(
            region_offset(Placement::PrimeStride { p: 11 }, 1, 65536, 16, 512),
            720_896
        );
        assert_eq!(region_offset(Placement::Contiguous, 1, 65536, 4, 256), 1024);
    }

    /// D22 已定项 2 的区域数是 3。
    #[test]
    fn format_constant_ring_regions() {
        assert_eq!(RING_REGIONS, 3);
    }
}

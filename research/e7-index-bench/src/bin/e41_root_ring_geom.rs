//! E41：根环槽几何 2×4×256 —— D22（单元原子性怎么合成）未定项 2 的候选形态站不站得住。
//!
//! ## 被测的那个形态（逐字）
//!
//! D22 未定项 2：「一个候选形态（**2 区域 × 4 槽 × 256 字节 = 2048 字节**）已提出，未验证。」
//! 同项还写下了本实验要判的那条判据：
//! 「**这一项不能只按崩溃恢复需要来定，必须同时满足『一圈的时长 ≥ 最坏的块重用延迟需求』**。」
//! 以及它与 fsync 率的绑定：D23 轴一已定「每次 fsync 发根」，
//! 而 E16 实测发根次数从「每 checkpoint 一次」变成「每 fsync 一次」是 **1000 倍**
//! （20 万次操作里 200 次 vs 200000 次）。
//!
//! ## 量三样
//!
//! 1. **轮换一圈**：`总槽数 ÷ 发根频率`。单位取**操作数**，不取挂钟——
//!    挂钟换台机器就变，而「多少次操作之内旧根会被覆盖」是结构量。
//! 2. **块重用延迟**：I-7.4（近 K 代块未被复用）逐字「最近 K 代根所引用的块，
//!    其物理范围均未被重新分配给其他对象（**K ≤ 根环深度**，取运行时生效值）」
//!    ⇒ 一圈（= 槽数 ÷ 发根频率）是 K 的上界，**不是 K 本身**。
//! 3. **一次撕裂打掉几个槽**：`原子宽度 ÷ 槽宽`，取 512 / 4096 / 65536 三档。
//!    本机现查 `nvme0n1`：`physical_block_size = logical_block_size = minimum_io_size = 512`。
//!
//! ## 判据与失败条款（立项时写下的，不是跑完才定的）
//!
//! - **几何够用**：一圈的时长 ≥ 最坏块重用延迟需求。
//! - **撕裂隔离**：一次写打掉的槽数必须为 **1**。⚠️ 本机 512 而候选槽宽 256
//!   ⇒ **预期这一条会红，那正是要量出来的东西，不是实现 bug**。
//! - **阳性对照**：槽宽设成 `max(physical_block_size, io_min)` 且每槽独占一个原子单元，
//!   打掉的槽数必须降到 1——否则模型根本没在按原子单元算，整轮作废。
//! - **失败条款**：两条判据都满足 ⇒ 候选成立，如实记录；
//!   **只有撕裂那条不满足 ⇒ 结论是「槽宽要抬」而不是「根环形态错」，两者不许混为一谈。**
//!
//! ## 全整数运算
//!
//! 发根频率写成分数（分子/分母），不走浮点——复跑要逐字节一致。

use e7_index_bench::Emitter;

// ── 候选形态（D22 未定项 2 逐字）──
const REGIONS: u64 = 2;
const SLOTS_PER_REGION: u64 = 4;
const SLOT_BYTES: u64 = 256;

// ── 本机现查（2026-08-30，`/sys/block/nvme0n1/queue/`）──
const HOST_PHYSICAL_BLOCK_SIZE: u64 = 512;
const HOST_IO_MIN: u64 = 512;

fn total_slots() -> u64 {
    REGIONS * SLOTS_PER_REGION
}

fn total_bytes() -> u64 {
    total_slots() * SLOT_BYTES
}

/// 轮换一圈要几次操作。发根频率写成分数 `num/den`（每次操作发几个根）。
/// 一圈 = 总槽数 ÷ 频率 = 槽数 × den / num。
fn cycle_ops(slots: u64, roots_num: u64, roots_den: u64) -> u64 {
    slots * roots_den / roots_num
}

/// 写一个槽会碰到几个原子单元。**起始偏移是输入**——立项时逐字要求
/// 「由 `(槽宽, 单元宽, 起始偏移)` 直接算出」，漏掉它就等于偷偷假设了一条
/// 「根槽按原子宽度对齐」的前提，而 [invariants.md](invariants.md) 里没有这一条。
fn atomic_units_touched(slot_off: u64, slot_bytes: u64, atomic: u64) -> u64 {
    ((slot_off % atomic) + slot_bytes).div_ceil(atomic)
}

/// 一次写把几个槽置于风险中：被碰到的那些原子单元覆盖得下几个槽宽。
fn slots_at_risk(slot_off: u64, slot_bytes: u64, atomic: u64) -> u64 {
    (atomic_units_touched(slot_off, slot_bytes, atomic) * atomic).div_ceil(slot_bytes)
}

/// 对齐摆放时的简写。**槽比原子单元还宽且对齐 ⇒ 只碰它自己。**
fn slots_hit_per_write(slot_bytes: u64, atomic: u64) -> u64 {
    slots_at_risk(0, slot_bytes, atomic)
}

/// 撕裂隔离成立 ⇔ 一次写只打掉一个槽。
fn tear_isolated(slot_bytes: u64, atomic: u64) -> bool {
    slots_hit_per_write(slot_bytes, atomic) == 1
}

/// 要保护 `delay_ops` 次操作的历史，在给定发根频率下需要几个槽。
/// 这是「一圈 ≥ 需求」的反解：槽数 = 需求 × 频率。
fn slots_required(delay_ops: u64, roots_num: u64, roots_den: u64) -> u64 {
    (delay_ops * roots_num).div_ceil(roots_den)
}

/// **整个根环装不装得进一个 `io_min`。** 装得进 ⇒ 设备内部一次 read-modify-write
/// 就能覆盖全环，所有见证者一起没。这是与撕裂**不同的机制**，
/// 治它的是把区域拉开到 ≥ `io_min`，不是抬槽宽。
fn ring_fits_in_one_io_min(ring_bytes: u64, io_min: u64) -> bool {
    ring_bytes <= io_min
}

/// 让撕裂隔离成立的最小槽宽：至少等于原子宽度。
fn min_slot_bytes_for_isolation(atomic: u64) -> u64 {
    atomic
}

// E16 实测的两档发根频率：每次 fsync 发根（20 万次操作 20 万次根）与
// 每 checkpoint 发根（20 万次操作 200 次根）。
const ROOT_RATES: [(&str, u64, u64); 5] = [
    ("per_fsync", 1, 1),
    ("half", 1, 2),
    ("tenth", 1, 10),
    ("hundredth", 1, 100),
    ("per_checkpoint", 1, 1000),
];

/// **撕裂的判定宽度只能取 `physical_block_size`**（D20 推论三：自证单元的原子宽度
/// 等于运行时探测到的 `physical_block_size`）。
/// ⚠️ **64 KiB 不在这一栏**：2026-08-30 逐行现查 `block/blk-settings.c:855-856`,
/// 堆叠取的是成员盘 `physical_block_size` 的 max；而 `drivers/md/raid0.c:393` 与
/// `raid5.c:7955` 都是 `lim.io_min = mddev->chunk_sectors << 9`——**chunk 落在 `io_min` 上**。
/// ⇒ 阵列上 pbs 仍是 512 / 4096。把 chunk 喂进这一栏会造出一个不存在的结论。
const ATOMICS: [u64; 2] = [512, 4096];

/// `io_min` 是**另一个机制**（设备内部 read-modify-write 损坏邻居，
/// D20 七机制表末行「写侧不发小于映射单元的写」）。它不判撕裂，
/// 治它的也不是抬槽宽，而是把两个区域拉开到 ≥ `io_min`（E34 的建议，未跑）。
/// 这里只报数，不下结论。
const IO_MINS: [u64; 3] = [512, 4096, 65536];
const SLOT_WIDTHS: [u64; 3] = [256, 512, 4096];

fn main() {
    let mut em = Emitter::new();

    println!(
        "{}",
        em.emit_raw(&format!(
            "name=config regions={REGIONS} slots_per_region={SLOTS_PER_REGION} \
             slot_bytes={SLOT_BYTES} total_slots={} total_bytes={} \
             host_pbs={HOST_PHYSICAL_BLOCK_SIZE} host_io_min={HOST_IO_MIN}",
            total_slots(),
            total_bytes(),
        ))
    );

    // ── 量一 / 量二：一圈要几次操作，就是块重用延迟的上界 ──
    for (name, num, den) in ROOT_RATES {
        println!(
            "{}",
            em.emit_raw(&format!(
                "name=cycle rate={name} roots_per_op={num}/{den} slots={} cycle_ops={} \
                 reuse_delay_ops={}",
                total_slots(),
                cycle_ops(total_slots(), num, den),
                cycle_ops(total_slots(), num, den),
            ))
        );
    }

    // ── 反解：要保护 N 次操作的历史，需要几个槽 ──
    for delay in [8u64, 100, 1000, 200_000] {
        for (name, num, den) in [ROOT_RATES[0], ROOT_RATES[4]] {
            println!(
                "{}",
                em.emit_raw(&format!(
                    "name=slots_required delay_ops={delay} rate={name} slots_needed={} \
                     candidate_slots={} enough={}",
                    slots_required(delay, num, den),
                    total_slots(),
                    u8::from(total_slots() >= slots_required(delay, num, den)),
                ))
            );
        }
    }

    // ── 量三：一次撕裂打掉几个槽 ──
    for slot in SLOT_WIDTHS {
        for atomic in ATOMICS {
            println!(
                "{}",
                em.emit_raw(&format!(
                    "name=tear slot_bytes={slot} atomic={atomic} slots_hit={} isolated={}",
                    slots_hit_per_write(slot, atomic),
                    u8::from(tear_isolated(slot, atomic)),
                ))
            );
        }
    }

    // ── 对齐维：立项要求的第三个输入。不对齐时槽宽等于原子宽度也照样跨两个单元。 ──
    for slot in SLOT_WIDTHS {
        for atomic in ATOMICS {
            for (tag, off) in [("aligned", 0u64), ("misaligned", slot / 2)] {
                println!(
                    "{}",
                    em.emit_raw(&format!(
                        "name=align slot_bytes={slot} atomic={atomic} placement={tag} slot_off={off} \
                         units_touched={} slots_at_risk={}",
                        atomic_units_touched(off, slot, atomic),
                        slots_at_risk(off, slot, atomic),
                    ))
                );
            }
        }
    }

    // ── io_min 那一栏：只报数，不判撕裂（它是设备内部 RMW，另一个机制）──
    for io_min in IO_MINS {
        println!(
            "{}",
            em.emit_raw(&format!(
                "name=io_min_span io_min={io_min} ring_bytes={} ring_fits_in_one_io_min={} \
                 mechanism=internal_rmw_not_tearing",
                total_bytes(),
                u8::from(ring_fits_in_one_io_min(total_bytes(), io_min)),
            ))
        );
    }

    // ── 阳性对照：槽宽抬到 max(pbs, io_min) 且每槽独占一个原子单元 ──
    let poscontrol_slot = HOST_PHYSICAL_BLOCK_SIZE.max(HOST_IO_MIN);
    println!(
        "{}",
        em.emit_raw(&format!(
            "name=poscontrol_tear slot_bytes={poscontrol_slot} atomic={HOST_PHYSICAL_BLOCK_SIZE} \
             slots_hit={} candidate_slots_hit={}",
            slots_hit_per_write(poscontrol_slot, HOST_PHYSICAL_BLOCK_SIZE),
            slots_hit_per_write(SLOT_BYTES, HOST_PHYSICAL_BLOCK_SIZE),
        ))
    );

    // ── 让撕裂隔离成立要把槽宽抬到多少 ──
    for atomic in ATOMICS {
        println!(
            "{}",
            em.emit_raw(&format!(
                "name=min_slot atomic={atomic} min_slot_bytes={} candidate_slot_bytes={SLOT_BYTES} \
                 ring_bytes_at_min={}",
                min_slot_bytes_for_isolation(atomic),
                total_slots() * min_slot_bytes_for_isolation(atomic),
            ))
        );
    }

    // ── 判决 ──
    println!(
        "{}",
        em.emit_raw(&format!(
            "name=verdict tear_isolated_on_host={} cycle_ops_per_fsync={} \
             reuse_requirement_source=none tear_axis=physical_block_size \
             io_min_axis=separate_mechanism",
            u8::from(tear_isolated(SLOT_BYTES, HOST_PHYSICAL_BLOCK_SIZE)),
            cycle_ops(total_slots(), 1, 1),
        ))
    );

    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **候选形态的算术必须复现 D22 逐字写下的那个数**：2 × 4 × 256 = 2048。
    #[test]
    fn the_candidate_shape_is_eight_slots_and_2048_bytes() {
        assert_eq!(total_slots(), 8);
        assert_eq!(total_bytes(), 2048);
    }

    /// **一圈的绝对值**：8 个槽、每次 fsync 发一个根 ⇒ **8 次操作**。
    /// 每 checkpoint 发一次（E16 实测 20 万次操作 200 次根，即 1/1000）⇒ 8000 次操作。
    /// 这两个数由 `(槽数, 发根频率)` 独立算出，不许从被测代码读回来。
    #[test]
    fn one_lap_is_eight_operations_under_per_fsync_roots() {
        assert_eq!(cycle_ops(8, 1, 1), 8);
        assert_eq!(cycle_ops(8, 1, 1000), 8000);
        // E16 实测的那个 1000 倍，在本模型里就是这两个数的比值
        assert_eq!(cycle_ops(8, 1, 1000) / cycle_ops(8, 1, 1), 1000);
    }

    /// **撕裂隔离在本机判红**，且红的幅度钉死：槽宽 256、原子宽度 512 ⇒ 打掉 2 个槽。
    /// 4 KiB 原子宽度上是 16 个，64 KiB 上是 256 个——**整个环一次全没**。
    #[test]
    fn the_candidate_slot_width_loses_more_than_one_slot_per_write() {
        assert_eq!(slots_hit_per_write(256, 512), 2);
        assert_eq!(slots_hit_per_write(256, 4096), 16);
        assert!(!tear_isolated(SLOT_BYTES, HOST_PHYSICAL_BLOCK_SIZE));
        // 判定宽度只在 ATOMICS 那两档里取，**不含 64 KiB**——
        // 那是 io_min 的量级（chunk），不是 physical_block_size 的量级。
        assert_eq!(ATOMICS, [512, 4096]);
    }

    /// **环里靠后的槽与第 0 个槽同价**：只要对齐，偏移多远都只碰一个单元。
    /// 变异测试抓出的盲区：偏移不取模时，第 8 个槽会被算成碰 9 个单元。
    #[test]
    fn a_slot_deep_in_the_ring_costs_the_same_as_the_first_one() {
        for k in 0..total_slots() {
            assert_eq!(
                slots_at_risk(k * 512, 512, 512),
                1,
                "第 {k} 个 512 字节槽（对齐摆放）该只碰一个原子单元"
            );
        }
        assert_eq!(atomic_units_touched(4096, 512, 512), 1);
        assert_eq!(atomic_units_touched(65536, 4096, 4096), 1);
    }

    /// **不对齐时，槽宽等于原子宽度也照样跨两个单元。**
    /// 立项逐字要求起始偏移是输入，而 [invariants.md] 里没有「根槽按原子宽度对齐」这一条
    /// ⇒ 阳性对照的「抬到 512 就降到 1」隐含了一条没人写下的前提。
    #[test]
    fn misalignment_defeats_a_slot_that_is_exactly_one_atomic_unit_wide() {
        assert_eq!(slots_at_risk(0, 512, 512), 1); // 对齐
        assert_eq!(slots_at_risk(256, 512, 512), 2); // 偏半个槽 ⇒ 跨两个单元
        assert_eq!(atomic_units_touched(256, 512, 512), 2);
        assert_eq!(atomic_units_touched(0, 512, 512), 1);
        // 更宽的槽同理
        assert_eq!(slots_at_risk(2048, 4096, 4096), 2);
    }

    /// **阳性对照**：槽宽抬到 `max(physical_block_size, io_min)` ⇒ 打掉的槽数降到 1。
    /// 测不出这个差别说明模型根本没在按原子单元算，整轮作废。
    #[test]
    fn positive_control_raising_the_slot_width_restores_isolation() {
        let s = HOST_PHYSICAL_BLOCK_SIZE.max(HOST_IO_MIN);
        assert_eq!(s, 512);
        assert_eq!(slots_hit_per_write(s, HOST_PHYSICAL_BLOCK_SIZE), 1);
        assert!(tear_isolated(s, HOST_PHYSICAL_BLOCK_SIZE));
        // 对照：候选槽宽在同一台机器上是 2
        assert_eq!(slots_hit_per_write(SLOT_BYTES, HOST_PHYSICAL_BLOCK_SIZE), 2);
    }

    /// **槽比原子单元还宽时，打掉的槽数是 1，永远不是 0。**
    /// 变异测试抓出的盲区：把 `>=` 写成 `>` 时，槽宽 4096、原子宽度 512 会走进除法分支
    /// 得到 `512 / 4096 = 0`——「一次写打掉 0 个槽」是无意义的答案，而没有任何检查看得见。
    #[test]
    fn a_slot_wider_than_the_atomic_unit_still_hits_exactly_one() {
        assert_eq!(slots_hit_per_write(4096, 512), 1);
        assert_eq!(slots_hit_per_write(512, 512), 1);
        assert_eq!(slots_hit_per_write(65536, 512), 1);
        for slot in SLOT_WIDTHS {
            for atomic in ATOMICS {
                assert!(
                    slots_hit_per_write(slot, atomic) >= 1,
                    "槽宽 {slot}、原子宽度 {atomic}：打掉的槽数不许是 0"
                );
            }
        }
    }

    /// **结论 4 的算术来源**：2048 字节的环装得进 4096 与 65536 的 `io_min`，装不进 512 的。
    /// ⚠️ 这一段此前零测试零变异，而它正是「真正抹掉全环的机制是设备内部 RMW」那条结论的出处。
    #[test]
    fn the_whole_ring_fits_inside_a_4k_or_larger_io_min() {
        assert_eq!(total_bytes(), 2048);
        assert!(!ring_fits_in_one_io_min(total_bytes(), 512));
        assert!(ring_fits_in_one_io_min(total_bytes(), 4096));
        assert!(ring_fits_in_one_io_min(total_bytes(), 65536));
        // 边界：环恰好等于 io_min 时算「装得进」——一次 RMW 正好盖住全环
        assert!(ring_fits_in_one_io_min(2048, 2048));
        assert!(!ring_fits_in_one_io_min(2049, 2048));
        // 逐档遍历，不是只判第一档
        for io_min in IO_MINS {
            assert_eq!(
                ring_fits_in_one_io_min(total_bytes(), io_min),
                io_min >= 2048,
                "io_min {io_min} 那一档判错"
            );
        }
    }

    /// **记一条等价变异**：`slot_bytes >= atomic` 里的 `>=` 换成 `>` 是**等价变异**，
    /// 因为 `slot == atomic` 时 else 分支的 `atomic / slot` 恰好也是 1。
    /// 抓不到它**不是盲区**——两个分支在全部输入上同值。
    #[test]
    fn the_two_branches_agree_exactly_at_the_boundary() {
        for w in [256u64, 512, 4096, 65536] {
            assert_eq!(w / w, 1, "边界上 else 分支给的就是 1，与提前返回的 1 同值");
            assert_eq!(slots_hit_per_write(w, w), 1);
        }
    }

    /// **反解必须向上取整**：不到一个槽的需求也得占一个槽。
    /// 变异测试抓出的盲区：写成整除时，「保护 8 次操作、每 1000 次操作发一个根」
    /// 会得到「需要 0 个槽」——而 0 个槽的环存不下任何见证者。
    #[test]
    fn slots_required_rounds_up_never_to_zero() {
        assert_eq!(slots_required(8, 1, 1000), 1);
        assert_eq!(slots_required(1, 1, 1000), 1);
        assert_eq!(slots_required(1001, 1, 1000), 2);
        for delay in [1u64, 8, 100, 1000, 200_000] {
            for (_, num, den) in ROOT_RATES {
                assert!(
                    slots_required(delay, num, den) >= 1,
                    "需求 {delay} 次操作、频率 {num}/{den}：算出 0 个槽"
                );
            }
        }
    }

    /// **阳性对照要对每一档原子宽度都跑，不是只跑本机那一档。**
    /// 三档各自把槽宽抬到该档的原子宽度，隔离必须全部恢复。
    #[test]
    fn positive_control_holds_for_every_atomic_width() {
        for a in ATOMICS {
            assert!(
                tear_isolated(min_slot_bytes_for_isolation(a), a),
                "原子宽度 {a}：槽宽抬到它之后隔离仍不成立"
            );
            assert!(
                !tear_isolated(SLOT_BYTES, a),
                "原子宽度 {a}：候选槽宽 256 居然隔离成立了"
            );
        }
    }

    /// **抬槽宽的代价**：隔离所需的最小槽宽 = 原子宽度 ⇒ 环的总字节跟着涨。
    /// 512 上是 4096 字节（候选的 2 倍），4 KiB 上是 32768（16 倍），64 KiB 上是 524288（256 倍）。
    #[test]
    fn the_price_of_isolation_is_pinned() {
        assert_eq!(total_slots() * min_slot_bytes_for_isolation(512), 4096);
        assert_eq!(total_slots() * min_slot_bytes_for_isolation(4096), 32768);
        assert_eq!(4096 / total_bytes(), 2);
        assert_eq!(32768 / total_bytes(), 16);
    }

    /// **反解**：要保护 1000 次操作的历史，在每次 fsync 发根之下需要 1000 个槽，
    /// 而候选只有 8 个。每 checkpoint 发根时只需 1 个。
    #[test]
    fn slots_required_scales_with_the_root_rate() {
        assert_eq!(slots_required(1000, 1, 1), 1000);
        assert_eq!(slots_required(1000, 1, 1000), 1);
        assert_eq!(slots_required(200_000, 1, 1), 200_000);
        assert!(slots_required(1000, 1, 1) > total_slots());
    }

    /// **校验路径**：一圈与反解互为逆运算，两条路径必须闭合。
    #[test]
    fn cycle_and_requirement_are_inverse() {
        for (_, num, den) in ROOT_RATES {
            let lap = cycle_ops(total_slots(), num, den);
            assert_eq!(
                slots_required(lap, num, den),
                total_slots(),
                "频率 {num}/{den}：一圈反解回去不等于槽数"
            );
        }
    }

    /// 校验路径自己要能红：把反解的分子分母调换，闭合必须断掉。
    #[test]
    fn the_inverse_check_itself_can_go_red() {
        let lap = cycle_ops(total_slots(), 1, 1000); // 8000
        assert_ne!(
            slots_required(lap, 1000, 1),
            total_slots(),
            "分子分母调换之后居然还闭合，这条校验是摆设"
        );
    }
}

//! E46：根环的区域间距与失败域 —— D22（单元原子性怎么合成）已定项 2 那一维的零覆盖。
//!
//! ## 它补的是什么洞
//!
//! E41（根环槽几何 2×4×256）实测了槽宽那一维，但它的模型里 `REGIONS` **只参与乘法**：
//! 没有区域间距、没有失败域两个输入，而那正是「2 区域」这个数唯一的理由。
//! E41 自己记了这一条：「**零覆盖，不是阴性结果**」。本实验补它。
//!
//! ## D22 已定项 2 自陈依赖的那个假设
//!
//! 逐字：「它依赖『盘首与盘尾失败域独立』这个假设，**本机没有证据**——
//! ZFS 的 4-label 设计吃的是同一个假设。」
//!
//! ⚠️ **本实验不去验证物理失败域**（那要真硬件）。它验证的是一件**纯几何、可判定**的事：
//! **在条带阵列上，「盘首」与「盘尾」这两个 LBA 会不会落在同一块物理盘上。**
//! 若会，那个假设在几何上就已经不成立了，不必等物理证据。
//!
//! ## 量三样
//!
//! 1. **落盘归属**：条带布局下，LBA `p` 落在第 `(p / chunk) % 设备数` 块盘上（RAID0 / RAID5 的
//!    数据分布形态）。两个区域落在同一块盘 ⇒ 失败域完全不独立。
//! 2. **一次内部 RMW 能不能同时打到两个区域**：`io_min` 宽的窗口能同时触到两区
//!    当且仅当 `间距 < io_min`。⇒ **间距 ≥ `io_min`** 是隔离的充分条件。
//! 3. **要几个区域**：容忍 `F` 个失败域同时失效，且至少留一个活的见证者，需要 `F + 1` 个区域，
//!    且它们**落在互不相同的失败域上**。
//!
//! ## 判据、阳性对照、失败条款
//!
//! - **判据一（同盘）**：候选布局（两区放盘首盘尾）在**多少种设备数**下落到同一块盘。
//!   落到同一块盘的比例不为 0 ⇒「盘首盘尾失败域独立」在几何上就不成立。
//! - **判据二（间距）**：一次 `io_min` 宽的 RMW 打到的区域数必须为 **1**。
//! - **阳性对照**（两条，各自独立）：
//!   ① 把两区放进**同一个 chunk** ⇒ 同盘判定必须为真；
//!   ② 把间距设成 0（两区相邻）⇒ 一次 RMW 打到的区域数必须为 2。
//!   测不出这两个差别 ⇒ 模型没在按几何算，整轮作废。
//! - **失败条款**：若同盘比例为 0 且间距判据在所有档上都满足，候选的「2 区域」成立，如实记录；
//!   若只有间距那条不满足，结论是「间距要抬」而不是「区域数错」——**两者不许混为一谈**。
//!
//! ## 它答不了的那一半
//!
//! **物理失败域的真实形状**（磁道、柱面、擦除块、控制器）本实验一概不碰。
//! 它只能证伪一个方向：**几何上就落同盘 ⇒ 假设已经不成立**；
//! 反过来「几何上不同盘」**不等于**物理失败域独立。

use e7_index_bench::Emitter;

/// 条带布局下，LBA `p` 落在哪块盘。`chunk` 是条带单元字节数，`devs` 是设备数。
/// 形态取自 Linux md：`raid0.c` / `raid5.c` 都把 chunk 写进 `io_min`，数据按 chunk 轮转。
fn device_of(address_in_bytes: u64, chunk: u64, device_count: u64) -> u64 {
    (address_in_bytes / chunk) % device_count
}

/// 两个区域是不是落在同一块盘上。
fn same_device(first_region_start: u64, second_region_start: u64, chunk: u64, device_count: u64) -> bool {
    device_of(first_region_start, chunk, device_count) == device_of(second_region_start, chunk, device_count)
}

/// 一个 `io_min` 宽的窗口最多能同时触到几个区域。
/// 两区分别占 `[p0, p0+size)` 与 `[p1, p1+size)`，`p1 > p0`。
/// 窗口触到两区 ⇔ 存在 `w` 使 `w < p0+size` 且 `w+io_min > p1` ⇔ `p1 - (p0+size) < io_min`。
fn regions_hit_by_one_read_modify_write(first_region_start: u64, second_region_start: u64, size: u64, io_min: u64) -> u64 {
    let gap = second_region_start.saturating_sub(first_region_start + size);
    if gap < io_min { 2 } else { 1 }
}

/// 容忍 `f` 个失败域同时失效、且至少留一个活见证者，要几个区域。
fn regions_needed(tolerated_failure_domains: u64) -> u64 {
    tolerated_failure_domains + 1
}

/// 盘尾那个区域的起点：设备总容量减去区域大小，按 chunk 对齐向下取整。
fn tail_position(capacity: u64, size: u64, chunk: u64) -> u64 {
    ((capacity - size) / chunk) * chunk
}

const CHUNKS: [u64; 3] = [64 * 1024, 512 * 1024, 4 * 1024 * 1024];
const DEVICE_COUNTS: [u64; 6] = [1, 2, 3, 4, 6, 8];
const IO_MINS: [u64; 3] = [512, 4096, 65536];
const TWO_REGION_RING_BYTES: u64 = 2048; // E41 候选：每区 4 槽 × 256 字节 = 1024；两区合计 2048
const CAPACITY: u64 = 10 * 1000 * 1000 * 1000 * 1000; // 10 TB，与 E40 同口径

fn main() {
    let mut emitter = Emitter::new();
    let region_bytes = TWO_REGION_RING_BYTES / 2; // 每个区域的字节数

    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=config region_bytes={region_bytes} ring_bytes={TWO_REGION_RING_BYTES} capacity={CAPACITY}"
        ))
    );

    // ── 量一：盘首与盘尾会不会落在同一块盘 ──
    let mut same_device_cases = 0u64;
    let mut total_cases = 0u64;
    for chunk in CHUNKS {
        for device_count in DEVICE_COUNTS {
            let head_address = 0u64;
            let tail_address = tail_position(CAPACITY, region_bytes, chunk);
            let is_same_device = same_device(head_address, tail_address, chunk, device_count);
            total_cases += 1;
            if is_same_device {
                same_device_cases += 1;
            }
            println!(
                "{}",
                emitter.emit_raw(&format!(
                    "name=domain chunk={chunk} devs={device_count} head_dev={} tail_lba={tail_address} tail_dev={} \
                     same_device={}",
                    device_of(head_address, chunk, device_count),
                    device_of(tail_address, chunk, device_count),
                    u8::from(is_same_device),
                ))
            );
        }
    }
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=domain_summary same_device_cases={same_device_cases} total_cases={total_cases} \
             head_tail_independent_geometrically={}",
            u8::from(same_device_cases == 0),
        ))
    );

    // ── 量二：一次内部 RMW 打到几个区域 ──
    for io_min in IO_MINS {
        for gap in [0u64, 512, 4096, 65536, 1 << 20] {
            let first_region_start = 0u64;
            let second_region_start = first_region_start + region_bytes + gap;
            println!(
                "{}",
                emitter.emit_raw(&format!(
                    "name=spacing io_min={io_min} gap={gap} regions_hit={} isolated={}",
                    regions_hit_by_one_read_modify_write(first_region_start, second_region_start, region_bytes, io_min),
                    u8::from(regions_hit_by_one_read_modify_write(first_region_start, second_region_start, region_bytes, io_min) == 1),
                ))
            );
        }
    }
    // 盘首盘尾那个候选布局的实际间距
    for io_min in IO_MINS {
        let tail_address = tail_position(CAPACITY, region_bytes, 64 * 1024);
        println!(
            "{}",
            emitter.emit_raw(&format!(
                "name=spacing_headtail io_min={io_min} gap={} regions_hit={}",
                tail_address - region_bytes,
                regions_hit_by_one_read_modify_write(0, tail_address, region_bytes, io_min),
            ))
        );
    }

    // ── 量三：要几个区域 ──
    for tolerated_failure_domains in 0u64..=3 {
        println!(
            "{}",
            emitter.emit_raw(&format!(
                "name=regions_needed tolerate_domains={tolerated_failure_domains} regions={} candidate_regions=2 enough={}",
                regions_needed(tolerated_failure_domains),
                u8::from(2 >= regions_needed(tolerated_failure_domains)),
            ))
        );
    }

    // ── 阳性对照 ──
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=poscontrol_same_chunk chunk=65536 devs=4 p0=0 p1=1024 same_device={} \
             expect=1",
            u8::from(same_device(0, 1024, 65536, 4)),
        ))
    );
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=poscontrol_zero_gap io_min=512 gap=0 regions_hit={} expect=2",
            regions_hit_by_one_read_modify_write(0, region_bytes, region_bytes, 512),
        ))
    );

    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;
    const REGION_BYTES: u64 = TWO_REGION_RING_BYTES / 2; // 1024

    /// **落盘归属的绝对值**：chunk 64 KiB、4 块盘 ⇒ LBA 0 在盘 0，
    /// LBA 65536 在盘 1，LBA 262144 回到盘 0（4 块盘轮一圈）。
    #[test]
    fn device_of_walks_the_stripe_round_robin() {
        assert_eq!(device_of(0, 65536, 4), 0);
        assert_eq!(device_of(65536, 65536, 4), 1);
        assert_eq!(device_of(65536 * 3, 65536, 4), 3);
        assert_eq!(device_of(65536 * 4, 65536, 4), 0); // 轮一圈回到盘 0
        assert_eq!(device_of(65535, 65536, 4), 0); // chunk 内不换盘
    }

    /// **本实验的要害**：10 TB、chunk 64 KiB 时，盘尾的 chunk 序号是 **152 587 890**。
    /// 独立算术：tail = ⌊(10¹³ − 1024) / 65536⌋ × 65536 = 9 999 999 959 040，
    /// 除以 65536 得 152 587 890；它 **能被 2、3、6 整除**（各位数字和 45 ⇒ 3 的倍数），
    /// ⇒ 在 2 / 3 / 6 块盘的池里，**盘尾与盘首落在同一块盘上**。
    #[test]
    fn head_and_tail_land_on_the_same_device_for_some_device_counts() {
        let chunk = 65536u64;
        let tail_address = tail_position(CAPACITY, REGION_BYTES, chunk);
        assert_eq!(tail_address, 9_999_999_959_040);
        assert_eq!(tail_address / chunk, 152_587_890);
        // 2 / 3 / 6 块盘：整除 ⇒ **与盘首同盘**
        for device_count in [2u64, 3, 6] {
            assert_eq!(device_of(tail_address, chunk, device_count), 0, "{device_count} 盘该同盘");
            assert!(same_device(0, tail_address, chunk, device_count));
        }
        // 4 / 8 块盘：152587890 % 4 = 2、% 8 = 2 ⇒ 不同盘
        assert_eq!(device_of(tail_address, chunk, 4), 2);
        assert_eq!(device_of(tail_address, chunk, 8), 2);
        assert!(!same_device(0, tail_address, chunk, 4));
    }

    /// **同盘不是罕见情形**：遍历三档 chunk × 六档设备数，数出同盘的格子数。
    /// 独立算术给出的期望是 10 / 18（单盘那六格必然同盘，另有四格轮到 0）。
    #[test]
    fn same_device_is_not_a_corner_case() {
        let mut same_device_cases = 0;
        let mut total_cases = 0;
        for chunk in CHUNKS {
            for device_count in DEVICE_COUNTS {
                total_cases += 1;
                if same_device(0, tail_position(CAPACITY, REGION_BYTES, chunk), chunk, device_count) {
                    same_device_cases += 1;
                }
            }
        }
        assert_eq!(total_cases, 18);
        // 独立算术逐格算出的是 7：chunk 64 KiB 档 4 格（1/2/3/6 盘）、
        // 512 KiB 档 2 格（1/2 盘）、4 MiB 档 1 格（单盘）。
        assert_eq!(same_device_cases, 7, "同盘的格子数");
        assert!(same_device_cases > 0, "只要有一格同盘，「盘首盘尾失败域独立」在几何上就不成立");
    }

    /// **单盘那一档必然同盘**——它只有一块盘。阳性对照的一半。
    #[test]
    fn single_device_pool_always_puts_both_regions_on_it() {
        for chunk in CHUNKS {
            assert!(same_device(0, tail_position(CAPACITY, REGION_BYTES, chunk), chunk, 1));
        }
    }

    /// **间距判据**：一次 `io_min` 宽的 RMW 打到两个区域，当且仅当间距 < `io_min`。
    /// 绝对值：间距 0 / 512 / 4096 在 io_min = 4096 下分别是 2 / 2 / 1。
    #[test]
    fn one_read_modify_write_reaches_both_regions_exactly_when_the_gap_is_smaller() {
        assert_eq!(regions_hit_by_one_read_modify_write(0, REGION_BYTES + 0, REGION_BYTES, 4096), 2);
        assert_eq!(regions_hit_by_one_read_modify_write(0, REGION_BYTES + 512, REGION_BYTES, 4096), 2);
        assert_eq!(regions_hit_by_one_read_modify_write(0, REGION_BYTES + 4095, REGION_BYTES, 4096), 2);
        assert_eq!(regions_hit_by_one_read_modify_write(0, REGION_BYTES + 4096, REGION_BYTES, 4096), 1); // 恰好等于就够
        // 三档 io_min 各自的门槛就是它自己
        for io_min in IO_MINS {
            assert_eq!(regions_hit_by_one_read_modify_write(0, REGION_BYTES + io_min - 1, REGION_BYTES, io_min), 2);
            assert_eq!(regions_hit_by_one_read_modify_write(0, REGION_BYTES + io_min, REGION_BYTES, io_min), 1);
        }
    }

    /// **盘首盘尾那个布局的间距远超任何 `io_min`** ⇒ 间距那一条它满足。
    /// ⇒ **候选布局倒在同盘那一条上，不是倒在间距上。**
    #[test]
    fn the_head_tail_layout_passes_the_spacing_test() {
        let tail_address = tail_position(CAPACITY, REGION_BYTES, 65536);
        for io_min in IO_MINS {
            assert_eq!(regions_hit_by_one_read_modify_write(0, tail_address, REGION_BYTES, io_min), 1);
        }
        assert!(tail_address - REGION_BYTES > 65536 * 1000);
    }

    /// **要几个区域**：容忍 F 个失败域要 F+1 个。候选的 2 个只够容忍 1 个。
    #[test]
    fn two_regions_tolerate_exactly_one_failure_domain() {
        assert_eq!(regions_needed(0), 1);
        assert_eq!(regions_needed(1), 2);
        assert_eq!(regions_needed(2), 3);
        assert_eq!(regions_needed(3), 4);
        assert!(2 >= regions_needed(1));
        assert!(2 < regions_needed(2), "候选的 2 个区域容忍不了 2 个失败域");
    }

    /// **阳性对照一**：把两区放进同一个 chunk ⇒ 同盘判定必须为真。
    /// 测不出这个差别说明模型没在按几何算。
    #[test]
    fn positive_control_same_chunk_is_always_the_same_device() {
        for device_count in DEVICE_COUNTS {
            assert!(same_device(0, 1024, 65536, device_count), "同一个 chunk 内必然同盘（{device_count} 盘）");
        }
        // 对照：跨一个 chunk 且设备数 > 1 ⇒ 不同盘
        for device_count in DEVICE_COUNTS.iter().filter(|&&candidate_device_count| candidate_device_count > 1) {
            assert!(!same_device(0, 65536, 65536, *device_count));
        }
    }

    /// **阳性对照二**：间距 0 ⇒ 一次 RMW 必然打到两个区域，三档 io_min 都要跑。
    #[test]
    fn positive_control_zero_gap_always_hits_both() {
        for io_min in IO_MINS {
            assert_eq!(regions_hit_by_one_read_modify_write(0, REGION_BYTES, REGION_BYTES, io_min), 2, "io_min {io_min} 档");
        }
    }
}

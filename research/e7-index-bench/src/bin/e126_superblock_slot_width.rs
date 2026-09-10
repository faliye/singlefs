//! # E126 超级块槽宽四条候选的代价
//!
//! 答 C232 挡着的那一格：超级块槽宽取哪一条候选。**只给代价，不给判决**——
//! 判决要走三方（.claude/rules/three-way-inference.md）。判据与失败条款见
//! `.claude/kb/experiments/126-超级块槽宽四条候选的代价.md`，**跑前写死**。
//!
//! ## 为什么这一格是空的（跑前现查，逐字）
//!
//! - `.claude/kb/decisions/18-块里携带什么信息.md` 第 775 行：
//!   `| 18 | 超级块槽 | 槽宽 | D22 未定项 9 | 自己的 | 占位，未定 | 不适用 | 不带 | 同上 |`
//!   ⇒ 第一个事务的字段表把它标成未定，且指回 D22 未定项 9 自己（循环）。
//! - D20 推论三第 69 行：「**自证单元**（根槽、journal 记录头） | **等于运行时探测到的
//!   `physical_block_size`，不许硬编码。**」⇒ **清单不含超级块槽。**
//! - `e115_superblock_completeness.rs` 第 33–34 行自陈：「⚠️ **它的自证单元清单逐字不含超级块槽**
//!   ——E100 把这两档套给超级块是类比延伸，不是条款覆盖。」
//! - C212 逐字：「超级块槽宽跟着根槽走」，而同一格又逐字「**三条候选互不相同，不许预设走哪一条**」。
//! - D22 已定项 8：「超级块每盘放一份……做成槽轮换，每盘至少 2 个槽。」**一个字没说多宽。**
//!
//! ## 被引用条款（逐字）
//!
//! - D2 已定硬要求 1：「**不发出小于 `io_min` 的写。** 在 Linux 上这个量是 `io_min`，
//!   不是 `physical_block_size`」
//! - D2 已定项 9（用户定案）：「第一个可运行目标跑 2 块盘」
//! - D18 已定项 9（用户定案）：「块 = 单元，一个单元一份头；扫描器按池内最小单元大小步进（第一版 16384）」
//! - `.claude/rules/fs-design.md`：「⚠️ **更强的一条：「比谁省」不构成任何判据。**」
//!   ⇒ 占用只报绝对字节，用来驳「这条很奢侈」，不做「谁更省」的判据。
//! - `.claude/rules/fs-design.md`：「**任何不做成单元的盘上空间，自动落在这些机制之外**」⇒ 候选丁的理由。
//! - E34（根环槽几何）：`io_min` = 65536 / `physical_block_size` = 512；
//!   并逐字「**暴露面 ≠ 损失面**」。
//! - 本机现查（2026-09-09）：`/sys/block/nvme0n1/queue/minimum_io_size` = 512、
//!   `physical_block_size` = 512。
//!
//! ## 它答不了的
//!
//! 不答「超级块该有哪些字段」（D22 未定项 9）——记录字节数是**输入**，按六格扫。
//! 不答 C212 整条冲突（还管根槽与 journal 记录两类）。
//! 不答真设备行为——`io_min` 三档是扫的参数。

use e7_index_bench::Emitter;

/// D18 已定项 9（用户定案）：池内最小单元大小，第一版 16384。
const MIN_UNIT_BYTES: u64 = 16384;
/// D2 已定项 9（用户定案）：第一个可运行目标跑 2 块盘。
const FIRST_VERSION_DISKS: u64 = 2;
/// 撕裂判定的扇区宽度。E115 数「跨 n 扇区 ⇒ 2ⁿ − 2 种可读但不一致的态」用的就是它。
const SECTOR: u64 = 512;
/// C231：树表单元指针那 59 字节的出处引的是 D22 已定项 7（**根记录**的字段表）。
const MISATTRIBUTED_PTR: u64 = 59;

/// 四条候选。**跑前写死，跑完不许改。**
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Arm {
    /// 甲：= physical_block_size（探测值，跟根槽走）
    ProbePbs,
    /// 乙：= max(physical_block_size, io_min)（探测值）
    ProbeMaxIoMin,
    /// 丙：= 一个格式常量（与探测解绑）
    FormatConst(u64),
    /// 丁：= 池内最小单元大小 ⇒ 超级块槽就是一个单元
    MinUnit,
}

impl Arm {
    fn name(self) -> String {
        match self {
            Arm::ProbePbs => "jia_probe_pbs".to_string(),
            Arm::ProbeMaxIoMin => "yi_probe_max_iomin".to_string(),
            Arm::FormatConst(n) => format!("bing_const_{n}"),
            Arm::MinUnit => "ding_min_unit".to_string(),
        }
    }
    /// 在一台 (pbs, io_min) 的设备上，这条候选取到的槽宽。
    fn width(self, pbs: u64, io_min: u64) -> u64 {
        match self {
            Arm::ProbePbs => pbs,
            Arm::ProbeMaxIoMin => pbs.max(io_min),
            Arm::FormatConst(n) => n,
            Arm::MinUnit => MIN_UNIT_BYTES,
        }
    }
    /// 是不是格式常量（不随设备变）。
    fn is_format_const(self) -> bool {
        matches!(self, Arm::FormatConst(_) | Arm::MinUnit)
    }
}

/// 判据 1：每盘占用与池占用。
fn footprint(width: u64, slots: u64, disks: u64) -> (u64, u64) {
    (width * slots, width * slots * disks)
}

/// 判据 2：装不装得下。
fn fits(width: u64, record_bytes: u64) -> bool {
    record_bytes <= width
}

/// 判据 3：D2 已定硬要求 1 合规——一次槽写是否 ≥ io_min。
fn honors_io_min(width: u64, io_min: u64) -> bool {
    width >= io_min
}

/// 判据 4：跨几个 512 扇区，以及「可读但不一致」的态数 2ⁿ − 2。
/// ⚠️ **暴露面不是损失面**（E34（根环槽几何） 逐字）：超级块无父、走已定三的自证校验和，
/// 撕裂由整槽校验和抓；这个数量的是「有多少种可读的中间态」，不是「会丢多少」。
/// 返回 `(跨扇区数 n, 2ⁿ − 2 的精确值)`；n > 63 时 u64 装不下，第二项返回 `None`
/// ——**报 n 本身就够**，它才是暴露面的绝对值。
/// ⚠️ 第一版把第二项写成 `(1u64 << n) - 2` 而单测只取了 512 / 4096 / 16384 三个点，
/// 三点上 n ≤ 32 全都不溢出；跑起来在候选乙的 65536（n = 128）上当场 panic。
/// 这正是 `.claude/rules/mutation-sampling.md` 说的**取样点不敏感**，不是等价变异。
fn tear_states(width: u64) -> (u32, Option<u64>) {
    let n = width.div_ceil(SECTOR) as u32;
    // n = 1 时 2¹ − 2 = 0：一个扇区内不存在「部分可读」的中间态。
    let states = if n <= 63 { Some((1u64 << n) - 2) } else { None };
    (n, states)
}

/// 判据 5：在一个异构池上，这条候选取到几个不同的槽宽值。
fn distinct_widths(arm: Arm, pool: &[(u64, u64)]) -> usize {
    let mut v: Vec<u64> = pool.iter().map(|&(p, m)| arm.width(p, m)).collect();
    v.sort_unstable();
    v.dedup();
    v.len()
}

fn main() {
    let mut em = Emitter::new();
    let mut out: Vec<String> = Vec::new();

    let arms = [
        Arm::ProbePbs,
        Arm::ProbeMaxIoMin,
        Arm::FormatConst(512),
        Arm::FormatConst(4096),
        Arm::MinUnit,
    ];
    // E115 / E124 的四格 + 合并臂两格。**两组一起报**：原值与「减 59」（C231 未修）。
    let records: [(&str, u64); 6] = [
        ("unmerged_w0_m0", 507),
        ("unmerged_w0_m1", 566),
        ("unmerged_w1_m0", 536),
        ("unmerged_w1_m1", 595),
        ("merged_w0", 389),
        ("merged_w1", 477),
    ];
    // 判据 3 的设备档：512 是本机 nvme0n1 现查值，65536 是 E34（根环槽几何） 在真 md/raid5 上量到的值。
    let devices: [(&str, u64, u64); 3] = [
        ("nvme_local", 512, 512),
        ("common_4k", 4096, 4096),
        ("md_raid5", 512, 65536),
    ];

    out.push(em.emit_raw(&format!(
        "name=config min_unit={MIN_UNIT_BYTES} disks={FIRST_VERSION_DISKS} sector={SECTOR} \
         arms={} records={} devices={}",
        arms.len(),
        records.len(),
        devices.len()
    )));

    // ── 判据 1 + 3 + 4：逐设备逐候选 ──
    for (dname, pbs, io_min) in devices {
        for arm in arms {
            let w = arm.width(pbs, io_min);
            let (n_sect, states) = tear_states(w);
            for slots in [2u64, 4, 8, 16] {
                let (per_disk, pool) = footprint(w, slots, FIRST_VERSION_DISKS);
                out.push(em.emit_raw(&format!(
                    "name=footprint dev={dname} arm={} pbs={pbs} io_min={io_min} width={w} \
                     slots={slots} per_disk_bytes={per_disk} pool_bytes={pool}",
                    arm.name()
                )));
            }
            out.push(em.emit_raw(&format!(
                "name=iomin dev={dname} arm={} width={w} io_min={io_min} honors={} \
                 sectors={n_sect} tear_states={}",
                arm.name(),
                u8::from(honors_io_min(w, io_min)),
                states.map_or_else(|| format!("2^{n_sect}-2"), |v| v.to_string())
            )));
        }
    }

    // ── 判据 2：装不装得下，两组（原值 / 减 59）──
    for (dname, pbs, io_min) in devices {
        for arm in arms {
            let w = arm.width(pbs, io_min);
            for (rname, rb) in records {
                let cut = rb - MISATTRIBUTED_PTR;
                out.push(em.emit_raw(&format!(
                    "name=fit dev={dname} arm={} width={w} record={rname} bytes={rb} \
                     fits={} bytes_minus59={cut} fits_minus59={}",
                    arm.name(),
                    u8::from(fits(w, rb)),
                    u8::from(fits(w, cut))
                )));
            }
        }
    }

    // ── 判据 5：异构池（pbs 512 与 4096 混合）上取到几个不同值 ──
    let hetero: [(u64, u64); 2] = [(512, 512), (4096, 4096)];
    for arm in arms {
        out.push(em.emit_raw(&format!(
            "name=hetero arm={} distinct_widths={} is_format_const={}",
            arm.name(),
            distinct_widths(arm, &hetero),
            u8::from(arm.is_format_const())
        )));
    }

    // 收尾行走 Emitter::finish()：`name=done emitted=N`，N 计入自身。
    // replay.sh 的完整性闸 2 认的就是这一行；第一版手写了 `name=count n=…`
    // 且没把自己算进去（171 vs 实际 172）⇒ 那样注册进 replay 会当场判红。
    out.push(em.finish());
    for l in &out {
        println!("{l}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **绝对值断言**：槽宽的绝对取值，由独立算术给出，不从代码读回来。
    #[test]
    fn arm_widths_absolute() {
        // 甲在本机盘上就是 512
        assert_eq!(Arm::ProbePbs.width(512, 512), 512);
        // 乙在 E34 真阵列（pbs 512 / io_min 65536）上被抬到 65536
        assert_eq!(Arm::ProbeMaxIoMin.width(512, 65536), 65536);
        // 丁恒等于池内最小单元大小
        assert_eq!(Arm::MinUnit.width(512, 65536), 16384);
        assert_eq!(MIN_UNIT_BYTES, 16384);
    }

    /// **阳性对照**（失败条款）：槽宽 512 那一档必须复现 E124 已入库的判决。
    #[test]
    fn positive_control_reproduces_e124_at_512() {
        assert!(fits(512, 507), "E124：507 装得下 512 槽");
        assert!(!fits(512, 566), "E124：566 爆");
        assert!(!fits(512, 536), "E124：536 爆");
        assert!(!fits(512, 595), "E124：595 爆");
        // 余量的绝对值：512 − 507 = 5（E124 逐字「余 5」）
        assert_eq!(512 - 507, 5);
    }

    /// **阴性对照**（失败条款）：16384 那一档六格必须全部装得下。
    #[test]
    fn negative_control_all_fit_at_min_unit() {
        for b in [507u64, 566, 536, 595, 389, 477] {
            assert!(fits(MIN_UNIT_BYTES, b), "16384 槽装不下 {b}，口径读错了");
        }
    }

    /// 占用的绝对值：独立算术。
    #[test]
    fn footprint_absolute() {
        // 甲 512 × 2 槽 × 2 盘 = 2048
        assert_eq!(footprint(512, 2, 2), (1024, 2048));
        // 丁 16384 × 2 槽 × 2 盘 = 65536
        assert_eq!(footprint(16384, 2, 2), (32768, 65536));
        // 丁最大档 16384 × 16 槽 × 2 盘 = 524288
        assert_eq!(footprint(16384, 16, 2), (262144, 524288));
        // 乙在真阵列上 65536 × 16 × 2 = 2097152
        assert_eq!(footprint(65536, 16, 2), (1048576, 2097152));
    }

    /// D2 已定硬要求 1 合规的绝对判定。
    #[test]
    fn io_min_compliance() {
        // 甲在 E34 真阵列上违规：512 < 65536
        assert!(!honors_io_min(512, 65536));
        // 乙恒合规
        assert!(honors_io_min(65536, 65536));
        // 丁在真阵列上仍然违规：16384 < 65536
        assert!(!honors_io_min(16384, 65536));
        // 丁在 4K 盘上合规
        assert!(honors_io_min(16384, 4096));
    }

    /// 撕裂暴露面：跨扇区数与态数的绝对值。
    #[test]
    fn tear_states_absolute() {
        assert_eq!(tear_states(512), (1, Some(0)));
        assert_eq!(tear_states(4096), (8, Some(254)));
        assert_eq!(tear_states(16384), (32, Some((1u64 << 32) - 2)));
        // 独立算术：4096 / 512 = 8，2⁸ − 2 = 254
        assert_eq!((1u64 << 8) - 2, 254);
        // ⚠️ **候选乙在 E34 真阵列上的取样点**：65536 / 512 = 128 ⇒ u64 装不下，必须给 None。
        // 第一版漏的就是这一格，跑起来才 panic。
        assert_eq!(tear_states(65536), (128, None));
        assert_eq!(65536u64 / 512, 128);
    }

    /// 三条变异跑出来没被抓，按 `.claude/rules/mutation-sampling.md` 三分之后补的取样点。
    /// 三条都**不是**盲区也**不是**等价变异，逐条判在下面。
    #[test]
    fn mutation_triage_sampling_points() {
        // ① `first_version_disks`（2 → 3）没被抓：**真盲区**——这个常量只在 main() 用，
        //    而 footprint_absolute 传的是字面量 2，从没断言过常量自己。补一条钉死它。
        assert_eq!(FIRST_VERSION_DISKS, 2, "D2 已定项 9 用户定案：第一版跑 2 块盘");

        // ② `fits_strict`（`<=` → `<`）没被抓：**取样点不敏感**——六个记录字节数没有一个
        //    恰好等于某档槽宽，边界从没被取到。补一个恰好相等的点。
        assert!(fits(512, 512), "记录恰好占满整槽必须算装得下");
        assert!(fits(16384, 16384));
        assert!(!fits(512, 513));

        // ③ `tear_sector_floor`（div_ceil → /）没被抓：**取样点不敏感**——四条候选产出的槽宽
        //    512 / 4096 / 16384 / 65536 全是 512 的整数倍，向上向下取整同值。
        //    补一个不是整数倍的宽度：600 字节跨 2 个扇区，向下取整会给 1。
        assert_eq!(tear_states(600).0, 2, "600 字节跨 2 个 512 扇区");
        assert_eq!(tear_states(513).0, 2);
        assert_eq!(tear_states(1024).0, 2);
    }

    /// 异构池上取到几个值：格式常量恒 1。
    #[test]
    fn hetero_distinct() {
        let pool = [(512u64, 512u64), (4096, 4096)];
        assert_eq!(distinct_widths(Arm::ProbePbs, &pool), 2);
        assert_eq!(distinct_widths(Arm::ProbeMaxIoMin, &pool), 2);
        assert_eq!(distinct_widths(Arm::MinUnit, &pool), 1);
        assert_eq!(distinct_widths(Arm::FormatConst(4096), &pool), 1);
    }

    /// C231 的「减 59」那一组：507 − 59 = 448，装得下 512。
    #[test]
    fn minus_59_group() {
        assert_eq!(507 - MISATTRIBUTED_PTR, 448);
        assert_eq!(536 - MISATTRIBUTED_PTR, 477);
        assert!(fits(512, 448));
        assert!(fits(512, 477));
        // 而 566 / 595 减 59 之后仍然爆
        assert_eq!(566 - MISATTRIBUTED_PTR, 507);
        assert!(fits(512, 507));
        assert_eq!(595 - MISATTRIBUTED_PTR, 536);
        assert!(!fits(512, 536));
    }
}

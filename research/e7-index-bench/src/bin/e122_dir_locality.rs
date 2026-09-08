//! # E122 按目录聚在一起买到什么
//!
//! **问的是** D8 已定项 3 那张表里「分配器（把子目录的数据放到父目录附近）」一行的收益。
//! 那一行的唯一依据逐字标着「⚠️ **未核实**——kb 内无出处无口径，要用先补」，
//! 而 D14 未定项 2（分配器提示的判据）2026-09-07 一轮三方论证之后，
//! 用户定「**先补收益数据再定**」。这一格补的就是它。
//!
//! ## 模型建在哪几条已定条款上（逐字）
//!
//! - **D14 已定项 3 + D4 已定项 2**：内联阈值 0，小文件一律走独立数据 extent、
//!   补齐到 32 KiB ⇒ **每个小文件各占一个整单元、永不共享**。
//!   ⇒ 聚集只能改变单元**之间**的相邻性，改不了单元内部。
//! - **D3 已定项 7**：落点粒度 16384 ⇒ 一个 32 KiB 数据单元占 **2 个槽**。
//! - **D3 已定项 5**：聚簇段是**运行时分配器政策不是格式位**，
//!   「聚簇段的位置与长度是运行时选择，盘上没有任何字段声明它」
//!   ⇒ 段大小在这里是**参数**，不是格式常量。
//!
//! ## 三条臂
//!
//! | 臂 | 落点怎么定 |
//! |---|---|
//! | `grouped` | 同一父目录下的对象在盘上连续 |
//! | `arrival` | 按创建到达顺序落点——目录之间**交错**的程度由参数 `stride` 定 |
//! | `random` | 伪随机落点。**判别力下界，不是候选** |
//!
//! ⚠️ **`grouped` 在遍历那一格按构造必胜**，所以那一格的数**不是结论**，
//! 结论在「`arrival` 差多少」以及「交错度多大时差距才出现」。
//! `stride = 1` 表示写完一个目录再写下一个（此时 `arrival` 应当**等于** `grouped`）；
//! `stride = F` 表示完全轮转。
//!
//! ## 跑前写死的判据与失败条款
//!
//! 1. **遍历代价**：读完一个目录的全部文件要发多少次连续读（把该目录的槽排序后数连续段）。
//!    逐格钉绝对值。`grouped` 必须恒为 1，不是 1 说明模型建错了。
//! 2. **回收代价**：删掉一个目录的全部文件之后，全空的段有多少个（段长 `SEG` 槽）。逐格钉绝对值。
//! 3. **阳性对照逐臂跑**：**没有交错**时 `arrival` 必须逐格等于 `grouped`——
//!    写完一个目录再写下一个，两者本来就是同一个落点序列。任一格不等 ⇒ 整轮作废。
//!    ⚠️ **跑前登记把参数标反了，留档不改**：登记写的是「`stride = 1` 时」，
//!    而 `stride` 的语义是「一个目录连着放几个再换下一个」⇒ `stride = 1` 是**完全轮转**、
//!    交错最厉害那一格，`stride = files` 才是没有交错。判据的**意思**没变，
//!    错的是主 agent 登记时给参数贴的标签。实测：`stride = 32 = files` 时两条臂都给 16。
//! 4. **阴性对照**：每目录 0 个文件 ⇒ 三条臂的连续段数都是 0。
//! 5. **判别力**：三条臂在至少一格上给出不同的数。
//! 6. **失败条款**：若 `grouped` 与 `arrival` 在**两个**指标上都逐格相同，
//!    如实记「按目录聚在本工程的模型下买不到东西」，**不许改模型**。
//!
//! ## 它答不了的
//!
//! 纯计数模型：没有真实 I/O、没有设备寻道时间、没有页缓存、没有崩溃点重放。
//! 「连续段数」是**代理指标**不是挂钟——它假设一次连续读比两次便宜，而便宜多少这里不答。
//! 不答「疑似短命」那一半：那一格在 D25 全文出现 0 次，是空白，不是这个实验能补的。
//! 确定性模型，**跑 N 轮说明的是没有隐藏状态，不是统计上稳定**。

/// D3 已定项 7：落点粒度 16384 字节。
const GRAIN: u64 = 16384;
/// D4 已定项 5：数据单元恒 32768 含头 ⇒ 占 2 个槽。
const SLOTS_PER_UNIT: u64 = 32768 / GRAIN;
/// 段长（槽）。聚簇段在仓里没有固定大小（D3 已定项 5：运行时政策、零格式位）⇒ **参数，要扫**。
/// ⚠️ 第一版只取了 128 一个值，而一个目录（32 个文件 × 2 槽）正好占 64 槽
/// ⇒ 三条臂的全空段数**全是 0**，那一格分不开任何东西。
/// 按 `.claude/rules/mutation-sampling.md`，这是取样点不敏感，处置是**补取样点**不是留档。
const SEGS: [u64; 3] = [32, 64, 128];
/// 负载规模。**做成常量而不是 `main` 里的局部变量**：变异测试发现
/// 「把 `main` 里的 `dirs` 改掉」一个测试都没红——单测各自硬写 16 / 32，
/// 没有任何东西钉住产物实际用的那组参数（真盲区，不是取样点不敏感）。
const DIRS: u64 = 16;
const FILES: u64 = 32;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Arm {
    Grouped,
    Arrival,
    Random,
}

impl Arm {
    fn name(self) -> &'static str {
        match self {
            Arm::Grouped => "grouped",
            Arm::Arrival => "arrival",
            Arm::Random => "random",
        }
    }
}

/// 伪随机：确定性 LCG，**不引入外部随机源**，复跑逐字节一致。
fn lcg(x: u64) -> u64 {
    x.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1_442_695_040_888_963_407)
}

/// 返回每个目录的槽号列表（已排序）。
/// `dirs` 个目录、每目录 `files` 个文件、交错步长 `stride`
/// （每次从一个目录连着放 `stride` 个文件再换下一个目录）。
fn place(arm: Arm, dirs: u64, files: u64, stride: u64) -> Vec<Vec<u64>> {
    let mut out: Vec<Vec<u64>> = vec![Vec::new(); dirs as usize];
    let total = dirs * files;
    match arm {
        Arm::Grouped => {
            // 同目录连续
            for d in 0..dirs {
                for f in 0..files {
                    let unit = d * files + f;
                    out[d as usize].push(unit * SLOTS_PER_UNIT);
                }
            }
        }
        Arm::Arrival => {
            // 到达顺序：每个目录连着放 stride 个，再换下一个目录
            let s = stride.max(1);
            let mut placed = vec![0u64; dirs as usize];
            let mut unit = 0u64;
            'outer: loop {
                for d in 0..dirs {
                    for _ in 0..s {
                        if placed[d as usize] >= files {
                            break;
                        }
                        out[d as usize].push(unit * SLOTS_PER_UNIT);
                        placed[d as usize] += 1;
                        unit += 1;
                        if unit >= total {
                            break 'outer;
                        }
                    }
                }
                if placed.iter().all(|&p| p >= files) {
                    break;
                }
            }
        }
        Arm::Random => {
            let mut x = 0x5eed_1234u64;
            let mut used = std::collections::BTreeSet::new();
            for d in 0..dirs {
                for _ in 0..files {
                    let mut slot;
                    loop {
                        x = lcg(x);
                        slot = (x % total) * SLOTS_PER_UNIT;
                        if used.insert(slot) {
                            break;
                        }
                    }
                    out[d as usize].push(slot);
                }
            }
        }
    }
    for v in out.iter_mut() {
        v.sort_unstable();
    }
    out
}

/// 一个目录的槽排序之后有多少个连续段。每个单元占 `SLOTS_PER_UNIT` 个槽。
fn runs(slots: &[u64]) -> u64 {
    if slots.is_empty() {
        return 0;
    }
    let mut n = 1;
    for w in slots.windows(2) {
        if w[1] != w[0] + SLOTS_PER_UNIT {
            n += 1;
        }
    }
    n
}

/// 删掉第 `victim` 个目录之后，全空的段有多少个。
fn empty_segs(all: &[Vec<u64>], victim: usize, seg: u64) -> u64 {
    let mut live = std::collections::BTreeSet::new();
    for (i, v) in all.iter().enumerate() {
        if i == victim {
            continue;
        }
        for &s in v {
            for k in 0..SLOTS_PER_UNIT {
                live.insert(s + k);
            }
        }
    }
    let max_slot = all
        .iter()
        .flatten()
        .copied()
        .max()
        .map(|m| m + SLOTS_PER_UNIT)
        .unwrap_or(0);
    let segs = max_slot.div_ceil(seg);
    (0..segs)
        .filter(|&g| (g * seg..(g + 1) * seg).all(|s| !live.contains(&s)))
        .count() as u64
}

fn main() {
    let mut em = e7_index_bench::Emitter::new();
    let dirs = DIRS;
    let files = FILES;
    println!(
        "{}",
        em.emit_raw(&format!(
            "name=config grain={GRAIN} slots_per_unit={SLOTS_PER_UNIT} segs={SEGS:?} dirs={dirs} files={files}"
        ))
    );
    for &stride in [1u64, 2, 4, 8, 32].iter() {
        for arm in [Arm::Grouped, Arm::Arrival, Arm::Random] {
            let all = place(arm, dirs, files, stride);
            let total_runs: u64 = all.iter().map(|v| runs(v)).sum();
            let worst = all.iter().map(|v| runs(v)).max().unwrap_or(0);
            let es: Vec<String> = SEGS
                .iter()
                .map(|&sg| format!("empty_segs_at_{sg}={}", empty_segs(&all, 0, sg)))
                .collect();
            println!(
                "{}",
                em.emit_raw(&format!(
                    "name=walk arm={} stride={stride} total_runs={total_runs} worst_dir_runs={worst} {}",
                    arm.name(),
                    es.join(" ")
                ))
            );
        }
    }
    // 阴性对照：每目录 0 个文件
    for arm in [Arm::Grouped, Arm::Arrival, Arm::Random] {
        let all = place(arm, dirs, 0, 1);
        println!(
            "{}",
            em.emit_raw(&format!(
                "name=negcontrol arm={} total_runs={}",
                arm.name(),
                all.iter().map(|v| runs(v)).sum::<u64>()
            ))
        );
    }
    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 判据 1：`grouped` 的每目录连续段数恒为 1。不是 1 说明模型建错了。
    #[test]
    fn t01_grouped_is_one_run_per_dir() {
        for &stride in [1u64, 2, 8, 32].iter() {
            let all = place(Arm::Grouped, 16, 32, stride);
            for (d, v) in all.iter().enumerate() {
                assert_eq!(runs(v), 1, "dir {d} stride {stride}");
            }
        }
    }

    /// 判据 3：**阳性对照逐臂跑**——没有交错时 `arrival` 必须逐格等于 `grouped`。
    /// ⚠️ 跑前登记把这一格标成 `stride = 1`，而那是完全轮转那一格；
    /// 没有交错的是 `stride = files`。登记那个标签留在这里，不改成对的。
    #[test]
    fn t02_positive_control_每条臂() {
        const PREREG_STRIDE: u64 = 1; // 登记时写的，标反了
        const NO_INTERLEAVE_STRIDE: u64 = 32; // files，真正没有交错的那一格
        assert_ne!(PREREG_STRIDE, NO_INTERLEAVE_STRIDE);
        let g = place(Arm::Grouped, 16, 32, NO_INTERLEAVE_STRIDE);
        let a = place(Arm::Arrival, 16, 32, NO_INTERLEAVE_STRIDE);
        assert_eq!(g, a, "写完一个目录再写下一个，两条臂本来就是同一个落点序列");
        let r = place(Arm::Random, 16, 32, NO_INTERLEAVE_STRIDE);
        assert_eq!(r.len(), g.len());
        assert_eq!(r.iter().map(|v| v.len()).sum::<usize>(), 16 * 32);
    }

    /// 判据 4：阴性对照——每目录 0 个文件，三条臂的连续段数都是 0。
    #[test]
    fn t03_negative_control_每条臂() {
        for arm in [Arm::Grouped, Arm::Arrival, Arm::Random] {
            let all = place(arm, 16, 0, 1);
            assert_eq!(all.iter().map(|v| runs(v)).sum::<u64>(), 0, "{}", arm.name());
        }
    }

    /// 判据 1 的绝对值：完全轮转时 `arrival` 每目录 32 段（每个文件各成一段）。
    #[test]
    fn t04_walk_runs_are_absolute() {
        // grouped 与 stride 无关，恒 16（16 个目录各 1 段）
        for &st in [1u64, 2, 4, 8, 32].iter() {
            let g: u64 = place(Arm::Grouped, 16, 32, st).iter().map(|v| runs(v)).sum();
            assert_eq!(g, 16, "stride {st}");
        }
        // arrival 的闭式：总段数 = 目录数 × 文件数 / stride = 512 / stride
        for &(st, want) in [(1u64, 512u64), (2, 256), (4, 128), (8, 64), (32, 16)].iter() {
            let a: u64 = place(Arm::Arrival, 16, 32, st).iter().map(|v| runs(v)).sum();
            assert_eq!(a, want, "stride {st}");
            assert_eq!(a * st, 512, "闭式：段数 × stride 恒等于对象总数");
        }
    }

    /// 判据 2 的绝对值：删掉一个目录之后的全空段数。
    #[test]
    fn t05_empty_segs_are_absolute() {
        let g = place(Arm::Grouped, 16, 32, 2);
        let a = place(Arm::Arrival, 16, 32, 2);
        // 一个目录 = 32 个文件 × 2 槽 = 64 槽
        assert_eq!(empty_segs(&g, 0, 32), 2, "段长 32：被删目录正好腾出两个整段");
        assert_eq!(empty_segs(&g, 0, 64), 1, "段长 64：正好一个整段");
        assert_eq!(empty_segs(&g, 0, 128), 0, "段长 128：只腾出半个段 ⇒ 0");
        for &sg in SEGS.iter() {
            assert_eq!(empty_segs(&a, 0, sg), 0, "arrival 交错之后一个整段都空不出来（段长 {sg}）");
        }
    }

    /// 判据 5：判别力——三条臂在 stride = 2 那格必须给出不止一种结果。
    #[test]
    fn t06_discriminating() {
        let mut set = std::collections::BTreeSet::new();
        for arm in [Arm::Grouped, Arm::Arrival, Arm::Random] {
            let all = place(arm, 16, 32, 2);
            set.insert(all.iter().map(|v| runs(v)).sum::<u64>());
        }
        assert!(set.len() >= 2, "三条臂全同 ⇒ 装置分不开它们");
    }

    /// 几何常量钉死。
    #[test]
    fn t07_constants() {
        assert_eq!(GRAIN, 16384);
        assert_eq!(SLOTS_PER_UNIT, 2);
        assert_eq!(SEGS, [32, 64, 128]);
        assert_eq!(DIRS, 16);
        assert_eq!(FILES, 32);
        assert_eq!(DIRS * FILES, 512, "产物里的对象总数");
    }

    /// 落点不重叠：三条臂都不许把两个单元放到同一个槽。
    #[test]
    fn t08_no_overlap() {
        for arm in [Arm::Grouped, Arm::Arrival, Arm::Random] {
            let all = place(arm, 16, 32, 4);
            let mut seen = std::collections::BTreeSet::new();
            for v in &all {
                for &s in v {
                    assert!(seen.insert(s), "{} 落点撞了 {}", arm.name(), s);
                }
            }
            assert_eq!(seen.len(), 16 * 32);
        }
    }

    /// **给 `random` 去重那一步的敏感取样点**。
    /// ⚠️ 这条是变异测试逼出来的：`M10_random不去重`（把去重条件短路成恒真）
    /// 在 16 × 32 = 512 个对象上**一个测试都没红**——LCG 的模是 2⁶⁴，
    /// 而 512 是 2 的幂 ⇒ `x % 512` 恰好走满周期、永不碰撞，去重那行成了死逻辑。
    /// 按 `.claude/rules/mutation-sampling.md` 三分，这是**取样点不敏感**不是等价变异：
    /// 换成非 2 的幂（15 × 32 = 480）当场碰撞 172 次。
    #[test]
    fn t10_random_dedup_needs_a_non_power_of_two_size() {
        let all = place(Arm::Random, 15, 32, 4);
        let mut seen = std::collections::BTreeSet::new();
        for v in &all {
            for &s in v {
                assert!(seen.insert(s), "非 2 的幂规模下落点撞了 {s}");
            }
        }
        assert_eq!(seen.len(), 15 * 32);
        assert_ne!((15u64 * 32).count_ones(), 1, "取样点必须不是 2 的幂，否则这条测不出东西");
    }

    /// `random` 是判别力下界，不是候选：它的连续段数必须**不优于** `arrival`。
    #[test]
    fn t09_random_is_the_floor() {
        let a: u64 = place(Arm::Arrival, 16, 32, 2).iter().map(|v| runs(v)).sum();
        let r: u64 = place(Arm::Random, 16, 32, 2).iter().map(|v| runs(v)).sum();
        assert!(r >= a, "random={r} arrival={a}");
    }
}

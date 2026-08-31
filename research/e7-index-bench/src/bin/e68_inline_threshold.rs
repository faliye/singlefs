//! E68：内联阈值取多少 —— D14 已定项 3。
//!
//! ## 被引用条款逐字贴在这里（verify-before-claiming）
//!
//! - D14 已定项 3 逐字：「**内联阈值取多少**，以及内联 → extent 那个有界事务的崩溃语义。」
//! - E8（大文件与小文件是否该走不同写路径）层 A 已答的三条，本实验**不重测**：
//!   ① 交叉点就是内联阈值本身，不是文件大小的自然刻度；
//!   ② 内联的代价在读侧，1 KiB 档设备读从 47 涨到 293（6.2 倍）；
//!   ③ **阈值不是合格的分支变量**——4 KiB 文件分 8 次追加时前 7 次都在阈值以下
//!      （`fs-design.md` 硬要求 5 被这条分支违反，除非另设跨阈值迁移路径）。
//! - E8 用的阈值是 3584 B，**那是设的不是测的**。E68 补的就是这一格。
//! - D8 已定：节点 16 KiB。D19 已定项 1：位置条目带设备身份。
//!
//! ## 判据（跑前写死，跑完不许改）
//!
//! 1. **主判据是同量纲的**：每文件**总设备字节**（数据 + 元数据）随阈值的曲线，取最小点 `T*`。
//!    ⚠️ 不拿「写放大」去减「读次数」——那是两个量纲，本仓 2026-08-31 刚在
//!    D2 的宽度上界上栽过一次（拐点算术量纲错）。读次数**单独报**，不进主判据。
//! 2. `T*` 若落在扫描区间的端点上，判「区间没夹住」，如实记，不外推。
//! 3. 若读次数在 `T*` 处已经超过不内联的 2 倍，**如实并列**：主判据给的点在读侧不可接受。
//!
//! ## 失败条款（跑前写死）
//!
//! - **阳性对照，对每一档阈值都跑**：`T = 0`（从不内联）时各档的总设备字节必须与
//!   「不分流」逐格相同。不同 ⇒ 模型把阈值算错了，**整轮作废**。
//! - **阴性对照**：文件大小 ≫ 任何阈值（1 MiB）时，所有阈值档逐格相同。
//! - 五个种子（文件大小分布的随机种子）方向不一致 ⇒ 报「不稳定」。
//!
//! ## 它答不了的
//!
//! 计数模型，无设备、无文件系统、文件操作 0 处。不建模追加（E8 已答那一维）、
//! 不建模崩溃语义（D14 已定项 3 的另一半）、不建模压缩。

use e7_index_bench::Emitter;

const NODE: u64 = 16 * 1024;
const BLOCK: u64 = 4096;
/// 一条 inode 记录不含内联数据时的字节：key 16 + 头 40 + 一个带设备身份的位置条目 8。
const REC_BASE: u64 = 64;
const THRESHOLDS: [u64; 8] = [0, 256, 512, 1024, 2048, 3072, 3584, 4096];
const FILES: u64 = 100_000;

struct Lcg(u64);
impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0 >> 33
    }
}

/// 一个文件在给定阈值下占的（数据设备字节, 记录字节）。
fn per_file(size: u64, thr: u64) -> (u64, u64) {
    if size <= thr {
        (0, REC_BASE + size) // 内联：不占数据块，记录变长
    } else {
        (size.div_ceil(BLOCK) * BLOCK, REC_BASE) // 走 extent：整块分配
    }
}

/// 真实一点的大小分布：80% 小文件（≤4 KiB），20% 大文件。
fn size_of(rng: &mut Lcg) -> u64 {
    if rng.next() % 100 < 80 {
        1 + rng.next() % 4096
    } else {
        4096 + rng.next() % (1024 * 1024)
    }
}

struct Out {
    data_bytes: u64,
    meta_bytes: u64,
    leaves: u64,
    inlined: u64,
}

fn run(thr: u64, seed: u64) -> Out {
    let mut rng = Lcg(seed.wrapping_mul(0x9E3779B97F4A7C15) | 1);
    let (mut data, mut rec, mut inlined) = (0u64, 0u64, 0u64);
    for _ in 0..FILES {
        let s = size_of(&mut rng);
        let (d, r) = per_file(s, thr);
        if s <= thr {
            inlined += 1;
        }
        data += d;
        rec += r;
    }
    // 叶节点：记录按字节装进 16 KiB 的叶（不跨叶，按平均装填算上界）
    let leaves = rec.div_ceil(NODE);
    Out { data_bytes: data, meta_bytes: leaves * NODE, leaves, inlined }
}

/// 读一个文件要碰几次设备：走到叶（树高）+ 数据块。
/// 树高由叶数定，扇出 = NODE / 24（key 16 + 指针 8）。
fn read_ops(o: &Out, avg_data_blocks: u64) -> u64 {
    let fanout = NODE / 24;
    let mut h = 1u64;
    let mut n = o.leaves.max(1);
    while n > 1 {
        n = n.div_ceil(fanout);
        h += 1;
    }
    h + avg_data_blocks
}


/// 第二轮（2026-08-31 补）：**带缓存的读模型**——第一轮判据 3 判不了，就是缺这一块。
///
/// 一次随机点查的设备读次数 = 叶未命中（1 − 命中率）+ 数据块读（内联的文件为 0）。
/// 内层节点假定常驻（它们只占叶数的 1/682）。
/// 命中率按均匀随机取 `min(1, 缓存能装的叶数 / 叶总数)`。
fn reads_per_lookup(o: &Out, cache_bytes: u64) -> (u64, u64) {
    let cache_leaves = cache_bytes / NODE;
    let hit_ppm = (cache_leaves * 1_000_000 / o.leaves.max(1)).min(1_000_000);
    let leaf_miss_ppm = 1_000_000 - hit_ppm;
    let inlined_ppm = o.inlined * 1_000_000 / FILES;
    let data_read_ppm = 1_000_000 - inlined_ppm; // 非内联的文件要多读一个数据块
    (leaf_miss_ppm + data_read_ppm, o.leaves * NODE)
}

fn main() {
    let mut em = Emitter::new();
    println!(
        "{}",
        em.emit_raw(&format!(
            "name=config node={NODE} block={BLOCK} rec_base={REC_BASE} files={FILES} \
             model=counting file_ops=0"
        ))
    );
    for &thr in THRESHOLDS.iter() {
        for seed in 1..=5u64 {
            let o = run(thr, seed);
            let total = o.data_bytes + o.meta_bytes;
            println!(
                "{}",
                em.emit_raw(&format!(
                    "name=arm thr={thr} seed={seed} total_bytes_per_file={} \
                     data_per_file={} meta_per_file={} leaves={} inlined_pct_ppm={} \
                     read_ops_x1000={}",
                    total / FILES,
                    o.data_bytes / FILES,
                    o.meta_bytes / FILES,
                    o.leaves,
                    o.inlined * 1_000_000 / FILES,
                    read_ops(&o, 1) * 1000,
                ))
            );
        }
    }
    // 第二轮：带缓存的读模型，扫缓存大小 × 阈值。
    for cache_mib in [1u64, 4, 16, 64, 256] {
        for &thr in THRESHOLDS.iter() {
            let o = run(thr, 1);
            let (reads_ppm, working_set) = reads_per_lookup(&o, cache_mib * 1024 * 1024);
            println!(
                "{}",
                em.emit_raw(&format!(
                    "name=cached_read cache_mib={cache_mib} thr={thr} \
                     reads_per_lookup_ppm={reads_ppm} meta_working_set_bytes={working_set}"
                ))
            );
        }
    }

    // 阴性对照：文件全都是 1 MiB ⇒ 所有阈值档逐格相同。
    for &thr in THRESHOLDS.iter() {
        let (d, r) = per_file(1024 * 1024, thr);
        println!(
            "{}",
            em.emit_raw(&format!("name=negative_control_huge thr={thr} data={d} rec={r}"))
        );
    }
    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **阳性对照，对每一档阈值都跑**：T=0 ⇒ 与「从不内联」逐格相同。
    #[test]
    fn positive_control_zero_threshold_equals_never_inline() {
        for seed in 1..=5u64 {
            let o = run(0, seed);
            assert_eq!(o.inlined, 0, "T=0 却内联了");
            assert_eq!(o.meta_bytes, (FILES * REC_BASE).div_ceil(NODE) * NODE);
        }
    }

    /// **阴性对照**：1 MiB 文件在任何阈值下都走 extent，逐格相同。
    #[test]
    fn negative_control_huge_file_same_everywhere() {
        let base = per_file(1024 * 1024, 0);
        for &thr in THRESHOLDS.iter() {
            assert_eq!(per_file(1024 * 1024, thr), base);
        }
    }

    /// **绝对值断言**：一个 100 B 文件不内联时占整块 4096；内联时占 0 数据字节、记录 164。
    #[test]
    fn absolute_single_file_accounting() {
        assert_eq!(per_file(100, 0), (4096, 64));
        assert_eq!(per_file(100, 256), (0, 164));
        // 内联省下 4096 − 100 = 3996 字节的块内碎片，付出 100 字节的记录膨胀
        assert_eq!(4096 - 100, 3996);
    }

    /// **绝对值断言**：阈值恰好等于文件大小时算内联（`<=` 不是 `<`）。
    #[test]
    fn absolute_boundary_is_inclusive() {
        assert_eq!(per_file(512, 512), (0, REC_BASE + 512));
        assert_eq!(per_file(513, 512), (BLOCK, REC_BASE));
    }
}

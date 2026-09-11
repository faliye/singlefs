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

const NODE_SIZE_BYTES: u64 = 16 * 1024;
const BLOCK_BYTES: u64 = 4096;
/// 一条 inode 记录不含内联数据时的字节：key 16 + 头 40 + 一个带设备身份的位置条目 8。
const RECORD_BASE_BYTES: u64 = 64;
const THRESHOLDS: [u64; 8] = [0, 256, 512, 1024, 2048, 3072, 3584, 4096];
const FILE_COUNT: u64 = 100_000;

struct LinearCongruentialGenerator(u64);
impl LinearCongruentialGenerator {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0 >> 33
    }
}

/// 一个文件在给定阈值下占的（数据设备字节, 记录字节）。
fn data_and_record_bytes_for_file(file_size_bytes: u64, inline_threshold_bytes: u64) -> (u64, u64) {
    if file_size_bytes <= inline_threshold_bytes {
        (0, RECORD_BASE_BYTES + file_size_bytes) // 内联：不占数据块，记录变长
    } else {
        (file_size_bytes.div_ceil(BLOCK_BYTES) * BLOCK_BYTES, RECORD_BASE_BYTES) // 走 extent：整块分配
    }
}

/// 真实一点的大小分布：80% 小文件（≤4 KiB），20% 大文件。
fn draw_file_size_bytes(random_number_generator: &mut LinearCongruentialGenerator) -> u64 {
    if random_number_generator.next() % 100 < 80 {
        1 + random_number_generator.next() % 4096
    } else {
        4096 + random_number_generator.next() % (1024 * 1024)
    }
}

struct ThresholdRunTotals {
    data_bytes: u64,
    meta_bytes: u64,
    leaves: u64,
    inlined_file_count: u64,
}

fn simulate_files_at_threshold(inline_threshold_bytes: u64, seed: u64) -> ThresholdRunTotals {
    let mut random_number_generator = LinearCongruentialGenerator(seed.wrapping_mul(0x9E3779B97F4A7C15) | 1);
    let (mut data_bytes_total, mut record_bytes_total, mut inlined_file_count) = (0u64, 0u64, 0u64);
    for _ in 0..FILE_COUNT {
        let sampled_file_size_bytes = draw_file_size_bytes(&mut random_number_generator);
        let (data_bytes_for_file, record_bytes_for_file) = data_and_record_bytes_for_file(sampled_file_size_bytes, inline_threshold_bytes);
        if sampled_file_size_bytes <= inline_threshold_bytes {
            inlined_file_count += 1;
        }
        data_bytes_total += data_bytes_for_file;
        record_bytes_total += record_bytes_for_file;
    }
    // 叶节点：记录按字节装进 16 KiB 的叶（不跨叶，按平均装填算上界）
    let leaves = record_bytes_total.div_ceil(NODE_SIZE_BYTES);
    ThresholdRunTotals { data_bytes: data_bytes_total, meta_bytes: leaves * NODE_SIZE_BYTES, leaves, inlined_file_count }
}

/// 读一个文件要碰几次设备：走到叶（树高）+ 数据块。
/// 树高由叶数定，扇出 = NODE_SIZE_BYTES / 24（key 16 + 指针 8）。
fn device_reads_per_file_read(totals: &ThresholdRunTotals, average_data_blocks_per_file: u64) -> u64 {
    let fanout = NODE_SIZE_BYTES / 24;
    let mut tree_height = 1u64;
    let mut nodes_at_level = totals.leaves.max(1);
    while nodes_at_level > 1 {
        nodes_at_level = nodes_at_level.div_ceil(fanout);
        tree_height += 1;
    }
    tree_height + average_data_blocks_per_file
}


/// 第二轮（2026-08-31 补）：**带缓存的读模型**——第一轮判据 3 判不了，就是缺这一块。
///
/// 一次随机点查的设备读次数 = 叶未命中（1 − 命中率）+ 数据块读（内联的文件为 0）。
/// 内层节点假定常驻（它们只占叶数的 1/682）。
/// 命中率按均匀随机取 `min(1, 缓存能装的叶数 / 叶总数)`。
fn reads_per_lookup(totals: &ThresholdRunTotals, cache_bytes: u64) -> (u64, u64) {
    let cache_leaves = cache_bytes / NODE_SIZE_BYTES;
    let hit_rate_parts_per_million = (cache_leaves * 1_000_000 / totals.leaves.max(1)).min(1_000_000);
    let leaf_miss_parts_per_million = 1_000_000 - hit_rate_parts_per_million;
    let inlined_parts_per_million = totals.inlined_file_count * 1_000_000 / FILE_COUNT;
    let data_read_parts_per_million = 1_000_000 - inlined_parts_per_million; // 非内联的文件要多读一个数据块
    (leaf_miss_parts_per_million + data_read_parts_per_million, totals.leaves * NODE_SIZE_BYTES)
}

fn main() {
    let mut emitter = Emitter::new();
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=config node={NODE_SIZE_BYTES} block={BLOCK_BYTES} rec_base={RECORD_BASE_BYTES} files={FILE_COUNT} \
             model=counting file_ops=0"
        ))
    );
    for &inline_threshold_bytes in THRESHOLDS.iter() {
        for seed in 1..=5u64 {
            let totals = simulate_files_at_threshold(inline_threshold_bytes, seed);
            let total_bytes = totals.data_bytes + totals.meta_bytes;
            println!(
                "{}",
                emitter.emit_raw(&format!(
                    "name=arm thr={inline_threshold_bytes} seed={seed} total_bytes_per_file={} \
                     data_per_file={} meta_per_file={} leaves={} inlined_pct_ppm={} \
                     read_ops_x1000={}",
                    total_bytes / FILE_COUNT,
                    totals.data_bytes / FILE_COUNT,
                    totals.meta_bytes / FILE_COUNT,
                    totals.leaves,
                    totals.inlined_file_count * 1_000_000 / FILE_COUNT,
                    device_reads_per_file_read(&totals, 1) * 1000,
                ))
            );
        }
    }
    // 第二轮：带缓存的读模型，扫缓存大小 × 阈值。
    for cache_size_mebibytes in [1u64, 4, 16, 64, 256] {
        for &inline_threshold_bytes in THRESHOLDS.iter() {
            let totals = simulate_files_at_threshold(inline_threshold_bytes, 1);
            let (reads_per_lookup_parts_per_million, working_set_bytes) = reads_per_lookup(&totals, cache_size_mebibytes * 1024 * 1024);
            println!(
                "{}",
                emitter.emit_raw(&format!(
                    "name=cached_read cache_mib={cache_size_mebibytes} thr={inline_threshold_bytes} \
                     reads_per_lookup_ppm={reads_per_lookup_parts_per_million} meta_working_set_bytes={working_set_bytes}"
                ))
            );
        }
    }

    // 阴性对照：文件全都是 1 MiB ⇒ 所有阈值档逐格相同。
    for &inline_threshold_bytes in THRESHOLDS.iter() {
        let (data_bytes_for_file, record_bytes_for_file) = data_and_record_bytes_for_file(1024 * 1024, inline_threshold_bytes);
        println!(
            "{}",
            emitter.emit_raw(&format!("name=negative_control_huge thr={inline_threshold_bytes} data={data_bytes_for_file} rec={record_bytes_for_file}"))
        );
    }
    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **阳性对照，对每一档阈值都跑**：T=0 ⇒ 与「从不内联」逐格相同。
    #[test]
    fn positive_control_zero_threshold_equals_never_inline() {
        for seed in 1..=5u64 {
            let totals = simulate_files_at_threshold(0, seed);
            assert_eq!(totals.inlined_file_count, 0, "T=0 却内联了");
            assert_eq!(totals.meta_bytes, (FILE_COUNT * RECORD_BASE_BYTES).div_ceil(NODE_SIZE_BYTES) * NODE_SIZE_BYTES);
        }
    }

    /// **阴性对照**：1 MiB 文件在任何阈值下都走 extent，逐格相同。
    #[test]
    fn negative_control_huge_file_same_everywhere() {
        let bytes_at_threshold_zero = data_and_record_bytes_for_file(1024 * 1024, 0);
        for &inline_threshold_bytes in THRESHOLDS.iter() {
            assert_eq!(data_and_record_bytes_for_file(1024 * 1024, inline_threshold_bytes), bytes_at_threshold_zero);
        }
    }

    /// **绝对值断言**：一个 100 B 文件不内联时占整块 4096；内联时占 0 数据字节、记录 164。
    #[test]
    fn absolute_single_file_accounting() {
        assert_eq!(data_and_record_bytes_for_file(100, 0), (4096, 64));
        assert_eq!(data_and_record_bytes_for_file(100, 256), (0, 164));
        // 内联省下 4096 − 100 = 3996 字节的块内碎片，付出 100 字节的记录膨胀
        assert_eq!(4096 - 100, 3996);
    }

    /// **绝对值断言**：阈值恰好等于文件大小时算内联（`<=` 不是 `<`）。
    #[test]
    fn absolute_boundary_is_inclusive() {
        assert_eq!(data_and_record_bytes_for_file(512, 512), (0, RECORD_BASE_BYTES + 512));
        assert_eq!(data_and_record_bytes_for_file(513, 512), (BLOCK_BYTES, RECORD_BASE_BYTES));
    }

    /// 内联计数必须与 data_and_record_bytes_for_file 的判定同一条边界——`simulate_files_at_threshold` 与 `data_and_record_bytes_for_file` 各写一遍
    /// `<=`，只对总量断言的话两处漂移不会被任何测试看见（变异审计补的）。
    #[test]
    fn inlined_counter_agrees_with_an_independent_recount() {
        for &inline_threshold_bytes in THRESHOLDS.iter() {
            let totals = simulate_files_at_threshold(inline_threshold_bytes, 3);
            let mut random_number_generator = LinearCongruentialGenerator(3u64.wrapping_mul(0x9E3779B97F4A7C15) | 1);
            let expected_inlined_file_count = (0..FILE_COUNT).filter(|_| draw_file_size_bytes(&mut random_number_generator) <= inline_threshold_bytes).count() as u64;
            assert_eq!(totals.inlined_file_count, expected_inlined_file_count, "thr={inline_threshold_bytes}: 内联计数与独立重算不等");
        }
    }

    /// 带缓存读模型的两个端点钉死（第二轮结论「最优阈值＝工作集装得进缓存那档」
    /// 出自这段代码，此前零断言）：缓存装下全部叶 ⇒ 读数恰为非内联占比；
    /// 零缓存 ⇒ 叶未命中恰为 100%。
    #[test]
    fn cached_read_model_pins_full_and_zero_cache() {
        let totals = simulate_files_at_threshold(512, 1);
        let inlined_parts_per_million = totals.inlined_file_count * 1_000_000 / FILE_COUNT;
        let (reads_with_full_cache, working_set_bytes) = reads_per_lookup(&totals, 1 << 40);
        assert_eq!(reads_with_full_cache, 1_000_000 - inlined_parts_per_million, "全缓存时读数应恰为非内联占比");
        assert_eq!(working_set_bytes, totals.leaves * NODE_SIZE_BYTES);
        let (reads_with_zero_cache, _) = reads_per_lookup(&totals, 0);
        assert_eq!(reads_with_zero_cache, 1_000_000 + (1_000_000 - inlined_parts_per_million),
            "零缓存时叶未命中应恰为 100%");
    }

    /// device_reads_per_file_read 的树高算术钉死：扇出 682（16 KiB / 24）下 682 叶高 2、683 叶高 3。
    /// 683 这一格同时钉住扇出常数——扇出算错一档它就变 2。
    #[test]
    fn read_count_pins_tree_height_and_fanout_arithmetic() {
        let totals_with_leaf_count = |leaves| ThresholdRunTotals { data_bytes: 0, meta_bytes: 0, leaves, inlined_file_count: 0 };
        assert_eq!(device_reads_per_file_read(&totals_with_leaf_count(1), 0), 1, "单叶树高 1");
        assert_eq!(device_reads_per_file_read(&totals_with_leaf_count(682), 0), 2, "682 叶恰好一层内节点");
        assert_eq!(device_reads_per_file_read(&totals_with_leaf_count(683), 0), 3, "683 叶要两层");
        assert_eq!(device_reads_per_file_read(&totals_with_leaf_count(400), 1), 3, "高 2 加一个数据块");
    }

    /// 大小分布必须两峰都在：≤4 KiB 与 >4 KiB 都要出现，
    /// 否则「大文件把所有阈值档拉平」的阴性对照空转。
    #[test]
    fn size_distribution_has_both_modes() {
        let mut random_number_generator = LinearCongruentialGenerator(1u64.wrapping_mul(0x9E3779B97F4A7C15) | 1);
        let file_sizes: Vec<u64> = (0..10_000).map(|_| draw_file_size_bytes(&mut random_number_generator)).collect();
        assert!(file_sizes.iter().any(|&file_size_bytes| file_size_bytes <= 4096), "没有小文件");
        assert!(file_sizes.iter().any(|&file_size_bytes| file_size_bytes > 4096), "没有大文件——阴性对照那一维空转");
    }
}

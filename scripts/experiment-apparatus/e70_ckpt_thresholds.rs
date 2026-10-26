//! E70：checkpoint 两个阈值的可行域 —— D16 已定项 5。
//!
//! ## 被引用条款逐字贴在这里
//!
//! - D16 已定项 2（2026-08-31 用户定案）：
//!   `触发 ⟺ (now − 上次 checkpoint ≥ T_time) ∨ (脏字节数 ≥ T_dirty)`。
//! - D16 已定项 5 正文写着两条耦合：「`T_dirty` 的上界绑 journal 环几何
//!   （I-8.1：一个 checkpoint 的最坏 journal 占用不许超过环装得下的量）；
//!   `T_time` 的下界绑 fsync 率（E44 实测本机 2785 次/秒）」。**本实验要验的正是这两句对不对。**
//! - I-8.1（环几何够大）：`环大小 ≥ F × 任一事务的最坏 journal 占用`，F ≥ 2。
//! - D23 已定项 4：记录头 **78 字节**。D23：记录承载**指针层目标态**（点名指针，不搬数据）。
//! - D8 已定：节点 **16 KiB**。E44 实测：本机 fsync **2785 次/秒**。
//! - E16 实测（祖先延到 checkpoint 时的祖先块/操作）：
//!   multistream ckpt=100 → 0.4966、ckpt=1000 → 0.0567；seq 100 → 0.0126、1000 → 0.0083。
//!
//! ## 判据（跑前写死，跑完不许改）
//!
//! 1. `T_dirty` 上界 = `(环大小/F − 记录头) × (节点大小 / 条目字节)`。
//!    **若它在候选环大小上大过一台机器可能有的脏页内存（本实验取 1 TiB 作荒谬线），
//!    就判「环几何不是 T_dirty 的约束」**——那说明 D16 已定项 5 正文那句耦合写错了方向，如实记。
//! 2. `T_time` 下界：给定摊销预算 `B`（块/操作），从 E16 的两点插值反解一次 checkpoint
//!    至少要盖住多少次操作，再按 fsync 率折成毫秒。
//! 3. 两个界必须**互不依赖**——若反解出的 `T_time` 又依赖 `T_dirty`，判「模型循环」，整轮作废。
//!
//! ## 失败条款
//!
//! - **阳性对照**：`F` 加倍 ⇒ `T_dirty` 上界必须恰好减半（线性关系自证）。
//! - **阴性对照**：条目字节 = 节点大小 ⇒ 上界塌成 `环大小/F − 头`（journal 与数据一样大，没有放大）。
//! - E16 只有两个点，**插值是对数线性外推**，超出 [100, 1000] 的部分标为外推，不当实测。
//!
//! ## 它答不了的
//!
//! 纯算术模型：没有事务层、没有 journal 实现，文件操作 0 处。
//! 「每个脏节点在 journal 里恰好占一个条目」是**假设**，D23 未定项里那条
//! 「一个事务恰不恰好产生一条记录」还开着（C37）。

use e7_index_bench::Emitter;

const RECORD_HEADER_BYTES: u64 = 78;
const NODE_SIZE_BYTES: u64 = 16 * 1024;
const ENTRY: u64 = 32; // 一个带设备身份的位置条目，D19 已定项 1
const FSYNC_PER_SECOND: u64 = 2785;
const ABSURD_DIRTY_BYTES: u64 = 1024 * 1024 * 1024 * 1024; // 1 TiB 荒谬线

/// I-8.1 反解：一次 checkpoint 允许的最大脏字节数。
fn dirty_bytes_threshold_upper_bound(ring_bytes: u64, ring_safety_factor: u64, entry_bytes: u64, node_bytes: u64) -> u64 {
    let journal_bytes_budget_per_checkpoint = ring_bytes / ring_safety_factor;
    if journal_bytes_budget_per_checkpoint <= RECORD_HEADER_BYTES {
        return 0;
    }
    (journal_bytes_budget_per_checkpoint - RECORD_HEADER_BYTES) * node_bytes / entry_bytes
}

/// E16 两点（ckpt=100 / 1000）对数线性插值：给定预算，反解 checkpoint 要盖住多少次操作。
fn operations_per_checkpoint_for_budget(blocks_per_operation_at_checkpoint_100: f64, blocks_per_operation_at_checkpoint_1000: f64, budget_blocks_per_operation: f64) -> f64 {
    // 代价(操作数) = blocks_per_operation_at_checkpoint_100 × (操作数/100)^power_law_exponent，
    // power_law_exponent = ln(blocks_per_operation_at_checkpoint_1000 / blocks_per_operation_at_checkpoint_100) / ln(10)
    let power_law_exponent = (blocks_per_operation_at_checkpoint_1000 / blocks_per_operation_at_checkpoint_100).ln() / 10f64.ln();
    100.0 * (budget_blocks_per_operation / blocks_per_operation_at_checkpoint_100).powf(1.0 / power_law_exponent)
}

fn main() {
    let mut emitter = Emitter::new();
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=config rec_header={RECORD_HEADER_BYTES} node={NODE_SIZE_BYTES} entry={ENTRY} \
             fsync_per_sec={FSYNC_PER_SECOND} model=arithmetic file_ops=0"
        ))
    );

    for ring_size_mebibytes in [16u64, 64, 256, 1024] {
        for ring_safety_factor in [2u64, 3] {
            let dirty_upper_bound_bytes = dirty_bytes_threshold_upper_bound(ring_size_mebibytes * 1024 * 1024, ring_safety_factor, ENTRY, NODE_SIZE_BYTES);
            println!(
                "{}",
                emitter.emit_raw(&format!(
                    "name=t_dirty ring_mib={ring_size_mebibytes} f={ring_safety_factor} upper_bytes={dirty_upper_bound_bytes} \
                     upper_gib={} absurd={}",
                    dirty_upper_bound_bytes / (1024 * 1024 * 1024),
                    u8::from(dirty_upper_bound_bytes > ABSURD_DIRTY_BYTES)
                ))
            );
        }
    }

    for (workload_name, blocks_per_operation_at_checkpoint_100, blocks_per_operation_at_checkpoint_1000) in [
        ("multistream", 0.4966f64, 0.0567f64),
        ("seq", 0.0126, 0.0083),
    ] {
        for budget_blocks_per_operation in [0.2f64, 0.1, 0.05] {
            let interpolated_operations = operations_per_checkpoint_for_budget(blocks_per_operation_at_checkpoint_100, blocks_per_operation_at_checkpoint_1000, budget_blocks_per_operation);
            // 预算已经宽于 ckpt=100 那一点 ⇒ 这条负载在 100 上就满足，没有下界
            let is_budget_met_at_checkpoint_100 = budget_blocks_per_operation >= blocks_per_operation_at_checkpoint_100;
            let operations_needed = if is_budget_met_at_checkpoint_100 { 0.0 } else { interpolated_operations };
            let time_threshold_milliseconds = operations_needed / FSYNC_PER_SECOND as f64 * 1000.0;
            let is_extrapolated = !is_budget_met_at_checkpoint_100 && !(100.0..=1000.0).contains(&operations_needed);
            println!(
                "{}",
                emitter.emit_raw(&format!(
                    "name=t_time load={workload_name} budget_x1000={} ops_needed={} \
                     t_time_ms_x10={} extrapolated={} already_ok={}",
                    (budget_blocks_per_operation * 1000.0) as u64,
                    operations_needed as u64,
                    (time_threshold_milliseconds * 10.0) as u64,
                    u8::from(is_extrapolated),
                    u8::from(is_budget_met_at_checkpoint_100)
                ))
            );
        }
    }

    // 阳性对照：F 加倍 ⇒ 上界减半。
    let upper_bound_with_factor_2 = dirty_bytes_threshold_upper_bound(64 * 1024 * 1024, 2, ENTRY, NODE_SIZE_BYTES);
    let upper_bound_with_factor_4 = dirty_bytes_threshold_upper_bound(64 * 1024 * 1024, 4, ENTRY, NODE_SIZE_BYTES);
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=positive_control_f_doubles a={upper_bound_with_factor_2} b={upper_bound_with_factor_4} halved={}",
            u8::from(upper_bound_with_factor_2 / 2 - upper_bound_with_factor_4 <= RECORD_HEADER_BYTES * (NODE_SIZE_BYTES / ENTRY) / 2)
        ))
    );
    // 阴性对照：条目字节 = 节点大小 ⇒ 没有放大。
    let upper_bound_with_entry_as_large_as_node = dirty_bytes_threshold_upper_bound(64 * 1024 * 1024, 2, NODE_SIZE_BYTES, NODE_SIZE_BYTES);
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=negative_control_no_amplification bytes={upper_bound_with_entry_as_large_as_node} expect={}",
            64 * 1024 * 1024 / 2 - RECORD_HEADER_BYTES
        ))
    );
    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **绝对值断言**：64 MiB 环、F=2、16 KiB 节点、32 B 条目
    /// ⇒ 上界 = (33554432/2 − 78) × 512 字节。
    #[test]
    fn absolute_dirty_threshold_upper_bound_for_64_mebibyte_ring() {
        let expected_upper_bound_bytes = (64u64 * 1024 * 1024 / 2 - RECORD_HEADER_BYTES) * (NODE_SIZE_BYTES / ENTRY);
        assert_eq!(dirty_bytes_threshold_upper_bound(64 * 1024 * 1024, 2, ENTRY, NODE_SIZE_BYTES), expected_upper_bound_bytes);
        assert_eq!(NODE_SIZE_BYTES / ENTRY, 512, "放大倍数就是节点/条目");
        assert_eq!(expected_upper_bound_bytes, 17_179_829_248, "64 MiB 环、F=2 的上界是 16 GiB 量级");
    }

    /// **阳性对照**：F 加倍 ⇒ 上界减半（差一个头字节的取整）。
    #[test]
    fn positive_control_doubling_safety_factor_halves_bound() {
        let upper_bound_with_factor_2 = dirty_bytes_threshold_upper_bound(64 * 1024 * 1024, 2, ENTRY, NODE_SIZE_BYTES);
        let upper_bound_with_factor_4 = dirty_bytes_threshold_upper_bound(64 * 1024 * 1024, 4, ENTRY, NODE_SIZE_BYTES);
        // 差恰好是头字节被多扣了一次：RECORD_HEADER_BYTES × 节点/条目 ÷ 2
        let header_rounding_slack_bytes = RECORD_HEADER_BYTES * (NODE_SIZE_BYTES / ENTRY) / 2;
        assert!(upper_bound_with_factor_2 / 2 - upper_bound_with_factor_4 <= header_rounding_slack_bytes, "a/2={} b={upper_bound_with_factor_4} slack={header_rounding_slack_bytes}", upper_bound_with_factor_2 / 2);
    }

    /// **阴性对照**：条目字节 = 节点大小 ⇒ 放大倍数 1。
    #[test]
    fn negative_control_no_amplification() {
        assert_eq!(
            dirty_bytes_threshold_upper_bound(64 * 1024 * 1024, 2, NODE_SIZE_BYTES, NODE_SIZE_BYTES),
            64 * 1024 * 1024 / 2 - RECORD_HEADER_BYTES
        );
    }

    /// 环小到装不下一个记录头 ⇒ 上界必须是 0，不许下溢绕回（变异审计补的：
    /// 此前没有任何测试走过 `journal_bytes_budget_per_checkpoint <= RECORD_HEADER_BYTES` 的护栏）。
    #[test]
    fn tiny_ring_yields_zero_not_underflow() {
        assert_eq!(dirty_bytes_threshold_upper_bound(100, 2, ENTRY, NODE_SIZE_BYTES), 0);
        assert_eq!(dirty_bytes_threshold_upper_bound(RECORD_HEADER_BYTES * 2, 2, ENTRY, NODE_SIZE_BYTES), 0, "预算恰等于头也该是 0");
    }

    /// 插值自证：喂 E16 的两个点回去，必须还原出 100 与 1000。
    #[test]
    fn interpolation_reproduces_measured_points() {
        let (blocks_per_operation_at_checkpoint_100, blocks_per_operation_at_checkpoint_1000) = (0.4966f64, 0.0567f64);
        assert!((operations_per_checkpoint_for_budget(blocks_per_operation_at_checkpoint_100, blocks_per_operation_at_checkpoint_1000, blocks_per_operation_at_checkpoint_100) - 100.0).abs() < 1.0);
        assert!((operations_per_checkpoint_for_budget(blocks_per_operation_at_checkpoint_100, blocks_per_operation_at_checkpoint_1000, blocks_per_operation_at_checkpoint_1000) - 1000.0).abs() < 5.0);
    }
}

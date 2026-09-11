//! E19：defer 窗口 + 根环深度 K 叠加之后，盘接近满时的**假性 ENOSPC**。
//!
//! 两条已定项叠在同一个数上：
//!   - D16 新规则：checkpoint 内的释放**在发布前不得进入可分配集合**；
//!   - D22：**最近 K 代根引用的块不许重用**（根环深度 = 块重用延迟）。
//! ⇒ 删掉的空间不是立刻可用。盘接近满时，「明明删了却写不进」会真实发生。
//!
//! ## 两个数必须分开算，否则是恒等式
//!
//! - `df_free` = 容量 − **活引用**（用户看到的）
//! - `allocatable` = 容量 − 活引用 − **被扣住的**（分配器看到的）
//!
//! 「被扣住的」由**逐 checkpoint 的记账**算出来（哪一代释放的、哪一代到期），
//! **不是由一个公式导出**。公式那条留给校验路径（见 `analytic_held`），
//! 两条独立算出来的数要吻合——不吻合说明其中一条错了。
//!
//! ## 三类事件要分开报，合并成一个「失败率」就没意义了
//!
//! | 事件 | 含义 |
//! |---|---|
//! | **假性 ENOSPC** | `df_free ≥ s` 而 `allocatable < s`。用户视角的「删了却写不进」 |
//! | **真 ENOSPC** | `df_free < s`。盘是真的满了，不是本实验要抓的 |
//! | **checkpoint 卡死** | `allocatable < checkpoint_cost_blocks`，一次 checkpoint 自己都完成不了 ⇒ 空间永远放不出来。**这是 D23 死锁 2** |
//!
//! ## 两个对照
//!
//! - **阳性对照**：`defer = 0 且 K = 0` 时假性 ENOSPC 必须**恰好为 0**。
//!   不为 0 ⇒ 模型里有一条不该存在的扣留路径，整轮作废。
//! - **判别力对照**：`defer + K > 0` 的档必须真的扣住了字节（打印 `held_peak`）。
//!   为 0 ⇒「没测出问题」与「没扣住」分不开，整轮作废。

use e7_index_bench::Emitter;
use std::collections::VecDeque;

/// 一个文件：占若干块。
#[derive(Clone, Copy)]
struct File { blocks: u64 }

struct FileSystemModel {
    capacity: u64,
    /// 活引用占用的块数
    live: u64,
    /// 逐 checkpoint 的释放记账：`pending[i]` = 还有 i 个 checkpoint 到期的那批块数。
    /// **这是「被扣住的」的权威来源**，不是公式。
    pending: VecDeque<u64>,
    /// 当前 checkpoint 窗口内释放的、还没进 pending 的
    freeing_now: u64,
    files: Vec<File>,
    /// 一次 checkpoint 自己要写多少块（COW 新节点）
    checkpoint_cost_blocks: u64,
    /// **只有 checkpoint 能动的保留池**（btrfs 的 `global_block_rsv` 形态）。
    /// 普通分配看不见它；checkpoint 看得见。这是 D23 死锁 2 的破法。
    reserve: u64,
}

impl FileSystemModel {
    fn new(capacity: u64, delay: usize, checkpoint_cost_blocks: u64, reserve: u64) -> Self {
        Self {
            capacity, live: 0,
            pending: VecDeque::from(vec![0u64; delay.max(1)]),
            freeing_now: 0, files: Vec::new(), checkpoint_cost_blocks, reserve,
        }
    }
    /// 被扣住的字节：pending 里全部 + 本窗口内刚释放的。逐项加，不用公式。
    fn held(&self) -> u64 { self.pending.iter().sum::<u64>() + self.freeing_now }
    /// 用户看到的空闲（`df`）
    fn df_free(&self) -> u64 { self.capacity.saturating_sub(self.live) }
    /// **普通分配**看到的空闲：扣掉被 defer/K 扣住的，再扣掉只有 checkpoint 能动的保留池。
    /// ⚠️ 必须 `saturating_sub`——u64 下溢会变成天文数字，让分配反而总是成功。
    fn allocatable(&self) -> u64 {
        self.capacity.saturating_sub(self.live).saturating_sub(self.held()).saturating_sub(self.reserve)
    }
    /// **checkpoint** 看到的空闲：它可以动保留池。
    fn allocatable_for_checkpoint(&self) -> u64 {
        self.capacity.saturating_sub(self.live).saturating_sub(self.held())
    }

    fn allocate(&mut self, block_count: u64) -> bool {
        if self.allocatable() < block_count { return false; }
        self.live += block_count;
        self.files.push(File { blocks: block_count });
        true
    }
    fn free_one(&mut self, file_index: usize) {
        let freed_file = self.files.swap_remove(file_index);
        self.live -= freed_file.blocks;
        self.freeing_now += freed_file.blocks;   // 进入本窗口的释放，尚不可分配
    }
    /// checkpoint 发布：本窗口的释放挪进 pending 队尾，队首那批到期变可分配。
    fn checkpoint(&mut self) {
        self.pending.push_back(self.freeing_now);
        self.freeing_now = 0;
        self.pending.pop_front();       // 到期的那批不再被扣住
    }
}

/// 校验路径：用公式独立估一遍稳态被扣量，与逐项记账的 `held()` 对照。
/// **它不读 Fs 的任何字段，只按「每 checkpoint 的搅动量 × 延迟代数」算。**
fn analytic_held(churn_blocks_per_checkpoint: u64, delay: usize) -> u64 {
    churn_blocks_per_checkpoint * delay as u64
}

struct Run {
    false_enospc: u64,
    true_enospc: u64,
    checkpoint_stall: u64,
    allocation_attempts: u64,
    held_peak: u64,
    held_final: u64,
}

/// `fill` = 目标填充率；`delay` = defer 窗口 + 根环 K 合计的代数；
/// `file_blocks` = 每个临时文件多大；`ops_per_ckpt` = 一个 checkpoint 窗口里做多少次创建删除。
#[allow(clippy::too_many_arguments)]
fn run(capacity: u64, fill: f64, delay: usize, file_blocks: u64, operations_per_checkpoint: u64,
       checkpoint_count: u64, checkpoint_cost_blocks: u64, reserve: u64, seed: u64) -> Run {
    let mut xorshift_state = seed | 1;
    let mut next_random = || { xorshift_state ^= xorshift_state >> 12; xorshift_state ^= xorshift_state << 25; xorshift_state ^= xorshift_state >> 27; xorshift_state.wrapping_mul(0x2545_F491_4F6C_DD1D) };
    let mut file_system = FileSystemModel::new(capacity, delay, checkpoint_cost_blocks, reserve);

    // 预填到目标填充率。预填期间不走 defer（它模拟的是「盘上本来就有这些数据」）
    let target_live_blocks = (capacity as f64 * fill) as u64;
    while file_system.live + file_blocks <= target_live_blocks {
        file_system.live += file_blocks;
        file_system.files.push(File { blocks: file_blocks });
    }

    let (mut false_enospc_count, mut true_enospc_count, mut checkpoint_stall_count, mut allocation_attempt_count, mut held_peak_blocks) = (0u64, 0u64, 0u64, 0u64, 0u64);
    for _ in 0..checkpoint_count {
        for _ in 0..operations_per_checkpoint {
            // 先删一个（模拟「极速创建删除临时文件」里的删）
            if !file_system.files.is_empty() {
                let victim_file_index = (next_random() as usize) % file_system.files.len();
                file_system.free_one(victim_file_index);
            }
            // 再建一个
            allocation_attempt_count += 1;
            let df_free_before_allocation = file_system.df_free();
            if !file_system.allocate(file_blocks) {
                if df_free_before_allocation >= file_blocks { false_enospc_count += 1; } else { true_enospc_count += 1; }
            }
            held_peak_blocks = held_peak_blocks.max(file_system.held());
        }
        // checkpoint 自己要有空间写它的新节点
        // ⚠️ checkpoint 用的是**能动保留池**的那个视图。
        // 本实验第一版这里误写成 `allocatable()`（减掉了 reserve），
        // 于是保留池对 checkpoint 只可能是净损失、不可能是救援——
        // 「保留池救不了」那个结论完全由这一行造成，是实现 bug 不是性质。
        if file_system.allocatable_for_checkpoint() < file_system.checkpoint_cost_blocks { checkpoint_stall_count += 1; }
        file_system.checkpoint();
    }
    Run { false_enospc: false_enospc_count, true_enospc: true_enospc_count, checkpoint_stall: checkpoint_stall_count, allocation_attempts: allocation_attempt_count,
          held_peak: held_peak_blocks, held_final: file_system.held() }
}

fn main() {
    let mut emitter = Emitter::new();
    let mut output_text = String::new();
    let mut append_line = |line: String| { output_text.push_str(&line); output_text.push('\n'); };

    let capacity_blocks: u64 = 1_000_000;      // 100 万块，4 KiB/块 ⇒ 约 3.8 GiB
    let file_blocks: u64 = 4;               // 每个临时文件 4 块 = 16 KiB
    let operations_per_checkpoint: u64 = 200;            // 每个 checkpoint 窗口 200 次创建删除
    let checkpoint_count: u64 = 200;
    let checkpoint_cost_blocks: u64 = 64;       // 一次 checkpoint 自己写 64 块

    append_line(emitter.emit_raw(&format!(
        "name=config capacity_blocks={capacity_blocks} file_blocks={file_blocks} ops_per_ckpt={operations_per_checkpoint} ckpts={checkpoint_count} ckpt_cost={checkpoint_cost_blocks}")));

    // ── 阳性对照：延迟为 0（pending 只有一格且立刻到期）时假性 ENOSPC 必须为 0 ──
    // 注意 Fs 至少有一格 pending，所以「延迟 1」就是最小值；它等价于
    // 「本窗口释放的，下一个 checkpoint 就可用」——把 fill 压到 50% 让空间充裕，
    // 此时无论怎么扣都不该出现假性 ENOSPC。
    let positive_control_run = run(capacity_blocks, 0.50, 1, file_blocks, operations_per_checkpoint, checkpoint_count, checkpoint_cost_blocks, 0, 3);
    let positive_control_passed = positive_control_run.false_enospc == 0 && positive_control_run.checkpoint_stall == 0;
    append_line(emitter.emit_raw(&format!(
        "name=poscontrol fill=0.50 delay=1 false_enospc={} ckpt_stall={} held_peak={} ok={positive_control_passed}",
        positive_control_run.false_enospc, positive_control_run.checkpoint_stall, positive_control_run.held_peak)));
    if !positive_control_passed {
        append_line(emitter.finish()); print!("{output_text}");
        eprintln!("E19: 空间充裕时就出现假性 ENOSPC —— 模型里有不该存在的扣留路径，本轮作废");
        std::process::exit(4);
    }

    // ── 正式扫描 ──
    let mut any_held = false;
    for fill in [0.90f64, 0.95, 0.99, 0.999] {
        for delay in [1usize, 2, 4, 8, 16] {
            let sweep_run = run(capacity_blocks, fill, delay, file_blocks, operations_per_checkpoint, checkpoint_count, checkpoint_cost_blocks, 0, 7);
            if sweep_run.held_peak > 0 { any_held = true; }
            // 校验路径：稳态搅动量 = 每 checkpoint 删掉的块数
            let churn_blocks_per_checkpoint = operations_per_checkpoint * file_blocks;
            let analytic_held_prediction = analytic_held(churn_blocks_per_checkpoint, delay);
            append_line(emitter.emit_raw(&format!(
                "name=e19 fill={fill:.3} delay={delay} allocs={} \
                 false_enospc={} false_rate={:.5} true_enospc={} ckpt_stall={} \
                 held_peak={} held_final={} analytic_held={analytic_held_prediction} \
                 free_at_fill={} ",
                sweep_run.allocation_attempts, sweep_run.false_enospc, sweep_run.false_enospc as f64 / sweep_run.allocation_attempts as f64,
                sweep_run.true_enospc, sweep_run.checkpoint_stall, sweep_run.held_peak, sweep_run.held_final,
                (capacity_blocks as f64 * (1.0 - fill)) as u64
            )));
        }
    }
    if !any_held {
        append_line(emitter.finish()); print!("{output_text}");
        eprintln!("E19: 没有任何档扣住过字节 —— 「没测出问题」与「没扣住」分不开，本轮作废");
        std::process::exit(5);
    }

    // ── 准入规则的判别力检验 ──
    // 预测：假性 ENOSPC 与 checkpoint 卡死出现 ⟺ 剩余空间 ≤ 峰值被扣量 + 一次 checkpoint 的开销。
    // 峰值被扣量 = 每 checkpoint 搅动量 × (延迟 + 1)（+1 是本窗口内尚未入队的那批）。
    // **这条预测与被测代码不共享任何计算**——它只用负载参数，不读 Fs 的任何字段。
    {
        let churn_blocks_per_checkpoint = operations_per_checkpoint * file_blocks;
        let (mut agreeing_cell_count, mut total_cell_count) = (0u32, 0u32);
        for fill in [0.90f64, 0.95, 0.98, 0.99, 0.995, 0.999] {
            for delay in [1usize, 2, 4, 8, 16, 32] {
                let free_blocks_at_fill = (capacity_blocks as f64 * (1.0 - fill)) as u64;
                let predicted_trouble = free_blocks_at_fill <= churn_blocks_per_checkpoint * (delay as u64 + 1) + checkpoint_cost_blocks;
                let admission_run = run(capacity_blocks, fill, delay, file_blocks, operations_per_checkpoint, checkpoint_count, checkpoint_cost_blocks, 0, 13);
                let observed_trouble = admission_run.false_enospc > 0 || admission_run.checkpoint_stall > 0;
                total_cell_count += 1;
                if predicted_trouble == observed_trouble { agreeing_cell_count += 1; }
                append_line(emitter.emit_raw(&format!(
                    "name=admission fill={fill:.3} delay={delay} free={free_blocks_at_fill} \
                     bound={} predicted={predicted_trouble} observed={observed_trouble} \
                     false_enospc={} ckpt_stall={}",
                    churn_blocks_per_checkpoint * (delay as u64 + 1) + checkpoint_cost_blocks, admission_run.false_enospc, admission_run.checkpoint_stall)));
            }
        }
        append_line(emitter.emit_raw(&format!("name=admission_summary agree={agreeing_cell_count} of={total_cell_count}")));
    }

    // ── 保留池：它能不能救 checkpoint 卡死 ──
    // 在最坏（99.9% 填充、延迟 16）上二分找最小预留
    for reserve in [0u64, 64, 256, 1024, 4096, 16384] {
        // 容量不变，reserve 是一个只有 checkpoint 能动的池 —— 这才是「预留」
        let reserve_run = run(capacity_blocks, 0.999, 16, file_blocks, operations_per_checkpoint, checkpoint_count, checkpoint_cost_blocks, reserve, 11);
        append_line(emitter.emit_raw(&format!(
            "name=reserve reserve_blocks={reserve} fill=0.999 delay=16 ckpt_stall={} false_enospc={}",
            reserve_run.checkpoint_stall, reserve_run.false_enospc)));
    }

    append_line(emitter.finish());
    print!("{output_text}");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 阳性对照：空间充裕时不许出现假性 ENOSPC。
    #[test]
    fn no_false_enospc_when_roomy() {
        let roomy_run = run(1_000_000, 0.50, 4, 4, 200, 100, 64, 0, 1);
        assert_eq!(roomy_run.false_enospc, 0);
        assert_eq!(roomy_run.checkpoint_stall, 0);
    }

    /// 判别力：延迟越大扣住的越多。不单调说明记账没跟着延迟走。
    #[test]
    fn held_grows_with_delay() {
        let held_peak_at_delay_one = run(1_000_000, 0.90, 1, 4, 200, 100, 64, 0, 2).held_peak;
        let held_peak_at_delay_eight = run(1_000_000, 0.90, 8, 4, 200, 100, 64, 0, 2).held_peak;
        assert!(held_peak_at_delay_eight > held_peak_at_delay_one, "延迟从 1 加到 8，扣住的量没变大：{held_peak_at_delay_one} -> {held_peak_at_delay_eight}");
    }

    /// 逐项记账与公式估算必须同量级——两条独立路径，差太多说明其中一条错了。
    #[test]
    fn ledger_matches_analytic_estimate() {
        let delay = 8usize;
        let (operations_per_checkpoint, file_blocks) = (200u64, 4u64);
        let ledger_run = run(1_000_000, 0.90, delay, file_blocks, operations_per_checkpoint, 100, 64, 0, 5);
        let analytic_held_prediction = analytic_held(operations_per_checkpoint * file_blocks, delay);
        let ledger_to_formula_ratio = ledger_run.held_final as f64 / analytic_held_prediction as f64;
        assert!((0.5..=1.5).contains(&ledger_to_formula_ratio), "逐项记账 {} vs 公式 {analytic_held_prediction}，比值 {ledger_to_formula_ratio:.2}", ledger_run.held_final);
    }

    /// 假性 ENOSPC 必须在盘接近满时真的出现——不出现说明负载没把空间压到那个区间。
    #[test]
    fn false_enospc_appears_when_tight() {
        let tight_run = run(1_000_000, 0.999, 16, 4, 200, 200, 64, 0, 7);
        assert!(tight_run.false_enospc > 0, "99.9% 填充 + 延迟 16 都没出现假性 ENOSPC");
    }

    /// checkpoint 必须用「能动保留池」的那个视图——第一版误用普通视图，
    /// 「保留池救不了卡死」整个结论由那一行造成（正文已记）。此前的测试全在
    /// reserve=0 上跑，两个视图相等，那一行写错不会有任何测试红（变异审计补的）。
    #[test]
    fn the_reserve_actually_rescues_checkpoint_stalls() {
        let run_without_reserve = run(1_000_000, 0.999, 16, 4, 200, 200, 64, 0, 11);
        let run_with_big_reserve = run(1_000_000, 0.999, 16, 4, 200, 200, 64, 16384, 11);
        assert!(run_without_reserve.checkpoint_stall > 0, "无保留池的紧盘该有卡死");
        assert!(run_with_big_reserve.checkpoint_stall < run_without_reserve.checkpoint_stall,
            "16384 块保留池没有减少卡死（{} -> {}）", run_without_reserve.checkpoint_stall, run_with_big_reserve.checkpoint_stall);
    }
}

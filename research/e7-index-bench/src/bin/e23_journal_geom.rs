//! E23：D23 已定项 2（定长环 vs 链式）与已定项 3（tail 存哪）的代价。
//!
//! **这两项不正交，必须一起量**：tail 的权威位置决定了空间什么时候能回收，
//! 而「空间什么时候能回收」正是环与链的分水岭。分开量会各自得出一个漂亮答案。
//!
//! ## 三个数，各自回答一个具体问题
//!
//! | 数 | 回答 |
//! |---|---|
//! | `steady_blocks` | 稳态每次 fsync 为 journal 付几块（记录 + tail 写） |
//! | `replay_blocks` | 崩溃后要重放几块——tail 落后多少就要多读多少 |
//! | `allocation_count` | 向分配器要过几次块。**链式独有，且它加重 D23 死锁 2** |
//!
//! ## 建模的要害：tail 推进与记录写是两件事
//!
//! `tail_inline`（XFS 形态）下 tail 只能**搭记录的便车**——不写记录就推不了 tail。
//! `tail_sb`（jbd2 形态）下 tail 可以在 checkpoint 完成时**单独写一次**推进。
//! ⇒ 空闲期崩溃时两者的重放量不同，而这正是 jbd2 那次 FUA 买到的东西。
//! 若把 tail 推进建模成「checkpoint 一完成就免费生效」，两条臂当场相等，实验归零。
//!
//! ## 口径
//!
//! 纯计数模型，不碰设备。块 = 4096 B。**数的是块，不是字节，也不是屏障**
//! （[decisions.md](decisions.md) D25 已登记 C25/C26 两笔口径欠账）。
//! 记录大小取 [decisions.md](decisions.md) D23 的头部 84 B + 每点名项 56 B，
//! 按 `physical_block_size` 向上对齐。

use e7_index_bench::Emitter;

const BLOCK: u64 = 4096;
const JOURNAL_RECORD_HEADER_BYTES: u64 = 84;
const ENTRY_BYTES: u64 = 56;

/// 已定项 2 的两条臂。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Shape {
    /// 定长环：mkfs 时定大小，写记录不分配，tail 推进即回收。
    Ring { blocks: u64 },
    /// 链式：每条记录的块动态分配，靠前一条记录里的指针串起来。
    Chain,
}

/// 已定项 3 的两条臂。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Tail {
    /// jbd2 形态：住 journal 超级块，固定位置、原地覆盖、FUA。
    /// 可以脱离记录单独推进 —— 这正是它那次 FUA 买到的东西。
    SuperBlock,
    /// XFS 形态：内联在每条记录头的 `tail_lsn` 里。零额外写，但只能搭便车推进。
    Inline,
}

impl Tail {
    fn label(self) -> &'static str {
        match self {
            Tail::SuperBlock => "tail_sb",
            Tail::Inline => "tail_inline",
        }
    }
    /// 头部是否需要那 8 字节的 `tail_lsn`。
    fn header_bytes(self) -> u64 {
        match self {
            Tail::SuperBlock => JOURNAL_RECORD_HEADER_BYTES - 8,
            Tail::Inline => JOURNAL_RECORD_HEADER_BYTES,
        }
    }
}

/// 一条记录**对齐后占几字节**：头 + 点名项，向上对齐到原子宽度 `physical_block_size`。
///
/// ⚠️ **必须按字节记，不能按块记。** 按块记会让 `physical_block_size` 这一维变成死代码：
/// `physical_block_size ≤ BLOCK` 时「先对齐到 pbs 再向上取整到 BLOCK」是恒等操作，
/// 于是 512 与 4096 给出完全相同的块数。变异测试抓到了这一条
/// （把对齐整个删掉，8 个测试一个都没红）。
///
/// 真正的机制是**打包**：pbs=512 时 8 条 140 B 的记录能挤进同一个 4096 块；
/// pbs=4096 时一条就占满一块。差 8 倍，而按记录独立算块数永远看不见。
fn record_bytes(named: u64, tail: Tail, physical_block_size: u64) -> u64 {
    let bytes = tail.header_bytes() + named * ENTRY_BYTES;
    bytes.div_ceil(physical_block_size) * physical_block_size
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
struct RunOutcome {
    record_blocks: u64,
    tail_blocks: u64,
    allocation_count: u64,
    checkpoint_count: u64,
    forced_checkpoint_count: u64,
    replay_blocks: u64,
    peak_journal_blocks: u64,
}

impl RunOutcome {
    fn steady_blocks(&self) -> u64 {
        self.record_blocks + self.tail_blocks
    }
}

/// 一轮。`named[i]` = 第 i 次 fsync 点名几项；`idle_after` = 第几次 fsync 之后进入空闲
/// （空闲期做一次 checkpoint，然后崩溃——这是分辨两种 tail 的那一格）。
fn simulate_journal_run(named: &[u64], shape: Shape, tail: Tail, physical_block_size: u64, checkpoint_every: usize, idle_after: Option<usize>) -> RunOutcome {
    let mut outcome = RunOutcome::default();
    let mut total_record_bytes: u64 = 0; // 记录字节总量，最后折成块——**折算只做一次**
    let mut live_journal_bytes: u64 = 0; // 尚未被 checkpoint 回收的 journal 字节
    let mut tail_lag: u64 = 0; // tail 落后于已回收位置的字节数

    for (fsync_index, &named_count) in named.iter().enumerate() {
        let aligned_record_bytes = record_bytes(named_count, tail, physical_block_size);

        // 环满则强制 checkpoint 腾地方。链式没有这个环 —— 它只会一直分配。
        if let Shape::Ring { blocks } = shape {
            if live_journal_bytes + aligned_record_bytes > blocks * BLOCK {
                outcome.forced_checkpoint_count += 1;
                outcome.checkpoint_count += 1;
                live_journal_bytes = 0;
                // checkpoint 之后 tail 该前进。谁能立刻兑现，取决于 tail 住哪。
                match tail {
                    Tail::SuperBlock => outcome.tail_blocks += 1, // 单独写一次，立刻兑现
                    Tail::Inline => tail_lag += aligned_record_bytes,         // 只能等下一条记录捎带
                }
            }
        }

        // 链式：分配由**字节流跨块边界**驱动，不是每条记录一次。
        // ⚠️ 按记录算会让打包这件事消失（512 扇区下 8 条挤一块仍记 8 次），
        // 单测 `chain_allocates_exactly_one_block_per_journal_block` 抓到过这一条。
        if shape == Shape::Chain {
            outcome.allocation_count += (total_record_bytes + aligned_record_bytes).div_ceil(BLOCK) - total_record_bytes.div_ceil(BLOCK);
        }
        total_record_bytes += aligned_record_bytes;
        live_journal_bytes += aligned_record_bytes;
        if tail == Tail::Inline {
            tail_lag = 0; // 记录写出去了，tail 搭上便车
        }
        outcome.peak_journal_blocks = outcome.peak_journal_blocks.max(live_journal_bytes.div_ceil(BLOCK));

        if (fsync_index + 1) % checkpoint_every == 0 {
            outcome.checkpoint_count += 1;
            live_journal_bytes = 0;
            match tail {
                Tail::SuperBlock => outcome.tail_blocks += 1,
                Tail::Inline => tail_lag += aligned_record_bytes,
            }
        }
    }

    // 空闲期：做一次 checkpoint 然后崩溃。要重放多少，取决于 tail 推没推得动。
    if let Some(idle_start_index) = idle_after {
        let tailing_record_bytes: u64 = named[idle_start_index.min(named.len())..].iter()
            .map(|&named_count| record_bytes(named_count, tail, physical_block_size)).sum();
        outcome.checkpoint_count += 1;
        match tail {
            Tail::SuperBlock => {
                outcome.tail_blocks += 1;
                outcome.replay_blocks = 0; // tail 单独推到最新 ⇒ 无需重放
            }
            Tail::Inline => {
                // 没有新记录可捎带 ⇒ tail 停在崩溃前最后一条记录处
                outcome.replay_blocks = tailing_record_bytes.max(tail_lag).div_ceil(BLOCK);
            }
        }
    }
    outcome.record_blocks = total_record_bytes.div_ceil(BLOCK);
    outcome
}

fn main() {
    let mut emitter = Emitter::new();
    let operation_count = 20_000usize;
    println!("{}", emitter.emit_raw(&format!(
        "name=config ops={operation_count} block={BLOCK} hdr={JOURNAL_RECORD_HEADER_BYTES} entry={ENTRY_BYTES}")));

    for physical_block_size in [512u64, 4096] {
        for (workload_label, named) in [
            ("grain1", vec![1u64; operation_count]),   // 细粒度：每次 fsync 点名 1 项
            ("grain10", vec![10u64; operation_count]), // 多流批 10：点名 10 项（E23 与 E16 口径一致）
        ] {
            for shape in [Shape::Ring { blocks: 4096 }, Shape::Chain] {
                for tail in [Tail::SuperBlock, Tail::Inline] {
                    let cell_outcome = simulate_journal_run(&named, shape, tail, physical_block_size, 1000, Some(operation_count - 500));
                    let shape_label = match shape { Shape::Ring { .. } => "ring", Shape::Chain => "chain" };
                    println!("{}", emitter.emit_raw(&format!(
                        "name=cell pbs={physical_block_size} wl={workload_label} shape={shape_label} tail={} rec_blocks={} tail_blocks={} \
                         steady={} allocs={} ckpts={} forced={} replay={} peak={}",
                        tail.label(), cell_outcome.record_blocks, cell_outcome.tail_blocks, cell_outcome.steady_blocks(),
                        cell_outcome.allocation_count, cell_outcome.checkpoint_count, cell_outcome.forced_checkpoint_count, cell_outcome.replay_blocks, cell_outcome.peak_journal_blocks)));
                }
            }
        }
    }
    // ── 环大小扫描：已定项 2 的关键一格「环太小会怎样」在上面 16 格里从没跑到
    // （forced 全是 0）。**阴性结果要能和「代码没跑到」分开**（rules/test-discipline.md）。
    // 扫描同时给出 D23 死锁 3 那条几何不变量的具体数：环要多大才不强制。
    for physical_block_size in [512u64, 4096] {
        let named = vec![1u64; operation_count];
        for ring_block_count in [4u64, 16, 64, 256, 1024, 4096] {
            let sweep_outcome = simulate_journal_run(&named, Shape::Ring { blocks: ring_block_count }, Tail::SuperBlock, physical_block_size, 1000, None);
            println!("{}", emitter.emit_raw(&format!(
                "name=ringsweep pbs={physical_block_size} ring_blocks={ring_block_count} forced={} ckpts={} tail_blocks={} peak={}",
                sweep_outcome.forced_checkpoint_count, sweep_outcome.checkpoint_count, sweep_outcome.tail_blocks, sweep_outcome.peak_journal_blocks)));
        }
    }

    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **绝对值必须被钉死，不能只让臂互比**（`rules/test-discipline.md`）。
    /// ⚠️ 断言里一律走常量，**不许把 84 / 56 硬写进测试自己的算术**——
    /// 变异测试抓到过：把 `ENTRY_BYTES` 从 56 改成 48，8 个测试一个都没红，
    /// 因为它们各自抄了一份字面量。
    #[test]
    fn record_size_is_exactly_the_documented_arithmetic() {
        assert_eq!(JOURNAL_RECORD_HEADER_BYTES, 84, "头部字节数与 decisions.md D23 的清单对不上");
        assert_eq!(ENTRY_BYTES, 56, "点名项 = 24 B 条目 + 32 B 校验和");
        let one_item_record_bytes = Tail::Inline.header_bytes() + ENTRY_BYTES;
        assert_eq!(one_item_record_bytes, 140);
        // pbs=512：140 B 向上对齐到 512
        assert_eq!(record_bytes(1, Tail::Inline, 512), 512);
        // pbs=4096：同一条记录被撑到 4096，浪费 3956 B
        assert_eq!(record_bytes(1, Tail::Inline, 4096), 4096);
        assert_eq!(record_bytes(1, Tail::Inline, 4096) - one_item_record_bytes, 3956);
    }

    /// **pbs 这一维必须真的改变结果。** 它曾经是死代码（按块记时 512 与 4096 完全相同），
    /// 由变异测试「把对齐整个删掉、一个测试都没红」抓出来。这条守住它不再退化。
    #[test]
    fn atomic_width_actually_changes_the_cost() {
        let small_sector_one_item_record_bytes = record_bytes(1, Tail::Inline, 512);
        let large_sector_one_item_record_bytes = record_bytes(1, Tail::Inline, 4096);
        assert_eq!(large_sector_one_item_record_bytes / small_sector_one_item_record_bytes, 8, "4Kn 上一条细粒度记录该占 512 扇区形态的 8 倍");
        // 而点名 10 项时差距塌下来 —— 方向也要钉住
        let small_sector_ten_item_record_bytes = record_bytes(10, Tail::Inline, 512);
        let large_sector_ten_item_record_bytes = record_bytes(10, Tail::Inline, 4096);
        assert_eq!(small_sector_ten_item_record_bytes, 1024);
        assert_eq!(large_sector_ten_item_record_bytes, 4096);
        assert!(large_sector_ten_item_record_bytes / small_sector_ten_item_record_bytes < large_sector_one_item_record_bytes / small_sector_one_item_record_bytes, "粒度变粗时 pbs 的影响该变小");
    }

    /// `tail_sb` 省掉头里那 8 字节，必须真的体现在算术里，否则两条臂只是换个名字。
    #[test]
    fn superblock_tail_actually_shrinks_the_header() {
        assert_eq!(Tail::SuperBlock.header_bytes(), JOURNAL_RECORD_HEADER_BYTES - 8);
        assert_eq!(Tail::Inline.header_bytes(), JOURNAL_RECORD_HEADER_BYTES);
        assert_eq!(Tail::SuperBlock.header_bytes(), 76);
    }

    /// **链式的分配次数必须恰好等于它写的块数。** 这条钉住 D23 死锁 2 的输入：
    /// journal 写本身要不要向分配器要空间。
    #[test]
    fn chain_allocates_exactly_one_block_per_journal_block() {
        let named = vec![1u64; 100];
        let chain_outcome = simulate_journal_run(&named, Shape::Chain, Tail::Inline, 4096, 1000, None);
        assert_eq!(chain_outcome.allocation_count, 100, "100 次 fsync、4Kn 上每条记录占满一块 ⇒ 100 次分配");
        assert_eq!(chain_outcome.allocation_count, chain_outcome.record_blocks, "分配次数应恰好等于记录块数");
        // pbs=512：8 条 512 B 记录挤进一个 4096 块 ⇒ 100 条只要 13 块
        let chain_outcome_with_512_byte_sectors = simulate_journal_run(&named, Shape::Chain, Tail::Inline, 512, 1000, None);
        assert_eq!(chain_outcome_with_512_byte_sectors.allocation_count, 13, "100 × 512 B = 51200 B ⇒ 13 块");
        assert_eq!(chain_outcome_with_512_byte_sectors.allocation_count, chain_outcome_with_512_byte_sectors.record_blocks);
    }

    /// **定长环一次分配都不许有。** 这是「journal 不可能被 COW 覆盖」的建模形态，
    /// 也是环相对链唯一的结构性优势——漏了它，整个已定项 2 就没有对照。
    #[test]
    fn ring_never_allocates() {
        let named = vec![1u64; 5000];
        for tail in [Tail::SuperBlock, Tail::Inline] {
            for physical_block_size in [512u64, 4096] {
                let ring_outcome = simulate_journal_run(&named, Shape::Ring { blocks: 4096 }, tail, physical_block_size, 1000, None);
                assert_eq!(ring_outcome.allocation_count, 0, "定长环不该向分配器要块（tail={tail:?} pbs={physical_block_size}）");
            }
        }
    }

    /// **阳性对照，且对每一条臂都跑**（`rules/test-discipline.md`）。
    /// 环小到装不下必须出现强制 checkpoint；环大到装得下必须一次都没有。
    /// 只测一侧分不清「机制生效」与「压根没触发」。
    #[test]
    fn ring_forces_checkpoints_only_when_it_is_too_small() {
        let named = vec![1u64; 3000];
        for tail in [Tail::SuperBlock, Tail::Inline] {
            for physical_block_size in [512u64, 4096] {
                let tight = simulate_journal_run(&named, Shape::Ring { blocks: 4 }, tail, physical_block_size, 1000, None);
                assert!(tight.forced_checkpoint_count > 0, "环只有 4 块却没强制过 checkpoint（tail={tail:?} pbs={physical_block_size}）");
                let roomy = simulate_journal_run(&named, Shape::Ring { blocks: 1_000_000 }, tail, physical_block_size, 1000, None);
                assert_eq!(roomy.forced_checkpoint_count, 0, "环足够大却仍强制 checkpoint（tail={tail:?} pbs={physical_block_size}）");
            }
        }
    }

    /// **两种 tail 在空闲崩溃这一格必须分开**——这是本实验存在的理由。
    /// 若建模成「checkpoint 一完成 tail 就免费生效」，两条臂当场相等，实验归零。
    #[test]
    fn the_two_tail_arms_differ_exactly_at_an_idle_crash() {
        let named = vec![1u64; 2000];
        let superblock_tail_outcome = simulate_journal_run(&named, Shape::Ring { blocks: 4096 }, Tail::SuperBlock, 4096, 1000, Some(1500));
        let inline_tail_outcome = simulate_journal_run(&named, Shape::Ring { blocks: 4096 }, Tail::Inline, 4096, 1000, Some(1500));
        assert_eq!(superblock_tail_outcome.replay_blocks, 0, "超级块 tail 能单独推进，空闲崩溃后不该有重放");
        assert_eq!(inline_tail_outcome.replay_blocks, 500, "内联 tail 只能搭便车 ⇒ 崩溃前最后 500 条各占一块");
        assert!(inline_tail_outcome.replay_blocks > superblock_tail_outcome.replay_blocks);
    }

    /// **而它买到那个是要付钱的**：超级块 tail 每次推进一次写，内联恒为零。
    #[test]
    fn superblock_tail_costs_exactly_one_write_per_advance() {
        let named = vec![1u64; 5000];
        let superblock_tail_outcome = simulate_journal_run(&named, Shape::Ring { blocks: 4096 }, Tail::SuperBlock, 4096, 1000, None);
        let inline_tail_outcome = simulate_journal_run(&named, Shape::Ring { blocks: 4096 }, Tail::Inline, 4096, 1000, None);
        assert_eq!(inline_tail_outcome.tail_blocks, 0, "内联 tail 不该有任何额外写");
        assert_eq!(superblock_tail_outcome.checkpoint_count, 5, "5000 次操作、每 1000 次一个 checkpoint");
        assert_eq!(superblock_tail_outcome.tail_blocks, superblock_tail_outcome.checkpoint_count, "超级块 tail 的写次数应恰好等于 checkpoint 次数");
    }
}

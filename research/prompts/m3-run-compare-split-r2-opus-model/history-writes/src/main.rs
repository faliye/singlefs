// 攻方腿模型（m3-run-compare-split-r2）：量第一轮正推腿那个推的数「随机历史开 GPU 约 60 步封顶」（2048 位持久掩码 ÷ 每次发布约 30 写）。
// 照崩溃注入快档的生成与执行（GenerationWeights::BROAD、两块 4 GiB 盘、不跑每步 checker、空间准入按公式），对每个种子跑一段 `steps` 步的历史，
// 录制流保留内容，切写表；给定状态表的写表从 mkfs 第一个不是清零的写起（E5 副本 crash_injection.rs 的 HistoryAfterTheMakeFilesystem::of），
// 这里数「崩溃窗口在第 k 步末尾时」表里有几次写（第 k 步跑完时流里的步数换成写表下标），报第一次超过 2048 的是第几步。
// 不摆崩溃状态、不恢复、不判：崩溃状态数 0。只依赖 singlefs-harness 与 singlefs-core。
// 用法：history-writes <首个种子偏移> <种子数> <步数…>
use singlefs_core::admission::SpaceAdmission;
use singlefs_harness::history::{
    execute_history_with, generate_history_with_weights, GenerationWeights, HistoryDeviceWidth,
    HistoryExecution, HistorySeed, PerStepChecker, StepPosition, SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
};
use singlefs_harness::memory_pool::writes_and_segments_with_stream_indexes;
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;

const MASK_BITS: usize = 2048;

fn main() {
    let arguments: Vec<String> = std::env::args().collect();
    let first_offset: u64 = arguments[1].parse().expect("首个种子偏移");
    let seeds: u64 = arguments[2].parse().expect("种子数");
    let step_counts: Vec<usize> = arguments[3..].iter().map(|text| text.parse().expect("步数")).collect();
    let execution = HistoryExecution {
        per_step_checker: PerStepChecker::Skipped,
        device_width: HistoryDeviceWidth::FourGibibytes,
        space_admission: SpaceAdmission::JudgedByTheFormula,
    };
    for steps in step_counts {
        for offset in first_offset..first_offset + seeds {
            let started = std::time::Instant::now();
            let history = generate_history_with_weights(
                HistorySeed(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE.wrapping_add(offset)),
                steps,
                &GenerationWeights::BROAD,
            );
            let stream = SharedStream::retaining_contents();
            let mut after_each_step: Vec<(StepPosition, usize)> = Vec::new();
            let run = execute_history_with(&history, execution, &stream, &mut |observation| {
                after_each_step.push((observation.position, stream.operation_count()));
            });
            let (writes, _segments, stream_indexes) = writes_and_segments_with_stream_indexes(
                &stream.retained_operations(),
                &execution.device_width.fixed_geometry(),
            );
            let writes_of_the_make_filesystem = stream_indexes
                .iter()
                .take_while(|index| **index < run.operations_written_by_make_filesystem)
                .count();
            let first_write_of_the_table = writes[..writes_of_the_make_filesystem]
                .iter()
                .position(|write| write.kind != StepKind::ZeroFill)
                .unwrap_or(writes_of_the_make_filesystem);
            // 第 k 步跑完时表里的写数：流下标 < 那一刻流长的写
            let mut table_writes_after_step: Vec<(usize, usize)> = Vec::new();
            for (position, operations_so_far) in &after_each_step {
                if let StepPosition::Operation(step_index) = position {
                    let writes_so_far = stream_indexes.iter().take_while(|index| **index < *operations_so_far).count();
                    table_writes_after_step.push((*step_index + 1, writes_so_far.saturating_sub(first_write_of_the_table)));
                }
            }
            let steps_run = table_writes_after_step.len();
            let at_end = table_writes_after_step.last().map_or(0, |(_, writes)| *writes);
            let first_over = table_writes_after_step
                .iter()
                .find(|(_, writes)| *writes > MASK_BITS)
                .map_or("none".to_string(), |(step, _)| step.to_string());
            let at_24 = table_writes_after_step.iter().find(|(step, _)| *step == 24).map_or("-".to_string(), |(_, w)| w.to_string());
            println!(
                "HISTORY_WRITES steps={steps} seed_offset={offset} steps_run={steps_run} ending={:?} mkfs_data_writes={} table_writes_at_step_24={at_24} table_writes_at_end={at_end} first_step_over_{MASK_BITS}={first_over} seconds={:.2}",
                std::mem::discriminant(&run.ending),
                writes_of_the_make_filesystem - first_write_of_the_table,
                started.elapsed().as_secs_f64()
            );
        }
    }
}

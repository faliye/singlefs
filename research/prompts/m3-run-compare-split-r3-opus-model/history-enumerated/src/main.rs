// 攻方腿模型（m3-run-compare-split-r3）：核第二轮攻方 Q2 那个数「随机历史写表第一次过 2048 在第 71–136 步」的口径。
// 那一份数的是录制流里 mkfs 第一个非清零写之后的写；GPU 判表的持久掩码装的是枚举写表（`crash::layer0_enumeration_tables` 的
// `enumerated_writes`：录制流的写在前，原地覆写的撕裂镜像与重放接在后面，`crash_judge_tables.rs` 建给定状态表时拿它的长度比 2048）。
// 这里照快档的生成与执行（GenerationWeights::BROAD、两块 4 GiB 盘、不跑每步 checker、空间准入按公式）跑每段历史，第 k 步跑完时取那一截写表，
// 照崩溃注入建给定状态表的同一条路（`HistoryAfterTheMakeFilesystem::of`，不展开）算枚举写表长度，两种口径各报第一次过 2048 的步。
// 不摆崩溃状态、不恢复、不判：崩溃状态数 0（不调 enumerate_layer0*，`closed_form_state_count` 无从算起，也无须算）。
// 用法：history-enumerated <首个种子偏移> <种子数> <步数>
use singlefs_checker_tier::crash::{layer0_enumeration_tables, Layer0SegmentExpansion};
use singlefs_checker_tier::crash_injection::HistoryAfterTheMakeFilesystem;
use singlefs_core::admission::SpaceAdmission;
use singlefs_harness::history::{
    execute_history_with, generate_history_with_weights, GenerationWeights, HistoryDeviceWidth,
    HistoryExecution, HistorySeed, PerStepChecker, StepPosition, SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
};
use singlefs_harness::memory_pool::writes_and_segments_with_stream_indexes;
use singlefs_harness::SharedStream;

const MASK_BITS: usize = 2048;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let first_offset: u64 = a[1].parse().unwrap();
    let seeds: u64 = a[2].parse().unwrap();
    let steps: usize = a[3].parse().unwrap();
    let execution = HistoryExecution {
        per_step_checker: PerStepChecker::Skipped,
        device_width: HistoryDeviceWidth::FourGibibytes,
        space_admission: SpaceAdmission::JudgedByTheFormula,
    };
    let not_expanded = |_: usize, _: &[usize]| Layer0SegmentExpansion::NotExpanded;
    for offset in first_offset..first_offset + seeds {
        let started = std::time::Instant::now();
        let history = generate_history_with_weights(
            HistorySeed(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE.wrapping_add(offset)),
            steps,
            &GenerationWeights::BROAD,
        );
        let stream = SharedStream::retaining_contents();
        let mut after_each_step: Vec<(usize, usize)> = Vec::new();
        let run = execute_history_with(&history, execution, &stream, &mut |observation| {
            if let StepPosition::Operation(step_index) = observation.position {
                after_each_step.push((step_index + 1, stream.operation_count()));
            }
        });
        let (writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes(
            &stream.retained_operations(),
            &execution.device_width.fixed_geometry(),
        );
        let mut first_recorded_over = None;
        let mut first_enumerated_over = None;
        let mut at_recorded_crossing = String::from("-");
        let mut tearable_at_end = 0usize;
        let mut last = (0usize, 0usize, 0usize);
        for (step, operations_so_far) in &after_each_step {
            let n = stream_indexes.iter().take_while(|index| **index < *operations_so_far).count();
            let prefix_segments: Vec<Vec<usize>> = segments
                .iter()
                .map(|segment| segment.iter().copied().filter(|w| *w < n).collect::<Vec<usize>>())
                .filter(|segment| !segment.is_empty())
                .collect();
            let table = HistoryAfterTheMakeFilesystem::of(
                &writes[..n],
                &prefix_segments,
                &stream_indexes[..n],
                run.operations_written_by_make_filesystem,
                execution,
            );
            let recorded = table.writes().len();
            if first_recorded_over.is_none() && recorded <= MASK_BITS / 4 {
                continue; // 录制口径还不到 512 的步不算枚举表（省时）；枚举写表要比录制写表多出 1536 个以上才会在这几步里先过 2048，由下面每一步的比值看得出
            }
            let tables = layer0_enumeration_tables(table.base(), table.writes(), table.segments(), &not_expanded);
            let enumerated = tables.enumerated_writes.len();
            let tearable = tables.is_tearable_by_write.iter().filter(|t| **t).count();
            if first_enumerated_over.is_none() && enumerated > MASK_BITS {
                first_enumerated_over = Some(*step);
            }
            if first_recorded_over.is_none() && recorded > MASK_BITS {
                first_recorded_over = Some(*step);
                at_recorded_crossing = format!("{recorded}/{enumerated}");
            }
            tearable_at_end = tearable;
            last = (*step, recorded, enumerated);
            if first_recorded_over.is_some() && first_enumerated_over.is_some() {
                break;
            }
        }
        let show = |v: Option<usize>| v.map_or("none".to_string(), |s| s.to_string());
        println!(
            "HISTORY_ENUMERATED steps={steps} seed_offset={offset} steps_run={} first_step_recorded_over_{MASK_BITS}={} first_step_enumerated_over_{MASK_BITS}={} recorded_and_enumerated_at_recorded_crossing={at_recorded_crossing} last_step_measured={} recorded_there={} enumerated_there={} tearable_there={tearable_at_end} seconds={:.2}",
            after_each_step.len(), show(first_recorded_over), show(first_enumerated_over), last.0, last.1, last.2, started.elapsed().as_secs_f64()
        );
    }
}

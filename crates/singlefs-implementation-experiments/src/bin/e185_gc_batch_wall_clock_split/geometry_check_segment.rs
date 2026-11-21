use super::*;

// ───────────────────────────── 单点核 geometry-check（第 2 次重跑登记第五节「单点核」） ─────────────────────────────

/// 单点核：每个点只造池、核几何（[`prepare_the_sampling_point_with_the_order`]），不量、不进计时窗口。点与点之间互相独立，分组派到几条线程上
/// （每组几份照 [`tasks_of_the_next_group`]：空着的线程与内存预算 ÷ 一份峰值，一份都装不下时保底一份），组内并行、组间串行；按给的点的次序
/// 打攥着的行（`name=geometry_attempt`、N11、N12、`name=geometry`）与 `name=pool`。开跑先做常量回比（停机条款第 1 条）。
pub(super) fn run_the_geometry_check(
    points: &[SamplingPoint],
    overwrite_order: OverwriteOrder,
) -> i32 {
    let settings = PreparationSettings::from_the_environment();
    let mut emitter = Emitter::new();
    emitter.line(&format!(
        "name=configuration segment=geometry_check points={} overwrite_order={} worker_threads={} memory_cap_bytes={} memory_cap_source={}",
        points.iter().map(|point| format!("{}:{}", point.fraction.label(), point.pool_data_units)).collect::<Vec<_>>().join(","),
        overwrite_order.label(),
        settings.threads,
        settings.memory_cap.bytes,
        settings.memory_cap.source.label(),
    ));
    if !check_the_apparatus_sha256(&mut emitter) {
        return EXIT_CODE_STOP_CLAUSE;
    }
    let disagreeing = constants_disagreeing_with_the_crates();
    for (name, local_value, crates_value) in &disagreeing {
        emitter.line(&format!(
            "name=stop_clause clause=1 constant={name} local={local_value} crates={crates_value}"
        ));
    }
    if !disagreeing.is_empty() {
        emitter.line(&format!(
            "name=verdict segment=geometry_check constant_mismatches={}",
            disagreeing.len()
        ));
        emitter.finish();
        return EXIT_CODE_STOP_CLAUSE;
    }
    let mut prepared_points: Vec<PreparedPoint> = Vec::new();
    let mut remaining_points: &[SamplingPoint] = points;
    // 迭代上界：每一圈至少造一个点（组的份数保底 1），points.len() 圈之内做完
    while !remaining_points.is_empty() {
        let peak_bytes_in_order: Vec<u64> = remaining_points
            .iter()
            .map(|point| {
                estimated_pool_footprint_bytes_before_construction(*point)
                    * POOLS_HELD_WHILE_PREPARING_A_SAMPLING_POINT
            })
            .collect();
        let group_size = tasks_of_the_next_group(
            &peak_bytes_in_order,
            settings.threads,
            settings.memory_cap.bytes,
        )
        .clamp(1, remaining_points.len());
        let (points_of_this_group, later_points) = remaining_points.split_at(group_size);
        let prepared_group: Vec<PreparedPoint> = std::thread::scope(|scope| {
            let running: Vec<_> = points_of_this_group
                .iter()
                .map(|point| {
                    scope.spawn(move || {
                        prepare_the_sampling_point_with_the_order(*point, overwrite_order)
                    })
                })
                .collect();
            running
                .into_iter()
                .map(|thread| {
                    thread
                        .join()
                        .unwrap_or_else(|panic_content| std::panic::resume_unwind(panic_content))
                })
                .collect()
        });
        prepared_points.extend(prepared_group);
        remaining_points = later_points;
    }
    for prepared in &prepared_points {
        for line in &prepared.lines {
            emitter.line(line);
        }
        emitter.line(&pool_line(prepared, 0));
    }
    emitter.line(&format!(
        "name=verdict segment=geometry_check constant_mismatches=0 overwrite_order={} points={} geometry_formed_points={}",
        overwrite_order.label(),
        prepared_points.len(),
        prepared_points
            .iter()
            .filter(|prepared| prepared.is_the_geometry_formed)
            .count(),
    ));
    emitter.finish();
    EXIT_CODE_OK
}

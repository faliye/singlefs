use super::*;

// ============================================================================
// 十、并行：一段历史一件，`E198_THREADS` 个线程从共享的件号取活（登记第十四节）；结果按历史名的次序合并，与线程数无关。
//    G1 那一段最长（段内一次次挂载攒行，只能串着做），排在最前取。
// ============================================================================

/// 一处自己的线程变量从进程环境取：设了照它，没设取 `SINGLEFS_THREAD_CAP`，再没设取登记的每台默认重活线程数
/// （判法在 `singlefs_verification::heavy_threads::own_variable_or_default_heavy_threads`；设了而不是正整数就 panic）。
fn own_variable_or_default_heavy_threads_from_the_process_environment(
    own_variable: &str,
) -> std::num::NonZeroUsize {
    own_variable_or_default_heavy_threads(
        own_variable,
        std::env::var(own_variable).ok().as_deref(),
        std::env::var(singlefs_verification::heavy_threads::THREAD_CAP_ENVIRONMENT_VARIABLE)
            .ok()
            .as_deref(),
    )
}

/// 工作线程数：`E198_THREADS`（变量名写在这一条语句里：门禁 code-tooling 的 heavy-thread-defaults 格按读线程变量的那一条语句认它有没有回退到登记值）。
pub(super) fn worker_threads() -> usize {
    own_variable_or_default_heavy_threads_from_the_process_environment("E198_THREADS").get()
}

pub(super) fn run_the_histories(
    specifications: &[HistorySpecification],
    threads: usize,
) -> Vec<HistoryResult> {
    let mut order: Vec<usize> = (0..specifications.len()).collect();
    order.sort_by_key(|index| !specifications[*index].shape_accumulates_rows());
    let next_job = AtomicUsize::new(0);
    let results: Mutex<Vec<(usize, HistoryResult)>> = Mutex::new(Vec::new());
    let started = std::time::Instant::now();
    std::thread::scope(|scope| {
        for _ in 0..threads.max(1).min(specifications.len().max(1)) {
            scope.spawn(|| loop {
                let position = next_job.fetch_add(1, Ordering::SeqCst);
                let Some(&index) = order.get(position) else {
                    return;
                };
                let result = run_the_history(&specifications[index]);
                eprintln!(
                    "  进度：{} 判完 {} 张，已用 {:.0} 秒",
                    result.name,
                    result.images.len(),
                    started.elapsed().as_secs_f64()
                );
                results
                    .lock()
                    .expect("结果表的锁没被毒")
                    .push((index, result));
            });
        }
    });
    let mut results = results.into_inner().expect("结果表的锁没被毒");
    results.sort_by_key(|(index, _)| *index);
    results.into_iter().map(|(_, result)| result).collect()
}

impl HistorySpecification {
    fn shape_accumulates_rows(&self) -> bool {
        matches!(
            self.shape,
            HistoryShape::Sweep {
                accumulate_rows_by_mounts: true,
                ..
            }
        )
    }
}

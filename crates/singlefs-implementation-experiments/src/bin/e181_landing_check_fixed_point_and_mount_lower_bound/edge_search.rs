//! 边界的搜索：并行探测、找一个过得去的下界、二分与边界历史，连同点名边界几何的变量。

use super::*;

/// HF9-边 的结果：边界与窗口里的次数。
#[derive(Clone, Debug, Default)]
pub(super) struct EdgeReport {
    pub(super) pairs_outside_the_clusters: u64,
    pub(super) filled_units: u64,
    pub(super) boundary_units: u64,
    /// 找下界的结果：在填过的底上观测到放行的单元数（下界一个也没找到时是 `None`）。
    pub(super) lower_bound_units: Option<u64>,
    /// 找下界时没放行的试写，每条是「单元数:拒因」（拒因见 [`RefusalClass::gate_and_gap_text`]）。
    pub(super) lower_bound_refused_tries: Vec<String>,
    pub(super) bisection_rounds: u64,
    pub(super) bisection_points_tried: u64,
    pub(super) bisection_points_passed: u64,
    /// 边没被观测夹住（下界放行、下界 + 1 被拒）时的原因；找到了是 `None`。没找到时窗口不试，窗口里的各项都是 0。
    pub(super) edge_not_found_reason: Option<String>,
    pub(super) window_first: u64,
    pub(super) window_last: u64,
    pub(super) published: u64,
    pub(super) refused_by_space_admission: u64,
    pub(super) refused_in_the_fixed_point: u64,
    pub(super) refused_other: u64,
    /// 窗口里前 64 个（n* − 63 … n*）之中，落得下放行而固定点拒的次数。
    pub(super) fixed_point_refusals_in_the_first_sixty_four: u64,
    pub(super) fixed_point_refused_units: Vec<u64>,
    pub(super) space_refused_units_first_and_last: (Option<u64>, Option<u64>),
}

/// 在 `base` 的几份拷贝上并行试这些单元数的整份换写（每份各自从 `base` 起，互不影响）：交回每个单元数的结局，按单元数升序。
/// 线程数见 [`probe_thread_count`]。
pub(super) fn probe_writes_in_parallel(
    base: &Pool,
    unit_counts: &[usize],
    content_bytes: &[u8],
) -> Result<Vec<(usize, WriteOutcome)>, String> {
    let threads = probe_thread_count();
    let next_index = std::sync::atomic::AtomicUsize::new(0);
    let finished: std::sync::Mutex<Vec<(usize, Result<WriteOutcome, String>)>> =
        std::sync::Mutex::new(Vec::new());
    std::thread::scope(|scope| {
        for _thread in 0..threads.min(unit_counts.len()) {
            scope.spawn(|| loop {
                let index = next_index.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                let Some(units) = unit_counts.get(index).copied() else {
                    break;
                };
                let mut fork = base.fork_of_a_sealed_base();
                let outcome = write_units_from_bytes_and_account(
                    &mut fork,
                    units,
                    CodeTwoTreeNodeCapacities::FromTheNodeFormat,
                    &content_bytes[..units * DATA_UNIT_BYTES_LOCAL],
                );
                finished.lock().expect("锁").push((units, outcome));
            });
        }
    });
    let mut results = finished.into_inner().expect("锁");
    results.sort_by_key(|(units, _)| *units);
    results
        .into_iter()
        .map(|(units, outcome)| outcome.map(|outcome| (units, outcome)))
        .collect()
}

/// 同时试写的线程数：设了 `E181_PROBE_THREADS` 照它，没设取每台默认重活线程数（`singlefs_verification::heavy_threads`）。
/// 每个线程各拷一份底，一份 3 GiB 级盘上 3 万多个单元的写，拷贝加发布的账要占 5 GiB 上下，所以长跑脚本按内存上限显式给这个变量。
fn probe_thread_count() -> usize {
    let thread_cap_value =
        std::env::var(singlefs_verification::heavy_threads::THREAD_CAP_ENVIRONMENT_VARIABLE).ok();
    singlefs_verification::heavy_threads::own_variable_or_default_heavy_threads(
        "E181_PROBE_THREADS",
        std::env::var("E181_PROBE_THREADS").ok().as_deref(),
        thread_cap_value.as_deref(),
    )
    .get()
}

/// 找下界最多试几次（每次都是一整份换写，几十到几百秒）。
const LOWER_BOUND_PROBES_AT_MOST: usize = 40;

/// [`find_a_lower_bound_that_passes`] 的结局。
#[derive(Debug)]
pub(super) enum LowerBoundSearch {
    /// `units` 是放行的那个数（发成，或被拒在固定点）；`refused_before` 是它之前没放行的试写（单元数与拒因，按试的次序，单元数严格递减）。
    Found {
        units: usize,
        refused_before: Vec<(usize, RefusalClass)>,
    },
    /// 从填过的单元数一路试到 1 个单元，或试满 [`LOWER_BOUND_PROBES_AT_MOST`] 次，都没有放行的。
    NotFound {
        reason: String,
        refused: Vec<(usize, RefusalClass)>,
    },
}

/// 在填过的底上找一个放行的单元数当多路二分的下界：先试 `filled_units`（它是在更小的底上发成的，在填过的底上要换掉一个同样大的文件，另算），
/// 被拒就按 [`units_to_try_after_a_refusal`] 往下找（空间准入那一道按缺口一步跳过去），直到放行；放行的定义与二分一致
/// （[`passed_the_landing_check_and_the_formula`]：发成，或被拒在固定点）。
pub(super) fn find_a_lower_bound_that_passes(
    filled_units: usize,
    mut try_one_unit_count: impl FnMut(usize) -> Result<WriteOutcome, String>,
) -> Result<LowerBoundSearch, String> {
    assert!(filled_units >= 1, "填过的底至少有 1 个单元");
    let mut units = filled_units;
    let mut refused: Vec<(usize, RefusalClass)> = Vec::new();
    loop {
        let outcome = try_one_unit_count(units)?;
        if passed_the_landing_check_and_the_formula(&outcome) {
            return Ok(LowerBoundSearch::Found {
                units,
                refused_before: refused,
            });
        }
        let WriteOutcome::Refused(refusal) = outcome else {
            unreachable!("发成的写算放行，上面已经返回");
        };
        refused.push((units, refusal.clone()));
        if units == 1 {
            return Ok(LowerBoundSearch::NotFound {
                reason: format!(
                    "填过的底上，从 {filled_units} 个单元一路试到 1 个单元的整份换写都没放行"
                ),
                refused,
            });
        }
        if refused.len() >= LOWER_BOUND_PROBES_AT_MOST {
            return Ok(LowerBoundSearch::NotFound {
                reason: format!(
                    "填过的底上试了 {LOWER_BOUND_PROBES_AT_MOST} 次整份换写还没放行，最后试的是 {units} 个单元"
                ),
                refused,
            });
        }
        let next =
            units_to_try_after_a_refusal(&refusal, u64::try_from(units).expect("单元数装得进 u64"));
        units = usize::try_from(next).expect("单元数装得进 usize");
    }
}

/// 二分已经见过的被拒试写：单元数最小的（已知的上界），和空间准入拒的里单元数最小的（带缺口）。
#[derive(Debug, Default)]
struct RefusalsSeen {
    lowest_refused_units: Option<usize>,
    nearest_space_refusal: Option<(usize, i128)>,
}

impl RefusalsSeen {
    fn note(&mut self, units: usize, refusal: &RefusalClass) {
        self.lowest_refused_units = Some(
            self.lowest_refused_units
                .map_or(units, |lowest| lowest.min(units)),
        );
        if let Some(shortfall_bytes) = refusal.space_shortfall_bytes() {
            if self
                .nearest_space_refusal
                .is_none_or(|(nearest, _)| units < nearest)
            {
                self.nearest_space_refusal = Some((units, shortfall_bytes));
            }
        }
    }

    /// 按最近一次空间准入拒的缺口预测的边：被拒的单元数减去缺口折成的单元数（需求随单元数线性降、每个单元在一块盘上两槽时，
    /// 这就是空间那一道放行的最大单元数）。只是预测：二分只信观测到的放行与被拒，预测错了照样收敛。
    fn predicted_edge(&self) -> Option<usize> {
        let (units, shortfall_bytes) = self.nearest_space_refusal?;
        let worth = usize::try_from(units_a_shortfall_is_worth(shortfall_bytes)).ok()?;
        units.checked_sub(worth)
    }
}

/// 多路二分一轮试哪些点（都严格落在 `(low, high)` 里，所以每轮至少让一端移动）：有预测的边时先取预测的边与它上面一格
/// （预测对了，一轮就夹住），余下的线程把 `(low, high)` 等分；没有预测就只等分。
pub(super) fn points_of_a_bisection_round(
    low: usize,
    high: usize,
    threads: usize,
    predicted_edge: Option<usize>,
) -> Vec<usize> {
    assert!(high - low > 1, "区间里至少还有一个点可试");
    let gap = high - low;
    let mut points: BTreeSet<usize> = BTreeSet::new();
    if let Some(edge) = predicted_edge {
        for candidate in [edge, edge + 1] {
            if candidate > low && candidate < high && points.len() < threads {
                points.insert(candidate);
            }
        }
    }
    for index in 1..=threads {
        if points.len() >= threads {
            break;
        }
        let point = low + gap * index / (threads + 1);
        if point > low && point < high {
            points.insert(point);
        }
    }
    if points.is_empty() {
        points.insert(low + gap / 2);
    }
    points.into_iter().collect()
}

/// 边查找的结局。
#[derive(Debug, PartialEq, Eq)]
pub(super) enum EdgeSearchEnd {
    /// 边界 n*：`boundary_units` 放行（观测到的），`boundary_units + 1` 被拒（观测到的）。
    Edge { boundary_units: usize },
    /// 没有被观测到的放行与被拒夹住的边。
    NotFound { reason: String },
}

#[derive(Debug)]
pub(super) struct EdgeSearch {
    pub(super) end: EdgeSearchEnd,
    /// 找到的下界（没找到时是 `None`）与它之前没放行的试写。
    pub(super) lower_bound_units: Option<usize>,
    pub(super) lower_bound_refused_before: Vec<(usize, RefusalClass)>,
    pub(super) bisection_rounds: u64,
    pub(super) bisection_points_tried: u64,
    pub(super) bisection_points_passed: u64,
}

/// HF9-边 找 n*（落得下与式子两道都放行的最大单元数）：先在填过的底上找一个观测到放行的下界（[`find_a_lower_bound_that_passes`]），
/// 再在 `(下界, 上界)` 里多路二分，上界取找下界时见过的最小的被拒单元数（没有就取 `upper_sentinel`），每轮的点见 [`points_of_a_bisection_round`]。
/// 收尾要求边被观测夹住：下界放行、下界 + 1 被拒；一直放行到 `upper_sentinel` 都没有被拒的，或被拒的最小单元数不是下界 + 1（放行与被拒次序不一致），
/// 都不交出边界，交 `NotFound`。
pub(super) fn search_for_the_edge(
    filled_units: usize,
    upper_sentinel: usize,
    threads: usize,
    mut try_one_unit_count: impl FnMut(usize) -> Result<WriteOutcome, String>,
    mut try_unit_counts: impl FnMut(&[usize]) -> Result<Vec<(usize, WriteOutcome)>, String>,
) -> Result<EdgeSearch, String> {
    assert!(threads >= 1, "至少一个线程");
    let (lower_bound_units, refused_before) =
        match find_a_lower_bound_that_passes(filled_units, &mut try_one_unit_count)? {
            LowerBoundSearch::Found {
                units,
                refused_before,
            } => (units, refused_before),
            LowerBoundSearch::NotFound { reason, refused } => {
                return Ok(EdgeSearch {
                    end: EdgeSearchEnd::NotFound { reason },
                    lower_bound_units: None,
                    lower_bound_refused_before: refused,
                    bisection_rounds: 0,
                    bisection_points_tried: 0,
                    bisection_points_passed: 0,
                });
            }
        };
    let mut seen = RefusalsSeen::default();
    for (units, refusal) in &refused_before {
        seen.note(*units, refusal);
    }
    let mut low = lower_bound_units;
    let mut high = seen
        .lowest_refused_units
        .map_or(upper_sentinel, |units| units.min(upper_sentinel));
    assert!(high > low, "上界 {high} 要比下界 {low} 大");
    let mut rounds = 0_u64;
    let mut points_tried = 0_u64;
    let mut points_passed = 0_u64;
    while high - low > 1 {
        rounds += 1;
        let points = points_of_a_bisection_round(low, high, threads, seen.predicted_edge());
        for (units, outcome) in try_unit_counts(&points)? {
            points_tried += 1;
            if passed_the_landing_check_and_the_formula(&outcome) {
                points_passed += 1;
                low = low.max(units);
            } else {
                let WriteOutcome::Refused(refusal) = &outcome else {
                    unreachable!("发成的写算放行，上面已经归了放行");
                };
                high = high.min(units);
                seen.note(units, refusal);
            }
        }
        high = high.max(low + 1);
    }
    let end = if seen.lowest_refused_units == Some(low + 1) {
        EdgeSearchEnd::Edge {
            boundary_units: low,
        }
    } else if let Some(lowest) = seen.lowest_refused_units {
        EdgeSearchEnd::NotFound {
            reason: format!(
                "被拒的试写里最小的是 {lowest} 个单元，不是放行的下界 {low} 之上一格：放行与被拒在下界附近次序不一致"
            ),
        }
    } else {
        EdgeSearchEnd::NotFound {
            reason: format!("一直放行到上限 {upper_sentinel} 个单元都没有被拒的试写"),
        }
    };
    Ok(EdgeSearch {
        end,
        lower_bound_units: Some(lower_bound_units),
        lower_bound_refused_before: refused_before,
        bisection_rounds: rounds,
        bisection_points_tried: points_tried,
        bisection_points_passed: points_passed,
    })
}

/// 在封存的底的一份拷贝上试一个单元数的整份换写。
pub(super) fn probe_one_write_on_the_sealed_base(
    base: &Pool,
    units: usize,
    content_bytes: &[u8],
) -> Result<WriteOutcome, String> {
    let mut outcomes = probe_writes_in_parallel(base, &[units], content_bytes)?;
    let (probed_units, outcome) = outcomes
        .pop()
        .ok_or_else(|| format!("试 {units} 个单元的整份换写没有结局"))?;
    assert_eq!(probed_units, units, "交回的结局是被试的那个单元数的");
    Ok(outcome)
}

/// 登记 5.1 节 HF9-边（G窄）：先换写到「段外槽对数的一半」的九成，再在落得下那一判刚放行的 n 附近逐单元试，
/// 记每次是哪一道拒。边界 n* = 落得下与式子两道都放行的最大 n（多路二分，在拷贝上试，被拒的不改池）；
/// 试的范围取 [n* − 63, n* + 64]（登记写「往上逐单元试 64 次，一次都没有时把范围加倍再试」，装置直接取加倍后的超集，
/// 前 64 个单独记数）。
pub(super) fn run_edge_history(
    collector: &mut FixedPointUpperBoundCollector,
    geometry: Geometry,
    content: &mut ContentBuffer,
) -> Result<EdgeReport, String> {
    let mut base =
        build_pool_after_the_first_file(geometry, CodeTwoTreeNodeCapacities::FromTheNodeFormat);
    let pairs = data_unit_pairs_outside_the_clusters(&base, 0);
    let filled = pairs / 2 * 9 / 10;
    let filled_units = usize::try_from(filled).expect("单元数");
    match run_one_write(
        collector,
        "edge_history_fill",
        geometry.name,
        0,
        &mut base,
        filled_units,
        CodeTwoTreeNodeCapacities::FromTheNodeFormat,
        content,
    )? {
        WriteOutcome::Published(_) => {}
        WriteOutcome::Refused(refusal) => {
            return Err(format!("填到九成那一次就被拒：{refusal:?}"));
        }
    }
    base.seal();
    let upper_sentinel = usize::try_from(pairs).expect("槽对数") + 1024;
    let content_bytes: &[u8] = content.of_units(upper_sentinel);
    let search = search_for_the_edge(
        filled_units,
        upper_sentinel,
        probe_thread_count(),
        |units| probe_one_write_on_the_sealed_base(&base, units, content_bytes),
        |points| probe_writes_in_parallel(&base, points, content_bytes),
    )?;
    let mut report = EdgeReport {
        pairs_outside_the_clusters: pairs,
        filled_units: filled,
        lower_bound_units: search
            .lower_bound_units
            .map(|units| u64::try_from(units).expect("单元数")),
        lower_bound_refused_tries: search
            .lower_bound_refused_before
            .iter()
            .map(|(units, refusal)| format!("{units}:{}", refusal.gate_and_gap_text()))
            .collect(),
        bisection_rounds: search.bisection_rounds,
        bisection_points_tried: search.bisection_points_tried,
        bisection_points_passed: search.bisection_points_passed,
        ..EdgeReport::default()
    };
    let boundary = match search.end {
        EdgeSearchEnd::Edge { boundary_units } => boundary_units,
        EdgeSearchEnd::NotFound { reason } => {
            report.edge_not_found_reason = Some(reason);
            return Ok(report);
        }
    };
    let window_first = boundary.saturating_sub(63).max(1);
    let window_last = boundary + 64;
    report.boundary_units = u64::try_from(boundary).expect("单元数");
    report.window_first = u64::try_from(window_first).expect("单元数");
    report.window_last = u64::try_from(window_last).expect("单元数");
    let window: Vec<usize> = (window_first..=window_last).collect();
    for (units, outcome) in probe_writes_in_parallel(&base, &window, content_bytes)? {
        let units_u64 = u64::try_from(units).expect("单元数");
        match &outcome {
            WriteOutcome::Published(write) => {
                collector.record_write("edge_history", geometry.name, units, write);
                report.published += 1;
            }
            WriteOutcome::Refused(refusal) => {
                collector.record_refusal("edge_history", geometry.name, units, units, refusal);
            }
        }
        match outcome {
            WriteOutcome::Published(_) => {}
            WriteOutcome::Refused(RefusalClass::SpaceAdmission { .. }) => {
                report.refused_by_space_admission += 1;
                let (first, _last) = report.space_refused_units_first_and_last;
                report.space_refused_units_first_and_last =
                    (first.or(Some(units_u64)), Some(units_u64));
            }
            WriteOutcome::Refused(RefusalClass::FixedPointPlacement { .. }) => {
                report.refused_in_the_fixed_point += 1;
                report.fixed_point_refused_units.push(units_u64);
                if units <= boundary {
                    report.fixed_point_refusals_in_the_first_sixty_four += 1;
                }
            }
            WriteOutcome::Refused(RefusalClass::Other { .. }) => report.refused_other += 1,
        }
    }
    Ok(report)
}

/// 每块盘上把最低的 `percent`% 个段外槽对标成已分配（只改空闲图，不写盘、不写分配记录）：
/// 扫描挂钟只读空闲图，所以拿它造「最低落点靠后」的状态，不必真写几 GiB 的数据（登记修订里写明代替了什么）。
pub(super) fn mark_the_lowest_pairs_outside_the_clusters(
    allocator: &mut PoolAllocator,
    percent: u64,
) {
    let cluster_segments: BTreeSet<SlotNumber> = allocator.cluster_segments().clone();
    for device_map in &mut allocator.devices {
        let total_pairs = device_map
            .unblocked_slot_pairs_outside_the_cluster_segments(&cluster_segments)
            .0;
        let wanted = total_pairs * percent / 100;
        let mut marked = 0;
        let mut candidate = device_map.unit_area_start().slot().0;
        let end = candidate + device_map.unit_area_slots();
        while marked < wanted && candidate + 1 < end {
            let inside_cluster = cluster_segments.iter().any(|segment| {
                candidate >= segment.0 && candidate < segment.0 + CLUSTER_SEGMENT_SLOTS_LOCAL
            });
            if !inside_cluster
                && device_map.is_free(SlotNumber(candidate))
                && device_map.is_free(SlotNumber(candidate + 1))
            {
                device_map.mark_allocated(SlotNumber(candidate), DATA_UNIT_SLOTS_LOCAL);
                marked += 1;
            }
            candidate += DATA_UNIT_SLOTS_LOCAL;
        }
        assert_eq!(marked, wanted, "单元区里装得下要标的槽对数");
    }
}

/// 定点上界那一格跑边那一段的几何（登记的窄几何加后加的小几何）可由这个变量往小缩，逗号分隔。
pub(super) const EDGE_GEOMETRIES_VARIABLE: &str = "E181_FIXED_POINT_UPPER_BOUND_EDGE_GEOMETRIES";

/// 跑边的几何只许往小设：没设取登记的两个；设了的每个名字都要是登记的两个之一，写了别的名字拒（不认得的几何不是缩，是另一趟）。
/// 交回原样的设定串，产物照它写 `edge_geometries=`。
pub(super) fn edge_geometries_narrowed_or_registered(
    value_of_the_variable: Option<&str>,
) -> Result<String, String> {
    let registered = [GEOMETRY_NARROW.name, GEOMETRY_SMALL.name];
    let Some(text) = value_of_the_variable else {
        return Ok(registered.join(","));
    };
    for name in text.split(',') {
        if !registered.contains(&name) {
            return Err(format!(
                "环境变量 {EDGE_GEOMETRIES_VARIABLE} 只许从登记的几何 {} 里挑，给了 {name:?}",
                registered.join(",")
            ));
        }
    }
    Ok(text.to_string())
}

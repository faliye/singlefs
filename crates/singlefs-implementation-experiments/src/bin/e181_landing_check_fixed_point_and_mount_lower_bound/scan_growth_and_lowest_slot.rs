//! 扫描的耗时随状态长，与各盘最低那一槽的耗时。

use super::*;

/// 一块盘上 `lowest_user_data_slot` 的挂钟：重复 `repetitions` 次，取中位数与最大值；扫过的候选数 = 从单元区起点按 2 槽步进到落点的步数加 1。
#[derive(Clone, Copy, Debug)]
pub(super) struct ScanTiming {
    pub(super) lowest_slot: Option<u64>,
    pub(super) scanned_candidates: u64,
    median_nanoseconds: u128,
    maximum_nanoseconds: u128,
}

fn median_and_maximum(mut samples: Vec<u128>) -> (u128, u128) {
    samples.sort_unstable();
    (
        samples[samples.len() / 2],
        *samples.last().expect("至少一次"),
    )
}

pub(super) fn time_the_lowest_user_data_slot(
    allocator: &PoolAllocator,
    device_index: usize,
    repetitions: usize,
) -> ScanTiming {
    let device_map = &allocator.devices[device_index];
    let cluster_segments = allocator.cluster_segments();
    let mut samples = Vec::with_capacity(repetitions);
    let mut answer = None;
    for _repetition in 0..repetitions {
        let started = Instant::now();
        answer = std::hint::black_box(
            device_map.lowest_user_data_slot(std::hint::black_box(cluster_segments)),
        );
        samples.push(started.elapsed().as_nanos());
    }
    let (median_nanoseconds, maximum_nanoseconds) = median_and_maximum(samples);
    let start = device_map.unit_area_start().slot().0;
    ScanTiming {
        lowest_slot: answer.map(|slot| slot.0),
        scanned_candidates: answer.map_or_else(
            || device_map.unit_area_slots() / DATA_UNIT_SLOTS_LOCAL,
            |slot| (slot.0 - start) / DATA_UNIT_SLOTS_LOCAL + 1,
        ),
        median_nanoseconds,
        maximum_nanoseconds,
    }
}

/// 落点那一道今天为一个数据单元已经花的挂钟：对分配器的拷贝调 `try_allocate_user_data`（每块盘一次 `lowest_user_data_slot` 加各盘同槽的核），
/// 拷贝不进计时。
fn time_the_placement_of_one_data_unit(
    allocator: &PoolAllocator,
    repetitions: usize,
) -> (u128, u128) {
    let generation = CheckpointTxg(1 << 40);
    let mut samples = Vec::with_capacity(repetitions);
    for _repetition in 0..repetitions {
        let mut copy = allocator.clone();
        let started = Instant::now();
        let placement = std::hint::black_box(copy.try_allocate_user_data(generation));
        samples.push(started.elapsed().as_nanos());
        assert!(placement.is_ok(), "这个状态下数据单元有落点：{placement:?}");
    }
    median_and_maximum(samples)
}

const SCAN_REPETITIONS: usize = 1000;
const PLACEMENT_REPETITIONS: usize = 200;

/// F11 的一个状态：几何、名字、每块盘的扫描挂钟、落点那一道的挂钟。
#[derive(Clone, Debug)]
struct ScanStateReport {
    pub(super) geometry: &'static str,
    pub(super) state: &'static str,
    pub(super) unit_area_slots: u64,
    scans: Vec<ScanTiming>,
    placement_median_nanoseconds: u128,
}

fn measure_scan_state(
    geometry: &'static str,
    state: &'static str,
    allocator: &PoolAllocator,
) -> ScanStateReport {
    let scans: Vec<ScanTiming> = (0..allocator.devices.len())
        .map(|device_index| {
            time_the_lowest_user_data_slot(allocator, device_index, SCAN_REPETITIONS)
        })
        .collect();
    let (placement_median, placement_maximum) =
        time_the_placement_of_one_data_unit(allocator, PLACEMENT_REPETITIONS);
    for (device_index, scan) in scans.iter().enumerate() {
        emit(&format!(
            "name=per_device_lowest_slot_scan geometry={geometry} state={state} device={device_index} unit_area_slots={} lowest_slot={} scanned_candidates={} median_nanoseconds={} maximum_nanoseconds={} repetitions={SCAN_REPETITIONS}",
            allocator.devices[device_index].unit_area_slots(),
            scan.lowest_slot.map_or_else(|| "none".to_string(), |slot| slot.to_string()),
            scan.scanned_candidates,
            scan.median_nanoseconds,
            scan.maximum_nanoseconds,
        ));
    }
    emit(&format!(
        "name=per_device_lowest_slot_placement geometry={geometry} state={state} median_nanoseconds={placement_median} maximum_nanoseconds={placement_maximum} repetitions={PLACEMENT_REPETITIONS}"
    ));
    ScanStateReport {
        geometry,
        state,
        unit_area_slots: allocator.devices[0].unit_area_slots(),
        scans,
        placement_median_nanoseconds: placement_median,
    }
}

/// 登记 5.3 节：G窄、G默认、G宽 各三种状态——刚写完第一个文件、段外槽对填了一半、填了九成（这两种用空闲图标出来）。
/// 先拒那一道多出的挂钟（登记第六节「先拒那一道扫描」那一项）= 每块盘一次扫描，落点那一道已花的挂钟（登记第六节「落点那一道今天已花」那一项）= 一个数据单元的落点，两者之比逐状态报。
fn run_scan_growth(content: &mut ContentBuffer) -> Result<Vec<ScanStateReport>, String> {
    let mut reports = Vec::new();
    for geometry in [GEOMETRY_NARROW, GEOMETRY_DEFAULT, GEOMETRY_WIDE] {
        let mut pool =
            build_pool_after_the_first_file(geometry, CodeTwoTreeNodeCapacities::FromTheNodeFormat);
        reports.push(measure_scan_state(
            geometry.name,
            "after_the_first_file",
            &pool.allocator,
        ));
        for (state, percent) in [("half_marked", 50_u64), ("ninety_percent_marked", 90)] {
            let mut marked = pool.allocator.clone();
            mark_the_lowest_pairs_outside_the_clusters(&mut marked, percent);
            reports.push(measure_scan_state(geometry.name, state, &marked));
        }
        if geometry.name == GEOMETRY_NARROW.name {
            let pairs = data_unit_pairs_outside_the_clusters(&pool, 0);
            for (state, percent) in [("half_real", 50_u64), ("ninety_percent_real", 90)] {
                let units = usize::try_from(pairs * percent / 100).expect("单元数");
                let mut collector = FixedPointUpperBoundCollector::default();
                let mut fork = pool.fork();
                match run_one_write(
                    &mut collector,
                    "per_device_lowest_slot_real_fill",
                    geometry.name,
                    0,
                    &mut fork,
                    units,
                    CodeTwoTreeNodeCapacities::FromTheNodeFormat,
                    content,
                )? {
                    WriteOutcome::Published(_) => {
                        reports.push(measure_scan_state(geometry.name, state, &fork.allocator));
                    }
                    WriteOutcome::Refused(refusal) => {
                        emit(&format!(
                            "name=per_device_lowest_slot_real_fill_refused geometry={} state={state} units={units} refusal={:?}",
                            geometry.name, refusal
                        ));
                    }
                }
            }
        }
    }
    Ok(reports)
}

pub(super) fn run_per_device_lowest_slot(content: &mut ContentBuffer) -> Result<(), String> {
    let reports = run_scan_growth(content)?;
    for state in [
        "after_the_first_file",
        "half_marked",
        "ninety_percent_marked",
    ] {
        let by_geometry: Vec<&ScanStateReport> = reports
            .iter()
            .filter(|report| report.state == state)
            .collect();
        let medians: Vec<String> = by_geometry
            .iter()
            .map(|report| {
                let scan = report
                    .scans
                    .iter()
                    .map(|scan| scan.median_nanoseconds)
                    .max()
                    .unwrap_or(0);
                format!("{}:{}:{scan}", report.geometry, report.unit_area_slots)
            })
            .collect();
        emit(&format!(
            "name=per_device_lowest_slot_growth state={state} geometry_unit_area_slots_slowest_device_median_nanoseconds={}",
            medians.join(",")
        ));
        for report in by_geometry {
            let scan_both_devices: u128 = report
                .scans
                .iter()
                .map(|scan| scan.median_nanoseconds)
                .sum();
            emit(&format!(
                "name=per_device_lowest_slot_ratio geometry={} state={state} gate_extra_nanoseconds_both_devices={scan_both_devices} placement_one_data_unit_nanoseconds={} gate_extra_over_placement_percent={}",
                report.geometry,
                report.placement_median_nanoseconds,
                scan_both_devices * 100 / report.placement_median_nanoseconds.max(1),
            ));
        }
    }
    let (states, unequal_and_landing, control_hits) = run_unequal_landing_states(content)?;
    emit(&format!(
        "name=verdict_per_device_lowest_slot unequal_states_checked={states} unequal_lowest_slots_with_the_landing_check_passing={unequal_and_landing} positive_control_unequal_lowest_slots_built={control_hits}"
    ));
    Ok(())
}

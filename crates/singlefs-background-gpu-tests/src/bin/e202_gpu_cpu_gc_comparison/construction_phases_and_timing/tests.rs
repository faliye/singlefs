use super::*;

/// 进度间隔取 0：造池的每一个阶段都打出进度行（阶段名、已做/至多几步、已用秒数、常驻内存），各阶段的读数按造的先后一阶段一项，
/// 各阶段发成的版数加起来等于造池一共发成的；一步是写满一个文件的一次发布：顺序写那一半 P ÷ 32 步、交错那一半 P ÷ 16 步，
/// 删垫片文件 P ÷ 32 + 1 步（最小池 P = 64：2、4、3 步）。
#[test]
fn every_construction_phase_prints_a_progress_line_when_the_interval_is_zero() {
    let mut lines: Vec<String> = Vec::new();
    let mut collect_the_line = |line: &str| lines.push(line.to_string());
    let image = {
        let mut clock =
            ConstructionClock::new("进度".to_string(), Duration::ZERO, &mut collect_the_line);
        smallest_pool_on_the_clock(ChunkStorage::SharedWhenTheBytesAreEqual, &mut clock)
    };
    for phase in ConstructionPhase::EVERY_PHASE {
        assert!(
            lines
                .iter()
                .any(|line| line.contains(&format!(" phase={} ", phase.name()))),
            "阶段 {} 没有进度行",
            phase.name()
        );
    }
    assert!(lines
        .iter()
        .all(|line| line.starts_with("E202 造池进度 pool=进度 ")
            && line.contains(" elapsed_seconds=")
            && line.contains(" resident_bytes=")));
    let phases = &image.construction.phases;
    assert_eq!(
        phases.iter().map(|timing| timing.phase).collect::<Vec<_>>(),
        ConstructionPhase::EVERY_PHASE.to_vec()
    );
    assert_eq!(
        phases.iter().map(|timing| timing.publishes).sum::<u64>(),
        image.construction.publishes
    );
    let steps_of = |phase: ConstructionPhase| {
        phases
            .iter()
            .find(|timing| timing.phase == phase)
            .map(|timing| timing.steps)
    };
    assert_eq!(steps_of(ConstructionPhase::SequentialHalf), Some(2));
    assert_eq!(steps_of(ConstructionPhase::InterleavedHalf), Some(4));
    assert_eq!(
        steps_of(ConstructionPhase::UnlinkingTheSpacerFiles),
        Some(3)
    );
    assert_eq!(image.construction.spacer_files, 3);
}

#[test]
fn the_resident_bytes_are_read_off_the_resident_set_size_line() {
    assert_eq!(
        resident_bytes_of_the_status("Name:\tx\nVmRSS:\t   2048 kB\nThreads:\t1\n"),
        Some(2048 * 1024)
    );
    assert_eq!(resident_bytes_of_the_status("Name:\tx\n"), None);
    assert!(resident_bytes_of_this_process().is_some_and(|bytes| bytes > 0));
}

/// 常驻内存的峰读的是 `VmHWM` 那一行，不是 `VmRSS`；本进程读得到，且不小于先读到的常驻内存（峰只涨不落）。
#[test]
fn the_peak_resident_bytes_are_read_off_the_high_water_mark_line() {
    let status = "Name:\tx\nVmHWM:\t   4096 kB\nVmRSS:\t   2048 kB\nThreads:\t1\n";
    assert_eq!(peak_resident_bytes_of_the_status(status), Some(4096 * 1024));
    assert_eq!(resident_bytes_of_the_status(status), Some(2048 * 1024));
    assert_eq!(
        peak_resident_bytes_of_the_status("Name:\tx\nVmRSS:\t 1 kB\n"),
        None
    );
    let resident_first = resident_bytes_of_this_process().expect("本进程的常驻内存读得到");
    let peak_after = peak_resident_bytes_of_this_process().expect("本进程的常驻内存的峰读得到");
    assert!(
        peak_after >= resident_first,
        "{peak_after} < {resident_first}"
    );
}

/// 每个 `name=construction_phase` 行带这次挂载开过的聚簇段数与单元区段数（登记第十二节修订第 14 条）：新池新建文件那一阶段没有挂载，两样是 `none`；
/// 别的阶段单元区段数等于最小那块盘的单元区整段数，造数据挂载写完时开过的聚簇段不是 0（每次发布的提交内生块都落在聚簇段里）。
#[test]
fn every_construction_phase_line_carries_the_cluster_segments_of_its_mount() {
    let image = {
        let mut progress_dropped = |_line: &str| {};
        let mut clock = ConstructionClock::new(
            "聚簇段".to_string(),
            CONSTRUCTION_PROGRESS_INTERVAL,
            &mut progress_dropped,
        );
        smallest_pool_on_the_clock(ChunkStorage::SharedWhenTheBytesAreEqual, &mut clock)
    };
    let unit_area_segments =
        local_unit_area_slots_of_device(smallest_device_bytes_the_make_filesystem_accepts())
            / LOCAL_CLUSTER_SEGMENT_SLOTS;
    for timing in &image.construction.phases {
        let fields = timing.fields();
        match timing.phase {
            ConstructionPhase::MakeFilesystemAndTheFirstFile => {
                assert_eq!(timing.cluster_segments, None);
                assert!(
                    fields.ends_with(" cluster_segments=none unit_area_segments=none"),
                    "{fields}"
                );
            }
            ConstructionPhase::DataMount
            | ConstructionPhase::CreatingTheInodes
            | ConstructionPhase::SequentialHalf
            | ConstructionPhase::AlignmentSpacer
            | ConstructionPhase::InterleavedHalf
            | ConstructionPhase::FragmentMount
            | ConstructionPhase::UnlinkingTheSpacerFiles
            | ConstructionPhase::ReleasingTheSpacerSlots
            | ConstructionPhase::ReadingTheGeometry => {
                let cluster_segments = timing
                    .cluster_segments
                    .unwrap_or_else(|| panic!("{} 没记聚簇段", timing.phase.name()));
                assert_eq!(cluster_segments.unit_area_segments, unit_area_segments);
                assert!(
                    fields.ends_with(&format!(
                        " cluster_segments={} unit_area_segments={unit_area_segments}",
                        cluster_segments.opened_by_this_mount
                    )),
                    "{fields}"
                );
            }
        }
    }
    let interleaved_half = image
        .construction
        .phases
        .iter()
        .find(|timing| timing.phase == ConstructionPhase::InterleavedHalf)
        .and_then(|timing| timing.cluster_segments)
        .expect("交错那一半记了聚簇段");
    assert!(
        interleaved_half.opened_by_this_mount > 0,
        "{interleaved_half:?}"
    );
}

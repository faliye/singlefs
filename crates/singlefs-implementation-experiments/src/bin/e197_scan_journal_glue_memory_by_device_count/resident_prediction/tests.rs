//! `resident_prediction` 的单测。
use super::super::tests::{
    measure_in_a_process_of_its_own, measure_in_a_process_of_its_own_with_environment,
    MeasuredCell, SCANNING_ARMS,
};
use super::*;

/// 登记第七节 A6 手算的四个大小（点名项 64、位置条目 16、节点指针 56、记录 376），推算脚本 `predict.py` 用的那一组。
const REGISTERED_HAND_COUNTED_SIZES: StructureSizes = StructureSizes {
    location_entry: 16,
    named_unit: 64,
    node_pointer: 56,
    journal_record: 376,
};

/// 登记修订三第四节 `resident_prediction.py` 用的四个大小（记录 320、点名项 64、位置条目 16、节点指针 56），装置单测 `structure_sizes_match_the_field_by_field_layout_of_the_current_structs` 钉着当前结构体就是这一组。
const RESIDENT_DESIGN_SIZES: StructureSizes = StructureSizes {
    location_entry: 16,
    named_unit: 64,
    node_pointer: 56,
    journal_record: 320,
};

#[test]
fn the_glibc_chunk_sizes_the_resident_prediction_uses_are_pinned() {
    for (request, chunk) in [
        (1u64, 32u64),
        (24, 32),
        (64, 80),
        (280, 288),
        (544, 560),
        (3712, 3728),
        (4096, 4112),
        (64 * 64, 4112),
    ] {
        assert_eq!(glibc_chunk_bytes(request), chunk, "请求 {request} 字节");
    }
}

#[test]
fn device_holds_the_ring_only_when_the_ring_is_at_most_one_quarter_of_it() {
    // 登记第七节 A4：4 GiB 装得下 768 MiB 的环（4 × 环长 = 3 221 225 472 ≤ 4 GiB）。
    assert_eq!(4 * JOURNAL_RING_DEFAULT_BYTES, 3_221_225_472);
    assert!(device_holds_the_ring(
        TEST_IMAGE_DEFAULT_BYTES,
        JOURNAL_RING_DEFAULT_BYTES
    ));
    assert!(device_holds_the_ring(
        3_221_225_472,
        JOURNAL_RING_DEFAULT_BYTES
    ));
    assert!(!device_holds_the_ring(
        3_221_225_471,
        JOURNAL_RING_DEFAULT_BYTES
    ));
    // 2 GiB 的盘装不下 768 MiB 的环（÷ 4 = 512 MiB），1 GiB 的更装不下。
    assert!(!device_holds_the_ring(2 << 30, JOURNAL_RING_DEFAULT_BYTES));
    assert!(!device_holds_the_ring(1 << 30, JOURNAL_RING_DEFAULT_BYTES));
    assert_eq!(
        JOURNAL_RING_START_SLOT * SLOT_BYTES + JOURNAL_RING_DEFAULT_BYTES,
        822_083_584
    );
}

#[test]
fn vec_capacity_after_pushes_matches_the_registered_values_and_a_real_vec() {
    for (count, capacity) in [
        (98_304u64, 131_072u64),
        (196_608, 262_144),
        (294_912, 524_288),
        (393_216, 524_288),
        (589_824, 1_048_576),
        (786_432, 1_048_576),
    ] {
        assert_eq!(vec_capacity_after_pushes(count), capacity);
    }
    for count in [1usize, 4, 5, 8, 9, 1000, 4096, 5000] {
        let mut vector: Vec<[u8; 320]> = Vec::new();
        for _ in 0..count {
            vector.push([0u8; 320]);
        }
        assert_eq!(
            u64::try_from(vector.capacity()).expect("容量装得进 u64"),
            vec_capacity_after_pushes(u64::try_from(count).expect("条数装得进 u64")),
            "逐条 push 到 {count} 条时 Vec 的容量"
        );
    }
}

#[test]
fn structure_sizes_match_the_field_by_field_layout_of_the_current_structs() {
    let sizes = structure_sizes();
    println!(
        "size_of：LocationEntry={} NamedUnit={} NodePointer={} JournalRecord={}（登记第七节 A6 手算：16、64、56、{}）",
        sizes.location_entry,
        sizes.named_unit,
        sizes.node_pointer,
        sizes.journal_record,
        REGISTERED_HAND_COUNTED_SIZES.journal_record
    );
    // 位置条目 4 + 8 + 4；点名项 2 × 16 + 1 + 8 + 8 + 10 = 59 按 8 对齐；节点指针 16 + 2 × 16 + 4 + 4；
    // 记录：结构体里是四条节点指针（树表、映射根、实例表、分配记录树根）4 × 56 + 七个 8 字节 + 三个 4 字节 + 四个一字节量 + 一个 Vec 24 = 320；登记手算写 376（按五条节点指针数的），
    // 逐字段重数是 320，以这条断言跑出的 size_of 为准，对不上照实报、不改登记。
    assert_eq!(
        sizes.location_entry,
        REGISTERED_HAND_COUNTED_SIZES.location_entry
    );
    assert_eq!(sizes.named_unit, REGISTERED_HAND_COUNTED_SIZES.named_unit);
    assert_eq!(
        sizes.node_pointer,
        REGISTERED_HAND_COUNTED_SIZES.node_pointer
    );
    assert_eq!(sizes.journal_record, 4 * 56 + 7 * 8 + 3 * 4 + 4 + 24);
}

#[test]
fn the_prediction_reproduces_the_resident_design_numbers_for_the_default_ring() {
    // 登记修订三第四节 `resident_prediction.py` 输出的 `resident` 各行（记录 320、点名项 64）：满槽 K = 64、半满与 K = 1 各抽几格，单位 GiB，容差 0.002。
    let gibibytes = |arm: Arm, devices: u64, filled: u64, named: u64| {
        predicted_resident_peak_bytes(arm, devices, filled, named, RESIDENT_DESIGN_SIZES)
            / f64::from(1u32 << 30)
    };
    let cases: [(Arm, u64, u64, u64, f64); 12] = [
        (Arm::Baseline, 1, RING_SLOTS, 64, 0.937),
        (Arm::Baseline, 2, RING_SLOTS, 64, 1.751),
        (Arm::Baseline, 3, RING_SLOTS, 64, 2.566),
        (Arm::Baseline, 4, RING_SLOTS, 64, 3.380),
        (Arm::BaselinePc, 4, RING_SLOTS, 64, 4.133),
        (Arm::Split, 2, RING_SLOTS, 64, 1.693),
        (Arm::Split, 4, RING_SLOTS, 64, 1.699),
        (Arm::SplitPc, 4, RING_SLOTS, 64, 3.380),
        (Arm::Baseline, 4, RING_SLOTS / 2, 64, 1.690),
        (Arm::Split, 4, RING_SLOTS / 2, 64, 0.849),
        (Arm::Baseline, 4, RING_SLOTS, 1, 0.427),
        (Arm::Split, 4, RING_SLOTS, 1, 0.222),
    ];
    for (arm, devices, filled, named, expected) in cases {
        let predicted = gibibytes(arm, devices, filled, named);
        assert!(
            (predicted - expected).abs() < 0.002,
            "{} {devices} 盘 {filled} 槽 K={named}：推 {predicted:.4} GiB，登记 {expected}",
            arm.name()
        );
    }
    // 经读缓存的两臂：胶水那一截同基线，另加缓存那一截（登记修订三第四节 `read_stage_cache` 行）。
    for (devices, named, expected) in [(4u64, 64u64, 6.460), (4, 1, 3.507)] {
        let predicted = gibibytes(Arm::ReadStageCache, devices, RING_SLOTS, named)
            + predicted_cache_bytes(devices * RING_SLOTS) / f64::from(1u32 << 30);
        assert!(
            (predicted - expected).abs() < 0.002,
            "read_stage_cache {devices} 盘 K={named}：推 {predicted:.4} GiB，登记 {expected}"
        );
    }
    assert_eq!(
        predicted_resident_peak_bytes(Arm::Empty, 4, RING_SLOTS, 64, RESIDENT_DESIGN_SIZES),
        0.0
    );
}

#[test]
fn the_split_pc_and_baseline_resident_predictions_are_the_same_and_split_equals_baseline_on_one_device(
) {
    // 登记修订三第四节：常驻页口径下 split_pc 与 baseline 同式；split 在 1 盘上同 baseline（A9 的依据）。
    for named in [64u64, 1] {
        for devices in 1..=4u64 {
            assert_eq!(
                predicted_resident_peak_bytes(
                    Arm::SplitPc,
                    devices,
                    RING_SLOTS,
                    named,
                    RESIDENT_DESIGN_SIZES
                ),
                predicted_resident_peak_bytes(
                    Arm::Baseline,
                    devices,
                    RING_SLOTS,
                    named,
                    RESIDENT_DESIGN_SIZES
                ),
                "{devices} 盘 K={named}"
            );
        }
        assert_eq!(
            predicted_resident_peak_bytes(Arm::Split, 1, RING_SLOTS, named, RESIDENT_DESIGN_SIZES),
            predicted_resident_peak_bytes(
                Arm::Baseline,
                1,
                RING_SLOTS,
                named,
                RESIDENT_DESIGN_SIZES
            )
        );
    }
}

#[test]
fn the_trajectory_expectation_reproduces_the_registered_last_device_end_numbers() {
    // 登记修订三第四节 `trajectory_last_device_end` 各行，单位 MiB，容差 0.05。
    let mebibytes = |devices: u64, filled: u64, fill: Fill, named: u64| {
        predicted_resident_at_the_last_device_end_bytes(
            devices,
            filled,
            fill,
            RING_SLOTS,
            named,
            RESIDENT_DESIGN_SIZES,
        ) / f64::from(1u32 << 20)
    };
    let cases: [(u64, u64, Fill, u64, f64); 7] = [
        (1, RING_SLOTS, Fill::Full, 64, 832.5),
        (4, RING_SLOTS, Fill::Full, 64, 3325.5),
        (1, RING_SLOTS / 2, Fill::Half, 64, 417.0),
        (4, RING_SLOTS / 2, Fill::Half, 64, 1663.5),
        (1, RING_SLOTS, Fill::Full, 1, 76.5),
        (4, RING_SLOTS, Fill::Full, 1, 301.5),
        (2, RING_SLOTS, Fill::Full, 64, 1663.5),
    ];
    for (devices, filled, fill, named, expected) in cases {
        let predicted = mebibytes(devices, filled, fill, named);
        assert!(
            (predicted - expected).abs() < 0.05,
            "{devices} 盘 {filled} 槽 K={named}：推 {predicted:.2} MiB，登记 {expected}"
        );
    }
}

/// 一个扫环格量到的 H 与登记式子推的 R（登记修订三第七节 A7）。
struct ScanningCellReading {
    label: String,
    arm: Arm,
    devices: u32,
    named: usize,
    ring_slots: u64,
    cell: MeasuredCell,
    predicted: f64,
}

impl ScanningCellReading {
    /// H ÷ R − 1。
    fn deviation(&self) -> f64 {
        self.cell.peak() / self.predicted - 1.0
    }

    /// 一格一行：量到的 H、推的 R、比值、差、轨迹、期末值与内核两个计数的原始读数。
    fn row(&self) -> String {
        format!(
            "[{}] {} {}盘 K={} 环={}槽：H={} R={:.0} H÷R−1={:+.4} H−R={:+.0} 轨迹={} 期末={} rss起点KiB={} VmHWM_KiB={}",
            self.label,
            self.arm.name(),
            self.devices,
            self.named,
            self.ring_slots,
            self.cell.number("resident_peak_over_start_bytes"),
            self.predicted,
            self.deviation(),
            self.cell.peak() - self.predicted,
            self.cell.fields["resident_at_each_device_end_bytes"],
            self.cell.number("resident_after_return_bytes"),
            self.cell.number("vm_rss_at_start_kib"),
            self.cell.number("vm_hwm_kib"),
        )
    }
}

/// 四个胶水臂在满槽、给定几何与盘数上各量一格（每格一个子进程），带上登记式子推的 R。
fn read_the_scanning_cells(
    label: &str,
    geometries: &[(usize, u64)],
    device_counts: &[u32],
    extra_environment: &[(&str, &str)],
) -> Vec<ScanningCellReading> {
    let sizes = structure_sizes();
    let mut readings = Vec::new();
    for &(named, ring_slots) in geometries {
        for arm in SCANNING_ARMS {
            for &devices in device_counts {
                let cell = measure_in_a_process_of_its_own_with_environment(
                    arm,
                    devices,
                    Fill::Full,
                    named,
                    ring_slots,
                    extra_environment,
                );
                let predicted = predicted_resident_peak_bytes(
                    arm,
                    u64::from(devices),
                    ring_slots,
                    u64::try_from(named).expect("点名项数装得进 u64"),
                    sizes,
                );
                readings.push(ScanningCellReading {
                    label: label.to_string(),
                    arm,
                    devices,
                    named,
                    ring_slots,
                    cell,
                    predicted,
                });
            }
        }
    }
    readings
}

/// 单测里 A7 取的几何：（每条记录点名项数 K，环槽数）。
const RESIDENT_PREDICTION_UNIT_GEOMETRIES: [(usize, u64); 2] = [(64, 8192), (1, 32768)];

/// 读数表的表头：这一趟跑在哪种内核、哪几个在线的处理器上（H 与内核的页计数、透明大页有关，两台各不相同时靠它认）。
/// 处理器只记进表头认机器、不定并行度，与另两样一样从内核的文件里原样读（`/sys/devices/system/cpu/online`，如 `0-31`）。
fn kernel_facts_line() -> String {
    let text_of = |path: &str| {
        std::fs::read_to_string(path)
            .map_or_else(|_| "读不出".to_string(), |text| text.trim().to_string())
    };
    format!(
        "环境：透明大页={} 内核={} 在线处理器={}",
        text_of("/sys/kernel/mm/transparent_hugepage/enabled"),
        text_of("/proc/sys/kernel/osrelease"),
        text_of("/sys/devices/system/cpu/online")
    )
}

/// 单测里量 A7 的子进程另带的环境：把 glibc 的 mmap 门槛钉在 128 KiB。
/// 默认环境下 glibc 每放掉一块 mmap 出来的大块就把门槛抬到那一块的大小（上限 32 MiB），
/// 之后中等大小的 Vec 走堆、翻倍留洞，留下的洞随臂、盘数、环槽数没有稳定的形状（登记修订四）；
/// 钉住门槛，登记 A7 的式子（大 Vec 按 `mremap` 长、没写过的页不常驻）才是单测要核的那个口径。
const FIXED_MMAP_THRESHOLD_ENVIRONMENT: [(&str, &str); 1] = [("MALLOC_MMAP_THRESHOLD_", "131072")];

#[test]
fn the_resident_peak_of_every_scanning_arm_stays_within_the_prediction() {
    // 登记修订三第七节 A7：H 对常驻页推算 R 的容差 ±5%，登记没有给 K = 1 单开的容差
    // （这里取登记的 ±5%，不对 K = 1 另放宽）。
    // 每一格都先量完、打出来，再一并判：第一处出界不停，别的格的读数同样要看。
    // 子进程固定 glibc 的 mmap 门槛（登记修订四）：默认环境下 split_pc 3 盘 K = 1 量到的 H 比 R 高约 29%，
    // 固定之后同一格低约 0.9%，二十四格最大偏差约 1.0%。
    let readings = read_the_scanning_cells(
        "mmap门槛固定131072",
        &RESIDENT_PREDICTION_UNIT_GEOMETRIES,
        &[1, 2, 3],
        &FIXED_MMAP_THRESHOLD_ENVIRONMENT,
    );
    assert_eq!(
        readings.len(),
        RESIDENT_PREDICTION_UNIT_GEOMETRIES.len() * SCANNING_ARMS.len() * 3
    );
    let table = std::iter::once(kernel_facts_line())
        .chain(readings.iter().map(ScanningCellReading::row))
        .collect::<Vec<_>>()
        .join("\n");
    println!("{table}");
    let outside: Vec<String> = readings
        .iter()
        .filter(|reading| reading.deviation().abs() > PREDICTION_TOLERANCE)
        .map(ScanningCellReading::row)
        .collect();
    assert!(
        outside.is_empty(),
        "A7 的 ±5% 之外 {} 格（共 {} 格）：\n{}\n全部格的读数：\n{table}",
        outside.len(),
        readings.len(),
        outside.join("\n")
    );
}

#[test]
fn on_one_device_split_and_baseline_peak_within_one_percent_of_each_other() {
    // 登记修订三第七节 A9：1 盘上 split 与 baseline 做的是同一串分配，H 之差 ≤ baseline 那一格 H 的 1%。取环大一点，1% 才比内核计数的攒批大。
    for (named, ring_slots) in [(64usize, 32768u64), (1, 131072)] {
        let baseline =
            measure_in_a_process_of_its_own(Arm::Baseline, 1, Fill::Full, named, ring_slots);
        let split = measure_in_a_process_of_its_own(Arm::Split, 1, Fill::Full, named, ring_slots);
        let difference = (split.peak() - baseline.peak()).abs() / baseline.peak();
        assert!(
            difference <= 0.01,
            "K={named}：split {} 对 baseline {}，差 {difference}",
            split.peak(),
            baseline.peak()
        );
        assert_eq!(
            baseline.fields["table_digest"],
            split.fields["table_digest"]
        );
    }
}

#[test]
fn the_trajectory_ends_at_the_registered_resident_pages_and_the_peak_is_at_least_the_largest_trajectory_value(
) {
    // 登记修订三第八节：baseline 与 baseline_pc 最后一盘那个轨迹值对「读到那一刻的常驻页」落在 ±3%；H ≥ 这一格轨迹里最大的那个数。
    let sizes = structure_sizes();
    for (named, ring_slots) in [(64usize, 8192u64), (1, 65536)] {
        for arm in [Arm::Baseline, Arm::BaselinePc] {
            for devices in [1u32, 2, 3] {
                let cell =
                    measure_in_a_process_of_its_own(arm, devices, Fill::Full, named, ring_slots);
                let ends = cell.numbers("resident_at_each_device_end_bytes");
                let context = format!("{} {devices} 盘 K={named}", arm.name());
                assert_eq!(
                    ends.len(),
                    usize::try_from(devices).expect("盘数"),
                    "{context} 轨迹点数"
                );
                let last = *ends.last().expect("至少一块盘");
                let predicted = predicted_resident_at_the_last_device_end_bytes(
                    u64::from(devices),
                    ring_slots,
                    Fill::Full,
                    ring_slots,
                    u64::try_from(named).expect("点名项数装得进 u64"),
                    sizes,
                );
                assert!(
                    (last as f64 / predicted - 1.0).abs() <= 0.03,
                    "{context}：最后一盘的轨迹值 {last}，推 {predicted:.0}"
                );
                let largest = *ends.iter().max().expect("至少一块盘");
                assert!(
                    cell.number("resident_peak_over_start_bytes") >= largest,
                    "{context}：H {} 小于轨迹最大值 {largest}",
                    cell.number("resident_peak_over_start_bytes")
                );
            }
        }
    }
}

use super::*;

pub(super) const SMALL_RING_SLOTS: u64 = 4096;
pub(super) const SCANNING_ARMS: [Arm; 4] =
    [Arm::Baseline, Arm::BaselinePc, Arm::Split, Arm::SplitPc];
/// 登记修订一的两条读缓存臂（C881 那一臂与它的阳性对照）。
pub(super) const READ_STAGE_CACHE_ARMS: [Arm; 2] =
    [Arm::ReadStageCache, Arm::ReadStageCacheKeepsZerosPc];

pub(super) fn small_cell(
    arm: Arm,
    devices: u32,
    fill: Fill,
    named: usize,
    ring_slots: u64,
) -> CellOutcome {
    measure_one_cell_in_one_process(
        CellSpecification {
            arm,
            devices,
            fill,
            named,
            repeat: 1,
        },
        ring_slots,
    )
}

// ----------------------------------------------------------------------------------------
// H 是整个进程的数：libtest 并行跑的别的用例会串进来，所以凡要读 H 的单测，每一格各起一个只跑一条用例的子进程（测试二进制自己，`--exact` 点名下面那条）量。
// 一个子进程只量一格：同一个进程里连着量几格，前一格放掉的堆页还常驻，后一格的 H 会偏小。
// ----------------------------------------------------------------------------------------

/// 子进程里要量的那一格：值是 `臂,盘数,填满档,点名项数,环槽数`；环境里没有它的用例（不是子进程）什么都不量。
const CELL_REQUEST_VARIABLE: &str = "E197_TEST_CELL_REQUEST";
const CELL_CHILD_TEST: &str =
    "tests::the_child_process_measures_the_one_cell_its_environment_names";

#[test]
fn the_child_process_measures_the_one_cell_its_environment_names() {
    let Ok(request) = std::env::var(CELL_REQUEST_VARIABLE) else {
        return;
    };
    let words: Vec<&str> = request.split(',').collect();
    let [arm, devices, fill, named, ring_slots] = words.as_slice() else {
        panic!("{CELL_REQUEST_VARIABLE} 要五段（臂,盘数,填满档,点名项数,环槽数）：{request}");
    };
    let specification = CellSpecification {
        arm: Arm::from_name(arm).expect("认得的臂"),
        devices: devices.parse().expect("盘数"),
        fill: Fill::from_name(fill).expect("认得的填满档"),
        named: named.parse().expect("点名项数"),
        repeat: 1,
    };
    let outcome =
        measure_one_cell_in_one_process(specification, ring_slots.parse().expect("环槽数"));
    println!("{}", outcome.result_line(structure_sizes()));
}

/// 子进程交回的一格结果行的字段。
pub(super) struct MeasuredCell {
    pub(super) fields: BTreeMap<String, String>,
}

impl MeasuredCell {
    pub(super) fn number(&self, name: &str) -> i64 {
        self.fields[name]
            .parse()
            .unwrap_or_else(|_| panic!("字段 {name} 不是整数：{}", self.fields[name]))
    }

    pub(super) fn numbers(&self, name: &str) -> Vec<i64> {
        self.fields[name]
            .split(',')
            .map(|word| word.parse().expect("轨迹的每一项是整数"))
            .collect()
    }

    /// H（字节）。
    pub(super) fn peak(&self) -> f64 {
        self.number("resident_peak_over_start_bytes") as f64
    }
}

pub(super) fn measure_in_a_process_of_its_own(
    arm: Arm,
    devices: u32,
    fill: Fill,
    named: usize,
    ring_slots: u64,
) -> MeasuredCell {
    measure_in_a_process_of_its_own_with_environment(arm, devices, fill, named, ring_slots, &[])
}

/// 同上，子进程另带 `extra_environment` 里的环境变量（A7 用它固定 glibc 的 mmap 门槛）。
pub(super) fn measure_in_a_process_of_its_own_with_environment(
    arm: Arm,
    devices: u32,
    fill: Fill,
    named: usize,
    ring_slots: u64,
    extra_environment: &[(&str, &str)],
) -> MeasuredCell {
    let output = std::process::Command::new(std::env::current_exe().expect("找得到测试二进制自己"))
        .args([
            "--exact",
            CELL_CHILD_TEST,
            "--nocapture",
            "--test-threads=1",
        ])
        .envs(extra_environment.iter().copied())
        .env(
            CELL_REQUEST_VARIABLE,
            format!(
                "{},{devices},{},{named},{ring_slots}",
                arm.name(),
                fill.name()
            ),
        )
        .output()
        .expect("起得了子进程");
    let standard_output = String::from_utf8_lossy(&output.stdout).to_string();
    let standard_error = String::from_utf8_lossy(&output.stderr).to_string();
    assert!(
        output.status.success(),
        "子进程量 {} {devices} 盘 {} K={named} 没成：{standard_output}{standard_error}",
        arm.name(),
        fill.name()
    );
    assert!(
        standard_output.contains("running 1 test"),
        "--exact 没点中那一条子进程用例：{standard_output}"
    );
    let line = standard_output
        .lines()
        // 测试框架先打「test <名> ... 」不换行，结果行接在同一行后面
        .find_map(|line| line.find("E7RESULT name=e197 ").map(|start| &line[start..]))
        .unwrap_or_else(|| panic!("子进程没打结果行：{standard_output}{standard_error}"));
    let fields = line
        .strip_prefix("E7RESULT ")
        .expect("以 E7RESULT 起头")
        .split_whitespace()
        .filter_map(|word| word.split_once('='))
        .map(|(name, value)| (name.to_string(), value.to_string()))
        .collect();
    MeasuredCell { fields }
}

#[test]
fn the_default_ring_geometry_numbers_are_pinned_for_one_to_four_devices() {
    let expected_reads: Vec<u64> = (1..=4u64).map(|devices| devices * RING_SLOTS).collect();
    assert_eq!(expected_reads, [196_608, 393_216, 589_824, 786_432]);
    assert_eq!(Arm::from_name("split_pc"), Some(Arm::SplitPc));
    assert_eq!(Arm::from_name("split-pc"), None);
    assert!(Arm::Baseline.scans() && !Arm::Empty.scans() && !Arm::MeasurementPc.scans());
    assert_eq!(
        Arm::from_name("read_stage_cache"),
        Some(Arm::ReadStageCache)
    );
    assert_eq!(
        Arm::from_name("read_stage_cache_keeps_zeros_pc"),
        Some(Arm::ReadStageCacheKeepsZerosPc)
    );
    assert!(READ_STAGE_CACHE_ARMS
        .iter()
        .all(|arm| arm.scans() && arm.reads_through_the_cache()));
    assert!(SCANNING_ARMS
        .iter()
        .all(|arm| !arm.reads_through_the_cache()));
    assert_eq!(Fill::from_name("half"), Some(Fill::Half));
    assert_eq!(Fill::from_name("full"), Some(Fill::Full));
}

#[test]
fn the_arguments_parse_into_a_cell_and_the_wrong_ones_are_refused() {
    let good = [
        "--arm",
        "split",
        "--devices",
        "4",
        "--fill",
        "half",
        "--named",
        "64",
        "--repeat",
        "5",
    ];
    assert_eq!(
        parse_the_arguments(&good),
        Ok(Request::Cell(CellSpecification {
            arm: Arm::Split,
            devices: 4,
            fill: Fill::Half,
            named: 64,
            repeat: 5,
        }))
    );
    let compare = [
        "--arm",
        "split",
        "--devices",
        "4",
        "--fill",
        "half",
        "--named",
        "64",
        "--repeat",
        "5",
        "--compare",
    ];
    assert_eq!(
        parse_the_arguments(&compare),
        Ok(Request::Compare(CellSpecification {
            arm: Arm::Split,
            devices: 4,
            fill: Fill::Half,
            named: 64,
            repeat: 5,
        }))
    );
    assert_eq!(
        parse_the_arguments(&["--diff-set"]),
        Ok(Request::DifferentialSet)
    );
    assert_eq!(parse_the_arguments(&["--checks"]), Ok(Request::Checks));
    let refused: [&[&str]; 8] = [
        &[],
        &["--arm", "split"],
        &[
            "--arm",
            "nope",
            "--devices",
            "1",
            "--fill",
            "full",
            "--named",
            "1",
            "--repeat",
            "1",
        ],
        &[
            "--arm",
            "split",
            "--devices",
            "0",
            "--fill",
            "full",
            "--named",
            "1",
            "--repeat",
            "1",
        ],
        &[
            "--arm",
            "split",
            "--devices",
            "1",
            "--fill",
            "full",
            "--named",
            "65",
            "--repeat",
            "1",
        ],
        &[
            "--arm",
            "split",
            "--devices",
            "1",
            "--fill",
            "full",
            "--named",
            "1",
            "--repeat",
            "11",
        ],
        &[
            "--arm",
            "split",
            "--devices",
            "1",
            "--fill",
            "full",
            "--named",
            "1",
            "--repeat",
            "0",
        ],
        &[
            "--arm",
            "split",
            "--devices",
            "1",
            "--fill",
            "full",
            "--named",
            "1",
            "--bogus",
            "1",
        ],
    ];
    for words in refused {
        assert!(parse_the_arguments(words).is_err(), "该拒：{words:?}");
    }
}

#[test]
fn the_read_stage_cache_holds_one_entry_per_nonzero_slot_per_device_and_its_control_holds_every_slot(
) {
    let ring_slots = 256;
    for devices in 1..=3u32 {
        for (fill, filled) in [(Fill::Full, 256u64), (Fill::Half, 128)] {
            let arm = small_cell(Arm::ReadStageCache, devices, fill, 2, ring_slots);
            let control = small_cell(
                Arm::ReadStageCacheKeepsZerosPc,
                devices,
                fill,
                2,
                ring_slots,
            );
            let device_count = u64::from(devices);
            let context = format!("{devices} 盘 {}", fill.name());
            assert_eq!(
                arm.cache_entries,
                Some(device_count * filled),
                "{context} 读缓存的项数（A10）"
            );
            assert_eq!(
                arm.cache_value_bytes,
                Some(device_count * filled * JOURNAL_RECORD_BYTES),
                "{context} 读缓存值的字节和（A10）"
            );
            assert_eq!(
                control.cache_entries,
                Some(device_count * ring_slots),
                "{context} 阳性对照的项数"
            );
            assert_eq!(
                control.cache_value_bytes,
                Some(device_count * ring_slots * JOURNAL_RECORD_BYTES),
                "{context} 阳性对照值的字节和"
            );
            assert!(arm.anchors_hold() && control.anchors_hold(), "{context}");
            assert_eq!(arm.table_digest, control.table_digest, "{context}");
        }
    }
}

#[test]
fn the_read_stage_cache_raises_the_peak_and_the_end_value_above_the_baseline_by_the_cache() {
    let named = 3;
    for devices in [1u32, 2] {
        let baseline = measure_in_a_process_of_its_own(
            Arm::Baseline,
            devices,
            Fill::Full,
            named,
            SMALL_RING_SLOTS,
        );
        let cached = measure_in_a_process_of_its_own(
            Arm::ReadStageCache,
            devices,
            Fill::Full,
            named,
            SMALL_RING_SLOTS,
        );
        let predicted = predicted_cache_bytes(u64::from(devices) * SMALL_RING_SLOTS);
        let peak_extra = cached.peak() - baseline.peak();
        let end_extra = (cached.number("resident_after_return_bytes")
            - baseline.number("resident_after_return_bytes")) as f64;
        assert!(
            (peak_extra / predicted - 1.0).abs() <= PREDICTION_TOLERANCE,
            "{devices} 盘：H 多出 {peak_extra}，推 {predicted:.0}（A11）"
        );
        assert!(
            (end_extra / predicted - 1.0).abs() <= PREDICTION_TOLERANCE,
            "{devices} 盘：期末值多出 {end_extra}，推 {predicted:.0}（A12）"
        );
    }
}

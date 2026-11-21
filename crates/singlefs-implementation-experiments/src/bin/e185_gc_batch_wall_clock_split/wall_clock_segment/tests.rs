use super::*;

#[test]
fn wall_clock_configuration_line_prints_no_part_of_the_image_directory() {
    let private_name = format!(
        "e185-configuration-line-private-directory-{}",
        std::process::id()
    );
    // 守卫认领配置行那一步建的目录：用例做完、断言失败、panic 都删掉它
    let directory = singlefs_verification::temporary_files::TemporaryPathRemovedOnDrop::claiming(
        std::env::temp_dir().join(&private_name),
    );
    let line = wall_clock_configuration_line(
        &directory,
        &every_sampling_point(),
        &preparation_settings_with(31, 28 << 30),
    );
    let device_number = std::fs::metadata(&directory)
        .expect("配置行那一步建了这个目录")
        .dev();
    let directory_text = directory.display().to_string();
    assert!(
        !line.contains(&directory_text),
        "配置行不打传入的目录 {directory_text}：{line}"
    );
    assert!(
        !line.contains(&private_name),
        "配置行连目录名的末段也不打：{line}"
    );
    assert!(
        !line.contains("image_directory="),
        "配置行没有 image_directory 这一栏：{line}"
    );
    assert!(
        line.contains(&format!(" image_device={device_number} ")),
        "配置行照打目录所在的设备号 {device_number}：{line}"
    );
    assert!(
        line.contains(" image_directory_is_cleared_at_boot="),
        "配置行打目录在不在开机清空的地方：{line}"
    );
    assert!(
        line.ends_with(" preparation_threads=31 preparation_memory_cap_bytes=30064771072 preparation_memory_cap_source=environment_variable preparation_units=24 preparation_units_started_at_once_by_the_estimates=15 preparation_plan_by_the_estimates=in_batches"),
        "配置行打准备阶段的线程数、内存上限与它从哪取、一共几份准备、按估算头一次派几份与分不分批：{line}"
    );
}

#[test]
fn device_numbers_split_into_the_major_and_minor_that_mountinfo_prints() {
    // 三个设备号由 makedev 的编码另算（python 的 os.major / os.minor 核过）：259:2、0:300、4096:0、0:65536
    assert_eq!(major_and_minor_of_the_device_number(66_306), (259, 2));
    assert_eq!(major_and_minor_of_the_device_number(1_048_620), (0, 300));
    assert_eq!(
        major_and_minor_of_the_device_number(17_592_186_044_416),
        (4096, 0)
    );
    assert_eq!(
        major_and_minor_of_the_device_number(268_435_456),
        (0, 65_536)
    );
}

#[test]
fn filesystem_type_comes_from_the_mountinfo_line_of_that_device() {
    let mount_information =
        "26 32 0:23 / /sys rw,nosuid,nodev,noexec,relatime shared:7 - sysfs sysfs rw\n\
                                 32 1 259:2 / / rw,relatime shared:1 - ext4 /dev/sample-disk rw\n\
                                 40 32 0:300 / /tmp rw,nosuid,nodev - tmpfs tmpfs rw,size=1024k\n";
    assert_eq!(
        filesystem_type_of_the_device(66_306, mount_information).as_deref(),
        Some("ext4")
    );
    assert_eq!(
        filesystem_type_of_the_device(1_048_620, mount_information).as_deref(),
        Some("tmpfs")
    );
    assert_eq!(
        filesystem_type_of_the_device(66_307, mount_information),
        None
    );
}

#[test]
fn image_directories_under_the_temporary_directory_or_on_a_memory_file_system_count_as_cleared_at_boot(
) {
    assert_eq!(
        is_cleared_at_boot(Some("ext4"), Some(Path::new("/tmp/e185-images"))),
        Some(true)
    );
    assert_eq!(
        is_cleared_at_boot(Some("tmpfs"), Some(Path::new("/srv/e185-images"))),
        Some(true)
    );
    assert_eq!(
        is_cleared_at_boot(Some("ext4"), Some(Path::new("/srv/e185-images"))),
        Some(false)
    );
    assert_eq!(
        is_cleared_at_boot(Some("ext4"), Some(Path::new("/tmpfiles/e185-images"))),
        Some(false),
        "按路径分量比，/tmpfiles 不在 /tmp 下"
    );
    assert_eq!(
        is_cleared_at_boot(None, Some(Path::new("/srv/e185-images"))),
        None
    );
    assert_eq!(
        is_cleared_at_boot(None, Some(Path::new("/tmp"))),
        Some(true)
    );
}

/// 登记第十二节 12.9：对照点每一轮在 K = 8 那一格里，每个设备臂挨着量基线、盘臂对照、CPU 臂对照各一次，次序按轮号轮换；
/// 别的格与别的取样点只量基线。
#[test]
fn row_one_controls_interleave_the_three_injections_inside_every_round() {
    let none = WriteInjection::Nothing;
    let sleep = WriteInjection::SleepTwoMilliseconds;
    let spin = WriteInjection::SpinTwoMillisecondsOfThreadCpu;
    assert_eq!(
        (1..=ROUNDS)
            .map(row_one_control_injections_of_the_round)
            .collect::<Vec<_>>(),
        vec![
            [none, sleep, spin],
            [sleep, spin, none],
            [spin, none, sleep],
            [none, sleep, spin],
            [sleep, spin, none],
        ]
    );
    for round in 1..=ROUNDS {
        let plan = measurements_of_the_round(CONTROL_POINT, round);
        assert_eq!(
            plan.len(),
            2 + 2 * 3 + 2,
            "第 {round} 轮：K = 4、16 各两臂，K = 8 两臂各三种注入"
        );
        for arm in DeviceArm::ALL {
            let positions: Vec<usize> = plan
                .iter()
                .enumerate()
                .filter(|(_position, (batch_units, planned_arm, _injection))| {
                    *batch_units == CONTROL_BATCH_UNITS && *planned_arm == arm
                })
                .map(|(position, _measurement)| position)
                .collect();
            assert_eq!(
                positions.len(),
                3,
                "第 {round} 轮 {}：三种注入各一次",
                arm.label()
            );
            assert_eq!(
                positions[2],
                positions[0] + 2,
                "第 {round} 轮 {}：三种注入挨着",
                arm.label()
            );
            assert_eq!(
                positions
                    .iter()
                    .map(|position| plan[*position].2)
                    .collect::<Vec<_>>(),
                row_one_control_injections_of_the_round(round).to_vec()
            );
        }
        assert!(plan
            .iter()
            .filter(|(batch_units, _arm, _injection)| *batch_units != CONTROL_BATCH_UNITS)
            .all(|(_batch_units, _arm, injection)| *injection == none));
    }
    for point in every_sampling_point()
        .into_iter()
        .filter(|point| *point != CONTROL_POINT)
    {
        let plan = measurements_of_the_round(point, 1);
        assert_eq!(plan.len(), BATCH_SIZES.len() * DeviceArm::ALL.len());
        assert!(plan
            .iter()
            .all(|(_batch_units, _arm, injection)| *injection == none));
    }
}

/// N14：一轮里第 1 次只因校准作废、第 2 次有效（批 CPU 差一倍），照原作废规则收下的是第 1 次。
#[test]
fn original_voiding_keeps_the_attempt_voided_only_by_the_calibration() {
    let attempts = [
        AttemptVoiding {
            is_valid_under_the_original_voiding: true,
            is_voided_by_the_calibration: true,
        },
        AttemptVoiding {
            is_valid_under_the_original_voiding: true,
            is_voided_by_the_calibration: false,
        },
    ];
    let batch_cpu_nanoseconds_of_each_attempt = [2_000_000_u64, 1_000_000];
    let kept = attempt_kept_under_the_original_voiding(&attempts).expect("有一次照原规则有效");
    assert_eq!(kept, 0);
    assert_eq!(batch_cpu_nanoseconds_of_each_attempt[kept], 2_000_000);
    let voided_by_the_original_rules_first = [
        AttemptVoiding {
            is_valid_under_the_original_voiding: false,
            is_voided_by_the_calibration: false,
        },
        attempts[1],
    ];
    assert_eq!(
        attempt_kept_under_the_original_voiding(&voided_by_the_original_rules_first),
        Some(1)
    );
    assert_eq!(attempt_kept_under_the_original_voiding(&[]), None);
}

/// 一条只有最后一批的意图：批 CPU、批挂钟、批等盘由参数给。
fn intent_with_one_batch(
    cpu_nanoseconds: u64,
    wall_nanoseconds: u64,
    wait_nanoseconds: i64,
) -> Vec<CallReading> {
    let mut call = call_reading(CallKind::FinalBatch, 262_144, None, 524_288, 1);
    call.cpu_nanoseconds = cpu_nanoseconds;
    call.wall_nanoseconds = wall_nanoseconds;
    call.device_totals.wait_nanoseconds = wait_nanoseconds;
    vec![call]
}

/// N14（第一段）：两种作废规则各收的几轮照原判定函数各算一遍。内存臂挂钟 10 ms；真盘臂 CPU 9 ms、挂钟 30 ms、等盘 3 ms 的两轮
/// ⇒ ρ = 3 判 cpu、挂钟比 3 判 disk、对账 0.1 判 compatible；把第 2 轮换成 CPU 30 ms ⇒ 对账 2.0，两轮不全相同判 unstable。
#[test]
fn wall_clock_cell_judgements_follow_the_registered_judgements_under_either_voiding() {
    let point = small_point(1024);
    let stored_of = |real_cpu_of_the_later_round: u64| -> Vec<StoredIntent> {
        [1_usize, 2]
            .into_iter()
            .flat_map(|round| {
                let real_cpu = if round == 2 {
                    real_cpu_of_the_later_round
                } else {
                    9_000_000
                };
                [
                    stored_intent_of(
                        &label_of_round(DeviceArm::Memory, round),
                        true,
                        intent_with_one_batch(4_000_000, 10_000_000, 5_500_000),
                    ),
                    stored_intent_of(
                        &label_of_round(DeviceArm::RealDisk, round),
                        true,
                        intent_with_one_batch(real_cpu, 30_000_000, 3_000_000),
                    ),
                ]
            })
            .collect()
    };
    assert_eq!(
        wall_clock_cell_judgements(&stored_of(9_000_000), point, 8),
        WallClockCellJudgements {
            rounds_on_the_real_disk: 2,
            paired_rounds: 2,
            rho: "cpu",
            wall_ratio: "disk",
            reconciliation: "compatible",
        }
    );
    assert_eq!(
        wall_clock_cell_judgements(&stored_of(30_000_000), point, 8).reconciliation,
        "unstable"
    );
    assert_eq!(
        wall_clock_cell_judgements(&stored_of(9_000_000), point, 4).rho,
        "not_run",
        "没有那一格的意图"
    );
}

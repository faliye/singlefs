use super::*;

/// 整个进程只建一份 Vulkan 实例：两次取到的是同一份，建的次数记 1（每建一次加载器就把本机每个 ICD 再 dlopen 一遍，
/// 静态 TLS 吃光之后 NVIDIA 的驱动装不上，`vulkan_instance_of_this_process` 的文档注释写了来历）。
/// 判别力（`crates/mutations.tsv`「GPU 取卡 Vulkan 实例每次新建」）：改成每次新建一份，两次取到的不是同一份、建的次数是 2。
#[test]
fn the_vulkan_instance_is_created_once_per_process_and_reused() {
    let first = super::vulkan_instance_of_this_process();
    let second = super::vulkan_instance_of_this_process();
    assert!(
        std::ptr::eq(first, second),
        "两次取到的 Vulkan 实例要是同一份"
    );
    assert_eq!(
        super::VULKAN_INSTANCES_CREATED_IN_THIS_PROCESS.load(std::sync::atomic::Ordering::SeqCst),
        1,
        "整个进程只建一份实例"
    );
}

/// J8、M4 第 11 条：块属性里的驱动版本取 wgpu adapter info 的 `driver` 与 `driver_info` 两项，空格接起来；一项空只取另一项，
/// 两项都空写一句没报（不写空串）。
#[test]
fn the_driver_version_of_a_card_joins_the_driver_name_and_its_information() {
    assert_eq!(
        super::driver_version_of_the_adapter("NVIDIA", "580.95.05"),
        "NVIDIA 580.95.05"
    );
    assert_eq!(
        super::driver_version_of_the_adapter("NVIDIA", " "),
        "NVIDIA"
    );
    assert_eq!(
        super::driver_version_of_the_adapter("", "580.95.05"),
        "580.95.05"
    );
    assert_eq!(
        super::driver_version_of_the_adapter("", ""),
        "driver-not-reported-by-wgpu"
    );
}

/// （攻方 R1）：`request_device` 交回 wgpu 的设备显存不够是显存不够（卡池报本次未跑），
/// 设备丢了这类别的错是起不来（判红）。错是照 wgpu 的错类型造的，不要显卡。
#[test]
fn device_request_refused_for_lack_of_memory_is_told_apart_from_other_refusals() {
    let refused = |device_error| {
        failure_of_a_device_request(
            &wgpu::RequestDeviceError::from(wgpu::wgc::instance::RequestDeviceError::Device(
                device_error,
            )),
            "序号 0 的卡",
        )
    };
    assert!(
        matches!(
            refused(wgpu::wgc::device::DeviceError::OutOfMemory),
            CardStartFailure::OutOfGpuMemory { .. }
        ),
        "起设备时显存不够：报本次未跑那一种"
    );
    assert!(
        matches!(
            refused(wgpu::wgc::device::DeviceError::Lost),
            CardStartFailure::DeviceNotStarted { .. }
        ),
        "别的错：判红那一种"
    );
}

/// （攻方 R2）：wgpu 没接住的 Out of Memory（判器内核在另起的线程上编，那里没有错误范围）先记在设备上、
/// 再照默认的做法 panic；校验错照样 panic、不记成显存不够。错是照 wgpu 的错类型造的，不要显卡。
#[test]
fn an_uncaptured_out_of_memory_is_recorded_on_the_device_before_the_panic() {
    let condition = DeviceConditionOfACard::default();
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            handle_an_uncaptured_wgpu_error(
                &condition,
                wgpu::Error::OutOfMemory {
                    source: Box::new(std::fmt::Error),
                },
            );
        }))
        .is_err(),
        "没接住的 Out of Memory 照默认的做法 panic"
    );
    assert_eq!(condition.out_of_memory_reports(), 1, "panic 之前记在设备上");
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            handle_an_uncaptured_wgpu_error(
                &condition,
                wgpu::Error::Validation {
                    source: Box::new(std::fmt::Error),
                    description: "绑定组布局对不上".to_string(),
                },
            );
        }))
        .is_err(),
        "校验错照样 panic"
    );
    assert_eq!(condition.out_of_memory_reports(), 1, "校验错不记成显存不够");
}

/// （攻方乙形、K3）：不归卡池的设备起之前就在它的总线上记下（起的那一段里卡池已经认得出），
/// 卡池的设备不记。总线号是造的，不与本机的卡重。
#[test]
fn device_outside_the_card_pool_is_marked_on_its_bus_before_it_starts_and_a_pool_device_is_not() {
    assert_eq!(
        started_with_its_owner_recorded(
            DeviceOwnerOnTheCard::OutsideTheCardPool,
            "fe:00.0",
            || {
                Ok(has_devices_started_outside_the_card_pool_on_the_bus(
                    "fe:00.0",
                ))
            }
        )
        .map(|(marked_while_starting, marked_on_the_bus)| (
            marked_while_starting,
            marked_on_the_bus.is_some()
        )),
        Ok((true, true)),
        "不归卡池：起的那一段里已经记着，起好交那一记"
    );
    assert_eq!(
        started_with_its_owner_recorded(DeviceOwnerOnTheCard::TheCardPool, "fd:00.0", || {
            Ok(has_devices_started_outside_the_card_pool_on_the_bus(
                "fd:00.0",
            ))
        })
        .map(|(marked_while_starting, marked_on_the_bus)| (
            marked_while_starting,
            marked_on_the_bus.is_some()
        )),
        Ok((false, false)),
        "卡池的：不记，也不交那一记"
    );
    assert!(!has_devices_started_outside_the_card_pool_on_the_bus(
        "fd:00.0"
    ));
}

/// （认出不走卡池设备的接线没有测试）：卡池读显存的那一份照这个进程起过不归卡池设备的总线认，
/// 那张卡上这个进程占着的记成分不出；别的卡照都走账。报告是造的，总线号不与本机的卡重。
#[test]
fn the_card_pool_reads_this_process_as_also_outside_the_books_on_a_bus_with_a_device_outside_the_pool(
) {
    let (_device, _marked_on_the_bus) = started_with_its_owner_recorded(
        DeviceOwnerOnTheCard::OutsideTheCardPool,
        "fc:00.0",
        || Ok(()),
    )
    .expect("造的设备起得来");
    let this_process = std::process::id();
    let report = ["00000000:FC:00.0", "00000000:FB:00.0"]
        .iter()
        .map(|bus| {
            format!(
                "GPU {bus}\n    FB Memory Usage\n        Total                                          : 16311 MiB\n\
                     \x20       Reserved                                       : 462 MiB\n\
                     \x20       Free                                           : 10000 MiB\n    Processes\n\
                     \x20       Process ID                                     : {this_process}\n\
                     \x20           Used GPU Memory                            : 1000 MiB\n"
            )
        })
        .collect::<String>();
    let read = card_memory_reader_of_the_card_pool(
        Box::new(move || Ok(report.clone())),
        vec!["fc:00.0".to_string(), "fb:00.0".to_string()],
    );
    assert_eq!(
        read().map(|readings| readings
            .iter()
            .map(|reading| reading.occupied_by_this_process)
            .collect::<Vec<OccupiedByThisProcess>>()),
        Ok(vec![
            OccupiedByThisProcess::AlsoOutsideTheBooks { mebibytes: 1000 },
            OccupiedByThisProcess::OnlyThroughTheBooks { mebibytes: 1000 },
        ])
    );
}

/// 不归卡池的那一台的句柄都丢了（那一记跟着丢），卡池读那张卡回到按账拆：这个进程占着的不再记成分不出。
/// 报告是造的，总线号不与本机的卡重、不与别的用例重。
/// 判别力（`crates/mutations.tsv`「显存账 不走账的设备丢掉时不减」）：那一记丢掉时不减，这条红。
#[test]
fn the_card_pool_reads_this_process_through_the_books_again_once_its_device_outside_the_pool_is_dropped(
) {
    let started_outside_the_pool = started_with_its_owner_recorded(
        DeviceOwnerOnTheCard::OutsideTheCardPool,
        "fa:00.0",
        || Ok(()),
    )
    .expect("造的设备起得来");
    let this_process = std::process::id();
    let report = format!(
        "GPU 00000000:FA:00.0\n    FB Memory Usage\n        Total                                          : 16311 MiB\n\
             \x20       Reserved                                       : 462 MiB\n\
             \x20       Free                                           : 10000 MiB\n    Processes\n\
             \x20       Process ID                                     : {this_process}\n\
             \x20           Used GPU Memory                            : 1000 MiB\n"
    );
    let read = card_memory_reader_of_the_card_pool(
        Box::new(move || Ok(report.clone())),
        vec!["fa:00.0".to_string()],
    );
    let occupied_by_this_process = || {
        read().map(|readings| {
            readings
                .iter()
                .map(|reading| reading.occupied_by_this_process)
                .collect::<Vec<OccupiedByThisProcess>>()
        })
    };
    assert_eq!(
        occupied_by_this_process(),
        Ok(vec![OccupiedByThisProcess::AlsoOutsideTheBooks {
            mebibytes: 1000
        }]),
        "不归卡池的那一台还在：分不出"
    );
    drop(started_outside_the_pool);
    assert_eq!(
        occupied_by_this_process(),
        Ok(vec![OccupiedByThisProcess::OnlyThroughTheBooks {
            mebibytes: 1000
        }]),
        "那一台丢了：回到按账拆"
    );
}

/// 起一台设备打一行起设备的 GPU 阶段现场（起没起好都打），点名卡的序号、总线号、设备归谁与结局；卡池的与不归卡池的各起一台各一行。
/// 不要卡。判别力（`crates/mutations.tsv`「GPU 阶段现场 起设备不打那一行」）：起完不打，这条红。
#[test]
fn starting_a_device_prints_one_device_start_lifecycle_line_naming_the_card_its_owner_and_the_outcome(
) {
    let lines = std::sync::Mutex::new(Vec::<String>::new());
    let trace: &dyn Fn(&str, std::fmt::Arguments<'_>) = &|phase, detail| {
        lines
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(format!("{phase} {detail}"));
    };
    assert_eq!(
        device_started_with_a_lifecycle_line(
            3,
            "f9:00.0",
            DeviceOwnerOnTheCard::TheCardPool,
            || Ok::<(), CardStartFailure>(()),
            trace,
        ),
        Ok(())
    );
    let short_of_memory = CardStartFailure::OutOfGpuMemory {
        reason: "造的显存不够".to_string(),
    };
    assert_eq!(
        device_started_with_a_lifecycle_line(
            4,
            "f8:00.0",
            DeviceOwnerOnTheCard::OutsideTheCardPool,
            || Err::<(), CardStartFailure>(short_of_memory.clone()),
            trace,
        ),
        Err(short_of_memory)
    );
    let lines = lines
        .into_inner()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    assert_eq!(lines.len(), 2, "起两台各打一行：{lines:?}");
    assert!(
        lines[0].starts_with(
            "device_start card=3 pci=\"f9:00.0\" owner=the-card-pool outcome=started start_microseconds="
        ),
        "{lines:?}"
    );
    assert!(
        lines[1].starts_with(
            "device_start card=4 pci=\"f8:00.0\" owner=outside-the-card-pool outcome=out-of-gpu-memory start_microseconds="
        ),
        "{lines:?}"
    );
}

/// 设备丢失的回调把这个设备记成丢了，连同 wgpu 给的原因与那一句（卡池照它重起）。不要卡。
#[test]
fn the_device_lost_callback_records_the_device_as_lost_with_the_reason_and_the_message() {
    let condition = DeviceConditionOfACard::default();
    record_the_loss_of_the_device(&condition, wgpu::DeviceLostReason::Unknown, "驱动重置");
    assert!(condition.is_lost());
    assert_eq!(
        condition.lost_reason(),
        Some("Unknown：驱动重置".to_string())
    );
}

/// （攻方甲形，wgpu-core 30.0.1 `src/device/resource.rs` 的 `handle_hal_error`、wgpu-hal 30.0.1
/// `src/lib.rs` 的 `DeviceError::OutOfMemory`（`#[error("Out of memory")]`））：wgpu 因显存不够丢设备时给的那一句认成显存不够，别的认成别的原因；回调照它记，之后一段因设备丢了而失败的活、
/// 不在同一组而这段活开始之后一条 Out of Memory 都没记过，也说成显存不够。不要卡。
/// 期望的那一句写死成 wgpu-hal 30.0.1 `DeviceError::OutOfMemory` 的
/// `#[error("Out of memory")]`，不与产品代码取自同一个 `to_string()`：wgpu 改了这一句，这里先红。
#[test]
fn device_lost_for_lack_of_memory_explains_a_failure_on_it_as_gpu_memory_short() {
    let out_of_memory = "Out of memory";
    assert_eq!(
        cause_of_the_device_loss(out_of_memory),
        CauseOfTheDeviceLoss::GpuMemoryShort
    );
    for other in [
        wgpu::hal::DeviceError::Lost.to_string(),
        wgpu::hal::DeviceError::Unexpected.to_string(),
        "驱动重置".to_string(),
    ] {
        assert_eq!(
            cause_of_the_device_loss(&other),
            CauseOfTheDeviceLoss::Other,
            "{other}"
        );
    }
    let condition = DeviceConditionOfACard::default();
    let work = crate::gpu_memory_ledger::WorkOnTheCardOfThisThread::begin();
    let reports_before = condition.out_of_memory_reports();
    record_the_loss_of_the_device(&condition, wgpu::DeviceLostReason::Unknown, out_of_memory);
    assert!(
        condition
            .out_of_memory_explaining_a_failure_of_the_work(
                work.identifier(),
                reports_before,
                condition.device_when_the_work_returns()
            )
            .is_some(),
        "设备因显存不够丢了：说成显存不够"
    );
}

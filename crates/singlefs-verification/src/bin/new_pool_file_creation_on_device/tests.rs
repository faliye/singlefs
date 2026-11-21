use super::{
    describe_publish_writes, entry_of_the_mode, pool_writes_between, publish_writes_against_device,
    registered_segments_of, switch_instance_and_publish_third_version, CountedDevice,
    DeviceCallCounts, SegmentClock,
};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
use singlefs_core::write_accounting::{
    WriteCallsAndBytes, WritesByStructureKind, WrittenStructureKind,
};
use singlefs_verification::fault_injection::{FaultInjectingBlockDevice, SharedFaultPlan};
use singlefs_verification::memory_pool::SparseBlockDevice;
use singlefs_verification::on_device_modes::{
    allocator_rebuilt_from_the_records_with_the_root_ring_of_this_process, fourth_file_content,
    publish_the_second_version, third_file_content, OnDeviceRunMode,
};
use singlefs_verification::scenario::{e142_parameters, run_new_pool_file_creation};
use singlefs_verification::segments::FixedGeometry;
use singlefs_verification::{RecordingBlockDevice, SharedStream};

/// 虚机里两块 virtio 盘的大小以外都照 E142 的几何：物理块 512、io_min 512。
const SPARSE_DEVICE_BYTES: u64 = 4 << 30;

/// 结果行里 `key=value` 那一段的值；没有这个字段是 `None`。
fn field<'line>(line: &'line str, key: &str) -> Option<&'line str> {
    line.split(' ')
        .find_map(|part| part.strip_prefix(key)?.strip_prefix('='))
}

/// `second-instance` 模式在宿主上照同一条路跑一遍：稀疏内存盘代替 virtio 盘，「冷重开」把镜像交给新句柄。
/// 发布 B 之后可写挂载取实例 2、写行 txg 5、暖机 txg 6 / 7，发布 C 是 txg 8；两段窗口按种类的合计都与设备一层相等；
/// 冷恢复择实例 2 的根读回第三版。
#[test]
fn second_instance_mode_mounts_writes_the_row_warms_up_publishes_the_third_version_and_every_window_matches(
) {
    let parameters = e142_parameters(512, 512);
    let stream = SharedStream::new();
    let geometry_for_faults = FixedGeometry {
        fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
        journal_ring_bytes: parameters.geometry.journal_ring_bytes,
        root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
    };
    let plan = SharedFaultPlan::unarmed(geometry_for_faults);
    let mut devices: Vec<(DeviceIdentity, CountedDevice<SparseBlockDevice>)> = (0..2u32)
        .map(|device_number| {
            let identity = DeviceIdentity(device_number);
            (
                identity,
                RecordingBlockDevice::with_shared_stream(
                    identity,
                    FaultInjectingBlockDevice::new(
                        identity,
                        SparseBlockDevice::new(SPARSE_DEVICE_BYTES, PhysicalBlockSizeInBytes(512)),
                        plan.clone(),
                    ),
                    stream.clone(),
                ),
            )
        })
        .collect();
    let run = run_new_pool_file_creation(&parameters, &mut devices, &stream, |_point, _devices| {})
        .expect("新池新建文件");
    let mut allocator = allocator_rebuilt_from_the_records_with_the_root_ring_of_this_process(
        &parameters,
        &devices,
        &run,
    );
    let second = publish_the_second_version(&parameters, &mut devices, &mut allocator, &run.output)
        .expect("发布 B");
    assert_eq!(second.root.checkpoint_txg, CheckpointTxg(4));
    let geometry = FixedGeometry {
        fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
        journal_ring_bytes: parameters.geometry.journal_ring_bytes,
        root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
    };

    let mut clock = SegmentClock::start();
    let switched = switch_instance_and_publish_third_version(
        &parameters,
        devices,
        &stream,
        &geometry,
        &mut clock,
        |closed_devices| {
            closed_devices
                .into_iter()
                .map(|(identity, closed)| {
                    let mut reopened =
                        SparseBlockDevice::new(SPARSE_DEVICE_BYTES, PhysicalBlockSizeInBytes(512));
                    reopened.image = closed.image;
                    (identity, reopened)
                })
                .collect()
        },
    )
    .expect("第二个实例");

    let line_named = |name: &str| -> Vec<&str> {
        switched
            .lines
            .iter()
            .map(String::as_str)
            .filter(|line| field(line, "name") == Some(name))
            .collect()
    };
    let mount_lines = line_named("writable_mount");
    assert_eq!(mount_lines.len(), 1, "{:#?}", switched.lines);
    let mount = mount_lines[0];
    assert_eq!(field(mount, "instance"), Some("2"), "{mount}");
    assert_eq!(field(mount, "chosen_root"), Some("1:4"), "B 的根：{mount}");
    assert_eq!(
        field(mount, "rows_written"),
        Some("1"),
        "实例 1 那一行：{mount}"
    );
    assert_eq!(field(mount, "row_publish_root"), Some("2:5"), "{mount}");
    assert_eq!(field(mount, "warm_up_txgs"), Some("6,7"), "{mount}");

    let publish_lines: Vec<(Option<&str>, Option<&str>)> = line_named("publish_writes")
        .into_iter()
        .map(|line| (field(line, "publish"), field(line, "txg")))
        .collect();
    assert_eq!(
        publish_lines,
        vec![
            (Some("instance_row"), Some("5")),
            (Some("warm_up"), Some("6")),
            (Some("warm_up"), Some("7")),
            (Some("third_transaction"), Some("8")),
        ]
    );
    let windows = line_named("publish_writes_against_device");
    let window_summary: Vec<(Option<&str>, Option<&str>, Option<&str>)> = windows
        .iter()
        .map(|line| {
            (
                field(line, "window"),
                field(line, "publishes"),
                field(line, "matches"),
            )
        })
        .collect();
    assert_eq!(
        window_summary,
        vec![
            (Some("writable_mount"), Some("3"), Some("true")),
            (Some("third_transaction"), Some("1"), Some("true")),
        ],
        "两段窗口按种类的合计都等于设备一层数的：{windows:#?}"
    );
    assert!(switched.every_window_matches_device);
    let third_lines = line_named("third_transaction");
    assert_eq!(third_lines.len(), 1, "{:#?}", switched.lines);
    assert_eq!(field(third_lines[0], "root_txg"), Some("8"));
    assert_eq!(
        field(third_lines[0], "transaction"),
        Some("1"),
        "事务号按实例从 1 起"
    );
    // 单元写那一段 28 = 数据单元、extent 根、inode 叶容器、inode 根、记账根、映射根、树表各两盘（14），
    // 加分配记录树按位置寻址（D8（核心索引结构） 已定项 14）这一次重写的节点两盘共 14 次写（`publish_writes` 那一行的
    // `allocation_record_tree_node_write_calls`）；按位置寻址之前那棵树只有一个节点，这一段是 16。
    assert_eq!(
        field(third_lines[0], "segments"),
        Some("28+2+1+2"),
        "发布 C 与发布 B 同型"
    );

    let cold: Vec<(DeviceIdentity, SparseBlockDevice)> = switched
        .devices
        .into_iter()
        .map(|(identity, device)| (identity, device.into_inner_and_operations().0.into_inner()))
        .collect();
    let report = recover(&cold, JournalPolicy::Consult);
    match report.outcome {
        RecoveryOutcome::FileRead { root, content } => {
            assert_eq!(root, (InstanceGeneration(2), CheckpointTxg(8)));
            assert!(content == third_file_content(), "冷恢复读回第三版");
        }
        RecoveryOutcome::NoFile { root } => panic!("冷恢复择到 {root:?} 却没有文件"),
        RecoveryOutcome::Failed { root, failure } => {
            panic!("冷恢复失败：{root:?} {failure:?}")
        }
    }
}

fn device_counts(write_calls: u64, written_bytes: u64) -> DeviceCallCounts {
    DeviceCallCounts {
        write_calls,
        written_bytes,
        force_unit_access_writes: 0,
        barrier_calls: 0,
    }
}

#[test]
fn publish_writes_line_gives_the_total_then_every_kind_in_report_order_with_zeros() {
    let mut writes = WritesByStructureKind::NOTHING_WRITTEN;
    writes.count_write_call(WrittenStructureKind::JournalRecord, &[0u8; 4096]);
    writes.count_write_call(WrittenStructureKind::JournalRecord, &[0u8; 4096]);
    writes.count_write_call(WrittenStructureKind::RootSlot, &[0u8; 512]);
    assert_eq!(
        describe_publish_writes("warm_up", 1, &writes),
        "name=publish_writes publish=warm_up txg=1 write_calls=3 written_bytes=8704 \
             data_unit_write_calls=0 data_unit_written_bytes=0 \
             extent_tree_node_write_calls=0 extent_tree_node_written_bytes=0 \
             inode_tree_leaf_container_write_calls=0 inode_tree_leaf_container_written_bytes=0 \
             inode_tree_root_write_calls=0 inode_tree_root_written_bytes=0 \
             allocation_record_tree_node_write_calls=0 allocation_record_tree_node_written_bytes=0 \
             accounting_tree_node_write_calls=0 accounting_tree_node_written_bytes=0 \
             deadlist_tree_node_write_calls=0 deadlist_tree_node_written_bytes=0 \
             deleted_inodes_tree_node_write_calls=0 deleted_inodes_tree_node_written_bytes=0 \
             tombstone_tree_node_write_calls=0 tombstone_tree_node_written_bytes=0 \
             lineage_tree_node_write_calls=0 lineage_tree_node_written_bytes=0 \
             logged_operations_tree_node_write_calls=0 logged_operations_tree_node_written_bytes=0 \
             retirement_tree_node_write_calls=0 retirement_tree_node_written_bytes=0 \
             central_mapping_tree_node_write_calls=0 central_mapping_tree_node_written_bytes=0 \
             tree_table_unit_write_calls=0 tree_table_unit_written_bytes=0 \
             instance_table_unit_write_calls=0 instance_table_unit_written_bytes=0 \
             journal_record_write_calls=2 journal_record_written_bytes=8192 \
             root_slot_write_calls=1 root_slot_written_bytes=512 \
             system_configuration_slot_write_calls=0 system_configuration_slot_written_bytes=0"
    );
}

#[test]
fn device_window_check_matches_only_when_both_write_calls_and_bytes_are_equal() {
    let mut first = WritesByStructureKind::NOTHING_WRITTEN;
    first.count_write_call(WrittenStructureKind::SystemConfigurationSlot, &[0u8; 4096]);
    let mut second = WritesByStructureKind::NOTHING_WRITTEN;
    second.count_write_call(WrittenStructureKind::RootSlot, &[0u8; 512]);
    let (line, matches) = publish_writes_against_device(
        "warm_up",
        &[&first, &second],
        WriteCallsAndBytes {
            write_calls: 2,
            written_bytes: 4608,
        },
    );
    assert!(matches, "两次发布合起来 2 次、4608 字节，与设备一层相等");
    assert_eq!(
            line,
            "name=publish_writes_against_device window=warm_up publishes=2 by_kind_write_calls=2 by_kind_written_bytes=4608 device_write_calls=2 device_written_bytes=4608 matches=true"
        );
    let (_, bytes_differ) = publish_writes_against_device(
        "warm_up",
        &[&first, &second],
        WriteCallsAndBytes {
            write_calls: 2,
            written_bytes: 4096,
        },
    );
    assert!(!bytes_differ, "字节差一个根槽：判不相等");
    let (calls_differ_line, calls_differ) = publish_writes_against_device(
        "new_pool_file_creation",
        &[&first],
        WriteCallsAndBytes {
            write_calls: 2,
            written_bytes: 4096,
        },
    );
    assert!(!calls_differ, "写调用差一次：判不相等");
    assert!(calls_differ_line.ends_with(" matches=false"));
}

#[test]
fn pool_writes_between_adds_the_differences_of_every_device() {
    let earlier = [device_counts(11, 172_544), device_counts(10, 172_032)];
    let later = [device_counts(22, 345_088), device_counts(20, 344_064)];
    assert_eq!(
        pool_writes_between(&earlier, &later),
        WriteCallsAndBytes {
            write_calls: 21,
            written_bytes: 344_576
        }
    );
}

#[test]
fn device_call_counts_subtract_field_by_field_and_describe_with_the_device_index() {
    let earlier = DeviceCallCounts {
        write_calls: 11,
        written_bytes: 172_544,
        force_unit_access_writes: 1,
        barrier_calls: 2,
    };
    let later = DeviceCallCounts {
        write_calls: 22,
        written_bytes: 345_088,
        force_unit_access_writes: 2,
        barrier_calls: 4,
    };
    let delta = later.since(earlier);
    assert_eq!(
        delta,
        DeviceCallCounts {
            write_calls: 11,
            written_bytes: 172_544,
            force_unit_access_writes: 1,
            barrier_calls: 2,
        }
    );
    assert_eq!(
            delta.describe(1),
            "device_1_writes=11 device_1_written_bytes=172544 device_1_force_unit_access_writes=1 device_1_barriers=2"
        );
}

/// 分段标记少打一段就要被拒收：段名与登记的逐项比，少一段、次序反了都 `matches=false`。
/// 「登记的」是 [`registered_segments_of`]，每一档写死一张表。
#[test]
fn segment_timing_registration_refuses_a_run_that_marked_one_segment_fewer() {
    let registered = registered_segments_of(OnDeviceRunMode::Direct);
    assert_eq!(
        registered,
        [
            "mkfs",
            "instance_acquisition",
            "warm_up",
            "new_pool_file_creation",
            "cold_reopen_and_recover"
        ]
    );

    let mut complete = SegmentClock::start();
    for segment in registered {
        let line = complete.mark(segment);
        assert!(
            line.starts_with(&format!(
                "name=segment_timing segment={segment} nanoseconds="
            )),
            "每段末尾一行带单调时钟的标记：{line}"
        );
        assert!(line.contains(" monotonic_nanoseconds="));
    }
    let (line, matches) = complete.registration_check("direct", registered);
    assert!(matches, "五段都打了：{line}");
    assert_eq!(
            line,
            "name=segment_timing_registration mode=direct \
             registered=mkfs,instance_acquisition,warm_up,new_pool_file_creation,cold_reopen_and_recover \
             marked=mkfs,instance_acquisition,warm_up,new_pool_file_creation,cold_reopen_and_recover \
             matches=true"
        );

    let mut missing_one = SegmentClock::start();
    for segment in registered.iter().filter(|name| **name != "warm_up") {
        missing_one.mark(segment);
    }
    let (missing_line, missing_matches) = missing_one.registration_check("direct", registered);
    assert!(
        !missing_matches,
        "少打了 warm_up 那一段，必须拒收：{missing_line}"
    );
    assert!(missing_line.ends_with("matches=false"));

    let mut out_of_order = SegmentClock::start();
    for segment in ["instance_acquisition", "mkfs"] {
        out_of_order.mark(segment);
    }
    assert!(
        !out_of_order
            .registration_check("direct", &["mkfs", "instance_acquisition"])
            .1,
        "次序反了也算对不上"
    );

    // 包含自检：各段之和不超过整条路的挂钟（段首尾相接，超过就是某一段重复计了）。
    let (containment_line, contained) = complete.containment_check();
    assert!(contained, "{containment_line}");
    assert!(containment_line.starts_with("name=segment_timing_check segments=5 sum_nanoseconds="));
}

/// 每一档登记的段都互不相同、而且都在这一档真会走到的那几段里：
/// 每档一个入口 fn（剪枝复查 4-2d ②：QEMU 门禁按档从各自的入口算来宾程序的闭包，不再六档共用 `main` 一把）：模式表里每一档的入口名字是
/// 「run_the_<模式参数，- 换成 _>_mode」，六档六个、两两不同（两档合回一个入口，这条红）。
#[test]
fn every_mode_has_its_own_entry_function_named_after_its_argument() {
    let mut names = Vec::new();
    for mode in OnDeviceRunMode::ALL {
        let (_entry, name) = entry_of_the_mode(mode);
        assert_eq!(
            name,
            format!("run_the_{}_mode", mode.argument().replace('-', "_")),
            "{mode:?} 的入口名照模式参数起"
        );
        names.push(name);
    }
    names.sort_unstable();
    names.dedup();
    assert_eq!(
        names.len(),
        OnDeviceRunMode::ALL.len(),
        "每档一个入口，两两不同"
    );
}

/// `second-instance` 比 `file-overwrite` 多两段（重开可写挂载、发布 C），后者比 `direct` 多一段（发布 B）；
/// `raise-rollback-floor` 比 `second-instance` 多两段（发布 D、抬 F）。
#[test]
fn every_mode_registers_its_own_segments_with_no_repeats() {
    for mode in OnDeviceRunMode::ALL {
        let registered = registered_segments_of(mode);
        let mut sorted = registered.to_vec();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), registered.len(), "{mode:?} 登记的段名有重复");
        assert_eq!(registered[0], "mkfs", "{mode:?} 第一段是 mkfs");
        assert_eq!(
            registered[registered.len() - 1],
            "cold_reopen_and_recover",
            "{mode:?} 最后一段是冷重开"
        );
    }
    assert_eq!(registered_segments_of(OnDeviceRunMode::Direct).len(), 5);
    assert_eq!(
        registered_segments_of(OnDeviceRunMode::FileOverwrite).len(),
        6
    );
    assert_eq!(
        registered_segments_of(OnDeviceRunMode::SecondInstance).len(),
        8
    );
    assert_eq!(
        registered_segments_of(OnDeviceRunMode::RaiseRollbackFloor),
        [
            "mkfs",
            "instance_acquisition",
            "warm_up",
            "new_pool_file_creation",
            "file_overwrite",
            "reopen_and_writable_mount",
            "third_transaction",
            "fourth_transaction",
            "raise_rollback_floor",
            "cold_reopen_and_recover"
        ],
        "raise-rollback-floor 比 second-instance 多发布 D 与抬 F 两段，排在发布 C 之后、冷重开之前"
    );
}

use super::{
    describe_new_pool_file_creation_path_failure, device_call_counts, new_pool_file_creation_paths,
    publish_the_fourth_version_and_describe,
    publish_the_fourth_version_and_raise_the_rollback_floor,
    publish_the_second_version_and_describe, publish_the_third_version_and_describe,
    raise_the_rollback_floor_and_describe, reopen_and_mount_writable, FailedRun, SecondInstanceRun,
};
use singlefs_verification::device_log::{expected_device_events, DeviceEvent};
use singlefs_verification::fault_injection::{FaultSchedule, InjectedFault};
use singlefs_verification::scenario::{NewPoolFileCreationPathStep, ScenarioPoint};

fn geometry_of(
    parameters: &singlefs_core::make_filesystem::MakeFilesystemParameters,
) -> FixedGeometry {
    FixedGeometry {
        fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
        journal_ring_bytes: parameters.geometry.journal_ring_bytes,
        root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
    }
}

/// 两块稀疏内存盘，外面各包一层数调用的注入包装、再包一层录制器（与虚机里那一套同形）。
fn counted_sparse_devices(
    stream: &SharedStream,
    plan: &SharedFaultPlan,
) -> Vec<(DeviceIdentity, CountedDevice<SparseBlockDevice>)> {
    (0..2u32)
        .map(|device_number| {
            let identity = DeviceIdentity(device_number);
            (
                identity,
                RecordingBlockDevice::with_shared_stream(
                    identity,
                    FaultInjectingBlockDevice::new(
                        identity,
                        SparseBlockDevice::new(SPARSE_DEVICE_BYTES, PhysicalBlockSizeInBytes(512)),
                        plan.clone(),
                    ),
                    stream.clone(),
                ),
            )
        })
        .collect()
}

/// 宿主上的「冷重开」：镜像交给新句柄。
fn reopen_sparse_devices(
    closed_devices: Vec<(DeviceIdentity, SparseBlockDevice)>,
) -> Vec<(DeviceIdentity, SparseBlockDevice)> {
    closed_devices
        .into_iter()
        .map(|(identity, closed)| {
            let mut reopened =
                SparseBlockDevice::new(SPARSE_DEVICE_BYTES, PhysicalBlockSizeInBytes(512));
            reopened.image = closed.image;
            (identity, reopened)
        })
        .collect()
}

fn lines_named<'lines>(lines: &'lines [String], name: &str) -> Vec<&'lines str> {
    lines
        .iter()
        .map(String::as_str)
        .filter(|line| field(line, "name") == Some(name))
        .collect()
}

fn failed_run_of<Success>(result: Result<Success, FailedRun>, what: &str) -> FailedRun {
    match result {
        Ok(_) => panic!("{what}：注入的写错该让它失败"),
        Err(failed) => failed,
    }
}

/// 失败那一段窗口的结果行：同一段里已经落盘的发布各一行（`persisted_write_calls` 按先后，各自的写调用数）、失败账一份
/// （`failed_write_calls` 条写已落盘）、两样相加与设备一层逐项相等（`window_write_calls` 条），停下的原因是注入的写错。
fn assert_the_failed_window_is_reconciled(
    lines: &[String],
    window: &str,
    persisted_write_calls: &[&str],
    failed_write_calls: &str,
    window_write_calls: &str,
) {
    let persisted_writes: Vec<(Option<&str>, Option<&str>, Option<&str>)> =
        lines_named(lines, "persisted_publish_writes")
            .into_iter()
            .map(|line| {
                (
                    field(line, "window"),
                    field(line, "publish"),
                    field(line, "write_calls"),
                )
            })
            .collect();
    let expected_persisted_writes: Vec<(Option<&str>, Option<&str>, Option<&str>)> =
        persisted_write_calls
            .iter()
            .enumerate()
            .map(|(index, write_calls)| {
                (
                    Some(window),
                    Some(["1", "2", "3"][index]),
                    Some(*write_calls),
                )
            })
            .collect();
    assert_eq!(
        persisted_writes, expected_persisted_writes,
        "同一段里失败之前已经落盘的发布各一行：{lines:#?}"
    );
    let failed_writes = lines_named(lines, "failed_publish_writes");
    assert_eq!(failed_writes.len(), 1, "失败账一份：{lines:#?}");
    assert_eq!(field(failed_writes[0], "window"), Some(window));
    assert_eq!(field(failed_writes[0], "failure"), Some("1"));
    assert_eq!(
        field(failed_writes[0], "write_calls"),
        Some(failed_write_calls),
        "失败之前已落盘的写：{}",
        failed_writes[0]
    );
    let failed_window = format!("{window}_failed");
    let windows: Vec<&str> = lines_named(lines, "publish_writes_against_device")
        .into_iter()
        .filter(|line| field(line, "window") == Some(failed_window.as_str()))
        .collect();
    assert_eq!(windows.len(), 1, "{lines:#?}");
    let publishes = (persisted_write_calls.len() + 1).to_string();
    assert_eq!(field(windows[0], "publishes"), Some(publishes.as_str()));
    assert_eq!(
        field(windows[0], "by_kind_write_calls"),
        Some(window_write_calls)
    );
    assert_eq!(
        field(windows[0], "device_write_calls"),
        Some(window_write_calls)
    );
    assert_eq!(
        field(windows[0], "by_kind_written_bytes"),
        field(windows[0], "device_written_bytes")
    );
    assert_eq!(
        field(windows[0], "matches"),
        Some("true"),
        "写入口交得出的账与设备一层逐项相等：{}",
        windows[0]
    );
    let stopped = lines_named(lines, "run_failed");
    assert_eq!(stopped.len(), 1, "{lines:#?}");
    assert_eq!(field(stopped[0], "step"), Some(window));
    assert!(
        stopped[0].contains("注入的写错"),
        "停下的原因是注入的那一次写错：{}",
        stopped[0]
    );
}

/// 增补 2 收口表第 58 行，发布 B 那一段：整池第 5 次写报错，前 4 次（头两个单元、每块盘各一份）已落盘。
/// 二进制不直接退出：失败账那 4 次写照打，与设备一层数到的逐项相等，再打停下的原因。
#[test]
fn second_version_publish_failing_midway_reports_the_writes_it_landed_and_they_equal_the_device_layer_count(
) {
    let parameters = e142_parameters(512, 512);
    let geometry = geometry_of(&parameters);
    let stream = SharedStream::new();
    let plan = SharedFaultPlan::unarmed(geometry);
    let mut devices = counted_sparse_devices(&stream, &plan);
    let run = run_new_pool_file_creation(&parameters, &mut devices, &stream, |_point, _devices| {})
        .expect("新池新建文件");
    plan.arm(FaultSchedule::the_nth_call_across_the_pool(
        InjectedFault::WriteFails,
        5,
    ));
    let failed = failed_run_of(
        publish_the_second_version_and_describe(
            &parameters,
            &mut devices,
            &plan,
            &stream,
            &geometry,
            &run,
        ),
        "发布 B",
    );
    assert_the_failed_window_is_reconciled(&failed.lines, "file_overwrite", &[], "4", "4");
    assert!(
        lines_named(&failed.lines, "file_overwrite").is_empty(),
        "失败的发布不打成功那一行"
    );
}

/// 增补 2 收口表第 58 行，发布 C 那一段：可写挂载做完之后装上注入，发布 C 整池第 5 次写报错。
/// 挂载那几行照打在前面，失败账 4 次写与设备一层（重开之后那个注入计划数的）逐项相等。
#[test]
fn third_version_publish_failing_midway_reports_the_writes_it_landed_and_they_equal_the_device_layer_count(
) {
    let parameters = e142_parameters(512, 512);
    let geometry = geometry_of(&parameters);
    let stream = SharedStream::new();
    let plan = SharedFaultPlan::unarmed(geometry);
    let mut devices = counted_sparse_devices(&stream, &plan);
    let run = run_new_pool_file_creation(&parameters, &mut devices, &stream, |_point, _devices| {})
        .expect("新池新建文件");
    let mut allocator = allocator_rebuilt_from_the_records_with_the_root_ring_of_this_process(
        &parameters,
        &devices,
        &run,
    );
    publish_the_second_version(&parameters, &mut devices, &mut allocator, &run.output)
        .expect("发布 B");
    let mut clock = SegmentClock::start();
    let mounted = match reopen_and_mount_writable(
        &parameters,
        devices,
        &stream,
        &geometry,
        SharedFaultPlan::unarmed(geometry),
        &mut clock,
        reopen_sparse_devices,
    ) {
        Ok(mounted) => mounted,
        Err(failed) => panic!("可写挂载：{failed:?}"),
    };
    assert!(mounted.mount_window_matches_device);
    mounted
        .plan
        .arm(FaultSchedule::the_nth_call_across_the_pool(
            InjectedFault::WriteFails,
            5,
        ));
    let failed = failed_run_of(
        publish_the_third_version_and_describe(
            &parameters,
            mounted,
            &stream,
            &geometry,
            &mut clock,
        ),
        "发布 C",
    );
    assert_eq!(
        lines_named(&failed.lines, "writable_mount").len(),
        1,
        "挂载那一行在失败那几行前面照打：{:#?}",
        failed.lines
    );
    assert_the_failed_window_is_reconciled(&failed.lines, "third_transaction", &[], "4", "4");
    assert!(lines_named(&failed.lines, "third_transaction").is_empty());
}

/// 暖机与新池新建文件的分界不是从流尾往回数几步：`ScenarioPoint::BeforeNewPoolFileCreation` 处量到的流长度，
/// 与 `new_pool_file_creation_publish.rs` 的 `warm_up_operation_count`（该文件第 189、201 行）量的是同一处
/// ——暖机 `warm_up()` 已调完、`publish_first_file()` 还没调。这里在内存盘上走同一条路径，用它切出五段，
/// 钉住该文件那五处断言的操作数：27、2、20、35、57（mkfs 清 [0, 根环基址)（C651（没有可用槽 0 时 checker 与实现找槽 1 读法不同））之后每块盘多一步整段清零：mkfs 23 → 25；
/// 两趟写序（D22（单元原子性怎么合成） 已定项 8 第 3 条）在 [0, 根环基址) 清零之后加作废那道屏障、两块盘各记一步：mkfs 25 → 27；C577（系统配置没见证到的最新根，乙罩不到） 之后每次发布末尾轮换之后多一道池屏障、两块盘各记一步：暖机两次发布
/// 18 → 20——第一次末尾那一道顶替了第二次开头那道没有写在前的屏障、只多第二次末尾那 2 步——、新池新建文件 33 → 35，
/// mkfs 之后整条 2 + 20 + 35 = 57）；取号写两轮（D22（单元原子性怎么合成） 已定项 8）之后取号多最后一轮两写与头一轮之后那道屏障
/// 两块盘各一步、2 → 6（最后一轮之后那道屏障归暖机，暖机照旧 20），mkfs 之后整条 6 + 20 + 35 = 61。
#[test]
fn the_boundary_marked_before_the_new_pool_file_creation_splits_warm_up_from_the_transaction() {
    let parameters = e142_parameters(512, 512);
    let geometry = geometry_of(&parameters);
    let stream = SharedStream::new();
    let plan = SharedFaultPlan::unarmed(geometry);
    let mut devices = counted_sparse_devices(&stream, &plan);
    let mut operation_count_before_new_pool_file_creation: Option<usize> = None;
    let run =
        run_new_pool_file_creation(
            &parameters,
            &mut devices,
            &stream,
            |point, _devices| match point {
                ScenarioPoint::AfterMakeFilesystem | ScenarioPoint::AfterInstanceAcquisition => {}
                ScenarioPoint::BeforeNewPoolFileCreation => {
                    operation_count_before_new_pool_file_creation = Some(stream.operation_count());
                }
            },
        )
        .expect("这条路没有摆注入故障，走得到底");
    let boundary = operation_count_before_new_pool_file_creation
        .expect("run_new_pool_file_creation 在新池新建文件之前叫过一次");
    let operations = stream.operations();
    let paths = new_pool_file_creation_paths(&operations, run.mkfs_operation_count, boundary);
    let lengths: Vec<usize> = paths.into_iter().map(|(_, slice)| slice.len()).collect();
    assert_eq!(
        lengths,
        vec![27, 6, 20, 35, 61],
        "mkfs / instance_acquisition / warm_up / transaction / post_mkfs_stream 五段的操作数：\
             与 new_pool_file_creation_publish.rs 断言的五个数逐字相同"
    );
}

/// 新池新建文件那条路：新池新建文件整池第 5 次写报错 ⇒ 失败账 4 次写，与设备一层逐项相等。
/// 暖机第二次空发布的根槽写（整池第 8 次写：第一次空发布 5 次，第二次的两条 journal 记录之后）报错 ⇒
/// 已经落盘的第一次空发布 5 次写由 `WarmUpFailed` 交回、失败账第二次那 2 次写，两样相加 7 次与设备一层逐项相等
/// （增补 2 收口表第 58 行：第一次空发布的账随错交回、两边的数都判）。
#[test]
fn new_pool_file_creation_path_failures_report_every_publish_of_the_failed_step_and_they_equal_the_device_layer_count(
) {
    let parameters = e142_parameters(512, 512);
    let geometry = geometry_of(&parameters);
    for (armed_at, failing_write, step) in [
        (
            ScenarioPoint::BeforeNewPoolFileCreation,
            5,
            NewPoolFileCreationPathStep::NewPoolFileCreation,
        ),
        (
            ScenarioPoint::AfterInstanceAcquisition,
            8,
            NewPoolFileCreationPathStep::WarmUp,
        ),
    ] {
        let stream = SharedStream::new();
        let plan = SharedFaultPlan::unarmed(geometry);
        let mut devices = counted_sparse_devices(&stream, &plan);
        let mut counts_after_acquisition: Option<Vec<DeviceCallCounts>> = None;
        let mut counts_before_new_pool_file_creation: Option<Vec<DeviceCallCounts>> = None;
        let result =
            run_new_pool_file_creation(&parameters, &mut devices, &stream, |point, devices| {
                let counts = device_call_counts(devices, &plan);
                let arm_here = matches!(
                    (&point, &armed_at),
                    (
                        ScenarioPoint::AfterInstanceAcquisition,
                        ScenarioPoint::AfterInstanceAcquisition
                    ) | (
                        ScenarioPoint::BeforeNewPoolFileCreation,
                        ScenarioPoint::BeforeNewPoolFileCreation
                    )
                );
                match point {
                    ScenarioPoint::AfterMakeFilesystem => {}
                    ScenarioPoint::AfterInstanceAcquisition => {
                        counts_after_acquisition = Some(counts);
                    }
                    ScenarioPoint::BeforeNewPoolFileCreation => {
                        counts_before_new_pool_file_creation = Some(counts);
                    }
                }
                if arm_here {
                    plan.arm(FaultSchedule::the_nth_call_across_the_pool(
                        InjectedFault::WriteFails,
                        failing_write,
                    ));
                }
            });
        let failure = match result {
            Ok(_) => panic!("{step:?}：注入的写错该让整条路停下"),
            Err(failure) => failure,
        };
        assert_eq!(failure.failed_step, step);
        let lines = describe_new_pool_file_creation_path_failure(
            &failure,
            counts_after_acquisition.as_deref(),
            counts_before_new_pool_file_creation.as_deref(),
            &device_call_counts(&devices, &plan),
        );
        match step {
            NewPoolFileCreationPathStep::NewPoolFileCreation => {
                assert_the_failed_window_is_reconciled(
                    &lines,
                    "new_pool_file_creation",
                    &[],
                    "4",
                    "4",
                );
            }
            NewPoolFileCreationPathStep::WarmUp => {
                assert_the_failed_window_is_reconciled(&lines, "warm_up", &["5"], "2", "7");
            }
            NewPoolFileCreationPathStep::MakeFilesystem
            | NewPoolFileCreationPathStep::InstanceAcquisition => {
                unreachable!("这两格没有摆")
            }
        }
    }
}

/// 可写挂载失败（增补 2 收口表第 58 行）：挂载把这次写入口的账随 `MountError::Publish` 交回，窗口从取号那道屏障算起，
/// 与设备一层逐项相等。发布 B 之后冷重开，注入摆在重开之后整池第 N 次写（取号写两轮，四次系统配置槽写是第 1 到 4 次）：
/// 第 5 次是写行那次发布的第一个写——没有已经落盘的发布、失败账 0 次写、窗口 0 次；
/// 第 55 次是第二次暖机的第 3 个写——写行 27 次与第一次暖机 21 次已经落盘、失败账 2 次写、窗口 50 次
/// （只写一轮取号时这两格摆在第 3、53 次）。
/// 写行 27 = 实例表两盘 2、分配记录树 14、记账 / 映射 / 树表各两盘 6、记录两盘 2、根槽 1、系统配置轮换 2；
/// 暖机 21 = 分配记录树 10、记账 / 映射 / 树表 6、记录 2、根槽 1、系统配置 2（分配记录树按位置寻址之后每次重写几个节点，
/// D8（核心索引结构） 已定项 14；之前那棵树只有一个节点，写行 15、暖机 13、这一格摆在第 33 次）。
#[test]
fn failed_writable_mount_reports_every_publish_it_wrote_and_they_equal_the_device_layer_count() {
    let parameters = e142_parameters(512, 512);
    let geometry = geometry_of(&parameters);
    for (failing_write, persisted_write_calls, failed_write_calls, window_write_calls) in
        [(5, &[][..], "0", "0"), (55, &["27", "21"][..], "2", "50")]
    {
        let stream = SharedStream::new();
        let plan = SharedFaultPlan::unarmed(geometry);
        let mut devices = counted_sparse_devices(&stream, &plan);
        let run =
            run_new_pool_file_creation(&parameters, &mut devices, &stream, |_point, _devices| {})
                .expect("新池新建文件");
        let mut allocator = allocator_rebuilt_from_the_records_with_the_root_ring_of_this_process(
            &parameters,
            &devices,
            &run,
        );
        publish_the_second_version(&parameters, &mut devices, &mut allocator, &run.output)
            .expect("发布 B");
        let mut clock = SegmentClock::start();
        let failed = failed_run_of(
            reopen_and_mount_writable(
                &parameters,
                devices,
                &stream,
                &geometry,
                SharedFaultPlan::armed(
                    geometry,
                    FaultSchedule::the_nth_call_across_the_pool(
                        InjectedFault::WriteFails,
                        failing_write,
                    ),
                ),
                &mut clock,
                reopen_sparse_devices,
            ),
            "可写挂载",
        );
        assert_the_failed_window_is_reconciled(
            &failed.lines,
            "reopen_and_writable_mount",
            persisted_write_calls,
            failed_write_calls,
            window_write_calls,
        );
    }
}

/// `second-instance` 那条路在宿主上走到发布 C（实例 2，txg 8）：稀疏内存盘代替 virtio 盘，「冷重开」把镜像交给新句柄。
/// 交回这一段的注入计划还没装注入。
fn second_instance_run_up_to_the_third_version(
    parameters: &singlefs_core::make_filesystem::MakeFilesystemParameters,
    geometry: &FixedGeometry,
    stream: &SharedStream,
    clock: &mut SegmentClock,
) -> SecondInstanceRun<SparseBlockDevice> {
    let plan = SharedFaultPlan::unarmed(*geometry);
    let mut devices = counted_sparse_devices(stream, &plan);
    let run = run_new_pool_file_creation(parameters, &mut devices, stream, |_point, _devices| {})
        .expect("新池新建文件");
    let mut allocator = allocator_rebuilt_from_the_records_with_the_root_ring_of_this_process(
        parameters, &devices, &run,
    );
    publish_the_second_version(parameters, &mut devices, &mut allocator, &run.output)
        .expect("发布 B");
    let switched = match switch_instance_and_publish_third_version(
        parameters,
        devices,
        stream,
        geometry,
        clock,
        reopen_sparse_devices,
    ) {
        Ok(switched) => switched,
        Err(failed) => panic!("第二个实例：{failed:?}"),
    };
    assert_eq!(
        switched.current_version.root.checkpoint_txg,
        CheckpointTxg(8),
        "发布 C"
    );
    switched
}

/// `raise-rollback-floor` 模式在宿主上照同一条路跑一遍（增补 2 收口表第 58 行「二进制那一侧判相等」，成功那一路）：
/// 发布 C（txg 8）之后同一次挂载里发布 D（txg 9，第四版），再把 F 抬到上限。发布 D 之后按新到旧数非空有效根是 D、C、B、A（txg 9、8、4、3），
/// 上限 = min(每块盘上最新的有效根, 第 4 新的非空有效根) = min(盘 1 的 txg 7, txg 3) = 3（D16（发布语义） 已定项 1「抬 F 的上限」；
/// 根环三个区域的设备归属是 0 / 1 / 0，按 txg 轮流落，盘 0 上最新的是 txg 9、盘 1 上是 txg 7）。推两次带 F = 3 的空发布 txg 10、11，
/// 两块盘各落一条（生效）。F 的两个数排在最前：没有发布 D 时第 4 新的非空有效根够不到 A，上限是 0，红在这里。
/// 发布 D 一行、抬 F 一行、三次发布各一行 `name=publish_writes`、两段窗口按种类的合计与设备一层逐项相等；录制流投到每块盘上的写与 FLUSH
/// 与设备一层数到的相等（宿主检查拿这个投法当程序的信念，发布 D 与抬 F 两段也一样）；冷恢复择 (2, 11) 读回第四版。
#[test]
fn raise_rollback_floor_mode_publishes_the_fourth_version_raises_the_floor_to_its_ceiling_above_zero_and_every_window_matches_the_device_layer_count(
) {
    let parameters = e142_parameters(512, 512);
    let geometry = geometry_of(&parameters);
    let stream = SharedStream::new();
    let mut clock = SegmentClock::start();
    let switched =
        second_instance_run_up_to_the_third_version(&parameters, &geometry, &stream, &mut clock);
    let lines_before_the_fourth_version = switched.lines.len();
    let operations_before_the_fourth_version = stream.operation_count();
    let counts_before_the_fourth_version = device_call_counts(&switched.devices, &switched.plan);
    let raised = match publish_the_fourth_version_and_raise_the_rollback_floor(
        &parameters,
        switched,
        &stream,
        &geometry,
        &mut clock,
    ) {
        Ok(raised) => raised,
        Err(failed) => panic!("发布 D、抬 F：{failed:?}"),
    };
    let new_lines = &raised.lines[lines_before_the_fourth_version..];

    let raise_line = lines_named(new_lines, "raise_rollback_floor");
    assert_eq!(raise_line.len(), 1, "{new_lines:#?}");
    let raise_line = raise_line[0];
    assert_eq!(
        field(raise_line, "ceiling"),
        Some("3"),
        "第 4 新的非空有效根是 A（txg 3），比盘 1 上最新的 txg 7 小：{raise_line}"
    );
    assert_eq!(
        field(raise_line, "requested_floor"),
        Some("3"),
        "F 抬到上限：{raise_line}"
    );
    assert_eq!(raised.current_version.root.rollback_floor, CheckpointTxg(3));
    assert_eq!(field(raise_line, "floor_before"), Some("0"), "{raise_line}");
    assert_eq!(field(raise_line, "publishes"), Some("2"), "{raise_line}");
    assert_eq!(
        field(raise_line, "root_txgs"),
        Some("10,11"),
        "{raise_line}"
    );
    assert_eq!(
        field(raise_line, "reclaimed"),
        Some("1"),
        "释放代 ≤ 3 的已释放落点（两盘同槽，按盘 0 报）：{raise_line}"
    );
    assert_eq!(
        raised.current_version.root.checkpoint_txg,
        CheckpointTxg(11)
    );

    let fourth_lines = lines_named(new_lines, "fourth_transaction");
    assert_eq!(fourth_lines.len(), 1, "{new_lines:#?}");
    assert_eq!(field(fourth_lines[0], "root_txg"), Some("9"));
    assert_eq!(
        field(fourth_lines[0], "transaction"),
        Some("2"),
        "同一个实例里发布 C 之后的下一个事务号"
    );
    assert_eq!(
        field(fourth_lines[0], "segments"),
        Some("28+2+1+2"),
        "发布 D 与发布 C 同型（单元写那一段怎么数见 second-instance 那条用例）"
    );

    let publish_lines: Vec<(Option<&str>, Option<&str>)> = lines_named(new_lines, "publish_writes")
        .into_iter()
        .map(|line| (field(line, "publish"), field(line, "txg")))
        .collect();
    assert_eq!(
        publish_lines,
        vec![
            (Some("fourth_transaction"), Some("9")),
            (Some("raise_rollback_floor"), Some("10")),
            (Some("raise_rollback_floor"), Some("11")),
        ]
    );
    let windows = lines_named(new_lines, "publish_writes_against_device");
    let window_summary: Vec<(Option<&str>, Option<&str>, Option<&str>)> = windows
        .iter()
        .map(|line| {
            (
                field(line, "window"),
                field(line, "publishes"),
                field(line, "matches"),
            )
        })
        .collect();
    assert_eq!(
        window_summary,
        vec![
            (Some("fourth_transaction"), Some("1"), Some("true")),
            (Some("raise_rollback_floor"), Some("2"), Some("true")),
        ],
        "发布 D 与抬 F 那一串空发布按种类的合计都等于设备一层数的：{windows:#?}"
    );
    for window in &windows {
        assert_eq!(
            field(window, "by_kind_write_calls"),
            field(window, "device_write_calls"),
            "{window}"
        );
        assert_eq!(
            field(window, "by_kind_written_bytes"),
            field(window, "device_written_bytes"),
            "{window}"
        );
    }
    assert!(raised.every_window_matches_device);
    assert_eq!(
        clock.marked_segments(),
        [
            "reopen_and_writable_mount",
            "third_transaction",
            "fourth_transaction",
            "raise_rollback_floor"
        ]
    );
    assert_eq!(
        lines_named(new_lines, "segment_timing").len(),
        2,
        "发布 D 与抬 F 各自的分段行：{new_lines:#?}"
    );

    let operations = stream.retained_operations();
    let new_operations = &operations[operations_before_the_fourth_version..];
    let counts_after_the_raise = device_call_counts(&raised.devices, &raised.plan);
    // C577 之后发布 D 以一道池屏障收尾，抬 F 那一串开的新写入口开头又一道，两道之间没有写：录制器
    // （`SharedStream::push` 的末尾一串屏障去重）把后一道并掉，设备照收两次 FLUSH，所以每块盘设备一层比录制流投出的多 1。
    // 录制器并掉相邻屏障、设备收两次，归第五步 55 号六档重录时定（合入后验证一报告第八节；主 agent 定不改装置、不改判据，照今天的数改钉）。
    // 盘 0 的 12 / 13 是合入后验证一那一次跑出来的；盘 1 同一个机制推的，没跑（singlefs-verification 由主 agent 经门禁 crash-point-replay 跑）。
    let flushes_the_recorder_merged_into_the_barrier_that_closed_the_publish_before_the_floor_raise: u64 = 1;
    for (index, identity) in [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .enumerate()
    {
        let counted = counts_after_the_raise[index].since(counts_before_the_fourth_version[index]);
        let projected = expected_device_events(new_operations, identity);
        let projected_writes = projected
            .iter()
            .filter(|event| matches!(event, DeviceEvent::Write { .. }))
            .count();
        let projected_flushes = projected
            .iter()
            .filter(|event| matches!(event, DeviceEvent::Flush))
            .count();
        assert!(
            counted.write_calls > 0,
            "发布 D 与抬 F 两段盘 {} 收到了写",
            identity.0
        );
        assert_eq!(
            u64::try_from(projected_writes).expect("件数"),
            counted.write_calls,
            "发布 D 与抬 F 两段盘 {}：录制流投出的写与设备一层数到的",
            identity.0
        );
        assert_eq!(
                u64::try_from(projected_flushes).expect("件数")
                    + flushes_the_recorder_merged_into_the_barrier_that_closed_the_publish_before_the_floor_raise,
                counted.barrier_calls + counted.force_unit_access_writes,
                "发布 D 与抬 F 两段盘 {}：录制流投出的 FLUSH 加录制器并掉的那一道，与设备一层转发的屏障 + FUA 写",
                identity.0
            );
    }

    let cold: Vec<(DeviceIdentity, SparseBlockDevice)> = raised
        .devices
        .into_iter()
        .map(|(identity, device)| (identity, device.into_inner_and_operations().0.into_inner()))
        .collect();
    let report = recover(&cold, JournalPolicy::Consult);
    match report.outcome {
        RecoveryOutcome::FileRead { root, content } => {
            assert_eq!(root, (InstanceGeneration(2), CheckpointTxg(11)));
            assert!(content == fourth_file_content(), "冷恢复读回第四版");
        }
        RecoveryOutcome::NoFile { root } => panic!("冷恢复择到 {root:?} 却没有文件"),
        RecoveryOutcome::Failed { root, failure } => {
            panic!("冷恢复失败：{root:?} {failure:?}")
        }
    }
}

/// Z10（`research/prompts/m2-final-code-r2-main-verification.md` 第四节第 5 条）：`raise-rollback-floor` 那条路在宿主上走到
/// 抬 F 之后冷重开，多打的那一行 `name=recover_cold_rollback_floor` 报的是冷重开择到的根 (2, 11) 与它从盘上读回的 F = 3，
/// 不是 `name=raise_rollback_floor` 里内存那一版的 `requested_floor`。同一段历史抬 F 之前的盘面上（发布 C 之后、择到 (2, 8)）读回的是 0：
/// 这一行跟着盘上的根走，不是写死的数。
#[test]
fn raise_rollback_floor_mode_prints_the_floor_read_back_from_the_chosen_root_after_the_cold_reopen()
{
    let parameters = e142_parameters(512, 512);
    let geometry = geometry_of(&parameters);
    let stream = SharedStream::new();
    let mut clock = SegmentClock::start();
    let switched =
        second_instance_run_up_to_the_third_version(&parameters, &geometry, &stream, &mut clock);
    let before_the_raise: Vec<(DeviceIdentity, SparseBlockDevice)> = switched
        .devices
        .iter()
        .map(|(identity, device)| {
            let mut copy =
                SparseBlockDevice::new(SPARSE_DEVICE_BYTES, PhysicalBlockSizeInBytes(512));
            copy.image = device.wrapped_device().wrapped_device().image.clone();
            (*identity, copy)
        })
        .collect();
    let line_before_the_raise =
        super::rollback_floor_read_back_from_the_chosen_root_line(&before_the_raise);
    assert_eq!(
        field(&line_before_the_raise, "chosen_root"),
        Some("2:8"),
        "发布 C 之后择到 (2, 8)：{line_before_the_raise}"
    );
    assert_eq!(
        field(&line_before_the_raise, "rollback_floor_on_disk"),
        Some("0"),
        "抬 F 之前盘上的 F 是 0：{line_before_the_raise}"
    );
    let raised = match publish_the_fourth_version_and_raise_the_rollback_floor(
        &parameters,
        switched,
        &stream,
        &geometry,
        &mut clock,
    ) {
        Ok(raised) => raised,
        Err(failed) => panic!("发布 D、抬 F：{failed:?}"),
    };
    let cold: Vec<(DeviceIdentity, SparseBlockDevice)> = raised
        .devices
        .into_iter()
        .map(|(identity, device)| (identity, device.into_inner_and_operations().0.into_inner()))
        .collect();
    let line = super::rollback_floor_read_back_from_the_chosen_root_line(&cold);
    assert_eq!(
        field(&line, "name"),
        Some("recover_cold_rollback_floor"),
        "{line}"
    );
    assert_eq!(
        field(&line, "chosen_root"),
        Some("2:11"),
        "冷重开择到最后一次抬 F 的空发布的根：{line}"
    );
    assert_eq!(
        field(&line, "rollback_floor_on_disk"),
        Some("3"),
        "所选根从盘上读回的 F 是抬到的上限 3：{line}"
    );
}

/// 增补 2 收口表第 58 行，发布 D 那一段（`raise-rollback-floor`）：发布 C 之后装上注入，发布 D 整池第 5 次写报错。
/// 挂载与发布 C 那几行照打在前面，失败账 4 次写与设备一层（重开之后那个注入计划数的）逐项相等；发布 D 没做成就不抬 F。
#[test]
fn fourth_version_publish_failing_midway_reports_the_writes_it_landed_and_they_equal_the_device_layer_count(
) {
    let parameters = e142_parameters(512, 512);
    let geometry = geometry_of(&parameters);
    let stream = SharedStream::new();
    let mut clock = SegmentClock::start();
    let switched =
        second_instance_run_up_to_the_third_version(&parameters, &geometry, &stream, &mut clock);
    switched
        .plan
        .arm(FaultSchedule::the_nth_call_across_the_pool(
            InjectedFault::WriteFails,
            5,
        ));
    let failed = failed_run_of(
        publish_the_fourth_version_and_raise_the_rollback_floor(
            &parameters,
            switched,
            &stream,
            &geometry,
            &mut clock,
        ),
        "发布 D",
    );
    for earlier_line in ["writable_mount", "third_transaction"] {
        assert_eq!(
            lines_named(&failed.lines, earlier_line).len(),
            1,
            "{earlier_line} 那一行在失败那几行前面照打：{:#?}",
            failed.lines
        );
    }
    assert_the_failed_window_is_reconciled(&failed.lines, "fourth_transaction", &[], "4", "4");
    assert!(lines_named(&failed.lines, "fourth_transaction").is_empty());
    assert!(
        lines_named(&failed.lines, "raise_rollback_floor").is_empty(),
        "发布 D 没做成就不抬 F：{:#?}",
        failed.lines
    );
    assert!(failed.cause.starts_with("发布 D："), "{}", failed.cause);
}

/// 抬 F 失败（增补 2 收口表第 58 行「`raise_rollback_floor` 同形」）：抬 F 的写入口是它自己开的、随错丢掉，
/// 已经落盘的那几次空发布与失败那一次已记的写随 `MountError::RaiseFloorSequencePublishFailed` 交回；`raise-rollback-floor`
/// 模式走的那个函数（`raise_the_rollback_floor_and_describe`）照可写挂载那一段的判法打失败账、与设备一层逐项比。
/// `raise-rollback-floor` 那条路走到发布 D（txg 9），装上注入，把 F 抬到上限（3）——推空发布直到两块盘上都有一条带这个 F 的根。
/// 注入摆在之后整池第 26 次写：先写系统配置那一步（SysPre，D16（发布语义） 已定项 1）两盘各一次、不属于任何一次发布，
/// 第一次空发布（txg 10）21 次写已经落盘（分配记录树按位置寻址重写的节点两盘 10 次、记账 / 映射 / 树表与 journal 记录每盘一份、
/// 根槽一次、系统配置每盘一次），第二次的第 3 个写报错 ⇒ 失败账 2 次写；窗口按种类的合计 2 + 21 + 2 = 25 次与设备一层逐项相等。
/// 前面几段的结果行照打在失败那几行前面，成功那一行不打。（SysPre 之前、分配记录树只有一个节点时这一格摆在第 16 次：空发布 13 次写。）
#[test]
fn failed_raise_of_the_rollback_floor_reports_every_publish_it_wrote_and_they_equal_the_device_layer_count(
) {
    let parameters = e142_parameters(512, 512);
    let geometry = geometry_of(&parameters);
    let stream = SharedStream::new();
    let mut clock = SegmentClock::start();
    let switched =
        second_instance_run_up_to_the_third_version(&parameters, &geometry, &stream, &mut clock);
    let published_the_fourth_version = match publish_the_fourth_version_and_describe(
        &parameters,
        switched,
        &stream,
        &geometry,
        &mut clock,
    ) {
        Ok(published) => published,
        Err(failed) => panic!("发布 D：{failed:?}"),
    };
    published_the_fourth_version
        .plan
        .arm(FaultSchedule::the_nth_call_across_the_pool(
            InjectedFault::WriteFails,
            26,
        ));
    let failed = failed_run_of(
        raise_the_rollback_floor_and_describe(
            &parameters,
            published_the_fourth_version,
            &stream,
            &geometry,
            &mut clock,
        ),
        "抬 F",
    );
    for earlier_line in ["writable_mount", "third_transaction", "fourth_transaction"] {
        assert_eq!(
            lines_named(&failed.lines, earlier_line).len(),
            1,
            "{earlier_line} 那一行在失败那几行前面照打：{:#?}",
            failed.lines
        );
    }
    assert_the_failed_window_is_reconciled(
        &failed.lines,
        "raise_rollback_floor",
        &["21"],
        "2",
        "25",
    );
    assert!(
        lines_named(&failed.lines, "raise_rollback_floor").is_empty(),
        "失败的抬 F 不打成功那一行"
    );
    assert!(failed.cause.starts_with("抬 F："), "{}", failed.cause);
}

/// 宿主检查（`new_pool_file_creation_device_log_check`）拿录制流投到每块盘上的事件当「程序的信念」：每个写一件、FUA 写之后一个 FLUSH、
/// 每道池屏障在每块盘上各一个 FLUSH（录制器把连续几道并成一道）。这个投法在发布 B、可写挂载、发布 C 三段上是否就是设备收到的，
/// 拿录制器外面那一层（注入包装，与录制器不共享计数）数到的核：每块盘、每段，写的件数相等，FLUSH 件数 = 转发的屏障 + FUA 写。
/// 录制器并掉的屏障若在设备上是两次，这里先红，虚机档的逐项比对不会冤判。C577 之后重开那一段开头就有这样一道（段里写明的
/// `flushes_the_recorder_merged`，照今天的数钉），怎么收归第五步 55 号六档重录时定。
#[test]
fn the_recorded_stream_projects_onto_each_device_the_writes_and_flushes_the_device_layer_counted() {
    let parameters = e142_parameters(512, 512);
    let geometry = geometry_of(&parameters);
    let stream = SharedStream::new();
    let plan = SharedFaultPlan::unarmed(geometry);
    let mut devices = counted_sparse_devices(&stream, &plan);
    let run = run_new_pool_file_creation(&parameters, &mut devices, &stream, |_point, _devices| {})
        .expect("新池新建文件");
    let operations_after_the_new_pool_file_creation = stream.operation_count();
    let counts_after_the_new_pool_file_creation = device_call_counts(&devices, &plan);
    let second = publish_the_second_version_and_describe(
        &parameters,
        &mut devices,
        &plan,
        &stream,
        &geometry,
        &run,
    );
    assert!(second.is_ok_and(|second| second.window_matches_device));
    let operations_after_the_file_overwrite = stream.operation_count();
    let counts_after_the_file_overwrite = device_call_counts(&devices, &plan);
    let mut clock = SegmentClock::start();
    let switched = match switch_instance_and_publish_third_version(
        &parameters,
        devices,
        &stream,
        &geometry,
        &mut clock,
        reopen_sparse_devices,
    ) {
        Ok(switched) => switched,
        Err(failed) => panic!("第二个实例：{failed:?}"),
    };
    let operations = stream.retained_operations();
    // 每一段里录制器并掉、设备照收的屏障数（每块盘）：C577 之后发布 B 以一道池屏障收尾，重开之后取号那个新写入口开头又一道，
    // 两道之间没有写，录制器（`SharedStream::push` 的末尾一串屏障去重）把后一道并进上一段，设备在这一段照收一次 FLUSH。
    // 录制器并掉相邻屏障、设备收两次，归第五步 55 号六档重录时定（合入后验证一报告第八节；主 agent 定不改装置、不改判据，照今天的数改钉）。
    // 发布 B 那一段是 0（合入后验证一那一次两块盘都过了）；重开那一段盘 0 的 16 / 17 是那一次跑出来的，盘 1 同一个机制推的，没跑
    // （singlefs-verification 由主 agent 经门禁 crash-point-replay 跑）。
    let windows = [
        (
            "file_overwrite",
            &operations
                [operations_after_the_new_pool_file_creation..operations_after_the_file_overwrite],
            0_u64,
        ),
        (
            "reopen_and_writable_mount_and_third_transaction",
            &operations[operations_after_the_file_overwrite..],
            1,
        ),
    ];
    for (index, identity) in [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .enumerate()
    {
        let file_overwrite_counted = counts_after_the_file_overwrite[index]
            .since(counts_after_the_new_pool_file_creation[index]);
        let after_reopen_counted = DeviceCallCounts::of(
            switched.devices[index]
                .1
                .wrapped_device()
                .plan()
                .counts_of_device(identity),
        );
        for ((window, slice, flushes_the_recorder_merged), counted) in windows
            .iter()
            .zip([file_overwrite_counted, after_reopen_counted])
        {
            let projected = expected_device_events(slice, identity);
            let projected_writes = projected
                .iter()
                .filter(|event| matches!(event, DeviceEvent::Write { .. }))
                .count();
            let projected_flushes = projected
                .iter()
                .filter(|event| matches!(event, DeviceEvent::Flush))
                .count();
            assert!(counted.write_calls > 0, "{window} 盘 {}", identity.0);
            assert_eq!(
                u64::try_from(projected_writes).expect("件数"),
                counted.write_calls,
                "{window} 盘 {}：录制流投出的写与设备一层数到的",
                identity.0
            );
            assert_eq!(
                    u64::try_from(projected_flushes).expect("件数") + flushes_the_recorder_merged,
                    counted.barrier_calls + counted.force_unit_access_writes,
                    "{window} 盘 {}：录制流投出的 FLUSH 加录制器并掉的那几道，与设备一层转发的屏障 + FUA 写",
                    identity.0
                );
        }
    }
}

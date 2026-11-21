//! 驱动两条流：新池新建文件流与抬 F 之后复用再卸载的流，录写表、段表与停机条款 S1 的回比。

use super::*;

// ───────────────────────────── 驱动两条流 ─────────────────────────────

/// 一条停机条款 S1 的回比：本地值与 crates 那一侧的值，两边按文本比。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct StopCheck {
    pub(super) name: String,
    pub(super) local: String,
    pub(super) crates: String,
}

impl StopCheck {
    pub(super) fn new(
        name: &str,
        local: impl std::fmt::Debug,
        crates: impl std::fmt::Debug,
    ) -> Self {
        Self {
            name: name.to_string(),
            local: format!("{local:?}"),
            crates: format!("{crates:?}"),
        }
    }
    pub(super) fn holds(&self) -> bool {
        self.local == self.crates
    }
}

/// 流一的三次调用：取号、暖机、新池新建文件（`tests/common/mod.rs` 的 `build_pool` 那三步），每次调用返回时记流长。
/// mkfs 那一截由 `experiment_pool::format_pool` 做；走完的三样交回给它（`into_built_pool`），镜像仍由池管、池丢掉时删。
fn drive_through_the_new_pool_file_creation(
    tag: &str,
    calls: &mut Vec<RecordedCall>,
) -> experiment_pool::BuiltPool {
    let mut formatted = experiment_pool::format_pool(tag);
    let mut devices = formatted
        .devices
        .take()
        .expect("format_pool 交回开着的设备");
    let stream = formatted.stream.clone();
    let genesis = formatted.genesis.clone();
    let mkfs_operation_count = formatted.mkfs_operation_count;
    let parameters = experiment_pool::parameters();
    let mut allocator = PoolAllocator::new(vec![
        DeviceFreeMap::new(DeviceIdentity(0), LOCAL_DEVICE_BYTES),
        DeviceFreeMap::new(DeviceIdentity(1), LOCAL_DEVICE_BYTES),
    ]);
    allocator.mark_format_time_units(
        Placement {
            slot: INSTANCE_TABLE_SLOT,
            span: 2,
        },
        Placement {
            slot: TREE_TABLE_GENESIS_SLOT,
            span: 1,
        },
    );
    // 照 `build_pool` 装上 mkfs 刚写下的根环（C551：没装的分配器记根就断言）。流一、流二在这个分配器上只记到 txg 4，
    // 环转不过一格，一个落点都不回收，写流与不装时相同。
    allocator.install_root_ring_occupancy(root_ring_occupancy_after_make_filesystem(
        &parameters,
        &genesis,
    ));
    let content = local_first_file_content();
    let (output, warm_up_operation_count) = {
        let mut pool = PoolWriter::new(&parameters, &mut devices);
        let instance = record_call(calls, &stream, mkfs_operation_count, "acquire", || {
            acquire_instance(&mut pool)
        })
        .expect("取号");
        assert_eq!(instance, InstanceGeneration(1), "新池第一次取号取 1");
        let warmed = record_call(calls, &stream, mkfs_operation_count, "warm_up", || {
            warm_up(&mut pool, &genesis.root, instance)
        })
        .expect("暖机");
        let warm_up_operation_count = stream.operation_count();
        let output = record_call(calls, &stream, mkfs_operation_count, "publish", || {
            publish_first_file(
                &mut pool,
                &mut allocator,
                warmed.roots.last().expect("暖机两代根"),
                FirstFile {
                    content: &content,
                    write_time_seconds: LOCAL_FIXED_WRITE_TIME_SECONDS,
                },
                instance,
                &warmed.last_record_bytes,
            )
        })
        .expect("新池新建文件");
        (output, warm_up_operation_count)
    };
    formatted.devices = Some(devices);
    formatted.into_built_pool(experiment_pool::AfterTheNewPoolFileCreation {
        allocator,
        output,
        warm_up_operation_count,
    })
}

/// 照抄 `tests/crash_enumeration_fixed_script_stream.rs` 的 `overwrite`（写时刻取本地常量 + 60）。
fn overwrite(
    pool: &mut experiment_pool::BuiltPool,
    content: &[u8],
    instance: InstanceGeneration,
) -> TransactionOutput {
    let parameters = experiment_pool::parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
    publish_overwrite(
        &mut writer,
        &mut pool.allocator,
        &pool.output,
        FirstFile {
            content,
            write_time_seconds: LOCAL_FIXED_WRITE_TIME_SECONDS + 60,
        },
        instance,
    )
    .expect("覆盖写")
}

/// 照抄同一份文件第 168–196 行 `unmount_recorded`。
fn unmount_recorded(pool: &mut experiment_pool::BuiltPool) -> UnmountRaisedTheFloor {
    let mut current = PoolVersion::WithFile(std::sync::Arc::new(pool.output.clone()));
    let stream = pool.stream.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let allocator = &mut pool.allocator;
    let unmounted = stream
        .record_entry(RecordedPublishEntry::Unmount, || {
            unmount(
                &experiment_pool::parameters(),
                devices,
                allocator,
                &mut current,
                ShadowLedger::On,
            )
        })
        .expect("正常卸载");
    pool.output = current
        .into_file_version()
        .expect("带文件的一版卸载之后仍带文件");
    match unmounted {
        Unmounted::FloorRaisedToTheCurrentVersion(raised) => raised,
        Unmounted::NothingWrittenOnAVersionWithoutFile {
            current: reported_version,
        } => {
            panic!("带文件的一版上卸载报成「树表 0 条、一个字节都不写」：{reported_version:?}")
        }
    }
}

fn version(instance: u32, checkpoint_txg: u64, content: Vec<u8>) -> PublishedVersion {
    PublishedVersion {
        instance: InstanceGeneration(instance),
        checkpoint_txg: CheckpointTxg(checkpoint_txg),
        content,
    }
}

/// 流二：照抄同一份文件第 218–356 行 `prepare` 里 `Script::ReuseAfterRaisingFloorThenNormalUnmount` 那一支，每次调用返回时记流长。
/// 脚本里那几条 `assert_eq!`（抬 F 两次空发布、E 落回 50176、卸载 txg 18 与 19）改成 S1 的回比，不在这里停。
fn drive_the_reuse_after_raising_the_floor_then_unmount(
    calls: &mut Vec<RecordedCall>,
) -> (
    experiment_pool::BuiltPool,
    Vec<PublishedVersion>,
    Vec<StopCheck>,
) {
    let mut checks = Vec::new();
    let mut pool = drive_through_the_new_pool_file_creation("e170-second-stream", calls);
    let stream = pool.stream.clone();
    let mkfs_operation_count = pool.mkfs_operation_count;
    let second = record_call(calls, &stream, mkfs_operation_count, "publish", || {
        overwrite(&mut pool, &second_content(), InstanceGeneration(1))
    });
    let mut versions = vec![
        version(1, 3, local_first_file_content()),
        version(1, 4, second_content()),
    ];
    pool.output = second;
    // 进程重开：取号 2、写行、暖机两次（txg 5 / 6 落盘 0、txg 7 落盘 1），文件还是第二次的内容。
    let mounted = record_call(
        calls,
        &stream,
        mkfs_operation_count,
        "mount_writable",
        || {
            let mut devices = pool.reopen_recorded();
            let mounted =
                mount_writable(&experiment_pool::parameters(), &mut devices).expect("可写挂载");
            pool.devices = Some(devices);
            mounted
        },
    );
    pool.allocator = mounted.allocator;
    pool.output = mounted
        .current
        .into_file_version()
        .expect("B 之后重开，现行那一版带文件");
    for checkpoint_txg in 5..=7 {
        versions.push(version(2, checkpoint_txg, second_content()));
    }
    let third = record_call(calls, &stream, mkfs_operation_count, "publish", || {
        overwrite(&mut pool, &third_content(), InstanceGeneration(2))
    });
    versions.push(version(2, 8, third_content()));
    pool.output = third;
    // 同一个会话里挂着的时候回退到 A 的根：一次向前发布 D（txg 9 落盘 0），文件回到第一次的内容。
    record_call(calls, &stream, mkfs_operation_count, "roll_back", || {
        let rollback_parameters = experiment_pool::parameters();
        let open_devices = pool.devices.as_mut().expect("镜像还开着");
        roll_back_by_a_forward_publish(
            &rollback_parameters,
            open_devices,
            &mut pool.allocator,
            &mut pool.output,
            RollbackTarget {
                instance: InstanceGeneration(1),
                checkpoint_txg: CheckpointTxg(3),
            },
        )
        .expect("挂着的时候回退到 A")
    });
    versions.push(version(2, 9, local_first_file_content()));
    let mut latest = Vec::new();
    for (checkpoint_txg, seed) in [(10u64, 13usize), (11, 17), (12, 19), (13, 23), (14, 29)] {
        latest = later_content(seed);
        let written = record_call(calls, &stream, mkfs_operation_count, "publish", || {
            overwrite(&mut pool, &latest, InstanceGeneration(2))
        });
        pool.output = written;
        versions.push(version(2, checkpoint_txg, latest.clone()));
    }
    let mut current = pool.output.clone();
    let raised = record_call(calls, &stream, mkfs_operation_count, "raise_floor", || {
        let raise_devices = pool.devices.as_mut().expect("镜像还开着");
        raise_rollback_floor(
            &experiment_pool::parameters(),
            raise_devices,
            &mut pool.allocator,
            &mut current,
            CheckpointTxg(11),
            ShadowLedger::On,
        )
        .expect("抬 F")
    });
    checks.push(StopCheck::new(
        "second_stream_raise_floor_publishes",
        LOCAL_SECOND_STREAM_RAISE_FLOOR_PUBLISHES,
        raised.publishes.len(),
    ));
    pool.output = current;
    for checkpoint_txg in 15..=16 {
        versions.push(version(2, checkpoint_txg, latest.clone()));
    }
    let reuse = record_call(calls, &stream, mkfs_operation_count, "publish", || {
        overwrite(&mut pool, &later_content(31), InstanceGeneration(2))
    });
    checks.push(StopCheck::new(
        "second_stream_e_data_unit_slot",
        LOCAL_SECOND_STREAM_REUSED_VERSION_DATA_UNIT_SLOT,
        reuse.data_pointers[0].locations[0].slot.0,
    ));
    versions.push(version(2, 17, later_content(31)));
    pool.output = reuse;
    let unmounted = record_call(calls, &stream, mkfs_operation_count, "unmount", || {
        unmount_recorded(&mut pool)
    });
    checks.push(StopCheck::new(
        "second_stream_unmount_txgs",
        LOCAL_SECOND_STREAM_UNMOUNT_TXGS.to_vec(),
        unmounted
            .publishes
            .iter()
            .map(|publish| publish.root.checkpoint_txg.0)
            .collect::<Vec<u64>>(),
    ));
    for checkpoint_txg in 18..=19 {
        versions.push(version(2, checkpoint_txg, later_content(31)));
    }
    (pool, versions, checks)
}

/// 一条流驱动完之后的样子：mkfs 之后的基镜像、mkfs 之后的录制流、写表与段、调用表、被判的根、版本表、S1 回比。
pub(super) struct PreparedStream {
    pub(super) name: StreamName,
    pub(super) base: MemoryPool,
    pub(super) operations: Vec<RetainedOperation>,
    pub(super) writes: Vec<RetainedWrite>,
    pub(super) segments: Vec<Vec<usize>>,
    pub(super) calls: Vec<RecordedCall>,
    pub(super) judged_root_index: usize,
    pub(super) versions: Vec<PublishedVersion>,
    pub(super) checks: Vec<StopCheck>,
}

pub(super) fn prepare_stream(name: StreamName) -> PreparedStream {
    let mut calls = Vec::new();
    let (pool, versions_of_the_script, mut checks) = match name {
        StreamName::NewPoolFileCreation => (
            drive_through_the_new_pool_file_creation("e170-first-stream", &mut calls),
            None,
            Vec::new(),
        ),
        StreamName::FixedScriptReuseAfterRaisingFloorThenUnmount => {
            let (pool, versions, checks) =
                drive_the_reuse_after_raising_the_floor_then_unmount(&mut calls);
            (pool, Some(versions), checks)
        }
    };
    let all_operations = pool.retained_operations();
    let mkfs_operation_count = pool.mkfs_operation_count;
    // 照抄 `tests/crash_enumeration_fixed_script_stream.rs` 的 `writes_and_segments_after_mkfs`：入口段（卸载入口发的那几段）一起交给切段（C557）。
    let entry_spans: Vec<RecordedEntrySpan> = pool
        .stream
        .entry_spans()
        .into_iter()
        .map(|span| RecordedEntrySpan {
            entry: span.entry,
            operations: span.operations.start - mkfs_operation_count
                ..span.operations.end - mkfs_operation_count,
        })
        .collect();
    let operations = all_operations[mkfs_operation_count..].to_vec();
    let (writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes_and_entries(
        &operations,
        &entry_spans,
        &experiment_pool::geometry(),
    );
    let base = pool.memory_pool_after_mkfs();
    let non_barrier_operations: Vec<usize> = operations
        .iter()
        .enumerate()
        .filter(|(_, retained)| retained.operation.kind != RecordedOperationKind::Barrier)
        .map(|(stream_index, _)| stream_index)
        .collect();
    let label = name.directory_name();
    checks.push(StopCheck::new(
        &format!("{label}_write_order_is_the_non_barrier_operation_order"),
        true,
        non_barrier_operations == stream_indexes,
    ));
    let (local_segment_lengths, local_writes, local_root_writes): (&[usize], usize, usize) =
        match name {
            StreamName::NewPoolFileCreation => (
                &LOCAL_FIRST_STREAM_SEGMENT_LENGTHS,
                LOCAL_FIRST_STREAM_WRITES,
                LOCAL_FIRST_STREAM_ROOT_WRITES,
            ),
            StreamName::FixedScriptReuseAfterRaisingFloorThenUnmount => (
                &LOCAL_SECOND_STREAM_SEGMENT_LENGTHS,
                LOCAL_SECOND_STREAM_WRITES,
                LOCAL_SECOND_STREAM_ROOT_WRITES,
            ),
        };
    checks.push(StopCheck::new(
        &format!("{label}_segment_lengths"),
        local_segment_lengths.to_vec(),
        segments.iter().map(Vec::len).collect::<Vec<usize>>(),
    ));
    checks.push(StopCheck::new(
        &format!("{label}_writes"),
        local_writes,
        writes.len(),
    ));
    let root_write_count = writes
        .iter()
        .filter(|write| write.kind == StepKind::RootRecordFua)
        .count();
    checks.push(StopCheck::new(
        &format!("{label}_root_writes"),
        local_root_writes,
        root_write_count,
    ));
    checks.push(StopCheck::new(
        &format!("{label}_device_count"),
        LOCAL_DEVICE_COUNT,
        base.devices.len(),
    ));
    checks.push(StopCheck::new(
        &format!("{label}_device_bytes"),
        LOCAL_DEVICE_BYTES,
        base.device_size_in_bytes,
    ));
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("写流里有根槽写");
    let versions = versions_of_the_script.unwrap_or_else(|| {
        // 流一的版本表照 `tests/crash_enumeration_new_pool_file_creation_stream.rs` 的 `single_version`。
        let (instance, checkpoint_txg) =
            singlefs_verification::memory_pool::root_identity_of_write(&writes[judged_root_index]);
        vec![PublishedVersion {
            instance,
            checkpoint_txg,
            content: local_first_file_content(),
        }]
    });
    if name == StreamName::NewPoolFileCreation {
        // 流一的驱动走的是新池新建文件那三步：整条流（含 mkfs）逐条与验证包库的新池新建文件那条路比（不拿实验包自己的造池比自己）。
        let reference = experiment_pool::new_pool_file_creation_of_the_verification_library();
        checks.push(StopCheck::new(
            "new_pool_file_creation_operations_equal_the_verification_scenario",
            true,
            reference.operations == all_operations,
        ));
        checks.push(StopCheck::new(
            "new_pool_file_creation_mkfs_operations_equal_the_verification_scenario",
            mkfs_operation_count,
            reference.mkfs_operation_count,
        ));
        checks.push(StopCheck::new(
            "new_pool_file_creation_warm_up_return_point_equal_the_verification_scenario",
            pool.warm_up_operation_count,
            reference.warm_up_operation_count,
        ));
    }
    drop(pool);
    PreparedStream {
        name,
        base,
        operations,
        writes,
        segments,
        calls,
        judged_root_index,
        versions,
        checks,
    }
}

#[cfg(test)]
mod tests;

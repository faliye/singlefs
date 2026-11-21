//! 观察者在崩溃镜像上用的几样：把搬迁源抹零、读快照与第一个文件，开着整理开关可写挂载一次、核续做把整理与删除推到底。

use super::*;

/// 一张镜像的副本上把这些源落点（两块盘各自那一槽）整槽抹成 0：整理搬走之后源落点进了 defer、不再被任何一版引用，抹掉它不该改任何读出来的内容。
#[must_use]
pub fn image_with_the_relocation_sources_zeroed(
    image: &MemoryPool,
    sources: &[(DeviceIdentity, SlotNumber)],
) -> MemoryPool {
    let mut zeroed = image.clone();
    let operations: Vec<RetainedOperation> = sources
        .iter()
        .map(|(device, slot)| RetainedOperation {
            operation: RecordedOperation {
                device: *device,
                kind: RecordedOperationKind::WriteZeroes,
                offset: singlefs_core::address::DeviceOffsetInBytes(slot.0 * SLOT_BYTES),
                length: SLOT_BYTES,
                content_hash: singlefs_verification::fnv1a_64_of_zeros(SLOT_BYTES),
            },
            contents: None,
        })
        .collect();
    zeroed.apply(&operations);
    zeroed
}

/// 在 `reader` 上读这一版的每个活快照：读出的要是建它那一刻第一个文件的内容。交回读了几个。
///
/// # Panics
/// 有一个活快照读不出、读成没有文件或读出别的内容。
pub fn assert_every_live_snapshot_of_the_version_reads_its_creation_content(
    reader: &dyn PoolReader,
    version: &VersionOfTheCompactionFlow,
    what: &str,
) -> usize {
    for snapshot in &version.live_snapshots {
        let expected = version
            .content_captured_by_the_snapshots
            .as_ref()
            .expect("有活快照的一版记着快照建的那一刻的内容");
        match read_the_first_file_of_a_snapshot(reader, &version.root, *snapshot) {
            Ok(SnapshotFileReading::File(content)) => assert!(
                &content == expected,
                "落到 {:?}（{:?}），{what}：活快照 {snapshot:?} 读出建它那一刻的内容（{} 字节），实际 {} 字节",
                version.version,
                version.kind,
                expected.len(),
                content.len()
            ),
            other => panic!(
                "落到 {:?}（{:?}），{what}：活快照 {snapshot:?} 要读出建它那一刻的文件，实际 {other:?}",
                version.version, version.kind
            ),
        }
    }
    version.live_snapshots.len()
}

/// 在 `reader` 上冷走读这一版的第一个文件：有文件的一版读出它那一刻的内容，没有文件的一版答没有文件。
///
/// # Panics
/// 读不出，或读出的与这一版不同。
pub fn assert_the_first_file_of_the_version_reads_its_content(
    reader: &dyn PoolReader,
    version: &VersionOfTheCompactionFlow,
    what: &str,
) {
    let mut mapping_fallbacks = 0usize;
    let read = walk_to_file(reader, &version.root, &mut mapping_fallbacks);
    assert!(
        read.as_ref()
            .is_ok_and(|content| content == &version.first_file_content),
        "落到 {:?}（{:?}），{what}：第一个文件读出这一版的内容（{:?} 字节），实际 {:?}",
        version.version,
        version.kind,
        version.first_file_content.as_ref().map(Vec::len),
        read.map(|content| content.map(|bytes| bytes.len()))
    );
}

// ============================== 在崩溃镜像上可写挂载一次、续做到底 ==============================

/// 在一张崩溃镜像落成的盘上可写挂载一次之后：挂载交回的东西、挂载发的每一次写（落在崩溃镜像上，按发的次序）、挂完的盘。
pub struct MountedWithTheCompactionSwitchesAfterTheCrash {
    pub mounted: Mounted,
    pub writes_of_the_mount: Vec<RetainedWrite>,
    pub disks_after_the_mount: MemoryPool,
    /// 挂载读过的每一段（连同 journal 记录槽提示问到的段，按发出次序）：交回镜像的观察者按它算读集见证。
    pub read_ranges_of_the_mount: Vec<singlefs_verification::RecordedReadRange>,
}

/// 在 `disks`（一张崩溃镜像落成的池）的副本上可写挂载一次，整理开关同录流（[`compaction_switches_of_the_crash_flows`]）：删除续做把队列走到空，
/// 整理续做把盘上那一条未完成的意图推到删了为止。挂载发的写录在一条新的录制流上，连同挂完的盘交回。
///
/// # Panics
/// 可写挂载做不成：调它的盘面按条款都挂得上。
#[must_use]
pub fn mount_writable_with_the_compaction_switches_on_the_crash_disks(
    disks: &MemoryPool,
) -> MountedWithTheCompactionSwitchesAfterTheCrash {
    attempt_the_writable_mount_with_the_compaction_switches_on_the_crash_disks(disks)
        .unwrap_or_else(|refused| panic!("这张盘上可写挂载做得成：{:?}", refused.refusal))
}

/// 同 [`mount_writable_with_the_compaction_switches_on_the_crash_disks`]，崩溃枚举用例的观察者用：挂载被拒时不 panic，由观察者判这一拒认不认。
///
/// # Errors
/// 可写挂载被拒：交回拒因与挂载用过的录制设备（`mount_transparency::image_after_the_refusal_into_the_undecidable_witness`）。
pub fn attempt_the_writable_mount_with_the_compaction_switches_on_the_crash_disks(
    disks: &MemoryPool,
) -> Result<MountedWithTheCompactionSwitchesAfterTheCrash, Box<WritableMountRefusedOnTheCrashDisks>>
{
    let mount_parameters = parameters();
    let stream = SharedStream::retaining_contents_and_reads();
    let mut devices: Vec<_> = disks
        .devices
        .iter()
        .map(|(identity, sectors)| {
            let mut device = SparseBlockDevice::new(
                disks.device_size_in_bytes,
                PhysicalBlockSizeInBytes(mount_parameters.geometry.physical_block_size),
            );
            device.image = sectors.clone();
            (
                *identity,
                RecordingBlockDevice::with_shared_stream(*identity, device, stream.clone()),
            )
        })
        .collect();
    let mounted = match mount_writable_with_compaction_switches(
        &mount_parameters,
        &mut devices,
        compaction_switches_of_the_crash_flows(),
    ) {
        Ok(mounted) => mounted,
        Err(refusal) => {
            return Err(Box::new(WritableMountRefusedOnTheCrashDisks {
                refusal,
                devices,
                stream,
            }));
        }
    };
    let disks_after_the_mount = MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.wrapped_device().image.clone()))
            .collect(),
        device_size_in_bytes: disks.device_size_in_bytes,
    };
    let (writes_of_the_mount, _segments_of_the_mount) =
        writes_and_segments(&stream.retained_operations(), &geometry());
    Ok(MountedWithTheCompactionSwitchesAfterTheCrash {
        mounted,
        writes_of_the_mount,
        disks_after_the_mount,
        read_ranges_of_the_mount: stream.read_ranges(),
    })
}

/// 可写挂载之后核续做（D8（核心索引结构） 已定项 5「续做是每次恢复都跑的常规动作」）：删除续做把队列走到空；恢复落到的那一版有活的整理意图时，
/// 整理续做推到意图删了为止（停在「没有活意图可续」，最后那一版 logged_ops 空、R 按记录全空），没有时一次都不发；挂完的那一版 logged_ops 空、
/// 删除队列与销毁中的快照都空。挂完的盘上第一个文件读出恢复落到的那一版的内容（续做不改文件内容），活快照读出建它那一刻的内容。
/// 交回整理续做发了几次。
///
/// # Panics
/// 上面任何一样对不上。
pub fn assert_the_mount_continued_the_compaction_and_the_deletions(
    landed: LandedVersionOfTheCompactionFlow<'_>,
    mounted: &MountedWithTheCompactionSwitchesAfterTheCrash,
) -> usize {
    let output = &mounted.mounted.output;
    assert!(
        matches!(
            output.deletion_queue_continuation_outcome,
            DeletionQueueContinuationOutcome::WalkedToEmpty
        ),
        "删除续做把队列走到空：{:?}",
        output.deletion_queue_continuation_outcome
    );
    let compaction_publishes = output
        .compaction_continuation
        .run
        .as_ref()
        .map_or(0, |run| run.versions_published.len());
    let LandedVersionOfTheCompactionFlow::Version(version) = landed else {
        assert_eq!(compaction_publishes, 0, "落到 A 之前：没有要续做的整理");
        assert!(
            mounted.mounted.current.version_of_trees().is_none(),
            "落到 A 之前：挂完照旧没有树"
        );
        return 0;
    };
    let current = mounted
        .mounted
        .current
        .version_of_trees()
        .expect("落到 A 起的一版：挂完有树");
    if version.has_a_live_compaction_intent {
        let run = output
            .compaction_continuation
            .run
            .as_ref()
            .expect("落到的一版有活的整理意图：挂载续做它");
        assert!(
            matches!(
                run.outcome,
                CompactionRunOutcome::Stopped(CompactionRunStop::NoLiveIntentLeftToContinue)
            ),
            "落到 {:?}（{:?}）：整理续做推到意图删了：{:?}",
            version.version,
            version.kind,
            run.outcome
        );
        assert!(
            run.versions_published
                .last()
                .is_some_and(|last| last.logged_operations.is_empty()
                    && !last.units_relocated_by_this_publish.is_empty()),
            "落到 {:?}（{:?}）：整理续做的最后一次是最后一批、同一次发布删意图",
            version.version,
            version.kind
        );
        assert!(
            region_is_empty_by_the_records(
                &current.allocation_records,
                region_of_the_crash_flows()
            ),
            "落到 {:?}（{:?}）：续做之后 R 按记录全空（停机谓词，判决 G4）",
            version.version,
            version.kind
        );
    } else {
        assert_eq!(
            compaction_publishes, 0,
            "落到 {:?}（{:?}）：没有活的整理意图，挂载一次整理都不发（不签新的）",
            version.version, version.kind
        );
    }
    assert!(
        current.logged_operations.is_empty(),
        "落到 {:?}（{:?}）：挂完 logged_ops 空",
        version.version,
        version.kind
    );
    assert!(
        current.deleted_inodes.is_empty() && current.snapshots_being_destroyed.is_empty(),
        "落到 {:?}（{:?}）：挂完删除队列与销毁中的快照都空",
        version.version,
        version.kind
    );
    let after_the_mount = VersionOfTheCompactionFlow {
        version: (current.root.instance, current.root.checkpoint_txg),
        kind: version.kind,
        root: current.root,
        first_file_content: version.first_file_content.clone(),
        live_snapshots: live_snapshot_trees_of(current),
        content_captured_by_the_snapshots: version.content_captured_by_the_snapshots.clone(),
        has_a_live_compaction_intent: false,
        relocation_sources_up_to_this_version: Vec::new(),
    };
    assert_the_first_file_of_the_version_reads_its_content(
        &mounted.disks_after_the_mount,
        &after_the_mount,
        "可写挂载续做之后",
    );
    assert_every_live_snapshot_of_the_version_reads_its_creation_content(
        &mounted.disks_after_the_mount,
        &after_the_mount,
        "可写挂载续做之后",
    );
    compaction_publishes
}

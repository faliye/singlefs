//! 池、发布记录与探测要的那几份字节：建池、剥掉与放回探测用不着的字节。

use super::*;

/// 池上每次发布的 txg 记在两个集合里：用户改动发的（非空发布）与别的（写行、暖机、抬 F、卸载的空发布）。
/// 登记 5.1 滞后量里「空发布的 txg 由装置自己记的历史日志认」。
#[derive(Clone, Debug, Default)]
pub(super) struct PublishLog {
    pub(super) user_change_txgs: BTreeSet<u64>,
    pub(super) empty_publish_txgs: BTreeSet<u64>,
}

pub(super) fn current_version(version: &PoolVersion) -> &TransactionOutput {
    match version {
        PoolVersion::WithFile(output) | PoolVersion::WithTreesAndNoFile(output) => {
            Arc::as_ref(output)
        }
        PoolVersion::WithoutFile(_) => stop_with("current_version_has_no_file", ""),
    }
}

pub(super) fn txg_of_pool_version(version: &PoolVersion) -> u64 {
    match version {
        PoolVersion::WithFile(output) | PoolVersion::WithTreesAndNoFile(output) => {
            output.root.checkpoint_txg.0
        }
        PoolVersion::WithoutFile(output) => output.root.checkpoint_txg.0,
    }
}

impl PublishLog {
    pub(super) fn note_mount(&mut self, output: &MountOutput) {
        self.empty_publish_txgs
            .insert(txg_of_pool_version(&output.row_publish));
        for warm_up in &output.warm_up_publishes {
            self.empty_publish_txgs.insert(txg_of_pool_version(warm_up));
        }
    }

    fn note_floor_raises(&mut self, raises: &[singlefs_core::mount::RaisedFloor]) {
        for raised in raises {
            for publish in &raised.publishes {
                self.empty_publish_txgs
                    .insert(publish.root.checkpoint_txg.0);
            }
        }
    }

    pub(super) fn note_published(&mut self, published: &UserChangePublished) {
        self.note_floor_raises(&published.floor_raises);
        for version in &published.versions_published {
            self.user_change_txgs.insert(version.root.checkpoint_txg.0);
        }
    }

    pub(super) fn note_refusal(&mut self, refusal: &UserChangeRefused) {
        match refusal {
            UserChangeRefused::NoSpaceAfterRaisingTheFloor(refused) => {
                self.note_floor_raises(&refused.floor_raises);
                for version in &refused.versions_published {
                    self.user_change_txgs.insert(version.root.checkpoint_txg.0);
                }
            }
            UserChangeRefused::NoSpaceEvenWithEverythingAFloorRaiseCouldRelease(refused) => {
                self.note_floor_raises(&refused.floor_raises);
                for version in &refused.versions_published {
                    self.user_change_txgs.insert(version.root.checkpoint_txg.0);
                }
            }
            UserChangeRefused::Publish {
                floor_raises,
                versions_published,
                ..
            } => {
                self.note_floor_raises(floor_raises);
                for version in versions_published {
                    self.user_change_txgs.insert(version.root.checkpoint_txg.0);
                }
            }
            UserChangeRefused::NoFileVersionToChange {
                floor_raises,
                versions_published,
                ..
            } => {
                self.note_floor_raises(floor_raises);
                for version in versions_published {
                    self.user_change_txgs.insert(version.root.checkpoint_txg.0);
                }
            }
            UserChangeRefused::DeviceTableOtherThanTheOneOfTheMount { .. }
            | UserChangeRefused::PhysicalBlockSizesProbedOnTheDevicesDifferFromTheSystemConfiguration {
                ..
            }
            | UserChangeRefused::DevicesWithoutASelfVerifiedSystemConfiguration { .. }
            | UserChangeRefused::OwnDeviceNumbersDifferFromTheDeviceTable { .. }
            | UserChangeRefused::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot {
                ..
            }
            | UserChangeRefused::FloorRaiseFailedWhilePushingForSpace(_) => {}
        }
    }
}

/// 一个挂着的池：参数、两块盘、会话和发布日志。
pub(super) struct Pool {
    pub(super) geometry: Geometry,
    pub(super) parameters: MakeFilesystemParameters,
    pub(super) devices: Vec<(DeviceIdentity, Disk<'static>)>,
    pub(super) session: MountedSession,
    pub(super) log: PublishLog,
}

pub(super) fn clone_session(session: &MountedSession) -> MountedSession {
    MountedSession {
        allocator: session.allocator.clone(),
        current: session.current.clone(),
        instance: session.instance,
        shadow_ledger: session.shadow_ledger,
        parameters_and_device_table: session.parameters_and_device_table.clone(),
        compaction: session.compaction.clone(),
    }
}

/// 现行那一版里数据单元的字节和前面各次发布的 journal 记录字节：探测的拷贝用不到，几万个单元时一份就上 GB，拷贝前先摘下来
/// （探测与整写同判那一步用带字节的整份拷贝核「摘掉不改判定」）。
pub(super) struct StrippedBytes {
    data_unit_bytes: Vec<(usize, Arc<[u8]>)>,
    earlier_records: Vec<singlefs_core::transaction::WrittenJournalRecord>,
}

fn current_version_mut(version: &mut PoolVersion) -> &mut TransactionOutput {
    match version {
        PoolVersion::WithFile(output) | PoolVersion::WithTreesAndNoFile(output) => {
            Arc::make_mut(output)
        }
        PoolVersion::WithoutFile(_) => stop_with("current_version_has_no_file", ""),
    }
}

pub(super) fn strip_bytes_the_probe_does_not_need(session: &mut MountedSession) -> StrippedBytes {
    let output = current_version_mut(&mut session.current);
    let mut data_unit_bytes = Vec::new();
    for (index, unit) in output.units.iter_mut().enumerate() {
        if matches!(unit.identity, TransactionUnit::Data(_)) {
            data_unit_bytes.push((index, std::mem::take(&mut unit.bytes)));
        }
    }
    let earlier_records = std::mem::take(&mut output.earlier_records_of_this_publish);
    StrippedBytes {
        data_unit_bytes,
        earlier_records,
    }
}

pub(super) fn restore_stripped_bytes(session: &mut MountedSession, stripped: StrippedBytes) {
    let output = current_version_mut(&mut session.current);
    for (index, bytes) in stripped.data_unit_bytes {
        output.units[index].bytes = bytes;
    }
    output.earlier_records_of_this_publish = stripped.earlier_records;
}

// ---------------------------------------------------------------------------------------------------------------------
// 造池：mkfs → 可写挂载 → 第一个文件（一个单元）→ 重开可写挂载
// ---------------------------------------------------------------------------------------------------------------------

/// 第一个文件发布之后、重开之前（mkfs 同一个进程）量到的锚点（登记第七节 A1、A2）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct FirstFileAnchors {
    pub(super) open_segment_start_slot: Option<u64>,
    pub(super) closed_free_slots_in_the_open_segment_on_device_zero: Option<u64>,
    pub(super) first_data_unit_slot: Option<u64>,
}

fn zero_content(data_units: u64) -> Vec<u8> {
    vec![0u8; usize::try_from(data_units * DATA_UNIT_PAYLOAD_BYTES).expect("内容字节数")]
}

pub(super) fn build_pool(geometry: Geometry) -> (Pool, FirstFileAnchors) {
    let parameters = geometry.parameters();
    let mut devices = geometry.blank_devices();
    if let Err(error) = make_filesystem(&parameters, &mut devices) {
        stop_with(
            "make_filesystem_refused",
            &format!("geometry={} error={error:?}", geometry.name),
        );
    }
    let mut log = PublishLog::default();
    let mut mounted: Mounted = match mount_writable(&parameters, &mut devices) {
        Ok(mounted) => mounted,
        Err(error) => stop_with(
            "first_mount_refused",
            &format!("geometry={} error={error:?}", geometry.name),
        ),
    };
    log.note_mount(&mounted.output);
    let instance = mounted.output.instance;
    let (root, record_bytes) = match &mounted.current {
        PoolVersion::WithoutFile(version) => (version.root, version.record_bytes.clone()),
        PoolVersion::WithFile(_) | PoolVersion::WithTreesAndNoFile(_) => stop_with(
            "new_pool_already_has_a_file",
            &format!("geometry={}", geometry.name),
        ),
    };
    let content = zero_content(1);
    let first = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut mounted.allocator,
            &root,
            FirstFile {
                content: &content,
                write_time_seconds: WRITE_TIME_SECONDS,
            },
            instance,
            &record_bytes,
        )
    };
    let first = match first {
        Ok(first) => first,
        Err(error) => stop_with(
            "first_file_refused",
            &format!("geometry={} error={error:?}", geometry.name),
        ),
    };
    log.user_change_txgs.insert(first.root.checkpoint_txg.0);
    let open_segment = mounted.allocator.open_segment();
    let anchors = FirstFileAnchors {
        open_segment_start_slot: open_segment.map(|slot| slot.0),
        closed_free_slots_in_the_open_segment_on_device_zero: open_segment.map(|segment_start| {
            u64::try_from(
                (0..CLUSTER_SEGMENT_SLOTS)
                    .filter(|offset| {
                        mounted.allocator.devices[0].is_free(SlotNumber(segment_start.0 + offset))
                    })
                    .count(),
            )
            .expect("槽数")
        }),
        first_data_unit_slot: first
            .units
            .iter()
            .find(|unit| matches!(unit.identity, TransactionUnit::Data(_)))
            .map(|unit| unit.slot.0),
    };
    drop(mounted);
    let remounted = match mount_writable(&parameters, &mut devices) {
        Ok(remounted) => remounted,
        Err(error) => stop_with(
            "remount_refused",
            &format!("geometry={} error={error:?}", geometry.name),
        ),
    };
    let (output, session) = MountedSession::of_the_mount(remounted);
    log.note_mount(&output);
    (
        Pool {
            geometry,
            parameters,
            devices,
            session,
            log,
        },
        anchors,
    )
}

// ---------------------------------------------------------------------------------------------------------------------
// 历史里的改动
// ---------------------------------------------------------------------------------------------------------------------

/// 全零内容的缓冲：探测与历史都从它切前缀（虚拟内存，没写过的页读出来全 0，不占常驻内存）。
pub(super) fn zero_bytes_of_units(units: u64) -> &'static [u8] {
    static ZERO_BYTES: std::sync::OnceLock<Vec<u8>> = std::sync::OnceLock::new();
    let all = ZERO_BYTES.get_or_init(|| {
        vec![
            0u8;
            usize::try_from(PROBE_FAR_ABOVE_ANY_JOURNAL_CAP_UNITS * DATA_UNIT_PAYLOAD_BYTES)
                .expect("探测缓冲字节数")
        ]
    });
    let length = usize::try_from(units * DATA_UNIT_PAYLOAD_BYTES).expect("内容字节数");
    assert!(length <= all.len(), "内容比探测缓冲还长：{units} 个单元");
    &all[..length]
}

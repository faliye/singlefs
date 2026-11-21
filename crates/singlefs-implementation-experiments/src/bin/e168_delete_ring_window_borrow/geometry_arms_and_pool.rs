//! 几何、臂与池：盘大小点、账的臂、写进一个文件的池与 checker 的违例。

use super::*;

// ---------------------------------------------------------------------------------------------------------------------
// 几何、臂与池
// ---------------------------------------------------------------------------------------------------------------------

/// 登记 5.7 的盘大小取样点（第一段用到的四个）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DeviceSizePoint {
    /// P0：mkfs 字面最小盘（journal 环 1 MiB，单元区 3 槽）。
    MakeFilesystemSmallest,
    /// P1(S, N)：对每个 N 找的最小可用盘（journal 环 1 MiB）。
    SmallestUsable,
    /// P2：单元区 384 槽、journal 环 6 MiB。
    Narrow,
    /// P4：4 GiB、产品默认环。
    Product,
}

impl DeviceSizePoint {
    pub(super) fn label(self) -> &'static str {
        match self {
            DeviceSizePoint::MakeFilesystemSmallest => "P0",
            DeviceSizePoint::SmallestUsable => "P1",
            DeviceSizePoint::Narrow => "P2",
            DeviceSizePoint::Product => "P4",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Geometry {
    pub(super) point: DeviceSizePoint,
    pub(super) device_bytes: u64,
    pub(super) journal_ring_bytes: u64,
    pub(super) slots_per_region: u64,
}

impl Geometry {
    pub(super) fn label(&self) -> String {
        format!("{}-S{}", self.point.label(), self.slots_per_region)
    }

    pub(super) fn unit_area_slots(&self) -> u64 {
        self.device_bytes / SLOT_BYTES - unit_area_start_slot(self.journal_ring_bytes)
    }

    pub(super) fn unit_areas(&self) -> Vec<UnitAreaOnOneDevice> {
        let area = UnitAreaOnOneDevice {
            start_slot: unit_area_start_slot(self.journal_ring_bytes),
            end_slot_exclusive: self.device_bytes / SLOT_BYTES,
        };
        vec![area; usize::try_from(DEVICE_COUNT).expect("盘数")]
    }
}

/// 登记 5.2 的账那一条轴。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum LedgerArm {
    /// N：待删占用 = 0、已承诺预留 = 0。
    NothingSetAside,
    /// W：已承诺预留 = (3S + 1) × W_max × 16384 × 2。
    RingWindowSetAside,
}

/// 登记 5.2 的被拒处置那一条轴。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RefusalHandling {
    /// A：那一批直接交回拒绝。
    Direct,
    /// B：经会话，落点被拒先推抬 F 的空发布再判，推满仍拒才算拒。
    ThroughTheSession,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Arm {
    pub(super) ledger: LedgerArm,
    pub(super) refusal: RefusalHandling,
}

impl Arm {
    pub(super) fn label(self) -> &'static str {
        match (self.ledger, self.refusal) {
            (LedgerArm::NothingSetAside, RefusalHandling::Direct) => "N-A",
            (LedgerArm::NothingSetAside, RefusalHandling::ThroughTheSession) => "N-B",
            (LedgerArm::RingWindowSetAside, RefusalHandling::Direct) => "W-A",
            (LedgerArm::RingWindowSetAside, RefusalHandling::ThroughTheSession) => "W-B",
        }
    }
}

pub(super) const ALL_ARMS: [Arm; 4] = [
    Arm {
        ledger: LedgerArm::NothingSetAside,
        refusal: RefusalHandling::Direct,
    },
    Arm {
        ledger: LedgerArm::NothingSetAside,
        refusal: RefusalHandling::ThroughTheSession,
    },
    Arm {
        ledger: LedgerArm::RingWindowSetAside,
        refusal: RefusalHandling::Direct,
    },
    Arm {
        ledger: LedgerArm::RingWindowSetAside,
        refusal: RefusalHandling::ThroughTheSession,
    },
];

pub(super) type Devices = Vec<(DeviceIdentity, SparseBlockDevice)>;

pub(super) fn make_filesystem_parameters(geometry: &Geometry) -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: *b"_SFS-E168-RINGW\0",
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size: PHYSICAL_BLOCK_BYTES,
            minimum_input_output_bytes: PHYSICAL_BLOCK_BYTES,
            fixed_structure_slot_spacing: 4096,
            journal_ring_bytes: geometry.journal_ring_bytes,
            root_ring_slots_per_region: RootRingSlotsPerRegion::from_system_configuration_field(
                geometry.slots_per_region,
            )
            .expect("每区槽数只取 4、8、16，都在挂载判的区间里"),
        },
    }
}

pub(super) fn blank_devices(device_bytes: u64) -> Devices {
    (0..DEVICE_COUNT)
        .map(|device_number| {
            (
                DeviceIdentity(u32::try_from(device_number).expect("盘号")),
                SparseBlockDevice::new(
                    device_bytes,
                    PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_BYTES),
                ),
            )
        })
        .collect()
}

/// `data_units` 个数据单元的内容（最后一个单元只写一半净荷）。
pub(super) fn content_needing(data_units: u64, seed: u64) -> Vec<u8> {
    let payload_bytes = u64::try_from(data_unit_payload_capacity()).expect("净荷字节数");
    (0..(data_units - 1) * payload_bytes + payload_bytes / 2)
        .map(|byte_position| {
            u8::try_from((byte_position * 31 + seed * 7 + 1) % 251).expect("小于 251")
        })
        .collect()
}

/// 挂着的池：A 臂直接拿会话的分配器与现行那一版发，B 臂经会话的 `publish_user_change` 发。
pub(super) struct MountedPool {
    pub(super) geometry: Geometry,
    pub(super) parameters: MakeFilesystemParameters,
    pub(super) devices: Devices,
    pub(super) session: MountedSession,
}

impl MountedPool {
    pub(super) fn current_file_version(&self) -> &TransactionOutput {
        match &self.session.current {
            PoolVersion::WithFile(version) | PoolVersion::WithTreesAndNoFile(version) => {
                std::sync::Arc::as_ref(version)
            }
            PoolVersion::WithoutFile(_) => stop_with(
                "current_version_has_no_file",
                &format!("geometry={}", self.geometry.label()),
            ),
        }
    }

    pub(super) fn image(&self) -> MemoryPool {
        MemoryPool {
            devices: self
                .devices
                .iter()
                .map(|(identity, device)| (*identity, device.image.clone()))
                .collect(),
            device_size_in_bytes: self.geometry.device_bytes,
        }
    }

    pub(super) fn free_slots_of_each_device(&self) -> Vec<u64> {
        self.session
            .allocator
            .devices
            .iter()
            .map(|map| map.free_slots())
            .collect()
    }
}

/// 5.1 第 1、2 步的结局：写成了，或在哪一步被拒。
pub(super) enum PoolWithAFile {
    Written(Box<MountedPool>),
    Refused {
        step: &'static str,
        detail: String,
        /// 被拒的是那次发布的 journal 记录数超过上限（`PublishError::JournalRecordsOfThePublishExceedTheLimit`）时带着条数与上限，
        /// 别的拒绝是 None（登记第十二节修订第 15 条：P1 的环按它放大，找不到盘时它进原因字段）。
        journal_records_over_the_limit: Option<JournalRecordsOverTheLimit>,
    },
}

/// 一次发布的 journal 记录条数与这个环的上限（前者大于后者才会被拒）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct JournalRecordsOverTheLimit {
    pub(super) records_of_the_publish: u64,
    pub(super) record_limit: u64,
}

fn journal_records_over_the_limit_of(error: &PublishError) -> Option<JournalRecordsOverTheLimit> {
    if let PublishError::JournalRecordsOfThePublishExceedTheLimit {
        records_of_the_publish,
        record_limit,
    } = error
    {
        Some(JournalRecordsOverTheLimit {
            records_of_the_publish: *records_of_the_publish,
            record_limit: *record_limit,
        })
    } else {
        None
    }
}

/// 5.1 第 1、2 步：mkfs → 可写挂载（新实例、写行、暖机）→ 第一个文件（一个单元）→（N > 1 时）顺序写成 N 个数据单元 →
/// 关进程、重开可写挂载一次（分配器装上从盘上读来的根环表）。式子那一判开着。
pub(super) fn pool_with_a_file(geometry: Geometry, data_units: u64) -> PoolWithAFile {
    let parameters = make_filesystem_parameters(&geometry);
    let mut devices = blank_devices(geometry.device_bytes);
    if let Err(error) = make_filesystem(&parameters, &mut devices) {
        return PoolWithAFile::Refused {
            step: "make_filesystem",
            detail: format!("{error:?}"),
            journal_records_over_the_limit: None,
        };
    }
    let mut mounted = match mount_writable(&parameters, &mut devices) {
        Ok(mounted) => mounted,
        Err(error) => {
            return PoolWithAFile::Refused {
                step: "first_mount",
                detail: format!("{error:?}"),
                journal_records_over_the_limit: None,
            }
        }
    };
    let instance = mounted.output.instance;
    let (root, record_bytes) = match &mounted.current {
        PoolVersion::WithoutFile(version) => (version.root, version.record_bytes.clone()),
        PoolVersion::WithFile(_) | PoolVersion::WithTreesAndNoFile(_) => stop_with(
            "new_pool_already_has_a_file",
            &format!("geometry={}", geometry.label()),
        ),
    };
    let first_content = content_needing(1, 1);
    let written = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        let first = publish_first_file(
            &mut writer,
            &mut mounted.allocator,
            &root,
            FirstFile {
                content: &first_content,
                write_time_seconds: WRITE_TIME_SECONDS,
            },
            instance,
            &record_bytes,
        );
        match first {
            Err(error) => Err(("first_file", error)),
            Ok(first) if data_units == 1 => Ok(first),
            Ok(first) => {
                let content = content_needing(data_units, 2);
                publish_sequential_write(
                    &mut writer,
                    &mut mounted.allocator,
                    &first,
                    FirstFile {
                        content: &content,
                        write_time_seconds: WRITE_TIME_SECONDS + 60,
                    },
                    instance,
                )
                .map_err(|error| ("sequential_write", error))
            }
        }
    };
    if let Err((step, error)) = written {
        return PoolWithAFile::Refused {
            step,
            detail: format!("{error:?}"),
            journal_records_over_the_limit: journal_records_over_the_limit_of(&error),
        };
    }
    drop(mounted);
    match mount_writable(&parameters, &mut devices) {
        Ok(remounted) => {
            let (_, session) = MountedSession::of_the_mount(remounted);
            PoolWithAFile::Written(Box::new(MountedPool {
                geometry,
                parameters,
                devices,
                session,
            }))
        }
        Err(error) => PoolWithAFile::Refused {
            step: "remount",
            detail: format!("{error:?}"),
            journal_records_over_the_limit: None,
        },
    }
}

/// 一块盘镜像的 sha256：按写过的扇区号次序，每段「扇区号 + 内容的 sha256」拼起来再取一次 sha256。
pub(super) fn device_image_digest(device: &SparseBlockDevice) -> String {
    let mut digests = String::new();
    for (sector, bytes) in device.image.written_sectors() {
        digests.push_str(&format!("{sector}:{};", sha256_hexadecimal(bytes)));
    }
    sha256_hexadecimal(digests.as_bytes())
}

/// 这份装置判池级 checker 的给定状态引擎（进程里一份，第一次用时起；能走两段的都走两段）：每张要判的盘作一张给定状态表，
/// 录入段写进家目录下给定状态表那一类的判定库，判段从库里读回判（配了 GPU 且编进了 `gpu` 特性在卡上判，否则由 CPU 判器判），判过的复用。
/// 产物只收这份装置自己打的行：引擎不调 `finish`，不打收尾行。
///
/// # Panics
/// 引擎起不了（配了 GPU 卡用不上、结论版本与编译时对不上……都不退回 CPU）。
fn given_states_engine_of_this_experiment() -> &'static GivenStatesEngine {
    static ENGINE: OnceLock<GivenStatesEngine> = OnceLock::new();
    ENGINE.get_or_init(|| {
        GivenStatesEngine::start_from_environment(
            GivenStatesItem {
                item_name: "e168_delete_ring_window_borrow::fixed".to_string(),
            },
            &singlefs_verification::crash_point_replay_progress::CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test(),
        )
        .unwrap_or_else(|refusal| panic!("起不了给定状态表的引擎：{refusal}"))
    })
}

/// 池级 checker 判红的不变量（停机条款 S7），「不变量:细节」一条一项：那张盘作一张给定状态表（基线就是那张盘、写表空、一个空持久集合）
/// 交 [`given_states_engine_of_this_experiment`] 两段判。细节是这一趟 CPU 判的才带原句，GPU 判的与从库里读回的是
/// `crash_given_states::POOL_CHECKER_VIOLATION_WITHOUT_ITS_DETAIL` 那一句（只进停机行，产物行只打条数 0）。
///
/// # Panics
/// 引擎判不了这张表（不退回别处判）；这一趟只录、或这张表归别的机器判（实验装置不分两趟跑、不分片）。
pub(super) fn checker_violations(table_name: &str, image: &MemoryPool) -> Vec<String> {
    let judged = given_states_engine_of_this_experiment()
        .judge_the_given_states(&GivenStatesOfATable {
            table_name: table_name.to_string(),
            base: image,
            writes: &[],
            segments: &[],
            seams: &[],
            persisted_sets: vec![Vec::new()],
        })
        .unwrap_or_else(|reason| {
            panic!("给定状态表 {table_name} 判不了（不退回别处判）：{reason}")
        });
    match judged.source {
        GivenStatesConclusionsSource::JudgedOrReadBackOnThisHost => {}
        GivenStatesConclusionsSource::PlaceholderOfATableLeftToAnotherHost
        | GivenStatesConclusionsSource::PlaceholderOfATableRecordedOnly => panic!(
            "实验装置不分只录、只判两趟跑，也不分片：起它时不设 SINGLEFS_CRASH_AMPLIFICATION_STAGE 与 SINGLEFS_CRASH_AMPLIFICATION_SHARE（{table_name}）"
        ),
    }
    judged
        .states
        .iter()
        .flat_map(|state| state.pool_checker_violations())
        .map(|(invariant, detail)| format!("{invariant}:{detail}"))
        .collect()
}

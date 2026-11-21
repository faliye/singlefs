use super::*;

// ───────────────────────────── 量一条意图（登记第五节「计时与计数怎么做」） ─────────────────────────────

/// 一条意图至多几次调用：签发、R 里至多 32 个数据单元一批一个、再留两次余量；超了就是整理停不下来，交回已有的。
const CALLS_OF_ONE_INTENT_AT_MOST: usize = 1 + 64 + 2;

/// 设备臂（问题单第 1 行的两臂）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum DeviceArm {
    Memory,
    RealDisk,
}

impl DeviceArm {
    pub(super) const ALL: [DeviceArm; 2] = [DeviceArm::Memory, DeviceArm::RealDisk];

    pub(super) const fn label(self) -> &'static str {
        match self {
            DeviceArm::Memory => "memory",
            DeviceArm::RealDisk => "real_disk",
        }
    }
}

/// 一次调用的读数。
#[derive(Clone, Debug)]
pub(super) struct CallReading {
    pub(super) kind: CallKind,
    pub(super) outcome: String,
    pub(super) wall_nanoseconds: u64,
    pub(super) cpu_nanoseconds: u64,
    pub(super) is_the_cpu_unreadable: bool,
    pub(super) device_totals: MethodTotals,
    pub(super) per_method: BTreeMap<DeviceMethod, MethodTotals>,
    pub(super) scan_read_bytes_in_the_region: u64,
    pub(super) scan_read_bytes_elsewhere: u64,
    /// 释放前读盘核读这一批搬迁源的字节（两块盘），不进扫描读的两栏（登记第十二节 12.5）。
    pub(super) release_check_read_bytes: u64,
    pub(super) read_bytes_after_the_first_write: u64,
    pub(super) write_bytes: u64,
    pub(super) relocation_copy_bytes: u64,
    pub(super) is_every_destination_written_whole: bool,
    pub(super) relocated_units: u64,
    pub(super) expected_scan_read_bytes: Option<u64>,
    pub(super) batches_observed: u64,
    pub(super) inclusion: InclusionVerdict,
}

impl CallReading {
    pub(super) fn wait_nanoseconds(&self) -> i64 {
        self.device_totals.wait_nanoseconds
    }
}

/// 第二段重放要的材料（只在 `ReplayCapture::On` 时收）：调用之前那一版的重放输入与会话状态、扫描读到的单元字节（批：调用之前从盘上照 R
/// 里的记录读一份；签发：调用之后照记下的读重读一份）、这一次记下的扫描读、搬走的单元的映射 key；另带释放前读盘核那几次读与它们读到的
/// 副本字节（调用之前从每块盘同号槽读一份，按读的次序取；甲段第二个子项重放的就是这些字节，登记第十二节 12.5）。
pub(super) struct CallReplayMaterial {
    pub(super) call_index: usize,
    pub(super) kind: CallKind,
    pub(super) version: VersionAsTheReplayReadsIt,
    pub(super) state: SessionStateForTheChoice,
    pub(super) unit_bytes_by_slot: BTreeMap<u64, Vec<u8>>,
    pub(super) scan_reads_in_the_region: Vec<RecordedRead>,
    pub(super) release_check_reads: Vec<RecordedRead>,
    pub(super) release_check_copies: Vec<Vec<u8>>,
    pub(super) relocated_keys: Vec<Vec<u8>>,
    /// 第 3 次重跑附带段（`function-split`）重放两次克隆要的两版；别的段不收（`None`）。
    pub(super) versions_around_the_call: Option<VersionsAroundTheCall>,
}

/// 一次调用前后的两版（第 3 次重跑登记第五节「第三段怎么量」：克隆调用之前那一版、克隆调用之后发成的那一版）。
pub(super) struct VersionsAroundTheCall {
    pub(super) before: TransactionOutput,
    pub(super) published: Option<TransactionOutput>,
}

/// 收不收第二段的重放材料。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ReplayCapture {
    Off,
    On,
    /// 收第二段的材料，另收调用前后的两版（第 3 次重跑附带段）。
    OnWithTheVersions,
}

/// 一条意图的读数：R、每次调用、挂载没做成的原因、重放材料。
pub(super) struct IntentReading {
    pub(super) region: Option<CompactionRegion>,
    pub(super) calls: Vec<CallReading>,
    pub(super) mount_failure: Option<String>,
    pub(super) replay_material: Vec<CallReplayMaterial>,
}

/// 照这一版 R 那块盘上的未释放记录，从盘上读一份 R 里每个单元的字节（计时窗口外）。
fn unit_bytes_of_the_region<Inner: BlockDevice>(
    devices: &[(DeviceIdentity, TimedBlockDevice<Inner>)],
    region: CompactionRegion,
    records: &[AllocationRecord],
) -> BTreeMap<u64, Vec<u8>> {
    records
        .iter()
        .filter(|record| {
            record.device == region.device()
                && !record.is_released
                && region.contains_slot(record.slot)
        })
        .filter_map(|record| {
            let length = usize::try_from(u64::from(record.span_slots) * LOCAL_SLOT_BYTES)
                .expect("一个单元的字节");
            PoolReader::read(
                devices,
                record.device,
                record.slot.to_device_offset(),
                length,
            )
            .map(|bytes| (record.slot.0, bytes))
        })
        .collect()
}

/// 一次读落在 R 里没有（盘、槽）。
fn read_is_inside(read: &RecordedRead, region: CompactionRegion) -> bool {
    read.device == region.device()
        && region.contains_slot(SlotNumber(read.offset / LOCAL_SLOT_BYTES))
}

/// 一次调用里记下的读分成的四栏（登记第十二节 12.5）。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct ReadsOfOneCall {
    /// 第一次写之前、落在 R 那块盘 R 里、不是释放前读盘核的读（停机条款第 2 条比的就是它们的字节）。
    scan_reads_in_the_region: Vec<RecordedRead>,
    /// 第一次写之前、落在这一批搬迁源的槽上（两块盘同号槽都算）、扣掉区域扫描对 R 那块盘每个槽的第一次读之后剩下的读。
    release_check_reads: Vec<RecordedRead>,
    scan_read_bytes_elsewhere: u64,
    read_bytes_after_the_first_write: u64,
}

/// 把一次调用里记下的读按读的次序分栏。区域扫描对 R 那块盘每个未释放单元只读一遍、读在释放前读盘核之前，所以 R 那块盘上一个槽的
/// 第一次读归扫描；同一个槽在 R 那块盘上再读、而这个槽是这一批的搬迁源，归释放前读盘核；别的盘上搬迁源那几个槽的读全归释放前读盘核
/// （扫描只读 R 那块盘）。不是搬迁源的槽上再读，照旧归扫描，停机条款第 2 条照样看得见。
fn split_the_reads_of_a_call(
    reads: &[RecordedRead],
    region: Option<CompactionRegion>,
    relocated_source_slots: &BTreeSet<u64>,
) -> ReadsOfOneCall {
    let mut split = ReadsOfOneCall::default();
    let mut slots_read_by_the_scan: BTreeSet<u64> = BTreeSet::new();
    for read in reads {
        if !read.is_before_the_first_write {
            split.read_bytes_after_the_first_write += read.length;
            continue;
        }
        let slot = read.offset / LOCAL_SLOT_BYTES;
        let is_on_a_relocated_source = relocated_source_slots.contains(&slot);
        match region {
            Some(scanned_region) if read_is_inside(read, scanned_region) => {
                let is_the_first_read_of_the_slot = slots_read_by_the_scan.insert(slot);
                if is_the_first_read_of_the_slot || !is_on_a_relocated_source {
                    split.scan_reads_in_the_region.push(*read);
                } else {
                    split.release_check_reads.push(*read);
                }
            }
            Some(_) | None => {
                if is_on_a_relocated_source {
                    split.release_check_reads.push(*read);
                } else {
                    split.scan_read_bytes_elsewhere += read.length;
                }
            }
        }
    }
    split
}

/// 照这一版 R 那块盘上的未释放记录，从每块盘同号槽各读一份（计时窗口外；释放前读盘核读的是两块盘的副本，第二段要这些字节重放它的 CRC）。
fn copies_of_the_region_on_every_device<Inner: BlockDevice>(
    devices: &[(DeviceIdentity, TimedBlockDevice<Inner>)],
    region: CompactionRegion,
    records: &[AllocationRecord],
) -> BTreeMap<(DeviceIdentity, u64), Vec<u8>> {
    let mut copies = BTreeMap::new();
    for record in records.iter().filter(|record| {
        record.device == region.device() && !record.is_released && region.contains_slot(record.slot)
    }) {
        let length = usize::try_from(u64::from(record.span_slots) * LOCAL_SLOT_BYTES)
            .expect("一个单元的字节");
        for (identity, _device) in devices {
            if let Some(bytes) =
                PoolReader::read(devices, *identity, record.slot.to_device_offset(), length)
            {
                copies.insert((*identity, record.slot.0), bytes);
            }
        }
    }
    copies
}

/// 搬走的单元在目的地占几字节：码 1 数据单元与打包单元 32768，码 2 节点 16384（按映射 key 的类标签）。
fn unit_bytes_of_the_mapping_key(mapping_key: &[u8]) -> u64 {
    if mapping_key.first() == Some(&UNIT_CLASS_DATA)
        || mapping_key.first() == Some(&UNIT_CLASS_PACKED)
    {
        LOCAL_DATA_UNIT_BYTES
    } else {
        LOCAL_NODE_BYTES
    }
}

/// 在一组套了计时包装的盘上可写挂载、装上开关，一次一次调 `run_compaction`（每次之前把「已经过去的发布数」设成 1000，一次调用恰好发一次或停），
/// 直到最后一批删了意图、或停了。每次调用：清账、读 `Instant` 与线程 CPU 时钟、调用、再按反的次序读（挂钟读数包着 CPU 读数，理由在 `timed_call`）；调用返回之后（计时窗口外）判这一次是哪一种、拆读写。
pub(super) fn measure_one_intent<Inner: BlockDevice>(
    devices: &mut Vec<(DeviceIdentity, TimedBlockDevice<Inner>)>,
    ledger: &Rc<RefCell<CallLedger>>,
    switches: CompactionSwitches,
    capture: ReplayCapture,
) -> IntentReading {
    let mut reading = IntentReading {
        region: switches.forced_region,
        calls: Vec::new(),
        mount_failure: None,
        replay_material: Vec::new(),
    };
    let mounted = match mount_writable_with_compaction_switches(
        &experiment_pool::parameters(),
        devices,
        switches,
    ) {
        Ok(mounted) => mounted,
        Err(error) => {
            reading.mount_failure = Some(field_token(&format!("{error:?}")));
            return reading;
        }
    };
    for (_identity, device) in devices.iter_mut() {
        device.stop_mirroring_the_journal_writes();
    }
    let (_mount_output, mut session) = MountedSession::of_the_mount(mounted);
    for call_index in 0..CALLS_OF_ONE_INTENT_AT_MOST {
        let records_before = version_of_trees(&session).allocation_records.clone();
        let batches_before = session.compaction.observations.batches_published;
        let captured_before = match capture {
            ReplayCapture::Off => None,
            ReplayCapture::On | ReplayCapture::OnWithTheVersions => Some((
                version_as_the_replay_reads_it(version_of_trees(&session)),
                session_state_for_the_choice(&session),
                reading
                    .region
                    .map(|region| {
                        unit_bytes_of_the_region(devices.as_slice(), region, &records_before)
                    })
                    .unwrap_or_default(),
                reading
                    .region
                    .map(|region| {
                        copies_of_the_region_on_every_device(
                            devices.as_slice(),
                            region,
                            &records_before,
                        )
                    })
                    .unwrap_or_default(),
            )),
        };
        let version_before_the_call = (capture == ReplayCapture::OnWithTheVersions)
            .then(|| version_of_trees(&session).clone());
        session.compaction.publishes_since_the_last_user_change =
            PUBLISHES_ALREADY_PASSED_LEAVING_A_BUDGET_OF_ONE;
        ledger.borrow_mut().open_a_new_window();
        let started = RawWallClockReading::now();
        let cpu_before = thread_cpu_nanoseconds();
        let run = session.run_compaction(devices);
        let cpu_after = thread_cpu_nanoseconds();
        let wall = started.elapsed();
        let (device_totals, per_method, reads, writes, is_the_device_cpu_unreadable) = {
            let window = ledger.borrow();
            (
                window.totals_of_every_method(),
                window.totals.clone(),
                window.reads.clone(),
                window.writes.clone(),
                window.is_the_cpu_unreadable_in_this_window,
            )
        };
        let (cpu_nanoseconds, is_the_call_cpu_unreadable) = match (cpu_before, cpu_after) {
            (Some(before), Some(after)) => (after.saturating_sub(before), false),
            (None, _) | (_, None) => (0, true),
        };
        let wall_nanoseconds = u64::try_from(wall.as_nanos()).expect("一次调用的挂钟装得进 u64");
        let Ok(run) = run else {
            reading.calls.push(CallReading {
                kind: CallKind::NoPublish,
                outcome: "refused_before_any_read_or_write".to_string(),
                wall_nanoseconds,
                cpu_nanoseconds,
                is_the_cpu_unreadable: is_the_call_cpu_unreadable || is_the_device_cpu_unreadable,
                device_totals,
                per_method,
                scan_read_bytes_in_the_region: 0,
                scan_read_bytes_elsewhere: 0,
                release_check_read_bytes: 0,
                read_bytes_after_the_first_write: 0,
                write_bytes: 0,
                relocation_copy_bytes: 0,
                is_every_destination_written_whole: true,
                relocated_units: 0,
                expected_scan_read_bytes: None,
                batches_observed: 0,
                inclusion: inclusion_verdict(
                    wall_nanoseconds,
                    cpu_nanoseconds,
                    device_totals.wait_nanoseconds,
                ),
            });
            break;
        };
        let published = run.versions_published.last();
        let intent_remains = published.is_some_and(|version| !version.logged_operations.is_empty());
        let relocated: Vec<&singlefs_core::transaction::UnitRelocatedByThisPublish> = published
            .map(|version| version.units_relocated_by_this_publish.iter().collect())
            .unwrap_or_default();
        let kind = kind_of_the_call(
            run.versions_published.len(),
            intent_remains,
            relocated.len(),
        );
        if kind == CallKind::Issue && reading.region.is_none() {
            reading.region = session
                .allocator
                .compaction()
                .live_intent()
                .map(|live| live.intent.region);
        }
        let destinations: Vec<(DeviceIdentity, u64, u64)> = relocated
            .iter()
            .flat_map(|unit| {
                let unit_bytes = unit_bytes_of_the_mapping_key(&unit.mapping_key);
                unit.destination.iter().map(move |location| {
                    (
                        location.device,
                        location.slot.to_device_offset().0,
                        unit_bytes,
                    )
                })
            })
            .collect();
        let (relocation_copy_bytes, is_every_destination_written_whole) =
            relocation_copy_bytes(&writes, &destinations);
        let relocated_source_slots: BTreeSet<u64> = relocated
            .iter()
            .flat_map(|unit| unit.source.iter().map(|location| location.slot.0))
            .collect();
        let ReadsOfOneCall {
            scan_reads_in_the_region,
            release_check_reads,
            scan_read_bytes_elsewhere,
            read_bytes_after_the_first_write,
        } = split_the_reads_of_a_call(&reads, reading.region, &relocated_source_slots);
        let scan_read_bytes_in_the_region: u64 = scan_reads_in_the_region
            .iter()
            .map(|read| read.length)
            .sum();
        let release_check_read_bytes: u64 =
            release_check_reads.iter().map(|read| read.length).sum();
        let expected_scan_read_bytes = match kind {
            CallKind::Issue | CallKind::Batch | CallKind::FinalBatch => reading
                .region
                .map(|region| expected_scan_read_bytes(&records_before, region)),
            CallKind::IntentDeletedWithoutABatch | CallKind::NoPublish => None,
        };
        if let (
            ReplayCapture::On | ReplayCapture::OnWithTheVersions,
            Some((version, state, region_bytes, copies_before)),
        ) = (capture, captured_before)
        {
            let release_check_copies: Vec<Vec<u8>> = release_check_reads
                .iter()
                .filter_map(|read| {
                    copies_before
                        .get(&(read.device, read.offset / LOCAL_SLOT_BYTES))
                        .filter(|bytes| length_in_bytes(bytes) == read.length)
                        .cloned()
                })
                .collect();
            let unit_bytes_by_slot = match kind {
                CallKind::Issue => reads
                    .iter()
                    .filter(|read| {
                        read.is_before_the_first_write && read.device == DeviceIdentity(0)
                    })
                    .filter_map(|read| {
                        let length = usize::try_from(read.length).expect("一次读的长度");
                        PoolReader::read(
                            devices.as_slice(),
                            read.device,
                            DeviceOffsetInBytes(read.offset),
                            length,
                        )
                        .map(|bytes| (read.offset / LOCAL_SLOT_BYTES, bytes))
                    })
                    .collect(),
                CallKind::Batch
                | CallKind::FinalBatch
                | CallKind::IntentDeletedWithoutABatch
                | CallKind::NoPublish => region_bytes,
            };
            reading.replay_material.push(CallReplayMaterial {
                call_index,
                kind,
                version,
                state,
                unit_bytes_by_slot,
                scan_reads_in_the_region: scan_reads_in_the_region.clone(),
                release_check_reads: release_check_reads.clone(),
                release_check_copies,
                relocated_keys: relocated
                    .iter()
                    .map(|unit| unit.mapping_key.clone())
                    .collect(),
                versions_around_the_call: version_before_the_call.map(|before| {
                    VersionsAroundTheCall {
                        before,
                        published: published.cloned(),
                    }
                }),
            });
        }
        reading.calls.push(CallReading {
            kind,
            outcome: field_token(&format!("{:?}", run.outcome)),
            wall_nanoseconds,
            cpu_nanoseconds,
            is_the_cpu_unreadable: is_the_call_cpu_unreadable || is_the_device_cpu_unreadable,
            device_totals,
            per_method,
            scan_read_bytes_in_the_region,
            scan_read_bytes_elsewhere,
            release_check_read_bytes,
            read_bytes_after_the_first_write,
            write_bytes: writes.iter().map(|write| write.length).sum(),
            relocation_copy_bytes,
            is_every_destination_written_whole,
            relocated_units: u64::try_from(relocated.len()).expect("一批的单元数"),
            expected_scan_read_bytes,
            batches_observed: session.compaction.observations.batches_published - batches_before,
            inclusion: inclusion_verdict(
                wall_nanoseconds,
                cpu_nanoseconds,
                device_totals.wait_nanoseconds,
            ),
        });
        let is_the_intent_over = match kind {
            CallKind::Issue | CallKind::Batch => {
                matches!(run.outcome, CompactionRunOutcome::Failed(_))
            }
            CallKind::FinalBatch | CallKind::IntentDeletedWithoutABatch | CallKind::NoPublish => {
                true
            }
        };
        if is_the_intent_over {
            break;
        }
    }
    reading
}

#[cfg(test)]
mod tests;

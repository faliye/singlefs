use super::*;

// ============================================================================
// 六、实例 2：在 R_h 的读回全零下可写挂载（取号、写行、暖机），接着 3 次 `Overwrite`；录制流就是要枚举崩溃状态的那条流。
// ============================================================================

/// 被藏根读回全零的字节区间（登记「读法写死」第一条）：R_h 那次发布写下的每个单元、每条 journal 记录、根槽；
/// 「已确认」档再加那次发布轮换写下的系统配置槽（每块盘较新的那一个）。`applied` 是已经落进基镜像的那次发布的录制流。
/// 记录照常读得出的阳性对照族不罩 journal 记录。
pub(super) fn fault_ranges_of_the_last_publish(
    family: Family,
    applied: &[RetainedOperation],
    fixed: &FixedGeometry,
) -> Vec<ReadsZeroRange> {
    if !family.hides_a_root() {
        return Vec::new();
    }
    let records_are_hidden_too = matches!(family, Family::HiddenRoot(_));
    let newer_configuration_is_hidden_too = family.confirmation() == Confirmation::Confirmed;
    applied
        .iter()
        .filter_map(|operation| {
            if !matches!(
                operation.operation.kind,
                RecordedOperationKind::Write | RecordedOperationKind::WriteForceUnitAccess
            ) {
                return None;
            }
            let label = match fixed.classify(&operation.operation) {
                StepKind::UnitWrite => "unit",
                StepKind::JournalRecord if records_are_hidden_too => "journal_record",
                StepKind::JournalRecord => return None,
                StepKind::RootRecordFua => "root_slot",
                StepKind::SystemConfigurationSlot if newer_configuration_is_hidden_too => {
                    "newer_system_configuration_slot"
                }
                StepKind::SystemConfigurationSlot => return None,
                StepKind::ZeroFill | StepKind::Discard | StepKind::Barrier => return None,
            };
            Some(ReadsZeroRange {
                device: operation.operation.device,
                offset: operation.operation.offset.0,
                length: operation.operation.length,
                label,
            })
        })
        .collect()
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct MountReport {
    /// 可写挂载被拒时的错误（拒在取号之前，盘上不动）。
    pub(super) refused: Option<String>,
    /// 挂载做成之后接着做的覆盖写停下的原因（没停下是 `None`）。
    pub(super) stopped_after_the_mount: Option<String>,
    pub(super) instance: Option<u32>,
    pub(super) row_publish_root: Option<RootKey>,
    pub(super) warm_up_publishes: usize,
    pub(super) shadow_ledger_branch: String,
    pub(super) isolated_slots_per_device: Vec<(u32, u64)>,
    pub(super) abandoned_roots_unreadable: u64,
    /// 核心层记着「槽里住着一条被抛弃的根」的根环槽 (区域, 槽)。
    pub(super) ring_slots_recorded_as_holding_an_abandoned_root: Vec<(u64, u64)>,
    /// 读回全零的区间各被读拦了几次，按标签合计。
    pub(super) intercepted_reads_by_label: BTreeMap<String, u64>,
    /// 这个进程之后写盖了的区间（读回全零就此解开），按标签合计。
    pub(super) ranges_written_over_by_label: BTreeMap<String, u64>,
}

pub(super) struct SecondInstance {
    pub(super) operations: Vec<RetainedOperation>,
    pub(super) mount: MountReport,
    /// 基镜像加上整条录制流 = 盘上真有的字节（录制流重建镜像的自检）。
    pub(super) replay_equals_the_devices: bool,
    /// 盘上真有的字节与基镜像逐字节相同（挂载被拒、一个字节没写时为真；A4 用）。
    pub(super) devices_equal_the_base: bool,
}

pub(super) fn run_the_second_instance(
    specification: &HistorySpecification,
    base: &MemoryPool,
    ranges: Vec<ReadsZeroRange>,
) -> SecondInstance {
    let parameters = pool_parameters();
    let stream = SharedStream::retaining_contents();
    let state = Rc::new(RefCell::new(ReadsZeroState::with_ranges(ranges)));
    let mut devices = recording_devices(base, &stream, &state);
    let mut report = MountReport::default();
    let shadow_ledger_is_off = matches!(
        specification.family,
        Family::HiddenRootRecordsReadableShadowLedgerOff(_)
    );
    let mount_result = if shadow_ledger_is_off {
        mount_writable_with_test_only_switches(
            &parameters,
            &mut devices,
            SpaceAdmission::JudgedByTheFormula,
            ShadowLedger::Off,
        )
    } else {
        mount_writable(&parameters, &mut devices)
    };
    match mount_result {
        Err(error) => report.refused = Some(format!("{error:?}")),
        Ok(mounted) => keep_going_after_the_mount(&parameters, &mut devices, mounted, &mut report),
    }
    for (range, (intercepted, written_over)) in {
        let state = state.borrow();
        state
            .ranges
            .iter()
            .cloned()
            .zip(
                state
                    .intercepted_reads
                    .iter()
                    .copied()
                    .zip(state.written_over.iter().copied()),
            )
            .collect::<Vec<_>>()
    } {
        *report
            .intercepted_reads_by_label
            .entry(range.label.to_string())
            .or_insert(0) += intercepted;
        *report
            .ranges_written_over_by_label
            .entry(range.label.to_string())
            .or_insert(0) += u64::from(written_over);
    }
    let operations = stream.retained_operations();
    let mut replayed = base.clone();
    replayed.apply(&operations);
    let on_the_devices = pool_of_the_recorded_devices(devices, base.device_size_in_bytes);
    SecondInstance {
        operations,
        mount: report,
        replay_equals_the_devices: replayed == on_the_devices,
        devices_equal_the_base: &on_the_devices == base,
    }
}

/// 挂载做成之后：记下挂载报的东西，再接着做 3 次 `Overwrite`（录制流接着往下记）。
fn keep_going_after_the_mount(
    parameters: &MakeFilesystemParameters,
    devices: &mut RecordedDevices,
    mounted: Mounted,
    report: &mut MountReport,
) {
    let Mounted {
        output,
        mut allocator,
        current,
    } = mounted;
    report.instance = Some(output.instance.0);
    report.row_publish_root = Some(root_key_of(output.row_publish.root()));
    report.warm_up_publishes = output.warm_up_publishes.len();
    report.shadow_ledger_branch = output.shadow_ledger_branch.to_string();
    report.isolated_slots_per_device = output
        .isolated_slots_per_device
        .iter()
        .map(|(device, slots)| (device.0, *slots))
        .collect();
    report.abandoned_roots_unreadable = output.abandoned_roots_unreadable;
    report.ring_slots_recorded_as_holding_an_abandoned_root = allocator
        .root_ring_occupancy()
        .map(|occupancy| {
            occupancy
                .ring_slots_recorded_as_holding_an_abandoned_root()
                .keys()
                .map(|ring_slot| (ring_slot.region, ring_slot.slot))
                .collect()
        })
        .unwrap_or_default();
    let instance = output.instance;
    let mut previous = match current {
        PoolVersion::WithFile(version) => std::sync::Arc::unwrap_or_clone(version),
        PoolVersion::WithTreesAndNoFile(_) | PoolVersion::WithoutFile(_) => {
            report.stopped_after_the_mount =
                Some("挂载之后现行那一版没有可覆盖写的文件".to_string());
            return;
        }
    };
    for overwrite_index in 0..OVERWRITES_AFTER_THE_MOUNT_LOCAL {
        let content = deterministic_content(1000 + overwrite_index, DATA_UNIT_PAYLOAD_BYTES_LOCAL);
        let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
        match publish_overwrite(
            &mut writer,
            &mut allocator,
            &previous,
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 10_000 + overwrite_index,
            },
            instance,
        ) {
            Ok(next) => previous = next,
            Err(error) => {
                report.stopped_after_the_mount =
                    Some(format!("挂载之后第 {overwrite_index} 次覆盖写：{error:?}"));
                return;
            }
        }
    }
}

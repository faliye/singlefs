//! 观测盘（按调用窗口藏落点、数调用）、崩溃恢复藏掉的落点、随机历史的账本、读回与挂载记录。

use super::*;

// ------------------------------------------------------------------------
// 13.2 盘与一次受观测的入口调用：harness 那一叠盘（`HistoryDevice`）外面包一层，数这次调用打到盘上的读（Q5）、写（K3 / K4），
//      崩溃恢复抛弃根那一步把点名的几段读回全 0（harness `DeviceReadingZerosOverHiddenRanges` 同一造法：读照样交下去、交回之前清零；
//      写到哪一段那一段就不再藏），并数被藏范围被读了几次（V2 对这几族不适用，只报数）。
// ------------------------------------------------------------------------

/// 盘上被藏起来的一段（读回全 0）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct HiddenRange {
    pub(super) device: DeviceIdentity,
    pub(super) offset: u64,
    pub(super) length: u64,
}

impl HiddenRange {
    fn overlaps(self, device: DeviceIdentity, offset: u64, length: u64) -> bool {
        self.device == device && self.offset < offset + length && offset < self.offset + self.length
    }

    /// 装置自己的第一遍读（`device_judgement`）用的那一种写法：读回全零、整次调用都坏。
    pub(super) fn as_third_run_fault(self) -> ThirdRunFault {
        ThirdRunFault {
            device: self.device,
            offset: self.offset,
            length: self.length,
            form: UnreadableForm::ReadsZeros,
            duration: FaultDuration::WholeCall,
            label: format!("hidden(device={},offset={})", self.device.0, self.offset),
        }
    }
}

/// 一次入口调用这一层数到的东西。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct CallWindow {
    pub(super) read_calls: u64,
    pub(super) read_bytes: u64,
    pub(super) write_calls: u64,
    pub(super) reads_over_hidden_ranges: u64,
    pub(super) reads_before_the_first_wait: Option<u64>,
}

pub(super) struct ObservedDevice<Inner: BlockDevice> {
    pub(super) identity: DeviceIdentity,
    inner: Inner,
    pub(super) hidden: Vec<HiddenRange>,
    pub(super) window: Rc<RefCell<CallWindow>>,
}

impl<Inner: BlockDevice> BlockDevice for ObservedDevice<Inner> {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        let length = u64::try_from(buffer.len()).expect("一次读的长度装得进 u64");
        {
            let mut window = self.window.borrow_mut();
            window.read_calls += 1;
            window.read_bytes += length;
        }
        self.inner.read_at(offset, buffer)?;
        let mut read_a_hidden_range = false;
        for hidden in &self.hidden {
            if !hidden.overlaps(self.identity, offset.0, length) {
                continue;
            }
            read_a_hidden_range = true;
            let start_in_buffer = hidden.offset.saturating_sub(offset.0);
            let end_in_buffer = (hidden.offset + hidden.length - offset.0).min(length);
            buffer[usize::try_from(start_in_buffer).expect("落在缓冲区里")
                ..usize::try_from(end_in_buffer).expect("落在缓冲区里")]
                .fill(0);
        }
        if read_a_hidden_range {
            self.window.borrow_mut().reads_over_hidden_ranges += 1;
        }
        Ok(())
    }

    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        let length = u64::try_from(bytes.len()).expect("一次写的长度装得进 u64");
        self.window.borrow_mut().write_calls += 1;
        let identity = self.identity;
        self.hidden
            .retain(|hidden| !hidden.overlaps(identity, offset.0, length));
        self.inner.write_at(offset, bytes, durability)
    }

    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.window.borrow_mut().write_calls += 1;
        let identity = self.identity;
        self.hidden
            .retain(|hidden| !hidden.overlaps(identity, offset.0, length));
        self.inner.write_zeroes_at(offset, length)
    }

    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        self.inner.barrier()
    }

    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }

    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }
}

/// 一次受观测的入口调用交回的读数（Q5、Q7；可写挂载另带臂的观测与等待标记）。
#[derive(Clone, Debug, Default)]
pub(super) struct ObservedCallReading {
    pub(super) window: CallWindow,
    pub(super) waits: u64,
    pub(super) observation: Option<ArmObservation>,
    pub(super) extras: Option<FourthRunArmExtras>,
}

/// 在 `devices` 外面包一层受观测的盘跑 `call`，跑完原样拆回。`installs_the_arm_hooks` = 可写挂载：装等待钩子（记第一个等待标记）、
/// 虚拟时钟归 0、跑完取臂的观测（臂的代码只在副本里有，`fourth_run_arm_hooks`）。
pub(super) fn observed_call<Inner: BlockDevice, Output>(
    devices: &mut Vec<(DeviceIdentity, Inner)>,
    hidden: &[HiddenRange],
    installs_the_arm_hooks: bool,
    call: impl FnOnce(&mut Vec<(DeviceIdentity, ObservedDevice<Inner>)>) -> Output,
) -> (Output, ObservedCallReading) {
    let window = Rc::new(RefCell::new(CallWindow::default()));
    let mut wrapped: Vec<(DeviceIdentity, ObservedDevice<Inner>)> = std::mem::take(devices)
        .into_iter()
        .map(|(identity, inner)| {
            (
                identity,
                ObservedDevice {
                    identity,
                    inner,
                    hidden: hidden
                        .iter()
                        .filter(|range| range.device == identity)
                        .copied()
                        .collect(),
                    window: Rc::clone(&window),
                },
            )
        })
        .collect();
    if installs_the_arm_hooks {
        third_run_arm_hooks::reset_the_virtual_clock();
        let window_for_the_hook = Rc::clone(&window);
        fourth_run_arm_hooks::install_the_marking_wait_hook(
            THIRD_RUN_VIRTUAL_INTERVAL,
            Box::new(move || {
                let mut marked = window_for_the_hook.borrow_mut();
                if marked.reads_before_the_first_wait.is_none() {
                    marked.reads_before_the_first_wait = Some(marked.read_calls);
                }
            }),
        );
    }
    let output = call(&mut wrapped);
    let taken = if installs_the_arm_hooks {
        fourth_run_arm_hooks::take_the_arm_observation_and_its_extras()
    } else {
        None
    };
    let waits = if installs_the_arm_hooks {
        third_run_arm_hooks::waits()
    } else {
        0
    };
    *devices = wrapped
        .into_iter()
        .map(|(identity, device)| (identity, device.inner))
        .collect();
    let window_reading = *window.borrow();
    (
        output,
        ObservedCallReading {
            window: window_reading,
            waits,
            observation: taken.as_ref().map(|(observation, _)| observation.clone()),
            extras: taken.map(|(_, extras)| extras),
        },
    )
}

pub(super) fn history_device_bytes() -> u64 {
    HistoryDeviceWidth::FourGibibytes.device_bytes()
}

/// harness 那一叠盘此刻的整份镜像（harness `image_of` 同一取法）。
pub(super) fn image_of_history_devices(devices: &[(DeviceIdentity, HistoryDevice)]) -> MemoryPool {
    MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| {
                (
                    *identity,
                    device.wrapped_device().wrapped_device().image.clone(),
                )
            })
            .collect(),
        device_size_in_bytes: history_device_bytes(),
    }
}

/// 一份镜像上的两块盘（崩溃状态的分支在它上面做可写挂载与读回）。
pub(super) fn plain_devices_of(image: &MemoryPool) -> Vec<(DeviceIdentity, SparseBlockDevice)> {
    devices_from_pool(image, history_device_bytes())
}

/// 按 checker 的读法读根环：(txg, 实例) 从新到旧、去重（harness `ring_roots_newest_first_of` 的同一取法）。
pub(super) fn ring_roots_newest_first_of_the_image(image: &MemoryPool) -> Vec<(u64, u32)> {
    let Some(geometry) = checker_image::chosen_system_configurations(image)
        .into_iter()
        .find_map(|(_, chosen)| chosen.map(|(_, geometry)| geometry))
    else {
        return Vec::new();
    };
    let mut roots: Vec<(u64, u32)> = checker_image::valid_roots(image, &geometry)
        .into_iter()
        .map(|(_, _, view)| (view.checkpoint_txg, view.instance))
        .collect();
    roots.sort_unstable_by(|left, right| right.cmp(left));
    roots.dedup();
    roots
}

/// 按 checker 的读法：根环里最新那条根带的 F（harness `newest_ring_root_floor` 的同一取法）。
pub(super) fn newest_ring_root_floor_of_the_image(image: &MemoryPool) -> Option<u64> {
    let geometry = checker_image::chosen_system_configurations(image)
        .into_iter()
        .find_map(|(_, chosen)| chosen.map(|(_, geometry)| geometry))?;
    checker_image::valid_roots(image, &geometry)
        .into_iter()
        .map(|(_, _, view)| view)
        .max_by_key(|view| (view.checkpoint_txg, view.instance))
        .map(|view| view.rollback_floor)
}

/// 崩溃恢复要藏起来的那几段（harness `ranges_hidden_by_the_crash_recovery` 的同一取法）：现行版本的根槽，与它那次发布每一条记录
/// 点名的每一份单元的第一个槽。那次发布一个单元都没点名时交回 None。
pub(super) fn ranges_hidden_by_the_crash_recovery(
    parameters: &MakeFilesystemParameters,
    current: &PoolVersion,
) -> Option<Vec<HiddenRange>> {
    let earlier_records = match current {
        PoolVersion::WithoutFile(version) => &version.earlier_records_of_this_publish,
        PoolVersion::WithFile(version) | PoolVersion::WithTreesAndNoFile(version) => {
            &version.earlier_records_of_this_publish
        }
    };
    let named_units: Vec<LocationEntry> = earlier_records
        .iter()
        .map(|written| &written.record)
        .chain(std::iter::once(current.record()))
        .flat_map(|record| record.named.iter())
        .flat_map(|named| named.locations)
        .collect();
    if named_units.is_empty() {
        return None;
    }
    let root_slot = singlefs_core::root_ring::target_for_publish(
        current.root().checkpoint_txg,
        parameters.geometry.root_ring_slots_per_region,
    );
    let root_slot_device =
        parameters.region_devices[usize::try_from(root_slot.region).expect("区域号小于 3")];
    let root_slot_offset = singlefs_core::root_ring::slot_offset(
        root_slot,
        parameters.geometry.fixed_structure_slot_spacing,
    );
    Some(
        std::iter::once(HiddenRange {
            device: root_slot_device,
            offset: root_slot_offset.0,
            length: u64::from(parameters.geometry.physical_block_size),
        })
        .chain(named_units.into_iter().map(|location| HiddenRange {
            device: location.device,
            offset: location.slot.0 * SLOT_BYTES_LOCAL,
            length: SLOT_BYTES_LOCAL,
        }))
        .collect(),
    )
}

// ------------------------------------------------------------------------
// 13.3 确认账（登记「读法写死」表「确认」「内容号跟根走」两行）：内容号是装置的写日志记的，按确认的先后递增，跟着根走；
//      harness 的内容可以重复（空内容），所以读回按根认内容号、再拿字节核（第十二节第二段修订）。管理员回退交回 `Ok` 另记一笔
//      （回退读法，只报数，第十二节第二段修订）。
// ------------------------------------------------------------------------

#[derive(Clone)]
pub(super) struct RandomConfirmation {
    /// 这笔确认落在哪一步（起点 0，第 i 步 i + 1，尾巴 = 步数 + 1）。
    pub(super) order: usize,
    pub(super) content_number: u64,
    pub(super) root: TimelineRoot,
    pub(super) unit_bytes: UnitBytesOfAVersion,
}

#[derive(Clone, Default)]
pub(super) struct RandomLedger {
    /// 内容号 → 那一版的文件字节（None = 没有文件）。内容号 0 是 mkfs 那一版。
    pub(super) contents: Vec<Option<Vec<u8>>>,
    /// 交回过 `Ok` 的内容号（没交回 `Ok` 的改内容调用也发一个号，记它要写的字节，读回落到它算「更新」）。
    pub(super) confirmed_numbers: BTreeSet<u64>,
    pub(super) content_number_by_root: BTreeMap<TimelineRoot, u64>,
    pub(super) confirmations: Vec<RandomConfirmation>,
    /// 管理员回退交回 `Ok` 的那几次：(步, 承接的内容号, 回退那次发布的根)。
    pub(super) rollback_confirmations: Vec<(usize, u64, TimelineRoot)>,
}

impl RandomLedger {
    pub(super) fn after_make_filesystem(genesis: TimelineRoot) -> Self {
        RandomLedger {
            contents: vec![None],
            confirmed_numbers: BTreeSet::from([0]),
            content_number_by_root: BTreeMap::from([(genesis, 0)]),
            confirmations: Vec::new(),
            rollback_confirmations: Vec::new(),
        }
    }

    pub(super) fn carry(&mut self, root: TimelineRoot, content_number: u64) {
        self.content_number_by_root.insert(root, content_number);
    }

    pub(super) fn content_number_of(&self, root: TimelineRoot) -> Option<u64> {
        self.content_number_by_root.get(&root).copied()
    }

    pub(super) fn new_content_number(&mut self, bytes: Option<Vec<u8>>) -> u64 {
        self.contents.push(bytes);
        u64::try_from(self.contents.len() - 1).expect("内容号装得进 u64")
    }

    pub(super) fn confirm(
        &mut self,
        order: usize,
        root_record: &RootRecord,
        content: Vec<u8>,
        image: &MemoryPool,
    ) {
        let content_number = self.new_content_number(Some(content));
        self.confirmed_numbers.insert(content_number);
        let root = root_pair(root_record);
        self.carry(root, content_number);
        self.confirmations.push(RandomConfirmation {
            order,
            content_number,
            root,
            unit_bytes: unit_bytes_referenced_by(image, root_record).unwrap_or_default(),
        });
    }

    pub(super) fn record_attempt(&mut self, content: Vec<u8>) -> u64 {
        self.new_content_number(Some(content))
    }

    /// 字面读法：`order` 那一步开始之前最后一笔改内容的确认（没有 = mkfs 那一版，内容号 0）。
    pub(super) fn last_confirmation_before(&self, order: usize) -> Option<&RandomConfirmation> {
        self.confirmations
            .iter()
            .rev()
            .find(|confirmation| confirmation.order < order)
    }

    pub(super) fn last_confirmed_number_before(&self, order: usize) -> u64 {
        self.last_confirmation_before(order)
            .map_or(0, |confirmation| confirmation.content_number)
    }

    /// 回退读法：改内容的确认与管理员回退交回 `Ok` 两种一起排，取 `order` 之前最后一笔的内容号。
    pub(super) fn last_confirmed_number_before_rollback_aware(&self, order: usize) -> u64 {
        let content = self
            .last_confirmation_before(order)
            .map(|confirmation| (confirmation.order, confirmation.content_number));
        let rollback = self
            .rollback_confirmations
            .iter()
            .rev()
            .find(|(rollback_order, _, _)| *rollback_order < order)
            .map(|(rollback_order, number, _)| (*rollback_order, *number));
        match (content, rollback) {
            (Some(content_event), Some(rollback_event)) => {
                if rollback_event.0 >= content_event.0 {
                    rollback_event.1
                } else {
                    content_event.1
                }
            }
            (Some((_, number)), None) | (None, Some((_, number))) => number,
            (None, None) => 0,
        }
    }
}

/// 一次冷启动读回按内容号分类（Q3 四类：最后确认那一版 / 更新 / 更旧 / 读失败），字面与回退两种读法各一份。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct RandomReadBack {
    pub(super) literal: String,
    rollback_aware: String,
    pub(super) root: Option<TimelineRoot>,
    pub(super) content_number: Option<u64>,
    /// 读失败时 `crates` 报的错原样（S5 的签名按它认：`MappingStillUnreadable { slot: … }`）。
    pub(super) failure_debug: Option<String>,
}

impl RandomReadBack {
    pub(super) fn literal_lost(&self) -> bool {
        self.literal.starts_with("older") || self.literal.starts_with("failed")
    }

    pub(super) fn rollback_aware_lost(&self) -> bool {
        self.rollback_aware.starts_with("older") || self.rollback_aware.starts_with("failed")
    }

    pub(super) fn render(&self, prefix: &str) -> String {
        format!(
            "{prefix}={} {prefix}_rollback_aware={} {prefix}_root={} {prefix}_content={}",
            self.literal,
            self.rollback_aware,
            key_text(self.root),
            self.content_number
                .map_or("none".to_string(), |number| number.to_string())
        )
    }
}

pub(super) fn classify_random_read_back(
    report: &RecoveryReport,
    ledger: &RandomLedger,
    literal_last: u64,
    rollback_aware_last: u64,
) -> RandomReadBack {
    let outcome_root = chosen_root_of(&report.outcome);
    let landing = report
        .effective_root
        .map(|(instance, txg)| (instance.0, txg.0))
        .or(outcome_root);
    let file: Option<Vec<u8>> = match &report.outcome {
        RecoveryOutcome::Failed { failure, .. } => {
            let failure_debug = format!("{failure:?}");
            let member = format!("failed:{}", error_member_of_debug(&failure_debug));
            return RandomReadBack {
                literal: member.clone(),
                rollback_aware: member,
                root: landing,
                content_number: None,
                failure_debug: Some(failure_debug),
            };
        }
        RecoveryOutcome::NoFile { .. } => None,
        RecoveryOutcome::FileRead { content, .. } => Some(content.clone()),
    };
    let by_root = landing.and_then(|root| ledger.content_number_of(root));
    let (number, how) = match by_root {
        Some(number) => {
            if ledger
                .contents
                .get(usize::try_from(number).expect("内容号"))
                != Some(&file)
            {
                let member = "failed:content_differs_from_the_ledger".to_string();
                return RandomReadBack {
                    literal: member.clone(),
                    rollback_aware: member,
                    root: landing,
                    content_number: Some(number),
                    failure_debug: None,
                };
            }
            (number, "")
        }
        None => {
            let candidates: Vec<u64> = ledger
                .contents
                .iter()
                .enumerate()
                .filter(|(_, bytes)| **bytes == file)
                .map(|(number, _)| u64::try_from(number).expect("内容号"))
                .collect();
            let chosen = if candidates.contains(&literal_last) {
                Some(literal_last)
            } else {
                candidates.last().copied()
            };
            match chosen {
                Some(number) => (number, ":by_bytes"),
                None => {
                    let member = "failed:content_not_in_the_ledger".to_string();
                    return RandomReadBack {
                        literal: member.clone(),
                        rollback_aware: member,
                        root: landing,
                        content_number: None,
                        failure_debug: None,
                    };
                }
            }
        }
    };
    let literal = if number == literal_last {
        format!("last_confirmed{how}")
    } else if number > literal_last {
        format!("newer:{number}{how}")
    } else {
        format!("older:{number}{how}")
    };
    let rollback_aware = if number == rollback_aware_last {
        format!("last_confirmed{how}")
    } else if !ledger.confirmed_numbers.contains(&number) {
        format!("newer:{number}{how}")
    } else {
        format!("older:{number}{how}")
    };
    RandomReadBack {
        literal,
        rollback_aware,
        root: landing,
        content_number: Some(number),
        failure_debug: None,
    }
}

// ------------------------------------------------------------------------
// 13.4 执行器：八种操作照 harness 执行器的入口与前提各写一遍（`history.rs` 的 `apply_publish_first_file`、`apply_publish_overwrite`、
//      `apply_publish_without_units`、`apply_mount_writable`、`apply_rollback_while_mounted`、`apply_raise_rollback_floor`、
//      `apply_cold_start_recover`、`apply_crash_recovery_abandoning_the_newest_root`），不跑模型、不判红就停（M22 那个开关除外）。
//      一臂拒了可写挂载，之后要会话的操作照 harness 的做法记「前提不满足」、照历史往下走（登记 5.3）。
// ------------------------------------------------------------------------

/// 这个进程里开着的可写会话（harness `WritableSession` 的同几样）。
pub(super) struct RandomSession {
    pub(super) allocator: PoolAllocator,
    pub(super) current: PoolVersion,
    pub(super) instance: InstanceGeneration,
    pub(super) shadow_ledger: ShadowLedger,
    pub(super) parameters_and_device_table: ParametersAndDeviceTableOfTheMount,
}

impl RandomSession {
    pub(super) fn of_the_mount(mounted: Mounted) -> Self {
        RandomSession {
            allocator: mounted.allocator,
            current: mounted.current,
            instance: mounted.output.instance,
            shadow_ledger: mounted.output.shadow_ledger,
            parameters_and_device_table: mounted.output.parameters_and_device_table,
        }
    }
}

/// 一次可写挂载（被测的，或第一次崩溃恢复抛弃根之前的合法状态）：Q4、Q5、Q7、Q2 的输入。
pub(super) struct RandomMount {
    pub(super) order: usize,
    pub(super) step_label: String,
    pub(super) operation: &'static str,
    pub(super) phase: &'static str,
    pub(super) base_fingerprint: u64,
    pub(super) summary: TestedMountSummary,
    pub(super) reading: ObservedCallReading,
    pub(super) judgement: Option<DeviceJudgement>,
    pub(super) hidden_ranges: usize,
    pub(super) literal_last: u64,
    pub(super) rollback_aware_last: u64,
    pub(super) effective_content: Option<u64>,
    pub(super) device_effective_content: Option<u64>,
    pub(super) older_version_returned: bool,
    pub(super) older_version_returned_rollback_aware: bool,
    pub(super) stop_clause_four_findings: Vec<String>,
    /// F6（登记第十节 F6 的改动）：崩溃恢复抛弃根那一步的被测挂载上 N-配置 判真、N-槽 判假；别的挂载 None。
    pub(super) failure_clause_six: Option<bool>,
}

#[cfg(test)]
mod tests;

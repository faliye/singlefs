//! 第 3 次跑的臂、故障时长与装置自己的块设备（按臂注入、虚拟时钟下的调用）。

use super::*;

// ============================================================================
// 十一、第 3 次跑（`research/prompts/e158-r3-prereg.md`）：C554 怎么修（岔路单 `research/prompts/c554-fix-forks.md` #1「挂载时有一条根（或它引用的账）暂时读不出」），
//      第一段（登记 5.7）：5.6 常量回比、第七节锚点、九条臂各一份副本、故障设备与虚拟时钟、确认账与真值、
//      PC-N、PC-O、PC-N0、PC-丢写、PC-只读（每一臂）；H1d 与 H1e（G0，全部臂）。PC-多读、Q5 与跨臂对拍在 `r3-compare` 模式里算。
//      第 2 次跑那几个模式（`r2-*`）调的函数这里只调用、不改语义；要改的一律复制一份（登记第一节「上一次执行员交来的两件」）。
// ============================================================================

/// 第 3 次跑的九条臂（登记 5.2 臂表的文件名拼写；丙-不验点名，不跑）。
pub(super) const THIRD_RUN_ARMS: [&str; 9] = [
    "today",
    "jia-cfg",
    "jia-cfg-carry",
    "jia-slot",
    "yi-cfg",
    "yi-cfg-carry",
    "yi-slot",
    "bing-cfg",
    "bing-cfg-carry",
];

/// 臂属于哪个候选（岔路单第 1 行：甲 拒可写、乙 重读后再判、丙 从记录重建、对照 今天）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ThirdRunCandidate {
    Today,
    Jia,
    Yi,
    Bing,
}

impl ThirdRunCandidate {
    pub(super) fn of(arm: &str) -> Option<ThirdRunCandidate> {
        match arm {
            "today" => Some(ThirdRunCandidate::Today),
            "jia-cfg" | "jia-cfg-carry" | "jia-slot" => Some(ThirdRunCandidate::Jia),
            "yi-cfg" | "yi-cfg-carry" | "yi-slot" => Some(ThirdRunCandidate::Yi),
            "bing-cfg" | "bing-cfg-carry" => Some(ThirdRunCandidate::Bing),
            _ => None,
        }
    }
}

/// 这一臂 N 判真之后交回的拒绝成员名（臂表 r3 行加进 `MountError` 的四个成员里的哪一个；第 22 条那一格另一个）。
pub(super) fn third_run_refusal_member(arm: &str) -> &'static str {
    match (ThirdRunCandidate::of(arm), arm.ends_with("-slot")) {
        (Some(ThirdRunCandidate::Bing), _) => "R3RebuildFromRecordsStoppedBelowTheWitness",
        (Some(ThirdRunCandidate::Jia | ThirdRunCandidate::Yi), true) => "R3UnreadableRootRingSlot",
        (Some(ThirdRunCandidate::Jia | ThirdRunCandidate::Yi), false) => {
            "R3NewerRootWitnessedByTheSystemConfiguration"
        }
        (Some(ThirdRunCandidate::Today) | None, _) => "none",
    }
}

/// 乙 的重读轮数 R（登记「读法写死」：D16（发布语义） 已定项 1「根槽这一次读坏」那一行的「重读一次」；5.6 倒数第二行，条款锚不回比）。
pub(super) const THIRD_RUN_REREAD_ROUNDS: u64 = 1;
/// 虚拟时钟的一格 δ（登记 5.6 倒数第二行）。
pub(super) const THIRD_RUN_VIRTUAL_INTERVAL: u64 = 1;
/// 单元类标签（D18（块里携带什么信息）；装置自己的重放按它取单元宽，开跑时与 `singlefs_core::unit` 那一份回比）。
pub(super) const UNIT_CLASS_DATA_LOCAL: u8 = 1;
pub(super) const UNIT_CLASS_INDEX_NODE_LOCAL: u8 = 2;
pub(super) const UNIT_CLASS_PACKED_LOCAL: u8 = 3;
/// 数据单元宽、节点宽（登记 5.6 第三行；`local_constants_checks` 里已回比）。
pub(super) const DATA_UNIT_BYTES_LOCAL: usize = 32768;
pub(super) const NODE_BYTES_LOCAL: usize = 16384;
/// 系统配置槽里 journal tail 与整槽校验和的位置（登记 5.6：tail 偏移 469 由装置自己解码找到、开跑时核；
/// 校验和偏移 155、宽 32 字节：`local_constants_checks` 里回比过偏移，宽度出自 `singlefs_checker` 的宽校验和口径）。
pub(super) const SYSTEM_CONFIGURATION_TAIL_OFFSET_LOCAL: usize = 469;
pub(super) const SYSTEM_CONFIGURATION_CHECKSUM_OFFSET_LOCAL: usize = 155;
pub(super) const WIDE_CHECKSUM_BYTES_LOCAL: usize = 32;
/// H1d / H1e 的 n1（登记 5.3：n1 ∈ {0, 1, 2, 3}）与 H1e 的 m（{1, 2, 3, 4}）。
pub(super) const THIRD_RUN_OVERWRITES: [u64; 4] = [0, 1, 2, 3];
pub(super) const THIRD_RUN_TAIL_OVERWRITES: [u64; 4] = [1, 2, 3, 4];

pub(super) fn third_run_arm_label() -> String {
    env::var("SINGLEFS_E158_ARM").unwrap_or_else(|_| "today".to_string())
}

/// 臂在 `crates/` 副本里记下的观测（`singlefs_core::mount::r3_take_observation` 的字段抄一份；今天那一臂没有）。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct ArmObservation {
    pub(super) witness_judged: bool,
    pub(super) newer_root_on_the_first_pass: bool,
    pub(super) witnessed_tail: u64,
    pub(super) selected_version_last_counter: Option<u64>,
    pub(super) undecidable: bool,
    pub(super) unreadable_ring_slots_on_the_first_pass: u64,
    pub(super) reread_rounds: u32,
    pub(super) rebuilt_from_records: bool,
    pub(super) shadow_ledger_table_unreadable_on_the_first_read: bool,
    pub(super) shadow_ledger_table_rereads: u32,
}

// ---------------------------------------------------------------------------
// 11.1 故障设备（登记 5.0 第三条）：一块盘一层、一次装完这块盘这次调用的全部落点；两种造法 × 四种时长；
//      每个落点在这次调用里的读次序单独计（O(m) 按它判）；T(τ) 读虚拟时钟；被拦下的读次数按落点记（V2）；
//      设备一层的读调用次数与读字节数按调用记（Q5，读坏的也数）；这次调用写过的落点之后不再拦。
// ---------------------------------------------------------------------------

/// 一个落点在这次入口调用里坏多久（登记「读法写死」表「时长」一行）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum FaultDuration {
    /// W：这一次入口调用里每一次读都坏。
    WholeCall,
    /// T(τ)：读坏到虚拟时钟走到 τ 为止。
    UntilVirtualTime(u64),
    /// O(m 只)：对这个落点的第 m 次读坏、其余照常。
    OnlyTheNthRead(u64),
    /// O(m 起)：第 m 次读起都坏。
    #[allow(
        dead_code,
        reason = "L5（代码审阅第 22 条那一格）归第二段；第一段只在单测里构造它"
    )]
    FromTheNthRead(u64),
}

impl FaultDuration {
    pub(super) fn name(self) -> String {
        match self {
            FaultDuration::WholeCall => "W".to_string(),
            FaultDuration::UntilVirtualTime(until) => format!("T{until}"),
            FaultDuration::OnlyTheNthRead(nth) => format!("O{nth}only"),
            FaultDuration::FromTheNthRead(nth) => format!("O{nth}from"),
        }
    }

    /// 对这个落点的第 `nth_read` 次读（从 1 数）在虚拟时刻 `virtual_now` 坏不坏。
    pub(super) fn is_active(self, nth_read: u64, virtual_now: u64) -> bool {
        match self {
            FaultDuration::WholeCall => true,
            FaultDuration::UntilVirtualTime(until) => virtual_now < until,
            FaultDuration::OnlyTheNthRead(nth) => nth_read == nth,
            FaultDuration::FromTheNthRead(nth) => nth_read >= nth,
        }
    }

    /// 装置自己的第一遍读（登记「读法写死」表 E_故障 那一格）：这个落点这次调用里第一次被读时坏不坏（虚拟时钟为 0）。
    fn is_active_on_the_first_read(self) -> bool {
        self.is_active(1, 0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ThirdRunFault {
    pub(super) device: DeviceIdentity,
    pub(super) offset: u64,
    pub(super) length: u64,
    pub(super) form: UnreadableForm,
    pub(super) duration: FaultDuration,
    pub(super) label: String,
}

impl ThirdRunFault {
    pub(super) fn overlaps(&self, device: DeviceIdentity, offset: u64, length: u64) -> bool {
        self.device == device && self.offset < offset + length && offset < self.offset + self.length
    }
}

#[derive(Default)]
pub(super) struct ThirdRunDeviceState {
    pub(super) faults: Vec<ThirdRunFault>,
    pub(super) written_over: Vec<bool>,
    pub(super) reads_of_the_target: Vec<u64>,
    pub(super) intercepted_reads: Vec<u64>,
    pub(super) read_calls: u64,
    pub(super) read_bytes: u64,
    pub(super) write_calls: u64,
    pub(super) barrier_calls: u64,
    pub(super) failing_write_call: Option<u64>,
    pub(super) failing_barrier_call: Option<u64>,
    pub(super) injected_failure_fired: bool,
    pub(super) applied: Vec<AppliedDeviceStep>,
}

impl ThirdRunDeviceState {
    pub(super) fn new(
        faults: Vec<ThirdRunFault>,
        failing_write_call: Option<u64>,
        failing_barrier_call: Option<u64>,
    ) -> Self {
        let count = faults.len();
        ThirdRunDeviceState {
            faults,
            written_over: vec![false; count],
            reads_of_the_target: vec![0; count],
            intercepted_reads: vec![0; count],
            failing_write_call,
            failing_barrier_call,
            ..ThirdRunDeviceState::default()
        }
    }

    pub(super) fn mark_written_over(&mut self, device: DeviceIdentity, offset: u64, length: u64) {
        for (fault, written_over) in self.faults.iter().zip(self.written_over.iter_mut()) {
            *written_over = *written_over || fault.overlaps(device, offset, length);
        }
    }
}

pub(super) struct ThirdRunDevice {
    pub(super) identity: DeviceIdentity,
    pub(super) inner: SparseBlockDevice,
    pub(super) state: Rc<RefCell<ThirdRunDeviceState>>,
}

impl BlockDevice for ThirdRunDevice {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        let length = u64::try_from(buffer.len()).expect("一次读的长度装得进 u64");
        let virtual_now = third_run_arm_hooks::virtual_now();
        let mut fails = false;
        let mut zeroed: Vec<(u64, u64)> = Vec::new();
        {
            let mut guard = self.state.borrow_mut();
            let state = &mut *guard;
            state.read_calls += 1;
            state.read_bytes += length;
            for index in 0..state.faults.len() {
                if state.written_over[index]
                    || !state.faults[index].overlaps(self.identity, offset.0, length)
                {
                    continue;
                }
                state.reads_of_the_target[index] += 1;
                let duration = state.faults[index].duration;
                if !duration.is_active(state.reads_of_the_target[index], virtual_now) {
                    continue;
                }
                state.intercepted_reads[index] += 1;
                match state.faults[index].form {
                    UnreadableForm::ReadFails => {
                        fails = true;
                    }
                    UnreadableForm::ReadsZeros => {
                        zeroed.push((state.faults[index].offset, state.faults[index].length));
                    }
                }
            }
        }
        if fails {
            return Err(injected_block_device_error("读"));
        }
        self.inner.read_at(offset, buffer)?;
        for (range_offset, range_length) in zeroed {
            let start = range_offset.saturating_sub(offset.0);
            let end = (range_offset + range_length)
                .saturating_sub(offset.0)
                .min(length);
            buffer[usize::try_from(start).expect("缓冲区里")
                ..usize::try_from(end).expect("缓冲区里")]
                .fill(0);
        }
        Ok(())
    }

    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        let mut state = self.state.borrow_mut();
        let ordinal = state.write_calls;
        state.write_calls += 1;
        let this_write_is_the_injected_failure = state.failing_write_call == Some(ordinal);
        if this_write_is_the_injected_failure {
            state.injected_failure_fired = true;
            return Err(injected_block_device_error("写"));
        }
        self.inner.write_at(offset, bytes, durability)?;
        let length = u64::try_from(bytes.len()).expect("写长装得进 u64");
        state.mark_written_over(self.identity, offset.0, length);
        state.applied.push(AppliedDeviceStep::Write {
            device: self.identity,
            offset: offset.0,
            bytes: bytes.to_vec(),
        });
        Ok(())
    }

    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        let mut state = self.state.borrow_mut();
        let ordinal = state.write_calls;
        state.write_calls += 1;
        if state.failing_write_call == Some(ordinal) {
            state.injected_failure_fired = true;
            return Err(injected_block_device_error("写"));
        }
        self.inner.write_zeroes_at(offset, length)?;
        state.mark_written_over(self.identity, offset.0, length);
        state.applied.push(AppliedDeviceStep::ZeroFill {
            device: self.identity,
            offset: offset.0,
            length,
        });
        Ok(())
    }

    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        let mut state = self.state.borrow_mut();
        let ordinal = state.barrier_calls;
        state.barrier_calls += 1;
        if state.failing_barrier_call == Some(ordinal) {
            state.injected_failure_fired = true;
            return Err(injected_block_device_error("屏障"));
        }
        self.inner.barrier()?;
        state.applied.push(AppliedDeviceStep::Barrier {
            device: self.identity,
        });
        Ok(())
    }

    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }

    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }
}

/// 一次受观测的入口调用交回的东西（可写挂载或冷启动恢复）。
pub(super) struct ThirdRunCall<Outcome> {
    pub(super) outcome: Outcome,
    pub(super) write_calls: u64,
    pub(super) read_calls: u64,
    pub(super) read_bytes: u64,
    pub(super) applied: Vec<AppliedDeviceStep>,
    pub(super) intercepted_reads: Vec<u64>,
    pub(super) injected_failure_fired: bool,
    pub(super) pool_after: MemoryPool,
    pub(super) waits: u64,
    pub(super) observation: Option<ArmObservation>,
}

impl<Outcome> ThirdRunCall<Outcome> {
    /// V2：点了名的落点里一次都没被拦下的（O(m) 的：第 m 次读没发生）。
    pub(super) fn unintercepted(&self, faults: &[ThirdRunFault]) -> Vec<String> {
        faults
            .iter()
            .zip(&self.intercepted_reads)
            .filter(|(_, intercepted)| **intercepted == 0)
            .map(|(fault, _)| fault.label.clone())
            .collect()
    }
}

fn third_run_call<Outcome>(
    base: &MemoryPool,
    faults: &[ThirdRunFault],
    failing_write_call: Option<u64>,
    failing_barrier_call: Option<u64>,
    call: impl FnOnce(&mut Vec<(DeviceIdentity, ThirdRunDevice)>) -> Outcome,
) -> ThirdRunCall<Outcome> {
    let state = Rc::new(RefCell::new(ThirdRunDeviceState::new(
        faults.to_vec(),
        failing_write_call,
        failing_barrier_call,
    )));
    let mut devices: Vec<(DeviceIdentity, ThirdRunDevice)> = devices_from_pool(base, IMAGE_BYTES)
        .into_iter()
        .map(|(identity, inner)| {
            (
                identity,
                ThirdRunDevice {
                    identity,
                    inner,
                    state: Rc::clone(&state),
                },
            )
        })
        .collect();
    third_run_arm_hooks::reset_the_virtual_clock();
    third_run_arm_hooks::install_the_wait_hook(THIRD_RUN_VIRTUAL_INTERVAL);
    let outcome = call(&mut devices);
    let observation = third_run_arm_hooks::take_the_arm_observation();
    let waits = third_run_arm_hooks::waits();
    let plain: Vec<(DeviceIdentity, SparseBlockDevice)> = devices
        .into_iter()
        .map(|(identity, device)| (identity, device.inner))
        .collect();
    let pool_after = memory_pool_of(&plain, IMAGE_BYTES);
    let state = state.borrow();
    ThirdRunCall {
        outcome,
        write_calls: state.write_calls,
        read_calls: state.read_calls,
        read_bytes: state.read_bytes,
        applied: state.applied.clone(),
        intercepted_reads: state.intercepted_reads.clone(),
        injected_failure_fired: state.injected_failure_fired,
        pool_after,
        waits,
        observation,
    }
}

pub(super) fn run_third_run_mount_writable(
    base: &MemoryPool,
    parameters: &MakeFilesystemParameters,
    faults: &[ThirdRunFault],
    failing_write_call: Option<u64>,
    failing_barrier_call: Option<u64>,
) -> ThirdRunCall<Result<Mounted, singlefs_core::mount::MountError>> {
    third_run_call(
        base,
        faults,
        failing_write_call,
        failing_barrier_call,
        |devices| mount_writable(parameters, devices),
    )
}

pub(super) fn run_third_run_recover(
    base: &MemoryPool,
    faults: &[ThirdRunFault],
) -> ThirdRunCall<singlefs_core::recovery::RecoveryReport> {
    third_run_call(base, faults, None, None, |devices| {
        recover(&*devices, JournalPolicy::Consult)
    })
}

/// 结局按 K0–K4 分（同第 2 次跑 `mount_outcome_class`，按设备一层写次数分 K3 / K4）。
pub(super) fn third_run_outcome_class(
    result: &Result<Mounted, singlefs_core::mount::MountError>,
    write_calls: u64,
) -> &'static str {
    mount_outcome_class(result, write_calls)
}

// ---------------------------------------------------------------------------
// 11.2 装置自己的解码（登记 5.0 第二条）：根槽、系统配置槽、journal 记录走 `singlefs_checker`（另一套解析代码），
//      择根取 (txg, 实例代号) 最大，按 D23（journal 的角色与格式） 已定项 14 五条口径自己重放出 E。
//      带故障的第一遍读：落点在这次调用里第一次被读时坏的，当读不出（读错）或读回全零（清零）。
// ---------------------------------------------------------------------------

/// 装置的第一遍读：`pool` 上盖着 `faults` 里「第一次读就坏」的那几个落点。
pub(super) struct FirstPassReader<'pool> {
    pub(super) pool: &'pool MemoryPool,
    pub(super) faults: &'pool [ThirdRunFault],
}

impl ImageReader for FirstPassReader<'_> {
    fn devices(&self) -> Vec<u32> {
        ImageReader::devices(self.pool)
    }
    fn device_bytes(&self, device: u32) -> Option<u64> {
        ImageReader::device_bytes(self.pool, device)
    }
    fn read(&self, device: u32, offset: u64, length: usize) -> Option<Vec<u8>> {
        let length_u64 = u64::try_from(length).expect("读长");
        let active: Vec<&ThirdRunFault> = self
            .faults
            .iter()
            .filter(|fault| {
                fault.duration.is_active_on_the_first_read()
                    && fault.overlaps(DeviceIdentity(device), offset, length_u64)
            })
            .collect();
        if active
            .iter()
            .any(|fault| fault.form == UnreadableForm::ReadFails)
        {
            return None;
        }
        let mut bytes = ImageReader::read(self.pool, device, offset, length)?;
        for fault in active {
            let start = fault.offset.saturating_sub(offset);
            let end = (fault.offset + fault.length)
                .saturating_sub(offset)
                .min(length_u64);
            bytes[usize::try_from(start).expect("读内")..usize::try_from(end).expect("读内")]
                .fill(0);
        }
        Some(bytes)
    }
    fn candidate_unit_slots(&self, device: u32) -> Option<Vec<u64>> {
        ImageReader::candidate_unit_slots(self.pool, device)
    }
    fn candidate_journal_slots(&self, device: u32) -> Option<Vec<u64>> {
        ImageReader::candidate_journal_slots(self.pool, device)
    }
}

#[cfg(test)]
mod third_run_tests;

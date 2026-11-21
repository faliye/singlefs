use super::*;

// ───────────────────────────── 计时包装设备（登记第五节「计时与计数怎么做」） ─────────────────────────────

/// 包装转给里面那块盘的每一种调用。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum DeviceMethod {
    Read,
    Write,
    WriteZeroes,
    Barrier,
    Discard,
    RereadAfterABadRead,
    ReadStraightFromTheDisk,
    KeepTheSelfVerifiedReread,
    NoteTheReadFailingSelfVerification,
    JournalRecordOffsetsHint,
}

impl DeviceMethod {
    pub(super) const ALL: [DeviceMethod; 10] = [
        DeviceMethod::Read,
        DeviceMethod::Write,
        DeviceMethod::WriteZeroes,
        DeviceMethod::Barrier,
        DeviceMethod::Discard,
        DeviceMethod::RereadAfterABadRead,
        DeviceMethod::ReadStraightFromTheDisk,
        DeviceMethod::KeepTheSelfVerifiedReread,
        DeviceMethod::NoteTheReadFailingSelfVerification,
        DeviceMethod::JournalRecordOffsetsHint,
    ];

    pub(super) const fn label(self) -> &'static str {
        match self {
            DeviceMethod::Read => "read",
            DeviceMethod::Write => "write",
            DeviceMethod::WriteZeroes => "write_zeroes",
            DeviceMethod::Barrier => "barrier",
            DeviceMethod::Discard => "discard",
            DeviceMethod::RereadAfterABadRead => "reread_after_a_bad_read",
            DeviceMethod::ReadStraightFromTheDisk => "read_straight_from_the_disk",
            DeviceMethod::KeepTheSelfVerifiedReread => "keep_the_self_verified_reread",
            DeviceMethod::NoteTheReadFailingSelfVerification => {
                "note_the_read_failing_self_verification"
            }
            DeviceMethod::JournalRecordOffsetsHint => "journal_record_offsets_hint",
        }
    }
}

/// 一种调用在一个计时窗口里的累计：次数、字节、挂钟、这期间本线程的 CPU、等盘（挂钟减 CPU，可以是负的：设备调用这一层 CPU 读数包着挂钟读数，纯 CPU 的调用里 CPU 多出读钟的工夫）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct MethodTotals {
    pub(super) calls: u64,
    pub(super) bytes: u64,
    pub(super) wall_nanoseconds: u64,
    pub(super) cpu_nanoseconds: u64,
    pub(super) wait_nanoseconds: i64,
}

/// 设备调用里的等盘时间：这次调用的挂钟减去同一调用里本线程 CPU 时间的增量（登记第一节「等盘读写时间」）。
fn device_wait_nanoseconds(wall_nanoseconds: u64, cpu_nanoseconds: u64) -> i64 {
    i64::try_from(wall_nanoseconds).expect("一次设备调用的挂钟装得进 i64")
        - i64::try_from(cpu_nanoseconds).expect("一次设备调用的 CPU 装得进 i64")
}

/// 记下的一次读：哪块盘、偏移、长度、是不是这个窗口里第一次写之前读的。第一次写之前的读有两样落在 R 的槽上：区域扫描对 R 那块盘
/// 每个未释放单元读一遍，随后发布在 `publish_admitted` 之前对这一批每个搬迁源按两条位置项各读一遍做释放前读盘核
/// （`transaction.rs` 里调 `copies_of_a_released_unit_failing_the_release_checksum_check` 的那一圈）；发布写下之后的读与 discard 的读
/// 在第一次写之后。两样怎么分开见 [`split_the_reads_of_a_call`]（登记第十二节 12.5）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct RecordedRead {
    pub(super) device: DeviceIdentity,
    pub(super) offset: u64,
    pub(super) length: u64,
    pub(super) is_before_the_first_write: bool,
}

/// 记下的一次写（`write_at`）：哪块盘、偏移、长度。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct RecordedWrite {
    pub(super) device: DeviceIdentity,
    pub(super) offset: u64,
    pub(super) length: u64,
}

/// 阳性对照往每次 `write_at` 里加的东西（登记第五节阳性对照表第 1 行）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum WriteInjection {
    Nothing,
    SleepTwoMilliseconds,
    SpinTwoMillisecondsOfThreadCpu,
}

impl WriteInjection {
    pub(super) const fn label(self) -> &'static str {
        match self {
            WriteInjection::Nothing => "none",
            WriteInjection::SleepTwoMilliseconds => "sleep_2_milliseconds_per_write",
            WriteInjection::SpinTwoMillisecondsOfThreadCpu => "spin_2_milliseconds_per_write",
        }
    }
}

/// 一组盘共用的账（两块盘一份）：一个计时窗口里每种调用的累计、读写的落点；另带跨窗口的两样——改过的 4 MiB 块（真盘臂下一轮只拷回这些块）
/// 与阳性对照的注入。窗口由调用方在一次 `run_compaction` 之前清（[`CallLedger::open_a_new_window`]）。
#[derive(Debug)]
pub(super) struct CallLedger {
    pub(super) totals: BTreeMap<DeviceMethod, MethodTotals>,
    pub(super) reads: Vec<RecordedRead>,
    pub(super) writes: Vec<RecordedWrite>,
    has_a_write_in_this_window: bool,
    pub(super) is_the_cpu_unreadable_in_this_window: bool,
    pub(super) chunks_written_since_the_copy: BTreeSet<(DeviceIdentity, u64)>,
    pub(super) injection: WriteInjection,
}

/// 真盘臂拷回与算摘要的块大小。
pub(super) const IMAGE_CHUNK_BYTES: u64 = 4 << 20;

impl CallLedger {
    pub(super) fn new(injection: WriteInjection) -> CallLedger {
        CallLedger {
            totals: BTreeMap::new(),
            reads: Vec::new(),
            writes: Vec::new(),
            has_a_write_in_this_window: false,
            is_the_cpu_unreadable_in_this_window: false,
            chunks_written_since_the_copy: BTreeSet::new(),
            injection,
        }
    }

    pub(super) fn open_a_new_window(&mut self) {
        self.totals.clear();
        self.reads.clear();
        self.writes.clear();
        self.has_a_write_in_this_window = false;
        self.is_the_cpu_unreadable_in_this_window = false;
    }

    fn note_the_call(
        &mut self,
        method: DeviceMethod,
        bytes: u64,
        wall: Duration,
        cpu_before: Option<u64>,
        cpu_after: Option<u64>,
    ) {
        let wall_nanoseconds =
            u64::try_from(wall.as_nanos()).expect("一次设备调用的挂钟装得进 u64");
        let cpu_nanoseconds = match (cpu_before, cpu_after) {
            (Some(before), Some(after)) => after.saturating_sub(before),
            (None, _) | (_, None) => {
                self.is_the_cpu_unreadable_in_this_window = true;
                0
            }
        };
        let totals = self.totals.entry(method).or_default();
        totals.calls += 1;
        totals.bytes += bytes;
        totals.wall_nanoseconds += wall_nanoseconds;
        totals.cpu_nanoseconds += cpu_nanoseconds;
        totals.wait_nanoseconds += device_wait_nanoseconds(wall_nanoseconds, cpu_nanoseconds);
    }

    fn note_the_changed_range(&mut self, device: DeviceIdentity, offset: u64, length: u64) {
        if length == 0 {
            return;
        }
        for chunk in offset / IMAGE_CHUNK_BYTES..=(offset + length - 1) / IMAGE_CHUNK_BYTES {
            self.chunks_written_since_the_copy.insert((device, chunk));
        }
    }

    #[cfg(test)]
    fn totals_of(&self, method: DeviceMethod) -> MethodTotals {
        self.totals.get(&method).copied().unwrap_or_default()
    }

    /// 这个窗口里全部设备调用的累计（等盘就是各调用等盘之和）。
    pub(super) fn totals_of_every_method(&self) -> MethodTotals {
        let mut sum = MethodTotals::default();
        for totals in self.totals.values() {
            sum.calls += totals.calls;
            sum.bytes += totals.bytes;
            sum.wall_nanoseconds += totals.wall_nanoseconds;
            sum.cpu_nanoseconds += totals.cpu_nanoseconds;
            sum.wait_nanoseconds += totals.wait_nanoseconds;
        }
        sum
    }
}

/// 真盘臂的 journal 记录偏移提示：可写挂载那一段里把单元区之前（系统配置、根环、journal 环）的写照抄进一块内存盘，挂载照它答哪几个记录槽
/// 可能有字节（同内存盘的答法 `SparseBlockDevice::journal_record_offsets_hint`），不在 `O_DIRECT` 镜像上把 768 MiB 的环逐槽读一遍。
/// 只在挂载里照抄（`is_mirroring`）；计时窗口里一个字节都不抄，那时再要提示就转给真盘（答 `None`、全环扫描）。
pub(super) struct JournalHintMirror {
    pub(super) image: SparseBlockDevice,
    pub(super) below_offset: u64,
    pub(super) is_mirroring: bool,
}

/// 计时包装：每一种调用都转给里面那块盘，前后各读一次 `Instant` 与本线程的线程 CPU 时钟，记进共用的账。
pub(super) struct TimedBlockDevice<Inner: BlockDevice> {
    inner: Inner,
    pub(super) identity: DeviceIdentity,
    ledger: Rc<RefCell<CallLedger>>,
    pub(super) journal_hint_mirror: Option<JournalHintMirror>,
}

/// 计一次设备调用：这一层 CPU 读数包着挂钟读数，一次 `run_compaction` 调用那一层反过来挂钟读数包着 CPU 读数（`measure_one_intent`）。
/// 两个钟走得一样快时这样嵌套，`CPU + 等盘 ≤ 挂钟` 不会因为读钟的先后被打破：等盘 = 设备挂钟 − 设备 CPU 少算了设备那一层读钟的工夫，
/// 调用那一层的挂钟多算了这一层读钟的工夫；包含自检反过来（停机条款第 6 条）只剩计时口径真错了这一种来路。
fn timed_call<Answer>(
    ledger: &RefCell<CallLedger>,
    method: DeviceMethod,
    bytes: u64,
    call: impl FnOnce() -> Answer,
) -> Answer {
    let cpu_before = thread_cpu_nanoseconds();
    let started = RawWallClockReading::now();
    let answer = call();
    let wall = started.elapsed();
    let cpu_after = thread_cpu_nanoseconds();
    ledger
        .borrow_mut()
        .note_the_call(method, bytes, wall, cpu_before, cpu_after);
    answer
}

pub(super) fn length_in_bytes(bytes: &[u8]) -> u64 {
    u64::try_from(bytes.len()).expect("一次设备调用的长度装得进 u64")
}

impl<Inner: BlockDevice> TimedBlockDevice<Inner> {
    pub(super) fn new(
        inner: Inner,
        identity: DeviceIdentity,
        ledger: &Rc<RefCell<CallLedger>>,
    ) -> Self {
        TimedBlockDevice {
            inner,
            identity,
            ledger: Rc::clone(ledger),
            journal_hint_mirror: None,
        }
    }

    fn note_the_read(&self, offset: DeviceOffsetInBytes, length: u64) {
        let mut ledger = self.ledger.borrow_mut();
        let is_before_the_first_write = !ledger.has_a_write_in_this_window;
        ledger.reads.push(RecordedRead {
            device: self.identity,
            offset: offset.0,
            length,
            is_before_the_first_write,
        });
    }

    pub(super) fn stop_mirroring_the_journal_writes(&mut self) {
        if let Some(mirror) = self.journal_hint_mirror.as_mut() {
            mirror.is_mirroring = false;
        }
    }
}

impl<Inner: BlockDevice> BlockDevice for TimedBlockDevice<Inner> {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        let length = length_in_bytes(buffer);
        let answer = timed_call(&self.ledger, DeviceMethod::Read, length, || {
            self.inner.read_at(offset, buffer)
        });
        self.note_the_read(offset, length);
        answer
    }

    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        let length = length_in_bytes(bytes);
        let injection = self.ledger.borrow().injection;
        let inner = &mut self.inner;
        let answer = timed_call(&self.ledger, DeviceMethod::Write, length, || {
            let written = inner.write_at(offset, bytes, durability);
            match injection {
                WriteInjection::Nothing => {}
                WriteInjection::SleepTwoMilliseconds => {
                    std::thread::sleep(Duration::from_millis(INJECTED_MILLISECONDS_PER_WRITE));
                }
                WriteInjection::SpinTwoMillisecondsOfThreadCpu => {
                    let spun = spin_until_the_thread_cpu_advances(
                        INJECTED_MILLISECONDS_PER_WRITE * 1_000_000,
                    );
                    assert!(
                        spun,
                        "线程 CPU 时钟读不到：CPU 臂的阳性对照做不了（读不到不当 0）"
                    );
                }
            }
            written
        });
        if let Some(mirror) = self.journal_hint_mirror.as_mut() {
            if mirror.is_mirroring && offset.0 < mirror.below_offset {
                mirror
                    .image
                    .write_at(offset, bytes, durability)
                    .expect("照抄进内存盘：偏移与长度已由真盘核过");
            }
        }
        let mut ledger = self.ledger.borrow_mut();
        ledger.has_a_write_in_this_window = true;
        ledger.writes.push(RecordedWrite {
            device: self.identity,
            offset: offset.0,
            length,
        });
        ledger.note_the_changed_range(self.identity, offset.0, length);
        answer
    }

    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        let inner = &mut self.inner;
        let answer = timed_call(&self.ledger, DeviceMethod::WriteZeroes, length, || {
            inner.write_zeroes_at(offset, length)
        });
        if let Some(mirror) = self.journal_hint_mirror.as_mut() {
            if mirror.is_mirroring && offset.0 < mirror.below_offset {
                mirror
                    .image
                    .write_zeroes_at(offset, length)
                    .expect("照抄进内存盘：偏移与长度已由真盘核过");
            }
        }
        let mut ledger = self.ledger.borrow_mut();
        ledger.has_a_write_in_this_window = true;
        ledger.note_the_changed_range(self.identity, offset.0, length);
        answer
    }

    fn discard_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<DiscardOutcome, BlockDeviceError> {
        let inner = &mut self.inner;
        let answer = timed_call(&self.ledger, DeviceMethod::Discard, length, || {
            inner.discard_at(offset, length)
        });
        if matches!(answer, Ok(DiscardOutcome::Applied)) {
            self.ledger
                .borrow_mut()
                .note_the_changed_range(self.identity, offset.0, length);
        }
        answer
    }

    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        let inner = &mut self.inner;
        timed_call(&self.ledger, DeviceMethod::Barrier, 0, || inner.barrier())
    }

    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }

    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }

    fn journal_record_offsets_hint(
        &self,
        ring_start: DeviceOffsetInBytes,
        ring_bytes: u64,
    ) -> Option<Vec<DeviceOffsetInBytes>> {
        timed_call(
            &self.ledger,
            DeviceMethod::JournalRecordOffsetsHint,
            0,
            || match &self.journal_hint_mirror {
                Some(mirror) if mirror.is_mirroring => mirror
                    .image
                    .journal_record_offsets_hint(ring_start, ring_bytes),
                Some(_) | None => self
                    .inner
                    .journal_record_offsets_hint(ring_start, ring_bytes),
            },
        )
    }

    fn reread_at_after_a_bad_read(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        let length = length_in_bytes(buffer);
        let answer = timed_call(
            &self.ledger,
            DeviceMethod::RereadAfterABadRead,
            length,
            || self.inner.reread_at_after_a_bad_read(offset, buffer),
        );
        self.note_the_read(offset, length);
        answer
    }

    fn keep_the_self_verified_reread(&self, offset: DeviceOffsetInBytes, bytes: &[u8]) {
        timed_call(
            &self.ledger,
            DeviceMethod::KeepTheSelfVerifiedReread,
            length_in_bytes(bytes),
            || self.inner.keep_the_self_verified_reread(offset, bytes),
        );
    }

    fn note_the_read_failing_self_verification_and_forget_its_bytes(
        &self,
        offset: DeviceOffsetInBytes,
        length_in_bytes_of_the_read: u64,
    ) {
        timed_call(
            &self.ledger,
            DeviceMethod::NoteTheReadFailingSelfVerification,
            length_in_bytes_of_the_read,
            || {
                self.inner
                    .note_the_read_failing_self_verification_and_forget_its_bytes(
                        offset,
                        length_in_bytes_of_the_read,
                    );
            },
        );
    }

    fn read_at_straight_from_the_disk(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        let length = length_in_bytes(buffer);
        let answer = timed_call(
            &self.ledger,
            DeviceMethod::ReadStraightFromTheDisk,
            length,
            || self.inner.read_at_straight_from_the_disk(offset, buffer),
        );
        self.note_the_read(offset, length);
        answer
    }
}

/// 一组盘套上计时包装，共用一份账。
pub(super) fn timed_devices<Inner: BlockDevice>(
    devices: Vec<(DeviceIdentity, Inner)>,
    ledger: &Rc<RefCell<CallLedger>>,
) -> Vec<(DeviceIdentity, TimedBlockDevice<Inner>)> {
    devices
        .into_iter()
        .map(|(identity, inner)| (identity, TimedBlockDevice::new(inner, identity, ledger)))
        .collect()
}

#[cfg(test)]
mod tests;

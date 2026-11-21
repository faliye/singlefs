//! 交给 GPU 算的那一侧：卡的标识、显存与卡清单，按段交 GPU 的条目与每条的结论，GPU 出错的种类与判卡不可用时读到的 PCI 状态，
//! [`BackgroundGpuCompute`] 特质、借出它的把手（[`BackgroundGpuComputeHandle`]）与一批交了 GPU 几次的记数。

use super::*;

/// 一张显卡的标识（`background_gpu_devices` 列表里的一项）：不透明的字节，只比相等。卡用什么认、怎么排是里程碑四设计轮（七·2）
/// 还挂着的岔路，这里不定格式：卡清单（[`GpuInventory`]）交出什么，调用方就点什么。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GpuDeviceIdentifier(Vec<u8>);

impl GpuDeviceIdentifier {
    #[must_use]
    pub fn of_opaque_bytes(opaque_bytes: Vec<u8>) -> Self {
        Self(opaque_bytes)
    }

    /// 标识的字节原样（产品 GPU 包照它当 PCI 地址读那张卡的 PCI 状态）。
    #[must_use]
    pub fn opaque_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// 显存的字节数：一张卡的显存总量，或 `background_gpu_memory_limit` 里给一张卡的上限（第 3 条）。
/// 后台 GPU 包切批与记账的字节数也用它（一段的占用、每条与每段的固定占用、卡上占着的、最小的一片）：都是卡上显存的字节，与上限直接比。
/// 份数、条数各是各的类型（后台 GPU 包的 `memory_limit_and_chunks`），乘份数在那边算。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct GpuMemoryInBytes(pub u64);

impl GpuMemoryInBytes {
    /// 两份显存字节数之和；超出 `u64` 交回 `None`。
    #[must_use]
    pub const fn checked_add(self, more: Self) -> Option<Self> {
        match self.0.checked_add(more.0) {
            Some(sum_in_bytes) => Some(Self(sum_in_bytes)),
            None => None,
        }
    }

    /// 扣掉 `less` 之后剩下的；`less` 比它多交回 `None`。
    #[must_use]
    pub const fn checked_sub(self, less: Self) -> Option<Self> {
        match self.0.checked_sub(less.0) {
            Some(difference_in_bytes) => Some(Self(difference_in_bytes)),
            None => None,
        }
    }

    /// 两份显存字节数之和，用在和装得进 `u64` 的地方（各段固定占用的常量、一段真建的缓冲合起来）。
    ///
    /// # Panics
    /// 和超出 `u64`。
    #[must_use]
    pub const fn plus(self, more: Self) -> Self {
        match self.checked_add(more) {
            Some(sum) => sum,
            None => panic!("显存字节数之和超出 u64"),
        }
    }
}

/// 几份显存字节数合起来（一段真建的每一份缓冲加起来）。
///
/// # Panics
/// 和超出 `u64`。
impl std::iter::Sum for GpuMemoryInBytes {
    fn sum<Parts: Iterator<Item = Self>>(parts: Parts) -> Self {
        parts.fold(Self(0), Self::plus)
    }
}

/// 本机有哪些显卡、每张多少显存：只读。GPU 那一侧（住哪是七·2 的岔路）实现它，测试里一份假清单实现它。
/// 挂载与切换的校验只读它；没有这一份清单（`None`）就是 GPU 不在。
pub trait GpuInventory {
    /// 本机全部显卡与各自的显存总量，按「本机全部显卡的列表」的次序：没给 `background_gpu_devices` 时，`background_gpu_memory_limit`
    /// 按这个次序一张卡一个（第 3 条）。怎么排由实现这份清单的一方定（七·2 的岔路）。同一张卡只出现一次。没开着的卡（不参与而没开的）照列。
    fn devices_with_total_memory(&self) -> Vec<(GpuDeviceIdentifier, GpuMemoryInBytes)>;
}

/// GC 搬迁放上 GPU 的三段（第 1 条）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GpuComputedSegment {
    /// 甲：扫描里的校验和核对——区域扫描读到的、判成可搬的单元，整单元 CRC-32C 与指着它的位置项里的比。
    UnitChecksumVerification,
    /// 乙：区域扫描里逐条记录的单元头解析与映射反查（钉住、隔离两判之后的那一段）。
    RegionRecordClassification,
    /// 丙：选段打分——每个聚簇段的活槽数。
    LiveSlotsPerClusterSegment,
}

/// 区域扫描里一条记录（连同它读到的单元）的条目键：甲、乙两段的两份核逐条比对时的条目身份，实现按它数连续不确定的轮数（第 8 条）。
/// R 在意图期内不作分配目的地（D26（后台整理与放置回收） 已定项 1 第 3 条），同一个键在两批之间指的是同一条记录。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RegionRecordEntryKey {
    pub device: DeviceIdentity,
    pub slot: SlotNumber,
    pub span_slots: u16,
}

impl RegionRecordEntryKey {
    #[must_use]
    pub fn of_the_record(record: &AllocationRecord) -> Self {
        Self {
            device: record.device,
            slot: record.slot,
            span_slots: record.span_slots,
        }
    }
}

/// 一个聚簇段的条目键：丙段按它数连续不确定的轮数（第 8 条）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ClusterSegmentEntryKey {
    pub device: DeviceIdentity,
    pub start_slot: SlotNumber,
}

/// 甲段要核的一个单元：区域扫描读到的整单元字节，与指着这个落点的那条位置项里记的整单元 CRC-32C。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnitChecksumToVerifyOnTheGpu<'unit> {
    pub entry_key: RegionRecordEntryKey,
    pub unit_bytes: &'unit [u8],
    pub expected_crc32c: u32,
}

/// 甲段一条的结局：算出的整单元 CRC-32C，与它对不对得上位置项。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnitChecksumOutcome {
    pub computed_crc32c: u32,
    pub matches_the_expected_crc32c: bool,
}

/// 乙段要分类的一条记录：区域扫描会去读的那一种（R 那块盘上、未释放、起点在 R 里、不罩出 R、不钉住、不隔离、跨度是 1 或 2），
/// 与读单元那一步的结局（跨度 1 读 16 KiB、跨度 2 读 32 KiB；`None` 是读不到）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegionRecordToClassifyOnTheGpu<'unit> {
    pub entry_key: RegionRecordEntryKey,
    pub record: AllocationRecord,
    pub unit_bytes: Option<&'unit [u8]>,
}

/// 一个条目这一轮的判定（第 8 条）。封闭集合。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GpuEntryDecision<Outcome> {
    /// 两份核结果相同：确认，交 CPU 照发布路径逐条核。
    ConfirmedByBothKernels(Outcome),
    /// 两份核结果不同：状态不确定，这一批不回收、不搬，留到下一轮重算。
    UncertainThisRound,
    /// 同一个条目连续 2 轮不确定：CPU 那一份实现单独判了一次，按它的判定处理。
    JudgedByTheCpuReference(Outcome),
}

impl<Outcome> GpuEntryDecision<Outcome> {
    /// 确认的与交 CPU 判的那一份结局；不确定的是 `None`。
    #[must_use]
    pub fn outcome(&self) -> Option<&Outcome> {
        match self {
            GpuEntryDecision::ConfirmedByBothKernels(outcome)
            | GpuEntryDecision::JudgedByTheCpuReference(outcome) => Some(outcome),
            GpuEntryDecision::UncertainThisRound => None,
        }
    }
}

/// 实现算一段时出的错（第 4 条「停下并报错」）。核心层对归卡、归输入的那几种做的是同一件事：这一批放弃、意图删掉（有活意图就单发一次只删意图的发布，
/// D26（后台整理与放置回收） 已定项 1 第 3 条）、那个后台任务停下并报这个错，不换回 CPU；
/// 一批里同一个条目键出现两次（[`Self::SameEntryKeyTwiceInOneBatch`]）只放弃这一批，后台任务不停、不报警，下一批照常算（第 8 条）。
/// 成员按停下之后该怎么处置分（修好或不再点那张卡、调大等 GPU 的上限、切回 `cpu`），出在哪张卡、什么原因带在成员里。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BackgroundGpuComputeFailed {
    /// 这张卡出错（起不了、设备丢了、等不成、回读坏了、核留下认不出的判定字）：修好它或切换时不再点它，或切回 `cpu`。
    /// `pci_state_when_judged_unusable` 是挂着时一批里卡当场报出错（[`GpuFailureOnTheDevice::UncapturedErrorOrDeviceLost`]）那一刻读的 PCI 状态；
    /// 别的原因、与开卡每一次没开成的原因（[`Self::GpuNotOpenedAfterTheReopenings`] 的 `failures_of_each_opening`）都是没读。
    /// 带读数的三个成员都把读数装箱：读数只在出错时有，装箱之后这个出错类型只多一个指针，装着它的
    /// [`BackgroundComputeError`]、报警与返回它的 `Result` 不过 clippy `result_large_err` 与 `large_enum_variant` 的线。
    GpuFailed {
        device: GpuDeviceIdentifier,
        cause: GpuFailureOnTheDevice,
        pci_state_when_judged_unusable: Box<PciStateOfTheGpuReadWhenJudgedUnusable>,
    },
    /// 等这张卡超过了 `background_gpu_wait_limit`：调大上限，或查这张卡是不是卡死了。
    WaitForTheGpuExceededTheLimit {
        device: GpuDeviceIdentifier,
        wait_limit: Duration,
        submission_waited_for: SubmissionWaitedForPastTheLimit,
    },
    /// 这一批的输入 GPU 那一侧第一版不支持，一张卡都没交活：切回 `cpu`（装不进一段的那一种也可以调大显存上限）。
    InputNotSupportedOnTheGpu(InputNotSupportedOnTheGpu),
    /// 挂着切换点到的这张卡开不成（起不了设备、编不了管线、量代价出错或超时、起不了开卡线程、开卡 panic、前一次开卡没回来），隔一段时间重开到上限也都没开成
    /// （第 4 条「间隔 1 分钟重开，最多重开 3 次……3 次都不成，GC 停下，报警里报出哪张卡、每次开不成的原因，等人处理，不自动再试」），
    /// 或这张卡在这一次挂载里已连着开了 4 次都没开成（开成一次清零）、这一次不再开（[`GpuFailureOnTheDevice::OpeningsInThisMountUsedUp`]）：修好它或切换时不再点它，或切回 `cpu`；
    /// 连着试满的那一种要卸载重挂。`failures_of_each_opening` 是每一次开不成照这张卡出错报成的那一个（[`Self::GpuFailed`] 或
    /// [`Self::WaitForTheGpuExceededTheLimit`]），按开的次序，头一次的在前；连着试满、这一次不再开的只有一个。
    /// `pci_state_when_judged_unusable` 是判它开不成那一刻读的 PCI 状态（每一次没开成而还要重开时不读）。
    GpuNotOpenedAfterTheReopenings {
        device: GpuDeviceIdentifier,
        failures_of_each_opening: Vec<BackgroundGpuComputeFailed>,
        pci_state_when_judged_unusable: Box<PciStateOfTheGpuReadWhenJudgedUnusable>,
    },
    /// 这张卡上一次提交等超过 `wait_limit` 之后，在同一张卡上重交了 `resubmissions` 次、次次等超过上限（「在间隔内重试三次，
    /// 如果gpu无法用就报警」「重试是把那一批重新交一次」）：这张卡用不了。修好它或切换时不再点它，或切回 `cpu`。
    /// `last_submission_waited_for` 是最后那一次等超过上限的是哪一次提交（重交之前先等前一次，那一次没等到也算一次）。
    /// `pci_state_when_judged_unusable` 是判它用不了那一刻读的 PCI 状态（每次超时重交之前不读）。
    GpuUnusableAfterResubmissions {
        device: GpuDeviceIdentifier,
        resubmissions: u32,
        wait_limit: Duration,
        last_submission_waited_for: SubmissionWaitedForPastTheLimit,
        pci_state_when_judged_unusable: Box<PciStateOfTheGpuReadWhenJudgedUnusable>,
    },
    /// 这一组设置点的卡的显存上限（扣掉卡上已占的）连最小的一片都装不下，参与的卡一张都不剩（
    /// 「不能因为装不下就拒绝」）：开卡、换设置时按与输入无关的最小的一片判，每一批切段时按这一批的最小的一片判（一张卡这一批切不出就这一批不分给它、
    /// 活分给别的卡，全都切不出才是这一种）。调大显存上限、点别的卡，或切回 `cpu`。按参与的次序列出每张卡、给它的上限、至少要多少与已占的，至少一张。
    EveryParticipatingGpuLeftOutUnderItsMemoryLimit {
        gpus_left_out: Vec<GpuLeftOutUnderItsMemoryLimit>,
    },
    /// 这一批参与的卡一张都不剩，其中至少一张是因为卡上此刻空闲的显存连这一批最小的一片都建不成（
    /// 给的上限之内建缓冲显存不足就一路切小重交，切到最小的一片都建不成，这一批就不用这张卡、它还没交成的活转给这一批别的参与的卡重交；
    /// 下一批照设置重新起）。腾出卡上的显存、点别的卡，或切回 `cpu`。`gpus_short_of_free_memory` 按不用的先后列出因空闲显存不用的卡，至少一张；
    /// `gpus_left_out_under_their_memory_limit` 是这一段切的时候因显存上限（扣掉已占的）不用的卡，可以是空。
    EveryParticipatingGpuShortOfFreeMemoryForTheSmallestPiece {
        gpus_short_of_free_memory: Vec<GpuShortOfFreeMemoryForTheSmallestPiece>,
        gpus_left_out_under_their_memory_limit: Vec<GpuLeftOutUnderItsMemoryLimit>,
    },
    /// 交进这一批的条目里同一个条目键出现了两次（两份核比对之前查出，连续不确定的计数一条不动、CPU 参照一次都不调）：是调用方组批的 bug，
    /// 不归哪张卡。两个位置是它在这一批里先后两次出现的位置。核心层收到它只放弃这一批：后台任务不停、不报警，下一批照常算
    /// （[`BackgroundComputeError::reach_of_the_error`]）。
    SameEntryKeyTwiceInOneBatch {
        position_of_the_first_occurrence: usize,
        position_of_the_second_occurrence: usize,
    },
    /// 调用方在停（[`CallerIsStoppingTheBackgroundGpu`] 置了）：实现在等卡的那一处放弃了那一等，这一段的判定一条都不交，那一次提交留在卡上
    /// （第 4 条「卸载时不等还在跑的提交，有这种提交的那张卡整张泄漏、不析构」）。不归哪张卡。调用方自己在停，不回锁、丢掉这一批（运行库就是这样做的：经
    /// `MountedSessionWithPendingChanges::note_the_batch_computed_out_of_the_lock_dropped_by_the_caller_without_relocking` 记下丢掉了它），
    /// 活意图留在盘上，下次挂载续做；交回锁里的照停下那个后台任务办（[`BackgroundComputeError::reach_of_the_error`]）。
    GaveUpWaitingForTheCallerIsStopping,
}

impl BackgroundGpuComputeFailed {
    /// 这个错点名的卡，按错里的次序：出在一张卡上的是那一张；参与的卡一张都不剩的是每一张被晾下的（因空闲显存不用的在前）；
    /// 不归哪张卡的（映射 key 比核装得下的长、一批里同一个条目键出现两次）是空。
    #[must_use]
    pub fn gpu_devices_named(&self) -> Vec<GpuDeviceIdentifier> {
        match self {
            BackgroundGpuComputeFailed::GpuFailed { device, .. }
            | BackgroundGpuComputeFailed::WaitForTheGpuExceededTheLimit { device, .. }
            | BackgroundGpuComputeFailed::GpuNotOpenedAfterTheReopenings { device, .. }
            | BackgroundGpuComputeFailed::GpuUnusableAfterResubmissions { device, .. }
            | BackgroundGpuComputeFailed::InputNotSupportedOnTheGpu(
                InputNotSupportedOnTheGpu::EntryLargerThanAChunkOnTheDevice { device, .. },
            ) => vec![device.clone()],
            BackgroundGpuComputeFailed::InputNotSupportedOnTheGpu(
                InputNotSupportedOnTheGpu::MappingKeyLongerThanTheKernelsHold { .. },
            ) => Vec::new(),
            BackgroundGpuComputeFailed::EveryParticipatingGpuLeftOutUnderItsMemoryLimit {
                gpus_left_out,
            } => gpus_left_out
                .iter()
                .map(|left_out| left_out.device.clone())
                .collect(),
            BackgroundGpuComputeFailed::EveryParticipatingGpuShortOfFreeMemoryForTheSmallestPiece {
                gpus_short_of_free_memory,
                gpus_left_out_under_their_memory_limit,
            } => gpus_short_of_free_memory
                .iter()
                .map(|short| short.device.clone())
                .chain(
                    gpus_left_out_under_their_memory_limit
                        .iter()
                        .map(|left_out| left_out.device.clone()),
                )
                .collect(),
            BackgroundGpuComputeFailed::SameEntryKeyTwiceInOneBatch { .. }
            | BackgroundGpuComputeFailed::GaveUpWaitingForTheCallerIsStopping => Vec::new(),
        }
    }

    /// 判那张卡用不了那一刻读的 PCI 状态：出在一张卡上、带着读数的三种交回它带的那一份（照样可以是没读）；
    /// 别的（等超过上限还没用完重交、输入不支持、参与的卡一张都不剩、一批里同一个条目键出现两次）是没读。
    #[must_use]
    pub fn pci_state_when_judged_unusable(&self) -> PciStateOfTheGpuReadWhenJudgedUnusable {
        match self {
            BackgroundGpuComputeFailed::GpuFailed {
                pci_state_when_judged_unusable,
                ..
            }
            | BackgroundGpuComputeFailed::GpuNotOpenedAfterTheReopenings {
                pci_state_when_judged_unusable,
                ..
            }
            | BackgroundGpuComputeFailed::GpuUnusableAfterResubmissions {
                pci_state_when_judged_unusable,
                ..
            } => PciStateOfTheGpuReadWhenJudgedUnusable::clone(pci_state_when_judged_unusable),
            BackgroundGpuComputeFailed::WaitForTheGpuExceededTheLimit { .. }
            | BackgroundGpuComputeFailed::InputNotSupportedOnTheGpu(..)
            | BackgroundGpuComputeFailed::EveryParticipatingGpuLeftOutUnderItsMemoryLimit { .. }
            | BackgroundGpuComputeFailed::EveryParticipatingGpuShortOfFreeMemoryForTheSmallestPiece {
                ..
            }
            | BackgroundGpuComputeFailed::SameEntryKeyTwiceInOneBatch { .. }
            | BackgroundGpuComputeFailed::GaveUpWaitingForTheCallerIsStopping => {
                PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError
            }
        }
    }
}

/// 一张卡这一批因空闲显存连最小的一片都建不成而不用（[`BackgroundGpuComputeFailed::EveryParticipatingGpuShortOfFreeMemoryForTheSmallestPiece`] 带着）：
/// 哪张卡、它最后那一段建不成时那一段合占的字节数（再压到它的一半就切不出最小的一片了）、这一段最小的一片要多少字节。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GpuShortOfFreeMemoryForTheSmallestPiece {
    pub device: GpuDeviceIdentifier,
    pub bytes_of_the_last_chunk_not_created: GpuMemoryInBytes,
    pub smallest_piece_bytes: GpuMemoryInBytes,
}

/// 一张卡因显存上限装不下最小的一片而不用：哪张卡、给它的上限、最小的一片要多少字节，
/// 与判的那一刻卡上已经占着的字节（这个进程里之前几次挂载泄漏在这张卡上的，或等超过上限之后还在跑的那一次提交的；不用的判据是
/// 「给的上限扣掉已占的不到最小的一片」）（[`BackgroundGpuComputeFailed::EveryParticipatingGpuLeftOutUnderItsMemoryLimit`] 带着；
/// 产品 GPU 包开卡、换设置与每一批切段时报的也是它）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GpuLeftOutUnderItsMemoryLimit {
    pub device: GpuDeviceIdentifier,
    pub memory_limit: GpuMemoryInBytes,
    pub smallest_piece_bytes: GpuMemoryInBytes,
    pub bytes_already_held_on_the_device: GpuMemoryInBytes,
}

/// 一张卡出错的原因（[`BackgroundGpuComputeFailed::GpuFailed`] 带着）。`reason` 是 GPU 那一侧报的原话。封闭集合。
/// 开卡那几种（起不了设备或编不了管线、起不了开卡线程、开卡 panic、前一次开卡没回来、叫停、连着试满）只在一张卡开不成的原因里出现
/// （[`BackgroundGpuComputeFailed::GpuNotOpenedAfterTheReopenings`] 的 `failures_of_each_opening`），出路各不相同。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GpuFailureOnTheDevice {
    /// wgpu 起不了这张卡的设备或编不了管线（驱动不对、显存不够；量代价那一段最小的一片装不进绑定上限也报这一种）。
    DeviceOrPipelineDidNotStart { reason: String },
    /// 系统起不了开卡的那条线程（`reason` 是起线程报的原话）：卡本身没碰过，查进程的线程与内存额度。
    OpeningThreadNotStarted { reason: String },
    /// 开卡那一步里 panic 了（`said_by_the_panic` 是 panic 带的原话；驱动里的与本包自己的断言都算）：照这一次没开成重开。
    OpeningPanicked { said_by_the_panic: String },
    /// 同一张卡上前一次开卡到点之后还没回来（那条线程还挂在驱动里），这一次不再起：出路只有重启进程。
    EarlierOpeningOfTheDeviceNotReturned,
    /// 开卡在开完之前叫停了（卸载、切到 `cpu`）：这一次没开，结局没人收。
    OpeningCalledOff,
    /// 这张卡在这一次挂载里已连着开了 `failed_openings` 次都没开成（开成一次清零），切换再点到它不再开（第 4 条：一张卡至多开 4 次）：要卸载重挂。
    OpeningsInThisMountUsedUp { failed_openings: u32 },
    /// GPU 那一侧报了没接住的错（校验错、显存不够）或设备丢了。
    UncapturedErrorOrDeviceLost { reason: String },
    /// 等一次提交没等成（不是超时）。
    WaitForTheSubmissionFailed { reason: String },
    /// 回读缓冲映射不成。
    ReadbackMappingFailed { reason: String },
    /// 读回的判定字认不得：核没写到那一条或写坏了。`entry_position` 是那一条在它那一次提交里的位置。
    KernelLeftAnUnrecognizedVerdictWord {
        entry_position: usize,
        verdict_word: u32,
    },
    /// 在卡上建一段的缓冲时显存不足（`bytes_held_by_the_chunk` 是那一段合占的字节数）。挂着时一批里遇到它按更小的片重切重交（第 3 条
    /// 「卡上此刻空闲的不到这个数也照常跑，只用空闲的那些」），切到最小的一片都建不成就这一批不用这张卡
    /// （[`BackgroundGpuComputeFailed::EveryParticipatingGpuShortOfFreeMemoryForTheSmallestPiece`]），不报出来；报出来的是挂载那一刻量代价时遇到的。
    OutOfMemoryCreatingTheBuffers {
        bytes_held_by_the_chunk: GpuMemoryInBytes,
        reason: String,
    },
}

/// 判一张卡用不了、要报警的那一刻照它的 PCI 地址读的本机 PCI 状态（D26（后台整理与放置回收） 已定项 13 第 4 条「判一张卡用不了、要报警的那一刻，查一次这张卡的硬件与链路」）。
/// 读在产品 GPU 包里做（`singlefs-background-gpu` 的 `pci_state_of_the_gpu` 模块）；核心层不读，只带着它进停下记录与报警，停着时每次叫起交回的报警带的是停下那一刻的同一份。
/// 读的时刻只有三种：挂着时一批在同一张卡上等超时用完、判它用不了（[`BackgroundGpuComputeFailed::GpuUnusableAfterResubmissions`]）；
/// 开卡重开用完都开不成（[`BackgroundGpuComputeFailed::GpuNotOpenedAfterTheReopenings`]，挂载那一刻拒绝挂载的那一份在产品 GPU 包的挂载拒绝里）；
/// 卡当场报出错（设备丢了、wgpu 报了没接住的错）使后台任务停下（[`BackgroundGpuComputeFailed::GpuFailed`] 带 [`GpuFailureOnTheDevice::UncapturedErrorOrDeviceLost`]）。
/// 封闭集合。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PciStateOfTheGpuReadWhenJudgedUnusable {
    /// 这个错不是那三种时刻之一，或不归哪张卡：没读。
    NotReadForThisError,
    /// 卡的标识当不了 PCI 设备目录下的一个目录名（不是 UTF-8、是空的、是 `.` 或 `..`、带 `/` 或 NUL 字节）：没读，免得读到那个目录之外去。
    PciBusAddressNotADirectoryName { device: GpuDeviceIdentifier },
    /// 读了：九样各自是读到、文件不在、读不通之一。装箱：九样连同地址与目录比另两个成员大三百多字节（clippy `large_enum_variant`），
    /// 这个枚举又放在出错成员里，不装箱就带得装着它的出错类型、报警与 `Result` 一起变大。
    Read(Box<PciStateOfTheGpu>),
}

impl PciStateOfTheGpuReadWhenJudgedUnusable {
    /// 照读数判这张卡在不在总线上（[`hardware_judgement_of_the_pci_state`]）；没读的交回 `None`。
    #[must_use]
    pub fn hardware_judgement(&self) -> Option<HardwareJudgementOfTheGpu> {
        match self {
            PciStateOfTheGpuReadWhenJudgedUnusable::Read(pci_state) => {
                Some(hardware_judgement_of_the_pci_state(pci_state))
            }
            PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError
            | PciStateOfTheGpuReadWhenJudgedUnusable::PciBusAddressNotADirectoryName { .. } => None,
        }
    }
}

/// 照一张卡的 PCI 地址读的那一份：照哪个地址、哪个目录读的，与九样各自的读数。读不到的那一样报读不到（[`PciFileReading::FileNotPresent`]、
/// [`PciFileReading::NotReadable`]），不当成 0，也不当成正常。链路四样（`current_link_speed`、`current_link_width`、`max_link_speed`、`max_link_width`）
/// 与高级错误报告的三类出错计数（`aer_dev_correctable`、`aer_dev_fatal`、`aer_dev_nonfatal`，每份是多行「名字 计数」）是文件原文、去掉末尾的换行，只报不判；
/// 判硬件坏只看目录在不在与使能位（[`hardware_judgement_of_the_pci_state`]）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PciStateOfTheGpu {
    /// 照这个地址读的：卡的标识原样（wgpu 适配器信息的 `device_pci_bus_id`，形如 `0000:01:00.0`）。
    pub pci_bus_address: String,
    /// 读的那个目录：PCI 设备目录（本机是 `/sys/bus/pci/devices`）下以地址为名的那一个。
    pub device_directory: PathBuf,
    /// 那个目录在不在：卡在不在总线上。
    pub device_directory_reading: PciDeviceDirectoryReading,
    /// `enable`：使能位。
    pub device_enable: PciFileReading<PciDeviceEnable>,
    /// `current_link_speed`：链路此刻的速度。
    pub current_link_speed: PciFileReading<String>,
    /// `current_link_width`：链路此刻的宽度。
    pub current_link_width: PciFileReading<String>,
    /// `max_link_speed`：链路最大的速度。
    pub maximum_link_speed: PciFileReading<String>,
    /// `max_link_width`：链路最大的宽度。
    pub maximum_link_width: PciFileReading<String>,
    /// `aer_dev_correctable`：可纠正的出错计数。
    pub advanced_error_reporting_correctable: PciFileReading<String>,
    /// `aer_dev_fatal`：致命的出错计数。
    pub advanced_error_reporting_fatal: PciFileReading<String>,
    /// `aer_dev_nonfatal`：不致命的出错计数。
    pub advanced_error_reporting_nonfatal: PciFileReading<String>,
}

/// 一张卡的 PCI 设备目录在不在（[`PciStateOfTheGpu::device_directory_reading`]）。封闭集合。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PciDeviceDirectoryReading {
    Present,
    /// 目录不在：卡不在总线上。
    NotPresent,
    /// 打开那个目录报了别的错（`system_error` 是系统报的原话）：读不通，不当成在、也不当成不在。
    NotReadable {
        system_error: String,
    },
}

/// PCI 设备目录下一个文件的读数。封闭集合。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PciFileReading<Content> {
    Read(Content),
    /// 文件不在（例：不报高级错误报告的设备没有那三份）。
    FileNotPresent,
    /// 读那个文件报了别的错（`system_error` 是系统报的原话）。
    NotReadable {
        system_error: String,
    },
}

/// `enable` 读到的：内核记的这个设备的使能计数（文件原文是一个十进制数），0 是关的。封闭集合。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PciDeviceEnable {
    Enabled {
        enable_count: u64,
    },
    Disabled,
    /// 原文不是一个十进制数（`text` 是去掉首尾空白的原文）：不判开关。
    UnrecognizedText {
        text: String,
    },
}

/// 照一张卡的 PCI 读数判的硬件好坏（[`hardware_judgement_of_the_pci_state`]）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HardwareJudgementOfTheGpu {
    /// 卡不在总线上：判硬件坏。
    OffTheBus(ReasonTheGpuIsOffTheBus),
    /// 没判出硬件坏（读到的链路与出错计数只报不判；使能位读不到也落在这里）。
    NotJudgedOffTheBus,
}

/// 判卡不在总线上的依据。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReasonTheGpuIsOffTheBus {
    /// 那个 PCI 地址的目录不在了。
    DeviceDirectoryNotPresent,
    /// 使能位读到了、是关的。
    DeviceDisabled,
}

/// 照一张卡的 PCI 读数判硬件坏没坏：只有那个地址的目录不在、或使能位读到了而是关的，才判卡不在总线上；
/// 目录读不通、使能位文件不在或读不通、原文认不得都不判坏；链路速度与宽度、出错计数不看。不读盘、不读内核导出的设备文件，只看交进来的读数。
#[must_use]
pub fn hardware_judgement_of_the_pci_state(
    pci_state: &PciStateOfTheGpu,
) -> HardwareJudgementOfTheGpu {
    match &pci_state.device_directory_reading {
        PciDeviceDirectoryReading::NotPresent => {
            return HardwareJudgementOfTheGpu::OffTheBus(
                ReasonTheGpuIsOffTheBus::DeviceDirectoryNotPresent,
            );
        }
        PciDeviceDirectoryReading::Present | PciDeviceDirectoryReading::NotReadable { .. } => {}
    }
    match &pci_state.device_enable {
        PciFileReading::Read(PciDeviceEnable::Disabled) => {
            HardwareJudgementOfTheGpu::OffTheBus(ReasonTheGpuIsOffTheBus::DeviceDisabled)
        }
        PciFileReading::Read(
            PciDeviceEnable::Enabled { .. } | PciDeviceEnable::UnrecognizedText { .. },
        )
        | PciFileReading::FileNotPresent
        | PciFileReading::NotReadable { .. } => HardwareJudgementOfTheGpu::NotJudgedOffTheBus,
    }
}

/// 等超过上限的是哪一次提交（[`BackgroundGpuComputeFailed::WaitForTheGpuExceededTheLimit`] 带着）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SubmissionWaitedForPastTheLimit {
    /// 这一批交给这张卡的那一次。
    SubmissionOfThisBatch,
    /// 这张卡上前一次等超过上限的那一次：这一批在这张卡上交活之前又等满了上限它还没做完（它的缓冲还占着卡上的显存，第 3 条）。
    EarlierTimedOutSubmissionStillRunning,
    /// 建一段的缓冲出错之后冲掉队列待写的那一次空提交：等满上限它还没做完（已建的那几份缓冲连同暂存还占着卡上的显存，第 3 条）。这一段没交到卡上。
    EmptySubmissionFlushingThePendingWrites,
}

/// 第一版不支持的输入（[`BackgroundGpuComputeFailed::InputNotSupportedOnTheGpu`] 带着）：条款只写了「一批放不下就切成几次交」，
/// 下面这几种切不小的怎么办没写。封闭集合。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InputNotSupportedOnTheGpu {
    /// 分到活的那张卡这一批连最小的一片都装不下，而装不下的不是显存上限（给的上限扣掉卡上已占的连最小的一片都装不下的卡，这一批不分给它，
    /// 第 3 条；参与的卡全都这样才报 [`BackgroundGpuComputeFailed::EveryParticipatingGpuLeftOutUnderItsMemoryLimit`]）：是单个缓冲的绑定上限。
    /// `entry_position` 是要切的那一条在这一批里的位置；`smallest_piece_bytes` 是那最小的一片（一个字节的输入、或乙段一个落点上的全部条目那一片，补到整字）
    /// 连同一段的固定占用合占的字节数，即这张卡至少要剩这么多（产品 GPU 包按这个数填）：是卡上显存的字节，不是主机上那一条输入的长度。
    EntryLargerThanAChunkOnTheDevice {
        device: GpuDeviceIdentifier,
        entry_position: usize,
        smallest_piece_bytes: GpuMemoryInBytes,
    },
    /// 映射里有一条 key 比核装得下的长：一张卡都还没分活就判出，不归哪张卡。`entry_position` 是它在映射条目里的位置。
    MappingKeyLongerThanTheKernelsHold {
        entry_position: usize,
        key_bytes: usize,
    },
}

/// 产品 GPU 包要实现的那一份：GC 搬迁放上 GPU 的三段各一个方法（第 1 条），两份独立的 GPU 核各算一次、逐条比，连续 2 轮不确定的条目交
/// CPU 那一份实现单独判（第 8 条）。连续不确定的计数随这份实现活：只在内存里，重挂换一份实现就清零。
/// 本机的卡清单也由它答（[`GpuInventory`]）：挂载与切换照它校验设置，没给实现就是 GPU 不在。
///
/// 三个方法的约定：交回的判定一条输入一条、按输入的次序；交 CPU 判的那一份就是核心层的 CPU 写法（甲 `checksum::crc32_castagnoli`，
/// 乙 `region_scan::scan_region_over_the_identity_index` 对单条记录，丙 `compaction_plan::live_slots_per_cluster_segment_by_the_records`）。
/// 出错时这一批的判定一条都不交、计数不动。条数对不上是实现的 bug，核心层断言。
pub trait BackgroundGpuCompute: GpuInventory {
    /// 甲：每个单元的整单元 CRC-32C 与 `expected_crc32c` 比。
    ///
    /// # Errors
    /// [`BackgroundGpuComputeFailed`]。
    fn verify_unit_checksums(
        &mut self,
        units: &[UnitChecksumToVerifyOnTheGpu<'_>],
    ) -> Result<Vec<GpuEntryDecision<UnitChecksumOutcome>>, BackgroundGpuComputeFailed>;

    /// 乙：`region` 里每条记录分一类，判法同核心层区域扫描在钉住、隔离两判之后那一段（读头 → 类标签 → key 与位置）。
    /// `mapping_entries` 是这一版的中央映射条目（区域扫描建按身份索引的那一份）。
    ///
    /// # Errors
    /// [`BackgroundGpuComputeFailed`]。
    fn classify_region_records(
        &mut self,
        region: CompactionRegion,
        records: &[RegionRecordToClassifyOnTheGpu<'_>],
        mapping_entries: &[CentralMappingEntry],
    ) -> Result<Vec<GpuEntryDecision<AllocationInRegion>>, BackgroundGpuComputeFailed>;

    /// 丙：`unit_area` 那块盘上每个整段的活槽数（未释放记录罩住的槽数），`segments` 是这几个整段按起点升序的条目键，一段一条。
    ///
    /// # Errors
    /// [`BackgroundGpuComputeFailed`]。
    fn count_live_slots_per_cluster_segment(
        &mut self,
        allocation_records: &[AllocationRecord],
        unit_area: UnitAreaOfOneDevice,
        segments: &[ClusterSegmentEntryKey],
    ) -> Result<Vec<GpuEntryDecision<u64>>, BackgroundGpuComputeFailed>;

    /// 从下一批起照这一组设置跑（第 1–3 条）：只在 `settings` 点的那几张卡上、按卡的顺序填满分活，每张卡上合占的显存不超过
    /// 给它的上限，每一处等 GPU 不超过等的上限。核心层在挂载那一刻（选 `gpu` 时）与每次通过校验的切换生效的那一批开头、开算之前调，
    /// 一批里三段都算完之前不再调（一批之内不换）。开着的卡实现不为它重开设备，三段的连续不确定计数照旧留着（第 8 条：只在重挂时清零）。
    /// `settings` 点到没开着的卡时，实现可以不在这一刻换上：在会话锁外开卡、开完了在之后的两批之间换上
    /// （[`Self::put_the_gpus_opened_since_the_settings_were_handed_in_in_effect`] 交回 [`LatestGpuSettingsHandedIn::PutInEffect`] 那一问），
    /// 开着的这段时间照旧照交来之前那一组跑，核心层把这一组记成还没生效。
    /// `settings` 的卡都在这份实现答的卡清单里（核心层照这份清单校验过）。
    fn use_the_settings_from_the_next_batch(&mut self, settings: &AdmittedGpuSettings);

    /// 从下一批起一张卡都不参与（核心层在挂载那一刻与切换生效的那一批开头，生效的那一组选 `cpu` 时调；第 2 条「切换之后不再参与的卡，切换生效时关掉」
    /// 用于切到 `cpu`：一张卡都不参与）：开着的卡全部关掉，还在等开卡的那一组与没取走的开卡结局丢掉、开卡叫停。之后再交来一组 `gpu` 的设置，
    /// 它点的卡都当没开的卡照重开的规矩开。不开卡的实现（测试里的假实现）没有这一样，什么都不做。
    fn use_no_gpu_from_the_next_batch(&mut self) {}

    /// 两批之间（每一批开头与整理两个入口判停没停之前，`BackgroundComputeOfThisMount::put_the_switch_requested_from_the_next_batch_in_effect` 里；
    /// 交设置之后当场也问一次）：最近交来的那一组此刻怎样了（[`LatestGpuSettingsHandedIn`]）。在会话锁外开的卡开完了就在这一刻换上；
    /// 还在开的不等。核心层只在交给实现的那一组还没生效时照答案办，别的时候问它只为让实现收下已开完的卡。
    /// 不开卡的实现（测试里的假实现）交来那一刻就换上：答换上了，连同 [`Self::failure_under_the_settings_handed_in`]。
    fn put_the_gpus_opened_since_the_settings_were_handed_in_in_effect(
        &mut self,
    ) -> LatestGpuSettingsHandedIn {
        LatestGpuSettingsHandedIn::PutInEffect {
            failure_under_the_settings: self.failure_under_the_settings_handed_in(),
        }
    }

    /// 照最近一次换上的那一组设置一张卡都用不上时，那一组下每一批三段都会交回的那个出错
    /// （例如参与的卡的显存上限连最小的一片都装不下、一张都不剩）；用得上交回 `None`。核心层不直接读它：
    /// [`Self::put_the_gpus_opened_since_the_settings_were_handed_in_in_effect`] 的缺省答法把它带进「换上了」那一种，
    /// 核心层照它判停（[`BackgroundComputeError::GpuUnusableUnderTheSettingsHandedIn`]），不等那一组下的头一批交回出错。只读，不交活。
    /// 不开卡的实现（测试里的假实现）没有这一样，交回 `None`。
    fn failure_under_the_settings_handed_in(&self) -> Option<BackgroundGpuComputeFailed> {
        None
    }

    /// 上一次交回以来三段各自在卡上是怎么交的（交了几片、重交了几次、用了哪几张卡），交回之后清零。核心层每一批三段算完（含出错放弃）之后读一次，
    /// 记进这一批的记账（[`GpuWorkOfTheBatch::submissions_on_the_gpus`]）。不开卡的实现（测试里的假实现）没有这一样，交回什么都没交的那一份。
    fn take_the_submissions_of_the_batch(&mut self) -> GpuSubmissionsOfTheBatch {
        GpuSubmissionsOfTheBatch::default()
    }

    /// 收下调用方在停的标记：建把手时交一次（[`BackgroundGpuComputeHandle::of_an_implementation`]、
    /// [`BackgroundGpuComputeHandle::of_a_shared_implementation`]）。调用方在停（运行库卸载、丢弃）时经把手置它
    /// （[`BackgroundGpuComputeHandle::tell_the_implementation_the_caller_is_stopping`]），意思是「调用方在停：正在等卡的那几处尽快放弃」：
    /// 实现每一处等卡的地方醒来就读它，置了就不再等、交回 [`BackgroundGpuComputeFailed::GaveUpWaitingForTheCallerIsStopping`]，
    /// 那一次提交留在卡上（D26（后台整理与放置回收） 已定项 13 第 4 条「卸载时不等还在跑的提交」）。调用方不停了（卸载没做成、运行库交回接着用）就撤掉它
    /// （[`BackgroundGpuComputeHandle::tell_the_implementation_the_caller_is_no_longer_stopping`]），之后照常等卡。
    /// 不等卡的实现（测试里的假实现）没有这一样，什么都不做。
    fn give_up_waiting_for_the_cards_once_the_caller_is_stopping(
        &mut self,
        _caller_is_stopping: CallerIsStoppingTheBackgroundGpu,
    ) {
    }
}

/// 调用方在停的标记（[`BackgroundGpuCompute::give_up_waiting_for_the_cards_once_the_caller_is_stopping`]）：住在把手这一层的一个原子量，
/// 克隆出的几份与把手共用它。只由把手造出、只由把手置与撤（[`BackgroundGpuComputeHandle::tell_the_implementation_the_caller_is_stopping`]、
/// [`BackgroundGpuComputeHandle::tell_the_implementation_the_caller_is_no_longer_stopping`]），置与撤都不取实现那把锁。
#[derive(Clone, Debug)]
pub struct CallerIsStoppingTheBackgroundGpu {
    is_the_caller_stopping: Arc<AtomicBool>,
}

impl CallerIsStoppingTheBackgroundGpu {
    fn caller_not_stopping_yet() -> Self {
        Self {
            is_the_caller_stopping: Arc::new(AtomicBool::new(false)),
        }
    }

    fn mark_the_caller_stopping(&self) {
        self.is_the_caller_stopping.store(true, Ordering::SeqCst);
    }

    fn mark_the_caller_no_longer_stopping(&self) {
        self.is_the_caller_stopping.store(false, Ordering::SeqCst);
    }

    /// 调用方在停了没有：等卡的那一处每次醒来读它。
    #[must_use]
    pub fn is_the_caller_stopping(&self) -> bool {
        self.is_the_caller_stopping.load(Ordering::SeqCst)
    }
}

/// 两批之间问实现的那一问（[`BackgroundGpuCompute::put_the_gpus_opened_since_the_settings_were_handed_in_in_effect`]）交回的：
/// 最近交给它的那一组此刻怎样了。核心层照它定这一组什么时候生效（第 4 条「开成了，下一批起换成新设置」）。封闭集合。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LatestGpuSettingsHandedIn {
    /// 实现还在会话锁外为它开卡：照旧照交来之前那一组跑，这一组还没生效。
    StillOpeningTheirCards,
    /// 换上了：从下一批起照它跑。`failure_under_the_settings` 是照它一张卡都用不上时每一批三段都会交回的那个出错（用得上是 `None`），
    /// 核心层照挂载那一刻的停法判（[`BackgroundComputeError::GpuUnusableUnderTheSettingsHandedIn`]）。
    PutInEffect {
        failure_under_the_settings: Option<BackgroundGpuComputeFailed>,
    },
    /// 它点到的一张卡重开到上限都没开成，或这张卡在这一次挂载里已连着试满、不再开（[`BackgroundGpuComputeFailed::GpuNotOpenedAfterTheReopenings`]）：
    /// 这一组不生效，实现照旧照交来之前那一组（核心层生效的那一组）；核心层把有 GPU 路径的后台任务当场记成出错停着、报警，不自动再试，
    /// 直到下一次通过校验的切换。
    NotOpened { failure: BackgroundGpuComputeFailed },
}

/// 调用方递进来的那一份 GPU 实现的把手：几处共用同一份（克隆的是把手，实现与它的连续不确定计数只有一份）。
/// 两份把手相等当且仅当指着同一份实现。另带一份卡清单的快照（[`GpuInventory`] 答的）：挂着切换的校验不等实现那把锁
/// （锁外算一批时它可能正被借着，会话锁里的切换不在它上面等）——那把锁空着就当场问实现、把答的记进快照，被借着（或中毒）就读快照。
/// 快照在建把手时取，之后每次校验当场问到时换（实现换卡、开卡之后答的卡清单随之进快照）。
/// 还带调用方在停的标记（[`CallerIsStoppingTheBackgroundGpu`]）：建把手时交给实现一次，之后置它不取实现那把锁。
#[derive(Clone)]
pub struct BackgroundGpuComputeHandle {
    shared: Arc<Mutex<dyn BackgroundGpuCompute + Send>>,
    inventory_snapshot: Arc<Mutex<Vec<(GpuDeviceIdentifier, GpuMemoryInBytes)>>>,
    caller_is_stopping: CallerIsStoppingTheBackgroundGpu,
}

/// GPU 实现那把锁中毒了：实现上一次被借出时 panic 过（那一次的 panic 已经往上传过）。核心层不带着可能半改的连续不确定计数再用它，
/// 照这个错停下有 GPU 路径的后台任务（[`BackgroundComputeError::GpuImplementationPanickedWhileLent`]），不 panic。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GpuImplementationPanickedWhileLent;

impl BackgroundGpuComputeHandle {
    /// 建把手：取卡清单的快照，把调用方在停的标记交给实现。
    #[must_use]
    pub fn of_an_implementation(
        mut implementation: impl BackgroundGpuCompute + Send + 'static,
    ) -> Self {
        let inventory_snapshot = implementation.devices_with_total_memory();
        let caller_is_stopping = CallerIsStoppingTheBackgroundGpu::caller_not_stopping_yet();
        implementation
            .give_up_waiting_for_the_cards_once_the_caller_is_stopping(caller_is_stopping.clone());
        Self {
            shared: Arc::new(Mutex::new(implementation)),
            inventory_snapshot: Arc::new(Mutex::new(inventory_snapshot)),
            caller_is_stopping,
        }
    }

    /// 调用方自己也留着一份的实现（测试读它记下的调用、改注入的故障）。建把手时借一次实现：取卡清单的快照，把调用方在停的标记交给它；
    /// 实现那把锁已中毒时快照是空的（切到 `gpu` 照 GPU 不在拒），标记也交不进去（之后借它都交回中毒那个错，不会再等卡）。
    #[must_use]
    pub fn of_a_shared_implementation(shared: Arc<Mutex<dyn BackgroundGpuCompute + Send>>) -> Self {
        let caller_is_stopping = CallerIsStoppingTheBackgroundGpu::caller_not_stopping_yet();
        let inventory_snapshot = shared
            .lock()
            .map(|mut implementation| {
                implementation.give_up_waiting_for_the_cards_once_the_caller_is_stopping(
                    caller_is_stopping.clone(),
                );
                implementation.devices_with_total_memory()
            })
            .unwrap_or_default();
        Self {
            shared,
            inventory_snapshot: Arc::new(Mutex::new(inventory_snapshot)),
            caller_is_stopping,
        }
    }

    /// 告诉实现调用方在停（运行库卸载、丢弃时调）：只置把手里那个原子标记，不取实现那把锁（锁外那一段此刻可能正借着它在等卡），
    /// 实现等卡的那几处醒来读到就放弃那一等（[`BackgroundGpuCompute::give_up_waiting_for_the_cards_once_the_caller_is_stopping`]）。
    /// 克隆出的把手共用这一个标记；调用方不停了才撤（[`Self::tell_the_implementation_the_caller_is_no_longer_stopping`]）。
    pub fn tell_the_implementation_the_caller_is_stopping(&self) {
        self.caller_is_stopping.mark_the_caller_stopping();
    }

    /// 告诉实现调用方不停了（运行库卸载时排空待发被拒、把运行库交回调用方接着用时调）：撤掉 [`Self::tell_the_implementation_the_caller_is_stopping`]
    /// 置的那个原子标记，同样不取实现那把锁；之后实现等卡的那几处照常等。克隆出的把手共用这一个标记。
    pub fn tell_the_implementation_the_caller_is_no_longer_stopping(&self) {
        self.caller_is_stopping.mark_the_caller_no_longer_stopping();
    }

    /// 借出实现（一批里每一段调用借一次、调完就放；两批之间交设置与问结局各借一次）。
    ///
    /// # Errors
    /// 实现那把锁中毒（上一次借出时 panic 过）⇒ [`GpuImplementationPanickedWhileLent`]。
    pub(crate) fn lock(
        &self,
    ) -> Result<
        MutexGuard<'_, dyn BackgroundGpuCompute + Send + 'static>,
        GpuImplementationPanickedWhileLent,
    > {
        self.shared
            .lock()
            .map_err(|_poisoned| GpuImplementationPanickedWhileLent)
    }

    /// 校验要的卡清单，不等实现那把锁：空着就当场问实现、答的记进快照再交回；被别处借着（锁外正算着一批）或中毒就交回快照。
    #[must_use]
    pub(crate) fn inventory_without_waiting(&self) -> Vec<(GpuDeviceIdentifier, GpuMemoryInBytes)> {
        match self.shared.try_lock() {
            Ok(implementation) => {
                let inventory_now = implementation.devices_with_total_memory();
                drop(implementation);
                self.inventory_snapshot
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .clone_from(&inventory_now);
                inventory_now
            }
            Err(TryLockError::WouldBlock | TryLockError::Poisoned(_)) => self
                .inventory_snapshot
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone(),
        }
    }
}

impl fmt::Debug for BackgroundGpuComputeHandle {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BackgroundGpuComputeHandle")
            .finish_non_exhaustive()
    }
}

impl PartialEq for BackgroundGpuComputeHandle {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.shared, &other.shared)
    }
}

impl Eq for BackgroundGpuComputeHandle {}

/// 一段里确认、不确定、交 CPU 判的条目各多少（第 8 条「每一批报确认、不确定、交 CPU 判的条目各多少」）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GpuEntryCounts {
    pub confirmed_by_both_kernels: u64,
    pub uncertain_this_round: u64,
    pub judged_by_the_cpu_reference: u64,
}

impl GpuEntryCounts {
    /// 把这一段交回的判定记进来。
    pub(crate) fn note_the_decisions<Outcome>(&mut self, decisions: &[GpuEntryDecision<Outcome>]) {
        for decision in decisions {
            match decision {
                GpuEntryDecision::ConfirmedByBothKernels(_) => self.confirmed_by_both_kernels += 1,
                GpuEntryDecision::UncertainThisRound => self.uncertain_this_round += 1,
                GpuEntryDecision::JudgedByTheCpuReference(_) => {
                    self.judged_by_the_cpu_reference += 1;
                }
            }
        }
    }
}

/// 一段在卡上是怎么交的（`.claude/rules/fs-design.md` 五条硬要求第 4 条：运行时看得出切没切片、重交了几次、用了哪几张卡）：
/// 交成的提交几次、这几次合起来交了几片（整条一片、切开的一条几片），等超过上限之后在同一张卡上重交了几次，建缓冲显存不足之后按更小的片重切了几次，
/// 分到活的是哪几张卡（按分活的次序，同一张卡只列一次）。一批里同一段算了几次（选段时逐段先扫）的，各次加起来。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GpuSubmissionsOfOneSegment {
    pub submissions_completed: u64,
    pub pieces_submitted: u64,
    pub resubmissions_after_timeouts: u64,
    pub recuts_after_running_out_of_memory: u64,
    pub devices_given_work: Vec<GpuDeviceIdentifier>,
}

impl GpuSubmissionsOfOneSegment {
    /// 把同一段后来算的那一次加进来：各数相加，卡照先后并起来、不重列。
    pub fn add(&mut self, later: &GpuSubmissionsOfOneSegment) {
        self.submissions_completed += later.submissions_completed;
        self.pieces_submitted += later.pieces_submitted;
        self.resubmissions_after_timeouts += later.resubmissions_after_timeouts;
        self.recuts_after_running_out_of_memory += later.recuts_after_running_out_of_memory;
        for device in &later.devices_given_work {
            if !self.devices_given_work.contains(device) {
                self.devices_given_work.push(device.clone());
            }
        }
    }
}

/// 一批三段各自在卡上是怎么交的（[`GpuSubmissionsOfOneSegment`]），由实现在一批算完之后交回（[`BackgroundGpuCompute::take_the_submissions_of_the_batch`]）。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GpuSubmissionsOfTheBatch {
    pub unit_checksum_verification: GpuSubmissionsOfOneSegment,
    pub region_record_classification: GpuSubmissionsOfOneSegment,
    pub live_slots_per_cluster_segment: GpuSubmissionsOfOneSegment,
}

impl GpuSubmissionsOfTheBatch {
    /// `segment` 那一段的那一份。
    pub fn of_the_segment_mut(
        &mut self,
        segment: GpuComputedSegment,
    ) -> &mut GpuSubmissionsOfOneSegment {
        match segment {
            GpuComputedSegment::UnitChecksumVerification => &mut self.unit_checksum_verification,
            GpuComputedSegment::RegionRecordClassification => {
                &mut self.region_record_classification
            }
            GpuComputedSegment::LiveSlotsPerClusterSegment => {
                &mut self.live_slots_per_cluster_segment
            }
        }
    }
}

/// 一批在 GPU 上算了什么：三段各自的条目数，甲段判成整单元 CRC-32C 对不上的单元数，与三段各自在卡上是怎么交的。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GpuWorkOfTheBatch {
    pub unit_checksum_verification: GpuEntryCounts,
    pub region_record_classification: GpuEntryCounts,
    pub live_slots_per_cluster_segment: GpuEntryCounts,
    /// 甲段确认或交 CPU 判出整单元 CRC-32C 与位置项对不上的可搬单元数。它们照第 8 条交 CPU：发布路径照常逐条核计划、释放之前读盘核
    /// （那一核读出对不上的那一份隔离，D19（块指针的结构与宽度预算） 已定项 5），区域扫描不因它改分类，结局与 CPU 路径相同；这里只报。
    pub units_whose_checksum_mismatched_on_the_gpu: u64,
    /// 三段各自交了几片、重交了几次、用了哪几张卡（实现在这一批算完之后交回的，出错放弃的那一批是出错之前与出错那一段交的）。
    pub submissions_on_the_gpus: GpuSubmissionsOfTheBatch,
}

impl GpuWorkOfTheBatch {
    /// 三段合起来的条目数。
    #[must_use]
    pub fn entries_of_the_three_segments(&self) -> GpuEntryCounts {
        let segments = [
            self.unit_checksum_verification,
            self.region_record_classification,
            self.live_slots_per_cluster_segment,
        ];
        GpuEntryCounts {
            confirmed_by_both_kernels: segments
                .iter()
                .map(|counts| counts.confirmed_by_both_kernels)
                .sum(),
            uncertain_this_round: segments
                .iter()
                .map(|counts| counts.uncertain_this_round)
                .sum(),
            judged_by_the_cpu_reference: segments
                .iter()
                .map(|counts| counts.judged_by_the_cpu_reference)
                .sum(),
        }
    }
}

/// 一段交回的判定：出错就转成这一批的错（[`BackgroundComputeError::GpuSegmentFailed`]），否则记进 `counts` 再交回。
///
/// # Panics
/// 判定的条数与输入的条数不等：违反 [`BackgroundGpuCompute`] 的约定，是实现的 bug。
pub(crate) fn decisions_of_one_segment<Outcome>(
    segment: GpuComputedSegment,
    entries_handed_in: usize,
    returned: Result<Vec<GpuEntryDecision<Outcome>>, BackgroundGpuComputeFailed>,
    counts: &mut GpuEntryCounts,
) -> Result<Vec<GpuEntryDecision<Outcome>>, BackgroundComputeError> {
    let decisions = returned
        .map_err(|failure| BackgroundComputeError::GpuSegmentFailed { segment, failure })?;
    assert_eq!(
        decisions.len(),
        entries_handed_in,
        "GPU 实现一条输入交一条判定（BackgroundGpuCompute 的约定）：{segment:?} 段交进 {entries_handed_in} 条"
    );
    counts.note_the_decisions(&decisions);
    Ok(decisions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gpu_memory_in_bytes_checked_add_gives_the_sum_and_refuses_past_u64() {
        assert_eq!(
            GpuMemoryInBytes(600).checked_add(GpuMemoryInBytes(400)),
            Some(GpuMemoryInBytes(1000))
        );
        assert_eq!(
            GpuMemoryInBytes(u64::MAX).checked_add(GpuMemoryInBytes(1)),
            None
        );
    }

    #[test]
    fn gpu_memory_in_bytes_checked_sub_gives_what_is_left_and_refuses_taking_more_than_there_is() {
        assert_eq!(
            GpuMemoryInBytes(1000).checked_sub(GpuMemoryInBytes(600)),
            Some(GpuMemoryInBytes(400))
        );
        assert_eq!(
            GpuMemoryInBytes(600).checked_sub(GpuMemoryInBytes(1000)),
            None
        );
    }

    #[test]
    fn gpu_memory_in_bytes_plus_gives_the_sum() {
        assert_eq!(
            GpuMemoryInBytes(16).plus(GpuMemoryInBytes(40)),
            GpuMemoryInBytes(56)
        );
    }

    #[test]
    #[should_panic(expected = "显存字节数之和超出 u64")]
    fn gpu_memory_in_bytes_plus_past_u64_panics() {
        let _sum = GpuMemoryInBytes(u64::MAX).plus(GpuMemoryInBytes(1));
    }

    #[test]
    fn gpu_memory_in_bytes_summed_over_the_parts_is_their_total() {
        let total: GpuMemoryInBytes = [8_u64, 16, 24].into_iter().map(GpuMemoryInBytes).sum();
        assert_eq!(total, GpuMemoryInBytes(48));
    }

    /// 同一段一批里算了两次（选段时逐段先扫）：各数相加，卡照先后并起来、同一张卡不重列。
    #[test]
    fn submissions_of_a_segment_computed_twice_in_a_batch_add_up_and_list_each_device_once() {
        let card = |address: &[u8]| GpuDeviceIdentifier::of_opaque_bytes(address.to_vec());
        let mut submissions = GpuSubmissionsOfOneSegment {
            submissions_completed: 2,
            pieces_submitted: 5,
            resubmissions_after_timeouts: 1,
            recuts_after_running_out_of_memory: 0,
            devices_given_work: vec![card(b"0000:41:00.0")],
        };
        submissions.add(&GpuSubmissionsOfOneSegment {
            submissions_completed: 3,
            pieces_submitted: 4,
            resubmissions_after_timeouts: 0,
            recuts_after_running_out_of_memory: 2,
            devices_given_work: vec![card(b"0000:01:00.0"), card(b"0000:41:00.0")],
        });
        assert_eq!(
            submissions,
            GpuSubmissionsOfOneSegment {
                submissions_completed: 5,
                pieces_submitted: 9,
                resubmissions_after_timeouts: 1,
                recuts_after_running_out_of_memory: 2,
                devices_given_work: vec![card(b"0000:41:00.0"), card(b"0000:01:00.0")],
            }
        );
    }

    /// 第 4 条「只有卡不在总线上（那个 PCI 地址的目录不在了，或使能位是关的）才判硬件坏」：目录不在就判不在总线上，使能位读没读到都一样。
    #[test]
    fn the_gpu_whose_device_directory_is_not_present_is_judged_off_the_bus() {
        assert_eq!(
            hardware_judgement_of_the_pci_state(&pci_state_of_the_card_at_one(
                PciDeviceDirectoryReading::NotPresent,
                PciFileReading::FileNotPresent,
            )),
            HardwareJudgementOfTheGpu::OffTheBus(
                ReasonTheGpuIsOffTheBus::DeviceDirectoryNotPresent
            )
        );
    }

    /// 同一条：目录在、使能位读到 0（关的）判不在总线上。
    #[test]
    fn the_gpu_whose_device_enable_reads_disabled_is_judged_off_the_bus() {
        assert_eq!(
            hardware_judgement_of_the_pci_state(&pci_state_of_the_card_at_one(
                PciDeviceDirectoryReading::Present,
                PciFileReading::Read(PciDeviceEnable::Disabled),
            )),
            HardwareJudgementOfTheGpu::OffTheBus(ReasonTheGpuIsOffTheBus::DeviceDisabled)
        );
    }

    /// 同一条「链路速度与宽度、出错计数只报不判」：目录在、使能位开着，链路降速降宽、出错计数不是 0，也不判坏。
    #[test]
    fn the_gpu_enabled_with_a_degraded_link_and_errors_counted_is_not_judged_off_the_bus() {
        assert_eq!(
            hardware_judgement_of_the_pci_state(&pci_state_of_the_card_at_one(
                PciDeviceDirectoryReading::Present,
                PciFileReading::Read(PciDeviceEnable::Enabled { enable_count: 1 }),
            )),
            HardwareJudgementOfTheGpu::NotJudgedOffTheBus
        );
    }

    /// 同一条「读不到的那一样报读不到，不当成 0」：目录在，使能位文件不在、读不通、原文认不得，都不判坏（不当成读到了关的）。
    #[test]
    fn the_gpu_whose_device_enable_is_not_read_is_not_judged_off_the_bus() {
        let judgements: Vec<HardwareJudgementOfTheGpu> = [
            PciFileReading::FileNotPresent,
            PciFileReading::NotReadable {
                system_error: "替身：读不通".to_string(),
            },
            PciFileReading::Read(PciDeviceEnable::UnrecognizedText {
                text: "替身：认不得".to_string(),
            }),
        ]
        .into_iter()
        .map(|device_enable| {
            hardware_judgement_of_the_pci_state(&pci_state_of_the_card_at_one(
                PciDeviceDirectoryReading::Present,
                device_enable,
            ))
        })
        .collect();
        assert_eq!(
            judgements,
            vec![HardwareJudgementOfTheGpu::NotJudgedOffTheBus; 3]
        );
    }

    /// 同一条：目录读不通（不是不在）不判坏，照使能位判。
    #[test]
    fn the_gpu_whose_device_directory_is_not_readable_is_judged_by_its_device_enable() {
        assert_eq!(
            hardware_judgement_of_the_pci_state(&pci_state_of_the_card_at_one(
                PciDeviceDirectoryReading::NotReadable {
                    system_error: "替身：打不开".to_string(),
                },
                PciFileReading::Read(PciDeviceEnable::Enabled { enable_count: 2 }),
            )),
            HardwareJudgementOfTheGpu::NotJudgedOffTheBus
        );
    }

    /// 只记下收到的调用方在停的标记的替身（三段不该被调）：标记放在用例也留着一份的格子里，实现那把锁被借着时用例照样读得到。
    struct GpuComputeKeepingTheCallerStoppingFlag {
        flag_received: Arc<Mutex<Option<CallerIsStoppingTheBackgroundGpu>>>,
    }

    impl GpuInventory for GpuComputeKeepingTheCallerStoppingFlag {
        fn devices_with_total_memory(&self) -> Vec<(GpuDeviceIdentifier, GpuMemoryInBytes)> {
            Vec::new()
        }
    }

    impl BackgroundGpuCompute for GpuComputeKeepingTheCallerStoppingFlag {
        fn verify_unit_checksums(
            &mut self,
            _units: &[UnitChecksumToVerifyOnTheGpu<'_>],
        ) -> Result<Vec<GpuEntryDecision<UnitChecksumOutcome>>, BackgroundGpuComputeFailed>
        {
            panic!("这条用例不算一批");
        }

        fn classify_region_records(
            &mut self,
            _region: CompactionRegion,
            _records: &[RegionRecordToClassifyOnTheGpu<'_>],
            _mapping_entries: &[CentralMappingEntry],
        ) -> Result<Vec<GpuEntryDecision<AllocationInRegion>>, BackgroundGpuComputeFailed> {
            panic!("这条用例不算一批");
        }

        fn count_live_slots_per_cluster_segment(
            &mut self,
            _allocation_records: &[AllocationRecord],
            _unit_area: UnitAreaOfOneDevice,
            _segments: &[ClusterSegmentEntryKey],
        ) -> Result<Vec<GpuEntryDecision<u64>>, BackgroundGpuComputeFailed> {
            panic!("这条用例不算一批");
        }

        fn use_the_settings_from_the_next_batch(&mut self, _settings: &AdmittedGpuSettings) {}

        fn give_up_waiting_for_the_cards_once_the_caller_is_stopping(
            &mut self,
            caller_is_stopping: CallerIsStoppingTheBackgroundGpu,
        ) {
            *self.flag_received.lock().expect("记标记的格子里不 panic") = Some(caller_is_stopping);
        }
    }

    /// 建把手时把调用方在停的标记交给实现；把手说调用方在停之后，实现手里那一份读到在停（自有的实现那一种建法）。
    #[test]
    fn a_handle_of_an_implementation_hands_it_the_flag_that_the_handle_sets_when_the_caller_is_stopping(
    ) {
        let flag_received = Arc::new(Mutex::new(None));
        let handle = BackgroundGpuComputeHandle::of_an_implementation(
            GpuComputeKeepingTheCallerStoppingFlag {
                flag_received: Arc::clone(&flag_received),
            },
        );
        let flag_of_the_implementation = flag_received
            .lock()
            .expect("记标记的格子里不 panic")
            .clone()
            .expect("建把手时把标记交给了实现");
        assert!(
            !flag_of_the_implementation.is_the_caller_stopping(),
            "建好时调用方没在停"
        );
        handle
            .clone()
            .tell_the_implementation_the_caller_is_stopping();
        assert!(
            flag_of_the_implementation.is_the_caller_stopping(),
            "克隆出的把手说调用方在停，实现手里那一份读到在停"
        );
    }

    /// 把手说调用方在停不取实现那把锁：那把锁被另一条线程借着时（锁外那一段正在等卡的样子），这一句立刻返回，实现手里那一份读到在停
    /// （调用方也留着一份的实现那一种建法）。
    #[test]
    fn a_handle_tells_the_implementation_lent_to_another_thread_that_the_caller_is_stopping_without_waiting(
    ) {
        let flag_received = Arc::new(Mutex::new(None));
        let shared: Arc<Mutex<dyn BackgroundGpuCompute + Send>> =
            Arc::new(Mutex::new(GpuComputeKeepingTheCallerStoppingFlag {
                flag_received: Arc::clone(&flag_received),
            }));
        let handle = BackgroundGpuComputeHandle::of_a_shared_implementation(Arc::clone(&shared));
        let flag_of_the_implementation = flag_received
            .lock()
            .expect("记标记的格子里不 panic")
            .clone()
            .expect("建把手时把标记交给了实现");
        let (lent_sender, lent_receiver) = std::sync::mpsc::channel::<()>();
        let (release_sender, release_receiver) = std::sync::mpsc::channel::<()>();
        let lending_thread = std::thread::spawn({
            let shared = Arc::clone(&shared);
            move || {
                let _lent = shared.lock().expect("替身没在别处 panic 过");
                lent_sender.send(()).expect("用例在等借出");
                release_receiver.recv().expect("用例叫放锁");
            }
        });
        lent_receiver.recv().expect("另一条线程借着实现");
        let (told_sender, told_receiver) = std::sync::mpsc::channel::<()>();
        let telling_thread = std::thread::spawn(move || {
            handle.tell_the_implementation_the_caller_is_stopping();
            told_sender.send(()).expect("用例在等这一句返回");
        });
        let told_while_lent = told_receiver.recv_timeout(Duration::from_secs(5));
        let is_stopping_seen_while_lent = flag_of_the_implementation.is_the_caller_stopping();
        release_sender.send(()).expect("借着的那条线程在等放锁");
        lending_thread.join().expect("借着的那条线程不 panic");
        telling_thread.join().expect("说在停的那条线程不 panic");
        assert_eq!(
            told_while_lent,
            Ok(()),
            "实现那把锁被借着时说调用方在停立刻返回"
        );
        assert!(
            is_stopping_seen_while_lent,
            "实现那把锁还借着时，实现手里那一份已读到在停"
        );
    }
}

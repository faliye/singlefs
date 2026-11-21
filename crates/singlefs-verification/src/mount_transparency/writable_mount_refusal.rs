//! 可写挂载拒绝合不合法：D23（journal 的角色与格式） 已定项 14 那张单子上的理由在崩溃镜像上复算，连同乙入判不出之拒的计数。

use super::*;

/// D23（journal 的角色与格式） 已定项 14 那张单子（C554 乙-配置续；C331 判不出改取 R2）上的拒可写理由。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RefusalReasonOnTheLegalRefusalList {
    /// 读阶段判据 N-配置 为真（系统配置见证判不出按真：R2，有槽设备报读错而没有一块盘两槽都自证过），重读一次仍为真。
    NewerPublishWitnessedBySystemConfiguration,
    /// 重建分配器判抛弃用的最新那条根的实例表读不出，重读一次仍读不出。
    InstanceTableOfTheNewestRootUnreadable,
    /// 读阶段判完、判据 N-配置 为假之后，取号之前的逐盘核或取号那一刻读系统配置判不出（R2）或某块盘一份自证过的都没有
    /// （`MountError::SystemConfigurationUnreadableAfterTheReadStage`；C664（挂载拒绝合法复算把两种拒因报成一个）之前并在 N-配置 里报）。
    SystemConfigurationUnreadableAfterTheReadStage,
    /// 重建分配器读根环那一遍有一块盘一条不被抛弃的自证有效根都没读出、整环重读一次，两遍逐槽并起来整个环仍一条都没有
    /// （`MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread` Q3「另起拒因、单子加一行」，
    /// 岔路单 `research/prompts/m3-batch10-code-r1-forks.md`；之前报 `Recovery(NoValidRoot)`、与读阶段择不到根同名，不在单子上；
    /// 两遍按槽并照 R1，判决 `research/prompts/m3-batch10-code-r2-main-verification.md` 第六节：第一遍读出过一条有效根的不拒）。
    NoValidRootInTheRootRingOfTheRebuiltAllocator,
    /// 「记录有缺口时拒可写」（F1 乙；判据照判决 `research/prompts/m3-batch33t-code-r1-main-verification.md` 第二节 T4 那一行，`F1_ANY`）：
    /// 水位之上一次没施加的发布，读得出的那几条记录的本次发布内序号不是 1..N 连着或缺末条，而它们点名的单元里有一个两份副本都验不过，
    /// 重读一次仍这样（`StillUnreadableAfterOneReread::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify`）。
    PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify,
    /// 可写挂载的 C 读一次没施加的发布（水位之上那几条记录拼的根，或比所选那一版新、按实例表有效、读得出的根）的账读不出、那次发布读得出的记录
    /// 不齐、从中央映射也现算不成，重读一次仍这样（`C_MAPPING` 收窄，判决 `research/prompts/m3-batch35-36-code-r1-main-verification.md`
    /// 第二节 K2 W1 / W2 H1、K2 W2 H3 两行；`StillUnreadableAfterOneReread::AccountOfAnUnappliedPublishWithIncompleteReadableRecords`，
    /// 「乙：core 另起错误成员」、列进单子）。`unapplied_publish` 是那一拒点名的 (实例代号, checkpoint_txg)，复算按它找那次发布。
    AccountOfAnUnappliedPublishWithIncompleteReadableRecords {
        unapplied_publish: (InstanceGeneration, CheckpointTxg),
    },
}

/// 拒可写的理由在不在 D23 已定项 14 那张单子上。别的理由（单元区墙、坏盘上才有的那几种……）照崩溃注入第二截今天的写法判。
#[must_use]
pub fn refusal_reason_on_the_legal_refusal_list(
    error: &MountError,
) -> Option<RefusalReasonOnTheLegalRefusalList> {
    match error {
        MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable) => {
            match **still_unreadable {
                StillUnreadableAfterOneReread::PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion { .. } => {
                    Some(RefusalReasonOnTheLegalRefusalList::NewerPublishWitnessedBySystemConfiguration)
                }
                StillUnreadableAfterOneReread::InstanceTableOfTheNewestRootForTheShadowLedger { .. } => {
                    Some(RefusalReasonOnTheLegalRefusalList::InstanceTableOfTheNewestRootUnreadable)
                }
                StillUnreadableAfterOneReread::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify { .. } => {
                    Some(RefusalReasonOnTheLegalRefusalList::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify)
                }
                StillUnreadableAfterOneReread::AccountOfAnUnappliedPublishWithIncompleteReadableRecords {
                    unapplied_publish,
                } => Some(
                    RefusalReasonOnTheLegalRefusalList::AccountOfAnUnappliedPublishWithIncompleteReadableRecords {
                        unapplied_publish: (
                            unapplied_publish.instance,
                            unapplied_publish.checkpoint_txg,
                        ),
                    },
                ),
                // 挂着时抬 F、管理员回退那几处读坏，与影子账读一条被抛弃根的账读不出（C393）：不在 D23 已定项 14 那张单子上。
                StillUnreadableAfterOneReread::InstanceTableOfTheNewestRootForTheEffectiveFloor { .. }
                | StillUnreadableAfterOneReread::RootRingSlotKnownToHoldARoot { .. }
                | StillUnreadableAfterOneReread::AccountOfAnAbandonedRoot { .. } => None,
            }
        }
        MountError::SystemConfigurationUnreadableAfterTheReadStage(_) => {
            Some(RefusalReasonOnTheLegalRefusalList::SystemConfigurationUnreadableAfterTheReadStage)
        }
        MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread => {
            Some(RefusalReasonOnTheLegalRefusalList::NoValidRootInTheRootRingOfTheRebuiltAllocator)
        }
        // 恢复报的错（读阶段择不到根那一拒在内，只读挂载同样挂不上）：不在单子上。
        MountError::Recovery(_)
        | MountError::FileVersionWithoutAnyJournalRecord
        | MountError::InstanceTableMalformed
        | MountError::InstanceTableUnreadable
        | MountError::Acquisition(_)
        | MountError::Publish(_)
        | MountError::RaiseFloorSequencePublishFailed(_)
        | MountError::RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite { .. }
        | MountError::RaiseFloorSystemConfigurationWriteFailedBeforeAnyRoot(_)
        | MountError::RequestedFloorBelowTheEffectiveFloorWhoseRaiseIsUndecided { .. }
        | MountError::RollbackFloorAboveCeiling { .. }
        | MountError::VersionWithoutFileUnitPointerOfABadImage { .. }
        | MountError::RollbackFloorCeilingNeedsUnreadableValidRootTreeTable { .. }
        | MountError::RollbackFloorCeilingRootRingSlotStillBadAfterOneReread { .. }
        | MountError::InstanceGenerationChangedBeforeAcquisition { .. }
        | MountError::SpaceAdmissionRefusedBeforeAcquisition { .. }
        | MountError::RowPublishAdmissionRefusedBeforeAcquisition { .. }
        | MountError::WarmUpAdmissionRefusedBeforeAcquisition { .. }
        | MountError::PlacementRefusedBeforeAcquisitionMountAdmissionUndecided { .. }
        | MountError::WritableMountRefusedByDevicesWithoutTheSelectedVersion { .. }
        | MountError::WritableDeviceCountBelowTheStripeWidthLowerBound { .. }
        | MountError::DeviceIdentitiesHandedInMoreThanOnce { .. }
        | MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration { .. }
        | MountError::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot { .. }
        | MountError::SequenceNumberPastTheTopOfItsRange(_)
        | MountError::FloorRaiseFailedAfterTheMountsPublishes(_) => None,
    }
}

/// 一块盘两个系统配置槽在崩溃镜像上各直接读一次：自证过、fsid 与本池相同的那几份，设备报读错（`PoolReader::read` 交空）的槽数，
/// 字节自证不过的槽数（[`system_configuration_slot_bytes_fail_self_verification`]：撕裂的轮换、读回全零；布局位不认识的槽不算），
/// 与解得开而 fsid 不是本池的槽数。后两样不进 R2 本身，只进可写挂载读阶段那一判（乙入判不出）：登记
/// `research/prompts/e205-preregistration.md`「乙入判不出」下一行「两种读法共用」写「自证过」要 fsid 与本池相同，别的池的槽按读得出而自证不过算。
struct SystemConfigurationSlotsReadOnTheCrashImage {
    self_verified: Vec<SystemConfiguration>,
    slots_reporting_a_read_error: u64,
    slots_failing_self_verification: u64,
    slots_of_another_pool: u64,
}

/// 一槽读得出的字节自证不过（池级 checker 判槽：magic 不对、整槽校验和不过、节点大小不是格式常量；读回全零落在 magic 不对里）。
/// 与可写挂载读系统配置槽时读的一方报读坏的那几样对得上（核心层 `SystemConfiguration::parse_slot` 拒成 `NotSelfDescribing`、
/// `NodeBytesNotTheFormatConstant` 的槽），这里不调核心层：布局位不认识、格式版本不认、区域数与每区槽数越界的槽是一份完整的系统配置、不算坏。
fn system_configuration_slot_bytes_fail_self_verification(bytes: &[u8]) -> bool {
    matches!(
        singlefs_checker::check_system_configuration_slot(bytes),
        Err(singlefs_checker::Verdict::BadMagic
            | singlefs_checker::Verdict::ChecksumMismatch
            | singlefs_checker::Verdict::NodeBytesNotTheFormatConstant)
    )
}

fn system_configuration_slots_read_on_the_crash_image(
    crash_image: &dyn PoolReader,
    device: DeviceIdentity,
    slot_spacing_in_bytes: u64,
    filesystem_identifier: &[u8; 16],
) -> SystemConfigurationSlotsReadOnTheCrashImage {
    let slot_bytes =
        usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("系统配置槽 4096 字节装得进 usize");
    let mut slots = SystemConfigurationSlotsReadOnTheCrashImage {
        self_verified: Vec::new(),
        slots_reporting_a_read_error: 0,
        slots_failing_self_verification: 0,
        slots_of_another_pool: 0,
    };
    // 迭代上界是一块盘的两个系统配置槽；跨轮携带的只有自证过的那几份与三样计数；没有提前出口。
    for slot_index in 0..SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE {
        let offset = DeviceOffsetInBytes(slot_index * slot_spacing_in_bytes);
        match crash_image.read(device, offset, slot_bytes) {
            None => slots.slots_reporting_a_read_error += 1,
            Some(bytes) => match crate::verification_read::parse_system_configuration(&bytes) {
                Ok(system_configuration) => {
                    if system_configuration.immutable.filesystem_identifier
                        == *filesystem_identifier
                    {
                        slots.self_verified.push(system_configuration);
                    } else {
                        slots.slots_of_another_pool += 1;
                    }
                }
                // 解不开的槽照产品读这一槽的那几关分：别的池的槽在 checker 另判的取值上解不开（区域数、加密类型这几样），同样算别的池的槽。
                Err(()) => {
                    if system_configuration_slot_bytes_fail_self_verification(&bytes) {
                        slots.slots_failing_self_verification += 1;
                    } else {
                        slots.slots_of_another_pool += u64::from(
                            crate::verification_read::system_configuration_slot_bytes_count_as_a_slot_of_another_pool(
                                &bytes,
                                filesystem_identifier,
                            ),
                        );
                    }
                }
            },
        }
    }
    slots
}

/// 乙入判不出（C971（多处读坏时可写挂载写出倒挂的新实例根），`research/prompts/e205-preregistration.md`「乙入判不出」那一行）在崩溃镜像上
/// 独立写的一份，不调实现：设备没报读错的那几块盘里有槽读得出而自证不过（撕裂、读回全零、解得开而 fsid 不是本池的），并进 R2「有槽设备报读错」那一半当读错计，
/// R2 的另一半（没有一块盘两槽都自证过）照旧。只管可写挂载读阶段判据 N-配置 那一判（[`newer_publish_witnessed_on_the_crash_image`]）；
/// 读阶段判完之后系统配置又读不出那一拒照 R2 本身判。判法在崩溃镜像上每槽只读一次：被拒的那次挂载两遍读之间的瞬时变化复算不出，
/// 「重读一次仍是这一形」在这里按读一次是这一形判。
fn some_slot_failing_self_verification_and_no_device_self_verified_both_slots_on_the_crash_image(
    slots_of_every_device: &[SystemConfigurationSlotsReadOnTheCrashImage],
) -> bool {
    let some_slot_failed_self_verification = slots_of_every_device
        .iter()
        .any(|slots| slots.slots_of_another_pool > 0)
        || slots_of_every_device
            .iter()
            .any(|slots| slots.slots_failing_self_verification > 0);
    let some_device_self_verified_both_slots = slots_of_every_device.iter().any(|slots| {
        u64::try_from(slots.self_verified.len()).expect("一块盘的系统配置槽数装得进 u64")
            == SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE
    });
    some_slot_failed_self_verification && !some_device_self_verified_both_slots
}

/// 系统配置见证判不出（R2，D22（单元原子性怎么合成） 已定项 8 射程「改取 R2」）在崩溃镜像上独立写的一份，
/// 不调实现的 `recovery::system_configuration_witness_is_undecidable`：判不出 ⟺ 有槽设备报读错，且没有一块盘两槽都自证过。
/// 撕裂的轮换（读得出、自证不过，设备没报错）不算判不出；一块盘报读错而另一块盘两槽都自证过也不算。
fn system_configuration_witness_undecidable_on_the_crash_image(
    slots_of_every_device: &[SystemConfigurationSlotsReadOnTheCrashImage],
) -> bool {
    let some_slot_reported_a_read_error = slots_of_every_device
        .iter()
        .any(|slots| slots.slots_reporting_a_read_error > 0);
    let some_device_self_verified_both_slots = slots_of_every_device.iter().any(|slots| {
        u64::try_from(slots.self_verified.len()).expect("一块盘的系统配置槽数装得进 u64")
            == SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE
    });
    some_slot_reported_a_read_error && !some_device_self_verified_both_slots
}

/// 某块盘两槽里一份本池自证过的系统配置都没有（C554 乙-配置续 Q1：这块盘不「可见」，D18（块里携带什么信息） 已定项 11）。
/// R2 之下这一形不并进判不出（一块盘两槽都撕裂、另一块盘两槽都自证过时 R2 判得出）；实现在读阶段判完之后的逐盘核与取号里拒它
/// （`MountError::SystemConfigurationUnreadableAfterTheReadStage`，与判据 N-配置 那一拒分开报，C664（挂载拒绝合法复算把两种拒因报成一个））。
fn some_device_without_any_self_verified_system_configuration_slot_on_the_crash_image(
    slots_of_every_device: &[SystemConfigurationSlotsReadOnTheCrashImage],
) -> bool {
    slots_of_every_device
        .iter()
        .any(|slots| slots.self_verified.is_empty())
}

/// 「读阶段判完之后系统配置又读不出」（D23（journal 的角色与格式） 已定项 14：取号那一刻按同一个判不出（R2）判，某块盘一份自证过的系统配置
/// 都没有时也拒）在崩溃镜像上复算：每块盘两槽各直接读一次，判不出（[`system_configuration_witness_undecidable_on_the_crash_image`]）或某块盘
/// 一份本池自证过的都没有（[`some_device_without_any_self_verified_system_configuration_slot_on_the_crash_image`]）⇒ 真；择不到系统配置
/// （`chosen_system_configuration` 是 `None`）也按真。
/// 两遍读之间的瞬时读错在崩溃镜像上复算不出（C664（挂载拒绝合法复算把两种拒因报成一个）），那一形怎么判见
/// [`judge_writable_mount_refusal_of_a_crash_state_of_the_pool`]。
fn system_configuration_unreadable_after_the_read_stage_on_the_crash_image(
    crash_image: &dyn PoolReader,
    chosen_system_configuration: Option<&SystemConfiguration>,
) -> bool {
    let Some(system_configuration) = chosen_system_configuration else {
        return true;
    };
    let immutable = system_configuration.immutable;
    let slot_spacing = u64::from(immutable.sizes.fixed_structure_slot_spacing);
    let slots_of_every_device: Vec<SystemConfigurationSlotsReadOnTheCrashImage> = crash_image
        .device_identities()
        .into_iter()
        .map(|device| {
            system_configuration_slots_read_on_the_crash_image(
                crash_image,
                device,
                slot_spacing,
                &immutable.filesystem_identifier,
            )
        })
        .collect();
    system_configuration_witness_undecidable_on_the_crash_image(&slots_of_every_device)
        || some_device_without_any_self_verified_system_configuration_slot_on_the_crash_image(
            &slots_of_every_device,
        )
}

/// 池级 checker 读崩溃镜像的口子（只交给它数系统配置槽，[`system_configuration_redundancy_report`]）：盘、盘的字节数与读都转给崩溃镜像，
/// 读不出（`PoolReader::read` 交空：设备报读错）的那几块盘记下来。系统配置槽的读法不扫单元区与 journal 环，两样扫描候选交空。
struct CrashImageReadByThePoolChecker<'image> {
    crash_image: &'image dyn PoolReader,
    devices_with_a_read_reporting_an_error: RefCell<BTreeSet<u32>>,
}

impl ImageReader for CrashImageReadByThePoolChecker<'_> {
    fn devices(&self) -> Vec<u32> {
        self.crash_image
            .device_identities()
            .into_iter()
            .map(|device| device.0)
            .collect()
    }
    fn device_bytes(&self, device: u32) -> Option<u64> {
        self.crash_image
            .device_size_in_bytes(DeviceIdentity(device))
    }
    fn read(&self, device: u32, offset: u64, length: usize) -> Option<Vec<u8>> {
        let bytes =
            self.crash_image
                .read(DeviceIdentity(device), DeviceOffsetInBytes(offset), length);
        if bytes.is_none() {
            self.devices_with_a_read_reporting_an_error
                .borrow_mut()
                .insert(device);
        }
        bytes
    }
    fn candidate_unit_slots(&self, _device: u32) -> Option<Vec<u64>> {
        Some(Vec::new())
    }
    fn candidate_journal_slots(&self, _device: u32) -> Option<Vec<u64>> {
        Some(Vec::new())
    }
}

/// 「读阶段判完之后系统配置又读不出」那一拒的判据在崩溃镜像上复算，判槽自证与逐盘数读池级 checker（改法 G4 那一臂用，
/// 「改读池级 checker」，`research/prompts/m3-close-g4-rewrite-r3-main-verification.md`）：
/// 每块盘几槽自证过取 [`system_configuration_redundancy_report`]，不调核心层的 `SystemConfiguration::parse_slot` 与
/// `choose_system_configuration`（`.claude/rules/fs-design.md`「运行时与 checker 必须用不同的算法」）。
/// 为真 ⟺ 某块盘一个自证过的槽都没有，或判不出（R2：有槽设备报读错，且没有一块盘两槽都自证过）。
/// 设备报读错按池级 checker 读系统配置槽时读不出的盘记（全池没有可用的槽 0 时它逐档读槽 1，那几档里读不出的也记）。
/// 与 U6 复算那一份（[`system_configuration_unreadable_after_the_read_stage_on_the_crash_image`]，核心层口径）的口径差：池级 checker 的
/// 「自证过」不分 fsid、自证过而不可择的槽也算，择不到系统配置不另按真；分得开的只有两盘全废与带别的池的槽那几形，
/// 两盘全废由 I-8.5 与 oracle 另判（同一份 那一行理由 ③）。
fn system_configuration_unreadable_after_the_read_stage_judged_by_the_pool_checker_counts_on_the_crash_image(
    crash_image: &dyn PoolReader,
) -> bool {
    let reader = CrashImageReadByThePoolChecker {
        crash_image,
        devices_with_a_read_reporting_an_error: RefCell::new(BTreeSet::new()),
    };
    let report = system_configuration_redundancy_report(&reader);
    let device_count = reader.devices().len();
    let some_device_without_any_self_verified_slot =
        !report.devices_without_a_self_verified_slot.is_empty();
    let no_device_self_verified_both_slots =
        report.devices_with_fewer_than_two_self_verified_slots.len() == device_count;
    let some_slot_reported_a_read_error = !reader
        .devices_with_a_read_reporting_an_error
        .borrow()
        .is_empty();
    some_device_without_any_self_verified_slot
        || (some_slot_reported_a_read_error && no_device_self_verified_both_slots)
}

/// 观察者认下的乙入判不出之拒（[`image_after_the_refusal_into_the_undecidable_witness`]）每认一态加一的计数名：这一态上流接在挂载后面的步（续做整理、删除、截断，写入）都跳过，
/// 观察者的「每个状态都挂过一次」按续做步数的计数加上它等于状态数，跳过的态数由它报出来（各条流的计数行打它）。
pub const WRITABLE_MOUNT_REFUSED_INTO_THE_UNDECIDABLE_WITNESS_COUNTER: &str =
    "writable_mount_refused_into_the_undecidable_witness";

/// 认下这一拒的态上只读挂载打开了恢复落到的那一版时加一的计数名。
pub const READ_ONLY_MOUNT_OPENED_AFTER_THE_REFUSAL_COUNTER: &str =
    "read_only_mount_opened_after_the_refusal";

/// 认下这一拒的态上恢复落到还没有文件版本的一版、只读挂载答树表 0 条（`OpenPoolForReadFailure::TreeTableHasNoEntries`）时加一的计数名。
pub const READ_ONLY_MOUNT_FOUND_NO_FILE_VERSION_AFTER_THE_REFUSAL_COUNTER: &str =
    "read_only_mount_found_no_file_version_after_the_refusal";

/// 认下这一拒的态上被跳过的续做本来还差 `steps_left` 步时加一的计数名：钉着「落到某一版的态数等于续做那么多步的态数」的用例按它补上被拒的那几态。
#[must_use]
pub fn continuation_skipped_on_the_refusal_counter(steps_left: usize) -> String {
    format!("continuation_skipped_on_the_refusal_with_steps_left_{steps_left}")
}

/// 一次在崩溃镜像落成的盘上被拒的可写挂载：拒因、挂载用过的那几块录制设备（盘上的字节就在里面）与录下它的读写的那条录制流。
pub struct WritableMountRefusedOnTheCrashDisks {
    pub refusal: MountError,
    pub devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)>,
    pub stream: SharedStream,
}

/// 拒因是不是乙入判不出：判据 N-配置 那一拒，两遍读的比法都是 [`WitnessedCounterComparison::SomeSlotFailedSelfVerificationAndNoDeviceSelfVerifiedBothSlots`]。
fn refusal_is_into_the_undecidable_witness(refusal: &MountError) -> bool {
    let MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable) = refusal else {
        return false;
    };
    let StillUnreadableAfterOneReread::PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion {
        first_read,
        reread,
    } = **still_unreadable
    else {
        return false;
    };
    [first_read, reread].iter().all(|reading| {
        reading.witness.comparison
            == WitnessedCounterComparison::SomeSlotFailedSelfVerificationAndNoDeviceSelfVerifiedBothSlots
    })
}

/// 观察者独立读四槽算的盘面（[`some_slot_failing_self_verification_and_no_device_self_verified_both_slots_on_the_crash_image`]，不调被测的判法）：
/// 每块盘都没有两槽自证过、有槽读得出而自证不过。
fn crash_disks_hold_the_shape_of_the_undecidable_witness(
    crash_disks: &MemoryPool,
    filesystem_identifier: &[u8; 16],
    geometry: &FixedGeometry,
) -> bool {
    let slot_spacing_in_bytes = u64::from(geometry.fixed_structure_slot_spacing);
    let slots_of_every_device: Vec<SystemConfigurationSlotsReadOnTheCrashImage> = crash_disks
        .device_identities()
        .into_iter()
        .map(|device| {
            system_configuration_slots_read_on_the_crash_image(
                crash_disks,
                device,
                slot_spacing_in_bytes,
                filesystem_identifier,
            )
        })
        .collect();
    some_slot_failing_self_verification_and_no_device_self_verified_both_slots_on_the_crash_image(
        &slots_of_every_device,
    )
}

/// 崩溃枚举用例里挂载观察者共用的一判（D22（单元原子性怎么合成） 已定项 8 射程「乙入判不出」与 C596（见证判不出的系统配置盘面只剩只读挂载） 的冲突格）：
/// 可写挂载在 `crash_disks` 落成的盘上被拒时，只认这一种拒——拒因恰是比法
/// [`WitnessedCounterComparison::SomeSlotFailedSelfVerificationAndNoDeviceSelfVerifiedBothSlots`]、盘面确是这一形（[`crash_disks_hold_the_shape_of_the_undecidable_witness`]），
/// 同一组录制设备上只读挂载做成（打开恢复落到的那一版；恢复落到还没有文件的一版的只读挂载答树表 0 条，那是这一版的常规答复，分开计数），挂载没发一次写、盘上逐字节不变。
/// 落到第 0 代的那一形，离线出路断言第 0 代之后可写挂载做成（`system_configuration_repair::rewrite_system_configuration_slots_for_the_asserted_version`）；这一判不走离线出路。
/// 认下了给 [`WRITABLE_MOUNT_REFUSED_INTO_THE_UNDECIDABLE_WITNESS_COUNTER`] 加一，交回没改过的盘：判段照样在它上面判池级 checker，流接在挂载后面的步由调用的观察者跳过。
/// 只读挂载的读录在同一条录制流上、进读集见证：去重时它的结论只复用给这几段字节相同的状态。
///
/// # Panics
/// 拒因不是这一种，盘面不是这一形，只读挂载做不成或落到的不是恢复落到的那一版，或挂载写过盘：观察者判红。
pub fn image_after_the_refusal_into_the_undecidable_witness(
    refused: &WritableMountRefusedOnTheCrashDisks,
    crash_disks: &MemoryPool,
    report: &RecoveryReport,
    filesystem_identifier: &[u8; 16],
    geometry: &FixedGeometry,
    counter_when_the_pool_checker_is_clean: &str,
    counts: &mut CrashPointReplayObserverCounts,
) -> ImageAfterTheObserver {
    if !refusal_is_into_the_undecidable_witness(&refused.refusal) {
        panic!(
            "可写挂载被拒，拒因不是乙入判不出（两遍读的比法都是 SomeSlotFailedSelfVerificationAndNoDeviceSelfVerifiedBothSlots）：{:?}",
            refused.refusal
        );
    }
    if !crash_disks_hold_the_shape_of_the_undecidable_witness(
        crash_disks,
        filesystem_identifier,
        geometry,
    ) {
        panic!(
            "可写挂载报乙入判不出，盘面不是这一形（每块盘都没有两槽自证过、有槽读得出而自证不过）：{:?}",
            refused.refusal
        );
    }
    match (mount_read_only(&refused.devices), &report.outcome) {
        (Ok(read_only), _) => {
            assert_eq!(
                Some((
                    read_only.effective_root.instance,
                    read_only.effective_root.checkpoint_txg
                )),
                report.effective_root,
                "可写挂载被拒的状态上只读挂载落到的就是恢复落到的那一版"
            );
            counts.add(READ_ONLY_MOUNT_OPENED_AFTER_THE_REFUSAL_COUNTER, 1);
        }
        (
            Err(MountReadOnlyFailure::Open(OpenPoolForReadFailure::TreeTableHasNoEntries)),
            RecoveryOutcome::NoFile { .. },
        ) => {
            counts.add(
                READ_ONLY_MOUNT_FOUND_NO_FILE_VERSION_AFTER_THE_REFUSAL_COUNTER,
                1,
            );
        }
        (Err(failure), outcome) => {
            panic!("可写挂载被拒的状态上只读挂载做不成（恢复读出 {outcome:?}）：{failure:?}")
        }
    }
    let (writes_on_the_crash_image, _segments_of_the_refused_mount) =
        writes_and_segments(&refused.stream.retained_operations(), geometry);
    assert!(
        writes_on_the_crash_image.is_empty(),
        "被拒的可写挂载发了写：{} 次",
        writes_on_the_crash_image.len()
    );
    let disks_after_the_observer = MemoryPool {
        devices: refused
            .devices
            .iter()
            .map(|(identity, device)| (*identity, device.wrapped_device().image.clone()))
            .collect(),
        device_size_in_bytes: crash_disks.device_size_in_bytes,
    };
    assert!(
        disks_after_the_observer == *crash_disks,
        "被拒的可写挂载之后盘上逐字节不变"
    );
    counts.add(
        WRITABLE_MOUNT_REFUSED_INTO_THE_UNDECIDABLE_WITNESS_COUNTER,
        1,
    );
    ImageAfterTheObserver {
        writes_on_the_crash_image,
        disks_after_the_observer,
        ranges_read_on_the_crash_image: ReadRangeOnTheCrashImage::of_the_recorded_read_ranges(
            &refused.stream.read_ranges(),
        ),
        invariants_judged: InvariantsJudgedOnTheImageAfterTheObserver::EveryImplementedInvariant,
        counter_when_the_pool_checker_is_clean: counter_when_the_pool_checker_is_clean.to_string(),
    }
}

/// 崩溃放量那一路的计数行用：每个观察者认下乙入判不出之拒、跳过后面那几步的态数（没有这样的态的观察者不列）。
#[must_use]
pub fn writable_mount_refusals_into_the_undecidable_witness_of_each_observer(
    observing: &ObservingOfTheRun,
) -> BTreeMap<String, u64> {
    observing
        .counts_of_each_observer
        .iter()
        .filter_map(|(observer, counts)| {
            counts
                .get(WRITABLE_MOUNT_REFUSED_INTO_THE_UNDECIDABLE_WITNESS_COUNTER)
                .map(|refused_states| (observer.clone(), *refused_states))
        })
        .collect()
}

/// 计数行里不带观察者计数的那几条流打这一行：这一趟认下乙入判不出之拒、跳过后面那几步的态数，不让跳过的态静默少判。
pub fn print_the_writable_mount_refusals_into_the_undecidable_witness(
    flow_name: &str,
    tally: &CrashPointReplayTally,
) {
    println!(
        "WRITABLE_MOUNT_REFUSALS_INTO_THE_UNDECIDABLE_WITNESS flow={flow_name} states={} states_skipping_the_steps_after_the_mount={}",
        tally.states,
        tally
            .observer_counts
            .count_named(WRITABLE_MOUNT_REFUSED_INTO_THE_UNDECIDABLE_WITNESS_COUNTER)
    );
}

/// 判据 N-配置（D23 已定项 14「可写挂载读到的更新状态读不出」那一段）在崩溃镜像上自己复算一遍，不经实现的 `mount::newer_publish_witness`：
/// 系统配置见证判不出——读得出而自证不过的槽当读错计时判不出（乙入判不出，
/// [`some_slot_failing_self_verification_and_no_device_self_verified_both_slots_on_the_crash_image`]），或 R2 本身判不出
/// （[`system_configuration_witness_undecidable_on_the_crash_image`]）——⇒ 按真；否则全部自证过的槽的 journal tail
/// 取最大为 c_见证（撕裂的槽跳过），0 ⇒ 假；所选那一版那次发布带末条标志的记录读得出 ⇒ c_见证 > 它的计数器；读不出而计数器等于 c_见证 的记录
/// 读得出 ⇒ 它的 (实例代号, checkpoint_txg) > 所选那一版；两条都读不出按真。
/// `chosen_system_configuration` 是判官择到的系统配置，择不到（`None`）按真。
/// `selected_version` 是 R1 落到的那一版（与可写挂载读阶段同一份盘、同一套择根与重放）；没有时判据无从复算，交回假。
fn newer_publish_witnessed_on_the_crash_image(
    crash_image: &dyn PoolReader,
    chosen_system_configuration: Option<&SystemConfiguration>,
    selected_version: Option<(InstanceGeneration, CheckpointTxg)>,
) -> bool {
    let Some(system_configuration) = chosen_system_configuration else {
        return true;
    };
    let immutable = system_configuration.immutable;
    let slot_spacing = u64::from(immutable.sizes.fixed_structure_slot_spacing);
    let slots_of_every_device: Vec<SystemConfigurationSlotsReadOnTheCrashImage> = crash_image
        .device_identities()
        .into_iter()
        .map(|device| {
            system_configuration_slots_read_on_the_crash_image(
                crash_image,
                device,
                slot_spacing,
                &immutable.filesystem_identifier,
            )
        })
        .collect();
    if some_slot_failing_self_verification_and_no_device_self_verified_both_slots_on_the_crash_image(
        &slots_of_every_device,
    ) {
        return true;
    }
    if system_configuration_witness_undecidable_on_the_crash_image(&slots_of_every_device) {
        return true;
    }
    let witnessed_journal_counter = slots_of_every_device
        .iter()
        .flat_map(|slots| slots.self_verified.iter())
        .map(|slot| slot.quantities.journal_tail)
        .max()
        .unwrap_or(0);
    witnessed_counter_holds_against_the_selected_version(
        crash_image,
        system_configuration,
        witnessed_journal_counter,
        selected_version,
    )
    .is_some()
}

/// 判据 N-配置 在 c_见证 与所选那一版比出来为真的那一支（D23（journal 的角色与格式） 已定项 14「可写挂载读到的更新状态读不出」那一段的后半）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WitnessedCounterBranchThatHolds {
    /// 所选那一版那次发布带末条标志的记录读得出，c_见证 大于它的计数器。
    WitnessedCounterAboveTheLastRecordOfTheSelectedVersion,
    /// 所选那一版的末条读不出，计数器等于 c_见证 的记录读得出、它的 (实例代号, checkpoint_txg) 大于所选那一版。
    RecordAtTheWitnessedCounterNewerThanTheSelectedVersion,
    /// 两条都读不出，按真。
    NeitherTheLastRecordOfTheSelectedVersionNorTheRecordAtTheWitnessedCounterReadable,
}

/// c_见证 已经取好之后的那一半：0 ⇒ 假；所选那一版那次发布带末条标志的记录读得出 ⇒ c_见证 > 它的计数器；读不出而计数器等于 c_见证 的记录读得出 ⇒
/// 它的 (实例代号, checkpoint_txg) > 所选那一版；两条都读不出按真。为真交回是哪一支，为假（或没有所选那一版、判据无从复算）交回 `None`。
fn witnessed_counter_holds_against_the_selected_version(
    crash_image: &dyn PoolReader,
    system_configuration: &SystemConfiguration,
    witnessed_journal_counter: u64,
    selected_version: Option<(InstanceGeneration, CheckpointTxg)>,
) -> Option<WitnessedCounterBranchThatHolds> {
    if witnessed_journal_counter == 0 {
        return None;
    }
    let selected_version = selected_version?;
    let records = scan_journal(crash_image, system_configuration);
    let last_record_counter_of_the_selected_version = records
        .values()
        .filter(|record| {
            (record.instance, record.checkpoint_txg) == selected_version
                && record.place_in_publish == JournalRecordPlaceInPublish::LastRecordOfThePublish
        })
        .map(|record| record.counter)
        .max();
    if let Some(last_record_counter) = last_record_counter_of_the_selected_version {
        return (witnessed_journal_counter > last_record_counter).then_some(
            WitnessedCounterBranchThatHolds::WitnessedCounterAboveTheLastRecordOfTheSelectedVersion,
        );
    }
    let record_at_the_witnessed_counter = records
        .values()
        .filter(|record| record.counter == witnessed_journal_counter)
        .map(|record| (record.instance, record.checkpoint_txg))
        .max();
    match record_at_the_witnessed_counter {
        Some(record) => (record > selected_version).then_some(
            WitnessedCounterBranchThatHolds::RecordAtTheWitnessedCounterNewerThanTheSelectedVersion,
        ),
        None => Some(
            WitnessedCounterBranchThatHolds::NeitherTheLastRecordOfTheSelectedVersionNorTheRecordAtTheWitnessedCounterReadable,
        ),
    }
}

/// C670（无读故障的崩溃态上可写挂载见证判据为真没有判官）：判据 N-配置 在没注入读故障的崩溃镜像上复算，为真交回是哪一支。
/// 没有读故障时设备不报读错，见证读判得出（R2，D22（单元原子性怎么合成） 已定项 8 射程：判不出 ⟺ 有槽设备报读错且没有一块盘两槽都自证过；
/// 撕裂的系统配置轮换不算判不出）：c_见证 取全部自证过、fsid 与本池相同的槽 journal tail 的最大值，自证不过的槽（撕裂的、没写过的）跳过——
/// 与 [`newer_publish_witnessed_on_the_crash_image`] 改取 R2 之后在撕裂的轮换上同一个取法（某块盘两槽都撕裂、一份自证过的都没有的那一形
/// 两边都照常比：实现拒它报的是另一个拒因，C664（挂载拒绝合法复算把两种拒因报成一个））。一份自证过的系统配置都择不到时 c_见证 为 0，交回 `None`。
fn witnessed_counter_holds_on_a_crash_image_without_read_faults(
    crash_image: &dyn PoolReader,
    chosen_system_configuration: Option<&SystemConfiguration>,
    selected_version: Option<(InstanceGeneration, CheckpointTxg)>,
) -> Option<WitnessedCounterBranchThatHolds> {
    let system_configuration = chosen_system_configuration?;
    let slot_spacing = u64::from(
        system_configuration
            .immutable
            .sizes
            .fixed_structure_slot_spacing,
    );
    let witnessed_journal_counter = crash_image
        .device_identities()
        .into_iter()
        .flat_map(|device| {
            verified_system_configuration_slots(
                crash_image,
                device,
                slot_spacing,
                &system_configuration.immutable.filesystem_identifier,
            )
        })
        .map(|slot| slot.quantities.journal_tail)
        .max()
        .unwrap_or(0);
    witnessed_counter_holds_against_the_selected_version(
        crash_image,
        system_configuration,
        witnessed_journal_counter,
        selected_version,
    )
}

/// 「重建分配器判抛弃用的最新那条根的实例表读不出」在崩溃镜像上复算：根环里择到的最新那条根指着的实例表沿链读不出、解不开（择不到根也算）。
fn instance_table_of_the_newest_root_unreadable_on_the_crash_image(
    crash_image: &dyn PoolReader,
    chosen_system_configuration: Option<&SystemConfiguration>,
) -> bool {
    let Some(system_configuration) = chosen_system_configuration else {
        return true;
    };
    choose_root(crash_image, system_configuration)
        .is_none_or(|newest| instance_table_of_root(crash_image, &newest).is_none())
}

/// 「重建分配器读根环整环重读一次、两遍逐槽并起来仍一条不被抛弃的自证有效根都没有」（Q3，并两遍照 R1）在崩溃镜像上复算：
/// 按最新那条根（`choose_root`）指着的实例表，根环里一条不被抛弃的自证有效根都没有。并两遍之后判据照旧只读一遍：崩溃镜像上没有瞬时读错，
/// 两遍读出的逐槽相同，并起来就是那一遍（任一遍自证过的根留着，两遍都没有才算没有）。被抛弃照 D23（journal 的角色与格式） 已定项 14 候选集那一格在这里另写一份：表里有那条根的实例的行
/// (i, Ti, Wi) 且它的 txg > Ti（不调实现的 `recovery::root_is_abandoned_by_the_instance_table`）。择不到系统配置、择不到根都按真；
/// 最新那条根的实例表读不出时没有「被抛弃」可判，不按表滤（读得出的根都算有效，与 F 生效值那一处的取法同向），择得到根就为假。
/// 没有读故障时最新那条根按它自己指着的表恒不被抛弃（表里的行只记更早的实例：写行只给比要取的号小的实例写，`mount.rs` 的
/// `instance_rows_to_write`），所以择得到根的崩溃镜像上这条判据为假；「按表判被抛弃」那半句在合法盘面上不改判据的值，等价性留档在
/// `writable_mount_refusal_for_no_valid_root_in_the_rebuilt_allocators_root_ring.rs`
/// 的 `a_refusal_for_no_valid_root_on_a_crash_image_holding_an_abandoned_root_is_illegal_because_the_newest_root_is_never_abandoned_by_its_own_table`。
fn no_valid_root_in_the_root_ring_under_the_newest_roots_instance_table_on_the_crash_image(
    crash_image: &dyn PoolReader,
    chosen_system_configuration: Option<&SystemConfiguration>,
) -> bool {
    let Some(system_configuration) = chosen_system_configuration else {
        return true;
    };
    let Some(newest) = choose_root(crash_image, system_configuration) else {
        return true;
    };
    let Some(table_of_the_newest_root) = instance_table_of_root(crash_image, &newest) else {
        return false;
    };
    let immutable = &system_configuration.immutable;
    !readable_roots(
        crash_image,
        &immutable.region_devices,
        &immutable.sizes,
        &immutable.filesystem_identifier,
    )
    .iter()
    .any(|root| {
        !table_of_the_newest_root
            .rows
            .iter()
            .any(|row| row.instance == root.instance && root.checkpoint_txg > row.selected_root_txg)
    })
}

/// 「记录有缺口时拒可写」（D23（journal 的角色与格式） 已定项 14，F1 乙）在崩溃镜像上复算，不调实现的
/// `mount::publishes_above_the_water_mark_missing_records_whose_named_units_do_not_verify`：水位（`selected_version`，R1 落到的那一版）之上
/// 按 (实例代号, checkpoint_txg) 分组的每一次发布，读得出的记录的本次发布内序号排好之后不是从 1 起逐个加一、或其中没有一条带末条标志，
/// 而它们点名的单元里有一个两份副本都验不过（[`named_unit_has_a_copy_that_verifies_on_the_crash_image`] 为假）——有这样一次发布即为真。
/// 崩溃镜像上没有瞬时读错，重读一次读出的与这一遍相同，判据只算一遍。择不到系统配置按真（与这一族别的复算同向）；
/// 没有所选那一版时水位无从取，交回假。
fn publish_above_the_water_mark_missing_records_whose_named_units_do_not_verify_on_the_crash_image(
    crash_image: &dyn PoolReader,
    chosen_system_configuration: Option<&SystemConfiguration>,
    selected_version: Option<(InstanceGeneration, CheckpointTxg)>,
) -> bool {
    let Some(system_configuration) = chosen_system_configuration else {
        return true;
    };
    let Some(selected_version) = selected_version else {
        return false;
    };
    let mut records_of_each_publish: BTreeMap<
        (InstanceGeneration, CheckpointTxg),
        Vec<JournalRecord>,
    > = BTreeMap::new();
    for record in scan_journal(crash_image, system_configuration).into_values() {
        let publish = (record.instance, record.checkpoint_txg);
        if publish > selected_version {
            records_of_each_publish
                .entry(publish)
                .or_default()
                .push(record);
        }
    }
    records_of_each_publish
        .values()
        .any(|records_of_the_publish| {
            let mut ordinals: Vec<u32> = records_of_the_publish
                .iter()
                .map(|record| record.ordinal_within_publish.0)
                .collect();
            ordinals.sort_unstable();
            let ordinals_run_from_one_without_a_gap = ordinals
                .iter()
                .zip(1u32..)
                .all(|(ordinal, expected_ordinal)| *ordinal == expected_ordinal);
            let carries_the_last_record_of_the_publish =
                records_of_the_publish.iter().any(|record| {
                    record.place_in_publish == JournalRecordPlaceInPublish::LastRecordOfThePublish
                });
            let some_named_unit_fails_on_both_copies = records_of_the_publish
                .iter()
                .flat_map(|record| &record.named)
                .any(|named| {
                    !named_unit_has_a_copy_that_verifies_on_the_crash_image(crash_image, named)
                });
            !(ordinals_run_from_one_without_a_gap && carries_the_last_record_of_the_publish)
                && some_named_unit_fails_on_both_copies
        })
}

/// 一条记录点名的一个单元在崩溃镜像上至少有一份副本验得过：某一条位置条目指的那一份读得出，按单元种类的长度（数据单元、打包单元 32 KiB，
/// 索引节点 16 KiB）读回的整单元 CRC-32C 等于条目里的。种类不认识的点名项（坏盘上的记录）哪一份都不算验得过。
fn named_unit_has_a_copy_that_verifies_on_the_crash_image(
    crash_image: &dyn PoolReader,
    named: &NamedUnit,
) -> bool {
    let unit_length_in_bytes = if named.unit_class == UNIT_CLASS_INDEX_NODE {
        NODE_BYTES
    } else if named.unit_class == UNIT_CLASS_DATA || named.unit_class == UNIT_CLASS_PACKED {
        DATA_UNIT_BYTES
    } else {
        return false;
    };
    let unit_bytes =
        usize::try_from(unit_length_in_bytes).expect("单元字节数 16384 或 32768 装得进 usize");
    named.locations.iter().any(|location| {
        crash_image
            .read(
                location.device,
                location.slot.to_device_offset(),
                unit_bytes,
            )
            .is_some_and(|bytes| crc32_castagnoli(&bytes) == location.unit_checksum)
    })
}

/// 「C 读一次没施加的发布的账读不出、读得出的记录不齐、从中央映射也现算不成」（`C_MAPPING` 收窄）那一拒在崩溃镜像上复算，不调实现的
/// `mount::placements_in_the_accounts_of_unapplied_publishes` 与 `mount::placements_in_the_accounts_of_newer_readable_roots`。
/// 判据：三样都成立即为真：
/// 1. 那一拒点名的 `unapplied_publish` 是水位（`selected_version`，R1 落到的那一版，按实例代号为主比）之上没施加的发布——崩溃镜像上读得出它的记录；
///    或是根环里比所选那一版新（txg 为主、实例代号破平局）、按最新那条根的实例表有效、读得出的根。被抛弃照 D23（journal 的角色与格式） 已定项 14
///    候选集那一格另写一份（表里有那条根的实例的行 (i, Ti, Wi) 且它的 txg > Ti）；最新那条根的实例表读不出时不按表滤，与
///    [`no_valid_root_in_the_root_ring_under_the_newest_roots_instance_table_on_the_crash_image`] 同一取法。
/// 2. 那次发布在崩溃镜像上读得出的记录不齐：本次发布内序号排好之后不是从 1 起逐个加一，或没有一条带末条标志（一条都读不出也算没有末条）。
/// 3. 它的账在崩溃镜像上读不出：按新根段里的树表指针找到的账根两份副本都验不过、或账根读不到（[`account_unreadable_on_the_crash_image`]）——
///    读得出的每条记录的新根段，与第 1 条里那条较新的根的根记录，各找一次，每一份都读不出才算。
///
/// 这一格只判必要条件：实现拒之前还要「从中央映射现算不成」，那一半崩溃镜像上不复算——复算为真不说明实现非拒不可，只说拒的理由在镜像上站得住。
/// 账只看到账根那一个节点：实现读账还读分配记录树根以下的节点，只有那几样读不出时实现照拒、这里复算为假（判不合法）。
/// 崩溃镜像上没有瞬时读错，重读一次读出的与这一遍相同，判据只算一遍。择不到系统配置按真（与这一族别的复算同向）；没有所选那一版时水位无从取，交回假。
fn account_of_an_unapplied_publish_with_incomplete_readable_records_on_the_crash_image(
    crash_image: &dyn PoolReader,
    chosen_system_configuration: Option<&SystemConfiguration>,
    selected_version: Option<(InstanceGeneration, CheckpointTxg)>,
    unapplied_publish: (InstanceGeneration, CheckpointTxg),
) -> bool {
    let Some(system_configuration) = chosen_system_configuration else {
        return true;
    };
    let Some(selected_version) = selected_version else {
        return false;
    };
    let readable_records_of_the_publish: Vec<JournalRecord> =
        scan_journal(crash_image, system_configuration)
            .into_values()
            .filter(|record| (record.instance, record.checkpoint_txg) == unapplied_publish)
            .collect();
    let is_an_unapplied_publish_above_the_water_mark =
        unapplied_publish > selected_version && !readable_records_of_the_publish.is_empty();
    let newer_valid_readable_root = newer_valid_readable_root_on_the_crash_image(
        crash_image,
        system_configuration,
        selected_version,
        unapplied_publish,
    );
    if !is_an_unapplied_publish_above_the_water_mark && newer_valid_readable_root.is_none() {
        return false;
    }
    let mut ordinals: Vec<u32> = readable_records_of_the_publish
        .iter()
        .map(|record| record.ordinal_within_publish.0)
        .collect();
    ordinals.sort_unstable();
    let ordinals_run_from_one_without_a_gap = ordinals
        .iter()
        .zip(1u32..)
        .all(|(ordinal, expected_ordinal)| *ordinal == expected_ordinal);
    let carries_the_last_record_of_the_publish =
        readable_records_of_the_publish.iter().any(|record| {
            record.place_in_publish == JournalRecordPlaceInPublish::LastRecordOfThePublish
        });
    if ordinals_run_from_one_without_a_gap && carries_the_last_record_of_the_publish {
        return false;
    }
    readable_records_of_the_publish
        .iter()
        .map(|record| {
            (
                record.new_tree_table,
                record.new_allocation_record_tree_root,
            )
        })
        .chain(
            newer_valid_readable_root
                .iter()
                .map(|root| (root.tree_table, root.allocation_record_tree_root)),
        )
        .all(
            |(tree_table, allocation_record_tree_root_in_the_new_root_segment)| {
                account_unreadable_on_the_crash_image(
                    crash_image,
                    &tree_table,
                    &allocation_record_tree_root_in_the_new_root_segment,
                )
            },
        )
}

/// 一版的账在崩溃镜像上读不出：账根两份副本都验不过（[`node_has_a_copy_that_verifies_on_the_crash_image`] 为假），或连账根在哪都读不到。
/// 账根按那一版的树表找（`mount::placements_referenced_by_root` 读账的同一条路，这里另写一份、不调它）：树表两份都验不过、解不开 ⇒
/// 读不到账根，算读不出；树表 0 条（没写过文件的一版）⇒ 账根是新根段里那条分配记录树根（空指针算读得出：mkfs 的第 0 代与照抄它的暖机根没有这棵树）；
/// 树表有条目 ⇒ 账根是 `TREE_KIND_ALLOCATION` 那一条的根指针（没有那一条算读得出：账里一条记录都没有）。
/// 主 agent 定的判据写的是「新根段里的分配记录树根」；带文件的一版，根记录与新根段里那一条恒是空指针（它的分配记录树住树表条目，
/// D8（核心索引结构） 已定项 8；`transaction.rs` 发布带文件的一版时那一项写 `NodePointer::empty_root()`），不是它的账——
/// 只看那一条，带文件的一版永远判不成读不出。这里按「新根段里的树表指针找到的账根」读，交主 agent 定（报告）。
/// 只看账根那一个节点：账根以下的节点读不出时实现照拒、这里判读得出。
fn account_unreadable_on_the_crash_image(
    crash_image: &dyn PoolReader,
    tree_table: &NodePointer,
    allocation_record_tree_root_in_the_new_root_segment: &NodePointer,
) -> bool {
    let node_bytes = usize::try_from(NODE_BYTES).expect("节点字节数 16384 装得进 usize");
    let Some(tree_table_bytes) = tree_table.locations.iter().find_map(|location| {
        crash_image
            .read(
                location.device,
                location.slot.to_device_offset(),
                node_bytes,
            )
            .filter(|bytes| crc32_castagnoli(bytes) == location.unit_checksum)
    }) else {
        return true;
    };
    let Ok(tree_table_node) = parse_index_node(&tree_table_bytes) else {
        return true;
    };
    let account_root = if tree_table_node.entries.is_empty() {
        *allocation_record_tree_root_in_the_new_root_segment
    } else {
        match TreeTableEntryRootPointersByKind::of_the_entries(
            &tree_table_node.entries,
            &[TREE_KIND_ALLOCATION],
        )
        .root_pointer_of_each_kind
        .get(&TREE_KIND_ALLOCATION)
        {
            Some(root_pointer_bytes) => {
                match crate::verification_read::node_pointer(root_pointer_bytes) {
                    Some(pointer) => pointer,
                    None => return true,
                }
            }
            None => return false,
        }
    };
    account_root != NodePointer::empty_root()
        && !node_has_a_copy_that_verifies_on_the_crash_image(crash_image, &account_root)
}

/// 根环里那条 (实例代号, checkpoint_txg) 等于 `publish` 的根，它比所选那一版新（择根的序：txg 为主、实例代号破平局）、读得出、按最新那条根
/// （`choose_root`）的实例表不被抛弃（表里有它的实例的行 (i, Ti, Wi) 且它的 txg > Ti 才算被抛弃，不调实现的
/// `recovery::root_is_abandoned_by_the_instance_table`；那张表读不出时不按表滤）：交回那条根，没有交 `None`。
fn newer_valid_readable_root_on_the_crash_image(
    crash_image: &dyn PoolReader,
    system_configuration: &SystemConfiguration,
    selected_version: (InstanceGeneration, CheckpointTxg),
    publish: (InstanceGeneration, CheckpointTxg),
) -> Option<RootRecord> {
    let table_of_the_newest_root = choose_root(crash_image, system_configuration)
        .and_then(|newest| instance_table_of_root(crash_image, &newest));
    let immutable = &system_configuration.immutable;
    readable_roots(
        crash_image,
        &immutable.region_devices,
        &immutable.sizes,
        &immutable.filesystem_identifier,
    )
    .into_iter()
    .find(|root| {
        let is_abandoned = table_of_the_newest_root.as_ref().is_some_and(|table| {
            table.rows.iter().any(|row| {
                row.instance == root.instance && root.checkpoint_txg > row.selected_root_txg
            })
        });
        (root.instance, root.checkpoint_txg) == publish
            && (root.checkpoint_txg, root.instance) > (selected_version.1, selected_version.0)
            && !is_abandoned
    })
}

/// 一条码 2 / 码 3 指针指的节点在崩溃镜像上至少有一份副本验得过：某一条位置条目指的那一份读得出，16 KiB 读回的整单元 CRC-32C 等于条目里的。
fn node_has_a_copy_that_verifies_on_the_crash_image(
    crash_image: &dyn PoolReader,
    pointer: &NodePointer,
) -> bool {
    let node_bytes = usize::try_from(NODE_BYTES).expect("节点字节数 16384 装得进 usize");
    pointer.locations.iter().any(|location| {
        crash_image
            .read(
                location.device,
                location.slot.to_device_offset(),
                node_bytes,
            )
            .is_some_and(|bytes| crc32_castagnoli(&bytes) == location.unit_checksum)
    })
}

/// 拒之后盘上逐字节同不同（调用方比拒之前与拒之后的整份镜像）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiskAfterTheRefusal {
    ByteForByteUnchanged,
    Changed,
}

/// 崩溃镜像上只读恢复读出的，在不在允许集合里（调用方拿理想模型判的那一截的结论交进来）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReadOnlyReadBackOfTheCrashImage {
    InsideTheAllowedSet,
    OutsideTheAllowedSet,
}

/// 崩溃态注没注入过读故障（D13（验证路线） 已定项 1：挂载透明的射程只管不带故障的状态；C670（无读故障的崩溃态上可写挂载见证判据为真没有判官））。
/// 没注入：崩溃镜像读得出的就是盘上的字节，造出这份盘面的历史里也没有一步在读故障下发过写。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReadFaultsInTheCrashState {
    NoneInjected,
    Injected,
}

/// 被拒的这次可写挂载自己在不在读故障下跑（判决 `research/prompts/m3-formal-code-b26-r1-main-verification.md` 第二节 J2
/// 「注入读故障的态上『新拒因复算为假不算不合法』（C3 / J2-a）」那一行，改法 G3）。与 [`ReadFaultsInTheCrashState`] 是两件事：
/// 那一样说造出崩溃镜像的历史里有没有读故障步，这一样说被拒的那次挂载读盘时有没有故障层——崩溃注入第二、三截那次挂载起在崩溃镜像上、
/// 盘是不带故障层的内存盘，历史里注入过读故障也不会在这次挂载里出瞬时读错。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReadFaultsUnderTheRefusedMount {
    /// 被拒的那次挂载读的就是崩溃镜像上的字节、不带故障层（崩溃注入第二、三截，E171 装置）：瞬时读错不会发生，
    /// 拒的判据在崩溃镜像上复算为假就是拒错了。
    TheRefusedMountReadTheCrashImageWithoutFaults,
    /// 被拒的那次挂载的读经故障层、读错可以是瞬时的：两遍读之间的瞬时读错在崩溃镜像上复算不出
    /// （C664（挂载拒绝合法复算把两种拒因报成一个）判别力自证里取号那一刻瞬时读错的态）。
    TheRefusedMountReadThroughInjectedFaults,
}

impl ReadFaultsUnderTheRefusedMount {
    /// 「读阶段判完之后系统配置又读不出」与「重建分配器读根环整环重读一次仍一条有效根都没有」（Q3）那两拒的判据在崩溃镜像上复算为假，
    /// 能不能是被拒那次挂载里一次瞬时读错（复算不出）。
    fn can_hide_a_transient_read_error_from_the_crash_image(self) -> bool {
        match self {
            Self::TheRefusedMountReadTheCrashImageWithoutFaults => false,
            Self::TheRefusedMountReadThroughInjectedFaults => true,
        }
    }
}

/// 单子上的理由，合法还差哪一条。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RefusalConditionNotMet {
    /// 那一条判据在崩溃镜像上复算为假：拒的理由是错的。
    CriterionFalseWhenRecomputedOnTheCrashImage,
    DiskChangedByTheRefusedMount,
    ReadOnlyReadBackOutsideTheAllowedSet,
    /// C670：崩溃态没注入读故障，判据 N-配置 在崩溃镜像上（撕裂的系统配置槽跳过）复算仍为真——正确实现的可写挂载在这份盘面上永远拒，
    /// 一次普通断电之后池只剩只读挂载；这样的崩溃态不许出现，拒是对的、造出这份盘面的那一段不对。带的是判据为真的那一支。
    WitnessCriterionTrueOnACrashImageWithoutReadFaults(WitnessedCounterBranchThatHolds),
    /// 改法 G4（判决 `research/prompts/m3-formal-code-b26-r1-main-verification.md` 第二节 J2「新拒因少一条 C670 同形的判法（J2-b）」那一行）：
    /// 崩溃态没注入读故障，拒因是「读阶段判完之后系统配置又读不出」，那一拒的判据在崩溃镜像上复算为真（按池级 checker 的逐盘数，某块盘
    /// 一个自证过的系统配置槽都没有，或见证判不出；`system_configuration_unreadable_after_the_read_stage_judged_by_the_pool_checker_counts_on_the_crash_image`）——mkfs 返回之后没有读故障的历史留不下这份盘面，正确实现在它上面永远拒可写；与 C670 那一条同形，拒是对的、造出这份盘面的那一段不对。
    /// 射程只到 mkfs 返回之后的崩溃态（D13（验证路线） 已定项 6）：mkfs 逐盘两槽在同一个循环里写、之后才逐盘屏障，mkfs 返回之前的崩溃态留得下这份盘面，
    /// 不归这一条判；崩溃注入第二、三截的候选段都取在 mkfs 之后。
    SystemConfigurationUnreadableAfterTheReadStageCriterionTrueOnACrashImageWithoutReadFaults,
    /// 与改法 G4 同形（Q3，判决 `research/prompts/m3-batch10-code-r1-main-verification.md` 第六节）：崩溃态没注入读故障，
    /// 拒因是「重建分配器读根环整环重读一次、两遍并起来仍一条不被抛弃的自证有效根都没有」，那一拒的判据在崩溃镜像上复算为真（按最新那条根的实例表，
    /// 环里一条不被抛弃的自证有效根都没有）——没有读故障的历史留不下这份盘面（读阶段择得到根、它按自己的实例表不被抛弃），正确实现在它上面
    /// 永远拒可写；拒是对的、造出这份盘面的那一段不对。
    NoValidRootInTheRootRingCriterionTrueOnACrashImageWithoutReadFaults,
}

/// 可写挂载被拒，拒得合不合法（拒绝合法 ⇔ 理由在 D23 已定项 14 那张单子上、那一条判据在崩溃镜像上复算为真、
/// 盘上逐字节不变、只读读出在允许集合里）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WritableMountRefusalVerdict {
    Legal(RefusalReasonOnTheLegalRefusalList),
    Illegal {
        reason: RefusalReasonOnTheLegalRefusalList,
        conditions_not_met: Vec<RefusalConditionNotMet>,
    },
    /// 理由不在单子上：照崩溃注入第二截今天的写法判（单元区墙计数，别的报红）。
    ReasonNotOnTheLegalRefusalList,
}

/// 崩溃镜像上按本池 fsid 跑一次只读恢复（看 journal）落到的那一版：判官的 `selected_version` 从这里取，不从被判的那次挂载取，判官与被判的不共用这个输入。
/// 不筛 fsid 的恢复在一块盘最新一槽是别的池的盘面上报各盘 fsid 对不上、没有这一版，判官复算判据就无从谈起。
#[must_use]
pub fn selected_version_of_the_pool_on_the_crash_image(
    crash_image: &dyn PoolReader,
    filesystem_identifier_of_the_pool: &[u8; 16],
) -> Option<(InstanceGeneration, CheckpointTxg)> {
    recover_of_the_pool(
        crash_image,
        JournalPolicy::Consult,
        filesystem_identifier_of_the_pool,
    )
    .effective_root
}

/// 判一次拒可写合不合法，只判 r5 判决 U6 那四条，不判 C670 与改法 G4 那两条（崩溃态知道注没注入读故障的调用方用
/// [`judge_writable_mount_refusal_of_a_crash_state_of_the_pool`]）。拒的判据在崩溃镜像（`crash_image`）上自己复算（不经 O1 判官，也不经实现判拒的那一段）；
/// `selected_version` 是崩溃镜像上只读恢复落到的那一版（盘上可能有别的池的槽时取 [`selected_version_of_the_pool_on_the_crash_image`]）。
/// `filesystem_identifier_of_the_pool` 是本池 fsid（建池参数里那一份）：复算每条判据用的系统配置只从这个 fsid 的槽里择，别的池的槽当自证不过、
/// 世代号再大也择不到（与可写挂载择系统配置同一个口径，C707（系统配置判不出里 fsid 不符落哪一边没写）；验证侧另写的一份
/// `verification_read::choose_system_configuration_of_the_pool`）。
#[must_use]
pub fn judge_writable_mount_refusal(
    crash_image: &dyn PoolReader,
    filesystem_identifier_of_the_pool: &[u8; 16],
    selected_version: Option<(InstanceGeneration, CheckpointTxg)>,
    refusal: &MountError,
    disk: DiskAfterTheRefusal,
    read_only_read_back: ReadOnlyReadBackOfTheCrashImage,
) -> WritableMountRefusalVerdict {
    let Some(reason) = refusal_reason_on_the_legal_refusal_list(refusal) else {
        return WritableMountRefusalVerdict::ReasonNotOnTheLegalRefusalList;
    };
    let system_configuration_of_the_pool =
        choose_system_configuration_of_the_pool(crash_image, filesystem_identifier_of_the_pool)
            .ok();
    verdict_of_a_refusal_on_the_legal_refusal_list(
        reason,
        conditions_not_met_by_a_refusal_on_the_legal_refusal_list(
            crash_image,
            system_configuration_of_the_pool.as_ref(),
            selected_version,
            reason,
            disk,
            read_only_read_back,
        ),
    )
}

/// 判崩溃注入第二、三截的一次拒可写合不合法：U6 那四条（[`judge_writable_mount_refusal`]），再加 C670 那一条——崩溃态没注入读故障
/// （`read_faults`）、理由是判据 N-配置，而判据在崩溃镜像上复算、撕裂的系统配置槽跳过之后仍为真（[`WitnessedCounterBranchThatHolds`]），
/// 拒得不合法（[`RefusalConditionNotMet::WitnessCriterionTrueOnACrashImageWithoutReadFaults`]）；与它同形的改法 G4——崩溃态没注入读故障、
/// 理由是「读阶段判完之后系统配置又读不出」，而那一拒的判据在崩溃镜像上复算为真，拒得不合法
/// （[`RefusalConditionNotMet::SystemConfigurationUnreadableAfterTheReadStageCriterionTrueOnACrashImageWithoutReadFaults`]）；与 G4 同形的 Q3——
/// 崩溃态没注入读故障、理由是「重建分配器读根环整环重读一次、两遍并起来仍一条不被抛弃的自证有效根都没有」，而那一拒的判据在崩溃镜像上复算为真，拒得不合法
/// （[`RefusalConditionNotMet::NoValidRootInTheRootRingCriterionTrueOnACrashImageWithoutReadFaults`]）。注入过读故障的崩溃态
/// 这三条都不判（D13（验证路线） 已定项 1 挂载透明的射程只管不带故障的状态）。理由是「读阶段判完之后系统配置又读不出」或 Q3 那一拒而被拒的那次挂载
/// 自己在读故障下跑的（`read_faults_under_the_refused_mount`），判据在崩溃镜像上复算为假不算不合法（两遍读之间的瞬时读错复算不出，C664（挂载拒绝合法复算
/// 把两种拒因报成一个））；被拒的那次挂载不带故障层的，崩溃态注没注入过读故障都照 U6 复算（判据为假即不合法，改法 G3）。
/// 复算判据用的系统配置只从 `filesystem_identifier_of_the_pool` 的槽里择（与 [`judge_writable_mount_refusal`] 同一个口径）：别的池的槽世代号再大也择不到。
#[must_use]
#[allow(
    clippy::too_many_arguments,
    reason = "参数各是判一次拒要的一样输入，不成一个概念"
)]
pub fn judge_writable_mount_refusal_of_a_crash_state_of_the_pool(
    crash_image: &dyn PoolReader,
    filesystem_identifier_of_the_pool: &[u8; 16],
    selected_version: Option<(InstanceGeneration, CheckpointTxg)>,
    refusal: &MountError,
    disk: DiskAfterTheRefusal,
    read_only_read_back: ReadOnlyReadBackOfTheCrashImage,
    read_faults: ReadFaultsInTheCrashState,
    read_faults_under_the_refused_mount: ReadFaultsUnderTheRefusedMount,
) -> WritableMountRefusalVerdict {
    let Some(reason) = refusal_reason_on_the_legal_refusal_list(refusal) else {
        return WritableMountRefusalVerdict::ReasonNotOnTheLegalRefusalList;
    };
    verdict_of_a_refusal_of_a_crash_state_under_the_chosen_system_configuration(
        crash_image,
        choose_system_configuration_of_the_pool(crash_image, filesystem_identifier_of_the_pool)
            .ok(),
        selected_version,
        reason,
        disk,
        read_only_read_back,
        read_faults,
        read_faults_under_the_refused_mount,
    )
}

/// 崩溃态那一支的判法（各条写在 [`judge_writable_mount_refusal_of_a_crash_state_of_the_pool`] 上），
/// 系统配置由调用方按本池 fsid 择好（`system_configuration_of_whichever_pool`，择不到是 `None`）。
#[allow(
    clippy::too_many_arguments,
    reason = "参数各是判一次拒要的一样输入，不成一个概念"
)]
fn verdict_of_a_refusal_of_a_crash_state_under_the_chosen_system_configuration(
    crash_image: &dyn PoolReader,
    system_configuration_of_whichever_pool: Option<SystemConfiguration>,
    selected_version: Option<(InstanceGeneration, CheckpointTxg)>,
    reason: RefusalReasonOnTheLegalRefusalList,
    disk: DiskAfterTheRefusal,
    read_only_read_back: ReadOnlyReadBackOfTheCrashImage,
    read_faults: ReadFaultsInTheCrashState,
    read_faults_under_the_refused_mount: ReadFaultsUnderTheRefusedMount,
) -> WritableMountRefusalVerdict {
    let mut conditions_not_met = conditions_not_met_by_a_refusal_on_the_legal_refusal_list(
        crash_image,
        system_configuration_of_whichever_pool.as_ref(),
        selected_version,
        reason,
        disk,
        read_only_read_back,
    );
    match (reason, read_faults) {
        (
            RefusalReasonOnTheLegalRefusalList::NewerPublishWitnessedBySystemConfiguration,
            ReadFaultsInTheCrashState::NoneInjected,
        ) => {
            if let Some(branch) = witnessed_counter_holds_on_a_crash_image_without_read_faults(
                crash_image,
                system_configuration_of_whichever_pool.as_ref(),
                selected_version,
            ) {
                conditions_not_met.push(
                    RefusalConditionNotMet::WitnessCriterionTrueOnACrashImageWithoutReadFaults(
                        branch,
                    ),
                );
            }
        }
        // 改法 G4：没有读故障时设备不报读错，某块盘两槽都自证不过只能是盘面本身如此——正确实现在 mkfs 返回之后没有读故障的历史里留不下它。
        // mkfs 返回之前的崩溃态不在这一条的射程：mkfs 逐盘两槽在同一个循环里写、之后才逐盘屏障，那一段留得下两槽都自证不过的盘
        // （D13（验证路线） 已定项 6）；调这一臂的崩溃注入第二、三截，候选段都取在 mkfs 之后。判槽自证读池级 checker 的逐盘数
        // （「改读池级 checker」），不用核心层的解析。
        (
            RefusalReasonOnTheLegalRefusalList::SystemConfigurationUnreadableAfterTheReadStage,
            ReadFaultsInTheCrashState::NoneInjected,
        ) => {
            if system_configuration_unreadable_after_the_read_stage_judged_by_the_pool_checker_counts_on_the_crash_image(
                crash_image,
            ) {
                conditions_not_met.push(
                    RefusalConditionNotMet::SystemConfigurationUnreadableAfterTheReadStageCriterionTrueOnACrashImageWithoutReadFaults,
                );
            }
        }
        // Q3 与 G4 同形：没有读故障时设备不报读错，读阶段择得到的根按它自己的实例表不被抛弃——这一拒的判据为真只能是盘面本身如此。
        (
            RefusalReasonOnTheLegalRefusalList::NoValidRootInTheRootRingOfTheRebuiltAllocator,
            ReadFaultsInTheCrashState::NoneInjected,
        ) => {
            if no_valid_root_in_the_root_ring_under_the_newest_roots_instance_table_on_the_crash_image(
                crash_image,
                system_configuration_of_whichever_pool.as_ref(),
            ) {
                conditions_not_met.push(
                    RefusalConditionNotMet::NoValidRootInTheRootRingCriterionTrueOnACrashImageWithoutReadFaults,
                );
            }
        }
        (
            RefusalReasonOnTheLegalRefusalList::NewerPublishWitnessedBySystemConfiguration
            | RefusalReasonOnTheLegalRefusalList::SystemConfigurationUnreadableAfterTheReadStage
            | RefusalReasonOnTheLegalRefusalList::NoValidRootInTheRootRingOfTheRebuiltAllocator,
            ReadFaultsInTheCrashState::Injected,
        )
        | (
            RefusalReasonOnTheLegalRefusalList::InstanceTableOfTheNewestRootUnreadable
            | RefusalReasonOnTheLegalRefusalList::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify
            | RefusalReasonOnTheLegalRefusalList::AccountOfAnUnappliedPublishWithIncompleteReadableRecords {
                ..
            },
            ReadFaultsInTheCrashState::NoneInjected | ReadFaultsInTheCrashState::Injected,
        ) => {}
    }
    // 读阶段之后系统配置又读不出、被拒的那次挂载自己在读故障下跑：两遍读之间的瞬时读错在崩溃镜像上复算不出，判据复算为假不算拒得不合法
    // （C664（挂载拒绝合法复算把两种拒因报成一个）的还清条件：带读故障的档里取号那一刻瞬时读错的态，分开报之后判合法）；
    // 盘上逐字节不变与只读读出照判。被拒的那次挂载不带故障层的照 U6 复算（判据为假即不合法）。
    // 重建分配器读根环重读仍一条有效根都没有（Q3）同一个办法：重建那一遍与它的重读读坏是这次挂载里的瞬时读错，崩溃镜像上复算不出。
    match reason {
        RefusalReasonOnTheLegalRefusalList::SystemConfigurationUnreadableAfterTheReadStage
        | RefusalReasonOnTheLegalRefusalList::NoValidRootInTheRootRingOfTheRebuiltAllocator => {
            if read_faults_under_the_refused_mount
                .can_hide_a_transient_read_error_from_the_crash_image()
            {
                conditions_not_met.retain(|condition| {
                    *condition
                        != RefusalConditionNotMet::CriterionFalseWhenRecomputedOnTheCrashImage
                });
            }
        }
        // 「记录有缺口时拒可写」那一拒没有这一条宽限：被拒的那次挂载在读故障下跑、判据在崩溃镜像上复算为假的照判不合法
        // （D23 已定项 14 那一段只写判据、没写读故障下这一拒怎么判；取不放宽的一边：复算不出的报红，不藏）。
        // `C_MAPPING` 那一拒同一个取法（被攻过零轮）。
        RefusalReasonOnTheLegalRefusalList::NewerPublishWitnessedBySystemConfiguration
        | RefusalReasonOnTheLegalRefusalList::InstanceTableOfTheNewestRootUnreadable
        | RefusalReasonOnTheLegalRefusalList::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify
        | RefusalReasonOnTheLegalRefusalList::AccountOfAnUnappliedPublishWithIncompleteReadableRecords {
            ..
        } => {}
    }
    verdict_of_a_refusal_on_the_legal_refusal_list(reason, conditions_not_met)
}

/// 理由在单子上（U6 第一条）之后，U6 另三条里没满足的：判据在崩溃镜像上复算为真、盘上逐字节不变、只读读出在允许集合里。
/// 复算判据用的系统配置是调用方择好的 `chosen_system_configuration`（择不到是 `None`，每条判据各自怎么处置写在各自的函数上）。
fn conditions_not_met_by_a_refusal_on_the_legal_refusal_list(
    crash_image: &dyn PoolReader,
    chosen_system_configuration: Option<&SystemConfiguration>,
    selected_version: Option<(InstanceGeneration, CheckpointTxg)>,
    reason: RefusalReasonOnTheLegalRefusalList,
    disk: DiskAfterTheRefusal,
    read_only_read_back: ReadOnlyReadBackOfTheCrashImage,
) -> Vec<RefusalConditionNotMet> {
    let criterion_holds_on_the_crash_image = match reason {
        RefusalReasonOnTheLegalRefusalList::NewerPublishWitnessedBySystemConfiguration => {
            newer_publish_witnessed_on_the_crash_image(
                crash_image,
                chosen_system_configuration,
                selected_version,
            )
        }
        RefusalReasonOnTheLegalRefusalList::InstanceTableOfTheNewestRootUnreadable => {
            instance_table_of_the_newest_root_unreadable_on_the_crash_image(
                crash_image,
                chosen_system_configuration,
            )
        }
        RefusalReasonOnTheLegalRefusalList::SystemConfigurationUnreadableAfterTheReadStage => {
            system_configuration_unreadable_after_the_read_stage_on_the_crash_image(
                crash_image,
                chosen_system_configuration,
            )
        }
        RefusalReasonOnTheLegalRefusalList::NoValidRootInTheRootRingOfTheRebuiltAllocator => {
            no_valid_root_in_the_root_ring_under_the_newest_roots_instance_table_on_the_crash_image(
                crash_image,
                chosen_system_configuration,
            )
        }
        RefusalReasonOnTheLegalRefusalList::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify => {
            publish_above_the_water_mark_missing_records_whose_named_units_do_not_verify_on_the_crash_image(
                crash_image,
                chosen_system_configuration,
                selected_version,
            )
        }
        RefusalReasonOnTheLegalRefusalList::AccountOfAnUnappliedPublishWithIncompleteReadableRecords {
            unapplied_publish,
        } => account_of_an_unapplied_publish_with_incomplete_readable_records_on_the_crash_image(
            crash_image,
            chosen_system_configuration,
            selected_version,
            unapplied_publish,
        ),
    };
    let mut conditions_not_met = Vec::new();
    if !criterion_holds_on_the_crash_image {
        conditions_not_met
            .push(RefusalConditionNotMet::CriterionFalseWhenRecomputedOnTheCrashImage);
    }
    match disk {
        DiskAfterTheRefusal::ByteForByteUnchanged => {}
        DiskAfterTheRefusal::Changed => {
            conditions_not_met.push(RefusalConditionNotMet::DiskChangedByTheRefusedMount);
        }
    }
    match read_only_read_back {
        ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet => {}
        ReadOnlyReadBackOfTheCrashImage::OutsideTheAllowedSet => {
            conditions_not_met.push(RefusalConditionNotMet::ReadOnlyReadBackOutsideTheAllowedSet);
        }
    }
    conditions_not_met
}

fn verdict_of_a_refusal_on_the_legal_refusal_list(
    reason: RefusalReasonOnTheLegalRefusalList,
    conditions_not_met: Vec<RefusalConditionNotMet>,
) -> WritableMountRefusalVerdict {
    if conditions_not_met.is_empty() {
        WritableMountRefusalVerdict::Legal(reason)
    } else {
        WritableMountRefusalVerdict::Illegal {
            reason,
            conditions_not_met,
        }
    }
}

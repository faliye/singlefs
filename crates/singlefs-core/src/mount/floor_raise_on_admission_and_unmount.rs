//! 抬 F 的两个入口：准入抬 F 到上限（一次准入里再推一串、推空不抬 F）与正常卸载，
//! 连同两个入口交回的结果、抬 F 停下的原因与每块盘上最新有效根的 txg。

use super::*;

/// 准入推的空发布做完之后的东西：一串抬 F 的空发布，或上限卡在挂载时读坏、读回全零的槽上推的一次不抬 F 的空发布
/// （[`FloorOfThePushedPublishes`] 分；两种都算进这次准入的已推次数，D16（发布语义） 已定项 1「准入」那一行的 B）。
#[derive(Clone, Debug)]
pub struct RaisedFloor {
    /// 这一次判过的上限：抬 F 那一串抬到的就是它；不抬 F 的那一次是并进那几槽之后的上限（不高于 F）。
    pub ceiling: CheckpointTxg,
    /// 第一次发布之前先把新 F 写进每块盘系统配置那一步的写（每块盘一次系统配置槽写，D16（发布语义） 已定项 1「抬 F 那一串」）：
    /// 不属于 `publishes` 里任何一次，与它们相加才等于这一串交给设备的写。不抬 F 的那一次 F 不变、不写系统配置，这一项是空账。
    pub system_configuration_writes_before_the_first_publish: WritesByStructureKind,
    /// 抬 F 那一串：带新 F 的空发布，直到每块盘上都有一条带新 F 的持久根（生效条件）。不抬 F 的那一次：那一次带现行 F 的空发布。
    pub publishes: Vec<TransactionOutput>,
    /// 这一串推的空发布抬没抬 F。
    pub floor_of_the_publishes: FloorOfThePushedPublishes,
    /// 生效之后回收的落点（两盘同槽，按盘 0 报）。不抬 F 的那一次不另回收，是空的。
    pub reclaimed: Vec<Placement>,
    /// 抬 F 重算影子账时读不出账、被跳过的被抛弃根的条数（与 `MountOutput::abandoned_roots_unreadable` 同一个量；
    /// 代码三方第三轮云端攻方腿打中：抬 F 那一处此前把计数丢掉了）。根环表里记着落点的被抛弃根、借 C 那一圈落点的被抛弃根与根槽这一遍读坏的
    /// 被抛弃根这一遍都不读盘，不计在内。不抬 F 的那一次不重算影子账，是 0。
    pub abandoned_roots_unreadable: u64,
}

/// 准入推的空发布抬没抬 F（[`RaisedFloor::floor_of_the_publishes`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FloorOfThePushedPublishes {
    /// 抬到上限（[`RaisedFloor::ceiling`]）：先把新 F 写进每块盘的系统配置，再推带新 F 的空发布到每块盘上都有一条（「抬 F 那一串」那一行）。
    RaisedToTheCeiling,
    /// 上限卡在挂载那一刻读坏或读回全零、之后这个进程没写过的槽上（并进它们之后上限不高于现行 F、不并它们时高于现行 F）：
    /// 推了一次不抬 F 的空发布（带现行 F、txg 加一），不写系统配置、不重算影子账与回收门槛
    /// （D16（发布语义） 已定项 1「抬 F 的上限」那一行，用户 2026-11-02 定「推空」；取 E173（上限卡在读坏槽的三种处置） 登记的「推空B」那一臂）。
    /// 推到这个进程写掉那几槽，上限就不再被它们卡住。`ceiling_from_the_roots_read` 是不并那几槽时的上限。
    KeptWhileTheCeilingIsHeldByRingSlotsBadSinceTheMount {
        ceiling_from_the_roots_read: CheckpointTxg,
    },
}

/// 正常卸载做完之后的样子（D16（发布语义） 已定项 1「正常卸载」）。
pub enum Unmounted {
    /// 现行那一版带文件：推了一串带卸载记号的空发布，F 抬到它的 txg。
    FloorRaisedToTheCurrentVersion(UnmountRaisedTheFloor),
    /// 现行那一版树表 0 条（还没写过文件）：一个字节都不写，卸载照常做成。回退候选要带文件，而这一版在它的时间线上还没有带文件的版本，
    /// 抬 F 买不到东西（主 agent 2026-09-26 定，实二交回第一问；推的，被攻过零轮）。
    NothingWrittenOnAVersionWithoutFile { current: RollbackTarget },
}

/// 正常卸载推了那一串空发布之后的东西。卸载不判上限，没有 `ceiling`。
pub struct UnmountRaisedTheFloor {
    /// 抬到的 F：卸载开始时现行那一版的 txg。
    pub rollback_floor: CheckpointTxg,
    /// 第一次发布之前先把新 F 写进每块盘系统配置那一步的写（同 [`RaisedFloor`] 那一项）。
    pub system_configuration_writes_before_the_first_publish: WritesByStructureKind,
    /// 带新 F、带卸载记号的空发布，直到每块盘上都有一条（第一版几何上 2 或 3 次）。
    pub publishes: Vec<TransactionOutput>,
    /// 生效之后回收的落点（同 [`RaisedFloor`] 那一项）。
    pub reclaimed: Vec<Placement>,
    /// 同 [`RaisedFloor`] 那一项。
    pub abandoned_roots_unreadable: u64,
}

/// 抬 F 那一串从哪个入口进来（D16（发布语义） 已定项 1「抬 F 那一串」：准入与卸载共用这一串，两个入口只差三样：
/// 判不判上限、根带不带卸载记号、回收出来的槽什么时候能发出去）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RaiseFloorEntry {
    /// 准入抬 F（`raise_rollback_floor`）：新 F 不高于上限（「抬 F 的上限」那一行的准入式子），这一串的根不带卸载记号。
    Admission,
    /// 正常卸载（`unmount`）：新 F 是现行那一版的 txg、不另判上限，这一串的根全带卸载记号（D22（单元原子性怎么合成） 已定项 7）。
    Unmount,
}

impl RaiseFloorEntry {
    pub(super) const fn unmount_marker(self) -> UnmountMarker {
        match self {
            RaiseFloorEntry::Admission => UnmountMarker::NotWrittenByTheUnmountSequence,
            RaiseFloorEntry::Unmount => UnmountMarker::WrittenByTheUnmountSequence,
        }
    }

    /// 这一串回收出来的槽什么时候能发出去（用户 2026-10-03 定「甲 GA」，`research/prompts/m3-unlink-code-r3-forks.md` 丙1 单盘幸存那一格；
    /// 判决 `research/prompts/m3-unlink-code-r3-main-verification.md` 第三节第 1 条）：准入那一格整截扣到生效（每块盘上都有带新 F 的根）；
    /// 卸载那一格只把释放代不高于「每块盘上最新有效根的 txg」最小者的那一截当场放开，整串预演与真发都能用，其余照准入那一格扣到生效
    /// （[`ReuseOfTheSlotsReclaimedByTheSequence`]）。
    /// 为什么只放那一截：恢复择根不看 F（`recovery::choose_root` 照 `(checkpoint_txg, 实例代号)` 取最大），丢掉一块盘之后择的是剩下那块盘上
    /// 最新的根，它可以低于系统配置里的新 F（判决第二节 N1 那一行）；一个槽的释放代 g 不高于每块盘上最新的有效根，就没有一块盘上最新的根引用它
    /// （引用它的根都早于 g），丢哪块盘恢复都择不到引用它的根。挂载续做推满仍放不下之后卸载靠这一截做成（槽被续做吃光，整截扣着的话这一串在预演里
    /// 取不到落点，`admission_raises_the_floor_before_refusing.rs` 里续做推满之后卸载、再挂续做那一条）。
    pub(super) const fn reuse_of_the_reclaimed_slots(
        self,
    ) -> ReuseOfTheSlotsReclaimedByTheSequence {
        match self {
            RaiseFloorEntry::Admission => ReuseOfTheSlotsReclaimedByTheSequence::HeldUntilFloorTakesEffect,
            RaiseFloorEntry::Unmount => ReuseOfTheSlotsReclaimedByTheSequence::ImmediatelyBelowTheNewestValidRootOnEveryDevice,
        }
    }
}

/// 抬 F 那一串回收出来的槽什么时候能发出去（[`RaiseFloorEntry::reuse_of_the_reclaimed_slots`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ReuseOfTheSlotsReclaimedByTheSequence {
    /// 整截扣到生效（带新 F 的根落满每块盘之后 `PoolAllocator::release_reclaim_holds` 才放开）。
    HeldUntilFloorTakesEffect,
    /// 释放代不高于 [`newest_valid_root_txg_on_every_device`] 的那一截当场能发，其余扣到生效。
    ImmediatelyBelowTheNewestValidRootOnEveryDevice,
}

/// 每块盘上最新的有效根的 txg 取最小（GA，[`RaiseFloorEntry::reuse_of_the_reclaimed_slots`]；可写挂载重建分配器时当场放开到哪儿、
/// 要不要整环重读也取它，甲、Q4 rr2，`rebuilt_allocator`）：`roots_on_their_devices` 是调用方这一遍从根环读出来的根，各带它归哪块盘，
/// `table` 是判被抛弃用的那张实例表（卸载那一格是现行那一版的，重建那一格是最新那条根指着的）。归盘由调用方定，两格都按读出它的那个槽
/// 所在的区域（mkfs 三个区域槽 0 各一份的第 0 代根各归各的盘，判决 `research/prompts/m3-batch10-code-r2-main-verification.md` 第二节 K2；
/// 卸载那一格 [`roots_on_the_devices_of_their_ring_slots`]，判决 `research/prompts/m3-close-b19-reason-r3-main-verification.md` 第二节 Q4 第 3 问）。
/// - 被抛弃的根（按 `table`，`abandoned_by_table`）不算。卸载那一格滤不滤结局相同：卸载接在一次可写挂载之后，那次挂载的暖机先在每块盘上
///   落了新实例的根（D16（发布语义） 已定项 8 戊），新实例的 txg 大于环里每一条根（D23（journal 的角色与格式） 已定项 14 第 3 条），
///   卸载时各盘最新的根都不是被抛弃的（判决 `research/prompts/m3-batch14-code-r1-main-verification.md` 第二节「单盘恢复择中被抛弃的根」那一行，
///   一次抽样 9027 个判定择中 0 次）。
/// - 这一遍读不出、自证不过的槽当没有根（与这一遍回收门槛同一个读数）：恢复同样择不到它；它日后又读得出的话那条根只会更新或更旧——更新的引用的已释放槽更少，
///   更旧的轮不到被择。读不出让最小值只会更小，当场放开的只会更少。
/// - 有一块盘上一条有效根都没读出来：交回 `None`（不按 txg 0 去比，说清是哪一格），两格各自处置：
///   卸载那一格这一串回收的槽整截扣到生效；重建那一格当场只放到环里最旧有效根（RGall；挂载时读坏的槽钉在它可能住过的最旧 txg 并进，
///   与回收门槛同一个读数），其余扣到这次挂载的暖机落满每块盘（`rebuilt_allocator`，用户 2026-10-04 选甲，
///   `research/prompts/m3-batch16-code-r1-forks.md`「甲 只放到窗口里最旧有效根」那一行）。
pub(super) fn newest_valid_root_txg_on_every_device(
    roots_on_their_devices: &[(DeviceIdentity, RootRecord)],
    table: &InstanceTableRecords,
    all_devices: &[DeviceIdentity],
) -> Option<CheckpointTxg> {
    let newest_on_each_device: Vec<Option<CheckpointTxg>> = all_devices
        .iter()
        .map(|device| {
            roots_on_their_devices
                .iter()
                .filter(|(device_of_the_root, root)| {
                    !abandoned_by_table(root, table) && *device_of_the_root == *device
                })
                .map(|(_, root)| root.checkpoint_txg)
                .max()
        })
        .collect();
    newest_on_each_device
        .into_iter()
        .collect::<Option<Vec<CheckpointTxg>>>()?
        .into_iter()
        .min()
}

/// 抬回退下界 F 到 `new_floor`（D16（发布语义） 已定项 1）。
///
/// 产品路径上的触发是准入（「准入」那一行：准入不够、或准入放行而落点取不到时先推空发布抬 F），经
/// [`raise_rollback_floor_to_the_admission_ceiling`] 抬到上限：挂着的会话在发布被拒时推（`mounted_session`），可写挂载在写行之后推
/// （`establish_instance`）。直接给一个 `new_floor` 的这一个入口是只供测试强制进入的那一档（`.claude/rules/fs-design.md`
/// 五条硬要求第 2 条）：抬到上限以下的某一格。
/// 推空发布直到每块盘上都有带新 F 的根（生效），然后把释放代 ≤ 新 F 的已释放落点回收；第一次发布之前先把新 F 写进每块盘的系统配置、
/// 过一道屏障（「抬 F 那一串」那一行）。
///
/// 调用方交进来的参数与盘表照可写挂载同一套核，写入口只按盘上择到的那一份参数建。
///
/// # Errors
/// 盘表里有身份交了不止一次（`DeviceIdentitiesHandedInMoreThanOnce`）、调用方的参数或盘表（盘数与 `devs`、每块盘的本盘设备号）
/// 与盘上择到的系统配置不一致、或有盘不「可见」（`CallerParametersDisagreeWithTheSelectedSystemConfiguration`）、有盘落后于现行那一版又缺它的单元或它的根
/// （`DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot`），都在回收、影子账与任何写之前；
/// 现行那一版的根指着的实例表读不出（`InstanceTableUnreadable`）或解不开（`InstanceTableMalformed`）；`new_floor` 低于盘上现算的 F 生效值
/// （`RequestedFloorBelowTheEffectiveFloorWhoseRaiseIsUndecided`）；`new_floor` 超过上限；算上限的错（[`rollback_floor_ceiling`]，
/// 含知道住着根的根环槽重读仍坏 `RollbackFloorCeilingRootRingSlotStillBadAfterOneReread`）；现行那一版的 txg 已到顶、这一串的 txg 装不下
/// （`SequenceNumberPastTheTopOfItsRange`，在回收、影子账与任何写之前）；
/// 那一串空发布在任何写之前的预演里有一次报错（`RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite`，一次都不发）；
/// 先写系统配置那一步报错（`RaiseFloorSystemConfigurationWriteFailedBeforeAnyRoot`，一条根都不发）；
/// 预演过了、真发时有一次发不出去（`RaiseFloorSequencePublishFailed`，带着先写系统配置那一步、前面已经落盘的那几次与失败那一次的写账）。
pub fn raise_rollback_floor<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    allocator: &mut PoolAllocator,
    current: &mut TransactionOutput,
    new_floor: CheckpointTxg,
    shadow_ledger: ShadowLedger,
) -> Result<RaisedFloor, MountError> {
    let raised = raise_the_floor_through(
        RaiseFloorEntry::Admission,
        parameters,
        devices,
        allocator,
        current,
        new_floor,
        shadow_ledger,
    )?;
    Ok(RaisedFloor {
        ceiling: raised
            .ceiling_judged_by_the_admission
            .expect("准入那一格判过上限才走到发布：`raise_the_floor_through` 只在准入入口算上限，这里是准入入口"),
        system_configuration_writes_before_the_first_publish: raised
            .system_configuration_writes_before_the_first_publish,
        publishes: raised.publishes,
        floor_of_the_publishes: FloorOfThePushedPublishes::RaisedToTheCeiling,
        reclaimed: raised.reclaimed,
        abandoned_roots_unreadable: raised.abandoned_roots_unreadable,
    })
}

/// 正常卸载（D16（发布语义） 已定项 1「正常卸载」，用户 2026-09-25 定 B1）：推一串空发布把 F 抬到现行那一版的 txg、不另判上限，
/// 走的是准入抬 F 那一串（影子账按新 F 重算、回收、整串预演、先把新 F 写进每块盘的系统配置过一道屏障、推到每块盘上都有一条带新 F 的根），
/// 这一串写的根在 flags 里带卸载记号（D22（单元原子性怎么合成） 已定项 7）。与准入那一格差在回收出来的槽里释放代不高于每块盘上最新有效根的
/// 那一截不扣到生效、预演与真发都能用，其余照样扣（用户 2026-10-03 定甲 GA，[`RaiseFloorEntry::reuse_of_the_reclaimed_slots`]）：
/// 挂载续做推满仍放不下、槽被吃光的盘上卸载照样做成，靠的就是这一串的预演算上那一截。第一版几何（R = 3、区域归属 0 / 1 / 0）下推 2 或 3 次：
/// 现行 txg 除以 3 余 1 时 3 次，余 0 或余 2 时 2 次。不写干净关闭标记，下一次挂载照旧走恢复。盘表、分配器与现行那一版只借给它
/// （`&mut`），返回之后调用方手里照样有；卸载之后不再拿它们发布，由调用方守，这里不拦。
/// 现行那一版树表 0 条时一个字节都不写，卸载照常做成（[`Unmounted::NothingWrittenOnAVersionWithoutFile`]）。
///
/// # Errors
/// 同 [`raise_rollback_floor`]（含调用方参数与盘表的那一套核），少一条「超过上限」；现行那一版树表 0 条时不报错、不核（一个字节都不写）。
pub fn unmount<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    allocator: &mut PoolAllocator,
    current: &mut PoolVersion,
    shadow_ledger: ShadowLedger,
) -> Result<Unmounted, MountError> {
    let current_file_version = match current {
        PoolVersion::WithFile(current_file_version)
        | PoolVersion::WithTreesAndNoFile(current_file_version) => current_file_version,
        PoolVersion::WithoutFile(current_version_without_file) => {
            return Ok(Unmounted::NothingWrittenOnAVersionWithoutFile {
                current: RollbackTarget {
                    instance: current_version_without_file.root.instance,
                    checkpoint_txg: current_version_without_file.root.checkpoint_txg,
                },
            });
        }
    };
    let rollback_floor = current_file_version.root.checkpoint_txg;
    // 抬 F 就地推进现行那一版：还有别人指着它（版本列表里那一份）时先拷一份自己的再推，别人指着的那一份不动。
    let raised = raise_the_floor_through(
        RaiseFloorEntry::Unmount,
        parameters,
        devices,
        allocator,
        Arc::make_mut(current_file_version),
        rollback_floor,
        shadow_ledger,
    )?;
    Ok(Unmounted::FloorRaisedToTheCurrentVersion(
        UnmountRaisedTheFloor {
            rollback_floor,
            system_configuration_writes_before_the_first_publish: raised
                .system_configuration_writes_before_the_first_publish,
            publishes: raised.publishes,
            reclaimed: raised.reclaimed,
            abandoned_roots_unreadable: raised.abandoned_roots_unreadable,
        },
    ))
}

/// 一次准入最多推几次发布：B = 4 + 2 k_tol，k_tol = 2 ⇒ 8（D16（发布语义） 已定项 1「准入」那一行）。数的是这次准入推的空发布
/// （抬 F 那几串与上限卡在读坏槽、读回全零的槽上推的不抬 F 的空发布都算，「抬 F 的上限」那一行）加上这次准入为之判的那一次发布
/// （发布路径是那次用户发布，可写挂载那一处是写行那次）。`df` 扣推空最坏残留 5 + (B − 2) × c_max（同一张表「`df`」那一行）；
/// 保留池按 ckpt_cost 现算（「准入」那一行，用户 2026-11-02 定），不按 B 留。
pub const PUBLISHES_PER_ADMISSION_AT_MOST: usize = 8;

/// 准入抬 F 到上限那一次的结局（[`raise_rollback_floor_to_the_admission_ceiling`]）。
#[derive(Debug)]
pub enum RaiseToTheAdmissionCeiling {
    /// 上限高于现行那一版的 F：推了那一串，F 抬到上限。公开入口 [`raise_rollback_floor_to_the_admission_ceiling`] 只交这一种推法；
    /// 准入推的那一处（[`push_one_floor_raise_within_the_admission_budget`]）上限卡在挂载时读坏、读回全零的槽上时推的那一次不抬 F 的空发布
    /// 也装在这里（[`RaisedFloor::floor_of_the_publishes`] 分）。
    Raised(RaisedFloor),
    /// 上限不高于现行那一版的 F：没有可抬的，一个字节都没写、分配器不动。
    FloorAlreadyAtTheCeiling {
        floor: CheckpointTxg,
        ceiling: CheckpointTxg,
    },
}

/// 准入抬 F（D16（发布语义） 已定项 1「准入」那一行：准入不够、或准入放行而落点取不到时先推空发布抬 F）：F 抬到这一刻算出的上限
/// （「抬 F 的上限」那一行的准入式子，[`rollback_floor_ceiling`]）。上限不高于现行那一版的 F 就不推。
/// 上限在这里算一次、交给 [`raise_rollback_floor`] 之后它在任何写之前按同一份输入再算一次：两次之间没有写，
/// 对不上（两次读之间的瞬时读错让第二次算得更低）时那边在任何写之前拒（`RollbackFloorAboveCeiling`）。
/// 调用方交进来的参数与盘表先照可写挂载同一套核（`caller_inputs_agreeing_with_the_disk`），
/// 之后只用盘上那一份参数；上限不高于 F、一个字节都不写的那一格也先核。
///
/// # Errors
/// 盘表里有身份交了不止一次、调用方的参数或盘表与盘上系统配置不一致（`DeviceIdentitiesHandedInMoreThanOnce`、
/// `CallerParametersDisagreeWithTheSelectedSystemConfiguration`）；选系统配置、读现行那一版的实例表、算上限的错（都在任何写之前；
/// 含一条有效根都没有的 `Recovery(NoValidRoot)`：调用方给的分配器不带这个进程的根环表时，根环全读坏就走到它）；
/// [`raise_rollback_floor`] 的错原样交回。
pub fn raise_rollback_floor_to_the_admission_ceiling<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    allocator: &mut PoolAllocator,
    current: &mut TransactionOutput,
    shadow_ledger: ShadowLedger,
) -> Result<RaiseToTheAdmissionCeiling, MountError> {
    let caller_inputs_agreeing_with_the_disk =
        caller_inputs_agreeing_with_the_disk(parameters, devices, allocator, current)?;
    raise_to_the_admission_ceiling_on_caller_inputs_agreeing_with_the_disk(
        &caller_inputs_agreeing_with_the_disk,
        devices,
        allocator,
        current,
        shadow_ledger,
    )
}

/// [`raise_rollback_floor_to_the_admission_ceiling`] 核过调用方的参数与盘表之后那一半：算上限、不高于 F 就不推，高于就抬到它。
fn raise_to_the_admission_ceiling_on_caller_inputs_agreeing_with_the_disk<Device: BlockDevice>(
    caller_inputs_agreeing_with_the_disk: &CallerInputsAgreeingWithTheDisk,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    allocator: &mut PoolAllocator,
    current: &mut TransactionOutput,
    shadow_ledger: ShadowLedger,
) -> Result<RaiseToTheAdmissionCeiling, MountError> {
    let ceilings = admission_ceilings_on_caller_inputs_agreeing_with_the_disk(
        caller_inputs_agreeing_with_the_disk,
        devices,
        allocator,
        current,
    )?;
    raise_to_the_admission_ceiling_judged(
        caller_inputs_agreeing_with_the_disk,
        devices,
        allocator,
        current,
        shadow_ledger,
        ceilings.not_passing_the_ring_slots_bad_since_the_mount,
    )
}

/// 这个进程这一刻的准入抬 F 的两个上限（[`rollback_floor_ceilings_of_this_process`]），判有效用现行那一版的实例表（从它的根指着的第 0 片沿链读）。
/// 只读盘。这一读与算上限读树表那一读读到只坏一份都进别的读那份账：F 抬不动、紧接着的 [`raise_the_floor_through`] 不跑时只有它们读到（D4（校验和位置） 已定项 4）。
///
/// # Errors
/// 现行那一版的实例表读不出或解不开；算上限的错（同 [`rollback_floor_ceiling`]）。
#[allow(
    clippy::ptr_arg,
    reason = "rollback_floor_ceilings_of_this_process 走 &Vec：要一个有大小的读者"
)]
fn admission_ceilings_on_caller_inputs_agreeing_with_the_disk<Device: BlockDevice>(
    caller_inputs_agreeing_with_the_disk: &CallerInputsAgreeingWithTheDisk,
    devices: &Vec<(DeviceIdentity, Device)>,
    allocator: &PoolAllocator,
    current: &TransactionOutput,
) -> Result<AdmissionCeilings, MountError> {
    let current_instance_table = allocator
        .reading_and_counting_the_other_unit_reads_with_one_copy_failing(devices, |reader| {
            instance_table_chain_of_root(reader, &current.root)
        })
        .map_err(|failure| instance_table_refusal(&failure))?
        .records;
    rollback_floor_ceilings_of_this_process(
        devices,
        &caller_inputs_agreeing_with_the_disk.system_configuration,
        &current.root,
        &current_instance_table,
        allocator,
    )
}

/// 上限 `ceiling` 判过之后那一步：不高于现行那一版的 F 就不推（一个字节都不写、分配器不动），高于就抬到它（[`raise_rollback_floor`]，
/// 它在任何写之前按同一份输入再算一次上限）。
fn raise_to_the_admission_ceiling_judged<Device: BlockDevice>(
    caller_inputs_agreeing_with_the_disk: &CallerInputsAgreeingWithTheDisk,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    allocator: &mut PoolAllocator,
    current: &mut TransactionOutput,
    shadow_ledger: ShadowLedger,
    ceiling: CheckpointTxg,
) -> Result<RaiseToTheAdmissionCeiling, MountError> {
    if ceiling <= current.root.rollback_floor {
        return Ok(RaiseToTheAdmissionCeiling::FloorAlreadyAtTheCeiling {
            floor: current.root.rollback_floor,
            ceiling,
        });
    }
    raise_rollback_floor(
        &caller_inputs_agreeing_with_the_disk.parameters_of_the_pool,
        devices,
        allocator,
        current,
        ceiling,
        shadow_ledger,
    )
    .map(RaiseToTheAdmissionCeiling::Raised)
}

/// 一次准入里为什么不再推抬 F 的空发布（推完仍不够时交回的原因）：只有「推满仍不够」的三种——预算用完、F 已在上限、
/// 抬 F 那一串自己也被空间不够拒（D16（发布语义） 已定项 1「准入」那一行：落点取不到也先推抬 F 再判，推不出空间就是推满仍不够）。
/// 抬 F 自己报的别的错（块设备错、读不出、预演被别的理由拒……）不在这里：它不是空间不够，原样往上交（代码审阅第 23 条，
/// [`push_one_floor_raise_within_the_admission_budget`] 的 `Err`）。
#[derive(Debug)]
pub enum FloorRaiseStop {
    /// 再推一串就超过一次准入最多的发布数（[`PUBLISHES_PER_ADMISSION_AT_MOST`]）：这次准入已经推了 `publishes_pushed` 次空发布，
    /// 下一串要推 `publishes_of_the_next_raise` 次（抬 F 那一串的次数），再加这次准入为之判的那一次发布。连一次不抬 F 的空发布都放不下
    /// （已推 + 1 + 1 > B）时也报这一格，不另判上限卡没卡在读坏槽上；放得下一次不抬 F 的空发布、放不下抬 F 那一串、而上限没卡在
    /// 挂载时读坏或读回全零的槽上时同样报这一格。
    PublishesPerAdmissionWouldBeExceeded {
        publishes_pushed: usize,
        publishes_of_the_next_raise: usize,
    },
    /// 上限不高于现行那一版的 F：抬不动（[`RaiseToTheAdmissionCeiling::FloorAlreadyAtTheCeiling`]）。上限卡在挂载时读坏或读回全零的槽上
    /// （不并它们时上限高于 F）不报这一格，推一次不抬 F 的空发布（[`FloorOfThePushedPublishes::KeptWhileTheCeilingIsHeldByRingSlotsBadSinceTheMount`]）。
    FloorAlreadyAtTheCeiling {
        floor: CheckpointTxg,
        ceiling: CheckpointTxg,
    },
    /// 抬 F 那一串自己的空发布、或上限卡在读坏槽上推的那一次不抬 F 的空发布取不到落点、或被空间准入拒
    /// （`mounted_session::refusal_is_short_of_space` 那两种）：这一串推不出空间。
    /// `refusal` 是报的错原样，只会是两个成员之一：`RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite`（抬 F 那一串在预演里被拒，
    /// 一次都没发，盘上逐字节不变、分配器与抬 F 之前逐项相同），或 `RaiseFloorSequencePublishFailed`（抬 F 那一串预演与真发分叉那一格：
    /// 真发时第 n 次在落盘之前被拒，先写系统配置那一步与前面 n − 1 次已落盘、在调用方的现行版本里；不抬 F 的那一次在落盘之前被拒也报它，
    /// 前面没有已落盘的、先写系统配置那一步是空账，发布路径把分配器退回发布之前那一份），两者的 `cause` 都是空间不够那两种之一。
    /// 装箱：不装箱就把 `MountError` 整个摊进这个枚举。
    FloorRaiseRefusedForSpace { refusal: Box<MountError> },
}

/// 抬 F 那一串报的错是不是空间不够那两种（取不到落点、空间准入不够，`mounted_session::refusal_is_short_of_space`）：
/// 预演里被拒，或真发时在落盘之前被拒。是 ⇒ 这一串推不出空间，算推满仍不够（[`FloorRaiseStop::FloorRaiseRefusedForSpace`]）；
/// 别的成员都不是空间不够。只看错，不读写盘。
fn floor_raise_refused_for_space(refusal: &MountError) -> bool {
    match refusal {
        MountError::RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite { cause, .. } => {
            refusal_is_short_of_space(cause)
        }
        MountError::RaiseFloorSequencePublishFailed(failed) => {
            refusal_is_short_of_space(&failed.cause)
        }
        MountError::Recovery(_)
        | MountError::FileVersionWithoutAnyJournalRecord
        | MountError::InstanceTableMalformed
        | MountError::InstanceTableUnreadable
        | MountError::Acquisition(_)
        | MountError::Publish(_)
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
        // txg 到顶不是空间不够；可写挂载那一处抬 F 的错装进的那个成员只由可写挂载交回、抬 F 自己报不出来。
        | MountError::SequenceNumberPastTheTopOfItsRange(_)
        | MountError::FloorRaiseFailedAfterTheMountsPublishes(_)
        // 重读一次仍读不出只由可写挂载的读阶段与重建分配器交回（C554 乙），读阶段之后系统配置又读不出只由可写挂载取号之前与取号交回：
        // 抬 F 报不出来，也不是空间不够。
        | MountError::NewerStateStillUnreadableAfterOneReread(_)
        | MountError::SystemConfigurationUnreadableAfterTheReadStage(_)
        // 重建分配器读根环重读那一遍仍一条有效根都没有只由可写挂载的重建分配器交回：抬 F 报不出来，也不是空间不够。
        | MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread => false,
    }
}

/// 一次准入里再推一串做成了什么：推了（抬 F 那一串，或上限卡在挂载时读坏、读回全零的槽上推的一次不抬 F 的空发布，
/// [`RaisedFloor::floor_of_the_publishes`] 分），或推满了、不再推（[`FloorRaiseStop`]）。推的时候报的别的错不在这里，是
/// [`push_one_floor_raise_within_the_admission_budget`] 的 `Err`。
#[derive(Debug)]
pub enum FloorRaisePushedWithinTheAdmissionBudget {
    Raised(RaisedFloor),
    Stopped(FloorRaiseStop),
}

/// 一次准入里再推一串空发布（D16（发布语义） 已定项 1「准入」「抬 F 的上限」两行）：先核调用方交进来的参数与盘表（照可写挂载同一套；
/// 之后只用盘上那一份参数），再算这个进程这一刻的两个上限（[`AdmissionCeilings`]），按上限与预算分三支：
/// - 上限卡在挂载时读坏或读回全零、之后这个进程没写过的槽上（并进它们之后不高于现行 F、不并时高于现行 F，
///   [`AdmissionCeilings::are_held_by_ring_slots_bad_since_the_mount`]）：推一次不抬 F 的空发布（带现行 F、txg 加一，
///   [`push_one_empty_publish_keeping_the_floor`]），算进这次准入的已推次数，预算按这一次判：已推 + 1 + 这次准入为之判的那一次发布
///   不超过 [`PUBLISHES_PER_ADMISSION_AT_MOST`]（用户 2026-11-02 定「推空」，E173（上限卡在读坏槽的三种处置） 登记的「推空B」那一臂：
///   受 B 限，不取不受 B 限的「推空W」）。推到这个进程写掉那几槽，上限就不再被它们卡住，之后照抬 F 那一串推。
/// - 上限高于现行 F：照今天抬到上限（[`raise_to_the_admission_ceiling_judged`]），预算按抬 F 那一串的次数判
///   （与抬 F 那一串按同一个纯算的函数数，`txgs_of_the_publishes_carrying_the_floor_to_every_device`）。
/// - 上限不高于现行 F、也没卡在那几槽上：不推（`FloorAlreadyAtTheCeiling`）。
///
/// 预算连一次不抬 F 的空发布都放不下时不算上限、照今天报预算用完；放得下一次而放不下抬 F 那一串时算上限，卡在那几槽上就推那一次、
/// 没卡就照今天报预算用完。推满仍不够（预算用完、F 已在上限、推的那一次或那一串自己也被空间不够拒）交回
/// [`FloorRaisePushedWithinTheAdmissionBudget::Stopped`]。
///
/// # Errors
/// 盘表里有身份交了不止一次、调用方的参数或盘表与盘上系统配置不一致（在判预算与任何写之前）；
/// 现行那一版的实例表读不出或解不开、算上限的错（[`rollback_floor_ceiling`] 那几种，在任何写之前；预算放得下一次不抬 F 的空发布才算上限）；
/// 抬 F 自己报的别的错（[`raise_rollback_floor`] 的错，原样；代码审阅第 23 条：块设备错不是空间不够）：
/// 在任何写之前被拒的，盘上与分配器不动；真发时落盘途中失败的（`MountError::RaiseFloorSequencePublishFailed`），
/// 前面已落盘的那几次在调用方的现行版本里；推的那一次不抬 F 的空发布自己报的、不是空间不够的错（同一个成员，见
/// [`push_one_empty_publish_keeping_the_floor`]）。
///
/// # Panics
/// 算抬 F 上限交回 `Recovery(NoValidRoot)`：这是本进程准入路径，调用处三个——可写挂载真做时写行之后与挂载续做里推、
/// 可写挂载取号之前只进内存那一趟预演把同样两处再走一遍时推（[`walk_the_continuation_after_acquisition_on_devices_keeping_writes_in_memory`]，
/// 分配器是这个进程挂载建的那一份的拷贝）、挂着的会话发布被拒时推；分配器都是这个进程挂载建的那一份或它的拷贝，
/// 读的都是这个进程写出的现行根（代码轮 `research/prompts/m3-close-b19-code-r2-main-verification.md` 第二节 U3 补列），
/// 这一错要「这个进程写出的现行根都经记根进了根环表」那条前提破了才走得到（[`rollback_floor_ceilings_of_this_process`] 的两条前提之一，C724
/// （准入抬 F 上限里同名拒因不在单子上））。公开入口 [`raise_rollback_floor_to_the_admission_ceiling`] 收调用方给的分配器，同一错照原样交回。
pub fn push_one_floor_raise_within_the_admission_budget<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    allocator: &mut PoolAllocator,
    current: &mut TransactionOutput,
    shadow_ledger: ShadowLedger,
    publishes_pushed: usize,
) -> Result<FloorRaisePushedWithinTheAdmissionBudget, MountError> {
    let caller_inputs_agreeing_with_the_disk =
        caller_inputs_agreeing_with_the_disk(parameters, devices, allocator, current)?;
    let parameters_of_the_pool = &caller_inputs_agreeing_with_the_disk.parameters_of_the_pool;
    let all_devices: Vec<DeviceIdentity> = devices.iter().map(|(identity, _)| *identity).collect();
    let device_of_txg = |txg: CheckpointTxg| {
        let target = target_for_publish(
            txg,
            parameters_of_the_pool.geometry.root_ring_slots_per_region,
        );
        parameters_of_the_pool.region_devices[usize::try_from(target.region).expect("区域号")]
    };
    let publishes_of_the_next_raise = txgs_of_the_publishes_carrying_the_floor_to_every_device(
        current.root.checkpoint_txg,
        &all_devices,
        &device_of_txg,
    )?
    .len();
    let budget_stop = FloorRaiseStop::PublishesPerAdmissionWouldBeExceeded {
        publishes_pushed,
        publishes_of_the_next_raise,
    };
    // 抬 F 那一串至少一次（每块盘都要落一条带新 F 的根），放不下一次不抬 F 的空发布就一定放不下抬 F 那一串。
    // 两样都数上这次准入为之判的那一次发布（末尾的 + 1）。
    let an_empty_publish_keeping_the_floor_exceeds_the_budget =
        publishes_pushed + 1 + 1 > PUBLISHES_PER_ADMISSION_AT_MOST;
    let some_raise_exceeds_the_budget =
        publishes_pushed + publishes_of_the_next_raise + 1 > PUBLISHES_PER_ADMISSION_AT_MOST;
    if an_empty_publish_keeping_the_floor_exceeds_the_budget {
        return Ok(FloorRaisePushedWithinTheAdmissionBudget::Stopped(
            budget_stop,
        ));
    }
    let ceilings = admission_ceilings_on_caller_inputs_agreeing_with_the_disk(
        &caller_inputs_agreeing_with_the_disk,
        devices,
        allocator,
        current,
    );
    // 本进程准入路径：现行根都经记根进了根环表，算上限的那一遍走不到「一条有效根都没有」（文档「# Panics」）。
    assert!(
        !matches!(
            ceilings,
            Err(MountError::Recovery(RecoveryFailure::NoValidRoot))
        ),
        "这个进程的准入算抬 F 上限读不出一条有效根（现行那一版 txg {:?}）：这个进程写出的现行根没经记根进根环表，它的槽读坏没有重读即拒\
         （D23（journal 的角色与格式） 已定项 14 射程两条前提之一，C724）",
        current.root.checkpoint_txg
    );
    let ceilings = ceilings?;
    let pushed = if ceilings.are_held_by_ring_slots_bad_since_the_mount(current.root.rollback_floor)
    {
        push_one_empty_publish_keeping_the_floor(
            parameters_of_the_pool,
            devices,
            allocator,
            current,
            ceilings,
        )
    } else if !some_raise_exceeds_the_budget {
        raise_to_the_admission_ceiling_judged(
            &caller_inputs_agreeing_with_the_disk,
            devices,
            allocator,
            current,
            shadow_ledger,
            ceilings.not_passing_the_ring_slots_bad_since_the_mount,
        )
    } else {
        return Ok(FloorRaisePushedWithinTheAdmissionBudget::Stopped(
            budget_stop,
        ));
    };
    match pushed {
        Ok(RaiseToTheAdmissionCeiling::Raised(raised)) => {
            Ok(FloorRaisePushedWithinTheAdmissionBudget::Raised(raised))
        }
        Ok(RaiseToTheAdmissionCeiling::FloorAlreadyAtTheCeiling { floor, ceiling }) => {
            Ok(FloorRaisePushedWithinTheAdmissionBudget::Stopped(
                FloorRaiseStop::FloorAlreadyAtTheCeiling { floor, ceiling },
            ))
        }
        Err(refusal) if floor_raise_refused_for_space(&refusal) => {
            Ok(FloorRaisePushedWithinTheAdmissionBudget::Stopped(
                FloorRaiseStop::FloorRaiseRefusedForSpace {
                    refusal: Box::new(refusal),
                },
            ))
        }
        Err(refusal) => Err(refusal),
    }
}

/// 上限卡在挂载时读坏或读回全零的槽上时推的那一次不抬 F 的空发布（D16（发布语义） 已定项 1「抬 F 的上限」那一行，用户 2026-11-02 定「推空」）：
/// 接在现行那一版后面发一次空发布，计划与会话推环、可写挂载暖机同一张（`empty_publish_plan_after_file_version`：带现行 F、txg 加一、
/// 重写固定点单元，不碰用户树、实例表照抄），不写系统配置、不重算影子账与回收门槛。发成了 `current` 换成发出来的那一版，交回
/// `Raised`（[`FloorOfThePushedPublishes::KeptWhileTheCeilingIsHeldByRingSlotsBadSinceTheMount`]）。这一次发布不过准入两判（没有普通分配，
/// D28（挂载期承诺量） 已定项 1），只受「取得到落点」约束；落盘之前被拒（取不到落点在内）时发布路径把分配器退回发布之前那一份、盘上不动。
/// 预算由调用方判过（[`push_one_floor_raise_within_the_admission_budget`]）。
///
/// # Errors
/// 这一次发布被拒或落盘途中失败：`RaiseFloorSequencePublishFailed`，`cause` 是发布路径的错原样，先写系统配置那一步是空账、前面没有已落盘的，
/// 失败账是这一次的写入口记下的（与抬 F 那一串的真发同一个成员、同一个装法，调用方按 `cause` 分空间不够与别的）。
fn push_one_empty_publish_keeping_the_floor<Device: BlockDevice>(
    parameters_of_the_pool: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    allocator: &mut PoolAllocator,
    current: &mut TransactionOutput,
    ceilings: AdmissionCeilings,
) -> Result<RaiseToTheAdmissionCeiling, MountError> {
    let mut pool = PoolWriter::new(parameters_of_the_pool, devices.as_mut_slice());
    let published = publish_version(
        &mut pool,
        allocator,
        empty_publish_plan_after_file_version(current, current.record.instance),
        Some(&*current),
    )
    .map_err(|cause| {
        MountError::RaiseFloorSequencePublishFailed(publish_sequence_failed(
            &pool,
            WritesByStructureKind::NOTHING_WRITTEN,
            Vec::new(),
            cause,
        ))
    })?;
    assert_eq!(
        published.root.rollback_floor, current.root.rollback_floor,
        "不抬 F 的空发布带的 F 就是现行那一版的 F：计划照抄现行那一版的根带的 F"
    );
    *current = published.clone();
    Ok(RaiseToTheAdmissionCeiling::Raised(RaisedFloor {
        ceiling: ceilings.not_passing_the_ring_slots_bad_since_the_mount,
        system_configuration_writes_before_the_first_publish:
            WritesByStructureKind::NOTHING_WRITTEN,
        publishes: vec![published],
        floor_of_the_publishes:
            FloorOfThePushedPublishes::KeptWhileTheCeilingIsHeldByRingSlotsBadSinceTheMount {
                ceiling_from_the_roots_read: ceilings.from_the_roots_read,
            },
        reclaimed: Vec::new(),
        abandoned_roots_unreadable: 0,
    }))
}

#[cfg(test)]
mod newest_valid_root_on_every_device_tests {
    use super::newest_valid_root_txg_on_every_device;
    use crate::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
    use crate::instance_table::{InstanceRow, InstanceTableRecords};
    use crate::pointer::NodePointer;
    use crate::root_record::{RootRecord, UnmountMarker};

    fn root_of(instance: u32, checkpoint_txg: u64) -> RootRecord {
        RootRecord {
            filesystem_identifier: [0; 16],
            unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,
            instance: InstanceGeneration(instance),
            checkpoint_txg: CheckpointTxg(checkpoint_txg),
            tree_table: NodePointer::empty_root(),
            tree_identifier_watermark: 0,
            rollback_floor: CheckpointTxg(0),
            instance_table: NodePointer::empty_root(),
            mapping_root: NodePointer::empty_root(),
            allocation_record_tree_root: NodePointer::empty_root(),
            reclaim_threshold: crate::address::CheckpointTxg(0),
            pinned_ring_slot: crate::root_record::PinnedRingSlot::NO_SLOT_PINNED,
        }
    }

    /// 这几条用例的落点公式：偶数 txg 落盘 0、奇数落盘 1（只为把每块盘上有哪几条根摆清楚，不是第一版几何）。
    fn device_of_an_even_or_odd_txg(checkpoint_txg: CheckpointTxg) -> DeviceIdentity {
        if checkpoint_txg.0.is_multiple_of(2) {
            DeviceIdentity(0)
        } else {
            DeviceIdentity(1)
        }
    }

    /// 实例 1 择过 txg 8（实例表那一行），它之后的 txg 9 被抛弃；实例 2 从 txg 12 起。盘 1 上最新的根是被抛弃的 txg 9，
    /// 最新的有效根是 txg 7：取最小时按 7 算（滤掉被抛弃的根），不按 9 算。只有被抛弃的根的那块盘算「一条有效根都没有」。
    /// 判别力：不滤被抛弃的根时第一条断言得 9、第二条得 Some(9)。
    #[test]
    fn an_abandoned_root_newest_on_a_device_does_not_stand_for_that_devices_newest_valid_root() {
        let table = InstanceTableRecords {
            rows: vec![InstanceRow {
                instance: InstanceGeneration(1),
                selected_root_txg: CheckpointTxg(8),
                applied_transaction_high_water: 0,
            }],
        };
        let both_devices = [DeviceIdentity(0), DeviceIdentity(1)];
        let on_their_devices = |roots: &[RootRecord]| -> Vec<(DeviceIdentity, RootRecord)> {
            roots
                .iter()
                .map(|root| (device_of_an_even_or_odd_txg(root.checkpoint_txg), *root))
                .collect()
        };
        let roots = [root_of(1, 7), root_of(1, 8), root_of(1, 9), root_of(2, 12)];
        assert_eq!(
            newest_valid_root_txg_on_every_device(&on_their_devices(&roots), &table, &both_devices,),
            Some(CheckpointTxg(7)),
            "盘 0 上最新的有效根 txg 12、盘 1 上 txg 7（txg 9 被抛弃、不算），取最小是 7"
        );
        let only_the_abandoned_root_on_device_one = [root_of(1, 8), root_of(1, 9), root_of(2, 12)];
        assert_eq!(
            newest_valid_root_txg_on_every_device(
                &on_their_devices(&only_the_abandoned_root_on_device_one),
                &table,
                &both_devices,
            ),
            None,
            "盘 1 上只有被抛弃的 txg 9：那块盘一条有效根都没有"
        );
    }
}

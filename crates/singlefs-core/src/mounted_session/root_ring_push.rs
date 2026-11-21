//! 被宽护卡在根环上限时把根环推到它够得着的最后一槽：预演推环与推完剩下的改动、宽护的出路
//! （[`StuckBehindTheWideProtection`]、`MountedSession::ways_out_of_the_wide_protection`）。

use super::*;

/// 推满仍不够时，挡着准入的是不是「这个进程写掉那一槽才放」的那一份隔离（用户 2026-10-08 弹窗定「显式推环命令加带出路的错」：
/// 不自动推环，准入因宽护卡住时报一个点名「被宽护卡住」的错、带上出路；用户 2026-10-09 弹窗定「甲：预演推环与准入，推不通就不列」：
/// 推环那一条出路预演过、推完这次改动放得下才列；同日弹窗定「改：预演延到要出路时才做」：报的时候不预演，调用方要出路时另调
/// [`MountedSession::ways_out_of_the_wide_protection`] 算，只要错误码、不看出路的调用方不付这一笔）。
///
/// 为什么不另开一个 [`UserChangeRefused`] 成员：验证包的装置（`e158_root_choice_repair`、`e168_delete_ring_window_borrow`）对
/// `UserChangeRefused` 与 `FloorRaiseStop` 写的是穷举 `match`，多一个成员它们就编不过，而验证包这一批不改（第 10z 批第十六块派发）。
/// 点名这一形的成员住在这里，外层照旧是 `NoSpaceAfterRaisingTheFloor`，交回的已落盘的东西不变。
#[derive(Debug)]
pub enum WideProtectionAtTheCeiling {
    /// 不是这一形：不再推的原因不是「F 已在上限」（预算用完、抬 F 那一串自己被拒），或 C 那一圈这一刻一个槽都没隔离。
    NotWhatStopsTheAdmission,
    /// 被宽护卡住（[`StuckBehindTheWideProtection`]）。装箱同 `NoSpaceAfterRaisingTheFloor`。
    StuckBehindTheWideProtection(Box<StuckBehindTheWideProtection>),
}

/// 被宽护卡住：准入被拒、会话推抬 F 停在上限（`FloorRaiseStop::FloorAlreadyAtTheCeiling`），而 C 那一圈（水位之上没施加的记录点名的落点、
/// 那几次发布的账、比所选那一版新的可读根的账；认不出回退时并进 R 那一槽的「现行账里全部已释放的落点」就是宽护，它是这一圈里最大的一份）
/// 还隔离着槽。这一圈只在这个进程写掉那一槽时解开（`allocator::PoolAllocator::record_root_written_by_this_process`），写被拒就不发布、
/// 那一槽写不掉，不管它就隔离到进程结束（第 10z 批第十四块实现员报告第六节，384 槽盘 22 形写全被拒）。
///
/// 这里分不出 C 那一圈里哪一份是宽护、哪一份是窄的那几份：分得出要在根环表里另记一样（`PoolAllocator` 的 `{:?}` 随之变，验证包 E171
/// 记它），这一份不记；出路对整圈算。
///
/// **宽护是挡着准入的之一，不是唯一**：同一刻没有 C 那一圈，准入照样可以被拒（第 10z 批代码轮第一轮攻方
/// `research/prompts/m3-batch10z-pqrs-code-r1-opus-output.md` 第一节：同一个池停在 R 之前、C 一槽都没有的对照臂，同一个长度也被拒的 27 格，
/// 这里照样报这一形）。所以报这一形不说明写掉 R 那一槽之后放得下：推完放不放得下由调用方要出路时那一份预演说
/// （[`MountedSession::ways_out_of_the_wide_protection`]），放不下（推完上限退到环里最旧的根、推环自己换下的那几份要等环再转一圈才回收，
/// 同一节「读法」）就不列推环。
///
/// 报这一形只读分配器（内存）：不拷分配器、不预演、一个盘都不读。预演要的「被拒在哪一版上」「这次改动还剩什么」记在这里（私有，
/// 只有会话造得出这一份），这次改动的内容拷一份进来（错误不借调用方的内容）。
#[derive(Debug)]
pub struct StuckBehindTheWideProtection {
    /// 推环要写到的那一槽：C 那一圈还没解开、记着落点的槽里，从现行那一版起按 txg 往后最后一个轮到的（R 那一槽）。
    pub last_ring_slot_the_push_reaches: RootRingSlot,
    /// R 的 txg：根环表里那一槽记着的有效根（挂载那一刻按最新那条根的实例表有效、记成有效根，写行之后才被抛弃的那一条）；
    /// 那一槽记的不是有效根（读坏、读回全零、没有根的记录）时 `None`。
    pub checkpoint_txg_of_the_last_ring_slot_the_push_reaches: Option<CheckpointTxg>,
    /// 从现行那一版起要发几次 F 不变的空发布才写掉 R 那一槽（不超过环长 3S）。
    pub empty_publishes_until_the_last_ring_slot_the_push_reaches_is_written: u64,
    /// C 那一圈每块盘上这一刻隔离着的槽数（按分配器里盘的次序，同一个槽只数一次）：宽护挡着的槽数。只数 C 那一圈罩着的，
    /// 被抛弃根罩着、C 不罩的不在其内。
    pub isolated_slots_on_each_device: Vec<(DeviceIdentity, u64)>,
    /// 被拒的那一次要接在后面的那一版（报这一形那一刻会话的现行那一版）的实例代号与 txg：要出路时会话不在这一版上就不预演
    /// （[`WaysOutOfTheWideProtectionRefused::SessionMovedPastTheVersionRefusedOnNotSupportedInTheFirstVersion`]）。报里不记是哪个会话报的：
    /// 别的会话的报这两个数恰好相同时不拦（同一成员的文档）。
    instance_of_the_version_refused_on: InstanceGeneration,
    checkpoint_txg_of_the_version_refused_on: CheckpointTxg,
    /// 被拒的那一次之后这次用户改动还剩什么要发（内容拷一份）：要出路时推完环照它再预演一次。
    remainder_of_the_change: RemainderOfTheUserChangeKeptByTheRefusal,
}

/// 调用方要出路时预演推环与推完之后这次改动的准入（用户 2026-10-09 弹窗定「甲：预演推环与准入，推不通就不列」「改：预演延到要出路时才做」）：
/// 在分配器的一份拷贝上、对只进内存的盘走一遍推环入口与推完之后这次改动剩下的发布（`rehearse_the_push_then_the_remainder_of_the_change`），
/// 一个写都没发到盘上，会话不动。
#[derive(Debug)]
pub enum RootRingPushRehearsal {
    /// 推环做成，推完之后报里剩下的那一段放得下（还剩这次改动时是走空再发它；只剩走空时是那一段走空，unlink 的意图已发成）：出路列推环。
    /// `isolated_slots_released_by_the_push_on_each_device` 是推完之后每块盘隔离的槽数
    /// 比推之前少了几个（按分配器里盘的次序；推完反而多的那块盘记 0）：C 那一圈与被抛弃根罩着同一个槽时，被抛弃根还在环里那一槽就不放，
    /// 推环途中写掉被抛弃根那一槽时它独占的也放出来——与 `isolated_slots_on_each_device` 不是同一个数。
    ChangeAdmittedAfterThePush {
        isolated_slots_released_by_the_push_on_each_device: Vec<(DeviceIdentity, u64)>,
    },
    /// 推环做成，推完之后这次改动照旧被拒（`refusal` 是预演里交回的那一种，例如推完上限退到环里最旧的根、推满仍不够）：出路不列推环。
    ChangeStillRefusedAfterThePush {
        isolated_slots_released_by_the_push_on_each_device: Vec<(DeviceIdentity, u64)>,
        refusal: Box<UserChangeRefused>,
    },
    /// 推环自己在预演里被拒（`RootRingPushRefused` 的哪一种原样）：出路不列推环。
    PushRefused(Box<RootRingPushRefused>),
}

/// 调用方要出路时算出来的（[`MountedSession::ways_out_of_the_wide_protection`]）：那一刻的预演结局与照它列的出路。
#[derive(Debug)]
pub struct WaysOutOfTheWideProtection {
    /// 在分配器的拷贝与只进内存的盘上预演推环、推完再预演这次改动（连同它会推的抬 F）的结局：推环这条出路列不列按它定。
    pub pushing_the_root_ring: RootRingPushRehearsal,
    /// 出路：卸载重挂一定列；推环（会丢掉最近可退到的状态，调用即确认）只在预演过推完这次改动放得下时列
    /// （[`RootRingPushRehearsal::ChangeAdmittedAfterThePush`]）。按这个次序。报给用户的文字由调用方照每一条带的数写。
    pub ways_out: Vec<WayOutOfTheWideProtection>,
}

/// 被宽护卡住时的一条出路（[`WaysOutOfTheWideProtection::ways_out`]）：说清做什么、做了会怎样要的数，文字由调用方写。封闭集合。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WayOutOfTheWideProtection {
    /// 正常卸载（`mount::unmount`）之后再可写挂载一次。`checkpoint_txg_of_the_root_in_that_ring_slot` 是宽护挡着的那一槽（R 那一槽）记着的有效根的 txg：有它时再挂按新实例的实例表
    /// 判它被抛弃，影子账只护它自己账里引用的落点，宽护不再出现；它的账读不出若是盘上永久坏，卸载之后再挂会被拒（C694 的 B），而会话在卸载时
    /// 已断开。`None`：R 那一槽（`ring_slot_held_by_the_wide_protection`）没有读得出的有效根，再挂时择的是卸载推出来的最后一版，R 那几条记录不在它的水位之上，
    /// C 那一圈照它重算、不再有这一槽，宽护不再出现。
    UnmountThenMountWritableAgain {
        ring_slot_held_by_the_wide_protection: RootRingSlot,
        checkpoint_txg_of_the_root_in_that_ring_slot: Option<CheckpointTxg>,
    },
    /// 调 [`MountedSession::push_the_root_ring_to_the_last_ring_slot_it_reaches_dropping_rollback_candidates_below_four_states`]：从现行那一版起发
    /// `empty_publishes_until_that_ring_slot_is_written` 次 F 不变的空发布，写掉根环里接下来轮到的同样多槽，R 那一槽（`ring_slot_held_by_the_wide_protection`）
    /// 是最后一槽；宽护随之解开，预演里每块盘放出来的隔离槽数是 `isolated_slots_released_on_each_device`。这几槽里原来的根换成同一个用户状态的
    /// 空发布根，回退候选集可以掉到少于 4 个不同状态（D16（发布语义） 已定项 1），调用即确认。推完之后做什么见 `after_the_push`。
    PushTheRootRingUpToTheRingSlotHeldByTheWideProtection {
        ring_slot_held_by_the_wide_protection: RootRingSlot,
        empty_publishes_until_that_ring_slot_is_written: u64,
        isolated_slots_released_on_each_device: Vec<(DeviceIdentity, u64)>,
        after_the_push: WhatIsLeftAfterThePush,
    },
}

/// 推环那一条出路推完之后还剩什么（按报里剩下什么分）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WhatIsLeftAfterThePush {
    /// 报里还剩这次改动：预演过推完之后这次改动放得下，调用方推完再发它。
    PublishTheChangeAgain,
    /// 这次 unlink 的意图已发成，报里只剩它之后的走空：预演过推完之后剩下的走空在下一次改动开头走、放得下；不再发这次改动
    /// （照「再发这次改动」再发，开头的走空走完、要 unlink 的文件已不在，被拒）。
    WalkTheQueueAtTheStartOfTheNextChange,
}

/// 要出路时没预演。成员按调用方要做的决定分：再发一次这次改动拿新的报、换回挂载时那一份盘表。
/// 两种都在预演之前返回：一个盘都没读、一个写都没发，会话不动。
#[derive(Debug)]
pub enum WaysOutOfTheWideProtectionRefused {
    /// 会话的现行那一版已不是报「被宽护卡住」时被拒的那一次要接在后面的那一版（之后又发过：推过环、别的改动发成、走空走了几步）：
    /// 报里的 R 那一槽、要推的次数与挡着的槽数说的是那一刻，照这一刻预演会与它们对不上，按哪一刻算条款没写，第一版不支持。
    /// 调用方要做的是再发一次这次改动、拿新的那一份报再要出路。这一道在盘表那一道之前：会话已不在那一版上、交进来的盘表又与挂载时的不同，
    /// 报的是这一种。同一个池卸载或崩了之后再挂取新实例，现行那一版的实例代号随之变，再挂之前的报在这一道拒成这一种
    /// （`instance_of_the_current_version` 是新实例的）。
    ///
    /// 这一道只比实例代号与 txg，报里不记是哪个会话报的。别的会话（同一份 mkfs 起的另一个池、读盘造的另一条会话）
    /// 的报若这两个数恰好与这个会话的现行那一版相同，这里不拦：照这个会话的状态（分配器、盘、现行那一版）预演，预演发的是报里拷下的那一份改动
    /// （不是这个会话自己要发的），卸载重挂那一条带的仍是报里的数
    /// （R 的 txg 或 R 那一槽），列推环那一条时其中的 R 那一槽与要推的次数也是报里的（第 10z 批代码轮攻方「打中一」；用户 2026-10-09 定撤掉会话身份、照实写）。
    /// 调用方要拿报去问报它的那个会话。
    SessionMovedPastTheVersionRefusedOnNotSupportedInTheFirstVersion {
        instance_refused_on: InstanceGeneration,
        checkpoint_txg_refused_on: CheckpointTxg,
        instance_of_the_current_version: InstanceGeneration,
        checkpoint_txg_of_the_current_version: CheckpointTxg,
    },
    /// 会话还在被拒的那一版上（实例代号与 txg 与报里记的相同），交进来的盘表的设备身份（按次序）却与这次挂载核过的那一份不同（多一块、少一块、换了次序、
    /// 同一个身份交两次；与 [`UserChangeRefused::DeviceTableOtherThanTheOneOfTheMount`] 同一判，`mount::ParametersAndDeviceTableOfTheMount::is_the_device_table_of_the_mount`）。
    /// 预演经推环的本体走（`MountedSession::push_the_root_ring_on`，交的是包过的盘），本体里核盘表那一套不看次序：不先核这一判时，
    /// 次序对调的盘表预演放得下、列推环，照出路二推完再发这次改动却拒成 `DeviceTableOtherThanTheOneOfTheMount`（第 10z 批代码轮攻方「打中二」）。
    /// 推环公开入口与 [`MountedSession::publish_user_change`] 先核同一判（[`RootRingPushRefused::DeviceTableOtherThanTheOneOfTheMount`]）：
    /// 三个入口收盘表同一判。调用方要做的是换回挂载时那一份盘表再要出路。
    DeviceTableOtherThanTheOneOfTheMount {
        device_table_of_the_mount: Vec<DeviceIdentity>,
        device_table_handed_in: Vec<DeviceIdentity>,
    },
}

impl StuckBehindTheWideProtection {
    /// 照这一份报与要出路时那一份预演列出路：卸载重挂一定列；推环只在预演过推完这次改动放得下时列。按这个次序。
    /// 推环那一条推完之后还剩什么按报里剩下什么分：还剩这次改动时是推完再发它；只剩走空时是意图已发成、剩下的走空在下一次改动开头走、
    /// 不再发这次改动。
    fn ways_out_after(
        &self,
        pushing_the_root_ring: &RootRingPushRehearsal,
    ) -> Vec<WayOutOfTheWideProtection> {
        let mut ways_out = vec![WayOutOfTheWideProtection::UnmountThenMountWritableAgain {
            ring_slot_held_by_the_wide_protection: self.last_ring_slot_the_push_reaches,
            checkpoint_txg_of_the_root_in_that_ring_slot: self
                .checkpoint_txg_of_the_last_ring_slot_the_push_reaches,
        }];
        match pushing_the_root_ring {
            RootRingPushRehearsal::ChangeAdmittedAfterThePush {
                isolated_slots_released_by_the_push_on_each_device,
            } => {
                let after_the_push = match &self.remainder_of_the_change {
                    RemainderOfTheUserChangeKeptByTheRefusal::WalkTheQueueThenPublishTheChange(
                        _,
                    ) => WhatIsLeftAfterThePush::PublishTheChangeAgain,
                    RemainderOfTheUserChangeKeptByTheRefusal::WalkTheQueue => {
                        WhatIsLeftAfterThePush::WalkTheQueueAtTheStartOfTheNextChange
                    }
                };
                ways_out.push(WayOutOfTheWideProtection::PushTheRootRingUpToTheRingSlotHeldByTheWideProtection {
                    ring_slot_held_by_the_wide_protection: self.last_ring_slot_the_push_reaches,
                    empty_publishes_until_that_ring_slot_is_written: self
                        .empty_publishes_until_the_last_ring_slot_the_push_reaches_is_written,
                    isolated_slots_released_on_each_device:
                        isolated_slots_released_by_the_push_on_each_device.clone(),
                    after_the_push,
                });
            }
            RootRingPushRehearsal::ChangeStillRefusedAfterThePush { .. }
            | RootRingPushRehearsal::PushRefused(_) => {}
        }
        ways_out
    }
}

/// 每块盘上这一刻隔离的槽数（D28（挂载期承诺量） 已定项 1 第九项那一项），按分配器里盘的次序。只读内存。
fn isolated_slots_of_each_device(allocator: &PoolAllocator) -> Vec<(DeviceIdentity, u64)> {
    allocator
        .devices
        .iter()
        .map(|device_map| (device_map.device, device_map.isolated_slots()))
        .collect()
}

/// 要出路时的预演（用户 2026-10-09 弹窗定「甲：预演推环与准入，推不通就不列」「改：预演延到要出路时才做」，
/// [`MountedSession::ways_out_of_the_wide_protection`]）：在分配器的一份拷贝上、对只进内存的盘
/// （[`DeviceKeepingWritesInMemory`]）先走一遍推环入口（与真调推环入口同一段，[`MountedSession::push_the_root_ring_on`]：它自己的整串预演、
/// 真发与末尾的断言都在），推完再把这次改动剩下的照会话发布那一段走一遍（`remainder`：准入被拒照样推抬 F，抬 F 的上限按推完的根环算——
/// 读的是盖着推环那几次写的盘）。读盘照读底下那几块盘；写一个都不发到底下那几块盘；会话的分配器与现行那一版不动。
///
/// 预演里的准入再被拒时报不报「被宽护卡住」只读内存（[`wide_protection_at_the_ceiling`] 不预演）；推环做成之后 C 那一圈一槽都不记着
/// （推环入口末尾那条断言），报的是 `NotWhatStopsTheAdmission`。推环在预演里被拒就不往下走。
fn rehearse_the_push_then_the_remainder_of_the_change<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &[(DeviceIdentity, Device)],
    allocator: &PoolAllocator,
    current: &TransactionOutput,
    instance: InstanceGeneration,
    shadow_ledger: ShadowLedger,
    remainder: RemainderOfTheUserChange<'_>,
) -> RootRingPushRehearsal {
    let mut devices_keeping_writes_in_memory: Vec<(
        DeviceIdentity,
        DeviceKeepingWritesInMemory<'_>,
    )> = devices
        .iter()
        .map(|(identity, device)| {
            let disk: &dyn BlockDevice = device;
            (*identity, DeviceKeepingWritesInMemory::over_the_disk(disk))
        })
        .collect();
    let mut rehearsed_allocator = allocator.clone();
    let mut rehearsed_version = PoolVersion::of_a_version_of_trees(current.clone());
    let isolated_before_the_push = isolated_slots_of_each_device(&rehearsed_allocator);
    if let Err(refusal) = MountedSession::push_the_root_ring_on(
        parameters,
        &mut devices_keeping_writes_in_memory,
        &mut rehearsed_allocator,
        &mut rehearsed_version,
        instance,
    ) {
        return RootRingPushRehearsal::PushRefused(Box::new(refusal));
    }
    let isolated_slots_released_by_the_push_on_each_device: Vec<(DeviceIdentity, u64)> =
        isolated_before_the_push
            .iter()
            .zip(isolated_slots_of_each_device(&rehearsed_allocator))
            .map(|((device, before), (device_after, after))| {
                assert_eq!(
                    *device, device_after,
                    "推环前后数的是同一个分配器的拷贝：盘的次序不变"
                );
                (*device, before.saturating_sub(after))
            })
            .collect();
    let remainder_after_the_push = match remainder {
        RemainderOfTheUserChange::WalkTheQueueThenPublishTheChange(change) => {
            // 预演里读回的那几次不并进这次挂载的累计：预演不是一次真改动（会话的分配器与现行那一版都不动）。
            MountedSession::publish_the_change_raising_the_floor_for_space(
                parameters,
                &mut devices_keeping_writes_in_memory,
                &mut rehearsed_allocator,
                &mut rehearsed_version,
                instance,
                shadow_ledger,
                change,
                &mut OldDataUnitReadsBackWithOneCopyFailingInThisChange::nothing_read_yet(),
            )
            .map(|_published| ())
        }
        RemainderOfTheUserChange::WalkTheQueue => {
            let mut version_to_walk = rehearsed_version.into_shared_version_of_trees().expect(
                "推环接在有树的一版后面、发的是带文件那一版的空发布（树表 0 条那一版推环入口先拒了），推完还是有树的一版",
            );
            publish_the_queue_until_it_is_empty_raising_the_floor_for_space(
                parameters,
                &mut devices_keeping_writes_in_memory,
                &mut rehearsed_allocator,
                &mut version_to_walk,
                instance,
                shadow_ledger,
                RemainderOfTheUserChange::WalkTheQueue,
                PublishedInThisChange {
                    floor_raises: Vec::new(),
                    refusals_that_pushed_the_floor_raises: Vec::new(),
                    versions_published: Vec::new(),
                },
            )
            .map(|_published_in_this_change| ())
        }
    };
    match remainder_after_the_push {
        Ok(()) => RootRingPushRehearsal::ChangeAdmittedAfterThePush {
            isolated_slots_released_by_the_push_on_each_device,
        },
        Err(refusal) => RootRingPushRehearsal::ChangeStillRefusedAfterThePush {
            isolated_slots_released_by_the_push_on_each_device,
            refusal: Box::new(refusal),
        },
    }
}

/// C 那一圈还没解开、记着落点的那几槽，与从 `current_txg` 起推环要推到哪儿：推环入口（[`MountedSession::push_the_root_ring_to_the_last_ring_slot_it_reaches_dropping_rollback_candidates_below_four_states`]）
/// 与「被宽护卡住」那一报（[`wide_protection_at_the_ceiling`]）读同一份。
struct RootRingPushTarget {
    /// 这几槽，按槽的次序。
    ring_slots: Vec<RootRingSlot>,
    /// 这几槽记着的全部落点。
    placements: Vec<PlacementOnDevice>,
    /// 最后一个轮到的那一槽（R 那一槽）与写掉它要发几次空发布。
    last_ring_slot_the_push_reaches: RootRingSlot,
    publishes_until_the_last_ring_slot_the_push_reaches_is_written: u64,
}

/// [`RootRingPushTarget`]：分配器没装根环表、或 C 那一圈这一刻没有记着落点的槽时 `None`。只读内存。
///
/// 每一槽要几次发布：txg 逐个加一、每次发布写 `target_for_publish(txg)` 那一槽（`root_ring::target_for_publish`），环长 3S 个连续的 txg
/// 里每一槽恰好轮到一次，所以从 `current_txg + 1` 起数到 3S 之内必找到；txg 加上去越过 u64 的那几个当找不到，取满一环 3S
/// （真发时那一次发布自己就按 txg 到顶拒，`PublishError::NextCheckpointTxgPastTheTopOfItsRange`，在推环的预演里先报出来）。
fn root_ring_push_target(
    allocator: &PoolAllocator,
    current_txg: CheckpointTxg,
) -> Option<RootRingPushTarget> {
    let occupancy = allocator.root_ring_occupancy()?;
    let holding = occupancy.ring_slots_holding_placements_named_by_unapplied_journal_records();
    let slots_per_region = occupancy.slots_per_region();
    let ring_length_in_txgs = ROOT_RING_REGIONS * slots_per_region.count();
    let publishes_until_written = |ring_slot: RootRingSlot| -> u64 {
        (1..=ring_length_in_txgs)
            .find(|publishes_ahead| {
                current_txg
                    .0
                    .checked_add(*publishes_ahead)
                    .is_some_and(|txg| {
                        target_for_publish(CheckpointTxg(txg), slots_per_region) == ring_slot
                    })
            })
            .unwrap_or(ring_length_in_txgs)
    };
    let (
        publishes_until_the_last_ring_slot_the_push_reaches_is_written,
        last_ring_slot_the_push_reaches,
    ) = holding
        .keys()
        .map(|ring_slot| (publishes_until_written(*ring_slot), *ring_slot))
        .max()?;
    Some(RootRingPushTarget {
        ring_slots: holding.keys().copied().collect(),
        placements: holding.into_values().flatten().collect(),
        last_ring_slot_the_push_reaches,
        publishes_until_the_last_ring_slot_the_push_reaches_is_written,
    })
}

/// 推满仍不够时报不报「被宽护卡住」（[`WideProtectionAtTheCeiling`]）：`stop` 是 F 已在上限、C 那一圈还有记着落点的槽、它们这一刻至少隔离着一个槽，
/// 才报；别的都照旧。判这三样与报的那几个数只读分配器（内存）。报的时候不预演（用户 2026-10-09 弹窗定「改：预演延到要出路时才做」）：
/// 只记下被拒在哪一版上（`current`）与这次改动还剩什么（`remainder`，内容拷一份），调用方要出路时
/// [`MountedSession::ways_out_of_the_wide_protection`] 照它们预演。
pub(super) fn wide_protection_at_the_ceiling(
    allocator: &PoolAllocator,
    current: &TransactionOutput,
    remainder: RemainderOfTheUserChange<'_>,
    stop: &FloorRaiseStop,
) -> WideProtectionAtTheCeiling {
    match stop {
        FloorRaiseStop::FloorAlreadyAtTheCeiling { .. } => {}
        FloorRaiseStop::PublishesPerAdmissionWouldBeExceeded { .. }
        | FloorRaiseStop::FloorRaiseRefusedForSpace { .. } => {
            return WideProtectionAtTheCeiling::NotWhatStopsTheAdmission;
        }
    }
    let Some(target) = root_ring_push_target(allocator, current.root.checkpoint_txg) else {
        return WideProtectionAtTheCeiling::NotWhatStopsTheAdmission;
    };
    let isolated_slots_on_each_device = allocator.isolated_slots_covered_by(&target.placements);
    if isolated_slots_on_each_device
        .iter()
        .all(|(_, isolated)| *isolated == 0)
    {
        return WideProtectionAtTheCeiling::NotWhatStopsTheAdmission;
    }
    let checkpoint_txg_of_the_last_ring_slot_the_push_reaches =
        allocator.root_ring_occupancy().and_then(|occupancy| {
            occupancy.checkpoint_txg_of_the_valid_root_recorded_on(
                target.last_ring_slot_the_push_reaches,
            )
        });
    WideProtectionAtTheCeiling::StuckBehindTheWideProtection(Box::new(
        StuckBehindTheWideProtection {
            last_ring_slot_the_push_reaches: target.last_ring_slot_the_push_reaches,
            checkpoint_txg_of_the_last_ring_slot_the_push_reaches,
            empty_publishes_until_the_last_ring_slot_the_push_reaches_is_written: target
                .publishes_until_the_last_ring_slot_the_push_reaches_is_written,
            isolated_slots_on_each_device,
            instance_of_the_version_refused_on: current.root.instance,
            checkpoint_txg_of_the_version_refused_on: current.root.checkpoint_txg,
            remainder_of_the_change: RemainderOfTheUserChangeKeptByTheRefusal::kept_from(remainder),
        },
    ))
}

/// 推环入口（[`MountedSession::push_the_root_ring_to_the_last_ring_slot_it_reaches_dropping_rollback_candidates_below_four_states`]）做成了：
/// 发成的每一版按先后（最后一版是会话的 `current`），与写掉的那几槽（C 那一圈在这几槽记着的落点都已解开）。
#[derive(Debug)]
pub struct RootRingPushed {
    /// 最后一版与会话的 `current` 指着同一份（[`VersionsPublishedInThisChange`] 那样共享）。
    pub versions_published: Vec<Arc<TransactionOutput>>,
    pub ring_slots_written: Vec<RootRingSlot>,
}

/// 推环入口没做成。成员按调用方要做的决定分：换回挂载时那一份盘表、等冻结着的那次发布重发、换盘表或换回这个池的盘、这一形第一版不支持、不必推、
/// 空间不够推不过去、发布路径的错。
#[derive(Debug)]
pub enum RootRingPushRefused {
    /// 交进来的盘表的设备身份（按次序）与这次挂载核过的那一份不同（多一块、少一块、换了次序、同一个身份交两次）：入口第一道，
    /// 与 [`UserChangeRefused::DeviceTableOtherThanTheOneOfTheMount`]、[`WaysOutOfTheWideProtectionRefused::DeviceTableOtherThanTheOneOfTheMount`]
    /// 同一判（`mount::ParametersAndDeviceTableOfTheMount::is_the_device_table_of_the_mount`，只比身份），三个入口收盘表同一判
    /// （用户 2026-10-10 弹窗定「也先比次序」）。不核这一判时次序对调的盘表照样推成（写的先后跟着盘表次序变），推完照它再发被拒成盘表那一种，
    /// 三个入口收不收同一份盘表不一致（判决 `research/prompts/m3-batch10z-xy-code-r1-main-verification.md` 第二节 K1-2）。
    /// 在任何读写之前返回：物理块大小没探、一个盘都没读、一个写都没发，盘上逐字节不变、分配器与会话不动。调用方要做的是换回挂载时那一份盘表再推。
    DeviceTableOtherThanTheOneOfTheMount {
        device_table_of_the_mount: Vec<DeviceIdentity>,
        device_table_handed_in: Vec<DeviceIdentity>,
    },
    /// 分配器上冻结着一次落盘途中失败、还没原样重发的发布（D23（journal 的角色与格式） 已定项 14「这一版的失败处置」；它的 txg 与实例代号）：
    /// 推环的第一次空发布本来就会被拒（`PublishError::PublishFrozenAfterAWriteFailureIsNotResentYet`），而整串预演走的是发布路径落盘之前那一段、
    /// 不判冻结，会在冻结着的分配器拷贝上照样过。比过盘表次序（只比身份、不读盘）之后就拒，在核盘表那一套、预演与任何读写之前：
    /// 盘上逐字节不变、分配器与会话不动。调用方要做的是先让那次发布原样重发（会话这一层没有重发的入口，等进程重开）。
    PublishFrozenAfterAWriteFailureIsNotResentYet {
        checkpoint_txg: CheckpointTxg,
        instance: InstanceGeneration,
    },
    /// 交进来的盘表没过挂着之后收盘表的那一套核（与抬 F 那一串的空发布同一套：身份交了不止一次、参数或盘表与盘上择到的系统配置不一致、
    /// 有盘不「可见」、有盘落后于现行那一版又缺它的单元或它的根；系统配置择不出来），原样带着那一种 `MountError`。
    /// 身份与次序已由入口第一道比过（`DeviceTableOtherThanTheOneOfTheMount`；要出路时的预演由要出路那一道比过），到这里的盘表身份与挂载时那一份
    /// 逐项相同，身份交了不止一次那一支走不到；
    /// 盘体对调（身份照旧、盘上的本盘设备号对不上）这类读盘才看得出的落在这里。一个写都没发，盘上逐字节不变、会话不动。
    DevicesHandedIn(Box<MountError>),
    /// 现行那一版树表 0 条：推环的空发布在这一版上是零单元发布（D16（发布语义） 已定项 9），会话层的推环在这一形上条款没写，第一版不支持。
    /// 会话在这一版上也不会被宽护卡住（用户改动在这一版上一律 `NoFileVersionToChange`）。在任何读写之前返回，盘上逐字节不变、会话不动。
    VersionWithoutFileNotSupportedInTheFirstVersion,
    /// C 那一圈这一刻没有记着落点的槽，或记着落点而它们罩着的槽这一刻一个都没隔离（全在现行账里仍分配、或已释放还在 defer 窗口里）：
    /// 这个进程写掉哪一槽都放不出东西，不推。一个写都没发，盘上逐字节不变、会话不动。
    NothingHeldUntilThisProcessWritesARingSlot,
    /// 整串空发布在任何写之前的预演里（分配器的一份拷贝上走发布路径落盘之前那一段）第 `refused_publish_in_the_push` 次（从 1 数）被拒：
    /// 推到半路空发布取不到落点（`PublishError::PlacementRefused`）就是这里。一个写都没发，盘上逐字节不变、分配器与会话不动。
    RefusedByTheRehearsalBeforeAnyWrite {
        refused_publish_in_the_push: usize,
        publishes_in_the_push: usize,
        cause: PublishError,
    },
    /// 预演过了、真发时有一次发不出去（块设备错这一类：落盘途中失败的那一次冻结在分配器上，之后再调入口拒成
    /// `PublishFrozenAfterAWriteFailureIsNotResentYet`；冻结着时入口在预演之前就拒，不进预演，所以不在这里。空间上预演与真发只在
    /// 读盘核换下的那一份对不上时分叉）：[`RootRingPushFailedAfterTheRehearsal`]。
    PublishFailedAfterTheRehearsal(Box<RootRingPushFailedAfterTheRehearsal>),
}

/// 推环真发时第 `failed_publish_in_the_push` 次（从 1 数）发不出去：那一次没成立（分配器换回它之前，发布路径的做法），
/// 前面那几次已落盘、在 `versions_published` 里，会话的 `current` 停在最后一次落盘的那一版上。
#[derive(Debug)]
pub struct RootRingPushFailedAfterTheRehearsal {
    pub failed_publish_in_the_push: usize,
    pub publishes_in_the_push: usize,
    pub cause: PublishError,
    pub versions_published: Vec<Arc<TransactionOutput>>,
}

impl MountedSession {
    /// 推环到 R 那一槽（用户 2026-10-08 弹窗定「显式推环命令加带出路的错」那一个管理入口）：在现行那一版后面接连发 F 不变的空发布
    /// （与可写挂载暖机同一张计划，`mount::empty_publish_plan_after_file_version`：重写固定点单元，不碰用户树、实例表照抄），
    /// 直到这个进程写掉 C 那一圈还没解开、记着落点的每一槽（R 那一槽最后轮到），宽护随之解开（`allocator::PoolAllocator::record_root_written_by_this_process`
    /// 摘下那一槽记着的落点、清掉别处不再护着的隔离位）。要几次见 [`StuckBehindTheWideProtection::empty_publishes_until_the_last_ring_slot_the_push_reaches_is_written`]，至多环长 3S。
    ///
    /// **调用即确认丢掉最近可退到的状态**：R 那一槽是 C 那一圈里最后轮到的一槽，推到它要写掉根环里从现行那一版起接下来轮到的那几槽（次数就是槽数，
    /// 不超过一环），那几槽里原来的根（回退候选）换成同一个用户状态的空发布根；推完之后回退候选集可以掉到少于 4 个不同状态，
    /// D16（发布语义） 已定项 1「回退候选集保留最近 4 个可退到的不同状态」只在调这个入口时破
    /// （第 10z 批调查报告 10z-p 第六节第 1 条：384 槽盘那一形推完只剩 2 个）。不自动调：准入被宽护卡住时只报 [`StuckBehindTheWideProtection`]，
    /// 调用方要出路时（[`Self::ways_out_of_the_wide_protection`]）预演过推完之后那次改动放得下才写明这一条出路。
    ///
    /// 第一道照 [`Self::publish_user_change`] 与 [`Self::ways_out_of_the_wide_protection`] 那样，把交进来的盘表与挂载时核过的那一份逐项比
    /// （次序也比，`mount::ParametersAndDeviceTableOfTheMount::is_the_device_table_of_the_mount`，只比身份、不读盘），不符就拒
    /// （`DeviceTableOtherThanTheOneOfTheMount`，带两份盘表），在任何读写之前：三个入口收盘表同一判（用户 2026-10-10 弹窗定「也先比次序」）。
    /// 这一判在这个公开入口里、不在本体（`push_the_root_ring_on`）里：要出路时的预演经本体走、交的是包过的盘，要出路那一边先核过同一判。
    /// 第二道看分配器上冻结着没重发的发布，冻结着就拒（`PublishFrozenAfterAWriteFailureIsNotResentYet`），在核盘表那一套与预演之前。
    /// C 那一圈记着落点、而它们罩着的槽这一刻一个都没隔离时不推（写掉那几槽放不出东西，`NothingHeldUntilThisProcessWritesARingSlot`），在任何写之前。
    /// 整串先在分配器的一份拷贝上预演（发布路径落盘之前那一段，与抬 F 那一串的预演同一个做法）：推到半路空发布取不到落点（没挡的槽不够这一串）
    /// 时一个写都不发，盘上逐字节不变、分配器与会话不动。预演过了才真发；真发时有一次发不出去，前面那几次已落盘、`current` 停在最后一次落盘的那一版上。
    /// 这些空发布不过准入两判（没有普通分配，D28（挂载期承诺量） 已定项 1），只受「取得到落点」约束。交进来的盘表照抬 F 那一串的
    /// 空发布同一套核（`mount::parameters_of_the_pool_agreeing_with_the_caller_inputs`），在任何写之前；写入口只按盘上那一份参数建。
    ///
    /// # Errors
    /// [`RootRingPushRefused`] 的七种。
    pub fn push_the_root_ring_to_the_last_ring_slot_it_reaches_dropping_rollback_candidates_below_four_states<
        Device: BlockDevice,
    >(
        &mut self,
        devices: &mut Vec<(DeviceIdentity, Device)>,
    ) -> Result<RootRingPushed, RootRingPushRefused> {
        let MountedSession {
            allocator,
            current,
            instance,
            shadow_ledger: _,
            parameters_and_device_table,
            compaction: _,
        } = self;
        let is_the_device_table_handed_in_the_one_of_the_mount =
            parameters_and_device_table.is_the_device_table_of_the_mount(devices);
        if !is_the_device_table_handed_in_the_one_of_the_mount {
            return Err(RootRingPushRefused::DeviceTableOtherThanTheOneOfTheMount {
                device_table_of_the_mount: parameters_and_device_table
                    .device_identities_in_table_order()
                    .to_vec(),
                device_table_handed_in: devices.iter().map(|(identity, _)| *identity).collect(),
            });
        }
        Self::push_the_root_ring_on(
            parameters_and_device_table.parameters_on_disk(),
            devices,
            allocator,
            current,
            *instance,
        )
    }

    /// 被宽护卡住之后调用方要出路时调（用户 2026-10-09 弹窗定「改：预演延到要出路时才做」）：照 `stuck` 那一份报记着的「这次改动还剩什么」，
    /// 在分配器的一份拷贝上、对只进内存的盘预演推环与推完之后这次改动剩下的发布（[`rehearse_the_push_then_the_remainder_of_the_change`]：
    /// 与真调推环入口、之后走报里剩下的那一段走同一段：报里还剩这次改动时是再发同一个改动；只剩走空时 unlink 的意图已发成，
    /// 是下一次改动开头的走空，不再发这次改动），照结局列出路（推完放得下才列推环）。报 `stuck` 的那一次不付这一笔。
    ///
    /// 收 `&self`：预演只读交进来的盘、在分配器的拷贝上走，会话的分配器、现行那一版一样都不动——类型上就动不了（`PoolAllocator` 里没有内部可变），
    /// 调用方不必为了要出路把会话交出去。盘表只读借：只进内存的盘只读借底下那块盘，写一个都发不到它上面。
    /// 交进来的盘表先照 [`Self::publish_user_change`] 与推环入口那样与挂载时核过的那一份逐项比（次序也比，三个入口同一判），
    /// 再照推环本体那一套核（在预演的推环里，核不过就是推环被拒、出路只列卸载再挂），与照出路二真推、再走报里剩下的那一段时同一个结局
    /// （报里还剩这次改动时是再发这次改动）。
    ///
    /// 第一道看会话还在不在被拒的那一版上（实例代号与 txg 都与报里记的相同），再比盘表：哪一道不过都不预演，一个盘都不读。
    /// 第一道只比这两个数，认不出 `stuck` 是不是这个会话报的：别的会话的报若这两个数恰好与这个会话的现行那一版相同，照样过，
    /// 照这个会话的状态预演（[`WaysOutOfTheWideProtectionRefused::SessionMovedPastTheVersionRefusedOnNotSupportedInTheFirstVersion`]）。
    /// 调用方要拿报去问报它的那个会话。
    ///
    /// # Errors
    /// [`WaysOutOfTheWideProtectionRefused`] 的两种。
    pub fn ways_out_of_the_wide_protection<Device: BlockDevice>(
        &self,
        devices: &[(DeviceIdentity, Device)],
        stuck: &StuckBehindTheWideProtection,
    ) -> Result<WaysOutOfTheWideProtection, WaysOutOfTheWideProtectionRefused> {
        let is_the_version_refused_on = |version_of_trees: &TransactionOutput| {
            version_of_trees.root.instance == stuck.instance_of_the_version_refused_on
                && version_of_trees.root.checkpoint_txg
                    == stuck.checkpoint_txg_of_the_version_refused_on
        };
        let version_refused_on = match &self.current {
            PoolVersion::WithFile(version_of_trees)
            | PoolVersion::WithTreesAndNoFile(version_of_trees)
                if is_the_version_refused_on(version_of_trees) =>
            {
                version_of_trees
            }
            PoolVersion::WithFile(_)
            | PoolVersion::WithTreesAndNoFile(_)
            | PoolVersion::WithoutFile(_) => {
                let current_root = self.current.root();
                return Err(
                    WaysOutOfTheWideProtectionRefused::SessionMovedPastTheVersionRefusedOnNotSupportedInTheFirstVersion {
                        instance_refused_on: stuck.instance_of_the_version_refused_on,
                        checkpoint_txg_refused_on: stuck.checkpoint_txg_of_the_version_refused_on,
                        instance_of_the_current_version: current_root.instance,
                        checkpoint_txg_of_the_current_version: current_root.checkpoint_txg,
                    },
                );
            }
        };
        if !self
            .parameters_and_device_table
            .is_the_device_table_of_the_mount(devices)
        {
            return Err(
                WaysOutOfTheWideProtectionRefused::DeviceTableOtherThanTheOneOfTheMount {
                    device_table_of_the_mount: self
                        .parameters_and_device_table
                        .device_identities_in_table_order()
                        .to_vec(),
                    device_table_handed_in: devices.iter().map(|(identity, _)| *identity).collect(),
                },
            );
        }
        let pushing_the_root_ring = rehearse_the_push_then_the_remainder_of_the_change(
            self.parameters_and_device_table.parameters_on_disk(),
            devices,
            &self.allocator,
            version_refused_on,
            self.instance,
            self.shadow_ledger,
            stuck.remainder_of_the_change.as_remainder(),
        );
        let ways_out = stuck.ways_out_after(&pushing_the_root_ring);
        Ok(WaysOutOfTheWideProtection {
            pushing_the_root_ring,
            ways_out,
        })
    }

    /// [`Self::push_the_root_ring_to_the_last_ring_slot_it_reaches_dropping_rollback_candidates_below_four_states`] 的本体，盘上择到的参数由调用方给：
    /// 要出路时的预演在只进内存的盘与分配器的拷贝上走同一段（[`rehearse_the_push_then_the_remainder_of_the_change`]）。
    ///
    /// # Errors
    /// 同那个入口。
    fn push_the_root_ring_on<Device: BlockDevice>(
        parameters_on_disk: &MakeFilesystemParameters,
        devices: &mut Vec<(DeviceIdentity, Device)>,
        allocator: &mut PoolAllocator,
        current: &mut PoolVersion,
        instance: InstanceGeneration,
    ) -> Result<RootRingPushed, RootRingPushRefused> {
        // D23（journal 的角色与格式） 已定项 14「这一版的失败处置」：冻结着的那次发布等原样重发。下面的整串预演走的是发布路径落盘之前那一段、
        // 不判冻结，在冻结着的分配器拷贝上照样过，真发第一次才被拒（第 10z 批代码轮第一轮攻方第二节「一处报法不准」）。
        if let Some(frozen) = allocator.frozen_publish() {
            return Err(
                RootRingPushRefused::PublishFrozenAfterAWriteFailureIsNotResentYet {
                    checkpoint_txg: frozen.checkpoint_txg(),
                    instance: frozen.instance(),
                },
            );
        }
        let start = match &*current {
            PoolVersion::WithFile(version_of_trees)
            | PoolVersion::WithTreesAndNoFile(version_of_trees) => Arc::clone(version_of_trees),
            PoolVersion::WithoutFile(_) => {
                return Err(RootRingPushRefused::VersionWithoutFileNotSupportedInTheFirstVersion);
            }
        };
        let parameters_of_the_pool = parameters_of_the_pool_agreeing_with_the_caller_inputs(
            parameters_on_disk,
            devices,
            allocator,
            &start,
        )
        .map_err(|refusal| RootRingPushRefused::DevicesHandedIn(Box::new(refusal)))?;
        let parameters = &parameters_of_the_pool;
        let Some(target) = root_ring_push_target(allocator, start.root.checkpoint_txg) else {
            return Err(RootRingPushRefused::NothingHeldUntilThisProcessWritesARingSlot);
        };
        // C 那一圈记着落点、它们罩着的槽却一个都没隔离（全在现行账里仍分配、或已释放还在 defer 窗口里）：写掉那几槽放不出东西，
        // 与没有记着落点的槽同一个结局（判决 `research/prompts/m3-batch10z-pqrs-code-r1-main-verification.md` 第二节「K1-1 三个量」那一行，正推注入级那一形）。
        let slots_the_push_would_release_isolation_of =
            allocator.isolated_slots_covered_by(&target.placements);
        if slots_the_push_would_release_isolation_of
            .iter()
            .all(|(_, isolated)| *isolated == 0)
        {
            return Err(RootRingPushRefused::NothingHeldUntilThisProcessWritesARingSlot);
        }
        let publishes_in_the_push =
            usize::try_from(target.publishes_until_the_last_ring_slot_the_push_reaches_is_written)
                .expect("至多环长 3S ≤ 48 次");
        // 预演：迭代上界是这一串的次数；跨轮携带的是预演出来的现行那一版（下一次的计划接着它）与分配器的拷贝。
        {
            let rehearsal_writer = PoolWriter::new(parameters, devices.as_mut_slice());
            // 分配器的拷贝是预演要就地改的那一份，照拷；预演起点的那一版只读，与 `start` 指着同一份。
            let mut copy = allocator.clone();
            let mut rehearsed = Arc::clone(&start);
            for publish_index in 0..publishes_in_the_push {
                let (_, next) = prepare_the_version_publish(
                    &rehearsal_writer,
                    &mut copy,
                    &empty_publish_plan_after_file_version(&rehearsed, instance),
                    Some(Arc::as_ref(&rehearsed)),
                    rehearsed.tree_identifiers,
                    ReleaseChecksumCheck::SkippedByTheDryRunBeforeAcquisition,
                    UnmountMarker::NotWrittenByTheUnmountSequence,
                    &DeletionAndLineageOfThePublish::NEITHER_A_DELETION_NOR_A_ROLLBACK,
                )
                .map_err(|cause| {
                    RootRingPushRefused::RefusedByTheRehearsalBeforeAnyWrite {
                        refused_publish_in_the_push: publish_index + 1,
                        publishes_in_the_push,
                        cause,
                    }
                })?;
                rehearsed = Arc::new(next);
            }
        }
        // 真发：迭代上界同上；跨轮携带的是落盘了的现行那一版与发成的每一版；出口只有发不出去那一次。
        let mut last_published = start;
        let mut versions_published = Vec::with_capacity(publishes_in_the_push);
        for publish_index in 0..publishes_in_the_push {
            let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
            match publish_version(
                &mut writer,
                allocator,
                empty_publish_plan_after_file_version(&last_published, instance),
                Some(Arc::as_ref(&last_published)),
            ) {
                Ok(next) => {
                    let next = Arc::new(next);
                    versions_published.push(Arc::clone(&next));
                    last_published = next;
                }
                Err(cause) => {
                    *current = PoolVersion::of_a_shared_version_of_trees(last_published);
                    return Err(RootRingPushRefused::PublishFailedAfterTheRehearsal(
                        Box::new(RootRingPushFailedAfterTheRehearsal {
                            failed_publish_in_the_push: publish_index + 1,
                            publishes_in_the_push,
                            cause,
                            versions_published,
                        }),
                    ));
                }
            }
        }
        *current = PoolVersion::of_a_shared_version_of_trees(last_published);
        let still_holding = allocator
            .root_ring_occupancy()
            .expect("推环之前读到了 C 那一圈，装着根环表")
            .ring_slots_holding_placements_named_by_unapplied_journal_records();
        assert!(
            target
                .ring_slots
                .iter()
                .all(|ring_slot| !still_holding.contains_key(ring_slot)),
            "推环写到了 R 那一槽：每次发布 txg 加一、写 target_for_publish(txg) 那一槽并在取完落点之后摘下 C 在那一槽记着的落点，\
             R 那一槽是这几槽里最后轮到的；还记着说明有一次发布没经分配器记根"
        );
        Ok(RootRingPushed {
            versions_published,
            ring_slots_written: target.ring_slots,
        })
    }
}

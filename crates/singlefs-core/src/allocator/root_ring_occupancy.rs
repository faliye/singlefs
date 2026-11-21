//! 根环的占用 `RootRingOccupancy`：环上各槽的根、挂载时读坏的槽能装的 txg、回收下限与被钉住的最低环槽。

use super::*;

/// 挂载那一刻读坏（设备报读错，或读得出、不是全零而自证不过）或读回全零的一个根环槽能住的 txg（D16（发布语义） 已定项 1 的「挂载时读不出的槽」，
/// 读不出、自证不过的根槽在回收门槛里钉在它可能住过的最旧 txg，重读读出有效根或确认为空之前门槛不越过它；
/// 准入抬 F 的上限照 Q3 按它在 (N − 3S, 新实例第一次发布的 txg − 1] 里能住的最大 txg 并）。读回全零的槽同一个算法、同样带这两个数
/// （[`RootRingOccupant::AllZeroAtMount`]）：回收门槛里照「自证不过」那一类钉住，准入抬 F 的上限同读坏的槽一样按
/// 能住的最大 txg 并（D16（发布语义） 已定项 1「抬 F 的上限」那一行）。
/// 不变量：最旧的不高于最大的（构造时断言）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TxgsARingSlotBadAtMountCanHold {
    oldest_txg_it_can_have_held: CheckpointTxg,
    highest_txg_it_can_hold: CheckpointTxg,
}

impl TxgsARingSlotBadAtMountCanHold {
    /// # Panics
    /// `oldest_txg_it_can_have_held` 高于 `highest_txg_it_can_hold`：两个数是从同一个槽、同一个 txg 上界算出来的，倒过来说明算错了。
    #[must_use]
    pub fn new(
        oldest_txg_it_can_have_held: CheckpointTxg,
        highest_txg_it_can_hold: CheckpointTxg,
    ) -> Self {
        assert!(
            oldest_txg_it_can_have_held <= highest_txg_it_can_hold,
            "一个槽可能住过的最旧 txg {oldest_txg_it_can_have_held:?} 不高于它能住的最大 txg {highest_txg_it_can_hold:?}"
        );
        Self {
            oldest_txg_it_can_have_held,
            highest_txg_it_can_hold,
        }
    }

    /// 落到这个槽上、不晚于新实例第一次发布之前那个 txg 的最小 txg：回收门槛钉在这里（窗口之外留在洞里的旧根也罩得住）。
    #[must_use]
    pub fn oldest_txg_it_can_have_held(self) -> CheckpointTxg {
        self.oldest_txg_it_can_have_held
    }

    /// 它在 (N − 3S, 新实例第一次发布的 txg − 1] 里能住的最大 txg：准入抬 F 的上限按它并（Q3）。
    #[must_use]
    pub fn highest_txg_it_can_hold(self) -> CheckpointTxg {
        self.highest_txg_it_can_hold
    }
}

/// 根环一个槽上这会儿住着哪条根。一个 txg 都住不了的槽（年轻的环里还没轮到的）不在 `RootRingOccupancy` 里：
/// 回收门槛取「环里最旧有效根」（D16（发布语义） 已定项 1，`research/prompts/m3-reclaim-window-r1-main-verification.md`
/// 第六节 Q1），看自证合法的根与挂载时读坏、读回全零的槽；读不出的被抛弃根影子账本来就罩不到（`mount::isolate_slots_referenced_only_by_abandoned_roots`）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RootRingOccupant {
    /// 挂载那一刻读回全零的槽：回收门槛照「自证不过」那一类钉在它可能住过的最旧 txg（盘上那一槽可能住着一条有效根、只是这一次读回了全零），
    /// 准入抬 F 的上限同读坏的槽一样按它能住的最大 txg 并（D16（发布语义） 已定项 1「抬 F 的上限」那一行）。
    /// 两个数与读坏的槽同一个算法（[`TxgsARingSlotBadAtMountCanHold`]）。
    /// 直到这个进程把那个槽写掉，或进程内记根时重读读出有效根、读出被抛弃的根才解开；重读读回全零不算确认为空，照旧钉着、照旧并进上限。
    /// 它不算「这个进程知道住着根的槽」。
    AllZeroAtMount {
        txgs_it_can_hold: TxgsARingSlotBadAtMountCanHold,
    },
    /// 挂载那一刻读坏的槽（设备报读错，或读得出、不是全零而自证不过）：回收门槛钉在它可能住过的最旧 txg、准入抬 F 的上限按它能住的最大 txg
    /// 并（[`TxgsARingSlotBadAtMountCanHold`]），直到这个进程把那个槽写掉，或进程内记根时重读读出有效根、读出被抛弃的根
    /// 它不算「这个进程知道住着根的槽」（[`RootRingOccupancy::ring_slots_known_to_hold_a_root`]）：
    /// D16 已定项 1「根槽这一次读坏」那一行，挂载那一刻就读坏、之后没写过的槽算上限时并进来、判回退候选时当没有根，不因为这里记了它就改成「读坏就拒」。
    BadAtMount(TxgsARingSlotBadAtMountCanHold),
    /// 挂载那一刻读坏的槽，这个进程在它成了环里最小的那个时重读、读出了一条自证过而按实例表仍然有效的根
    /// （D16（发布语义） 已定项 1 的 E2，`research/prompts/m3-root-ring-code-r1-main-verification.md` 第三节表二之后那一段）：
    /// 照读出来的 txg 进环里最旧有效根。它算「这个进程知道住着根的槽」（[`RootRingOccupancy::ring_slots_known_to_hold_a_root`]；D16 已定项 1
    /// 「根槽这一次读坏」那一行，`research/prompts/m3-root-ring-code-r2-main-verification.md` 第六节 RS1）：
    /// 之后那一槽再读坏，算上限、判回退候选时重读一次、仍坏就拒，不按没有根往下走。
    ValidRootReadWhenTheSlotBadAtMountWasReread { checkpoint_txg: CheckpointTxg },
    /// 重建分配器读根环有一块盘没读出有效根、整环重读了一遍，两遍按槽并起来是一条自证过的根，而另一遍那一槽没自证过（读不出、读得出而
    /// 自证不过或读回全零）：并出来的照读出来的那条根算（有效的进环里最旧有效根，被抛弃的带着它引用的落点，离开根环时按它清），
    /// 另记读出来的那整条根记录 `root_record_self_verified_on_one_rebuild_pass`。抬 F 那一串（准入与卸载）读它读坏、重读一次仍坏时拿这条记录顶上，
    /// 照它本身算、不拒这一次抬：
    /// 有效的照它本身进准入上限与候选集，被抛弃的照它记着的落点进影子账、不进上限；有效还是被抛弃照那一遍判抛弃的实例表判。
    /// 不按挂载那一刻读坏钉在能住的 txg：那样被抛弃的那一半会掉出影子账（I-7.4 红），
    /// 准入上限也不卡在它能住的最大 txg。管理员回退判候选与算 F_生效 那一遍把它算进
    /// 「这个进程知道住着根的槽」（[`RootRingOccupancy::ring_slots_known_to_hold_a_root`]），读坏重读仍坏就拒。
    RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {
        root: RootReadOnOneRebuildPass,
        root_record_self_verified_on_one_rebuild_pass: Box<RootRecord>,
    },
    /// 按实例表判仍然有效的根，txg 在不在 F 之下都算（可再分配谓词取 max(F_生效, 环里最旧有效根)，F 那一半另记）。
    ValidRoot { checkpoint_txg: CheckpointTxg },
    /// 按实例表判被抛弃的根（D23（journal 的角色与格式） 已定项 14），带着它的 (实例代号, txg) 与它引用的每个落点——挂载那一刻仍被候选根或现行账引用、
    /// 因而没隔离的也在内：它们之后在这次挂载里被回收时要补隔离，这条根离开根环时按它清。
    /// 账读不出（凭据放行）的、影子账关着（只供测试）的，这一串是空的：它什么都没隔离，离开根环时也什么都不清。
    /// 抬 F 那一串（准入与卸载）重算影子账时，这一槽这一遍读出的被抛弃根与这里记着的是同一条（实例代号与 txg 都对上）就照这一串算、
    /// 不再从盘上读它的账（[`RootRingOccupancy::placements_recorded_for_the_abandoned_root`]，D16（发布语义） 已定项 1「根槽这一次读坏」那一行末句）。
    AbandonedRoot {
        instance: InstanceGeneration,
        checkpoint_txg: CheckpointTxg,
        referenced_placements: Vec<PlacementOnDevice>,
    },
}

/// [`RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses`] 那一槽里读出来的根按最新那条根的实例表是哪一类：
/// 与 [`RootRingOccupant::ValidRoot`]、[`RootRingOccupant::AbandonedRoot`] 两类同一个分法、同样带着的东西。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RootReadOnOneRebuildPass {
    ValidRoot {
        checkpoint_txg: CheckpointTxg,
    },
    AbandonedRoot {
        referenced_placements: Vec<PlacementOnDevice>,
    },
}

/// 分配器眼里的根环，只住内存：挂载时从盘上读（`mount::rebuilt_allocator`），mkfs 同一个进程里按 mkfs 刚写下的样子装
/// （`make_filesystem::root_ring_occupancy_after_make_filesystem`），之后这个进程每写一条根记一条
/// （`PoolAllocator::record_root_written_by_this_process`）。它回答两件事：环里最旧有效根往前挪了没有（D16（发布语义） 已定项 1 的
/// 可再分配谓词，C518（一次挂载之内环转过一圈之后不回收）），这一次盖掉的是不是一条被抛弃的根（D28（挂载期承诺量） 已定项 1
/// 第九项「被抛弃的根被轮转覆写时清零」，C503（隔离位清零的时机条文与实现说反话））。
#[derive(Clone, Debug)]
pub struct RootRingOccupancy {
    pub(super) slots_per_region: RootRingSlotsPerRegion,
    pub(super) occupants: BTreeMap<RootRingSlot, RootRingOccupant>,
    /// 挂载时生效的 F。抬 F 生效之后不跟着抬：释放代 ≤ 新 F 的抬 F 自己已经回收过，之后释放的释放代都高于新 F，
    /// 谓词里 F 那一半再抬也多收不回一个槽。
    pub(super) effective_floor: CheckpointTxg,
    pub(super) latest_recorded_txg: CheckpointTxg,
    /// 见 [`RootRingOccupancy::instance_table_judging_the_rereads`]。
    pub(super) instance_table_judging_the_rereads: InstanceTableRecords,
    /// 可写挂载那一刻水位之上、没被施加的 journal 记录点名的落点（C694 的 C
    /// 第三节第 1 条：选法取水位之上全部记录），按那条记录的 txg 在根环里对应的槽归组：那条记录所属的发布若落过根，根就住在那一槽，
    /// 这个进程写掉那一槽之前那条根还可能被下一次恢复择到。挂载那一刻仍被候选根或现行账引用、因而没隔离的落点也在内：
    /// 它们之后在这次挂载里被回收时要补隔离。只在 [`PoolAllocator::record_root_written_by_this_process`] 写掉那一槽、那次发布取完落点之后
    /// 解开；进程内重读那一槽（D16（发布语义） 已定项 1 读回全零的槽）读出什么都不解开它。
    pub(super) placements_named_by_unapplied_journal_records:
        BTreeMap<RootRingSlot, Vec<PlacementOnDevice>>,
}

impl RootRingOccupancy {
    /// 从盘上读出来的那一刻：`occupants` 是每个读得出的根环槽上的根与挂载时读坏的槽，`effective_floor` 是恢复后生效的 F，
    /// `latest_txg_before_the_next_publish` 是这个进程下一条根的 txg 减一（挂载时 = 新实例第一次发布的 txg − 1：
    /// 环里只有记录、没有根的那几个 txg 这时已经跳过，它们的槽上照旧是读出来的内容）。
    #[must_use]
    pub fn read_from_the_ring(
        slots_per_region: RootRingSlotsPerRegion,
        occupants: Vec<(RootRingSlot, RootRingOccupant)>,
        effective_floor: CheckpointTxg,
        latest_txg_before_the_next_publish: CheckpointTxg,
    ) -> Self {
        Self {
            slots_per_region,
            occupants: occupants.into_iter().collect(),
            effective_floor,
            latest_recorded_txg: latest_txg_before_the_next_publish,
            instance_table_judging_the_rereads: InstanceTableRecords { rows: Vec::new() },
            placements_named_by_unapplied_journal_records: BTreeMap::new(),
        }
    }

    /// 还没解开的、水位之上没施加的 journal 记录点名的全部落点（[`RootRingOccupancy`] 那一项的注释），按根环槽的次序摊平：
    /// 抬 F 按新候选集重算隔离时拿它（`mount::raise_rollback_floor` 一族）。
    #[must_use]
    pub fn placements_named_by_unapplied_journal_records(&self) -> Vec<PlacementOnDevice> {
        self.placements_named_by_unapplied_journal_records
            .values()
            .flatten()
            .copied()
            .collect()
    }

    /// 抬 F 那一串（准入与卸载）重算影子账时一条被抛弃根的落点（D16（发布语义） 已定项 1「根槽这一次读坏」那一行末句「抬 F 那一串（准入与卸载）里的被抛弃根」）：
    /// 这条根（`instance`、`checkpoint_txg`）的 txg 在根环里对应的那一槽上，表里记着的是挂载那一刻读出的被抛弃根
    /// （[`RootRingOccupant::AbandonedRoot`]，或两遍里只有一遍自证过的槽里被抛弃的那一半，按那一遍读出的整条根记录认），而且就是这一条——
    /// 实例代号与 txg 都对上，不只按槽号对——就交回挂载时算出、记着的那一串落点；记着的是别的根、有效根、挂载时读坏或读回全零的槽，
    /// 或这个进程已写掉那一槽（换成有效根），交 `None`。只读内存里的表，不读盘。
    #[must_use]
    pub fn placements_recorded_for_the_abandoned_root(
        &self,
        instance: InstanceGeneration,
        checkpoint_txg: CheckpointTxg,
    ) -> Option<Vec<PlacementOnDevice>> {
        let ring_slot = target_for_publish(checkpoint_txg, self.slots_per_region);
        match self.occupants.get(&ring_slot)? {
            RootRingOccupant::AbandonedRoot {
                instance: recorded_instance,
                checkpoint_txg: recorded_checkpoint_txg,
                referenced_placements,
            } => ((*recorded_instance, *recorded_checkpoint_txg) == (instance, checkpoint_txg))
                .then(|| referenced_placements.clone()),
            RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {
                root:
                    RootReadOnOneRebuildPass::AbandonedRoot {
                        referenced_placements,
                    },
                root_record_self_verified_on_one_rebuild_pass,
            } => ((
                root_record_self_verified_on_one_rebuild_pass.instance,
                root_record_self_verified_on_one_rebuild_pass.checkpoint_txg,
            ) == (instance, checkpoint_txg))
                .then(|| referenced_placements.clone()),
            RootRingOccupant::ValidRoot { .. }
            | RootRingOccupant::BadAtMount(_)
            | RootRingOccupant::AllZeroAtMount { .. }
            | RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread { .. }
            | RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {
                root: RootReadOnOneRebuildPass::ValidRoot { .. },
                ..
            } => None,
        }
    }

    /// 抬 F 那一串（准入与卸载）重算影子账时，一条根环表里没记着落点的被抛弃根借 C 那一圈记着的落点（D16（发布语义） 已定项 1「根槽这一次读坏」
    /// 那一行）：
    /// 可写挂载自己写行之后才按现行那一版的实例表判成被抛弃的根，挂载那一刻按最新那条根的表判有效、记成有效根，不带落点。
    /// 这条根（`instance`、`checkpoint_txg`）的 txg 在根环里对应的那一槽上，表里记着的是一条有效根、txg 与它相同（实例代号记着的就也比：
    /// 两遍里只有一遍自证过的有效那一半带整条根记录），而 C 那一圈（水位之上没施加的记录点名的落点、那几次发布的账、比所选那一版新的可读根的账，
    /// [`RootRingOccupancy`] 那一项）在那一槽还记着、没解开，就交回那一槽记着的落点；那一槽记的是被抛弃根、挂载时读坏或读回全零的槽、
    /// 有效根而 txg 不同（同一槽差一整圈的另一次发布），或 C 在那一槽什么都没记（这个进程写掉那一槽时解开了，或挂载时就没记），交 `None`，
    /// 调用方照今天读盘。C 在那一槽有条目而落点是空的（凭据放行那一形）交 `Some(空)`：条目在就是记着了，调用方照空串算、不读账。
    /// 只读内存里的表，不读盘。
    #[must_use]
    pub fn placements_named_by_unapplied_journal_records_for_the_root_recorded_valid_on_its_ring_slot(
        &self,
        instance: InstanceGeneration,
        checkpoint_txg: CheckpointTxg,
    ) -> Option<Vec<PlacementOnDevice>> {
        let ring_slot = target_for_publish(checkpoint_txg, self.slots_per_region);
        let is_the_same_root_recorded_valid = match self.occupants.get(&ring_slot)? {
            RootRingOccupant::ValidRoot {
                checkpoint_txg: recorded_checkpoint_txg,
            }
            | RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread {
                checkpoint_txg: recorded_checkpoint_txg,
            } => *recorded_checkpoint_txg == checkpoint_txg,
            RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {
                root: RootReadOnOneRebuildPass::ValidRoot { .. },
                root_record_self_verified_on_one_rebuild_pass,
            } => {
                (
                    root_record_self_verified_on_one_rebuild_pass.instance,
                    root_record_self_verified_on_one_rebuild_pass.checkpoint_txg,
                ) == (instance, checkpoint_txg)
            }
            RootRingOccupant::AbandonedRoot { .. }
            | RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {
                root: RootReadOnOneRebuildPass::AbandonedRoot { .. },
                ..
            }
            | RootRingOccupant::BadAtMount(_)
            | RootRingOccupant::AllZeroAtMount { .. } => false,
        };
        if !is_the_same_root_recorded_valid {
            return None;
        }
        self.placements_named_by_unapplied_journal_records
            .get(&ring_slot)
            .cloned()
    }

    /// 根环表里记成被抛弃根的槽（[`RootRingOccupant::AbandonedRoot`]）与那条根挂载时记着的落点，按槽的次序：抬 F 那一串读根环时这几槽读坏、
    /// 重读仍坏不拒，照这里记着的落点进影子账、不进上限（D16（发布语义） 已定项 1「根槽这一次读坏」那一行 H1
    /// 「被抛弃根的槽不拒」）。两遍里只有一遍自证过的槽不在其内：抬 F 那一遍拿它记着的整条根记录顶上。
    /// 这个进程写掉那一槽之后就不在其内。
    #[must_use]
    pub fn ring_slots_recorded_as_holding_an_abandoned_root(
        &self,
    ) -> BTreeMap<RootRingSlot, Vec<PlacementOnDevice>> {
        self.occupants
            .iter()
            .filter_map(|(ring_slot, occupant)| match occupant {
                RootRingOccupant::AbandonedRoot {
                    referenced_placements,
                    ..
                } => Some((*ring_slot, referenced_placements.clone())),
                RootRingOccupant::ValidRoot { .. }
                | RootRingOccupant::BadAtMount(_)
                | RootRingOccupant::AllZeroAtMount { .. }
                | RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread { .. }
                | RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses { .. } => None,
            })
            .collect()
    }

    /// C 那一圈（水位之上没施加的记录点名的落点、那几次发布的账、比所选那一版新的可读根的账，认不出回退时的宽护在其中，[`RootRingOccupancy`]
    /// 那一项的注释）还没解开、记着落点的槽，各带记着的落点，按槽的次序：这个进程写掉那一槽之前它们一直罩着（回收出来的照补隔离）。
    /// 会话推环推到哪一槽、推满仍不够时报不报「被宽护卡住」读它（`mounted_session`）。
    /// 记着空串的槽（凭据放行那一形）不在其内：它什么都不罩。只读内存里的表。
    #[must_use]
    pub fn ring_slots_holding_placements_named_by_unapplied_journal_records(
        &self,
    ) -> BTreeMap<RootRingSlot, Vec<PlacementOnDevice>> {
        self.placements_named_by_unapplied_journal_records
            .iter()
            .filter(|(_, placements)| !placements.is_empty())
            .map(|(ring_slot, placements)| (*ring_slot, placements.clone()))
            .collect()
    }

    /// 这一槽上表里记着的有效根的 txg（挂载那一刻读出、按最新那条根的实例表有效的，进程内重读读出的，两遍里只有一遍自证过而有效的，
    /// 或这个进程写的）；记的是被抛弃根、挂载那一刻读坏或读回全零的槽，或表里没记这一槽，交 `None`。只读内存里的表。
    #[must_use]
    pub fn checkpoint_txg_of_the_valid_root_recorded_on(
        &self,
        ring_slot: RootRingSlot,
    ) -> Option<CheckpointTxg> {
        match self.occupants.get(&ring_slot)? {
            RootRingOccupant::ValidRoot { checkpoint_txg }
            | RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread { checkpoint_txg }
            | RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {
                root: RootReadOnOneRebuildPass::ValidRoot { checkpoint_txg },
                ..
            } => Some(*checkpoint_txg),
            RootRingOccupant::AbandonedRoot { .. }
            | RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {
                root: RootReadOnOneRebuildPass::AbandonedRoot { .. },
                ..
            }
            | RootRingOccupant::BadAtMount(_)
            | RootRingOccupant::AllZeroAtMount { .. } => None,
        }
    }

    /// 环里还在的被抛弃根引用着的全部落点，加还没解开的、水位之上没施加的记录点名的全部落点：清隔离位之前要看的那一份
    /// （一个槽两边任一边还罩着就不清），回收出来的槽里它们罩着的补隔离。
    pub(super) fn placements_the_isolation_still_protects(&self) -> Vec<PlacementOnDevice> {
        let mut protected = self.placements_referenced_by_abandoned_roots();
        protected.extend(self.placements_named_by_unapplied_journal_records());
        protected
    }

    /// 根环每区槽数 S（系统配置偏移 362）：准入式子里环窗口按它算（`admission::ring_window_on_one_device`）。
    #[must_use]
    pub fn slots_per_region(&self) -> RootRingSlotsPerRegion {
        self.slots_per_region
    }

    /// 这个进程知道住着一条根的根环槽（D16（发布语义） 已定项 1「根槽这一次读坏」那一行）：挂载那一刻读得出、自证过的槽
    /// （有效的与被抛弃的都算），这个进程之后写过、FUA 返回过的槽——发布在落盘之前或落盘途中失败时分配器整个换回发布之前那一份，
    /// 这张表随之退回，记着的只剩根落盘做成的那几次——，与进程内记根时重读读出了有效根的槽
    /// （[`RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread`]）。挂载那一刻读坏、
    /// 之后又没写过也没重读出有效根的槽不在其内（[`RootRingOccupant::BadAtMount`] 与 [`RootRingOccupant::AllZeroAtMount`] 记着的槽也不在）。
    /// 重建分配器两遍里只有一遍自证过的槽（[`RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses`]）在其内：管理员回退判候选、
    /// 算 F_生效 读到它仍坏照旧拒；抬 F 那一遍另按 [`Self::ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes`] 处置
    /// （`mount::raise_rollback_floor` 一族把它们从这一份里拿掉）。
    #[must_use]
    pub fn ring_slots_known_to_hold_a_root(&self) -> BTreeSet<RootRingSlot> {
        self.occupants
            .iter()
            .filter(|(_, occupant)| match occupant {
                RootRingOccupant::ValidRoot { .. }
                | RootRingOccupant::AbandonedRoot { .. }
                | RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread { .. }
                | RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses { .. } => true,
                RootRingOccupant::BadAtMount(_) | RootRingOccupant::AllZeroAtMount { .. } => false,
            })
            .map(|(ring_slot, _)| *ring_slot)
            .collect()
    }

    /// 重建分配器两遍里只有一遍自证过、还记在表里的槽（[`RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses`]），
    /// 各带重建那一遍自证过的那整条根记录，按槽的次序：抬 F 那一串读它读坏、重读一次仍坏时拿这条记录顶上。
    /// 这个进程写掉那一槽之后就不在其内。
    #[must_use]
    pub fn ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes(
        &self,
    ) -> BTreeMap<RootRingSlot, RootRecord> {
        self.occupants
            .iter()
            .filter_map(|(ring_slot, occupant)| match occupant {
                RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {
                    root_record_self_verified_on_one_rebuild_pass,
                    ..
                } => Some((*ring_slot, **root_record_self_verified_on_one_rebuild_pass)),
                RootRingOccupant::ValidRoot { .. }
                | RootRingOccupant::BadAtMount(_)
                | RootRingOccupant::AllZeroAtMount { .. }
                | RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread { .. }
                | RootRingOccupant::AbandonedRoot { .. } => None,
            })
            .collect()
    }

    /// 环里最旧有效根的 txg（D16（发布语义） 已定项 1「环里最旧有效根」：回收门槛从一圈窗口退回整个环）：
    /// 这张表里全部有效根与挂载时读坏、读回全零的槽（钉在它可能住过的最旧 txg）取最小，
    /// 不按 txg 离这个进程记到的最新根多远滤。
    /// 一条都没有时 `None`。
    pub(super) fn oldest_valid_root_in_the_ring(&self) -> Option<CheckpointTxg> {
        self.occupants
            .values()
            .filter_map(|occupant| match occupant {
                RootRingOccupant::ValidRoot { checkpoint_txg, .. }
                | RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread {
                    checkpoint_txg,
                }
                | RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {
                    root: RootReadOnOneRebuildPass::ValidRoot { checkpoint_txg },
                    ..
                } => Some(*checkpoint_txg),
                RootRingOccupant::BadAtMount(txgs_it_can_hold) => {
                    Some(txgs_it_can_hold.oldest_txg_it_can_have_held())
                }
                RootRingOccupant::AllZeroAtMount { txgs_it_can_hold } => {
                    Some(txgs_it_can_hold.oldest_txg_it_can_have_held())
                }
                RootRingOccupant::AbandonedRoot { .. }
                | RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {
                    root: RootReadOnOneRebuildPass::AbandonedRoot { .. },
                    ..
                } => None,
            })
            .min()
    }

    /// 环里还在的被抛弃根引用着的全部落点。
    fn placements_referenced_by_abandoned_roots(&self) -> Vec<PlacementOnDevice> {
        self.occupants
            .values()
            .flat_map(|occupant| match occupant {
                RootRingOccupant::AbandonedRoot {
                    referenced_placements,
                    ..
                }
                | RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {
                    root:
                        RootReadOnOneRebuildPass::AbandonedRoot {
                            referenced_placements,
                        },
                    ..
                } => referenced_placements.as_slice(),
                RootRingOccupant::ValidRoot { .. }
                | RootRingOccupant::BadAtMount(_)
                | RootRingOccupant::AllZeroAtMount { .. }
                | RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread { .. }
                | RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {
                    root: RootReadOnOneRebuildPass::ValidRoot { .. },
                    ..
                } => &[],
            })
            .copied()
            .collect()
    }

    /// 挂载那一刻读坏、还记在表里的槽（[`RootRingOccupant::BadAtMount`]），各带它能住的 txg，按槽的次序。回收门槛与 E2 重读按其中
    /// 可能住过的最旧 txg 取；准入抬 F 的上限按其中能住的最大 txg 并，连同读回全零的槽一起
    /// （[`Self::ring_slots_bad_or_reading_back_all_zero_at_mount`]）。读回全零的槽不在这一份里。
    #[must_use]
    pub fn ring_slots_bad_at_mount(
        &self,
    ) -> BTreeMap<RootRingSlot, TxgsARingSlotBadAtMountCanHold> {
        self.occupants
            .iter()
            .filter_map(|(ring_slot, occupant)| match occupant {
                RootRingOccupant::BadAtMount(txgs_it_can_hold) => {
                    Some((*ring_slot, *txgs_it_can_hold))
                }
                RootRingOccupant::ValidRoot { .. }
                | RootRingOccupant::AllZeroAtMount { .. }
                | RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread { .. }
                | RootRingOccupant::AbandonedRoot { .. }
                | RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses { .. } => None,
            })
            .collect()
    }

    /// 挂载那一刻读坏、还记在表里的槽，各钉在它可能住过的最旧 txg：回收门槛与 E2 重读按它取。
    #[must_use]
    pub fn ring_slots_bad_at_mount_pinned_at_the_oldest_txg_they_can_have_held(
        &self,
    ) -> BTreeMap<RootRingSlot, CheckpointTxg> {
        self.ring_slots_bad_at_mount()
            .into_iter()
            .map(|(ring_slot, txgs_it_can_hold)| {
                (ring_slot, txgs_it_can_hold.oldest_txg_it_can_have_held())
            })
            .collect()
    }

    /// 挂载那一刻读坏或读回全零、还记在表里的槽（[`RootRingOccupant::BadAtMount`] 与 [`RootRingOccupant::AllZeroAtMount`]），
    /// 各带它能住的 txg，按槽的次序：准入抬 F 的上限按其中能住的最大 txg 并（`mount::ring_slots_bad_or_reading_back_all_zero_at_mount_by`；
    /// D16（发布语义） 已定项 1「抬 F 的上限」那一行：读坏槽与读回全零的槽都并进上限，读回全零那一半）。
    /// 一个槽在表里只有一项，两类不会同时有它。
    #[must_use]
    pub fn ring_slots_bad_or_reading_back_all_zero_at_mount(
        &self,
    ) -> BTreeMap<RootRingSlot, TxgsARingSlotBadAtMountCanHold> {
        self.occupants
            .iter()
            .filter_map(|(ring_slot, occupant)| match occupant {
                RootRingOccupant::BadAtMount(txgs_it_can_hold)
                | RootRingOccupant::AllZeroAtMount { txgs_it_can_hold } => {
                    Some((*ring_slot, *txgs_it_can_hold))
                }
                RootRingOccupant::ValidRoot { .. }
                | RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread { .. }
                | RootRingOccupant::AbandonedRoot { .. }
                | RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses { .. } => None,
            })
            .collect()
    }

    /// 挂载那一刻读回全零、还记在表里的槽（[`RootRingOccupant::AllZeroAtMount`]），各钉在它可能住过的最旧 txg，按槽的次序。
    /// 准入抬 F 的上限按它们能住的最大 txg 并，经 [`Self::ring_slots_bad_or_reading_back_all_zero_at_mount`] 取，不经这一份。
    #[must_use]
    pub fn ring_slots_reading_back_all_zero_at_mount_pinned_at_the_oldest_txg_they_can_have_held(
        &self,
    ) -> BTreeMap<RootRingSlot, CheckpointTxg> {
        self.occupants
            .iter()
            .filter_map(|(ring_slot, occupant)| match occupant {
                RootRingOccupant::AllZeroAtMount { txgs_it_can_hold } => {
                    Some((*ring_slot, txgs_it_can_hold.oldest_txg_it_can_have_held()))
                }
                RootRingOccupant::ValidRoot { .. }
                | RootRingOccupant::BadAtMount(_)
                | RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread { .. }
                | RootRingOccupant::AbandonedRoot { .. }
                | RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses { .. } => None,
            })
            .collect()
    }

    /// 回收门槛与 E2 重读看的那几个槽：挂载那一刻读坏的与读回全零的，各钉在它可能住过的最旧 txg（读回全零的照「自证不过」那一类钉住）。
    /// 一个槽在表里只有一项，两类不会同时有它。
    #[must_use]
    pub fn ring_slots_pinned_since_the_mount_at_the_oldest_txg_they_can_have_held(
        &self,
    ) -> BTreeMap<RootRingSlot, CheckpointTxg> {
        let mut pinned = self.ring_slots_bad_at_mount_pinned_at_the_oldest_txg_they_can_have_held();
        pinned.extend(
            self.ring_slots_reading_back_all_zero_at_mount_pinned_at_the_oldest_txg_they_can_have_held(),
        );
        pinned
    }

    /// 进程内重读那几个槽时判「按实例表仍然有效」用的那张表（D16（发布语义） 已定项 1「环里最旧有效根」：按实例表判仍然有效）。
    /// 装上根环表时是空表；可写挂载在写行那次发布的表一定下来、第一次记根之前换成它（`mount::establish_instance`）——
    /// 池级 checker 按镜像里最新那条根的实例表判，写行之后最新那条根带的就是写行那一张。挂载装上根环表到换表之间不记根，空表没人读；
    /// mkfs 同一个进程里一直是空表（表里没有挂载时读坏的槽，用不到它）。
    #[must_use]
    pub fn instance_table_judging_the_rereads(&self) -> &InstanceTableRecords {
        &self.instance_table_judging_the_rereads
    }
}

/// 进程内重读一个挂载那一刻读坏或读回全零的根环槽读到了什么（D16（发布语义） 已定项 1 的 E2：
/// 重读读出有效根之前回收门槛不越过它；读回全零不算确认为空，照「自证不过」那一类钉着）。封闭集合，`match` 不写通配臂。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RereadOfARootRingSlotBadAtMount {
    /// 读得出、自证过，按实例表仍然有效：照读出来的 txg 算（回收门槛取整个环，RGall，读出来多旧都算）；那一槽从此算「这个进程知道住着根的槽」。
    ValidRoot { checkpoint_txg: CheckpointTxg },
    /// 读得出、自证过，读出来的根按实例表被抛弃：当没有根。
    AbandonedByTheInstanceTable,
    /// 读回全零：不算确认为空（盘上那一槽可能住着一条有效根、这一次又读回了全零），照旧钉在它可能住过的最旧 txg。
    AllZero,
    /// 仍读不出，或读得出、不是全零而自证不过：照旧钉在它可能住过的最旧 txg。
    StillUnreadableOrNotSelfVerified,
}

/// 环里最旧有效根，挂载那一刻读坏的槽按 E2 并进来（D16（发布语义） 已定项 1；回收门槛取整个环
/// RGall；读坏的槽钉在它可能住过的最旧 txg；读回全零的槽同样钉着）：`valid_root_txgs` 是已知
/// 住着有效根的 txg，`bad_at_mount` 是挂载那一刻读坏或读回全零、各按它可能住过的最旧 txg 钉着的槽；两者里最小的那个若是一个钉着的槽，
/// 先重读那一槽（`reread`）：读出有效根就照它的 txg 换进去、读出被抛弃的根就拿掉，再取最小；仍读不出、读得出而自证不过或读回全零
/// 才按钉着的 txg 算。
/// 交回最小的 txg（一条都没有时 `None`）与每个重读过的槽读到了什么（按重读的次序）。
///
/// 循环：每轮要么换掉或拿掉一个读坏的槽（`bad_at_mount` 少一个），要么在最小的是一个仍读坏的槽、或最小的不是读坏的槽时返回；
/// 轮数至多 `bad_at_mount` 的条数加一。跨轮携带两张表与重读结局。
pub fn oldest_txg_in_the_ring_rereading_slots_bad_at_mount(
    valid_root_txgs: &[CheckpointTxg],
    mut bad_at_mount: BTreeMap<RootRingSlot, CheckpointTxg>,
    reread: &mut dyn FnMut(RootRingSlot) -> RereadOfARootRingSlotBadAtMount,
) -> (
    Option<CheckpointTxg>,
    Vec<(RootRingSlot, RereadOfARootRingSlotBadAtMount)>,
) {
    let mut valid_root_txgs: Vec<CheckpointTxg> = valid_root_txgs.to_vec();
    let mut rereads = Vec::new();
    loop {
        let oldest_valid_root = valid_root_txgs.iter().copied().min();
        let oldest_bad_slot = bad_at_mount
            .iter()
            .min_by_key(|(_, pinned_txg)| **pinned_txg)
            .map(|(ring_slot, pinned_txg)| (*ring_slot, *pinned_txg));
        let Some((ring_slot, pinned_txg)) = oldest_bad_slot else {
            return (oldest_valid_root, rereads);
        };
        if oldest_valid_root.is_some_and(|oldest| oldest < pinned_txg) {
            return (oldest_valid_root, rereads);
        }
        let reread_of_the_slot = reread(ring_slot);
        rereads.push((ring_slot, reread_of_the_slot));
        match reread_of_the_slot {
            RereadOfARootRingSlotBadAtMount::ValidRoot { checkpoint_txg } => {
                bad_at_mount.remove(&ring_slot);
                valid_root_txgs.push(checkpoint_txg);
            }
            RereadOfARootRingSlotBadAtMount::AbandonedByTheInstanceTable => {
                bad_at_mount.remove(&ring_slot);
            }
            RereadOfARootRingSlotBadAtMount::StillUnreadableOrNotSelfVerified
            | RereadOfARootRingSlotBadAtMount::AllZero => {
                return (Some(pinned_txg), rereads);
            }
        }
    }
}

/// 分配器记一条根那一刻的回收门槛与还钉着的槽里最小的钉值（D16（发布语义） 已定项 1「根槽这一次读坏」那一行「钉也落盘」，
/// C773（读坏与全零根槽的回收门槛钉只在内存里））：[`PoolAllocator::record_root_written_by_this_process`] 记完这条根、
/// 按谓词回收之后记下（[`PoolAllocator::reclaim_floor_when_the_latest_root_was_recorded`]），带单元的发布在记根之后、装根之前读它，
/// 写根记录里的回收门槛 t 与钉槽 p（采样点在回收之后、装根之前）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReclaimFloorWhenTheRootWasRecorded {
    /// 这一次回收用的门槛：max(挂载时生效的 F, 环里最旧有效根并上钉值)（`mount::reclaim_floor`）。
    pub reclaim_floor: CheckpointTxg,
    /// 记了这条根之后还钉着的槽（挂载那一刻读坏或读回全零、这个进程还没写掉、重读也没解开的）里按（钉值, 槽）最小的那一个；
    /// 一个都没钉着是 `None`。
    pub lowest_pin: Option<LowestPinnedRingSlot>,
}

/// 记一条根之后还钉着的槽里按（钉值, 槽）最小的那一个：钉值与槽一起往下带到写根那一刻（C773（读坏与全零根槽的回收门槛钉只在内存里），
///  采乙）。三个区域的槽 0 钉值都是第 0 代根的 txg 0
/// （`mount::oldest_txg_a_ring_slot_bad_at_mount_can_have_held`），钉值不再唯一定槽：只带钉值就还原不出钉的是哪一槽，钉槽 p 存的是槽。
/// 平局按槽（区域在前、槽号在后）破，同一份根环表每次记根取到的是同一槽。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LowestPinnedRingSlot {
    pub pinned_txg: CheckpointTxg,
    pub ring_slot: RootRingSlot,
}

impl LowestPinnedRingSlot {
    /// 钉着的槽在根环里的编码 `3·槽 + 区域`：钉槽 p 存它（0 到 3S 之内与落点公式 `root_ring::target_for_publish` 落到这一槽的最小 txg 相同）。
    #[must_use]
    pub fn ring_slot_encoding(self) -> u64 {
        ROOT_RING_REGIONS * self.ring_slot.slot + self.ring_slot.region
    }
}

impl ReclaimFloorWhenTheRootWasRecorded {
    /// 根记录里的回收门槛 t：max(这条根自己带的 F, 分配器记这条根时的回收门槛)。
    /// 分配器那一半用的是挂载时生效的 F、抬 F 生效之后不跟着抬（[`RootRingOccupancy`] 的 `effective_floor`），所以要再与这条根自己的 F 取大：
    /// 抬 F 那一串自己已经回收到新 F。
    #[must_use]
    pub fn reclaim_threshold_of_a_root_carrying(
        self,
        rollback_floor_of_the_root: CheckpointTxg,
    ) -> CheckpointTxg {
        self.reclaim_floor.max(rollback_floor_of_the_root)
    }

    /// 根记录里的钉槽 p：按（钉值, 槽）最小的那一槽的编码 `3·槽 + 区域`，不是它的钉值（槽号为 0 的槽钉值是 0、编码是区域号）。
    #[must_use]
    pub fn pinned_ring_slot(self) -> crate::root_record::PinnedRingSlot {
        crate::root_record::PinnedRingSlot::of_the_lowest_pin(
            self.lowest_pin
                .map(LowestPinnedRingSlot::ring_slot_encoding),
        )
    }
}

#[cfg(test)]
mod tests;

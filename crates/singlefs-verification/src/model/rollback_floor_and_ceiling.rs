//! 回退下界 F 的生效值、抬 F 的上限与掉电之后系统配置里留下的 F。

use super::*;

/// 说谎的设备吞掉几条根的根槽写之后，实现现读根环算出的准入抬 F 的上限两个数（[`IdealModel::rollback_floor_ceilings_without_the_roots`]）；
/// 实现那一侧同名的两个数在 `crates/singlefs-core/src/mount.rs` 的 `AdmissionCeilings`。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModelRollbackFloorCeilingsWithoutTheRoots {
    /// 只照读得出的根算：那几条根拿掉。
    pub from_the_roots_read: ModelCheckpointTxg,
    /// 再并上那几条根留下的、这个会话挂载那一刻读回全零的槽：准入抬 F 抬到的是它，实现报的也是它。
    pub merging_the_ring_slots_left_all_zero: ModelCheckpointTxg,
}

impl ModelRollbackFloorCeilingsWithoutTheRoots {
    /// 并进全零槽把上限压低了：实现推空（上限卡在那几槽上）时报的上限就是并进之后的那个，不并时的比它高
    /// （D16（发布语义） 已定项 1「抬 F 的上限」那一行那一括注）。
    #[must_use]
    pub fn are_lowered_by_the_ring_slots_left_all_zero(self) -> bool {
        self.merging_the_ring_slots_left_all_zero < self.from_the_roots_read
    }
}

/// 掉电之后盘上系统配置里的 F 落在模型答的两个值里的哪一个
/// （[`IdealModel::settle_floor_left_in_the_system_configurations_by_a_lost_power`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModelFloorLeftByALostPower {
    /// 还是掉电之前的 F_生效：这次改动没有走到抬 F 那一串写系统配置那一步。
    KeptAtTheEffectiveFloor,
    /// 是模型算的上限：抬 F 那一串把新 F 写进了系统配置，掉电截在它的第一条根之前。
    RaisedToTheCeiling,
}

impl IdealModel {
    /// F_生效 = 各幸存盘最新持久有效根所带 F 的最大值（D16（发布语义） 已定项 1「生效」根那一半）：有效 = 按最新那条根指着的实例表
    /// 不被抛弃；每块盘取落在它上面的有效根里 (txg, 实例) 最大的那一条。一条根都没有时 0。
    /// 系统配置那一半（SysPre）只记掉电留下的那一份：抬 F 那一串要么在任何写之前被拒、要么推到每块盘，系统配置里的 F 与那一串的根带的相同，
    /// 只有掉电截在写完系统配置之后、第一条根之前时高出根这一半（[`Self::settle_floor_left_in_the_system_configurations_by_a_lost_power`]）。
    /// 被抛弃时间线上的根带的 F 算不算，D16 写着仍开着；这里照「有效根」的字面，不算。
    #[must_use]
    pub fn effective_rollback_floor(&self) -> ModelCheckpointTxg {
        let newest_table = Rc::clone(&self.newest_root().instance_table_rows);
        let mut newest_valid_root_per_device: BTreeMap<ModelDeviceIdentity, &ModelRoot> =
            BTreeMap::new();
        for root in self.ring.values() {
            if Self::is_abandoned_by(root, &newest_table) {
                continue;
            }
            let newest = newest_valid_root_per_device
                .entry(device_holding_the_root_of(root.key.checkpoint_txg))
                .or_insert(root);
            if root.key > newest.key {
                *newest = root;
            }
        }
        newest_valid_root_per_device
            .values()
            .map(|root| root.rollback_floor)
            .max()
            .unwrap_or(ModelCheckpointTxg(0))
            .max(self.floor_left_in_the_system_configurations_by_a_lost_power)
    }

    /// 一次发布掉电在它的第一个 journal 记录写上之后，判盘上系统配置里读得出的 F 的最大值（D16（发布语义） 已定项 1「生效」「抬 F 那一串」）并记下。
    /// 掉电之前抬 F 那一串最多走到把新 F 写进每块盘的系统配置（它的第一条根就要写 journal），所以这个值只能是掉电之前的 F_生效，或这一刻模型算的上限
    /// （[`Self::rollback_floor_ceiling`]）；两个值都由模型自己算，胶水只报盘上读到的是哪一个。
    ///
    /// # Errors
    /// 读到的值不是这两个（抬过了上限、抬到一半、低于 F_生效）：交回 [`ModelDisagreementAspect::RollbackFloor`] 那一格，模型不动。
    pub fn settle_floor_left_in_the_system_configurations_by_a_lost_power(
        &mut self,
        floor_read_from_the_system_configurations: ModelCheckpointTxg,
    ) -> Result<ModelFloorLeftByALostPower, ModelDisagreement> {
        let effective_floor_before_the_lost_power = self.effective_rollback_floor();
        if floor_read_from_the_system_configurations == effective_floor_before_the_lost_power {
            return Ok(ModelFloorLeftByALostPower::KeptAtTheEffectiveFloor);
        }
        let ceiling = self.rollback_floor_ceiling();
        if ceiling == Some(floor_read_from_the_system_configurations) {
            self.floor_left_in_the_system_configurations_by_a_lost_power =
                floor_read_from_the_system_configurations;
            return Ok(ModelFloorLeftByALostPower::RaisedToTheCeiling);
        }
        Err(ModelDisagreement::new(
            ModelDisagreementAspect::RollbackFloor,
            format!(
                "掉电之后系统配置里的 F 只能是原值 {} 或这一刻的上限 {:?}（D16（发布语义） 已定项 1）",
                effective_floor_before_the_lost_power.0,
                ceiling.map(|ceiling_txg| ceiling_txg.0)
            ),
            format!(
                "盘上系统配置里读得出的 F 最大是 {}",
                floor_read_from_the_system_configurations.0
            ),
        ))
    }

    /// 按实例表判是不是被抛弃的：表里有它那个实例的行 (i, Ti, Wi) 且 txg > Ti（D23（journal 的角色与格式） 已定项 14）。
    /// 用哪一张表：最新那条根指着的（恢复与回退择的就是它；不建崩溃时它也是会话的现行版本）。
    pub(super) fn is_abandoned_by(root: &ModelRoot, table: &[ModelInstanceRow]) -> bool {
        table.iter().any(|row| {
            row.instance == root.key.instance && root.key.checkpoint_txg > row.selected_root_txg
        })
    }

    /// 抬 F 的上限（D16（发布语义） 已定项 1）：min(每块盘上最新的有效根, 第 4 新的非空有效根)；非空有效根不足 4 个时取最旧的有效根。
    /// 有效 = 按最新根的实例表不被抛弃 ∧ txg ≥ 现行的 F（「非空」从盘上怎么认；B2 的窗口那一项 RGall 拿掉）；现行的 F 取 F_生效（不建崩溃时与
    /// 现行根带的相等）。非空 = inode 树、extent 树的根指针与前一条有效根（按 (txg, 实例) 排）的不同，最旧的有效根与「没有这两棵树」比。
    /// 一块盘上没有有效根时不算它（条款没写这一格；不建崩溃时抬 F 与暖机都推到每块盘都有，走不到）。预想，跟收口表第 ② 行
    /// （候选集下界用哪个 F、被抛弃根带的 F 算不算都在那一行里重议）。
    /// 挂载那一刻读坏或读回全零的槽并进上限（「抬 F 的上限」那一行「并」）这里不并：模型不建故障，根环里落点公式
    /// 落到过的槽都住着模型写下的根，没有读回全零的槽可并（推的）；说谎的设备吞了根槽写留下的全零槽，故障注入按注入点认那一格时并
    /// （[`IdealModel::rollback_floor_ceilings_without_the_roots`]）。
    #[must_use]
    pub fn rollback_floor_ceiling(&self) -> Option<ModelCheckpointTxg> {
        let current_floor = self.effective_rollback_floor();
        let newest_table = Rc::clone(&self.newest_root().instance_table_rows);
        // 上限与「非空」在环里全部有效根上取：B2原来只取一圈窗口里的 RGall 与回收门槛一起退回整个环。
        let mut valid: Vec<&ModelRoot> = self
            .ring
            .values()
            .filter(|root| {
                !Self::is_abandoned_by(root, &newest_table)
                    && root.key.checkpoint_txg >= current_floor
            })
            .collect();
        valid.sort_by_key(|root| root.key);
        let mut newest_valid_per_device: BTreeMap<ModelDeviceIdentity, ModelCheckpointTxg> =
            BTreeMap::new();
        for root in &valid {
            let newest = newest_valid_per_device
                .entry(device_holding_the_root_of(root.key.checkpoint_txg))
                .or_insert(root.key.checkpoint_txg);
            *newest = (*newest).max(root.key.checkpoint_txg);
        }
        let newest_on_every_device = newest_valid_per_device.values().copied().min()?;
        // 「非空」比的是两棵用户可见树连同树表里快照那几对条目的根指针（`mount::user_visible_trees_changed`）：建快照那次与删快照的
        // 最后一批算非空，意图那次与不是最后一批的批不算（只改记号、前驱与 deadlist）；整理搬了这几个根单元之一的那一批算非空（根指针换成映射位置，
        // 「照字节比，搬根算非空」），签发、删意图与只搬别的单元的那几次不算。
        let mut non_empty: Vec<(ModelRootKey, ModelUserVisibleState)> = Vec::new();
        let mut previous_state = ModelUserVisibleState {
            trees_written_by: None,
            snapshot_roots: Vec::new(),
            root_relocations: Vec::new(),
        };
        for root in &valid {
            let state = root.user_visible_state();
            if state != previous_state {
                non_empty.push((root.key, state.clone()));
            }
            previous_state = state;
        }
        // 「4 个不同状态」去重（「非空」从盘上怎么认那一段末尾）：从新到旧数，状态（两棵用户可见树是哪次发布写的、快照条目抄的是谁）与比它新、
        // 已计入的根相同的不计入——管理员回退发出的根状态同回退目标。去重之后不足 4 个取最旧的有效根（根环容量边界）。
        non_empty.sort_unstable_by_key(|(key, _)| std::cmp::Reverse(*key));
        let mut counted_states: Vec<ModelUserVisibleState> = Vec::new();
        let mut distinct_newest_first: Vec<ModelCheckpointTxg> = Vec::new();
        for (key, state) in non_empty {
            if !counted_states.contains(&state) {
                counted_states.push(state);
                distinct_newest_first.push(key.checkpoint_txg);
            }
        }
        let fourth_newest_non_empty = match distinct_newest_first.get(3) {
            Some(fourth) => *fourth,
            None => valid.first()?.key.checkpoint_txg,
        };
        Some(newest_on_every_device.min(fourth_newest_non_empty))
    }

    /// 说谎的设备吞掉了 `removed` 那几条根的根槽写之后，实现从盘上现读根环算出的准入抬 F 的上限，两个数（故障注入按注入点认
    /// 「抬 F 的上限」与推空那两格，实七）：
    /// - 不并的那个：把那几条根从根环里拿掉，照 [`IdealModel::rollback_floor_ceiling`] 同一个算法算——实现看不见它们。
    /// - 并的那个：再并上那几条根留下的、这个会话挂载那一刻读回全零的槽各能住的最大 txg（不低于现行 F 的那几个，取最小；
    ///   D16（发布语义） 已定项 1「抬 F 的上限」那一行：读坏槽与读回全零的槽都并进上限「并」；
    ///   `IdealModel::highest_txgs_the_ring_slots_left_all_zero_by_the_roots_can_hold`）。没有这样的槽时两个数相同。
    ///
    /// 拿掉之后根环空了、或算不出上限时 `None`。
    #[must_use]
    pub fn rollback_floor_ceilings_without_the_roots(
        &self,
        removed: &[ModelRootKey],
    ) -> Option<ModelRollbackFloorCeilingsWithoutTheRoots> {
        let mut view_without_the_roots = self.clone();
        view_without_the_roots
            .ring
            .retain(|_, root| !removed.contains(&root.key));
        if view_without_the_roots.ring.is_empty() {
            return None;
        }
        let from_the_roots_read = view_without_the_roots.rollback_floor_ceiling()?;
        let current_floor = view_without_the_roots.effective_rollback_floor();
        let lowest_highest_txg_of_a_ring_slot_left_all_zero = self
            .highest_txgs_the_ring_slots_left_all_zero_by_the_roots_can_hold(removed)
            .into_iter()
            .filter(|highest_txg_it_can_hold| *highest_txg_it_can_hold >= current_floor)
            .min();
        Some(ModelRollbackFloorCeilingsWithoutTheRoots {
            from_the_roots_read,
            merging_the_ring_slots_left_all_zero: lowest_highest_txg_of_a_ring_slot_left_all_zero
                .map_or(from_the_roots_read, |highest_txg_it_can_hold| {
                    from_the_roots_read.min(highest_txg_it_can_hold)
                }),
        })
    }

    /// `removed` 那几条根（说谎的设备吞掉了它们的根槽写）里，在这个会话挂载那一刻读回全零、这个会话之后没写过的槽各能住的最大 txg
    /// （D16（发布语义） 已定项 1「环里最旧有效根」那一行「挂载时读不出的槽」那一句：(N − 3S, 新实例第一次发布的 txg − 1] 里落到这一槽的最大 txg；
    /// 实现 `crates/singlefs-core/src/mount.rs` 的 `txgs_ring_slots_bad_at_mount_can_hold`）。一条根算进来要三样都成立：
    /// - 它仍在模型的根环里：它那一槽之后没被别的根写过（写过的话盘上那一槽住着后来那条根，不是全零）；
    /// - 它不是这个会话的实例写的：这个会话写过、FUA 返回过的槽是「这个进程知道住着根的槽」，读坏重读仍坏就拒这次抬 F，不并
    ///   （故障注入认那一格按的是读坏的槽，[`ModelReportedUnreadablePlace::RootRingSlotStillBadAfterOneReread`]）；
    /// - 那一槽在它之前没住过根：txg 不小于根环区域数（区域槽 0 有 mkfs 种的第 0 代根）、小于环长 3S（再往后那一槽住着早一圈的根）。
    ///   模型不记一槽里被盖掉的旧根，早一圈的根读出来是什么不在这里算。
    ///
    /// 能住的最大 txg：不大于这个会话第一次发布的 txg 减一、落到这一槽的最大 txg（模型的 txg 逐次加一，那一槽之后没被写过，取到的就是它自己的 txg；
    /// 它也落在 N − 3S 之上，N 不大于那个 txg 减一）。这个会话第一次发布的 txg 取根环里这个会话的实例写的根里最小的那条（含被吞的）；
    /// 没有会话、或根环里一条这个会话的根都没有（那几条都被后来的盖掉，那一槽也就都写过了）时交回空。
    fn highest_txgs_the_ring_slots_left_all_zero_by_the_roots_can_hold(
        &self,
        removed: &[ModelRootKey],
    ) -> Vec<ModelCheckpointTxg> {
        let Some(session) = &self.session else {
            return Vec::new();
        };
        let Some(first_txg_of_the_session) = self
            .ring
            .values()
            .filter(|root| root.key.instance == session.instance)
            .map(|root| root.key.checkpoint_txg)
            .min()
        else {
            return Vec::new();
        };
        let ring_length_in_txgs = ROOT_RING_REGIONS * self.slots_per_region;
        self.ring
            .values()
            .filter(|root| removed.contains(&root.key))
            .filter(|root| root.key.instance != session.instance)
            .filter(|root| {
                (ROOT_RING_REGIONS..ring_length_in_txgs).contains(&root.key.checkpoint_txg.0)
            })
            .filter_map(|root| {
                let txgs_from_the_root_to_the_last_before_the_session = first_txg_of_the_session
                    .0
                    .checked_sub(1)?
                    .checked_sub(root.key.checkpoint_txg.0)?;
                Some(ModelCheckpointTxg(
                    root.key.checkpoint_txg.0
                        + ring_length_in_txgs
                            * (txgs_from_the_root_to_the_last_before_the_session
                                / ring_length_in_txgs),
                ))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 抬 F 的上限（D16（发布语义） 已定项 1）：有效根 txg 0–9，非空的是 3（有了文件）与 6–9（覆盖写），4、5（写行、暖机）照抄 3 的文件、
    /// 0–2 没有文件，都不算。第 4 新的非空根是 6；每块盘上最新的有效根：盘 0 是 9（区域 0），盘 1 是 7（区域 1）⇒ 上限 min(7, 6) = 6。
    /// 抬到 6 不拒，抬到 7 必须拒。
    #[test]
    fn the_ceiling_is_the_fourth_newest_non_empty_root_capped_by_the_newest_root_on_every_device() {
        let model = model_after_four_overwrites_in_a_second_instance();
        assert_eq!(model.rollback_floor_ceiling(), Some(ModelCheckpointTxg(6)));
        let at_the_ceiling = model
            .answer_raise_rollback_floor(ModelCheckpointTxg(6))
            .expect("会话开着");
        assert!(
            at_the_ceiling.required_refusals.is_empty(),
            "{at_the_ceiling:?}"
        );
        assert_eq!(
            at_the_ceiling
                .expected_roots
                .iter()
                .map(|root| root.key)
                .collect::<Vec<_>>(),
            vec![key(10, 2), key(11, 2)],
            "txg 10 落盘 1、txg 11 落盘 0：两块盘都有带新 F 的根就停"
        );
        let above = model
            .answer_raise_rollback_floor(ModelCheckpointTxg(7))
            .expect("会话开着");
        assert_eq!(
            above.required_refusals,
            BTreeSet::from([ModelRefusalReason::FloorAboveCeiling])
        );
    }

    /// 场景：实例 1 写第一个文件（txg 3）之后覆盖写九次（txg 4–12），txg 7 那次的根槽写被说谎的设备吞掉；再可写挂载（实例 2，写行 13、暖机 14）。
    /// 预期：拿掉 (7, 1) 之后只照读得出的根算，每块盘上最新的有效根取最小是 13、第 4 新的不同状态是 9，不并的上限 9；
    /// txg 7 那一槽（区域 1 槽 2，S = 8，环长 24 之内头一次被写）挂载那一刻读回全零，能住的最大 txg 7 不低于 F = 0，
    /// 并进来的上限 min(9, 7) = 7（D16（发布语义） 已定项 1「抬 F 的上限」那一行「并」）。
    /// 判别力：不并读回全零的槽时两个数都是 9（`crates/mutations.tsv`「故障注入的模型重算上限不并被吞根留下的全零槽」）。
    #[test]
    fn the_ceiling_without_a_root_swallowed_before_the_mount_merges_the_ring_slot_it_left_all_zero()
    {
        let mut model = two_device_model();
        model.acquire_and_warm_up_in_the_make_filesystem_process();
        let first = model
            .answer_publish_first_file(&[3])
            .expect("会话开着、现行 txg 2");
        succeed(&mut model, &first);
        for content_byte in 4_u8..=12 {
            let overwrite = model
                .answer_publish_overwrite(&[content_byte])
                .expect("会话开着、现行版本带文件");
            succeed(&mut model, &overwrite);
        }
        model.close_session();
        let mount = model.answer_mount_writable();
        succeed(&mut model, &mount);
        assert_eq!(
            model.newest_root().key,
            key(14, 2),
            "前提：实例 2 写行 13、暖机 14 之后现行那一版是 (14, 2)"
        );
        assert_eq!(
            model.rollback_floor_ceilings_without_the_roots(&[key(7, 1)]),
            Some(ModelRollbackFloorCeilingsWithoutTheRoots {
                from_the_roots_read: ModelCheckpointTxg(9),
                merging_the_ring_slots_left_all_zero: ModelCheckpointTxg(7),
            }),
            "不并的上限 min(13, 9) = 9，并进 txg 7 留下的全零槽之后 7"
        );
    }

    /// 抬 F 到 6 之后 F_生效 = 6：回退到 txg 6 的根是候选（txg ≥ F_生效，D16（发布语义） 已定项 1「回退候选集」），txg 5 的必须拒。
    /// 掉电留下的 F 还是原值：模型逐字段不动，这次掉电对 F 没有影响。判别力：只认上限时这里 F 0 不等于上限 6，判成对不上。
    #[test]
    fn lost_power_leaving_the_effective_floor_in_the_system_configurations_changes_nothing() {
        let mut model = model_after_four_overwrites_in_a_second_instance();
        let model_before = model.clone();
        assert_eq!(
            model.settle_floor_left_in_the_system_configurations_by_a_lost_power(
                ModelCheckpointTxg(0)
            ),
            Ok(ModelFloorLeftByALostPower::KeptAtTheEffectiveFloor)
        );
        assert_eq!(model, model_before);
    }

    /// 掉电留下的 F 是上限 6（根环里没有一条根带着它）：它成为 F_生效，下一个实例的第一条根带着它。
    /// 判别力：不记下，或 F_生效 不取这一半，F_生效 还是 0，两条断言都红。
    #[test]
    fn lost_power_leaving_the_ceiling_in_the_system_configurations_carries_it_into_the_next_instance(
    ) {
        let mut model = model_after_four_overwrites_in_a_second_instance();
        assert_eq!(model.effective_rollback_floor(), ModelCheckpointTxg(0));
        assert_eq!(model.rollback_floor_ceiling(), Some(ModelCheckpointTxg(6)));
        assert_eq!(
            model.settle_floor_left_in_the_system_configurations_by_a_lost_power(
                ModelCheckpointTxg(6)
            ),
            Ok(ModelFloorLeftByALostPower::RaisedToTheCeiling)
        );
        assert_eq!(model.effective_rollback_floor(), ModelCheckpointTxg(6));
        model.close_session();
        let mount = model.answer_mount_writable();
        assert_eq!(
            mount
                .expected_roots
                .first()
                .expect("可写挂载至少写行那一条根")
                .rollback_floor,
            ModelCheckpointTxg(6)
        );
    }

    /// 系统配置里的 F 超过模型算的上限（实现抬过了上限）：判成对不上，模型不动。判别力：判据放宽成不低于上限时这一条红。
    #[test]
    fn lost_power_leaving_a_floor_above_the_ceiling_disagrees_and_leaves_the_model_as_it_was() {
        let mut model = model_after_four_overwrites_in_a_second_instance();
        let model_before = model.clone();
        let disagreement = model
            .settle_floor_left_in_the_system_configurations_by_a_lost_power(ModelCheckpointTxg(7))
            .expect_err("上限是 6，7 抬过了上限");
        assert_eq!(disagreement.aspect, ModelDisagreementAspect::RollbackFloor);
        assert_eq!(model, model_before);
    }

    /// 系统配置里的 F 在原值与上限之间（抬 F 那一串只抬到上限，不抬到一半）：判成对不上。判别力：判据放宽成不超过上限时这一条红。
    #[test]
    fn lost_power_leaving_a_floor_between_the_effective_floor_and_the_ceiling_disagrees() {
        let mut model = model_after_four_overwrites_in_a_second_instance();
        let disagreement = model
            .settle_floor_left_in_the_system_configurations_by_a_lost_power(ModelCheckpointTxg(3))
            .expect_err("原值 0、上限 6，3 两个都不是");
        assert_eq!(disagreement.aspect, ModelDisagreementAspect::RollbackFloor);
    }

    /// 系统配置里的 F 低于 F_生效（根上的 F 先写进系统配置，同盘根上的 F 不高于它）：判成对不上。判别力：判据放宽成不超过原值时这一条红。
    #[test]
    fn lost_power_leaving_a_floor_below_the_effective_floor_disagrees() {
        let mut model = model_after_four_overwrites_in_a_second_instance();
        let raise = model
            .answer_raise_rollback_floor(ModelCheckpointTxg(6))
            .expect("会话开着");
        succeed(&mut model, &raise);
        assert_eq!(model.effective_rollback_floor(), ModelCheckpointTxg(6));
        let disagreement = model
            .settle_floor_left_in_the_system_configurations_by_a_lost_power(ModelCheckpointTxg(5))
            .expect_err("原值 6，5 低于它");
        assert_eq!(disagreement.aspect, ModelDisagreementAspect::RollbackFloor);
    }

    /// 「4 个不同状态」去重（D16（发布语义） 已定项 1「「非空」从盘上怎么认」末段）：实例 2 覆盖写到 txg 9（各一个状态）之后回退到 (7, 2)，
    /// 回退写的 (10, 2) 状态同 txg 7。从新到旧计入 10、9、8，7 与 10 同一个状态不计，第 4 个是 6；每块盘上最新的有效根取小是 9
    /// （盘 0 最新 9、盘 1 最新 10）⇒ 上限 6。不去重时第 4 个是 7。
    #[test]
    fn after_a_forward_rollback_the_ceiling_counts_the_repeated_state_once() {
        let mut model = model_after_four_overwrites_in_a_second_instance();
        let rollback = model
            .answer_rollback_while_mounted(key(7, 2))
            .expect("会话开着、现行版本带文件");
        succeed(&mut model, &rollback);
        assert_eq!(model.rollback_floor_ceiling(), Some(ModelCheckpointTxg(6)));
    }

    /// F_生效 = 各幸存盘最新持久有效根所带 F 的最大值（D16（发布语义） 已定项 1「生效」）：抬 F 半路停在第一次（txg 10 落盘 1）时，
    /// 盘 1 上最新的有效根就带着 6，F_生效 已是 6。
    #[test]
    fn raised_floor_carried_by_the_newest_root_of_one_device_takes_effect() {
        let mut model = model_after_four_overwrites_in_a_second_instance();
        let raise = model
            .answer_raise_rollback_floor(ModelCheckpointTxg(6))
            .expect("会话开着");
        let first_publish_only = raise.expected_roots[..1].to_vec();
        for root in first_publish_only {
            model.write_root(root);
        }
        assert_eq!(model.effective_rollback_floor(), ModelCheckpointTxg(6));
    }

    /// 搬根算非空（「照字节比，搬根算非空」）：签发与只搬别的单元的一批不换状态，搬了活头 inode 树根的那一批换了状态，
    /// 抬 F 的上限跟着从第 4 新的非空根 6 挪到 7（不算它时还是 6）；搬一个这一版不引用的根单元模型答不了。
    #[test]
    fn compaction_publish_relocating_a_user_visible_root_is_a_new_state_for_the_ceiling() {
        let mut model = model_after_four_overwrites_in_a_second_instance();
        let state_of = |judged_model: &IdealModel| {
            judged_model
                .current_version_in_the_session()
                .expect("会话开着")
                .user_visible_state()
        };
        let before = state_of(&model);
        assert_eq!(
            model.rollback_floor_ceiling(),
            Some(ModelCheckpointTxg(6)),
            "非空 9、8、7、6、3：第 4 新的是 6"
        );
        succeed_with_the_compaction_publish(
            &mut model,
            &compaction_publish_seen(0, true, false, Vec::new()),
        );
        succeed_with_the_compaction_publish(
            &mut model,
            &compaction_publish_seen(2, true, false, Vec::new()),
        );
        assert_eq!(state_of(&model), before, "签发、只搬别的单元：状态不变");
        assert_eq!(model.rollback_floor_ceiling(), Some(ModelCheckpointTxg(6)));
        let inode_root_of_the_live_head = ModelUserVisibleRootUnit {
            tree: ModelUserVisibleTree::InodeTree,
            birth_txg: ModelCheckpointTxg(9),
        };
        let unreferenced_root = ModelUserVisibleRootUnit {
            tree: ModelUserVisibleTree::InodeTree,
            birth_txg: ModelCheckpointTxg(3),
        };
        assert_eq!(
            aspect_of_the_compaction_publish(
                &model,
                &compaction_publish_seen(1, true, false, vec![unreferenced_root])
            ),
            Err(ModelDisagreementAspect::NotModeled),
            "这一版不引用 txg 3 写的 inode 树根"
        );
        let relocated = succeed_with_the_compaction_publish(
            &mut model,
            &compaction_publish_seen(1, true, false, vec![inode_root_of_the_live_head]),
        );
        assert_eq!(
            relocated
                .user_visible_root_relocations
                .get(&inode_root_of_the_live_head),
            Some(&ModelCheckpointTxg(12)),
            "搬它的那一次记进去"
        );
        assert_ne!(state_of(&model), before, "搬了 inode 树根：非空");
        assert_eq!(
            model.rollback_floor_ceiling(),
            Some(ModelCheckpointTxg(7)),
            "非空 12、9、8、7：第 4 新的是 7"
        );
    }
}

//! 模型对可写挂载、回退、抬 F、卸载与冷启动的回答。

use super::*;

impl IdealModel {
    /// 可写挂载（`mount::mount_writable`）：所选根 = 环里最新的那条（不建崩溃，没有要施加的记录）。调之前先 `close_session`。
    #[must_use]
    pub fn answer_mount_writable(&self) -> ModelAnswer {
        let chosen = self.newest_root().clone();
        let previous_row = ModelInstanceRow {
            instance: chosen.key.instance,
            selected_root_txg: chosen.key.checkpoint_txg,
            // W = 这次恢复施加的记录里最大的事务号（D23（journal 的角色与格式） 已定项 14 第 4 条）：不建崩溃，一条都不施加。
            applied_transaction_high_water: 0,
        };
        self.answer_establishing_an_instance(
            ModelOperationKind::MountWritable,
            &chosen,
            Some(previous_row),
        )
    }

    /// 崩溃恢复抛弃根（执行器的 `HistoryOperation::CrashRecoveryAbandoningTheNewestRoot`）：挂载期间环里最新那条根的根槽与它那次发布
    /// 点名的单元整次挂载读回全 0。那条根是这个进程发的，系统配置在它的根槽 FUA 之后轮换（D16（发布语义） 已定项 7），见证着它；
    /// 读阶段落到它前一条根、判出系统配置见证过比所选那一版新的发布，重读一次仍全 0，取号之前拒可写（C554 乙，
    /// `research/prompts/c554-fix-forks.md` #1「挂载时有一条根（或它引用的账）暂时读不出」；重读一次取自 D16（发布语义） 已定项 1「根槽这一次读坏」那一行）：一个字节都不写，
    /// 模型状态不变（会话照旧关着，根环、F、取过的号都不动）。系统配置没见证到最新那条根的那一形（它那次发布的轮换没落盘）乙罩不到、
    /// 照旧抛弃它，这一步造不出，模型不答。调之前先 `close_session`。
    ///
    /// # Errors
    /// 环里只有一条根：读阶段没有前一条可落（执行器在盘上的根环少于两条时不调，这一支只在模型与盘上的根环对不上时走到）。
    pub fn answer_mount_writable_with_the_newest_root_unreadable(
        &self,
    ) -> Result<ModelAnswer, ModelDisagreement> {
        let newest = self.newest_root().key;
        if self.ring.len() < 2 {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::NotModeled,
                format!("模型根环里只有 {newest:?} 一条根，崩溃恢复没有前一条根可落"),
                "执行器按盘上的根环调了崩溃恢复抛弃根".to_string(),
            ));
        }
        let mut answer = self.answer_for_publishes(
            ModelOperationKind::MountWritable,
            BTreeSet::from([
                ModelRefusalReason::NewerPublishWitnessedBySystemConfigurationStillUnreadableAfterOneReread,
            ]),
            Vec::new(),
        );
        // 可写挂载在读阶段判完、取号之前拒：拒之前一个写都不发（实现的契约：`NewerStateStillUnreadableAfterOneReread` 取号之前交回）。
        answer.walls_are_judged_before_the_first_write = true;
        Ok(answer)
    }

    /// 可写挂载，而每块盘自证过、世代号最大的那一槽系统配置整次挂载读回全 0（执行器的
    /// `HistoryOperation::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros`）：读回全 0 的槽读得出而自证不过、
    /// 设备不报读错，每块盘只剩一槽自证过、没有一块盘两槽都自证过。乙入判不出（C971（多处读坏时可写挂载写出倒挂的新实例根），
    /// `research/prompts/e205-preregistration.md`「乙入判不出」那一行）把这几槽并进 R2「有槽设备报读错」那一半当读错计，判据 N-配置 按真处置；
    /// 重读一次仍读回全 0，取号之前拒可写（`NewerStateStillUnreadableAfterOneReread`）：一个字节都不写，模型状态不变（会话照旧关着，
    /// 根环、F、取过的号都不动）。拒不看根环里有没有比所选那一版新的发布，所以不比环。调之前先 `close_session`。
    #[must_use]
    pub fn answer_mount_writable_with_no_device_holding_two_self_verified_system_configuration_slots(
        &self,
    ) -> ModelAnswer {
        let required_refusals = BTreeSet::from([
            ModelRefusalReason::NewerPublishWitnessedBySystemConfigurationStillUnreadableAfterOneReread,
        ]);
        let mut answer = self.answer_for_publishes(
            ModelOperationKind::MountWritable,
            required_refusals,
            Vec::new(),
        );
        // 读阶段判完、取号之前就拒：拒之前一个写都不发（实现的契约：`NewerStateStillUnreadableAfterOneReread` 取号之前交回）。
        answer.walls_are_judged_before_the_first_write = true;
        answer
    }

    /// 可写挂载，而 `roots_whose_root_slot_is_missing` 那几条根的根槽写没落盘（说谎的设备吞了；它们的记录与单元都落了）：
    /// 择根落到除它们之外 (txg, 实例) 最大的那一条（所选根），恢复再施加 (实例代号, checkpoint_txg) 严格大于所选根、与所选根同实例的
    /// 那几次发布的记录（前缀不跨实例边界，D23（journal 的角色与格式） 已定项 14 注 1），由记录重建出它们；上一个实例那一行的 T 取
    /// 实际走到的那条根，W 取施加的那几次发布的记录里最大的事务号（已定项 14 第 4 条；空发布的事务号 0 不进 max，已定项 19 ①）。
    /// 名单里的根不在模型的根环里、或比所选根旧的，不改答案。调之前先 `close_session`。
    /// 故障注入拿它认「写的实例表行」那一形（实八）：实现照条款写 W，[`IdealModel::answer_mount_writable`] 不建崩溃、答 W = 0。
    ///
    /// # Errors
    /// 名单把根环里的根拿光了：择根没有一条可落。
    pub fn answer_mount_writable_with_the_roots_rebuilt_from_their_records(
        &self,
        roots_whose_root_slot_is_missing: &[ModelRootKey],
    ) -> Result<ModelAnswer, ModelDisagreement> {
        let mut view_without_the_missing_roots = self.clone();
        view_without_the_missing_roots
            .ring
            .retain(|_, root| !roots_whose_root_slot_is_missing.contains(&root.key));
        if view_without_the_missing_roots.ring.is_empty() {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::NotModeled,
                format!(
                    "模型根环里的根全在根槽没落盘的名单里（{roots_whose_root_slot_is_missing:?}），择根没有一条可落"
                ),
                "执行器调了可写挂载".to_string(),
            ));
        }
        let selected = view_without_the_missing_roots.newest_root().clone();
        let mut newer_than_the_selected: Vec<&ModelRoot> = self
            .ring
            .values()
            .filter(|root| root.key > selected.key)
            .collect();
        newer_than_the_selected.sort_by_key(|root| root.key);
        // 迭代上界是根环的槽数；跨轮带的是走到的那条根与 W；提前出口只有「下一条换了实例」（前缀不跨实例边界）。
        let mut effective = &selected;
        let mut applied_transaction_high_water = 0;
        for rebuilt in newer_than_the_selected {
            if rebuilt.key.instance != selected.key.instance {
                break;
            }
            applied_transaction_high_water = applied_transaction_high_water
                .max(rebuilt.highest_transaction_number_of_its_publish);
            effective = rebuilt;
        }
        let previous_row = ModelInstanceRow {
            instance: effective.key.instance,
            selected_root_txg: effective.key.checkpoint_txg,
            applied_transaction_high_water,
        };
        Ok(self.answer_establishing_an_instance(
            ModelOperationKind::MountWritable,
            effective,
            Some(previous_row),
        ))
    }

    /// 管理员回退（`mount::roll_back_by_a_forward_publish`，D23（journal 的角色与格式） 已定项 14）：挂着时的一次向前发布。
    /// 候选集 = 根环里按现行那一版的实例表判仍然有效 ∧ txg ≥ F_生效 ∧ 带文件；做成时写一条根——txg 与 jsn 接着现行那一版，
    /// 实例、实例表、F 照现行那一版的，文件与用户可见的几个角色（数据、extent 根、inode 叶与根）的分配代取回退目标那一版的，
    /// 固定点单元（分配记录树、记账树、映射树、树表）这次重写。模型不建坏盘，「在任何写之前拒」的账与单元那几格走不到。
    ///
    /// # Errors
    /// 模型里没有可写会话，或现行版本没有文件（执行器只在带文件时调）。
    pub fn answer_rollback_while_mounted(
        &self,
        target: ModelRootKey,
    ) -> Result<ModelAnswer, ModelDisagreement> {
        let session = self.open_session()?;
        let current = &session.current;
        if !current
            .contents
            .is_a_version_with_a_file_or_a_file_being_deleted()
        {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::NotModeled,
                format!(
                    "模型里现行版本 {:?} 没有文件：{:?}",
                    current.key, current.contents
                ),
                "实现的现行版本带文件（执行器调了回退）".to_string(),
            ));
        }
        let mut required_refusals = BTreeSet::new();
        // 第一版有快照就拒回退（判决 S8、岔路 3）：现行那一版带着快照（销毁中的也算）时在任何读之前拒，别的候选判据不再看。
        if !current.snapshots.is_empty() {
            required_refusals
                .insert(ModelRefusalReason::RollbackWithSnapshotsWhichTheFirstVersionRefuses);
            let mut answer = self.answer_for_publishes(
                ModelOperationKind::RollbackWhileMounted,
                required_refusals,
                Vec::new(),
            );
            answer.rollback_target = Some(target);
            answer.walls_are_judged_before_the_first_write = true;
            return Ok(answer);
        }
        let target_root = self.root_with_key(target);
        match target_root {
            None => {
                required_refusals.insert(ModelRefusalReason::RollbackTargetNotInRing);
            }
            Some(target_root) => {
                if target.checkpoint_txg < self.effective_rollback_floor() {
                    required_refusals.insert(ModelRefusalReason::RollbackTargetBelowEffectiveFloor);
                }
                if Self::is_abandoned_by(target_root, &current.instance_table_rows) {
                    required_refusals.insert(ModelRefusalReason::RollbackTargetOnAbandonedTimeline);
                }
                if !target_root.contents.has_trees() {
                    required_refusals.insert(ModelRefusalReason::RollbackTargetWithoutFile);
                }
                // 目标那一版带快照：实现在候选集那一判之后、把目标那一版重建出来再判（`mount::roll_back_by_a_forward_publish`），
                // 所以只在候选判据都过了时才是这一条。
                if required_refusals.is_empty() && !target_root.snapshots.is_empty() {
                    required_refusals.insert(
                        ModelRefusalReason::RollbackWithSnapshotsWhichTheFirstVersionRefuses,
                    );
                }
            }
        }
        let roots = match (target_root, required_refusals.is_empty()) {
            (Some(target_root), true) => {
                let rollback_txg = ModelCheckpointTxg(current.key.checkpoint_txg.0 + 1);
                let rollback_root = self.next_root_with_the_user_visible_trees(
                    current,
                    session.instance,
                    rollback_txg,
                    ModelJournalCounter(self.highest_journal_counter.0 + 1),
                    ModelPublishKind::EmptyOnFileVersion,
                    current.rollback_floor,
                    None,
                    Rc::clone(&current.instance_table_rows),
                    UserVisibleTreesOfTheNewVersion::TakenFromTheRollbackTarget(target_root),
                    InodesOtherThanTheFirstChangeOfThePublish::Carried,
                );
                vec![rollback_root]
            }
            (Some(_) | None, _) => Vec::new(),
        };
        let mut answer = self.answer_for_publishes(
            ModelOperationKind::RollbackWhileMounted,
            required_refusals,
            roots,
        );
        answer.rollback_target = Some(target);
        // 回退那一步在任何写之前判完它的拒（候选集、账、单元），做成就是一次发布。
        answer.walls_are_judged_before_the_first_write = true;
        Ok(answer)
    }

    /// 可写挂载与回退共用的后半段：取号、写行那次发布、暖机到本实例的根覆盖每块盘（D16（发布语义） 已定项 8 戊，至多 R 次）。
    /// `previous_row` 为 None 只在 mkfs 同一个进程里（不写行）。
    pub(super) fn answer_establishing_an_instance(
        &self,
        operation: ModelOperationKind,
        base: &ModelRoot,
        previous_row: Option<ModelInstanceRow>,
    ) -> ModelAnswer {
        let instance = ModelInstanceGeneration(self.highest_acquired_instance.0 + 1);
        // 行：给 [max(上一个实例, 1), 新实例) 里每个实例一行——上一个实例那一行，中间实例 (i, 0, 0)；实例 0 不写行
        // （D18（块里携带什么信息） 已定项 11；D23（journal 的角色与格式） 已定项 14）。
        let rows_to_write: Vec<ModelInstanceRow> = match previous_row {
            None => Vec::new(),
            Some(previous_row) => (previous_row.instance.0.max(1)..instance.0)
                .map(|row_instance| {
                    if row_instance == previous_row.instance.0 {
                        previous_row
                    } else {
                        ModelInstanceRow {
                            instance: ModelInstanceGeneration(row_instance),
                            selected_root_txg: ModelCheckpointTxg(0),
                            applied_transaction_high_water: 0,
                        }
                    }
                })
                .collect(),
        };
        // 新实例的第一次发布的 txg 与 jsn：环里全部根与全部记录里的最大值 + 1（D23（journal 的角色与格式） 已定项 14 第 3 条）。
        let first_txg = ModelCheckpointTxg(self.highest_published_txg.0 + 1);
        let first_journal_counter = ModelJournalCounter(self.highest_journal_counter.0 + 1);
        // 新实例的根带的 F = 恢复后生效的 F（D16（发布语义） 已定项 1「生效」）：预想，跟收口表第 ② 行。
        let rollback_floor = self.effective_rollback_floor();
        let required_refusals = BTreeSet::new();
        let has_trees = base.contents.has_trees();
        // 树表 0 条、而这一版的实例表已经不是 mkfs 那一片（上一次挂载在这一版上写过行）：写行那次发布建起这一版自己的分配记录树、
        // 根指针住根记录（C512（树表 0 条的一版上被换下的实例表记在哪没有条款）），账取得回来，模型一条拒绝都不列——实现要是在这一格上拒，
        // 对拍当场报「模型说该成、实现拒了」。实现拒在 `MountError::VersionWithoutFileUnitPointerOfABadImage` 的只有
        // 「树表或实例表不是 mkfs 写的那一版、而这一版又没有自己的分配记录树」那种手造坏镜像，随机历史里造不出来。
        // 实例表多于一片不再拒（尾片先、一片写满 369 行再开下一片）：写行那次发布整条链重写，
        // 有几片就多几个实例表单元（`next_root` 按这次之后的行数现算片数）。
        let table_after: Rc<Vec<ModelInstanceRow>> = if rows_to_write.is_empty() {
            Rc::clone(&base.instance_table_rows)
        } else {
            let mut rows = base.instance_table_rows.as_ref().clone();
            rows.extend_from_slice(&rows_to_write);
            Rc::new(rows)
        };
        let (row_kind, warm_up_kind) = match (has_trees, rows_to_write.is_empty()) {
            (true, _) => (
                ModelPublishKind::RowsOnFileVersion,
                ModelPublishKind::EmptyOnFileVersion,
            ),
            // 树表 0 条、要写的行不为空：只重写实例表那一个单元；之后的暖机仍是零单元。
            (false, false) => (
                ModelPublishKind::RowsOnVersionWithoutFile,
                ModelPublishKind::ZeroUnit,
            ),
            (false, true) => (ModelPublishKind::ZeroUnit, ModelPublishKind::ZeroUnit),
        };
        let mut roots = vec![self.next_root(
            base,
            instance,
            first_txg,
            first_journal_counter,
            row_kind,
            rollback_floor,
            None,
            Rc::clone(&table_after),
        )];
        let mut covered: BTreeSet<ModelDeviceIdentity> =
            BTreeSet::from([device_holding_the_root_of(first_txg)]);
        let mut warm_up_publishes: u64 = 0;
        while self
            .geometry
            .devices
            .iter()
            .any(|device| !covered.contains(device))
            && warm_up_publishes < ROOT_RING_REGIONS
        {
            let previous = roots.last().expect("至少有写行那一次").clone();
            let next_txg = ModelCheckpointTxg(previous.key.checkpoint_txg.0 + 1);
            covered.insert(device_holding_the_root_of(next_txg));
            roots.push(self.next_root(
                &previous,
                instance,
                next_txg,
                ModelJournalCounter(previous.journal_counter.0 + 1),
                warm_up_kind,
                rollback_floor,
                None,
                Rc::clone(&table_after),
            ));
            warm_up_publishes += 1;
        }
        let mut answer = self.answer_for_publishes(operation, required_refusals, roots);
        answer.walls_are_judged_before_the_first_write = true;
        answer.expected_mount = Some((instance, rows_to_write));
        answer.session_after_success = ModelSessionAfterSuccess::OpenedAs(instance);
        answer
    }

    /// 抬 F（`mount::raise_rollback_floor`）到 `new_floor`：推空发布直到每块盘上都有带新 F 的根（D16（发布语义） 已定项 1「生效」），
    /// 至多 R 次。
    ///
    /// # Errors
    /// 模型里没有可写会话或现行版本没有文件（执行器只在带文件时调）；`new_floor` 低于现行的 F（条款没写往下抬，生成器只取现行 F 及以上，
    /// 走到这里说明模型与实现的 F 已经对不上）。
    pub fn answer_raise_rollback_floor(
        &self,
        new_floor: ModelCheckpointTxg,
    ) -> Result<ModelAnswer, ModelDisagreement> {
        let session = self.open_session()?;
        let current = &session.current;
        // 抬 F 的空发布要重写固定点单元，接在有树的一版后面：执行器只在带文件（含删除中）的一版上调，
        // 挂载与会话为空间推的那几串还会接在有树而文件集合为空的一版后面。
        if !current.contents.has_trees() {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::SessionState,
                format!("模型里现行版本 {:?} 树表 0 条", current.key),
                "实现的现行版本有树（执行器调了抬 F，或挂载、会话推了抬 F）".to_string(),
            ));
        }
        if new_floor < current.rollback_floor {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::NotModeled,
                format!(
                    "往下抬 F（{} < 现行 {}）条款没写，模型不答",
                    new_floor.0, current.rollback_floor.0
                ),
                "执行器按实现的现行 F 取了目标".to_string(),
            ));
        }
        let ceiling = self.rollback_floor_ceiling();
        let mut required_refusals = BTreeSet::new();
        match ceiling {
            Some(ceiling) if new_floor > ceiling => {
                required_refusals.insert(ModelRefusalReason::FloorAboveCeiling);
            }
            Some(_) => {}
            None => {
                return Err(ModelDisagreement::new(
                    ModelDisagreementAspect::NotModeled,
                    "模型里一条有效根都没有，上限算不出".to_string(),
                    "实现调了抬 F".to_string(),
                ))
            }
        }
        let roots = self.empty_publishes_carrying_the_floor_to_every_device(session, new_floor);
        let mut answer = self.answer_for_publishes(
            ModelOperationKind::RaiseRollbackFloor,
            required_refusals,
            roots,
        );
        answer.rollback_floor_ceiling = ceiling;
        // 实现在任何写之前把这一串整串预演一遍（`mount::raise_rollback_floor`），哪一次撞墙都在第一次写之前拒、一次都不发。
        answer.walls_are_judged_before_the_first_write = true;
        Ok(answer)
    }

    /// 抬 F 那一串（D16（发布语义） 已定项 1「抬 F 那一串」：准入与卸载共用）接在会话现行那一版后面写出的根：带 `new_floor` 的空发布
    /// （有树的一版上重写固定点单元），推到每块盘上都有一条为止（「生效」那一行），至多 R 次。
    /// 先写系统配置那一步不属于任何一次发布（已定项 7），模型不建系统配置，这里只有根。
    fn empty_publishes_carrying_the_floor_to_every_device(
        &self,
        session: &ModelSession,
        new_floor: ModelCheckpointTxg,
    ) -> Vec<ModelRoot> {
        let current = &session.current;
        let mut roots: Vec<ModelRoot> = Vec::new();
        let mut covered: BTreeSet<ModelDeviceIdentity> = BTreeSet::new();
        // 迭代上界是根环区域数 R；跨轮带的是已写出的根与已罩到的盘；没有提前出口。
        while self
            .geometry
            .devices
            .iter()
            .any(|device| !covered.contains(device))
            && u64::try_from(roots.len()).expect("次数") < ROOT_RING_REGIONS
        {
            let previous = roots.last().unwrap_or(current).clone();
            let next_txg = ModelCheckpointTxg(previous.key.checkpoint_txg.0 + 1);
            covered.insert(device_holding_the_root_of(next_txg));
            let journal_counter = if roots.is_empty() {
                ModelJournalCounter(self.highest_journal_counter.0 + 1)
            } else {
                ModelJournalCounter(previous.journal_counter.0 + 1)
            };
            roots.push(self.next_root(
                &previous,
                session.instance,
                next_txg,
                journal_counter,
                ModelPublishKind::EmptyOnFileVersion,
                new_floor,
                None,
                Rc::clone(&previous.instance_table_rows),
            ));
        }
        roots
    }

    /// 正常卸载（`mount::unmount`，D16（发布语义） 已定项 1「正常卸载」）：推一串空发布把 F 抬到现行那一版的 txg，走抬 F 那一串
    /// （推到每块盘上都有一条带新 F 的根，第一版几何上 2 或 3 次），不另判上限（「抬 F 的上限」那一行「卸载抬 F」）；
    /// 做完之后这个进程不再有可写会话。
    ///
    /// 现行那一版树表 0 条（还没写过文件）：一个字节都不写、F 不动，答零条根、会话关
    /// （选项「乙：不推，条款照实现写」；与 `mount::Unmounted::NothingWrittenOnAVersionWithoutFile` 对得上）。
    /// 回退候选要带文件，而这一版在它的时间线上还没有带文件的版本，抬 F 买不到东西。
    ///
    /// 模型不比的两样：这一串的根在 flags 里带卸载记号（D22（单元原子性怎么合成） 已定项 7）——[`ObservedRoot`] 不带 flags；
    /// 先写系统配置那一步——模型不建系统配置。容量墙同抬 F：实现在任何写之前把这一串整串预演一遍，模型答允许拒绝的区间。
    ///
    /// # Errors
    /// 模型里没有可写会话：会话状态与实现对不上。
    pub fn answer_unmount(&self) -> Result<ModelAnswer, ModelDisagreement> {
        let session = self.open_session()?;
        let current = &session.current;
        let roots = if current.contents.has_trees() {
            let floor_raised_to_the_current_txg = current.key.checkpoint_txg;
            self.empty_publishes_carrying_the_floor_to_every_device(
                session,
                floor_raised_to_the_current_txg,
            )
        } else {
            Vec::new()
        };
        let mut answer =
            self.answer_for_publishes(ModelOperationKind::Unmount, BTreeSet::new(), roots);
        // 实现在任何写之前把这一串整串预演一遍（`mount::unmount` 走的就是抬 F 那一串），哪一次撞墙都在第一次写之前拒、一次都不发。
        answer.walls_are_judged_before_the_first_write = true;
        answer.session_after_success = ModelSessionAfterSuccess::Closed;
        Ok(answer)
    }

    /// 冷启动读回（`recovery::recover`，看 journal）：所选根 = 环里 (txg, 实例) 最大的那条，读回它那一版的文件。调之前先 `close_session`。
    #[must_use]
    pub fn answer_cold_start_recover(&self) -> ModelAnswer {
        let newest = self.newest_root();
        let read_back = match newest.contents.live_file() {
            Some(file) => ModelReadBack::FileRead {
                root: newest.key,
                content: Rc::clone(&file.content),
            },
            None => ModelReadBack::NoFile { root: newest.key },
        };
        ModelAnswer {
            operation: ModelOperationKind::ColdStartRecover,
            required_refusals: BTreeSet::new(),
            expected_roots: Vec::new(),
            expected_mount: None,
            expected_read_back: Some(read_back),
            rollback_floor_ceiling: None,
            rollback_target: None,
            planned_publish_upper_bounds: Vec::new(),
            walls_are_judged_before_the_first_write: true,
            session_after_success: ModelSessionAfterSuccess::Closed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 实例表多于一片（尾片先、一片写满 369 行再开下一片）：可写挂载不再拒，写行那次发布整条链重写，
    /// 链上每一片都是一个重写的单元、各占两个槽。第一个文件之后号推到 370（等于连着 369 次取号之后崩溃），
    /// 下一次可写挂载取 371、写 [1, 371) 共 370 行 ⇒ 两片：写行那次发布占 2 × 2 + 3（记账树、映射树、树表）槽，
    /// 加分配记录树这次至多重写的节点数。
    #[test]
    fn mount_that_writes_more_rows_than_one_page_holds_counts_every_page_of_the_instance_table_chain(
    ) {
        let mut model = two_device_model();
        model.acquire_and_warm_up_in_the_make_filesystem_process();
        let first = model
            .answer_publish_first_file(&[3])
            .expect("会话开着、现行 txg 2");
        succeed(&mut model, &first);
        model.close_session();
        model.highest_acquired_instance = ModelInstanceGeneration(370);
        let mount = model.answer_mount_writable();
        assert!(mount.required_refusals.is_empty(), "{mount:?}");
        let (instance, rows_written) = mount.expected_mount.clone().expect("挂载做成");
        assert_eq!(
            (instance, rows_written.len()),
            (ModelInstanceGeneration(371), 370)
        );
        let row_publish = &mount.expected_roots[0];
        assert_eq!(row_publish.instance_table_rows.len(), 370);
        let slots_before = model.newest_root().occupied_slots_upper_bound_per_device;
        let slots_outside_the_allocation_record_tree = 2 * 2 + 3;
        assert_eq!(
            row_publish.occupied_slots_upper_bound_per_device - slots_before,
            slots_outside_the_allocation_record_tree
                + model.allocation_record_tree_nodes_rewritten_upper_bound(
                    slots_before + slots_outside_the_allocation_record_tree
                ),
            "两片实例表各 2 槽、三个固定点单元各 1 槽，加分配记录树这次至多重写的节点"
        );
    }

    /// 回退是挂着时的一次向前发布（D23（journal 的角色与格式） 已定项 14）：写一条根 (12, 2)，实例、F 照现行那一版，文件是 txg 6 写的那一版、
    /// 数据单元的分配代写回 6；实例 2 的 7–11 不被抛弃，之后再回退到 (9, 2) 照样是候选。冷启动读回最新那条根——回退写的那一版。
    #[test]
    fn the_forward_rollback_to_the_effective_floor_publishes_one_root_and_leaves_the_newer_roots_candidates(
    ) {
        let mut model = model_after_four_overwrites_in_a_second_instance();
        let raise = model
            .answer_raise_rollback_floor(ModelCheckpointTxg(6))
            .expect("会话开着");
        succeed(&mut model, &raise);
        assert_eq!(model.effective_rollback_floor(), ModelCheckpointTxg(6));
        assert_eq!(
            model
                .answer_rollback_while_mounted(key(5, 2))
                .expect("会话开着、现行版本带文件")
                .required_refusals,
            BTreeSet::from([ModelRefusalReason::RollbackTargetBelowEffectiveFloor])
        );
        assert_eq!(
            model
                .answer_rollback_while_mounted(key(2, 1))
                .expect("会话开着、现行版本带文件")
                .required_refusals,
            BTreeSet::from([
                ModelRefusalReason::RollbackTargetBelowEffectiveFloor,
                ModelRefusalReason::RollbackTargetWithoutFile
            ]),
            "暖机根 (2, 1) 在 F 之下、也没有文件"
        );
        let rollback = model
            .answer_rollback_while_mounted(key(6, 2))
            .expect("会话开着、现行版本带文件");
        assert_eq!(
            rollback
                .expected_roots
                .iter()
                .map(|root| root.key)
                .collect::<Vec<_>>(),
            vec![key(12, 2)],
            "一次发布，接着现行那一版（txg 11）、实例照旧"
        );
        let rollback_root = &rollback.expected_roots[0];
        assert_eq!(rollback_root.rollback_floor, ModelCheckpointTxg(6));
        assert_eq!(
            rollback_root
                .contents
                .live_file()
                .map(|file| file.written_by),
            Some(key(6, 2))
        );
        assert_eq!(
            rollback_root.data_unit_written_at.as_slice(),
            [ModelCheckpointTxg(6)],
            "复活的数据单元分配代写回目标那一版账里的"
        );
        assert_eq!(
            rollback_root.role_written_at.get(&ModelUnitRole::TreeTable),
            Some(&ModelCheckpointTxg(12)),
            "固定点单元这次重写"
        );
        succeed(&mut model, &rollback);
        assert!(
            model
                .answer_rollback_while_mounted(key(9, 2))
                .expect("会话开着、现行版本带文件")
                .required_refusals
                .is_empty(),
            "向前发布不抛弃 (9, 2)"
        );
        model.close_session();
        let cold_start = model.answer_cold_start_recover();
        let Some(ModelReadBack::FileRead { root, content }) = cold_start.expected_read_back else {
            panic!("回退那一版带文件：{cold_start:?}");
        };
        assert_eq!(root, key(12, 2));
        assert_eq!(content.as_ref(), &[6]);
    }

    fn refusal_of_the_one_reread(wrote_anything: bool) -> ObservedOutcome {
        ObservedOutcome::Refused {
            member: "MountError::NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion)".to_string(),
            reason: ObservedRefusalReason::Explained(
                ModelRefusalReason::NewerPublishWitnessedBySystemConfigurationStillUnreadableAfterOneReread,
            ),
            publishes_completed: 0,
            wrote_anything,
            reported_ceiling: None,
            reported_root_ring_slot_still_bad_after_one_reread: None,
        }
    }

    /// 崩溃恢复抛弃根那一步：最新那条根 (2, 9) 是这个会话发的、系统配置见证过，整次挂载读不出
    /// ⇒ 模型答拒可写，理由就是「重读一次仍读不出」这一条，一次发布都不做。实现照这样拒（取号之前、录制流一步不多）对得上、
    /// 模型状态逐项不变；实现做成了、拒之前写了盘、拒的理由不是这一条，各对不上在自己那一格。
    #[test]
    fn crash_recovery_that_cannot_read_the_witnessed_newest_root_is_refused_before_any_write_and_leaves_the_model_unchanged(
    ) {
        let mut model = model_after_four_overwrites_in_a_second_instance();
        model.close_session();
        assert_eq!(model.newest_root().key, key(9, 2));
        let answer = model
            .answer_mount_writable_with_the_newest_root_unreadable()
            .expect("环里 (2, 9) 之前还有根");
        assert_eq!(
            answer.required_refusals,
            BTreeSet::from([
                ModelRefusalReason::NewerPublishWitnessedBySystemConfigurationStillUnreadableAfterOneReread
            ]),
            "系统配置见证过的最新那条根重读一次仍读不出：拒可写"
        );
        assert_eq!(
            (answer.expected_roots.len(), &answer.expected_mount),
            (0, &None),
            "拒了就一条根都不写、不取号"
        );
        let model_before = model.clone();
        let counts = model
            .judge_and_advance(&answer, &refusal_of_the_one_reread(false))
            .expect("照答案拒：取号之前、一个字节都没写");
        assert_eq!(counts.required_refusals_matched, 1);
        assert_eq!(model, model_before, "拒了之后模型状态逐项不变");
        let wrote_before_refusing = model
            .clone()
            .judge_and_advance(&answer, &refusal_of_the_one_reread(true))
            .expect_err("拒之前录制流里多了写");
        assert_eq!(
            wrote_before_refusing.aspect,
            ModelDisagreementAspect::WroteBeforeRefusing
        );
        let other_reason = model
            .clone()
            .judge_and_advance(
                &answer,
                &ObservedOutcome::Refused {
                    member: "MountError::NewerStateStillUnreadableAfterOneReread(InstanceTableOfTheNewestRootForTheShadowLedger)".to_string(),
                    reason: ObservedRefusalReason::Unexplained,
                    publishes_completed: 0,
                    wrote_anything: false,
                    reported_ceiling: None,
                    reported_root_ring_slot_still_bad_after_one_reread: None,
                },
            )
            .expect_err("拒的不是「重读一次仍读不出」那一条");
        assert_eq!(other_reason.aspect, ModelDisagreementAspect::RefusalReason);
        let succeeded = model
            .clone()
            .judge_and_advance(
                &answer,
                &ObservedOutcome::Succeeded(ObservedEffect::Mount {
                    instance: ModelInstanceGeneration(3),
                    rows_written: Vec::new(),
                    roots: Vec::new(),
                }),
            )
            .expect_err("实现照旧抛弃最新那条根、挂载做成");
        assert_eq!(
            succeeded.aspect,
            ModelDisagreementAspect::SucceededWhenModelRequiresRefusal
        );
    }

    /// 每块盘最新那一槽系统配置读回全 0 的可写挂载：没有一块盘两槽都自证过，乙入判不出（C971）⇒ 模型答拒可写，理由是「重读一次仍读不出」
    /// 那一条，一次发布都不做。实现照这样拒（取号之前、录制流一步不多）对得上、模型状态逐项不变；实现做成了在「模型要拒、实现做成」那一格对不上。
    #[test]
    fn writable_mount_with_no_device_holding_two_self_verified_system_configuration_slots_is_refused_before_any_write(
    ) {
        let mut model = model_after_four_overwrites_in_a_second_instance();
        model.close_session();
        let answer = model
            .answer_mount_writable_with_no_device_holding_two_self_verified_system_configuration_slots();
        assert_eq!(
            answer.required_refusals,
            BTreeSet::from([
                ModelRefusalReason::NewerPublishWitnessedBySystemConfigurationStillUnreadableAfterOneReread
            ]),
            "每块盘最新那一槽读回全 0、重读仍全 0：乙入判不出拒可写"
        );
        assert_eq!(
            (
                answer.expected_roots.len(),
                &answer.expected_mount,
                answer.walls_are_judged_before_the_first_write
            ),
            (0, &None, true),
            "拒了就一条根都不写、不取号，拒在第一个写之前"
        );
        let model_before = model.clone();
        let counts = model
            .judge_and_advance(&answer, &refusal_of_the_one_reread(false))
            .expect("照答案拒：取号之前、一个字节都没写");
        assert_eq!(counts.required_refusals_matched, 1);
        assert_eq!(model, model_before, "拒了之后模型状态逐项不变");
        let succeeded = model
            .clone()
            .judge_and_advance(
                &answer,
                &ObservedOutcome::Succeeded(ObservedEffect::Mount {
                    instance: ModelInstanceGeneration(3),
                    rows_written: Vec::new(),
                    roots: Vec::new(),
                }),
            )
            .expect_err("实现不认读回全 0 的槽、挂载做成");
        assert_eq!(
            succeeded.aspect,
            ModelDisagreementAspect::SucceededWhenModelRequiresRefusal
        );
    }

    /// 一版里最左那片之外的 inode 叶容器角色与各自的分配代。
    fn inode_leaf_containers_right_of_the_leftmost(
        root: &ModelRoot,
    ) -> BTreeMap<ModelUnitRole, ModelCheckpointTxg> {
        root.role_written_at
            .iter()
            .filter(|(role, _)| matches!(role, ModelUnitRole::InodeLeafRightOfTheLeftmost(_)))
            .map(|(role, written_at)| (*role, *written_at))
            .collect()
    }

    /// 只供测试的一版：照 `base` 抄，inode 树换成 `leaf_containers` 片叶容器（从 1 数，最左那片算 1），最左那片之外的第 k 片记成
    /// txg `first_written_at + k` 写的（各片分配代互不相同，回退取错一片就看得出来）。
    fn version_with_inode_leaf_containers(
        base: &ModelRoot,
        leaf_containers: u64,
        first_written_at: ModelCheckpointTxg,
    ) -> ModelRoot {
        let mut version = base.clone();
        version
            .role_written_at
            .retain(|role, _| !matches!(role, ModelUnitRole::InodeLeafRightOfTheLeftmost(_)));
        for position_from_the_leftmost in 1..leaf_containers {
            version.role_written_at.insert(
                ModelUnitRole::inode_leaf_container_at(position_from_the_leftmost),
                ModelCheckpointTxg(first_written_at.0 + position_from_the_leftmost),
            );
        }
        version
    }

    /// 模型回退那次发布对最左那片之外的叶容器的处置（`next_root_with_the_user_visible_trees` 取回退目标那一支里丢现行那几片、取目标那几片的两句），
    /// 结构单测（`research/prompts/m3-formal-code-b30-r1-main-verification.md` 第二节 M1「第 k 片都对」那一行的改法 B4）：
    /// 现行与目标各取 1、2、3、4、5、30、100、120、135 片（135 是一个码 2 根装得下的上界，D8（核心索引结构） 已定项 6）的每一种组合，回退写出的那一版
    /// 最左那片之外的叶容器（角色与分配代）恰好等于目标那一版的——现行有、目标没有的那几片这一版也没有，目标的每一片都取回、分配代取目标的。
    /// 30、100、120 罩只按片数触发、不按位置触发的错（最左之外各 29、99、119 片）：1–5 与 135 之间没有取样点时，只在目标装 60–118 片才错的回退照绿
    /// （第 32–33 批代码轮判决 `research/prompts/m3-formal-code-b33-r1-main-verification.md` 第二节 P1「B4」那一行，`.claude/rules/mutation-sampling.md` 第三类）；
    /// 只有 100 时，最左之外的片数落在 6–58、5–98、100–133 里才错的回退照绿（
    /// `research/prompts/m3-formal-code-b34-r1-main-verification.md` 第二节 P5「B4」那一行），30 落进前两个窗口、120 落进第三个。
    /// 按取样点逐片往外加历史用例追不完（回退只取回或只丢某几片，只有建到那几片之外的历史抓得到），这一条不跑历史。
    #[test]
    fn rollback_takes_exactly_the_inode_leaf_containers_right_of_the_leftmost_of_its_target_for_every_pair_of_container_counts(
    ) {
        const CONTAINER_COUNTS: [u64; 9] = [1, 2, 3, 4, 5, 30, 100, 120, 135];
        let model = model_after_four_overwrites_in_a_second_instance();
        let newest = model.newest_root().clone();
        let mut pairs_judged = 0;
        for current_leaf_containers in CONTAINER_COUNTS {
            for target_leaf_containers in CONTAINER_COUNTS {
                let target = version_with_inode_leaf_containers(
                    &newest,
                    target_leaf_containers,
                    ModelCheckpointTxg(1000),
                );
                let current = version_with_inode_leaf_containers(
                    &newest,
                    current_leaf_containers,
                    ModelCheckpointTxg(2000),
                );
                let after_the_rollback = model.next_root_with_the_user_visible_trees(
                    &current,
                    current.key.instance,
                    ModelCheckpointTxg(current.key.checkpoint_txg.0 + 1),
                    ModelJournalCounter(model.highest_journal_counter.0 + 1),
                    ModelPublishKind::EmptyOnFileVersion,
                    current.rollback_floor,
                    None,
                    Rc::clone(&current.instance_table_rows),
                    UserVisibleTreesOfTheNewVersion::TakenFromTheRollbackTarget(&target),
                    InodesOtherThanTheFirstChangeOfThePublish::Carried,
                );
                assert_eq!(
                    inode_leaf_containers_right_of_the_leftmost(&after_the_rollback),
                    inode_leaf_containers_right_of_the_leftmost(&target),
                    "现行 {current_leaf_containers} 片、目标 {target_leaf_containers} 片：回退写出的那一版最左那片之外的叶容器要恰好是目标那一版的"
                );
                assert_eq!(
                    inode_leaf_containers_right_of_the_leftmost(&target).len(),
                    usize::try_from(target_leaf_containers - 1).expect("片数小于 135"),
                    "搭建：目标那一版最左那片之外有 {} 片",
                    target_leaf_containers - 1
                );
                pairs_judged += 1;
            }
        }
        assert_eq!(
            pairs_judged,
            CONTAINER_COUNTS.len() * CONTAINER_COUNTS.len()
        );
    }

    /// 回退那一版搬过的根只取目标那一版的（判决 `research/prompts/m4-segment-code-r2-main-verification.md` 第三节 H1）：签发（txg 10）之后
    /// 三批各搬一次活头 inode 树根（txg 11、12、13，每次一个新状态），回退到签发那一版 (10, 2) 写 (14, 2)。实现回退时树表根指针取目标那一版的字节，
    /// 新根与 (10, 2) 同一个状态、与它前一条 (13, 2) 不同：(14, 2) 不带「最后一次搬它」的记录，算非空；从新到旧数 14、13、12、11（9 与 14 同一个状态不计），
    /// 第 4 新的是 11，两块盘上最新的取小是 13 ⇒ 上限 11。把现行那一版的记录并进来时 14 与 13 同一个状态、不算非空，数成 13、12、11、9，上限 9。
    #[test]
    fn rollback_after_batches_relocated_a_root_the_target_references_carries_only_the_relocations_of_the_target(
    ) {
        let mut model = model_after_four_overwrites_in_a_second_instance();
        succeed_with_the_compaction_publish(
            &mut model,
            &compaction_publish_seen(0, true, false, Vec::new()),
        );
        let target = model
            .current_version_in_the_session()
            .expect("会话开着")
            .clone();
        assert_eq!(target.key, key(10, 2), "签发那一版是回退目标");
        let inode_root_of_the_live_head = ModelUserVisibleRootUnit {
            tree: ModelUserVisibleTree::InodeTree,
            birth_txg: ModelCheckpointTxg(9),
        };
        for relocation_txg in 11_u64..=13 {
            let relocated = succeed_with_the_compaction_publish(
                &mut model,
                &compaction_publish_seen(1, true, false, vec![inode_root_of_the_live_head]),
            );
            assert_eq!(
                relocated.key,
                key(relocation_txg, 2),
                "照抄意图的一批各一次 txg"
            );
        }
        let before_the_rollback = model
            .current_version_in_the_session()
            .expect("会话开着")
            .clone();
        assert_eq!(
            before_the_rollback
                .user_visible_root_relocations
                .get(&inode_root_of_the_live_head),
            Some(&ModelCheckpointTxg(13)),
            "现行那一版最后一次搬 inode 树根是 txg 13"
        );
        let rollback = model
            .answer_rollback_while_mounted(key(10, 2))
            .expect("会话开着、现行版本带文件");
        assert!(rollback.required_refusals.is_empty(), "{rollback:?}");
        let rollback_root = rollback.expected_roots[0].clone();
        assert_eq!(rollback_root.key, key(14, 2));
        assert_eq!(
            rollback_root.user_visible_root_relocations, target.user_visible_root_relocations,
            "搬过的根只取目标那一版的（目标那一版之后才搬的不并进来）"
        );
        assert_eq!(
            rollback_root.user_visible_state(),
            target.user_visible_state(),
            "回退那一版与目标那一版同一个状态"
        );
        assert_ne!(
            rollback_root.user_visible_state(),
            before_the_rollback.user_visible_state(),
            "回退那一版与它前一条（搬了根的那一批）不同：非空"
        );
        succeed(&mut model, &rollback);
        assert_eq!(
            model.rollback_floor_ceiling(),
            Some(ModelCheckpointTxg(11)),
            "非空从新到旧 14、13、12、11：第 4 新的是 11"
        );
    }

    /// 回退牵涉的两版带快照就拒：现行那一版带着快照时要求拒成 `RollbackWithSnapshotsWhichTheFirstVersionRefuses`、
    /// 一条根都不写；删掉快照之后回退到建快照那一版（txg 10，树表里有那一对）同样要求拒；回退到建快照之前的 (6, 2) 不再因为快照拒。
    #[test]
    fn the_model_requires_a_rollback_from_or_to_a_version_carrying_a_snapshot_to_be_refused() {
        let mut model = model_after_four_overwrites_in_a_second_instance();
        let creation = model.answer_create_snapshot().expect("建快照");
        succeed(&mut model, &creation);
        let from_the_snapshot = model
            .answer_rollback_while_mounted(key(6, 2))
            .expect("会话开着、现行版本带文件");
        assert_eq!(
            from_the_snapshot.required_refusals,
            BTreeSet::from([ModelRefusalReason::RollbackWithSnapshotsWhichTheFirstVersionRefuses]),
            "现行那一版带着快照：要求拒"
        );
        assert!(from_the_snapshot.expected_roots.is_empty(), "一条根都不写");
        let intent = model.answer_destroy_snapshot(0).expect("意图");
        succeed(&mut model, &intent);
        let final_batch = model.answer_next_step_of_the_walk().expect("最后一批");
        succeed(&mut model, &final_batch);
        let to_the_snapshot = model
            .answer_rollback_while_mounted(key(10, 2))
            .expect("会话开着、现行版本带文件");
        assert_eq!(
            to_the_snapshot.required_refusals,
            BTreeSet::from([ModelRefusalReason::RollbackWithSnapshotsWhichTheFirstVersionRefuses]),
            "目标那一版带着快照：要求拒"
        );
        let before_the_snapshot = model
            .answer_rollback_while_mounted(key(6, 2))
            .expect("会话开着、现行版本带文件");
        assert!(
            !before_the_snapshot
                .required_refusals
                .contains(&ModelRefusalReason::RollbackWithSnapshotsWhichTheFirstVersionRefuses),
            "两版都没有快照：不因为快照拒：{:?}",
            before_the_snapshot.required_refusals
        );
    }
}

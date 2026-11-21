//! 核对待判行那一族的取活与写回（`library_pending_pool` 模块文档「核对待判行那一族」）：扫表挑块、认领、重读、续认领、撤掉与放给别的服务、一帧带守卫写回核对行并删核对待判行。

use super::*;

/// 核对待判行那一族（文件头「核对待判行那一族」）。
impl<Side: VerificationPackageSide> LibraryPendingPool<Side> {
    /// 核对行那一帧没写成、结论位留在服务里的块。
    #[must_use]
    pub fn refused_facts_verifier_writes(&self) -> &[RefusedFactsVerifierWrite] {
        &self.refused_facts_verifier_writes
    }

    /// 认领着、在核的核对块数。
    #[must_use]
    pub fn facts_claimed_count(&self) -> usize {
        self.facts_claimed.len()
    }

    /// 已核过、核对待判行带守卫删掉了的块数。
    #[must_use]
    pub fn verified_facts_rows_deleted(&self) -> u64 {
        self.verified_facts_rows_deleted
    }

    /// 扫核对待判行挑能取的块（不认领）：在核的、读不回材料的不挑；
    /// 在库上现算已核过的不挑、带守卫删掉它的核对待判行。按键排。
    /// 弄坏开关 facts-verifier-rows-not-scanned：一行都不扫。
    ///
    /// # Errors
    /// 读写判定库失败。
    pub fn facts_verifier_candidates(
        &mut self,
    ) -> Result<Vec<FactsVerifierPendingRowOfTheLibrary>, LibraryFailure> {
        if self
            .break_switches
            .is_on(BreakSwitch::FactsVerifierRowsNotScanned)
            || self.facts_claimed.len() >= CLAIMED_BLOCKS_WATERMARK
        {
            return Ok(Vec::new());
        }
        let wanted =
            (CLAIMED_BLOCKS_WATERMARK - self.facts_claimed.len()).min(CANDIDATES_PER_ROUND_LIMIT);
        let mut scanned = match self.side.facts_verifier_pending_rows() {
            Ok(scanned) => scanned,
            Err(error) => {
                let reason = library_failure_or_the_reason(error)?;
                let line = format!(
                    "{PENDING_ROW_SKIPPED_LINE_PREFIX} 扫不了核对待判行（库里有一行不合验证包的布局）：{reason}"
                );
                self.name_once(FACTS_VERIFIER_SCAN_PROBLEM_NAMED_KEY.to_vec(), line);
                return Ok(Vec::new());
            }
        };
        let is_capped_round = scanned.len() > wanted;
        let held_by_others_now: BTreeSet<Vec<u8>> = if is_capped_round {
            scanned.sort_by_key(|pending| pending.row.pool_entry_number);
            let my_holder = self.holder();
            self.claims
                .list_claims(b"gpu-claim/facts-verifier/")
                .unwrap_or_default()
                .into_iter()
                .filter(|claim| claim.holder_identity != my_holder)
                .map(|claim| claim.key)
                .collect()
        } else {
            BTreeSet::new()
        };
        let mut candidates = Vec::new();
        let mut verified_deleted_this_round = 0usize;
        for pending in scanned {
            if self.facts_claimed.contains_key(&pending.name)
                || self.facts_materials_not_read.contains(&pending.name)
                || (is_capped_round && held_by_others_now.contains(&pending.name.claim_key()))
            {
                continue;
            }
            if !self
                .break_switches
                .is_on(BreakSwitch::FactsVerifierVerifiedBlockVerifiedAgain)
            {
                match self.side.facts_verifier_block_is_verified(&pending.row) {
                    Ok(true) => {
                        self.delete_the_pending_row_of_a_verified_facts_block(&pending)?;
                        verified_deleted_this_round += 1;
                        if is_capped_round
                            && verified_deleted_this_round >= REUSABLE_DELETIONS_PER_ROUND_LIMIT
                        {
                            break;
                        }
                        continue;
                    }
                    Ok(false) => {}
                    Err(error) => {
                        let reason = library_failure_or_the_reason(error)?;
                        let line = format!(
                            "{PENDING_ROW_SKIPPED_LINE_PREFIX} {} 核对待判行核没核过现算不了：{reason}",
                            pending.name
                        );
                        self.name_once(pending.name.pending_key(&self.key_space), line);
                        continue;
                    }
                }
            }
            candidates.push(pending);
            if is_capped_round && candidates.len() >= wanted {
                break;
            }
        }
        Ok(candidates)
    }

    /// 已核过的块：一帧写结论只删它的核对待判行，删键的期待是这里读到的那一份（判段在这之后补写的不删）；别的持有者持着的不删。
    /// 这一帧没有核对行：守卫取这一族的认领键（认领键只记认领、从不当数据行写，样子恒为「不在」）。
    fn delete_the_pending_row_of_a_verified_facts_block(
        &mut self,
        pending: &FactsVerifierPendingRowOfTheLibrary,
    ) -> Result<(), LibraryFailure> {
        let claim_key = pending.name.claim_key();
        let request = GuardedConclusionWrite {
            holder_identity: self.holder(),
            claim_key: claim_key.clone(),
            guard_key: claim_key,
            guard_expectation: GuardRowState::Absent,
            entries: Vec::new(),
            deleted_keys: vec![GuardedDeletion {
                key: pending.name.pending_key(&self.key_space),
                expectation: GuardRowState::of_stored_value(Some(&pending.row.to_value_bytes())),
            }],
        };
        match self.claims.write_guarded_conclusion(&request)? {
            GuardedConclusionWriteOutcome::Written(_) => {
                self.verified_facts_rows_deleted += 1;
                self.print_a_service_line(format!(
                    "{VERIFIED_FACTS_BLOCK_ROW_DELETED_LINE_PREFIX} {}",
                    pending.name
                ));
            }
            GuardedConclusionWriteOutcome::Refused { refusal, message } => {
                let line = format!(
                    "{PENDING_ROW_SKIPPED_LINE_PREFIX} {} 已核过、核对待判行这一回没删（{refusal:?}：{message}）：下一轮照它此刻的样子再定",
                    pending.name
                );
                self.name_once(pending.name.claim_key(), line);
            }
        }
        Ok(())
    }

    /// 认领挑出来的核对块：认领到的重读核对待判行（不在了或指的是另一块的不取、放掉刚认领到的），别的持有者持着的不取；
    /// 取到的按（录下的流程、事实表）分组，每组照组里键最小的那一块读回一份材料交回；读不回的那一组点名一次、这个进程里不再取，认领留到租期过了。
    ///
    /// # Errors
    /// 认领、重读、放认领时读写判定库失败。
    pub fn claim_the_facts_verifier_candidates(
        &mut self,
        candidates: Vec<FactsVerifierPendingRowOfTheLibrary>,
    ) -> Result<Vec<FactsVerifierJobOfTheLibrary>, LibraryFailure> {
        let mut taken = Vec::new();
        let mut held_by_others_this_round = BTreeSet::new();
        for pending in candidates {
            if self.facts_claimed.contains_key(&pending.name) {
                continue;
            }
            match claim_a_facts_verifier_pending_block(
                &mut self.claims,
                &pending.name,
                self.claim_lease_milliseconds,
            )? {
                ClaimAnswer::Claimed => {}
                ClaimAnswer::HeldByOther {
                    holder,
                    remaining_lease_milliseconds,
                } if !self
                    .break_switches
                    .is_on(BreakSwitch::ClaimHeldByOthersIgnored) =>
                {
                    let observation = (pending.name, holder.clone());
                    if !self.facts_held_by_others_last_round.contains(&observation) {
                        self.print_a_service_line(format!(
                            "{FACTS_VERIFIER_CLAIM_HELD_BY_OTHER_LINE_PREFIX} {} holder={} remaining_lease_milliseconds={remaining_lease_milliseconds}",
                            pending.name,
                            hexadecimal_text(&holder)
                        ));
                    }
                    held_by_others_this_round.insert(observation);
                    continue;
                }
                ClaimAnswer::HeldByOther { .. } => {}
                ClaimAnswer::ClaimKeyWritten => {
                    let line = format!(
                        "{PENDING_ROW_SKIPPED_LINE_PREFIX} {} 库里有核对认领那把键（认领键不当数据行写）：不取",
                        pending.name
                    );
                    self.name_once(pending.name.claim_key(), line);
                    continue;
                }
            }
            if self
                .break_switches
                .is_on(BreakSwitch::FactsVerifierRowNotRereadAfterTheClaim)
            {
                taken.push(pending);
                continue;
            }
            if let Some(reread) = self.reread_the_facts_verifier_row_after_the_claim(pending)? {
                taken.push(reread);
            }
        }
        self.facts_held_by_others_last_round = held_by_others_this_round;
        let mut groups: BTreeMap<MaterialGroupOfARow, Vec<FactsVerifierPendingRowOfTheLibrary>> =
            BTreeMap::new();
        for pending in taken {
            groups
                .entry(material_group_of(&pending.row))
                .or_default()
                .push(pending);
        }
        let mut jobs = Vec::new();
        for blocks in groups.into_values() {
            let mut valid_blocks = Vec::with_capacity(blocks.len());
            let mut group_material = None;
            for pending in blocks {
                match self.side.facts_verifier_material(&pending.row) {
                    Ok(material) => {
                        if group_material.is_none() {
                            group_material = Some(material);
                        }
                        valid_blocks.push(pending);
                    }
                    Err(reason) => {
                        let holder = self.holder();
                        let _ = self.release_the_claim_keys(&holder, &[pending.name.claim_key()]);
                        let line = format!(
                            "{PENDING_ROW_SKIPPED_LINE_PREFIX} {} 读不回核对材料（这个进程里不再取它，认领已放）：{reason}",
                            pending.name
                        );
                        self.name_once(pending.name.pending_key(&self.key_space), line);
                        self.facts_materials_not_read.insert(pending.name);
                    }
                }
            }
            let Some(material) = group_material else {
                continue;
            };
            for pending in &valid_blocks {
                self.facts_claimed.insert(
                    pending.name,
                    ClaimedFactsVerifierBlock {
                        row: pending.row.clone(),
                        renewed_at: Instant::now(),
                        claim_lost_to: None,
                    },
                );
                self.print_a_service_line(format!(
                    "{FACTS_VERIFIER_BLOCK_CLAIMED_LINE_PREFIX} {} states={}",
                    pending.name,
                    pending.row.state_ordinals.end - pending.row.state_ordinals.start
                ));
            }
            jobs.push(FactsVerifierJobOfTheLibrary {
                blocks: valid_blocks,
                material,
            });
        }
        Ok(jobs)
    }

    /// 认领到之后重读这一块的核对待判行：还是同一块的交回重读到的那一行；不在了、指的是另一块、重读不了（行不合布局，点名一次）的，
    /// 以自己的持有者放掉刚认领到的认领、交 `None`。
    ///
    /// # Errors
    /// 重读、放认领时读写判定库失败。
    fn reread_the_facts_verifier_row_after_the_claim(
        &mut self,
        pending: FactsVerifierPendingRowOfTheLibrary,
    ) -> Result<Option<FactsVerifierPendingRowOfTheLibrary>, LibraryFailure> {
        let why = match self.side.facts_verifier_pending_row(&pending.name) {
            Ok(Some(reread)) if is_the_same_facts_verifier_block(&reread, &pending.row) => {
                return Ok(Some(FactsVerifierPendingRowOfTheLibrary {
                    name: pending.name,
                    row: reread,
                }));
            }
            Ok(Some(_)) => "pending-row-is-another-block",
            Ok(None) => "pending-row-gone",
            Err(error) => {
                let reason = library_failure_or_the_reason(error)?;
                let line = format!(
                    "{PENDING_ROW_SKIPPED_LINE_PREFIX} {} 认领到之后重读不了核对待判行：{reason}",
                    pending.name
                );
                self.name_once(pending.name.pending_key(&self.key_space), line);
                "pending-row-unreadable"
            }
        };
        let holder = self.holder();
        let released = self.release_the_claim_keys(&holder, &[pending.name.claim_key()])?;
        self.print_a_service_line(format!(
            "{FACTS_VERIFIER_CLAIM_RELEASED_AFTER_THE_REREAD_LINE_PREFIX} {} why={why} released={}",
            pending.name, released.released
        ));
        Ok(None)
    }

    /// 在核的块里上一次续认领过了三分之一租期的，续一次；续不上（别的持有者持着）的交回名字，块照核，写回时那一帧核认领、不写。
    ///
    /// # Errors
    /// 读写判定库失败。
    pub fn renew_the_facts_verifier_claims(
        &mut self,
    ) -> Result<Vec<FactsVerifierBlockName>, LibraryFailure> {
        let renew_after = Duration::from_millis(u64::from(self.claim_lease_milliseconds) / 3);
        let due: Vec<FactsVerifierBlockName> = self
            .facts_claimed
            .iter()
            .filter(|(_, claimed)| claimed.renewed_at.elapsed() >= renew_after)
            .map(|(name, _)| *name)
            .collect();
        let mut lost = Vec::new();
        for name in due {
            let answer = claim_a_facts_verifier_pending_block(
                &mut self.claims,
                &name,
                self.claim_lease_milliseconds,
            )?;
            if answer == ClaimAnswer::Claimed {
                if let Some(claimed) = self.facts_claimed.get_mut(&name) {
                    claimed.renewed_at = Instant::now();
                }
            } else {
                if let ClaimAnswer::HeldByOther { holder, .. } = &answer {
                    if let Some(claimed) = self.facts_claimed.get_mut(&name) {
                        claimed.claim_lost_to = Some(holder.clone());
                    }
                }
                lost.push(name);
            }
        }
        Ok(lost)
    }

    /// 主循环把这一块撤了（核它的卡坏了）：不再当在核，认领留到租期过了，下一轮扫到再取。
    pub fn withdraw_the_facts_verifier_block(&mut self, name: &FactsVerifierBlockName) {
        self.facts_claimed.remove(name);
    }

    /// 这一台核不了这一块（文件头「放认领」）：不再当在核，当场以自己的持有者放掉它的认领，别的服务随即取得到。
    ///
    /// # Errors
    /// 放认领时读写判定库失败（认领留到租期过了）。
    pub fn release_the_facts_verifier_claim_left_to_another_service(
        &mut self,
        name: &FactsVerifierBlockName,
    ) -> Result<ClaimsReleased, LibraryFailure> {
        self.facts_claimed.remove(name);
        let holder = self.holder();
        self.release_the_claim_keys(&holder, &[name.claim_key()])
    }

    fn refuse_the_facts_verifier_write(
        &mut self,
        name: FactsVerifierBlockName,
        bits_of_each_state: Vec<u32>,
        reason: String,
    ) -> RefusedFactsVerifierWrite {
        let holder = self.holder();
        let _ = self.release_the_claim_keys(&holder, &[name.claim_key()]);
        let refused = RefusedFactsVerifierWrite {
            name,
            bits_of_each_state,
            reason,
        };
        self.print_a_service_line(format!(
            "{FACTS_VERIFIER_WRITE_REFUSED_LINE_PREFIX} {} states={} reason={}",
            refused.name,
            refused.bits_of_each_state.len(),
            refused.reason
        ));
        self.refused_facts_verifier_writes.push(refused.clone());
        refused
    }

    /// 验证包编的一帧放进判定库服务的写结论帧（文件头「核对待判行那一族」「写回」）：认领键与持有者是服务的，守卫键是核对行、期待是编帧之前读到的
    /// 那一份，条目是要写的核对行（库里已有逐字节相同的那一行时没有条目），删键是核对待判行、期待是 `row_as_read`（不在了是「不在」）。
    /// 弄坏开关 facts-verifier-row-guard-taken-at-the-write、facts-verifier-pending-row-guard-taken-at-the-write：那一个期待改取写之前此刻库里的样子；
    /// facts-verifier-frame-under-the-image-judgement-claim-key：认领键写成镜像判定那一族的形。
    ///
    /// # Errors
    /// 核对行的值长过一段（写结论那一帧不分段，一帧原子写不下）；弄坏开关下读此刻的样子失败。
    fn guarded_write_of_the_facts_verifier_frame(
        &mut self,
        name: &FactsVerifierBlockName,
        row_as_read: Option<&FactsVerifierPendingRow>,
        frame: &FactsVerifierRowsForOneFrame,
    ) -> Result<GuardedConclusionWrite, String> {
        if let Some((key, value)) = frame
            .verifier_row_to_put
            .as_ref()
            .filter(|(_, value)| value.len() > SEGMENT_BYTES)
        {
            return Err(format!(
                "核对行（键 {}）的值 {} 字节，长过一段（{SEGMENT_BYTES} 字节）：写结论那一帧不分段，原子写不下",
                hexadecimal_text(&key[..key.len().min(16)]),
                value.len()
            ));
        }
        let mut guard_expectation =
            GuardRowState::of_stored_value(frame.verifier_row_as_read.as_deref());
        let mut pending_expectation = GuardRowState::of_stored_value(
            row_as_read.map(|_| frame.pending_row_as_read.as_slice()),
        );
        let is_the_row_guard_taken_at_the_write = self
            .break_switches
            .is_on(BreakSwitch::FactsVerifierRowGuardTakenAtTheWrite);
        let is_the_pending_guard_taken_at_the_write = self
            .break_switches
            .is_on(BreakSwitch::FactsVerifierPendingRowGuardTakenAtTheWrite);
        if is_the_row_guard_taken_at_the_write || is_the_pending_guard_taken_at_the_write {
            let mut rows_now = self
                .claims
                .read(&[
                    (Table::Conclusions, frame.verifier_row_key.clone()),
                    (Table::Conclusions, frame.pending_row_key_to_delete.clone()),
                ])
                .map_err(|failure| format!("读两个守卫此刻的样子失败：{failure}"))?
                .into_iter();
            let verifier_row_now = rows_now.next().flatten();
            let pending_row_now = rows_now.next().flatten();
            if is_the_row_guard_taken_at_the_write {
                guard_expectation = GuardRowState::of_stored_value(verifier_row_now.as_deref());
            }
            if is_the_pending_guard_taken_at_the_write {
                pending_expectation = GuardRowState::of_stored_value(pending_row_now.as_deref());
            }
        }
        let claim_key = if self
            .break_switches
            .is_on(BreakSwitch::FactsVerifierFrameUnderTheImageJudgementClaimKey)
        {
            gpu_claim_key(&name.stream.0, name.block_number)
        } else {
            name.claim_key()
        };
        Ok(GuardedConclusionWrite {
            holder_identity: self.holder(),
            claim_key,
            guard_key: frame.verifier_row_key.clone(),
            guard_expectation,
            entries: frame
                .verifier_row_to_put
                .iter()
                .map(|(key, value)| (Table::Conclusions, key.clone(), value.clone()))
                .collect(),
            deleted_keys: vec![GuardedDeletion {
                key: frame.pending_row_key_to_delete.clone(),
                expectation: pending_expectation,
            }],
        })
    }

    /// 一块核完：编那一帧，一帧写下核对行、删核对待判行、放认领，守卫不过照种类重编、重读再写（文件头「核对待判行那一族」「写回」）。
    /// 弄坏开关 facts-verifier-guard-refusal-not-retried：守卫不过当场报红。
    ///
    /// # Errors
    /// 那一帧没写成（结论位留在 [`Self::refused_facts_verifier_writes`]、打了红行）。
    ///
    /// # Panics
    /// 这一块不在认领着、在核的核对块里（主循环交回的块与认领对不上）。
    pub fn write_back_the_facts_verifier_block(
        &mut self,
        name: &FactsVerifierBlockName,
        bits_of_each_state: Vec<u32>,
    ) -> Result<FactsVerifierWrittenBack, RefusedFactsVerifierWrite> {
        let ClaimedFactsVerifierBlock {
            row, claim_lost_to, ..
        } = self.facts_claimed.remove(name).unwrap_or_else(|| {
            panic!("{name} 不在这个服务认领着的核对块里：主循环交回的块与认领对不上")
        });
        if let Some(holder_identity) = claim_lost_to {
            let reason = format!(
                "认领被别的持有者 {} 持着，这一帧不写",
                hexadecimal_text(&holder_identity)
            );
            return Err(self.refuse_the_facts_verifier_write(*name, bits_of_each_state, reason));
        }
        let mut row_as_read = Some(row.clone());
        let mut last_refusal = String::new();
        let is_retried = !self
            .break_switches
            .is_on(BreakSwitch::FactsVerifierGuardRefusalNotRetried);
        // 迭代上界：CONCLUSION_WRITE_ATTEMPTS；每一圈要么写成、要么拒、要么重编重读之后再写一次
        for _ in 0..CONCLUSION_WRITE_ATTEMPTS {
            let row_of_the_frame = row_as_read.clone().unwrap_or_else(|| row.clone());
            let frame =
                match self
                    .side
                    .facts_verifier_frame(name, &row_of_the_frame, &bits_of_each_state)
                {
                    Ok(frame) => frame,
                    Err(reason) => {
                        self.facts_materials_not_read.insert(*name);
                        let reason = format!("编不成核对行那一帧：{reason}");
                        return Err(self.refuse_the_facts_verifier_write(
                            *name,
                            bits_of_each_state,
                            reason,
                        ));
                    }
                };
            let request = match self.guarded_write_of_the_facts_verifier_frame(
                name,
                row_as_read.as_ref(),
                &frame,
            ) {
                Ok(request) => request,
                Err(reason) => {
                    self.facts_materials_not_read.insert(*name);
                    return Err(self.refuse_the_facts_verifier_write(
                        *name,
                        bits_of_each_state,
                        reason,
                    ));
                }
            };
            let refusal = match self.claims.write_guarded_conclusion(&request) {
                Ok(GuardedConclusionWriteOutcome::Written(acknowledgement)) => {
                    let verifier_row_written = frame.verifier_row_to_put.is_some();
                    self.print_a_service_line(format!(
                        "{FACTS_VERIFIER_WRITTEN_LINE_PREFIX} {name} states={} verifier_row_written={verifier_row_written} already_applied={}",
                        bits_of_each_state.len(),
                        acknowledgement.is_already_applied
                    ));
                    return Ok(FactsVerifierWrittenBack {
                        verifier_row_written,
                    });
                }
                Ok(GuardedConclusionWriteOutcome::Refused { refusal, .. }) => refusal,
                Err(failure) => {
                    let reason = format!("核对行那一帧没写成：{failure}");
                    return Err(self.refuse_the_facts_verifier_write(
                        *name,
                        bits_of_each_state,
                        reason,
                    ));
                }
            };
            match refusal {
                GuardRefusal::ClaimHeldByOther {
                    holder_identity, ..
                } => {
                    let reason = format!(
                        "认领被别的持有者 {} 持着，这一帧不写",
                        hexadecimal_text(&holder_identity)
                    );
                    return Err(self.refuse_the_facts_verifier_write(
                        *name,
                        bits_of_each_state,
                        reason,
                    ));
                }
                GuardRefusal::GuardRowChanged { current } => {
                    last_refusal = format!("核对行编帧之后变了（此刻 {current:?}）");
                    if !is_retried {
                        return Err(self.refuse_the_facts_verifier_write(
                            *name,
                            bits_of_each_state,
                            last_refusal,
                        ));
                    }
                }
                GuardRefusal::DeletedRowChanged { key, current } => {
                    last_refusal = format!(
                        "删键 {} 那一行读完之后变了（此刻 {current:?}）",
                        hexadecimal_text(&key[..key.len().min(16)])
                    );
                    if !is_retried {
                        return Err(self.refuse_the_facts_verifier_write(
                            *name,
                            bits_of_each_state,
                            last_refusal,
                        ));
                    }
                    if key == name.pending_key(&self.key_space) {
                        match self.side.facts_verifier_pending_row(name) {
                            Ok(None) => row_as_read = None,
                            Ok(Some(reread)) if is_the_same_facts_verifier_block(&reread, &row) => {
                                row_as_read = Some(reread);
                            }
                            Ok(Some(_)) => {
                                let reason = format!(
                                    "{last_refusal}；重读到的核对待判行指的是另一块（块键、流程、事实表或区间不同）"
                                );
                                return Err(self.refuse_the_facts_verifier_write(
                                    *name,
                                    bits_of_each_state,
                                    reason,
                                ));
                            }
                            Err(error) => {
                                let reason =
                                    format!("{last_refusal}；重读核对待判行失败：{error:?}");
                                return Err(self.refuse_the_facts_verifier_write(
                                    *name,
                                    bits_of_each_state,
                                    reason,
                                ));
                            }
                        }
                    }
                }
            }
        }
        let reason = format!(
            "核对行那一帧守卫不过 {CONCLUSION_WRITE_ATTEMPTS} 次，最后一次：{last_refusal}"
        );
        Err(self.refuse_the_facts_verifier_write(*name, bits_of_each_state, reason))
    }
}

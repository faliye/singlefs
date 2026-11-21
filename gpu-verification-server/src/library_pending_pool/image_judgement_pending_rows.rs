//! 镜像判定那一族待判行的取活与写回（`library_pending_pool` 模块文档「判完」「认领」「写回」）：扫库挑块、认领、认领之后重读、续认领、撤掉与放给别的服务、一帧带守卫写回结论并删待判行。

use super::*;

impl<Side: VerificationPackageSide> LibraryPendingPool<Side> {
    /// 扫库挑能取的块（不认领）：已在判的不挑；不是强制现判的在库上现算判到了哪一步，判完而可复用的不挑、带守卫删掉它的待判行。
    /// 按键排。
    ///
    /// # Errors
    /// 读写判定库失败。
    pub fn candidates(&mut self) -> Result<Vec<PendingRowOfTheLibrary>, LibraryFailure> {
        if self.claimed.len() >= CLAIMED_BLOCKS_WATERMARK {
            return Ok(Vec::new());
        }
        let wanted =
            (CLAIMED_BLOCKS_WATERMARK - self.claimed.len()).min(CANDIDATES_PER_ROUND_LIMIT);
        let mut scanned = match self.side.pending_rows() {
            Ok(scanned) => scanned,
            Err(error) => {
                let reason = library_failure_or_the_reason(error)?;
                let line = format!(
                    "{PENDING_ROW_SKIPPED_LINE_PREFIX} 扫不了待判行（库里有一行不合验证包的布局）：{reason}"
                );
                self.name_once(SCAN_PROBLEM_NAMED_KEY.to_vec(), line);
                return Ok(Vec::new());
            }
        };
        let is_capped_round = scanned.len() > wanted;
        let held_by_others_now: BTreeSet<Vec<u8>> = if is_capped_round {
            scanned.sort_by_key(|pending| pending.row.pool_entry_number);
            let my_holder = self.holder();
            self.claims
                .list_claims(b"gpu-claim/")
                .unwrap_or_default()
                .into_iter()
                .filter(|claim| claim.holder_identity != my_holder)
                .map(|claim| claim.key)
                .collect()
        } else {
            BTreeSet::new()
        };
        let mut candidates = Vec::new();
        let mut reusable_deleted_this_round = 0usize;
        for pending in scanned {
            if self.claimed.contains_key(&pending.name)
                || self.streams_not_configured.contains(&pending.name)
                || (is_capped_round && held_by_others_now.contains(&pending.name.claim_key()))
            {
                continue;
            }
            let reads_the_library = pending.row.forced == ForcedRejudging::NotForced
                || self
                    .break_switches
                    .is_on(BreakSwitch::ForcedBlockReadsTheLibrary);
            if reads_the_library {
                match self.side.judging_in_the_library(&pending.row) {
                    Ok(GpuPendingBlockInTheLibrary::JudgedAndReusable) => {
                        if !self
                            .break_switches
                            .is_on(BreakSwitch::ReusableBlockPendingRowKept)
                        {
                            self.delete_the_pending_row_of_a_reusable_block(&pending)?;
                            reusable_deleted_this_round += 1;
                            if is_capped_round
                                && reusable_deleted_this_round >= REUSABLE_DELETIONS_PER_ROUND_LIMIT
                            {
                                break;
                            }
                        }
                        continue;
                    }
                    Ok(
                        GpuPendingBlockInTheLibrary::NotJudged
                        | GpuPendingBlockInTheLibrary::JudgedRed,
                    ) => {}
                    Err(error) => {
                        let reason = library_failure_or_the_reason(error)?;
                        let line = format!(
                            "{PENDING_ROW_SKIPPED_LINE_PREFIX} {} 判到了哪一步现算不了：{reason}",
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

    /// 判完而可复用的块：一帧写结论只删它的待判行，删键的期待是这里读到的那一份（判段读完之后升成强制现判的不删，下一轮照强制现判取）；
    /// 别的持有者持着的不删（归持有者判）。这一帧没有结论行：守卫取认领键（认领键只记认领、从不当数据行写，样子恒为「不在」），
    /// 核的是删键与认领。
    fn delete_the_pending_row_of_a_reusable_block(
        &mut self,
        pending: &PendingRowOfTheLibrary,
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
                self.reusable_rows_deleted += 1;
                eprintln!("{REUSABLE_BLOCK_ROW_DELETED_LINE_PREFIX} {}", pending.name);
            }
            GuardedConclusionWriteOutcome::Refused { refusal, message } => {
                let line = format!(
                    "{PENDING_ROW_SKIPPED_LINE_PREFIX} {} 判完而可复用、待判行这一回没删（{refusal:?}：{message}）：下一轮照它此刻的样子再定",
                    pending.name
                );
                self.name_once(pending.name.claim_key(), line);
            }
        }
        Ok(())
    }

    /// 认领挑出来的块：认领到的重读待判行（文件头「认领」：不在了或指的是另一块的不取、放掉刚认领到的），还是同一块的配上流交回；
    /// 别的持有者持着的不取；配不出流的不取（点名一次，认领留到租期过了，这个进程里不再挑它）。
    ///
    /// # Errors
    /// 认领、重读、放认领时读写判定库失败。
    pub fn claim_the_candidates(
        &mut self,
        candidates: Vec<PendingRowOfTheLibrary>,
    ) -> Result<ClaimRound<Side::Materials>, LibraryFailure> {
        let mut round = ClaimRound {
            taken: Vec::new(),
            held_by_others: Vec::new(),
        };
        let mut held_by_others_this_round = BTreeSet::new();
        for pending in candidates {
            if self.claimed.contains_key(&pending.name) {
                continue;
            }
            match claim_a_pending_block(
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
                    if !self.held_by_others_last_round.contains(&observation) {
                        self.print_a_service_line(format!(
                            "{CLAIM_HELD_BY_OTHER_LINE_PREFIX} {} holder={} remaining_lease_milliseconds={remaining_lease_milliseconds}",
                            pending.name,
                            hexadecimal_text(&holder)
                        ));
                    }
                    held_by_others_this_round.insert(observation);
                    round.held_by_others.push((pending.name, holder));
                    continue;
                }
                ClaimAnswer::HeldByOther { .. } => {}
                ClaimAnswer::ClaimKeyWritten => {
                    let line = format!(
                        "{PENDING_ROW_SKIPPED_LINE_PREFIX} {} 库里有认领那把键（认领键不当数据行写）：不取",
                        pending.name
                    );
                    self.name_once(pending.name.claim_key(), line);
                    continue;
                }
            }
            let pending = if self
                .break_switches
                .is_on(BreakSwitch::PendingRowNotRereadAfterTheClaim)
            {
                pending
            } else {
                match self.reread_after_the_claim(pending)? {
                    Some(reread) => reread,
                    None => continue,
                }
            };
            let stream = match self.side.stream_of_the_row(&pending.row) {
                Ok(stream) => stream,
                Err(reason) => {
                    let holder = self.holder();
                    let _ = self.release_the_claim_keys(&holder, &[pending.name.claim_key()]);
                    let line = format!(
                        "{PENDING_ROW_SKIPPED_LINE_PREFIX} {} 配不出流（这个进程里不再取它，认领已放）：{reason}",
                        pending.name
                    );
                    self.name_once(pending.name.pending_key(&self.key_space), line);
                    self.streams_not_configured.insert(pending.name);
                    continue;
                }
            };
            self.claimed.insert(
                pending.name,
                ClaimedBlock {
                    row: pending.row.clone(),
                    renewed_at: Instant::now(),
                    claim_lost_to: None,
                },
            );
            self.print_a_service_line(format!(
                "{BLOCK_CLAIMED_LINE_PREFIX} {} states={}",
                pending.name,
                pending.row.state_ordinals.end - pending.row.state_ordinals.start
            ));
            round.taken.push(TakenBlock { pending, stream });
        }
        self.held_by_others_last_round = held_by_others_this_round;
        Ok(round)
    }

    /// 认领到之后重读这一块的待判行（文件头「认领」）：还是同一块的交回重读到的那一行；不在了、指的是另一块、重读不了（行不合布局，点名一次）的，
    /// 以自己的持有者放掉刚认领到的认领、交 `None`。
    ///
    /// # Errors
    /// 重读、放认领时读写判定库失败。
    fn reread_after_the_claim(
        &mut self,
        pending: PendingRowOfTheLibrary,
    ) -> Result<Option<PendingRowOfTheLibrary>, LibraryFailure> {
        let why = match self.side.pending_row(&pending.name) {
            Ok(Some(reread)) if is_the_same_block(&reread, &pending.row) => {
                return Ok(Some(PendingRowOfTheLibrary {
                    name: pending.name,
                    row: reread,
                }));
            }
            Ok(Some(_)) => "pending-row-is-another-block",
            Ok(None) => "pending-row-gone",
            Err(error) => {
                let reason = library_failure_or_the_reason(error)?;
                let line = format!(
                    "{PENDING_ROW_SKIPPED_LINE_PREFIX} {} 认领到之后重读不了待判行：{reason}",
                    pending.name
                );
                self.name_once(pending.name.pending_key(&self.key_space), line);
                "pending-row-unreadable"
            }
        };
        let holder = self.holder();
        let released = self.release_the_claim_keys(&holder, &[pending.name.claim_key()])?;
        self.print_a_service_line(format!(
            "{CLAIM_RELEASED_AFTER_THE_REREAD_LINE_PREFIX} {} why={why} released={}",
            pending.name, released.released
        ));
        Ok(None)
    }

    /// 在判的块里上一次续认领过了三分之一租期的，续一次；续不上（别的持有者持着）的交回名字，块照判，写回时那一帧核认领、不写。
    ///
    /// # Errors
    /// 读写判定库失败。
    pub fn renew_the_claims(&mut self) -> Result<Vec<PendingBlockName>, LibraryFailure> {
        let renew_after = Duration::from_millis(u64::from(self.claim_lease_milliseconds) / 3);
        let due: Vec<PendingBlockName> = self
            .claimed
            .iter()
            .filter(|(_, claimed)| claimed.renewed_at.elapsed() >= renew_after)
            .map(|(name, _)| *name)
            .collect();
        let mut lost = Vec::new();
        for name in due {
            let answer =
                claim_a_pending_block(&mut self.claims, &name, self.claim_lease_milliseconds)?;
            if answer == ClaimAnswer::Claimed {
                if let Some(claimed) = self.claimed.get_mut(&name) {
                    claimed.renewed_at = Instant::now();
                }
            } else {
                if let ClaimAnswer::HeldByOther { holder, .. } = &answer {
                    if let Some(claimed) = self.claimed.get_mut(&name) {
                        claimed.claim_lost_to = Some(holder.clone());
                    }
                }
                lost.push(name);
            }
        }
        Ok(lost)
    }

    /// 主循环把这一块撤了（判它的卡坏了）：不再当在判，认领留到租期过了，下一轮扫到再取。
    pub fn withdraw(&mut self, name: &PendingBlockName) {
        self.claimed.remove(name);
    }

    /// 这一台判不了这一块（文件头「放认领」）：不再当在判，当场以自己的持有者放掉它的认领，别的服务随即取得到。
    ///
    /// # Errors
    /// 放认领时读写判定库失败（认领留到租期过了）。
    pub fn release_the_claim_left_to_another_service(
        &mut self,
        name: &PendingBlockName,
    ) -> Result<ClaimsReleased, LibraryFailure> {
        self.claimed.remove(name);
        let holder = self.holder();
        self.release_the_claim_keys(&holder, &[name.claim_key()])
    }

    fn refuse(
        &mut self,
        name: PendingBlockName,
        verdicts: Vec<StateVerdict>,
        reason: String,
    ) -> RefusedConclusionWrite {
        let holder = self.holder();
        let _ = self.release_the_claim_keys(&holder, &[name.claim_key()]);
        let refused = RefusedConclusionWrite {
            name,
            verdicts,
            reason,
        };
        eprintln!(
            "{CONCLUSION_WRITE_REFUSED_LINE_PREFIX} {} states={} reason={}",
            refused.name,
            refused.verdicts.len(),
            refused.reason
        );
        if !self
            .break_switches
            .is_on(BreakSwitch::RefusedConclusionsDropped)
        {
            self.refused_writes.push(refused.clone());
        }
        refused
    }

    /// 一帧写结论要带的：认领键、守卫那一行与读到的那一份、条目（强制现判的凭据插在守卫那一行之前）、删键（待判行的期待是 `row_as_read`，
    /// 不在了是「不在」；一份红整份换上时旧的那几行由守卫那一行担着，期待取写之前此刻的样子）。
    ///
    /// # Errors
    /// 有一行的值长过一段（写结论那一帧不分段，一帧原子写不下）；读旧那几行此刻的样子失败。
    fn guarded_write_of_the_frame(
        &mut self,
        name: &PendingBlockName,
        row_as_read: Option<&GpuPendingRow>,
        frame: &ConclusionFrame,
    ) -> Result<(GuardedConclusionWrite, bool), String> {
        if let Some((key, value)) = frame
            .rows_to_put
            .iter()
            .find(|(_, value)| value.len() > SEGMENT_BYTES)
        {
            return Err(format!(
                "键 {} 的值 {} 字节，长过一段（{SEGMENT_BYTES} 字节）：写结论那一帧不分段，原子写不下",
                hexadecimal_text(&key[..key.len().min(16)]),
                value.len()
            ));
        }
        let mut entries: Vec<(Table, Vec<u8>, Vec<u8>)> = frame
            .rows_to_put
            .iter()
            .map(|(key, value)| (Table::Conclusions, key.clone(), value.clone()))
            .collect();
        let forced_row = row_as_read.filter(|row| {
            row.forced == ForcedRejudging::Forced
                && !self
                    .break_switches
                    .is_on(BreakSwitch::RejudgedRowNotWritten)
        });
        if let Some(row) = forced_row {
            let credential = (
                Table::Conclusions,
                name.rejudged_key(&self.key_space, row.pool_entry_number),
                GpuRejudgedRow {
                    holder_identity: self.holder(),
                    conclusion_digest: frame.conclusion_digest,
                }
                .to_value_bytes(),
            );
            let is_the_guard_row_last = entries
                .last()
                .is_some_and(|(_, key, _)| *key == frame.guard_key);
            let position = if is_the_guard_row_last {
                entries.len() - 1
            } else {
                entries.len()
            };
            entries.insert(position, credential);
        }
        let replaced_rows: Vec<(Table, Vec<u8>)> = frame
            .keys_to_delete
            .iter()
            .map(|key| (Table::Conclusions, key.clone()))
            .collect();
        let replaced_rows_now = self
            .claims
            .read(&replaced_rows)
            .map_err(|failure| format!("读一份红整份换上时旧的那几行失败：{failure}"))?;
        let mut deleted_keys: Vec<GuardedDeletion> = frame
            .keys_to_delete
            .iter()
            .zip(replaced_rows_now)
            .map(|(key, value)| GuardedDeletion {
                key: key.clone(),
                expectation: GuardRowState::of_stored_value(value.as_deref()),
            })
            .collect();
        deleted_keys.push(GuardedDeletion {
            key: name.pending_key(&self.key_space),
            expectation: GuardRowState::of_stored_value(
                row_as_read.map(GpuPendingRow::to_value_bytes).as_deref(),
            ),
        });
        Ok((
            GuardedConclusionWrite {
                holder_identity: self.holder(),
                claim_key: name.claim_key(),
                guard_key: frame.guard_key.clone(),
                guard_expectation: GuardRowState::of_stored_value(
                    frame.guard_row_as_read.as_deref(),
                ),
                entries,
                deleted_keys,
            },
            forced_row.is_some(),
        ))
    }

    /// 一块判完：编结论帧，一帧写下结论、删待判行（强制现判的加凭据行），守卫不过照种类重编、重读再写（文件头「写回」）。
    ///
    /// # Errors
    /// 那一帧没写成（结论留在 [`Self::refused_writes`]、打了红行）。
    ///
    /// # Panics
    /// 这一块不在认领着、在判的块里（主循环交回的块与认领对不上）。
    pub fn write_back(
        &mut self,
        name: &PendingBlockName,
        verdicts: Vec<StateVerdict>,
        card: &CardThatJudgedAPendingBlock,
    ) -> Result<WrittenBack, RefusedConclusionWrite> {
        let ClaimedBlock {
            row, claim_lost_to, ..
        } = self.claimed.remove(name).unwrap_or_else(|| {
            panic!("{name} 不在这个服务认领着的块里：主循环交回的块与认领对不上")
        });
        if let Some(holder_identity) = claim_lost_to {
            let reason = format!(
                "认领被别的持有者 {} 持着，这一帧不写",
                hexadecimal_text(&holder_identity)
            );
            return Err(self.refuse(*name, verdicts, reason));
        }
        let mut row_as_read = Some(row.clone());
        let mut last_refusal = String::new();
        // 迭代上界：CONCLUSION_WRITE_ATTEMPTS；每一圈要么写成、要么拒、要么重编重读之后再写一次
        for _ in 0..CONCLUSION_WRITE_ATTEMPTS {
            // 结论照待判行里写的结论版本键编帧
            let frame = match self.side.conclusion_frame(&row, verdicts.clone(), card) {
                Ok(frame) => frame,
                Err(reason) => {
                    self.streams_not_configured.insert(*name);
                    return Err(self.refuse(*name, verdicts, format!("编不成结论帧：{reason}")));
                }
            };
            let (request, wrote_the_rejudged_row) =
                match self.guarded_write_of_the_frame(name, row_as_read.as_ref(), &frame) {
                    Ok(request) => request,
                    Err(reason) => {
                        self.streams_not_configured.insert(*name);
                        return Err(self.refuse(*name, verdicts, reason));
                    }
                };
            let refusal = match self.claims.write_guarded_conclusion(&request) {
                Ok(GuardedConclusionWriteOutcome::Written(acknowledgement)) => {
                    self.print_a_service_line(format!(
                        "{CONCLUSION_WRITTEN_LINE_PREFIX} {name} states={} rejudged_row_written={wrote_the_rejudged_row} already_applied={}",
                        verdicts.len(),
                        acknowledgement.is_already_applied
                    ));
                    return Ok(WrittenBack {
                        conclusion_digest: frame.conclusion_digest,
                        wrote_the_rejudged_row,
                    });
                }
                Ok(GuardedConclusionWriteOutcome::Refused { refusal, .. }) => refusal,
                Err(failure) => {
                    let reason = format!("写结论那一帧没写成：{failure}");
                    return Err(self.refuse(*name, verdicts, reason));
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
                    return Err(self.refuse(*name, verdicts, reason));
                }
                GuardRefusal::GuardRowChanged { current } => {
                    last_refusal = format!("结论块那一行编帧之后变了（此刻 {current:?}）");
                    if self
                        .break_switches
                        .is_on(BreakSwitch::GuardRowChangeNotReread)
                    {
                        return Err(self.refuse(*name, verdicts, last_refusal));
                    }
                }
                GuardRefusal::DeletedRowChanged { key, current } => {
                    last_refusal = format!(
                        "删键 {} 那一行读完之后变了（此刻 {current:?}）",
                        hexadecimal_text(&key[..key.len().min(16)])
                    );
                    if key == name.pending_key(&self.key_space) {
                        if self
                            .break_switches
                            .is_on(BreakSwitch::PendingRowChangeNotReread)
                        {
                            return Err(self.refuse(*name, verdicts, last_refusal));
                        }
                        match self.side.pending_row(name) {
                            Ok(None) => row_as_read = None,
                            Ok(Some(reread)) if is_the_same_block(&reread, &row) => {
                                row_as_read = Some(reread);
                            }
                            Ok(Some(_)) => {
                                let reason = format!(
                                    "{last_refusal}；重读到的待判行指的是另一块（块键、流程、判表、区间或结论版本键不同）"
                                );
                                return Err(self.refuse(*name, verdicts, reason));
                            }
                            Err(error) => {
                                let reason = format!("{last_refusal}；重读待判行失败：{error:?}");
                                return Err(self.refuse(*name, verdicts, reason));
                            }
                        }
                    }
                }
            }
        }
        let reason = format!(
            "写结论那一帧守卫不过 {CONCLUSION_WRITE_ATTEMPTS} 次，最后一次：{last_refusal}"
        );
        Err(self.refuse(*name, verdicts, reason))
    }
}

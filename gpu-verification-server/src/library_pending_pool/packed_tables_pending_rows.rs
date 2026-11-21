//! 判表待判行那一族的取活与写回（`library_pending_pool` 模块文档「判表待判行那一族」）：扫表挑块、认领、重读、续认领、撤掉与放给别的服务、一帧带守卫写回结论字并删判表待判行。

use super::*;

/// 读回的判表对不对得上这张判表（文件头「判表待判行那一族」「流」）：流键里的判表摘要要与名字里的相同、态数要与行里的相同；
/// 对不上交回为什么（行里旋钮的摘要与名字里的是否相同，取源码那一步已核）。
fn packed_tables_stream_mismatch<Materials>(
    pending: &PackedTablesPendingRowOfTheLibrary,
    stream: &StreamOfTheRow<Materials>,
) -> Option<String> {
    if stream.stream.tables_digest != pending.name.packed_judge_tables_digest {
        return Some(format!(
            "读回的判表的摘要是 {}，名字里的是 {}",
            hexadecimal_text(&stream.stream.tables_digest),
            hexadecimal_text(&pending.name.packed_judge_tables_digest)
        ));
    }
    if stream.facts.state_count != pending.row.state_count {
        return Some(format!(
            "读回的判表有 {} 个态，判表待判行写的是 {} 个",
            stream.facts.state_count, pending.row.state_count
        ));
    }
    None
}

/// 判表待判行那一族（文件头「判表待判行那一族」）。
impl<Side: VerificationPackageSide> LibraryPendingPool<Side> {
    /// 结论字那一帧没写成、结论字留在服务里的判表。
    #[must_use]
    pub fn refused_packed_tables_writes(&self) -> &[RefusedPackedTablesWrite] {
        &self.refused_packed_tables_writes
    }

    /// 认领着、在判的判表张数。
    #[must_use]
    pub fn packed_tables_claimed_count(&self) -> usize {
        self.packed_tables_claimed.len()
    }

    /// 结论字已在表 `21`、判表待判行带守卫删掉了的张数。
    #[must_use]
    pub fn packed_tables_rows_deleted(&self) -> u64 {
        self.packed_tables_rows_deleted
    }

    /// 扫判表待判行挑能取的判表（不认领）：在判的、配不出流的不挑；
    /// 在库上现算结论字已在表 `21` 的不挑、带守卫删掉它的判表待判行。按键排。
    /// 弄坏开关 packed-tables-rows-not-scanned：一行都不扫；packed-tables-verdict-words-judged-again：不问结论字在不在。
    ///
    /// # Errors
    /// 读写判定库失败。
    pub fn packed_tables_candidates(
        &mut self,
    ) -> Result<Vec<PackedTablesPendingRowOfTheLibrary>, LibraryFailure> {
        if self
            .break_switches
            .is_on(BreakSwitch::PackedTablesRowsNotScanned)
        {
            return Ok(Vec::new());
        }
        let scanned = match self.side.packed_tables_pending_rows() {
            Ok(scanned) => scanned,
            Err(error) => {
                let reason = library_failure_or_the_reason(error)?;
                let line = format!(
                    "{PENDING_ROW_SKIPPED_LINE_PREFIX} 扫不了判表待判行（库里有一行不合验证包的布局）：{reason}"
                );
                self.name_once(PACKED_TABLES_SCAN_PROBLEM_NAMED_KEY.to_vec(), line);
                return Ok(Vec::new());
            }
        };
        let mut candidates = Vec::new();
        for pending in scanned {
            if self.packed_tables_claimed.contains_key(&pending.name)
                || self.packed_tables_not_configured.contains(&pending.name)
            {
                continue;
            }
            if !self
                .break_switches
                .is_on(BreakSwitch::PackedTablesVerdictWordsJudgedAgain)
            {
                match self
                    .side
                    .packed_tables_verdict_words_are_in_the_library(&pending.name, &pending.row)
                {
                    Ok(true) => {
                        self.delete_the_pending_row_of_judged_packed_tables(&pending)?;
                        continue;
                    }
                    Ok(false) => {}
                    Err(error) => {
                        let reason = library_failure_or_the_reason(error)?;
                        let line = format!(
                            "{PENDING_ROW_SKIPPED_LINE_PREFIX} {} 判表结论字在不在现算不了：{reason}",
                            pending.name
                        );
                        self.name_once(pending.name.pending_key(&self.key_space), line);
                        continue;
                    }
                }
            }
            candidates.push(pending);
        }
        Ok(candidates)
    }

    /// 结论字已在表 `21` 的判表：一帧写结论只删它的判表待判行，删键的期待是这里读到的那一份（判段在这之后补写、升成强制现判的不删，
    /// 下一轮照它此刻的样子再定）；别的持有者持着的不删。这一帧没有结论行：守卫取这一族的认领键（认领键只记认领、从不当数据行写，样子恒为「不在」）。
    fn delete_the_pending_row_of_judged_packed_tables(
        &mut self,
        pending: &PackedTablesPendingRowOfTheLibrary,
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
                self.packed_tables_rows_deleted += 1;
                self.print_a_service_line(format!(
                    "{PACKED_TABLES_PENDING_ROW_DELETED_LINE_PREFIX} {}",
                    pending.name
                ));
            }
            GuardedConclusionWriteOutcome::Refused { refusal, message } => {
                let line = format!(
                    "{PENDING_ROW_SKIPPED_LINE_PREFIX} {} 结论字已在、判表待判行这一回没删（{refusal:?}：{message}）：下一轮照它此刻的样子再定",
                    pending.name
                );
                self.name_once(pending.name.claim_key(), line);
            }
        }
        Ok(())
    }

    /// 认领挑出来的判表：认领到的重读判表待判行（不在了或指的是另一张的不取、放掉刚认领到的），别的持有者持着的不取；
    /// 还是同一张的配上流（照行读回判表、照旋钮栏取源码），流键或态数对不上、读不回的点名一次、这个进程里不再取，认领留到租期过了。
    /// 弄坏开关 packed-tables-row-not-reread-after-the-claim：认领到之后不重读。
    ///
    /// # Errors
    /// 认领、重读、放认领时读写判定库失败。
    pub fn claim_the_packed_tables_candidates(
        &mut self,
        candidates: Vec<PackedTablesPendingRowOfTheLibrary>,
    ) -> Result<Vec<TakenPackedTables<Side::Materials>>, LibraryFailure> {
        let mut taken = Vec::new();
        let mut held_by_others_this_round = BTreeSet::new();
        for pending in candidates {
            if self.packed_tables_claimed.contains_key(&pending.name) {
                continue;
            }
            match claim_a_packed_tables_pending_row(
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
                    if !self
                        .packed_tables_held_by_others_last_round
                        .contains(&observation)
                    {
                        self.print_a_service_line(format!(
                            "{PACKED_TABLES_CLAIM_HELD_BY_OTHER_LINE_PREFIX} {} holder={} remaining_lease_milliseconds={remaining_lease_milliseconds}",
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
                        "{PENDING_ROW_SKIPPED_LINE_PREFIX} {} 库里有判表认领那把键（认领键不当数据行写）：不取",
                        pending.name
                    );
                    self.name_once(pending.name.claim_key(), line);
                    continue;
                }
            }
            let pending = if self
                .break_switches
                .is_on(BreakSwitch::PackedTablesRowNotRereadAfterTheClaim)
            {
                pending
            } else {
                match self.reread_the_packed_tables_row_after_the_claim(pending)? {
                    Some(reread) => reread,
                    None => continue,
                }
            };
            let configured = self
                .side
                .packed_tables_stream(&pending.name, &pending.row)
                .and_then(
                    |stream| match packed_tables_stream_mismatch(&pending, &stream) {
                        Some(mismatch) => Err(mismatch),
                        None => Ok(stream),
                    },
                );
            let stream = match configured {
                Ok(stream) => stream,
                Err(reason) => {
                    let holder = self.holder();
                    let _ = self.release_the_claim_keys(&holder, &[pending.name.claim_key()]);
                    let line = format!(
                        "{PENDING_ROW_SKIPPED_LINE_PREFIX} {} 配不出流（这个进程里不再取它，认领已放）：{reason}",
                        pending.name
                    );
                    self.name_once(pending.name.pending_key(&self.key_space), line);
                    self.packed_tables_not_configured.insert(pending.name);
                    continue;
                }
            };
            self.packed_tables_claimed.insert(
                pending.name,
                ClaimedPackedTables {
                    row: pending.row,
                    renewed_at: Instant::now(),
                    claim_lost_to: None,
                },
            );
            self.print_a_service_line(format!(
                "{PACKED_TABLES_CLAIMED_LINE_PREFIX} {} states={}",
                pending.name, pending.row.state_count
            ));
            taken.push(TakenPackedTables { pending, stream });
        }
        self.packed_tables_held_by_others_last_round = held_by_others_this_round;
        Ok(taken)
    }

    /// 认领到之后重读这张判表的判表待判行：还是同一张的交回重读到的那一行；不在了、指的是另一张、重读不了（行不合布局，点名一次）的，
    /// 以自己的持有者放掉刚认领到的认领、交 `None`。
    ///
    /// # Errors
    /// 重读、放认领时读写判定库失败。
    fn reread_the_packed_tables_row_after_the_claim(
        &mut self,
        pending: PackedTablesPendingRowOfTheLibrary,
    ) -> Result<Option<PackedTablesPendingRowOfTheLibrary>, LibraryFailure> {
        let why = match self.side.packed_tables_pending_row(&pending.name) {
            Ok(Some(reread)) if is_the_same_packed_tables_row(&reread, &pending.row) => {
                return Ok(Some(PackedTablesPendingRowOfTheLibrary {
                    name: pending.name,
                    row: reread,
                }));
            }
            Ok(Some(_)) => "pending-row-is-another-table",
            Ok(None) => "pending-row-gone",
            Err(error) => {
                let reason = library_failure_or_the_reason(error)?;
                let line = format!(
                    "{PENDING_ROW_SKIPPED_LINE_PREFIX} {} 认领到之后重读不了判表待判行：{reason}",
                    pending.name
                );
                self.name_once(pending.name.pending_key(&self.key_space), line);
                "pending-row-unreadable"
            }
        };
        let holder = self.holder();
        let released = self.release_the_claim_keys(&holder, &[pending.name.claim_key()])?;
        self.print_a_service_line(format!(
            "{PACKED_TABLES_CLAIM_RELEASED_AFTER_THE_REREAD_LINE_PREFIX} {} why={why} released={}",
            pending.name, released.released
        ));
        Ok(None)
    }

    /// 在判的判表里上一次续认领过了三分之一租期的，续一次；续不上（别的持有者持着）的交回名字，照判，写回时那一帧核认领、不写。
    ///
    /// # Errors
    /// 读写判定库失败。
    pub fn renew_the_packed_tables_claims(
        &mut self,
    ) -> Result<Vec<PackedTablesName>, LibraryFailure> {
        let renew_after = Duration::from_millis(u64::from(self.claim_lease_milliseconds) / 3);
        let due: Vec<PackedTablesName> = self
            .packed_tables_claimed
            .iter()
            .filter(|(_, claimed)| claimed.renewed_at.elapsed() >= renew_after)
            .map(|(name, _)| *name)
            .collect();
        let mut lost = Vec::new();
        for name in due {
            let answer = claim_a_packed_tables_pending_row(
                &mut self.claims,
                &name,
                self.claim_lease_milliseconds,
            )?;
            if answer == ClaimAnswer::Claimed {
                if let Some(claimed) = self.packed_tables_claimed.get_mut(&name) {
                    claimed.renewed_at = Instant::now();
                }
            } else {
                if let ClaimAnswer::HeldByOther { holder, .. } = &answer {
                    if let Some(claimed) = self.packed_tables_claimed.get_mut(&name) {
                        claimed.claim_lost_to = Some(holder.clone());
                    }
                }
                lost.push(name);
            }
        }
        Ok(lost)
    }

    /// 主循环把这张判表撤了（判它的卡坏了、登记不上）：不再当在判，认领留到租期过了，下一轮扫到再取。
    pub fn withdraw_the_packed_tables(&mut self, name: &PackedTablesName) {
        self.packed_tables_claimed.remove(name);
    }

    /// 这一台判不了这张判表（文件头「放认领」）：不再当在判，当场以自己的持有者放掉它的认领，别的服务随即取得到。
    ///
    /// # Errors
    /// 放认领时读写判定库失败（认领留到租期过了）。
    pub fn release_the_packed_tables_claim_left_to_another_service(
        &mut self,
        name: &PackedTablesName,
    ) -> Result<ClaimsReleased, LibraryFailure> {
        self.packed_tables_claimed.remove(name);
        let holder = self.holder();
        self.release_the_claim_keys(&holder, &[name.claim_key()])
    }

    fn refuse_the_packed_tables_write(
        &mut self,
        name: PackedTablesName,
        verdicts: Vec<StateVerdict>,
        reason: String,
    ) -> Box<RefusedPackedTablesWrite> {
        let holder = self.holder();
        let _ = self.release_the_claim_keys(&holder, &[name.claim_key()]);
        let refused = RefusedPackedTablesWrite {
            name,
            verdicts,
            reason,
        };
        self.print_a_service_line(format!(
            "{PACKED_TABLES_VERDICT_WORDS_WRITE_REFUSED_LINE_PREFIX} {} states={} reason={}",
            refused.name,
            refused.verdicts.len(),
            refused.reason
        ));
        self.refused_packed_tables_writes.push(refused.clone());
        Box::new(refused)
    }

    /// 验证包编的一帧放进判定库服务的写结论帧（文件头「判表待判行那一族」「写回」）：认领键与持有者是服务的，守卫键是表 `21` 那一行、
    /// 期待是编帧之前读到的那一份，条目是验证包那一帧要写的行，删键是判表待判行、期待是 `row_as_read`（不在了是「不在」）。
    /// 弄坏开关 packed-tables-verdict-words-guard-taken-at-the-write、packed-tables-pending-row-guard-taken-at-the-write：那一个期待改取写之前此刻库里的样子。
    ///
    /// # Errors
    /// 有一行的值长过一段（写结论那一帧不分段，一帧原子写不下）；弄坏开关下读此刻的样子失败。
    fn guarded_write_of_the_packed_tables_frame(
        &mut self,
        name: &PackedTablesName,
        row_as_read: Option<&PackedTablesPendingRow>,
        frame: &PackedTablesFrame,
    ) -> Result<GuardedConclusionWrite, String> {
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
        let mut guard_expectation =
            GuardRowState::of_stored_value(frame.verdict_words_row_as_read.as_deref());
        let mut pending_expectation = GuardRowState::of_stored_value(
            row_as_read.map(|_| frame.pending_row_as_read.as_slice()),
        );
        let is_the_verdict_words_guard_taken_at_the_write = self
            .break_switches
            .is_on(BreakSwitch::PackedTablesVerdictWordsGuardTakenAtTheWrite);
        let is_the_pending_guard_taken_at_the_write = self
            .break_switches
            .is_on(BreakSwitch::PackedTablesPendingRowGuardTakenAtTheWrite);
        if is_the_verdict_words_guard_taken_at_the_write || is_the_pending_guard_taken_at_the_write
        {
            let mut rows_now = self
                .claims
                .read(&[
                    (Table::Conclusions, frame.verdict_words_row_key.clone()),
                    (Table::Conclusions, frame.pending_row_key_to_delete.clone()),
                ])
                .map_err(|failure| format!("读两个守卫此刻的样子失败：{failure}"))?
                .into_iter();
            let verdict_words_row_now = rows_now.next().flatten();
            let pending_row_now = rows_now.next().flatten();
            if is_the_verdict_words_guard_taken_at_the_write {
                guard_expectation =
                    GuardRowState::of_stored_value(verdict_words_row_now.as_deref());
            }
            if is_the_pending_guard_taken_at_the_write {
                pending_expectation = GuardRowState::of_stored_value(pending_row_now.as_deref());
            }
        }
        Ok(GuardedConclusionWrite {
            holder_identity: self.holder(),
            claim_key: name.claim_key(),
            guard_key: frame.verdict_words_row_key.clone(),
            guard_expectation,
            entries: frame
                .rows_to_put
                .iter()
                .map(|(key, value)| (Table::Conclusions, key.clone(), value.clone()))
                .collect(),
            deleted_keys: vec![GuardedDeletion {
                key: frame.pending_row_key_to_delete.clone(),
                expectation: pending_expectation,
            }],
        })
    }

    /// 一张判表判完：编那一帧，一帧写下表 `21` 那一行、删判表待判行、放认领，守卫不过照种类重编、重读再写（文件头「判表待判行那一族」「写回」）。
    /// 弄坏开关 packed-tables-guard-refusal-not-retried：守卫不过当场报红。
    ///
    /// # Errors
    /// 那一帧没写成（结论字留在 [`Self::refused_packed_tables_writes`]、打了红行）；交回的那一份装箱，名字的两段摘要放在值里太大。
    ///
    /// # Panics
    /// 这张判表不在认领着、在判的判表里（主循环交回的与认领对不上）。
    pub fn write_back_the_packed_tables(
        &mut self,
        name: &PackedTablesName,
        verdicts: Vec<StateVerdict>,
        card: &CardThatJudgedAPendingBlock,
    ) -> Result<PackedTablesWrittenBack, Box<RefusedPackedTablesWrite>> {
        let ClaimedPackedTables {
            row, claim_lost_to, ..
        } = self.packed_tables_claimed.remove(name).unwrap_or_else(|| {
            panic!("{name} 不在这个服务认领着的判表里：主循环交回的与认领对不上")
        });
        if let Some(holder_identity) = claim_lost_to {
            let reason = format!(
                "认领被别的持有者 {} 持着，这一帧不写",
                hexadecimal_text(&holder_identity)
            );
            return Err(self.refuse_the_packed_tables_write(*name, verdicts, reason));
        }
        let device = judging_device_of_the_card(card);
        let mut row_as_read = Some(row);
        let mut last_refusal = String::new();
        let is_retried = !self
            .break_switches
            .is_on(BreakSwitch::PackedTablesGuardRefusalNotRetried);
        // 迭代上界：CONCLUSION_WRITE_ATTEMPTS；每一圈要么写成、要么拒、要么重编重读之后再写一次
        for _ in 0..CONCLUSION_WRITE_ATTEMPTS {
            let row_of_the_frame = row_as_read.unwrap_or(row);
            let frame =
                match self
                    .side
                    .packed_tables_frame(name, &row_of_the_frame, &verdicts, &device)
                {
                    Ok(frame) => frame,
                    Err(reason) => {
                        self.packed_tables_not_configured.insert(*name);
                        let reason = format!("编不成结论字那一帧：{reason}");
                        return Err(self.refuse_the_packed_tables_write(*name, verdicts, reason));
                    }
                };
            let request = match self.guarded_write_of_the_packed_tables_frame(
                name,
                row_as_read.as_ref(),
                &frame,
            ) {
                Ok(request) => request,
                Err(reason) => {
                    self.packed_tables_not_configured.insert(*name);
                    return Err(self.refuse_the_packed_tables_write(*name, verdicts, reason));
                }
            };
            let refusal = match self.claims.write_guarded_conclusion(&request) {
                Ok(GuardedConclusionWriteOutcome::Written(acknowledgement)) => {
                    let rows_written = frame.rows_to_put.len();
                    self.print_a_service_line(format!(
                        "{PACKED_TABLES_VERDICT_WORDS_WRITTEN_LINE_PREFIX} {name} states={} rows_written={rows_written} already_applied={}",
                        verdicts.len(),
                        acknowledgement.is_already_applied
                    ));
                    return Ok(PackedTablesWrittenBack { rows_written });
                }
                Ok(GuardedConclusionWriteOutcome::Refused { refusal, .. }) => refusal,
                Err(failure) => {
                    let reason = format!("结论字那一帧没写成：{failure}");
                    return Err(self.refuse_the_packed_tables_write(*name, verdicts, reason));
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
                    return Err(self.refuse_the_packed_tables_write(*name, verdicts, reason));
                }
                GuardRefusal::GuardRowChanged { current } => {
                    last_refusal = format!("表 21 那一行编帧之后变了（此刻 {current:?}）");
                    if !is_retried {
                        return Err(self.refuse_the_packed_tables_write(
                            *name,
                            verdicts,
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
                        return Err(self.refuse_the_packed_tables_write(
                            *name,
                            verdicts,
                            last_refusal,
                        ));
                    }
                    if key == name.pending_key(&self.key_space) {
                        match self.side.packed_tables_pending_row(name) {
                            Ok(None) => row_as_read = None,
                            Ok(Some(reread)) if is_the_same_packed_tables_row(&reread, &row) => {
                                row_as_read = Some(reread);
                            }
                            Ok(Some(_)) => {
                                let reason = format!(
                                    "{last_refusal}；重读到的判表待判行指的是另一张（旋钮栏或态数不同）"
                                );
                                return Err(
                                    self.refuse_the_packed_tables_write(*name, verdicts, reason)
                                );
                            }
                            Err(error) => {
                                let reason =
                                    format!("{last_refusal}；重读判表待判行失败：{error:?}");
                                return Err(
                                    self.refuse_the_packed_tables_write(*name, verdicts, reason)
                                );
                            }
                        }
                    }
                }
            }
        }
        let reason = format!(
            "结论字那一帧守卫不过 {CONCLUSION_WRITE_ATTEMPTS} 次，最后一次：{last_refusal}"
        );
        Err(self.refuse_the_packed_tables_write(*name, verdicts, reason))
    }
}

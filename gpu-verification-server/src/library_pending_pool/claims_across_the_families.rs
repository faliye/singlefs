//! 三族待判行共用的放认领（`library_pending_pool` 模块文档「放认领」）：以持有者放认领键，列库里全部待判行的名字与认领键，起来时放上一个进程留下的，停的时候放自己的。

use super::*;

impl<Side: VerificationPackageSide> LibraryPendingPool<Side> {
    /// 以 `holder` 放掉这几把认领键上的认领（放认领帧，一帧至多 [`CLAIM_RELEASE_KEYS_PER_FRAME`] 把；只放 `holder` 名下的）。
    ///
    /// # Errors
    /// 读写判定库失败；回的结局个数与键数不同。
    pub(super) fn release_the_claim_keys(
        &mut self,
        holder: &[u8],
        claim_keys: &[Vec<u8>],
    ) -> Result<ClaimsReleased, LibraryFailure> {
        let mut released = ClaimsReleased::default();
        for keys_of_one_frame in claim_keys.chunks(CLAIM_RELEASE_KEYS_PER_FRAME) {
            let keys: Vec<(Table, Vec<u8>)> = keys_of_one_frame
                .iter()
                .map(|key| (Table::Conclusions, key.clone()))
                .collect();
            let outcomes = self.claims.release_claims_as(holder, &keys)?;
            if outcomes.len() != keys.len() {
                return Err(LibraryFailure::from(ClientFailure::Protocol(format!(
                    "放 {} 把认领回了 {} 个结局",
                    keys.len(),
                    outcomes.len()
                ))));
            }
            for outcome in outcomes {
                match outcome {
                    ClaimReleaseOutcome::Released => released.released += 1,
                    ClaimReleaseOutcome::NotClaimed => released.not_claimed += 1,
                    ClaimReleaseOutcome::HeldByOther => released.held_by_others += 1,
                }
            }
        }
        Ok(released)
    }

    /// 库里此刻全部待判行的名字；扫不了（有一行不合布局）的点名一次、交空（那一回放不了认领，认领到租期过了自己失效）。
    ///
    /// # Errors
    /// 读判定库失败。
    fn names_of_every_pending_row(&mut self) -> Result<Vec<PendingBlockName>, LibraryFailure> {
        match self.side.pending_rows() {
            Ok(rows) => Ok(rows.into_iter().map(|pending| pending.name).collect()),
            Err(error) => {
                let reason = library_failure_or_the_reason(error)?;
                let line = format!(
                    "{PENDING_ROW_SKIPPED_LINE_PREFIX} 扫不了待判行（库里有一行不合验证包的布局）：{reason}"
                );
                self.name_once(SCAN_PROBLEM_NAMED_KEY.to_vec(), line);
                Ok(Vec::new())
            }
        }
    }

    /// 库里此刻全部核对待判行的名字；扫不了（有一行不合布局）的点名一次、交空（那一回放不了这一族的认领，认领到租期过了自己失效）。
    ///
    /// # Errors
    /// 读判定库失败。
    fn names_of_every_facts_verifier_pending_row(
        &mut self,
    ) -> Result<Vec<FactsVerifierBlockName>, LibraryFailure> {
        match self.side.facts_verifier_pending_rows() {
            Ok(rows) => Ok(rows.into_iter().map(|pending| pending.name).collect()),
            Err(error) => {
                let reason = library_failure_or_the_reason(error)?;
                let line = format!(
                    "{PENDING_ROW_SKIPPED_LINE_PREFIX} 扫不了核对待判行（库里有一行不合验证包的布局）：{reason}"
                );
                self.name_once(FACTS_VERIFIER_SCAN_PROBLEM_NAMED_KEY.to_vec(), line);
                Ok(Vec::new())
            }
        }
    }

    /// 库里此刻全部判表待判行的名字；扫不了（有一行不合布局）的点名一次、交空（那一回放不了这一族的认领，认领到租期过了自己失效）。
    ///
    /// # Errors
    /// 读判定库失败。
    fn names_of_every_packed_tables_pending_row(
        &mut self,
    ) -> Result<Vec<PackedTablesName>, LibraryFailure> {
        match self.side.packed_tables_pending_rows() {
            Ok(rows) => Ok(rows.into_iter().map(|pending| pending.name).collect()),
            Err(error) => {
                let reason = library_failure_or_the_reason(error)?;
                let line = format!(
                    "{PENDING_ROW_SKIPPED_LINE_PREFIX} 扫不了判表待判行（库里有一行不合验证包的布局）：{reason}"
                );
                self.name_once(PACKED_TABLES_SCAN_PROBLEM_NAMED_KEY.to_vec(), line);
                Ok(Vec::new())
            }
        }
    }

    /// 库里此刻三族全部待判块的认领键（放认领用）：待判行那一族的、核对待判行那一族的、判表待判行那一族的，给了借卡键的再加卡键。
    /// 弄坏开关 facts-verifier-claims-left-at-the-release：不交核对待判行那一族的；packed-tables-claims-left-at-the-release：不交判表待判行那一族的。
    ///
    /// # Errors
    /// 读判定库失败。
    fn claim_keys_of_every_pending_block(&mut self) -> Result<Vec<Vec<u8>>, LibraryFailure> {
        let mut keys: Vec<Vec<u8>> = self
            .names_of_every_pending_row()?
            .iter()
            .map(PendingBlockName::claim_key)
            .collect();
        if !self
            .break_switches
            .is_on(BreakSwitch::FactsVerifierClaimsLeftAtTheRelease)
        {
            keys.extend(
                self.names_of_every_facts_verifier_pending_row()?
                    .iter()
                    .map(FactsVerifierBlockName::claim_key),
            );
        }
        if !self
            .break_switches
            .is_on(BreakSwitch::PackedTablesClaimsLeftAtTheRelease)
        {
            keys.extend(
                self.names_of_every_packed_tables_pending_row()?
                    .iter()
                    .map(PackedTablesName::claim_key),
            );
        }
        keys.extend(self.card_lease.as_ref().map(CardLeaseKeys::card_key));
        Ok(keys)
    }

    /// 起来时（取活之前）放上一个服务进程留下的认领（文件头「放认领」）：`serving_line` 是状态目录里 `serving` 那一份的内容（没有这份文件是 `None`）。
    /// 上一个进程没了就以它的持有者放掉库里三族各待判块上它的认领；还活着、那一行认不出、判不了活没活的不放，打一行说明。
    ///
    /// # Errors
    /// 扫库、放认领时读写判定库失败。
    pub fn release_the_claims_left_by_the_previous_process(
        &mut self,
        serving_line: Option<&str>,
    ) -> Result<PreviousProcessClaims, LibraryFailure> {
        let Some(serving_line) = serving_line else {
            return Ok(PreviousProcessClaims::NoPreviousProcess);
        };
        let previous = match PreviousServiceProcess::of_the_serving_line(serving_line.trim_end()) {
            Ok(previous) => previous,
            Err(reason) => {
                return Ok(self.keep_the_previous_claims(format!("「在跑」那一行认不出：{reason}")));
            }
        };
        let liveness = match previous.process.liveness() {
            Ok(liveness) => liveness,
            Err(reason) => {
                return Ok(self.keep_the_previous_claims(format!(
                    "判不了上一个进程 {} 还在不在：{reason}",
                    previous.process.process_number
                )));
            }
        };
        let is_taken_for_gone = self
            .break_switches
            .is_on(BreakSwitch::PreviousProcessTakenForGone);
        match liveness {
            ProcessLiveness::Running if !is_taken_for_gone => {
                self.print_a_service_line(format!(
                    "{PREVIOUS_PROCESS_CLAIMS_KEPT_LINE_PREFIX} why=still-running process={} holder={}",
                    previous.process.process_number,
                    hexadecimal_text(&previous.holder)
                ));
                return Ok(PreviousProcessClaims::KeptWhileItRuns);
            }
            ProcessLiveness::Running | ProcessLiveness::Gone => {}
        }
        if self
            .break_switches
            .is_on(BreakSwitch::PreviousProcessClaimsKept)
        {
            return Ok(PreviousProcessClaims::Released(ClaimsReleased::default()));
        }
        let claim_keys = self.claim_keys_of_every_pending_block()?;
        let released = self.release_the_claim_keys(&previous.holder, &claim_keys)?;
        self.print_a_service_line(format!(
            "{CLAIMS_RELEASED_LINE_PREFIX} why=previous-process-gone process={} holder={} released={} not_claimed={} held_by_others={}",
            previous.process.process_number,
            hexadecimal_text(&previous.holder),
            released.released,
            released.not_claimed,
            released.held_by_others
        ));
        Ok(PreviousProcessClaims::Released(released))
    }

    fn keep_the_previous_claims(&mut self, reason: String) -> PreviousProcessClaims {
        self.print_a_service_line(format!(
            "{PREVIOUS_PROCESS_CLAIMS_KEPT_LINE_PREFIX} why=undetermined reason={reason}"
        ));
        PreviousProcessClaims::KeptUndetermined { reason }
    }

    /// 停的时候（取活循环收尾之后：入了池的块已判完写回，余下的是撤了的、配不出流的、读不回材料的）以自己的持有者放掉三族还持着的认领，
    /// 别的持有者随即取得到，不等租期。
    ///
    /// # Errors
    /// 扫库、放认领时读写判定库失败。
    pub fn release_the_claims_of_this_process(&mut self) -> Result<ClaimsReleased, LibraryFailure> {
        if self.break_switches.is_on(BreakSwitch::ClaimsKeptAtTheStop) {
            return Ok(ClaimsReleased::default());
        }
        let claim_keys = self.claim_keys_of_every_pending_block()?;
        let holder = self.holder();
        let released = self.release_the_claim_keys(&holder, &claim_keys)?;
        self.claimed.clear();
        self.facts_claimed.clear();
        self.packed_tables_claimed.clear();
        self.print_a_service_line(format!(
            "{CLAIMS_RELEASED_LINE_PREFIX} why=stopping holder={} released={} not_claimed={} held_by_others={}",
            hexadecimal_text(&holder),
            released.released,
            released.not_claimed,
            released.held_by_others
        ));
        Ok(released)
    }
}

//! 认领相关的帧（`reliable_client` 模块文档「认领」）：认领、放认领、列认领与 GPU 块的认领。

use super::*;

impl ReliableClient {
    /// 以 `holder` 的身份认领几块（`gpu_judging_rows` 文件头「认领」）：认领键是各块的 `gpu-claim/` 那一把，租期 `lease_milliseconds`；
    /// 结局与 [`Self::claim`] 相同、次序与 `blocks` 相同。持有者身份不是这个客户端的身份（同一个客户端可以替几个判线程认领）。
    ///
    /// # Errors
    /// 同 [`Self::claim_as`]。
    pub fn claim_gpu_blocks(
        &mut self,
        holder: &GpuClaimHolder,
        blocks: &[(Vec<u8>, u64)],
        lease_milliseconds: u32,
    ) -> Result<Vec<ClaimOutcome>, ClientFailure> {
        let keys: Vec<(Table, Vec<u8>)> = blocks
            .iter()
            .map(|(stream_key, block_number)| {
                (Table::Conclusions, gpu_claim_key(stream_key, *block_number))
            })
            .collect();
        self.claim_as(&holder.identity(), &keys, lease_milliseconds)
    }

    pub fn submit_claim(
        &mut self,
        keys: &[(Table, Vec<u8>)],
        lease_milliseconds: u32,
    ) -> Result<OperationIdentifier, ClientFailure> {
        let holder_identity = self.client_identity.clone();
        self.submit_claim_as(&holder_identity, keys, lease_milliseconds)
    }

    /// 以 `holder_identity`（1 到 255 字节，不一定是这个客户端的身份）认领一批键：GPU 服务一个客户端替几张卡、几个判线程各自认领
    /// （`gpu_judging_rows` 文件头「认领」）。别的同 [`Self::submit_claim`]。
    ///
    /// # Errors
    /// 持有者身份长度不对（[`ClientFailure::Protocol`]，没发）；帧体超过帧长上限；发的时候连不上、服务停了。
    pub fn submit_claim_as(
        &mut self,
        holder_identity: &[u8],
        keys: &[(Table, Vec<u8>)],
        lease_milliseconds: u32,
    ) -> Result<OperationIdentifier, ClientFailure> {
        if holder_identity.is_empty() || holder_identity.len() > MAXIMUM_CLIENT_IDENTITY_BYTES {
            return Err(ClientFailure::Protocol(format!(
                "认领的持有者身份 {} 字节，要 1 到 {MAXIMUM_CLIENT_IDENTITY_BYTES} 字节；没发",
                holder_identity.len()
            )));
        }
        self.wait_for_window()?;
        let borrowed: Vec<(Table, &[u8])> = keys
            .iter()
            .map(|(table, key)| (*table, key.as_slice()))
            .collect();
        let frame = encode_claim_frame(0, holder_identity, lease_milliseconds, &borrowed);
        if let Some(failure) = oversize_frame_failure(&frame, &format!("认领 {} 个键", keys.len()))
            .filter(|_| !self.is_enabled(BreakSwitch::OversizeFrameSentAndRetried))
        {
            return Err(failure);
        }
        let operation = self.register(
            OperationKind::Claim {
                lease_milliseconds,
                submitted_at: Instant::now(),
            },
            frame,
        );
        self.send_unsent()?;
        Ok(operation)
    }

    pub fn claim(
        &mut self,
        keys: &[(Table, Vec<u8>)],
        lease_milliseconds: u32,
    ) -> Result<Vec<ClaimOutcome>, ClientFailure> {
        let holder_identity = self.client_identity.clone();
        self.claim_as(&holder_identity, keys, lease_milliseconds)
    }

    /// 以 `holder_identity` 认领一批键并等结局（[`Self::submit_claim_as`]）。
    ///
    /// # Errors
    /// 同 [`Self::submit_claim_as`]；等结局时连不上、忙到上限（退 75），服务拒了这一帧。
    pub fn claim_as(
        &mut self,
        holder_identity: &[u8],
        keys: &[(Table, Vec<u8>)],
        lease_milliseconds: u32,
    ) -> Result<Vec<ClaimOutcome>, ClientFailure> {
        let operation = self.submit_claim_as(holder_identity, keys, lease_milliseconds)?;
        match self.wait_for(operation)? {
            Completion::Claims { outcomes, .. } => Ok(outcomes),
            Completion::Failed { failure, .. } => Err(failure),
            other @ (Completion::WriteAcknowledged { .. }
            | Completion::Values { .. }
            | Completion::Statistics { .. }
            | Completion::ScanEntries { .. }
            | Completion::PrefixDeleted { .. }
            | Completion::ClaimsReleased { .. }
            | Completion::ClaimListing { .. }
            | Completion::ConclusionRefused { .. }) => Err(ClientFailure::Protocol(format!(
                "认领收到的不是认领结局：{other:?}"
            ))),
        }
    }

    /// 以 `holder_identity` 放掉这几把 GPU 认领键上的认领并等结局（`protocol` 文件头「放认领」）：结局逐键一个、次序与 `keys` 相同；
    /// 只放这个持有者名下的，别的持有者持着的回「别人持有」、不动。不进追加日志，忙、断线与删前缀同一套。
    ///
    /// # Errors
    /// 持有者身份长度不对、帧体超过帧长上限（[`ClientFailure::Protocol`]，没发）；连不上、忙到上限、服务停了（退 75）；
    /// 被拒（有一把键不是 GPU 认领键）、回的不是放认领的结局。
    pub fn release_claims_as(
        &mut self,
        holder_identity: &[u8],
        keys: &[(Table, Vec<u8>)],
    ) -> Result<Vec<ClaimReleaseOutcome>, ClientFailure> {
        if holder_identity.is_empty() || holder_identity.len() > MAXIMUM_CLIENT_IDENTITY_BYTES {
            return Err(ClientFailure::Protocol(format!(
                "放认领的持有者身份 {} 字节，要 1 到 {MAXIMUM_CLIENT_IDENTITY_BYTES} 字节；没发",
                holder_identity.len()
            )));
        }
        self.wait_for_window()?;
        let borrowed: Vec<(Table, &[u8])> = keys
            .iter()
            .map(|(table, key)| (*table, key.as_slice()))
            .collect();
        let frame = encode_release_claims_frame(0, holder_identity, &borrowed);
        if let Some(failure) =
            oversize_frame_failure(&frame, &format!("放认领 {} 个键", keys.len()))
                .filter(|_| !self.is_enabled(BreakSwitch::OversizeFrameSentAndRetried))
        {
            return Err(failure);
        }
        let operation = self.register(OperationKind::ReleaseClaims, frame);
        self.send_unsent()?;
        match self.wait_for(operation)? {
            Completion::ClaimsReleased { outcomes, .. } => Ok(outcomes),
            Completion::Failed { failure, .. } => Err(failure),
            other @ (Completion::WriteAcknowledged { .. }
            | Completion::Values { .. }
            | Completion::Claims { .. }
            | Completion::Statistics { .. }
            | Completion::ScanEntries { .. }
            | Completion::PrefixDeleted { .. }
            | Completion::ClaimListing { .. }
            | Completion::ConclusionRefused { .. }) => Err(ClientFailure::Protocol(format!(
                "放认领收到的不是放认领的结局：{other:?}"
            ))),
        }
    }

    /// 以 `holder_identity` 放掉几块的认领（认领键是各块的 `gpu-claim/` 那一把，`gpu_judging_rows` 文件头「认领」）；结局同 [`Self::release_claims_as`]。
    ///
    /// # Errors
    /// 同 [`Self::release_claims_as`]。
    pub fn release_gpu_block_claims(
        &mut self,
        holder_identity: &[u8],
        blocks: &[(Vec<u8>, u64)],
    ) -> Result<Vec<ClaimReleaseOutcome>, ClientFailure> {
        let keys: Vec<(Table, Vec<u8>)> = blocks
            .iter()
            .map(|(stream_key, block_number)| {
                (Table::Conclusions, gpu_claim_key(stream_key, *block_number))
            })
            .collect();
        self.release_claims_as(holder_identity, &keys)
    }

    /// 列认领一帧，等回应（`protocol.rs` 文件头「列认领」）：键以 `prefix` 起头（要以 `gpu-claim/` 起头）、租期此刻没到的认领，按键升序，从排在 `after_key` 之后的
    /// 第一把键起（空是从前缀起），至多 `maximum_entries` 条（至少 1）。一帧被截断时调用方接着用这一帧最后一条的键续列。不进追加日志，忙、断线与按前缀扫同一套。
    ///
    /// # Errors
    /// 帧体超过帧长上限（[`ClientFailure::Protocol`]，没发）；连不上、忙到上限、服务停了（退 75）；被拒（前缀不是 GPU 认领键的族前缀、条数上限 0、
    /// 续扫起点不以前缀起头、一条都装不下一帧）、回的不是列认领的结局。
    pub fn list_claims_after(
        &mut self,
        prefix: &[u8],
        after_key: &[u8],
        maximum_entries: u32,
    ) -> Result<ClaimListingFrame, ClientFailure> {
        self.wait_for_window()?;
        let frame = encode_list_claims_frame(0, prefix, after_key, maximum_entries);
        if let Some(failure) = oversize_frame_failure(
            &frame,
            &format!(
                "列认领（前缀 {} 字节、续扫起点 {} 字节）",
                prefix.len(),
                after_key.len()
            ),
        )
        .filter(|_| !self.is_enabled(BreakSwitch::OversizeFrameSentAndRetried))
        {
            return Err(failure);
        }
        let operation = self.register(OperationKind::ListClaims, frame);
        self.send_unsent()?;
        match self.wait_for(operation)? {
            Completion::ClaimListing {
                is_truncated,
                claims,
                ..
            } => Ok(ClaimListingFrame {
                is_truncated,
                claims,
            }),
            Completion::Failed { failure, .. } => Err(failure),
            other @ (Completion::WriteAcknowledged { .. }
            | Completion::Values { .. }
            | Completion::Claims { .. }
            | Completion::Statistics { .. }
            | Completion::ScanEntries { .. }
            | Completion::PrefixDeleted { .. }
            | Completion::ClaimsReleased { .. }
            | Completion::ConclusionRefused { .. }) => Err(ClientFailure::Protocol(format!(
                "列认领收到的不是列认领的结局：{other:?}"
            ))),
        }
    }

    /// 列出 `prefix` 下此刻有效（租期没到）的全部认领：一帧 [`CLAIM_LISTING_ENTRIES_PER_FRAME`] 条、接着列到不再截断，按键升序；
    /// 每帧的续扫起点取上一帧最后一条的键。服务说还有却一条都没回（续不下去）回 [`ClientFailure::Protocol`]。
    ///
    /// # Errors
    /// 同 [`Self::list_claims_after`]。
    pub fn list_claims(&mut self, prefix: &[u8]) -> Result<Vec<ListedClaim>, ClientFailure> {
        let mut claims: Vec<ListedClaim> = Vec::new();
        let mut after_key: Vec<u8> = Vec::new();
        loop {
            let frame =
                self.list_claims_after(prefix, &after_key, CLAIM_LISTING_ENTRIES_PER_FRAME)?;
            if let Some(lowest_key_claim) = frame.claims.first() {
                if !after_key.is_empty() && lowest_key_claim.key <= after_key {
                    return Err(ClientFailure::Protocol(format!(
                        "列认领的续扫没往前走：续扫起点之后回的最小的键 {:02x?} 不大于续扫起点 {:02x?}",
                        lowest_key_claim.key, after_key
                    )));
                }
            }
            let Some(highest_key_claim) = frame.claims.last() else {
                if frame.is_truncated {
                    return Err(ClientFailure::Protocol(
                        "列认领说前缀下还有认领，却一条都没回：续不下去".to_string(),
                    ));
                }
                return Ok(claims);
            };
            after_key = highest_key_claim.key.clone();
            claims.extend(frame.claims);
            if !frame.is_truncated {
                return Ok(claims);
            }
        }
    }
}

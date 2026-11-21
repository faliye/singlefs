//! 读、统计、按前缀扫与按前缀删：点读（回应超帧长时折半重读）、统计、扫与带续扫起点的扫、删前缀。

use super::*;

impl ReliableClient {
    pub fn submit_read(
        &mut self,
        key_count: u32,
        capacity_in_bytes: usize,
        fill: impl FnOnce(&mut PointReadFrameBuilder),
    ) -> Result<OperationIdentifier, ClientFailure> {
        self.wait_for_window()?;
        let mut builder = begin_point_read_frame(0, key_count, capacity_in_bytes);
        fill(&mut builder);
        let frame = builder.finish();
        if let Some(failure) = oversize_frame_failure(&frame, &format!("成批点读 {key_count} 个键"))
            .filter(|_| !self.is_enabled(BreakSwitch::OversizeFrameSentAndRetried))
        {
            return Err(failure);
        }
        let operation = self.register(OperationKind::Read, frame);
        self.send_unsent()?;
        Ok(operation)
    }

    /// 成批点读，等回应：一帧问全部键，回应越过帧长上限时拆成几帧（文件头「回应越过帧长上限就拆键」）。读失败回错误，不当成键不在。
    pub fn read(
        &mut self,
        keys: &[(Table, Vec<u8>)],
    ) -> Result<Vec<Option<Vec<u8>>>, ClientFailure> {
        let result = self.read_halving_on_an_oversized_answer(keys);
        match result {
            Err(_) if self.is_enabled(BreakSwitch::ReadFailureReportedAsAbsent) => {
                Ok(vec![None; keys.len()])
            }
            Ok(_) | Err(_) => result,
        }
    }

    /// 一帧问 `keys`；服务答回应越过帧长上限（整帧拒成 `MalformedPayload`）而键不止一个，拆成前后两半各自照这样问（递归，深度不超过键数的
    /// 二进制位数），按原次序接起来。回应多大事先不知道（不分段的值至多一段长，按这个上界切一帧只装得下三四个键），所以先整帧问、被拒再拆。
    /// 别的帧内容不对的拒也是 `MalformedPayload`，同样拆到单键，单键还被拒就把那一个拒照原样交回。
    fn read_halving_on_an_oversized_answer(
        &mut self,
        keys: &[(Table, Vec<u8>)],
    ) -> Result<Vec<Option<Vec<u8>>>, ClientFailure> {
        match self.read_one_frame(keys) {
            Err(ClientFailure::Rejected {
                status: RejectionStatus::MalformedPayload,
                message: _,
            }) if keys.len() > 1 => {
                let (front_half, back_half) = keys.split_at(keys.len() / 2);
                let mut values = self.read_halving_on_an_oversized_answer(front_half)?;
                values.extend(self.read_halving_on_an_oversized_answer(back_half)?);
                Ok(values)
            }
            answered @ (Ok(_) | Err(_)) => answered,
        }
    }

    fn read_one_frame(
        &mut self,
        keys: &[(Table, Vec<u8>)],
    ) -> Result<Vec<Option<Vec<u8>>>, ClientFailure> {
        let key_count = u32::try_from(keys.len())
            .map_err(|_| ClientFailure::Protocol("一帧的键数超过 u32".to_string()))?;
        let capacity = keys
            .iter()
            .map(|(_, key)| POINT_READ_BYTES_PER_KEY + key.len())
            .sum();
        let operation = self.submit_read(key_count, capacity, |builder| {
            for (table, key) in keys {
                builder.key(*table, key);
            }
        })?;
        match self.wait_for(operation)? {
            Completion::Values { values, .. } => Ok(values),
            Completion::Failed { failure, .. } => Err(failure),
            other @ (Completion::WriteAcknowledged { .. }
            | Completion::Claims { .. }
            | Completion::Statistics { .. }
            | Completion::ScanEntries { .. }
            | Completion::PrefixDeleted { .. }
            | Completion::ClaimsReleased { .. }
            | Completion::ClaimListing { .. }
            | Completion::ConclusionRefused { .. }) => Err(ClientFailure::Protocol(format!(
                "成批点读收到的不是值：{other:?}"
            ))),
        }
    }

    pub fn statistics(&mut self) -> Result<Vec<StatisticsCounter>, ClientFailure> {
        self.wait_for_window()?;
        let operation = self.register(OperationKind::Statistics, encode_statistics_frame(0));
        self.send_unsent()?;
        match self.wait_for(operation)? {
            Completion::Statistics { counters, .. } => Ok(counters),
            Completion::Failed { failure, .. } => Err(failure),
            other @ (Completion::WriteAcknowledged { .. }
            | Completion::Values { .. }
            | Completion::Claims { .. }
            | Completion::ScanEntries { .. }
            | Completion::PrefixDeleted { .. }
            | Completion::ClaimsReleased { .. }
            | Completion::ClaimListing { .. }
            | Completion::ConclusionRefused { .. }) => Err(ClientFailure::Protocol(format!(
                "统计收到的不是计数：{other:?}"
            ))),
        }
    }

    /// 按前缀扫一帧，等回应：`table` 里键以 `prefix` 开头的行按键升序、至多 `maximum_entries` 行，交回截没截断与回来的行（删除记号、
    /// 分段存的清单与段照服务里存着的原样交）。忙、断线照文件头「按前缀扫」那一条，与点读同一套。
    ///
    /// # Errors
    /// 帧体超过帧长上限（[`ClientFailure::Protocol`]，没发）；连不上、忙到上限、服务停了（退 75）；被拒、回的不是扫回的行。
    pub fn scan(
        &mut self,
        table: Table,
        prefix: &[u8],
        maximum_entries: u32,
    ) -> Result<(bool, Vec<ScanEntry>), ClientFailure> {
        self.scan_after(table, prefix, &[], maximum_entries)
    }

    /// 按前缀扫一帧、从续扫起点之后起（`protocol.rs` 文件头「按前缀扫」）：`after_key` 空是从前缀起，不空（上一帧回的最后一把键，要以前缀起头）是从排在它之后的
    /// 第一把键起；别的照 [`Self::scan`]。一帧被截断时调用方接着用这一帧的最后一把键续扫，不往下拆前缀（Python 客户端 `raw_scan_stream` 同一条）。
    ///
    /// # Errors
    /// 同 [`Self::scan`]；续扫起点不以前缀起头的服务回状态 1（[`ClientFailure::Rejected`]）。
    pub fn scan_after(
        &mut self,
        table: Table,
        prefix: &[u8],
        after_key: &[u8],
        maximum_entries: u32,
    ) -> Result<(bool, Vec<ScanEntry>), ClientFailure> {
        self.wait_for_window()?;
        let frame = encode_prefix_scan_frame_after(0, table, prefix, maximum_entries, after_key);
        if let Some(failure) = oversize_frame_failure(
            &frame,
            &format!(
                "按前缀扫（前缀 {} 字节、续扫起点 {} 字节）",
                prefix.len(),
                after_key.len()
            ),
        )
        .filter(|_| !self.is_enabled(BreakSwitch::OversizeFrameSentAndRetried))
        {
            return Err(failure);
        }
        let operation = self.register(OperationKind::Scan, frame);
        self.send_unsent()?;
        match self.wait_for(operation)? {
            Completion::ScanEntries {
                is_truncated,
                entries,
                ..
            } => Ok((is_truncated, entries)),
            Completion::Failed { failure, .. } => Err(failure),
            other @ (Completion::WriteAcknowledged { .. }
            | Completion::Values { .. }
            | Completion::Claims { .. }
            | Completion::Statistics { .. }
            | Completion::PrefixDeleted { .. }
            | Completion::ClaimsReleased { .. }
            | Completion::ClaimListing { .. }
            | Completion::ConclusionRefused { .. }) => Err(ClientFailure::Protocol(format!(
                "按前缀扫收到的不是扫回的行：{other:?}"
            ))),
        }
    }

    /// 删前缀一帧，等回应（`protocol.rs` 文件头「删前缀」）：`after_key` 空是从前缀起，不空是从排在它之后的第一把键起；`count_only` 为真只数不删。
    /// 忙、断线与按前缀扫同一套；删不进追加日志，断线重连之后这一帧照样重发（删幂等，重发只会让这一帧报的数少一批）。
    ///
    /// # Errors
    /// 帧体超过帧长上限（[`ClientFailure::Protocol`]，没发）；连不上、忙到上限、服务停了（退 75）；被拒（前缀空、落在记账那一段、续扫起点不以前缀起头）、回的不是删前缀的结局。
    pub fn delete_prefix_frame(
        &mut self,
        prefix: &[u8],
        after_key: &[u8],
        maximum_keys: u32,
        count_only: bool,
    ) -> Result<PrefixDeletionFrame, ClientFailure> {
        self.wait_for_window()?;
        let frame = encode_delete_prefix_frame(0, prefix, after_key, maximum_keys, count_only);
        if let Some(failure) =
            oversize_frame_failure(&frame, &format!("删前缀（前缀 {} 字节）", prefix.len()))
                .filter(|_| !self.is_enabled(BreakSwitch::OversizeFrameSentAndRetried))
        {
            return Err(failure);
        }
        let operation = self.register(OperationKind::DeletePrefix, frame);
        self.send_unsent()?;
        match self.wait_for(operation)? {
            Completion::PrefixDeleted {
                keys_in_frame,
                is_exhausted,
                last_key,
                ..
            } => Ok(PrefixDeletionFrame {
                keys_in_frame,
                is_exhausted,
                last_key,
            }),
            Completion::Failed { failure, .. } => Err(failure),
            other @ (Completion::WriteAcknowledged { .. }
            | Completion::Values { .. }
            | Completion::Claims { .. }
            | Completion::Statistics { .. }
            | Completion::ScanEntries { .. }
            | Completion::ClaimsReleased { .. }
            | Completion::ClaimListing { .. }
            | Completion::ConclusionRefused { .. }) => Err(ClientFailure::Protocol(format!(
                "删前缀收到的不是删前缀的结局：{other:?}"
            ))),
        }
    }

    /// 数或删 `prefix` 下的全部键：一帧一帧接着来（每帧续扫起点取上一帧回的最后一把键），直到前缀下没有更多的键；回数到或删掉的总数。
    /// `count_only` 为真只数不删（干跑）。每帧至多碰 `maximum_keys_per_frame` 把键。
    ///
    /// # Errors
    /// 同 [`Self::delete_prefix_frame`]；服务说前缀下还有键却没回最后一把键（续不下去）回 [`ClientFailure::Protocol`]。
    pub fn delete_prefix(
        &mut self,
        prefix: &[u8],
        maximum_keys_per_frame: u32,
        count_only: bool,
    ) -> Result<u64, ClientFailure> {
        let mut total = 0_u64;
        let mut after_key = Vec::new();
        loop {
            let frame =
                self.delete_prefix_frame(prefix, &after_key, maximum_keys_per_frame, count_only)?;
            total += u64::from(frame.keys_in_frame);
            if frame.is_exhausted {
                return Ok(total);
            }
            if frame.last_key.is_empty() {
                return Err(ClientFailure::Protocol(
                    "删前缀说前缀下还有键，却没回最后一把键：续不下去".to_string(),
                ));
            }
            after_key = frame.last_key;
        }
    }
}

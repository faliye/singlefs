//! 写帧的提交与等确认：普通写、带日志的写、不在才写、带守卫的写结论、GPU 待判行与结论的写，一批条目的提交，轮询与等到全部确认。

use super::*;

impl ReliableClient {
    /// 写一帧：记进追加日志、发出去，不等确认；回操作号与请求号。
    pub fn submit_write(
        &mut self,
        entry_count: u32,
        capacity_in_bytes: usize,
        fill: impl FnOnce(&mut WriteFrameBuilder),
    ) -> Result<(OperationIdentifier, u64), ClientFailure> {
        self.submit_journaled_frame(
            &format!("成批写 {entry_count} 条"),
            |client_identity, request_number| {
                let mut builder = begin_write_frame(
                    0,
                    client_identity,
                    request_number,
                    entry_count,
                    capacity_in_bytes,
                );
                fill(&mut builder);
                builder.finish()
            },
        )
    }

    /// 带请求号的一帧（成批写、不在才写、写结论）：分请求号、编帧、核帧长、记进追加日志、发出去，不等确认；回操作号与请求号。
    /// `build_frame` 拿客户端身份与分到的请求号编整帧（序号字段写 0，发时换）；`what` 说是哪一种帧、几条（帧长超限时报）。
    fn submit_journaled_frame(
        &mut self,
        what: &str,
        build_frame: impl FnOnce(&[u8], u64) -> Vec<u8>,
    ) -> Result<(OperationIdentifier, u64), ClientFailure> {
        self.wait_for_window()?;
        let request_number = self.next_request_number;
        self.next_request_number += 1;
        let frame = build_frame(&self.client_identity, request_number);
        if let Some(failure) =
            oversize_frame_failure(&frame, &format!("{what}（请求号 {request_number}）"))
                .filter(|_| !self.is_enabled(BreakSwitch::OversizeFrameSentAndRetried))
        {
            // 没记日志、没发：这个请求号收回，下一帧照用。
            self.next_request_number = request_number;
            return Err(failure);
        }
        self.activity.enter(ClientActivity::WritingJournal);
        let journaled = self.journal.append_write(request_number, &frame);
        self.activity.enter(ClientActivity::Outside);
        journaled.map_err(ClientFailure::Journal)?;
        let operation = self.register(OperationKind::Write { request_number }, frame);
        self.send_unsent()?;
        Ok((operation, request_number))
    }

    /// 不在才写一帧（`protocol.rs` 文件头「不在才写」）：记进追加日志、发出去，不等确认；回操作号与请求号。
    ///
    /// # Errors
    /// 帧体超过帧长上限（[`ClientFailure::Protocol`]，没记日志、没发）；追加日志写不进；发的时候连不上、服务停了。
    pub fn submit_write_if_absent(
        &mut self,
        entries: &[TableEntry],
    ) -> Result<(OperationIdentifier, u64), ClientFailure> {
        let entry_count = u32::try_from(entries.len())
            .map_err(|_| ClientFailure::Protocol("一帧的条数超过 u32".to_string()))?;
        let capacity: usize = entries
            .iter()
            .map(|(_, key, value)| WRITE_BYTES_PER_ENTRY + key.len() + value.len())
            .sum();
        self.submit_journaled_frame(
            &format!("不在才写 {entry_count} 条"),
            |client_identity, request_number| {
                let mut builder = begin_write_if_absent_frame(
                    0,
                    client_identity,
                    request_number,
                    entry_count,
                    capacity,
                );
                for (table, key, value) in entries {
                    builder.entry(*table, key, value);
                }
                builder.finish()
            },
        )
    }

    /// 不在才写一帧并等确认：键已在的那几条不动，回真写了几条（[`WriteAcknowledgement`]）。
    ///
    /// # Errors
    /// 同 [`Self::submit_write_if_absent`]；等确认时连不上、忙到上限、存储一直出错（退 75），服务拒了这一帧。
    pub fn write_if_absent(
        &mut self,
        entries: &[TableEntry],
    ) -> Result<WriteAcknowledgement, ClientFailure> {
        let (operation, _) = self.submit_write_if_absent(entries)?;
        match self.wait_for(operation)? {
            Completion::WriteAcknowledged {
                entries_written,
                is_already_applied,
                ..
            } => Ok(WriteAcknowledgement {
                entries_written,
                is_already_applied,
            }),
            other @ (Completion::Values { .. }
            | Completion::Claims { .. }
            | Completion::Statistics { .. }
            | Completion::ScanEntries { .. }
            | Completion::PrefixDeleted { .. }
            | Completion::ClaimsReleased { .. }
            | Completion::ClaimListing { .. }
            | Completion::Failed { .. }
            | Completion::ConclusionRefused { .. }) => Err(ClientFailure::Protocol(format!(
                "不在才写收到的不是写确认：{other:?}"
            ))),
        }
    }

    /// 写结论一帧（`protocol.rs` 文件头「写结论」）：记进追加日志、发出去，不等确认；回操作号与请求号。
    /// 守卫不过的那一帧收到状态 7 时从追加日志里勾掉、交 [`Completion::ConclusionRefused`]（文件头「写结论」）。
    ///
    /// # Errors
    /// 持有者身份不在 1 到 255 字节之内（[`ClientFailure::Protocol`]，没记日志、没发）；别的同 [`Self::submit_write_if_absent`]。
    pub fn submit_guarded_conclusion_write(
        &mut self,
        request: &GuardedConclusionWrite,
    ) -> Result<(OperationIdentifier, u64), ClientFailure> {
        let holder_bytes = request.holder_identity.len();
        if holder_bytes == 0 || holder_bytes > MAXIMUM_CLIENT_IDENTITY_BYTES {
            return Err(ClientFailure::Protocol(format!(
                "写结论的持有者身份 {holder_bytes} 字节，要 1 到 {MAXIMUM_CLIENT_IDENTITY_BYTES} 字节；没记日志、没发"
            )));
        }
        self.submit_journaled_frame(
            &format!(
                "写结论 {} 条、删 {} 把",
                request.entries.len(),
                request.deleted_keys.len()
            ),
            |client_identity, request_number| {
                encode_guarded_conclusion_write_frame(0, client_identity, request_number, request)
            },
        )
    }

    /// 写结论一帧并等结局：落下了回 [`GuardedConclusionWriteOutcome::Written`]，守卫不过回 [`GuardedConclusionWriteOutcome::Refused`]
    /// （不是错：调用方照种类定，`protocol.rs` 文件头「回应帧」状态 7）。
    ///
    /// # Errors
    /// 同 [`Self::submit_guarded_conclusion_write`]；等结局时连不上、忙到上限、存储一直出错（退 75），服务拒了这一帧（帧的写法不对）。
    pub fn write_guarded_conclusion(
        &mut self,
        request: &GuardedConclusionWrite,
    ) -> Result<GuardedConclusionWriteOutcome, ClientFailure> {
        let (operation, _) = self.submit_guarded_conclusion_write(request)?;
        match self.wait_for(operation)? {
            Completion::WriteAcknowledged {
                entries_written,
                is_already_applied,
                ..
            } => Ok(GuardedConclusionWriteOutcome::Written(
                WriteAcknowledgement {
                    entries_written,
                    is_already_applied,
                },
            )),
            Completion::ConclusionRefused {
                refusal, message, ..
            } => Ok(GuardedConclusionWriteOutcome::Refused { refusal, message }),
            other @ (Completion::Values { .. }
            | Completion::Claims { .. }
            | Completion::Statistics { .. }
            | Completion::ScanEntries { .. }
            | Completion::PrefixDeleted { .. }
            | Completion::ClaimsReleased { .. }
            | Completion::ClaimListing { .. }
            | Completion::Failed { .. }) => Err(ClientFailure::Protocol(format!(
                "写结论收到的不是写确认或守卫不过：{other:?}"
            ))),
        }
    }

    /// GPU 待判行（`gpu_judging_rows` 文件头「待判行」）照不在才写写进去：同一块的待判行已在就不重写。
    /// `rows` 每条是（流键，块号，待判行）；回真写了几条。
    ///
    /// # Errors
    /// 同 [`Self::write_if_absent`]。
    pub fn write_gpu_pending_rows(
        &mut self,
        rows: &[(Vec<u8>, u64, GpuPendingRow)],
    ) -> Result<WriteAcknowledgement, ClientFailure> {
        let entries: Vec<TableEntry> = rows
            .iter()
            .map(|(stream_key, block_number, row)| {
                (
                    Table::Conclusions,
                    gpu_pending_key(stream_key, *block_number),
                    row.encode(),
                )
            })
            .collect();
        self.write_if_absent(&entries)
    }

    /// 写一块 GPU 判出的结论（`gpu_judging_rows` 文件头「写结论」）：认领键、删的待判行、强制现判的那一行由 [`GpuConclusion::guarded_write`] 拼。
    /// 弄坏开关 rejudged-row-dropped 下拼好之后去掉 `gpu-rejudged/` 那一行。
    ///
    /// # Errors
    /// 结论拼不成（强制现判而既不写守卫那一行、帧里的期待又是「不在」，结论摘要无从取）回 [`ClientFailure::Protocol`]；别的同 [`Self::write_guarded_conclusion`]。
    pub fn write_gpu_conclusion(
        &mut self,
        conclusion: &GpuConclusion,
    ) -> Result<GuardedConclusionWriteOutcome, ClientFailure> {
        let mut request = conclusion
            .guarded_write()
            .map_err(ClientFailure::Protocol)?;
        if self.is_enabled(BreakSwitch::RejudgedRowDropped) {
            request
                .entries
                .retain(|(_, key, _)| !key.starts_with(GPU_REJUDGED_FAMILY_PREFIX));
        }
        self.write_guarded_conclusion(&request)
    }

    pub fn submit_write_entries(
        &mut self,
        entries: &[(Table, Vec<u8>, Vec<u8>)],
    ) -> Result<(OperationIdentifier, u64), ClientFailure> {
        let entry_count = u32::try_from(entries.len())
            .map_err(|_| ClientFailure::Protocol("一帧的条数超过 u32".to_string()))?;
        let capacity = entries
            .iter()
            .map(|(_, key, value)| WRITE_BYTES_PER_ENTRY + key.len() + value.len())
            .sum();
        self.submit_write(entry_count, capacity, |builder| {
            for (table, key, value) in entries {
                builder.entry(*table, key, value);
            }
        })
    }

    /// 收回应，至多等 `maximum_wait`；回这期间做完的帧。写失败与连不上从这里回错。
    pub fn poll(&mut self, maximum_wait: Duration) -> Result<Vec<Completion>, ClientFailure> {
        if self.completions.is_empty() {
            if self.operations.is_empty() {
                // 没有在等的帧：照调用方给的时长睡，调用方拿它当节拍。
                self.activity.enter(ClientActivity::Sleeping);
                thread::sleep(maximum_wait);
                self.activity.enter(ClientActivity::Outside);
            } else {
                self.pump_once(maximum_wait)?;
            }
        }
        Ok(self.completions.drain(..).collect())
    }

    /// 等某一帧做完；别的帧做完的留在队里给 `poll`。
    pub(super) fn wait_for(
        &mut self,
        operation: OperationIdentifier,
    ) -> Result<Completion, ClientFailure> {
        loop {
            if let Some(position) = self
                .completions
                .iter()
                .position(|completion| completion.operation() == operation)
            {
                return Ok(self
                    .completions
                    .remove(position)
                    .expect("position 是上面找到的"));
            }
            self.pump_once(POLL_SLICE)?;
        }
    }

    /// 等全部写帧都确认；写确认从完成队里拿掉，别的留着。
    pub fn wait_all_acknowledged(&mut self) -> Result<(), ClientFailure> {
        while self.outstanding_writes() > 0 {
            self.pump_once(POLL_SLICE)?;
            self.completions
                .retain(|completion| !matches!(completion, Completion::WriteAcknowledged { .. }));
        }
        self.completions
            .retain(|completion| !matches!(completion, Completion::WriteAcknowledged { .. }));
        Ok(())
    }
}

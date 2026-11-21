//! GPU 批处理服务的待判行与强制现判凭据（`1D`、`1E`）。

use super::*;

/// 待判行里的强制现判：服务见 [`Self::Forced`] 一律现判、不读库里已有的结论，写结论那一帧另写一行强制现判凭据（[`GpuRejudgedRow`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ForcedRejudging {
    NotForced,
    Forced,
}

/// 待判行的值（[`VerdictStoreTable::GpuPendingRows`]）：指向录入表那一块（块键），不另抄旋钮与判表。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GpuPendingRow {
    /// 录入表那一块的键：服务判完照它（连同结论种类与 [`Self::conclusion_version`]）写结论块。
    pub recorded_block: VerdictBlockKey,
    /// 录下的流程（`crash_amplification::RecordedFlow`，库里「录下的流程」那张表）的摘要：服务照它读回流、排计划，
    /// 解结论字要的盘序、基镜像的逐盘自证槽数与按发布分的态数都从这条流来。
    pub recorded_flow_digest: [u8; SHA256_DIGEST_BYTES],
    /// 打包判表的摘要（`sha256(PackedJudgeTables::to_bytes())`，与库里存判表的键同一个）：服务照它读判表。
    pub packed_judge_tables_digest: [u8; SHA256_DIGEST_BYTES],
    /// 这一块的状态在判表里的序号区间（点的起点加块在点里的区间）。
    pub state_ordinals: std::ops::Range<u64>,
    /// 结论写在这一把版本下：由结论种类与判器引到的被检测产品代码定，不带判器的代码（服务照行的数据判，不看它是哪一版判器）。
    pub conclusion_version: ConclusionVersion,
    pub forced: ForcedRejudging,
    /// 入池序号：写这一行那一刻的纳秒数（自 UNIX 纪元），服务按它排先后；强制现判凭据的键带它。
    pub pool_entry_number: u64,
}

/// 待判行值的字节数：块键、录下的流程的摘要、判表摘要、态序号区间两端、结论版本键、强制现判、入池序号。
const GPU_PENDING_ROW_VALUE_BYTES: usize = BLOCK_KEY_BYTES
    + SHA256_DIGEST_BYTES
    + SHA256_DIGEST_BYTES
    + 8
    + 8
    + SHA256_DIGEST_BYTES
    + 1
    + 8;

impl GpuPendingRow {
    /// 存进库的字节（头一个字节起就是块键，没有别的头）：块键、录下的流程的摘要、判表摘要、态序号区间的起止（各 8 字节大端）、结论版本键、
    /// 强制现判（`00` 不强制、`01` 强制）、入池序号（8 字节大端）。
    #[must_use]
    pub fn to_value_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(GPU_PENDING_ROW_VALUE_BYTES);
        bytes.extend_from_slice(&self.recorded_block.to_block_key_bytes());
        bytes.extend_from_slice(&self.recorded_flow_digest);
        bytes.extend_from_slice(&self.packed_judge_tables_digest);
        bytes.extend_from_slice(&self.state_ordinals.start.to_be_bytes());
        bytes.extend_from_slice(&self.state_ordinals.end.to_be_bytes());
        bytes.extend_from_slice(&self.conclusion_version.0);
        bytes.push(match self.forced {
            ForcedRejudging::NotForced => 0x00,
            ForcedRejudging::Forced => 0x01,
        });
        bytes.extend_from_slice(&self.pool_entry_number.to_be_bytes());
        bytes
    }

    /// [`Self::to_value_bytes`] 的逆；长度不对、强制现判那个字节认不出交回 `None`。
    #[must_use]
    pub fn from_value_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != GPU_PENDING_ROW_VALUE_BYTES {
            return None;
        }
        let (recorded_block, rest) = VerdictBlockKey::from_block_key_bytes_with_suffix(bytes)?;
        let mut cursor = ByteCursor {
            bytes: rest,
            position: 0,
        };
        let recorded_flow_digest =
            <[u8; SHA256_DIGEST_BYTES]>::try_from(cursor.take(SHA256_DIGEST_BYTES)?).ok()?;
        let packed_judge_tables_digest =
            <[u8; SHA256_DIGEST_BYTES]>::try_from(cursor.take(SHA256_DIGEST_BYTES)?).ok()?;
        let start = cursor.big_endian_u64()?;
        let end = cursor.big_endian_u64()?;
        let conclusion_version =
            <[u8; SHA256_DIGEST_BYTES]>::try_from(cursor.take(SHA256_DIGEST_BYTES)?).ok()?;
        let forced = match cursor.take(1)?.first()? {
            0x00 => ForcedRejudging::NotForced,
            0x01 => ForcedRejudging::Forced,
            _ => return None,
        };
        let pool_entry_number = cursor.big_endian_u64()?;
        cursor.is_at_the_end().then_some(Self {
            recorded_block,
            recorded_flow_digest,
            packed_judge_tables_digest,
            state_ordinals: start..end,
            conclusion_version: ConclusionVersion(conclusion_version),
            forced,
            pool_entry_number,
        })
    }
}

/// 强制现判凭据的值（[`VerdictStoreTable::GpuRejudgedRows`]）：判它的服务持有者身份、它写下的那一份结论的摘要
/// （`crash_amplification::ConclusionSpelledOutInVerdictVectors::conclusion_digest`）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GpuRejudgedRow {
    pub holder_identity: Vec<u8>,
    pub conclusion_digest: [u8; SHA256_DIGEST_BYTES],
}

impl GpuRejudgedRow {
    /// 存进库的字节：持有者身份的字节数（4 字节大端）与身份、结论摘要。
    #[must_use]
    pub fn to_value_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(4 + self.holder_identity.len() + SHA256_DIGEST_BYTES);
        bytes.extend_from_slice(
            &u32::try_from(self.holder_identity.len())
                .expect("持有者身份是进程随机号、卡序号与判线程号，几十个字节")
                .to_be_bytes(),
        );
        bytes.extend_from_slice(&self.holder_identity);
        bytes.extend_from_slice(&self.conclusion_digest);
        bytes
    }

    /// [`Self::to_value_bytes`] 的逆；读不满、末尾多字节交回 `None`。
    #[must_use]
    pub fn from_value_bytes(bytes: &[u8]) -> Option<Self> {
        let mut cursor = ByteCursor { bytes, position: 0 };
        let identity_length = usize::try_from(cursor.big_endian_u32()?).ok()?;
        let holder_identity = cursor.take(identity_length)?.to_vec();
        let conclusion_digest =
            <[u8; SHA256_DIGEST_BYTES]>::try_from(cursor.take(SHA256_DIGEST_BYTES)?).ok()?;
        cursor.is_at_the_end().then_some(Self {
            holder_identity,
            conclusion_digest,
        })
    }
}

/// 写一行待判行的结局（[`VerdictStore::put_the_gpu_pending_row_unless_present_or_raise_it_to_forced`]）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GpuPendingRowPut {
    /// 这一行原来不在，写下的是调用方给的那一份。
    Written,
    /// 这一行已经在（别的趟写的），没重写：交回库里那一份。
    AlreadyPresent(GpuPendingRow),
    /// 这一行已经在而不强制现判，调用方给的这一份强制现判：照调用方给的整份写下（强制、它的入池序号），`replaced` 是库里原来那一份。
    RaisedToForced { replaced: GpuPendingRow },
}

/// 一行待判行在 `key_space` 那一份判定库里的整把键：GPU 批处理服务写结论那一帧删它、照它核删的是自己读到的那一份
/// （真库与判段登记的临时库各连各的客户端，键接上那一份的键空间）。
#[must_use]
pub fn key_bytes_of_the_gpu_pending_row_in(
    key_space: &LibraryKeySpace,
    stream: &GpuPendingStreamKey,
    block_number: u64,
) -> Vec<u8> {
    key_bytes_in_the_key_space(
        key_space,
        &VerdictStoreKey::GpuPendingRow(stream, block_number)
            .to_key_bytes()
            .0,
    )
}

/// 一行待判行在全仓那一份判定库里的整把键（[`LibraryKeySpace::TheOneLibrary`]：键原样，[`key_bytes_of_the_gpu_pending_row_in`]）。
#[must_use]
pub fn key_bytes_of_the_gpu_pending_row_in_the_one_library(
    stream: &GpuPendingStreamKey,
    block_number: u64,
) -> Vec<u8> {
    key_bytes_of_the_gpu_pending_row_in(&LibraryKeySpace::TheOneLibrary, stream, block_number)
}

/// 一行强制现判凭据在 `key_space` 那一份判定库里的整把键（同 [`key_bytes_of_the_gpu_pending_row_in`]）：
/// GPU 批处理服务现判强制现判的块时与结论同一帧写它。
#[must_use]
pub fn key_bytes_of_the_gpu_rejudged_row_in(
    key_space: &LibraryKeySpace,
    pool_entry_number: u64,
    stream: &GpuPendingStreamKey,
    block_number: u64,
) -> Vec<u8> {
    key_bytes_in_the_key_space(
        key_space,
        &VerdictStoreKey::GpuRejudgedRow(pool_entry_number, stream, block_number)
            .to_key_bytes()
            .0,
    )
}

/// 一行强制现判凭据在全仓那一份判定库里的整把键（[`key_bytes_of_the_gpu_rejudged_row_in`]）。
#[must_use]
pub fn key_bytes_of_the_gpu_rejudged_row_in_the_one_library(
    pool_entry_number: u64,
    stream: &GpuPendingStreamKey,
    block_number: u64,
) -> Vec<u8> {
    key_bytes_of_the_gpu_rejudged_row_in(
        &LibraryKeySpace::TheOneLibrary,
        pool_entry_number,
        stream,
        block_number,
    )
}

impl VerdictStore {
    /// 写一行待判行，不在才写（协议「待判行」：判段重录同一块时，行已在就不重写）；行已在而不强制现判、这一份强制现判的，改成强制
    /// （复用哨兵抽中强制现判的那一块撞上别的趟写下的不强制行：照不强制判，服务见库里已判就不判，判段就等不到强制现判凭据）。
    /// 先在服务里认领这一行的键——认到（这把键从没写过）就写、写帧确认时认领随之释放；服务答已有结论（键写过，可能是服务判完删掉之后留的删除记号）
    /// 就读回，在的照上面判：交回库里那一份，或照这一份整份写下改成强制；不在的照写。别的客户端认领着就隔一会再问。
    /// 认领着的那一刻到写下之间别人读不到它；删除记号之后那一次写与改成强制那一次写不经认领（两个判段同时补写同一块时后写的盖先写的，
    /// 两份只差入池序号与强制现判，强制的那一份不会被不强制的盖回去：不强制的那一份只在行不在时写）。
    ///
    /// # Errors
    /// 读写、认领不成；等过 [`PRUNE_SERVER_NUMBER_CLAIM_WAIT_LIMIT`] 还被别人认领着；库里那一行不合布局。
    pub fn put_the_gpu_pending_row_unless_present_or_raise_it_to_forced(
        &self,
        stream: &GpuPendingStreamKey,
        block_number: u64,
        row: &GpuPendingRow,
    ) -> Result<GpuPendingRowPut, VerdictStoreError> {
        let key = VerdictStoreKey::GpuPendingRow(stream, block_number).to_key_bytes();
        let waiting_started = std::time::Instant::now();
        loop {
            match self.service.claim(&key.0, "认领待判行")? {
                ClaimOutcome::Claimed => {
                    self.put_one_row_through_the_batch(&key.0, &row.to_value_bytes())?;
                    return Ok(GpuPendingRowPut::Written);
                }
                ClaimOutcome::AlreadyConcluded => {
                    return match self
                        .read_gpu_pending_rows(&[(*stream, block_number)])?
                        .pop()
                    {
                        Some(Some(present)) => match (present.forced, row.forced) {
                            (ForcedRejudging::NotForced, ForcedRejudging::Forced) => {
                                self.put_one_row_through_the_batch(&key.0, &row.to_value_bytes())?;
                                Ok(GpuPendingRowPut::RaisedToForced { replaced: present })
                            }
                            (ForcedRejudging::NotForced, ForcedRejudging::NotForced)
                            | (ForcedRejudging::Forced, ForcedRejudging::NotForced)
                            | (ForcedRejudging::Forced, ForcedRejudging::Forced) => {
                                Ok(GpuPendingRowPut::AlreadyPresent(present))
                            }
                        },
                        Some(None) | None => {
                            self.put_one_row_through_the_batch(&key.0, &row.to_value_bytes())?;
                            Ok(GpuPendingRowPut::Written)
                        }
                    };
                }
                ClaimOutcome::HeldByOther { .. } => {
                    if waiting_started.elapsed() >= PRUNE_SERVER_NUMBER_CLAIM_WAIT_LIMIT {
                        return Err(self.service.failed(
                            PRUNE_SERVER_UNAVAILABLE_EXIT_CODE,
                            format!(
                                "待判行（块号 {block_number}）被别的客户端认领着，等过 {} 秒还没放",
                                PRUNE_SERVER_NUMBER_CLAIM_WAIT_LIMIT.as_secs()
                            ),
                        ));
                    }
                    std::thread::sleep(PRUNE_SERVER_NUMBER_CLAIM_RETRY_INTERVAL);
                }
            }
        }
    }

    /// 一批待判行（流键与块号），交回的次序与 `rows` 相同；不在（没写过、服务判完删了）是 `None`。成批点读。
    ///
    /// # Errors
    /// 服务读不成；库里的一行不合布局（[`GpuPendingRow::from_value_bytes`] 认不出）。
    pub fn read_gpu_pending_rows(
        &self,
        rows: &[(GpuPendingStreamKey, u64)],
    ) -> Result<Vec<Option<GpuPendingRow>>, VerdictStoreError> {
        let keys: Vec<VerdictStoreKeyBytes> = rows
            .iter()
            .map(|(stream, block_number)| {
                VerdictStoreKey::GpuPendingRow(stream, *block_number).to_key_bytes()
            })
            .collect();
        let key_slices: Vec<&[u8]> = keys.iter().map(|key| key.0.as_slice()).collect();
        self.read_values_kept_in_or_out(&key_slices)?
            .into_iter()
            .zip(&keys)
            .map(|(value, key)| match value {
                None => Ok(None),
                Some(value_bytes) => GpuPendingRow::from_value_bytes(&value_bytes)
                    .map(Some)
                    .ok_or_else(|| {
                        stored_bytes_mismatch(
                            VerdictStoreTable::GpuPendingRows,
                            &key.0,
                            "待判行的值不合布局",
                        )
                    }),
            })
            .collect()
    }

    /// 库里全部待判行（流键、块号、行），按键的次序（同一条流的按块号）：GPU 批处理服务从这里取待判的块。
    ///
    /// # Errors
    /// 服务扫不成；库里的一行键或值不合布局。
    pub fn every_gpu_pending_row(
        &self,
    ) -> Result<Vec<(GpuPendingStreamKey, u64, GpuPendingRow)>, VerdictStoreError> {
        let mut rows = Vec::new();
        self.visit_entries_with_key_prefix_kept_in_or_out(
            VerdictStoreKey::WholeTablePrefix(VerdictStoreTable::GpuPendingRows).to_key_bytes(),
            &mut |(key_bytes, value_bytes)| {
                let not_in_the_layout = || {
                    stored_bytes_mismatch(
                        VerdictStoreTable::GpuPendingRows,
                        &key_bytes,
                        "待判行的键或值不合布局",
                    )
                };
                let after_the_tag = key_bytes
                    .get(KEY_TAG_BYTES..)
                    .ok_or_else(not_in_the_layout)?;
                let (stream_bytes, block_number_bytes) = after_the_tag
                    .split_at_checked(SHA256_DIGEST_BYTES)
                    .ok_or_else(not_in_the_layout)?;
                let stream = GpuPendingStreamKey(
                    <[u8; SHA256_DIGEST_BYTES]>::try_from(stream_bytes)
                        .map_err(|_wrong_length| not_in_the_layout())?,
                );
                let block_number = u64::from_be_bytes(
                    <[u8; 8]>::try_from(block_number_bytes)
                        .map_err(|_wrong_length| not_in_the_layout())?,
                );
                let row =
                    GpuPendingRow::from_value_bytes(&value_bytes).ok_or_else(not_in_the_layout)?;
                rows.push((stream, block_number, row));
                Ok(())
            },
        )?;
        Ok(rows)
    }

    /// 删一行待判行（写删除记号）。产品里由 GPU 批处理服务在写结论的那一帧里删；判段不删。
    ///
    /// # Errors
    /// 服务写不成。
    pub fn delete_the_gpu_pending_row(
        &self,
        stream: &GpuPendingStreamKey,
        block_number: u64,
    ) -> Result<(), VerdictStoreError> {
        let mut batch = LibraryWriteBatch::default();
        batch.delete(
            VerdictStoreKey::GpuPendingRow(stream, block_number)
                .to_key_bytes()
                .0,
        );
        self.write_batch_keeping_the_kept_out_rows_in_this_process(batch)
    }

    /// 写一行强制现判凭据。产品里由 GPU 批处理服务在现判强制现判那一块、写结论的那一帧里写。
    ///
    /// # Errors
    /// 服务写不成。
    pub fn store_the_gpu_rejudged_row(
        &self,
        pool_entry_number: u64,
        stream: &GpuPendingStreamKey,
        block_number: u64,
        row: &GpuRejudgedRow,
    ) -> Result<(), VerdictStoreError> {
        self.put_one_row_through_the_batch(
            &VerdictStoreKey::GpuRejudgedRow(pool_entry_number, stream, block_number)
                .to_key_bytes()
                .0,
            &row.to_value_bytes(),
        )
    }

    /// 读一行强制现判凭据；不在是 `None`。
    ///
    /// # Errors
    /// 服务读不成；库里那一行不合布局。
    pub fn read_gpu_rejudged_row(
        &self,
        pool_entry_number: u64,
        stream: &GpuPendingStreamKey,
        block_number: u64,
    ) -> Result<Option<GpuRejudgedRow>, VerdictStoreError> {
        let key = VerdictStoreKey::GpuRejudgedRow(pool_entry_number, stream, block_number);
        match self.read_one_stored_value(&key)? {
            None => Ok(None),
            Some(value_bytes) => GpuRejudgedRow::from_value_bytes(&value_bytes)
                .map(Some)
                .ok_or_else(|| {
                    stored_bytes_mismatch(
                        VerdictStoreTable::GpuRejudgedRows,
                        &key.to_key_bytes().0,
                        "强制现判凭据的值不合布局",
                    )
                }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 一行写死的待判行：强制现判那个字节与入池序号由调用方给。
    fn a_gpu_pending_row(forced: ForcedRejudging, pool_entry_number: u64) -> GpuPendingRow {
        GpuPendingRow {
            recorded_block: block_key_named("gpu-pending", 64),
            recorded_flow_digest: [0x10; SHA256_DIGEST_BYTES],
            packed_judge_tables_digest: [0x21; SHA256_DIGEST_BYTES],
            state_ordinals: 64..96,
            conclusion_version: ConclusionVersion([0x43; SHA256_DIGEST_BYTES]),
            forced,
            pool_entry_number,
        }
    }

    /// 待判行与强制现判凭据的值照写下的读回原样；待判行的值头一个字节起就是块键，强制现判那个字节认不出、多一个字节都读不出。
    /// 判别力（`crates/mutations.tsv`）：强制现判写成 `00`（读回成不强制），这条红。
    #[test]
    fn gpu_pending_and_rejudged_rows_read_back_as_written() {
        for forced in [ForcedRejudging::NotForced, ForcedRejudging::Forced] {
            let row = a_gpu_pending_row(forced, 1_700_000_000_123_456_789);
            let bytes = row.to_value_bytes();
            assert_eq!(bytes.len(), GPU_PENDING_ROW_VALUE_BYTES, "待判行的值定长");
            assert_eq!(
                GpuPendingRow::from_value_bytes(&bytes),
                Some(row.clone()),
                "{forced:?}：读回原样"
            );
            assert_eq!(
                &bytes[..BLOCK_KEY_BYTES],
                &row.recorded_block.to_block_key_bytes()[..],
                "{forced:?}：值的头一个字节起就是块键"
            );
            let mut one_more_byte = bytes.clone();
            one_more_byte.push(0);
            assert_eq!(
                GpuPendingRow::from_value_bytes(&one_more_byte),
                None,
                "多一个字节读不出"
            );
        }
        let mut unknown_forced = a_gpu_pending_row(ForcedRejudging::Forced, 7).to_value_bytes();
        let forced_position = GPU_PENDING_ROW_VALUE_BYTES - 8 - 1;
        unknown_forced[forced_position] = 0x02;
        assert_eq!(
            GpuPendingRow::from_value_bytes(&unknown_forced),
            None,
            "强制现判那个字节认不出读不出"
        );
        let rejudged = GpuRejudgedRow {
            holder_identity: b"service-7/card-0/thread-3".to_vec(),
            conclusion_digest: [0x54; SHA256_DIGEST_BYTES],
        };
        assert_eq!(
            GpuRejudgedRow::from_value_bytes(&rejudged.to_value_bytes()),
            Some(rejudged),
            "强制现判凭据读回原样"
        );
    }

    /// 待判行不在才写：头一次写下（强制现判的）；同一把键再写一份不同的（入池序号不同、不强制）交回库里那一份、库里不变；
    /// 服务判完删掉之后（删除记号）再写照写；库里全部待判行照键列得出；强制现判凭据写下读得回、别的入池序号读不到。
    /// 判别力（`crates/mutations.tsv`）：行已在时照写不误（交回写下了），这条红。
    #[test]
    fn a_gpu_pending_row_is_put_only_when_it_is_not_there() {
        let library = TestLibrary::new("gpu-pending-row-unless-present");
        let store = library.open();
        let stream = GpuPendingStreamKey([0x65; SHA256_DIGEST_BYTES]);
        let first = a_gpu_pending_row(ForcedRejudging::Forced, 1);
        let second = a_gpu_pending_row(ForcedRejudging::NotForced, 2);
        assert_eq!(
            store
                .put_the_gpu_pending_row_unless_present_or_raise_it_to_forced(&stream, 5, &first)
                .expect("写得进库"),
            GpuPendingRowPut::Written,
            "头一次写下"
        );
        assert_eq!(
            store
                .put_the_gpu_pending_row_unless_present_or_raise_it_to_forced(&stream, 5, &second)
                .expect("写得进库"),
            GpuPendingRowPut::AlreadyPresent(first.clone()),
            "行已在：不重写，交回库里那一份"
        );
        assert_eq!(
            store
                .read_gpu_pending_rows(&[(stream, 5), (stream, 6)])
                .expect("读得了库"),
            vec![Some(first.clone()), None],
            "库里仍是头一份；没写的那一块不在"
        );
        assert_eq!(
            store.every_gpu_pending_row().expect("扫得了库"),
            vec![(stream, 5, first)],
            "库里全部待判行"
        );
        store
            .delete_the_gpu_pending_row(&stream, 5)
            .expect("删得了");
        assert_eq!(
            store
                .read_gpu_pending_rows(&[(stream, 5)])
                .expect("读得了库"),
            vec![None],
            "删掉之后读成不在"
        );
        assert_eq!(
            store
                .put_the_gpu_pending_row_unless_present_or_raise_it_to_forced(&stream, 5, &second)
                .expect("写得进库"),
            GpuPendingRowPut::Written,
            "删掉之后（删除记号）再写照写"
        );
        let rejudged = GpuRejudgedRow {
            holder_identity: b"stand-in".to_vec(),
            conclusion_digest: [0x76; SHA256_DIGEST_BYTES],
        };
        store
            .store_the_gpu_rejudged_row(2, &stream, 5, &rejudged)
            .expect("写得进库");
        assert_eq!(
            store
                .read_gpu_rejudged_row(2, &stream, 5)
                .expect("读得了库"),
            Some(rejudged),
            "强制现判凭据读得回"
        );
        assert_eq!(
            store
                .read_gpu_rejudged_row(3, &stream, 5)
                .expect("读得了库"),
            None,
            "别的入池序号下没有"
        );
    }

    /// 强制现判撞上已有的不强制行：不强制的那一行在库里，再写一份不强制的不重写；写一份强制现判的照它整份写下（强制、它的入池序号）、
    /// 交回原来那一份；之后不强制的、另一份强制的都不再改它（强制的不被盖回不强制）。
    /// 判别力（`crates/mutations.tsv`）：在而不强制时照旧交回库里那一份、不改成强制，这条红。
    #[test]
    fn a_gpu_pending_row_not_forced_is_raised_to_forced_by_a_forced_one_and_never_lowered() {
        let library = TestLibrary::new("gpu-pending-row-raised-to-forced");
        let store = library.open();
        let stream = GpuPendingStreamKey([0x66; SHA256_DIGEST_BYTES]);
        let not_forced = a_gpu_pending_row(ForcedRejudging::NotForced, 1);
        let forced = a_gpu_pending_row(ForcedRejudging::Forced, 2);
        let put = |row: &GpuPendingRow| {
            store
                .put_the_gpu_pending_row_unless_present_or_raise_it_to_forced(&stream, 9, row)
                .expect("写得进库")
        };
        assert_eq!(put(&not_forced), GpuPendingRowPut::Written, "头一次写下");
        assert_eq!(
            put(&a_gpu_pending_row(ForcedRejudging::NotForced, 3)),
            GpuPendingRowPut::AlreadyPresent(not_forced.clone()),
            "不强制撞上不强制：不重写"
        );
        assert_eq!(
            put(&forced),
            GpuPendingRowPut::RaisedToForced {
                replaced: not_forced
            },
            "强制撞上不强制：改成强制，交回原来那一份"
        );
        assert_eq!(
            store
                .read_gpu_pending_rows(&[(stream, 9)])
                .expect("读得了库"),
            vec![Some(forced.clone())],
            "库里是强制现判的那一份（连同它的入池序号）"
        );
        for later in [
            a_gpu_pending_row(ForcedRejudging::NotForced, 4),
            a_gpu_pending_row(ForcedRejudging::Forced, 5),
        ] {
            assert_eq!(
                put(&later),
                GpuPendingRowPut::AlreadyPresent(forced.clone()),
                "强制的那一行之后 {later:?} 不再改它"
            );
        }
    }
}

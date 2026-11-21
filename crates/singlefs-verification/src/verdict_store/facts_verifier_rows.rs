//! 核对表（`08`）与核对待判行（`1F`）：核对块的行与 GPU 批处理服务核完写回的那一帧。

use super::*;

/// 核对待判行的流键：一份录下的流程与它的事实表那一条流（`crash_amplification::facts_verifier_pending_stream_key`）。
/// 块号只在一条流里有意义：同一份事实表、不同的流程是几条流，核对待判行互不相盖。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FactsVerifierPendingStreamKey(pub [u8; SHA256_DIGEST_BYTES]);

/// 核对待判行的值（[`VerdictStoreTable::FactsVerifierPendingRows`]）：指向录入表那一块（块键），事实表与流程只带摘要。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FactsVerifierPendingRow {
    /// 录入表那一块的键：服务核完照它（连同 [`Self::facts_digest`] 拼的核对键）写核对行。
    pub recorded_block: VerdictBlockKey,
    /// 录下的流程（`crash_amplification::RecordedFlow`）的摘要：服务照它读回流、排计划，段展开从这条流来。
    pub recorded_flow_digest: [u8; SHA256_DIGEST_BYTES],
    /// 事实表（`crash_facts::FlowFacts`，库里「事实表」那张表）的摘要：服务照它读回事实表。
    pub facts_digest: [u8; SHA256_DIGEST_BYTES],
    /// 这一块的状态在事实表里的序号区间（点的起点加块在点里的区间）。
    pub state_ordinals: std::ops::Range<u64>,
    /// 入池序号：写这一行那一刻的纳秒数（自 UNIX 纪元），服务按它排先后。
    pub pool_entry_number: u64,
}

/// 核对待判行值的字节数：块键、录下的流程的摘要、事实表摘要、态序号区间两端、入池序号。
const FACTS_VERIFIER_PENDING_ROW_VALUE_BYTES: usize =
    BLOCK_KEY_BYTES + SHA256_DIGEST_BYTES + SHA256_DIGEST_BYTES + 8 + 8 + 8;

impl FactsVerifierPendingRow {
    /// 存进库的字节（头一个字节起就是块键，没有别的头）：块键、录下的流程的摘要、事实表摘要、态序号区间的起止（各 8 字节大端）、
    /// 入池序号（8 字节大端）。
    #[must_use]
    pub fn to_value_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(FACTS_VERIFIER_PENDING_ROW_VALUE_BYTES);
        bytes.extend_from_slice(&self.recorded_block.to_block_key_bytes());
        bytes.extend_from_slice(&self.recorded_flow_digest);
        bytes.extend_from_slice(&self.facts_digest);
        bytes.extend_from_slice(&self.state_ordinals.start.to_be_bytes());
        bytes.extend_from_slice(&self.state_ordinals.end.to_be_bytes());
        bytes.extend_from_slice(&self.pool_entry_number.to_be_bytes());
        bytes
    }

    /// [`Self::to_value_bytes`] 的逆；长度不对交回 `None`。
    #[must_use]
    pub fn from_value_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != FACTS_VERIFIER_PENDING_ROW_VALUE_BYTES {
            return None;
        }
        let (recorded_block, rest) = VerdictBlockKey::from_block_key_bytes_with_suffix(bytes)?;
        let mut cursor = ByteCursor {
            bytes: rest,
            position: 0,
        };
        let recorded_flow_digest =
            <[u8; SHA256_DIGEST_BYTES]>::try_from(cursor.take(SHA256_DIGEST_BYTES)?).ok()?;
        let facts_digest =
            <[u8; SHA256_DIGEST_BYTES]>::try_from(cursor.take(SHA256_DIGEST_BYTES)?).ok()?;
        let start = cursor.big_endian_u64()?;
        let end = cursor.big_endian_u64()?;
        let pool_entry_number = cursor.big_endian_u64()?;
        cursor.is_at_the_end().then_some(Self {
            recorded_block,
            recorded_flow_digest,
            facts_digest,
            state_ordinals: start..end,
            pool_entry_number,
        })
    }
}

/// 写一行核对待判行的结局（[`VerdictStore::put_the_facts_verifier_pending_row_unless_present`]）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FactsVerifierPendingRowPut {
    /// 这一行原来不在，写下的是调用方给的那一份。
    Written,
    /// 这一行已经在（别的趟写的），没重写：交回库里那一份。
    AlreadyPresent(FactsVerifierPendingRow),
}

/// 一行核对待判行在 `key_space` 那一份判定库里的整把键：GPU 批处理服务写核对行那一帧删它（连哪一份库的客户端就写哪一份的键空间）。
#[must_use]
pub fn key_bytes_of_the_facts_verifier_pending_row_in(
    key_space: &LibraryKeySpace,
    stream: &FactsVerifierPendingStreamKey,
    block_number: u64,
) -> Vec<u8> {
    key_bytes_in_the_key_space(
        key_space,
        &VerdictStoreKey::FactsVerifierPendingRow(stream, block_number)
            .to_key_bytes()
            .0,
    )
}

/// 一行核对待判行在全仓那一份判定库里的整把键（[`LibraryKeySpace::TheOneLibrary`]：键原样，[`key_bytes_of_the_facts_verifier_pending_row_in`]）。
#[must_use]
pub fn key_bytes_of_the_facts_verifier_pending_row_in_the_one_library(
    stream: &FactsVerifierPendingStreamKey,
    block_number: u64,
) -> Vec<u8> {
    key_bytes_of_the_facts_verifier_pending_row_in(
        &LibraryKeySpace::TheOneLibrary,
        stream,
        block_number,
    )
}

/// 一块的核对结果在一帧里要写的行（[`VerdictStore::facts_verifier_rows_for_one_frame`]）：GPU 批处理服务把它放进自己的写核对行那一帧，
/// 这一帧原子地落。两个守卫都在写的那一刻核，此刻库里那一行与读到的不同就整帧拒：核对行守卫防盖掉别人刚写的核对行，
/// 核对待判行守卫防删掉判段在服务读完之后补写的那一行。帧里的键都是编它的那一份库接上键空间之后的整把键（服务的客户端原样写）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FactsVerifierRowsForOneFrame {
    /// 要写的核对行（键、值，键是库里的整把键字节）；编之前库里已有逐字节相同的那一行时是 `None`（只删核对待判行）。
    pub verifier_row_to_put: Option<(Vec<u8>, Vec<u8>)>,
    /// 核对行守卫：这一块在这一把核对键下的核对行的键，与编之前读到的值（不在是 `None`）。
    pub verifier_row_key: Vec<u8>,
    pub verifier_row_as_read: Option<Vec<u8>>,
    /// 要删的核对待判行的键，与服务取这一行时读到的值。
    pub pending_row_key_to_delete: Vec<u8>,
    pub pending_row_as_read: Vec<u8>,
}

/// 核对表里的一行：这一块用哪份事实表核的、每个状态的结论位。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerifierBlockRow {
    pub facts_digest: [u8; 32],
    pub bits: Vec<u32>,
}

/// 核对表一行的值：事实表摘要 32 字节，再每个状态一个结论位（u32 小端）。[`VerdictStore::read_verifier_block`] 照它读回。
fn verifier_block_value_bytes(facts_digest: &[u8; 32], bits_of_each_state: &[u32]) -> Vec<u8> {
    let mut value = facts_digest.to_vec();
    for bits in bits_of_each_state {
        value.extend_from_slice(&bits.to_le_bytes());
    }
    value
}

impl VerdictStore {
    /// 写一行核对待判行，不在才写（判段重核同一块时，行已在就不重写）。先在服务里认领这一行的键——认到（这把键从没写过）就写、写帧确认时
    /// 认领随之释放；服务答已有结论（键写过，可能是服务核完删掉之后留的删除记号）就读回，在的交回库里那一份，不在的照写。
    /// 别的客户端认领着就隔一会再问。
    ///
    /// # Errors
    /// 读写、认领不成；等过 [`PRUNE_SERVER_NUMBER_CLAIM_WAIT_LIMIT`] 还被别人认领着；库里那一行不合布局。
    pub fn put_the_facts_verifier_pending_row_unless_present(
        &self,
        stream: &FactsVerifierPendingStreamKey,
        block_number: u64,
        row: &FactsVerifierPendingRow,
    ) -> Result<FactsVerifierPendingRowPut, VerdictStoreError> {
        let key = VerdictStoreKey::FactsVerifierPendingRow(stream, block_number).to_key_bytes();
        let waiting_started = std::time::Instant::now();
        loop {
            match self.service.claim(&key.0, "认领核对待判行")? {
                ClaimOutcome::Claimed => {
                    self.put_one_row_through_the_batch(&key.0, &row.to_value_bytes())?;
                    return Ok(FactsVerifierPendingRowPut::Written);
                }
                ClaimOutcome::AlreadyConcluded => {
                    return match self
                        .read_facts_verifier_pending_rows(&[(*stream, block_number)])?
                        .pop()
                    {
                        Some(Some(present)) => {
                            Ok(FactsVerifierPendingRowPut::AlreadyPresent(present))
                        }
                        Some(None) | None => {
                            self.put_one_row_through_the_batch(&key.0, &row.to_value_bytes())?;
                            Ok(FactsVerifierPendingRowPut::Written)
                        }
                    };
                }
                ClaimOutcome::HeldByOther { .. } => {
                    if waiting_started.elapsed() >= PRUNE_SERVER_NUMBER_CLAIM_WAIT_LIMIT {
                        return Err(self.service.failed(
                            PRUNE_SERVER_UNAVAILABLE_EXIT_CODE,
                            format!(
                                "核对待判行（块号 {block_number}）被别的客户端认领着，等过 {} 秒还没放",
                                PRUNE_SERVER_NUMBER_CLAIM_WAIT_LIMIT.as_secs()
                            ),
                        ));
                    }
                    std::thread::sleep(PRUNE_SERVER_NUMBER_CLAIM_RETRY_INTERVAL);
                }
            }
        }
    }

    /// 一批核对待判行（流键与块号），交回的次序与 `rows` 相同；不在（没写过、服务核完删了）是 `None`。成批点读。
    ///
    /// # Errors
    /// 服务读不成；库里的一行不合布局（[`FactsVerifierPendingRow::from_value_bytes`] 认不出）。
    pub fn read_facts_verifier_pending_rows(
        &self,
        rows: &[(FactsVerifierPendingStreamKey, u64)],
    ) -> Result<Vec<Option<FactsVerifierPendingRow>>, VerdictStoreError> {
        let keys: Vec<VerdictStoreKeyBytes> = rows
            .iter()
            .map(|(stream, block_number)| {
                VerdictStoreKey::FactsVerifierPendingRow(stream, *block_number).to_key_bytes()
            })
            .collect();
        let key_slices: Vec<&[u8]> = keys.iter().map(|key| key.0.as_slice()).collect();
        self.read_values_kept_in_or_out(&key_slices)?
            .into_iter()
            .zip(&keys)
            .map(|(value, key)| match value {
                None => Ok(None),
                Some(value_bytes) => FactsVerifierPendingRow::from_value_bytes(&value_bytes)
                    .map(Some)
                    .ok_or_else(|| {
                        stored_bytes_mismatch(
                            VerdictStoreTable::FactsVerifierPendingRows,
                            &key.0,
                            "核对待判行的值不合布局",
                        )
                    }),
            })
            .collect()
    }

    /// 库里全部核对待判行（流键、块号、行），按键的次序（同一条流的按块号）：GPU 批处理服务从这里取要核的块。
    ///
    /// # Errors
    /// 服务扫不成；库里的一行键或值不合布局。
    pub fn every_facts_verifier_pending_row(
        &self,
    ) -> Result<Vec<(FactsVerifierPendingStreamKey, u64, FactsVerifierPendingRow)>, VerdictStoreError>
    {
        let mut rows = Vec::new();
        self.visit_entries_with_key_prefix_kept_in_or_out(
            VerdictStoreKey::WholeTablePrefix(VerdictStoreTable::FactsVerifierPendingRows)
                .to_key_bytes(),
            &mut |(key_bytes, value_bytes)| {
                let not_in_the_layout = || {
                    stored_bytes_mismatch(
                        VerdictStoreTable::FactsVerifierPendingRows,
                        &key_bytes,
                        "核对待判行的键或值不合布局",
                    )
                };
                let after_the_tag = key_bytes
                    .get(KEY_TAG_BYTES..)
                    .ok_or_else(not_in_the_layout)?;
                let (stream_bytes, block_number_bytes) = after_the_tag
                    .split_at_checked(SHA256_DIGEST_BYTES)
                    .ok_or_else(not_in_the_layout)?;
                let stream = FactsVerifierPendingStreamKey(
                    <[u8; SHA256_DIGEST_BYTES]>::try_from(stream_bytes)
                        .map_err(|_wrong_length| not_in_the_layout())?,
                );
                let block_number = u64::from_be_bytes(
                    <[u8; 8]>::try_from(block_number_bytes)
                        .map_err(|_wrong_length| not_in_the_layout())?,
                );
                let row = FactsVerifierPendingRow::from_value_bytes(&value_bytes)
                    .ok_or_else(not_in_the_layout)?;
                rows.push((stream, block_number, row));
                Ok(())
            },
        )?;
        Ok(rows)
    }

    /// 一块的核对结果（`verifier_digest` 是核对键，`bits_of_each_state` 每个状态一个结论位）连同删核对待判行要在一帧里写的行
    /// （[`FactsVerifierRowsForOneFrame`]）：核对行照 [`Self::store_verifier_block`] 同一种值编，编之前先读这一块在这把核对键下的那一行当守卫，
    /// 读到的与要写的逐字节相同就不写它、只删核对待判行。不写库：GPU 批处理服务拿它在自己的写核对行那一帧里写。
    ///
    /// # Errors
    /// 服务读不成。
    pub fn facts_verifier_rows_for_one_frame(
        &self,
        stream: &FactsVerifierPendingStreamKey,
        block_number: u64,
        pending_row_as_read: &FactsVerifierPendingRow,
        verifier_digest: &[u8; SHA256_DIGEST_BYTES],
        bits_of_each_state: &[u32],
    ) -> Result<FactsVerifierRowsForOneFrame, VerdictStoreError> {
        let verifier_row_key =
            VerdictStoreKey::VerifierBlock(&pending_row_as_read.recorded_block, verifier_digest)
                .to_key_bytes()
                .0;
        let verifier_row_as_read = self
            .read_values_kept_in_or_out(&[verifier_row_key.as_slice()])?
            .pop()
            .expect("点读一把键交回一项");
        let verifier_row_value =
            verifier_block_value_bytes(&pending_row_as_read.facts_digest, bits_of_each_state);
        let is_stored_identically =
            verifier_row_as_read.as_deref() == Some(verifier_row_value.as_slice());
        let verifier_row_key_in_the_service = self.service.namespaced_key(&verifier_row_key);
        let verifier_row_to_put = (!is_stored_identically)
            .then(|| (verifier_row_key_in_the_service.clone(), verifier_row_value));
        Ok(FactsVerifierRowsForOneFrame {
            verifier_row_to_put,
            verifier_row_key: verifier_row_key_in_the_service,
            verifier_row_as_read,
            pending_row_key_to_delete: key_bytes_of_the_facts_verifier_pending_row_in(
                &self.service.key_space,
                stream,
                block_number,
            ),
            pending_row_as_read: pending_row_as_read.to_value_bytes(),
        })
    }

    /// 删一行核对待判行（写删除记号）。产品里由 GPU 批处理服务在写核对行的那一帧里删；判段不删。
    ///
    /// # Errors
    /// 服务写不成。
    pub fn delete_the_facts_verifier_pending_row(
        &self,
        stream: &FactsVerifierPendingStreamKey,
        block_number: u64,
    ) -> Result<(), VerdictStoreError> {
        let mut batch = LibraryWriteBatch::default();
        batch.delete(
            VerdictStoreKey::FactsVerifierPendingRow(stream, block_number)
                .to_key_bytes()
                .0,
        );
        self.write_batch_keeping_the_kept_out_rows_in_this_process(batch)
    }

    /// 单测替 GPU 批处理服务把一帧的核对行写进这个库（写核对行、删核对待判行同一批），不核守卫：服务那一侧的写帧核守卫。
    /// 帧里的键是接上键空间之后的整把键，经这一份写时先去掉键空间那一段（这一份写的时候再接上）。
    ///
    /// # Errors
    /// 服务写不成。
    #[cfg(test)]
    pub(crate) fn write_the_facts_verifier_rows_of_one_frame(
        &self,
        frame: &FactsVerifierRowsForOneFrame,
    ) -> Result<(), VerdictStoreError> {
        let mut batch = LibraryWriteBatch::default();
        if let Some((key, value)) = &frame.verifier_row_to_put {
            batch.put(self.service.key_without_the_key_space(key), value);
        }
        batch.delete(
            self.service
                .key_without_the_key_space(&frame.pending_row_key_to_delete),
        );
        self.write_batch_keeping_the_kept_out_rows_in_this_process(batch)
    }

    /// 核对表写一块（第 ② 段）：这一版核对代码在这一块每个状态的结论位。同一键再写就是同一份结论，照写。
    ///
    /// # Errors
    /// 服务写不成。
    pub fn store_verifier_block(
        &mut self,
        block_key: &VerdictBlockKey,
        verifier_digest: &[u8; 32],
        facts_digest: &[u8; 32],
        bits_of_each_state: &[u32],
    ) -> Result<(), VerdictStoreError> {
        let key_bytes = VerdictStoreKey::VerifierBlock(block_key, verifier_digest).to_key_bytes();
        let mut batch = LibraryWriteBatch::default();
        batch.put(
            &key_bytes.0,
            verifier_block_value_bytes(facts_digest, bits_of_each_state),
        );
        self.write_batch_keeping_the_kept_out_rows_in_this_process(batch)
    }

    /// 读核对表的一块；这一版核对代码没核过这一块交回 None。
    ///
    /// # Errors
    /// 服务读不成；值的长度对不上。
    pub fn read_verifier_block(
        &self,
        block_key: &VerdictBlockKey,
        verifier_digest: &[u8; 32],
    ) -> Result<Option<VerifierBlockRow>, VerdictStoreError> {
        let key_bytes = VerdictStoreKey::VerifierBlock(block_key, verifier_digest).to_key_bytes();
        let Some(value) = self.read_value_kept_in_or_out(&key_bytes.0)? else {
            return Ok(None);
        };
        if value.len() < 32 || !(value.len() - 32).is_multiple_of(4) {
            return Err(stored_bytes_mismatch(
                VerdictStoreTable::VerifierVerdicts,
                &value,
                "值要是 32 字节事实摘要加每状态 4 字节",
            ));
        }
        let mut facts_digest = [0u8; 32];
        facts_digest.copy_from_slice(&value[..32]);
        let (chunks, _remainder) = value[32..].as_chunks::<4>();
        let bits = chunks
            .iter()
            .map(|chunk| u32::from_le_bytes(*chunk))
            .collect();
        Ok(Some(VerifierBlockRow { facts_digest, bits }))
    }

    /// 核对表里这一块的每一把核对键（块键之后那 32 字节：核对那一组判法单元指纹与事实表摘要的摘要），按键序。
    ///
    /// # Errors
    /// 服务读不成；键字节认不回来。
    pub fn verifier_row_keys_of_block(
        &self,
        block_key: &VerdictBlockKey,
    ) -> Result<Vec<[u8; SHA256_DIGEST_BYTES]>, VerdictStoreError> {
        let prefix =
            VerdictStoreKey::RowsOfABlockPrefix(VerdictStoreTable::VerifierVerdicts, block_key)
                .to_key_bytes();
        let prefix_length = prefix.0.len();
        let mut found = Vec::new();
        for entry in self.entries_with_key_prefix_kept_in_or_out(prefix) {
            let (key_bytes, _value_bytes) = entry?;
            found.push(
                <[u8; SHA256_DIGEST_BYTES]>::try_from(&key_bytes[prefix_length..]).map_err(
                    |_| {
                        stored_bytes_mismatch(
                            VerdictStoreTable::VerifierVerdicts,
                            &key_bytes,
                            "核对表的键在块键之后要正好是 32 字节",
                        )
                    },
                )?,
            );
        }
        Ok(found)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 一行写死的核对待判行：入池序号由调用方给。
    fn a_facts_verifier_pending_row(pool_entry_number: u64) -> FactsVerifierPendingRow {
        FactsVerifierPendingRow {
            recorded_block: block_key_named("facts-verifier-pending", 64),
            recorded_flow_digest: [0x11; SHA256_DIGEST_BYTES],
            facts_digest: [0x22; SHA256_DIGEST_BYTES],
            state_ordinals: 64..96,
            pool_entry_number,
        }
    }

    /// 核对待判行的值照写下的读回原样、头一个字节起就是块键；多一个字节、少一个字节都读不出。
    /// 判别力（`crates/mutations.tsv`）：写值时录下的流程的摘要与事实表摘要次序对调，这条红。
    #[test]
    fn facts_verifier_pending_rows_read_back_as_written() {
        let row = a_facts_verifier_pending_row(1_700_000_000_123_456_789);
        let bytes = row.to_value_bytes();
        assert_eq!(
            bytes.len(),
            FACTS_VERIFIER_PENDING_ROW_VALUE_BYTES,
            "核对待判行的值定长"
        );
        assert_eq!(
            FactsVerifierPendingRow::from_value_bytes(&bytes),
            Some(row.clone()),
            "读回原样"
        );
        assert_eq!(
            &bytes[..BLOCK_KEY_BYTES],
            &row.recorded_block.to_block_key_bytes()[..],
            "值的头一个字节起就是块键"
        );
        let mut one_more_byte = bytes.clone();
        one_more_byte.push(0);
        assert_eq!(
            FactsVerifierPendingRow::from_value_bytes(&one_more_byte),
            None,
            "多一个字节读不出"
        );
        assert_eq!(
            FactsVerifierPendingRow::from_value_bytes(&bytes[..bytes.len() - 1]),
            None,
            "少一个字节读不出"
        );
    }

    /// 核对待判行不在才写：头一次写下；同一把键再写一份不同的（入池序号不同）交回库里那一份、库里不变；库里全部核对待判行照键列得出；
    /// 服务那一帧（核对行连同删核对待判行）写下之后读成不在、核对行读得回；之后（删除记号）再写照写。
    /// 判别力（`crates/mutations.tsv`）：行已在时照写不误（交回写下了），这条红。
    #[test]
    fn a_facts_verifier_pending_row_is_put_only_when_it_is_not_there() {
        let library = TestLibrary::new("facts-verifier-pending-row-unless-present");
        let store = library.open();
        let stream = FactsVerifierPendingStreamKey([0x67; SHA256_DIGEST_BYTES]);
        let first = a_facts_verifier_pending_row(1);
        let second = a_facts_verifier_pending_row(2);
        assert_eq!(
            store
                .put_the_facts_verifier_pending_row_unless_present(&stream, 5, &first)
                .expect("写得进库"),
            FactsVerifierPendingRowPut::Written,
            "头一次写下"
        );
        assert_eq!(
            store
                .put_the_facts_verifier_pending_row_unless_present(&stream, 5, &second)
                .expect("写得进库"),
            FactsVerifierPendingRowPut::AlreadyPresent(first.clone()),
            "行已在：不重写，交回库里那一份"
        );
        assert_eq!(
            store
                .read_facts_verifier_pending_rows(&[(stream, 5), (stream, 6)])
                .expect("读得了库"),
            vec![Some(first.clone()), None],
            "库里仍是头一份；没写的那一块不在"
        );
        assert_eq!(
            store.every_facts_verifier_pending_row().expect("扫得了库"),
            vec![(stream, 5, first.clone())],
            "库里全部核对待判行"
        );
        let verifier_digest = [0x78; SHA256_DIGEST_BYTES];
        let frame = store
            .facts_verifier_rows_for_one_frame(&stream, 5, &first, &verifier_digest, &[0, 3])
            .expect("读得了库");
        store
            .write_the_facts_verifier_rows_of_one_frame(&frame)
            .expect("写得进库");
        assert_eq!(
            store
                .read_facts_verifier_pending_rows(&[(stream, 5)])
                .expect("读得了库"),
            vec![None],
            "那一帧删掉了核对待判行"
        );
        assert_eq!(
            store
                .read_verifier_block(&first.recorded_block, &verifier_digest)
                .expect("读得了库"),
            Some(VerifierBlockRow {
                facts_digest: first.facts_digest,
                bits: vec![0, 3],
            }),
            "那一帧写下了核对行"
        );
        assert_eq!(
            store
                .put_the_facts_verifier_pending_row_unless_present(&stream, 5, &second)
                .expect("写得进库"),
            FactsVerifierPendingRowPut::Written,
            "删掉之后（删除记号）再写照写"
        );
    }

    /// 服务那一帧的两个守卫：核对行不在时守卫读到 `None`、要写核对行；同一把核对键下已存着逐字节相同的那一行时守卫读到它、不再写核对行
    /// （只删核对待判行）；结论位不同照写。核对待判行的守卫是服务读到的那一份的字节，删的是这一行的键。
    /// 判别力（`crates/mutations.tsv`）：库里已存着逐字节相同的核对行也照写，这条红。
    #[test]
    fn a_facts_verifier_frame_writes_no_verifier_row_already_stored_identically() {
        let library = TestLibrary::new("facts-verifier-frame-guards");
        let store = library.open();
        let stream = FactsVerifierPendingStreamKey([0x68; SHA256_DIGEST_BYTES]);
        let row = a_facts_verifier_pending_row(9);
        let verifier_digest = [0x79; SHA256_DIGEST_BYTES];
        let pending_row_key =
            key_bytes_of_the_facts_verifier_pending_row_in(store.key_space(), &stream, 5);
        let on_an_empty_library = store
            .facts_verifier_rows_for_one_frame(&stream, 5, &row, &verifier_digest, &[0, 1])
            .expect("读得了库");
        assert_eq!(
            (
                on_an_empty_library.verifier_row_as_read.clone(),
                on_an_empty_library.verifier_row_to_put.is_some(),
                on_an_empty_library.pending_row_key_to_delete.clone(),
                on_an_empty_library.pending_row_as_read.clone(),
            ),
            (None, true, pending_row_key.clone(), row.to_value_bytes()),
            "核对行不在：守卫读到不在、要写核对行；删的是这一行，守卫是服务读到的那一份"
        );
        store
            .write_the_facts_verifier_rows_of_one_frame(&on_an_empty_library)
            .expect("写得进库");
        let stored_identically = store
            .facts_verifier_rows_for_one_frame(&stream, 5, &row, &verifier_digest, &[0, 1])
            .expect("读得了库");
        assert_eq!(
            (
                stored_identically.verifier_row_as_read.is_some(),
                stored_identically.verifier_row_to_put.clone(),
                stored_identically.pending_row_key_to_delete,
            ),
            (true, None, pending_row_key),
            "已存着逐字节相同的核对行：守卫读到它，不再写核对行，只删核对待判行"
        );
        let with_other_bits = store
            .facts_verifier_rows_for_one_frame(&stream, 5, &row, &verifier_digest, &[2, 1])
            .expect("读得了库");
        assert!(
            with_other_bits.verifier_row_to_put.is_some(),
            "结论位不同：照写核对行"
        );
    }

    /// 场景：一次性库（临时服务里的一段名字空间）上编一块核对结果的那一帧（GPU 批处理服务照判段的登记连过来、用它的客户端原样写）。
    /// 预期：帧里每一把键（要写的核对行、核对行守卫、要删的核对待判行）都是那一段名字空间接上库里的键：以这一份的键空间起头
    /// （头一个字节是一次性库的那一个），去掉名字空间之后就是全仓那一份的键原样。
    /// 判别力（`crates/mutations.tsv`）：核对行的键不接名字空间、删的核对待判行取全仓那一份的键，各让这一条红。
    #[test]
    fn facts_verifier_frame_keys_of_a_disposable_library_carry_its_namespace() {
        let library = TestLibrary::new("facts-verifier-frame-keys-namespaced");
        let store = library.open();
        let namespace = store.key_space().key_prefix().to_vec();
        assert_eq!(
            namespace.first(),
            Some(&DISPOSABLE_LIBRARY_KEY_SPACE_LEADING_BYTE),
            "前提：一次性库的键空间以一次性库那一个字节起头"
        );
        let stream = FactsVerifierPendingStreamKey([0x69; SHA256_DIGEST_BYTES]);
        let row = a_facts_verifier_pending_row(11);
        let verifier_digest = [0x7A; SHA256_DIGEST_BYTES];
        let frame = store
            .facts_verifier_rows_for_one_frame(&stream, 5, &row, &verifier_digest, &[0, 1])
            .expect("读得了库");
        let verifier_row_key_in_the_one_library =
            VerdictStoreKey::VerifierBlock(&row.recorded_block, &verifier_digest)
                .to_key_bytes()
                .0;
        let mut verifier_row_key_in_the_namespace = namespace.clone();
        verifier_row_key_in_the_namespace.extend_from_slice(&verifier_row_key_in_the_one_library);
        let mut pending_row_key_in_the_namespace = namespace;
        pending_row_key_in_the_namespace.extend_from_slice(
            &key_bytes_of_the_facts_verifier_pending_row_in_the_one_library(&stream, 5),
        );
        assert_eq!(
            (
                frame.verifier_row_key.clone(),
                frame
                    .verifier_row_to_put
                    .as_ref()
                    .map(|(key, _value)| key.clone()),
                frame.pending_row_key_to_delete.clone(),
            ),
            (
                verifier_row_key_in_the_namespace.clone(),
                Some(verifier_row_key_in_the_namespace),
                pending_row_key_in_the_namespace,
            ),
            "核对行守卫、要写的核对行、要删的核对待判行都接上这一份的名字空间"
        );
    }
}

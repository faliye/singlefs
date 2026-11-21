//! 判表待判行与判表结论字（`20`、`21`）：打包判表交 GPU 批处理服务判、结论字写回的那一帧。

use super::*;

/// 判表待判行与判表结论字两张表（[`VerdictStoreTable::PackedTablesPendingRows`]、[`VerdictStoreTable::PackedTablesVerdictWords`]）的键里
/// 表标签之后的两段，都是判的输入数据：打包判表的摘要（`sha256(PackedJudgeTables::to_bytes())`，与库里存判表的键同一个）、
/// 三棵容器树的旋钮编成三栏之后的摘要（`crash_amplification::container_capacity_knobs_digest`）。内核源码与 GPU 判器的代码不进键：
/// 服务照行的数据判，判法改了对不对靠对拍与复用哨兵抓。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PackedTablesKey {
    pub packed_judge_tables_digest: [u8; SHA256_DIGEST_BYTES],
    pub container_capacity_knobs_digest: [u8; SHA256_DIGEST_BYTES],
}

/// [`PackedTablesKey`] 在键里表标签之后占的字节数：两段各 32 字节。
pub const PACKED_TABLES_KEY_BYTES_AFTER_THE_TAG: usize = 2 * SHA256_DIGEST_BYTES;

impl PackedTablesKey {
    /// 键里表标签之后的字节：判表摘要、旋钮摘要，次序就是这两样。
    #[must_use]
    pub fn to_bytes_after_the_tag(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(PACKED_TABLES_KEY_BYTES_AFTER_THE_TAG);
        bytes.extend_from_slice(&self.packed_judge_tables_digest);
        bytes.extend_from_slice(&self.container_capacity_knobs_digest);
        bytes
    }

    /// [`Self::to_bytes_after_the_tag`] 的逆；不是恰好 [`PACKED_TABLES_KEY_BYTES_AFTER_THE_TAG`] 个字节交回 `None`。
    #[must_use]
    pub fn from_bytes_after_the_tag(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != PACKED_TABLES_KEY_BYTES_AFTER_THE_TAG {
            return None;
        }
        let mut cursor = ByteCursor { bytes, position: 0 };
        let packed_judge_tables_digest =
            <[u8; SHA256_DIGEST_BYTES]>::try_from(cursor.take(SHA256_DIGEST_BYTES)?).ok()?;
        let container_capacity_knobs_digest =
            <[u8; SHA256_DIGEST_BYTES]>::try_from(cursor.take(SHA256_DIGEST_BYTES)?).ok()?;
        Some(Self {
            packed_judge_tables_digest,
            container_capacity_knobs_digest,
        })
    }
}

/// 判表待判行里一栏容器容量旋钮的字节数：有无一个字节（`00` 没装旋钮、`01` 装了）、每片记录数 8 字节大端（没装旋钮写 0）。
const CONTAINER_CAPACITY_KNOB_COLUMN_BYTES: usize = 1 + 8;

/// 判表待判行里三栏容器容量旋钮（墓碑、谱系、退役的次序）合起来的字节数。
pub const CONTAINER_CAPACITY_KNOB_COLUMNS_BYTES: usize = 3 * CONTAINER_CAPACITY_KNOB_COLUMN_BYTES;

/// 三棵容器树的旋钮照判表待判行的写法编成三栏（墓碑、谱系、退役的次序，[`CONTAINER_CAPACITY_KNOB_COLUMN_BYTES`] 一栏）：
/// 服务照读回的那一份取内核源码（`JudgeKernelSources::of_the_judge_given_the_container_capacity_knobs`）。
#[must_use]
pub fn container_capacity_knob_columns_bytes(
    knobs: ContainerCapacityKnobs,
) -> [u8; CONTAINER_CAPACITY_KNOB_COLUMNS_BYTES] {
    let mut bytes = [0u8; CONTAINER_CAPACITY_KNOB_COLUMNS_BYTES];
    let knobs_of_the_columns = [
        knobs
            .tombstone
            .map(TombstoneContainerCapacityKnob::records_per_container),
        knobs
            .lineage
            .map(LineageContainerCapacityKnob::records_per_container),
        knobs
            .retirement
            .map(RetirementContainerCapacityKnob::records_per_container),
    ];
    let (columns, no_bytes_left_over) =
        bytes.as_chunks_mut::<CONTAINER_CAPACITY_KNOB_COLUMN_BYTES>();
    assert!(
        no_bytes_left_over.is_empty(),
        "三栏合起来的字节数是一栏的整三倍"
    );
    for (column, records_per_container) in columns.iter_mut().zip(knobs_of_the_columns) {
        if let Some(records_per_container) = records_per_container {
            column[0] = 0x01;
            column[1..].copy_from_slice(&records_per_container.to_be_bytes());
        }
    }
    bytes
}

/// [`container_capacity_knob_columns_bytes`] 的逆；不是恰好 [`CONTAINER_CAPACITY_KNOB_COLUMNS_BYTES`] 个字节、有无那一个字节不是 `00` 或 `01`、
/// 没装旋钮的那一栏值不是 0、装了的那一栏越出它那一棵树的取值范围，交回 `None`。
#[must_use]
pub fn container_capacity_knobs_of_the_columns(bytes: &[u8]) -> Option<ContainerCapacityKnobs> {
    if bytes.len() != CONTAINER_CAPACITY_KNOB_COLUMNS_BYTES {
        return None;
    }
    let mut records_per_container_of_each_column =
        [None; CONTAINER_CAPACITY_KNOB_COLUMNS_BYTES / CONTAINER_CAPACITY_KNOB_COLUMN_BYTES];
    let (columns, no_bytes_left_over) = bytes.as_chunks::<CONTAINER_CAPACITY_KNOB_COLUMN_BYTES>();
    assert!(
        no_bytes_left_over.is_empty(),
        "上面核过恰好三栏的字节数，切成整栏不剩"
    );
    for (records_per_container, column) in
        records_per_container_of_each_column.iter_mut().zip(columns)
    {
        let value = u64::from_be_bytes(<[u8; 8]>::try_from(&column[1..]).ok()?);
        *records_per_container = match (column[0], value) {
            (0x00, 0) => None,
            (0x01, records) => Some(records),
            (_, _) => return None,
        };
    }
    let [tombstone, lineage, retirement] = records_per_container_of_each_column;
    Some(ContainerCapacityKnobs {
        tombstone: match tombstone {
            None => None,
            Some(records) => Some(TombstoneContainerCapacityKnob::new(records)?),
        },
        lineage: match lineage {
            None => None,
            Some(records) => Some(LineageContainerCapacityKnob::new(records)?),
        },
        retirement: match retirement {
            None => None,
            Some(records) => Some(RetirementContainerCapacityKnob::new(records)?),
        },
    })
}

/// 判表待判行的值（[`VerdictStoreTable::PackedTablesPendingRows`]）：判表与内核源码只带摘要（在键里），行里是服务照它取内核源码的旋钮栏与核判表用的态数。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PackedTablesPendingRow {
    /// 三棵容器树的旋钮（服务照它取内核源码：`JudgeKernelSources::of_the_judge_given_the_container_capacity_knobs`；
    /// 观察者交回的盘那一种恒写产品那一份 `ContainerCapacityKnobs::PRODUCT`，取出的就是判段上卡那一处编的 `of_the_judge()` 那三份）。
    pub container_capacity_knobs: ContainerCapacityKnobs,
    /// 判表的态数（`PackedJudgeTables::state_count`）：服务读回判表后核，不等就拒这一行。
    pub state_count: u64,
    pub forced: ForcedRejudging,
    /// 入池序号：写这一行那一刻的纳秒数（自 UNIX 纪元），服务按它排先后。
    pub pool_entry_number: u64,
}

/// 判表待判行值的字节数：三栏旋钮、态数、强制现判、入池序号。
const PACKED_TABLES_PENDING_ROW_VALUE_BYTES: usize =
    CONTAINER_CAPACITY_KNOB_COLUMNS_BYTES + 8 + 1 + 8;

impl PackedTablesPendingRow {
    /// 存进库的字节：三栏旋钮（[`container_capacity_knob_columns_bytes`]）、态数（8 字节大端）、强制现判（0 不强制、1 强制）、
    /// 入池序号（8 字节大端）。
    #[must_use]
    pub fn to_value_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(PACKED_TABLES_PENDING_ROW_VALUE_BYTES);
        bytes.extend_from_slice(&container_capacity_knob_columns_bytes(
            self.container_capacity_knobs,
        ));
        bytes.extend_from_slice(&self.state_count.to_be_bytes());
        bytes.push(match self.forced {
            ForcedRejudging::NotForced => 0,
            ForcedRejudging::Forced => 1,
        });
        bytes.extend_from_slice(&self.pool_entry_number.to_be_bytes());
        bytes
    }

    /// [`Self::to_value_bytes`] 的逆；长度不对、旋钮栏读不出（[`container_capacity_knobs_of_the_columns`]）、强制现判那个字节认不出交回 `None`。
    #[must_use]
    pub fn from_value_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != PACKED_TABLES_PENDING_ROW_VALUE_BYTES {
            return None;
        }
        let mut cursor = ByteCursor { bytes, position: 0 };
        let container_capacity_knobs = container_capacity_knobs_of_the_columns(
            cursor.take(CONTAINER_CAPACITY_KNOB_COLUMNS_BYTES)?,
        )?;
        let state_count = cursor.big_endian_u64()?;
        let forced = match cursor.take(1)?.first()? {
            0 => ForcedRejudging::NotForced,
            1 => ForcedRejudging::Forced,
            _ => return None,
        };
        let pool_entry_number = cursor.big_endian_u64()?;
        cursor.is_at_the_end().then_some(Self {
            container_capacity_knobs,
            state_count,
            forced,
            pool_entry_number,
        })
    }
}

/// 写一行判表待判行的结局（[`VerdictStore::put_the_packed_tables_pending_row_unless_present_or_raise_it_to_forced`]）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PackedTablesPendingRowPut {
    /// 这一行原来不在，写下的是调用方给的那一份。
    Written,
    /// 这一行已经在（别的趟写的），没重写：交回库里那一份。
    AlreadyPresent(PackedTablesPendingRow),
    /// 这一行已经在而不强制现判，调用方给的这一份强制现判：照调用方给的整份写下，`replaced` 是库里原来那一份。
    RaisedToForced { replaced: PackedTablesPendingRow },
}

/// 一行判表待判行在 `key_space` 那一份判定库里的整把键：GPU 批处理服务写判表结论字那一帧删它。
#[must_use]
pub fn key_bytes_of_the_packed_tables_pending_row_in(
    key_space: &LibraryKeySpace,
    key: &PackedTablesKey,
) -> Vec<u8> {
    key_bytes_in_the_key_space(
        key_space,
        &VerdictStoreKey::PackedTablesPendingRow(key)
            .to_key_bytes()
            .0,
    )
}

/// 一行判表待判行在全仓那一份判定库里的整把键（[`LibraryKeySpace::TheOneLibrary`]：键原样，[`key_bytes_of_the_packed_tables_pending_row_in`]）。
#[must_use]
pub fn key_bytes_of_the_packed_tables_pending_row_in_the_one_library(
    key: &PackedTablesKey,
) -> Vec<u8> {
    key_bytes_of_the_packed_tables_pending_row_in(&LibraryKeySpace::TheOneLibrary, key)
}

/// 一行判表结论字在 `key_space` 那一份判定库里的整把键（同 [`key_bytes_of_the_packed_tables_pending_row_in`]）。
#[must_use]
pub fn key_bytes_of_the_packed_tables_verdict_words_in(
    key_space: &LibraryKeySpace,
    key: &PackedTablesKey,
) -> Vec<u8> {
    key_bytes_in_the_key_space(
        key_space,
        &VerdictStoreKey::PackedTablesVerdictWords(key)
            .to_key_bytes()
            .0,
    )
}

/// 一行判表结论字在全仓那一份判定库里的整把键（[`key_bytes_of_the_packed_tables_verdict_words_in`]）。
#[must_use]
pub fn key_bytes_of_the_packed_tables_verdict_words_in_the_one_library(
    key: &PackedTablesKey,
) -> Vec<u8> {
    key_bytes_of_the_packed_tables_verdict_words_in(&LibraryKeySpace::TheOneLibrary, key)
}

/// 一行判表结论字的值里结论字之外的那一截（设备属性：卡型与驱动版本各几十个字节、GPU 判器组键 32 字节）在一段
/// （`prune_server::segmented_values::SEGMENT_BYTES`）里留的余量，远大于它。
#[cfg(feature = "gpu")]
const PACKED_TABLES_VERDICT_WORDS_ROW_ALLOWANCE_BYTES: u64 = 64 * 1024;

/// 一张判表的结论字一帧装得下的最多态数：GPU 批处理服务写结论那一帧不分段，一行的值不长过一段
/// （`prune_server::segmented_values::SEGMENT_BYTES`）；去掉余量（[`PACKED_TABLES_VERDICT_WORDS_ROW_ALLOWANCE_BYTES`]）、态数那 8 字节，
/// 再除以一个态的结论字字节数。一行判表结论字是一张判表的全部状态、不切行：判段写判表待判行之前照它拒更大的表。
#[cfg(feature = "gpu")]
pub const MOST_STATES_OF_A_PACKED_TABLE_IN_ONE_VERDICT_WORDS_FRAME: u64 =
    (prune_server::segmented_values::SEGMENT_BYTES as u64
        - PACKED_TABLES_VERDICT_WORDS_ROW_ALLOWANCE_BYTES
        - 8)
        / (crate::crash_judge_gpu::VERDICT_WORDS as u64 * 4);

/// 判表结论字的值（[`VerdictStoreTable::PackedTablesVerdictWords`]）：一张判表全部状态的 GPU 结论字（态序号 0 起连着）与写它的设备。
#[cfg(feature = "gpu")]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackedTablesVerdictWordsRow {
    pub verdict_words: Vec<[u32; crate::crash_judge_gpu::VERDICT_WORDS]>,
    pub judged_on: JudgingDevice,
}

#[cfg(feature = "gpu")]
impl PackedTablesVerdictWordsRow {
    /// 存进库的字节：态数（8 字节小端）、逐态 `VERDICT_WORDS` 个字（各 4 字节小端）、设备属性（[`JudgingDevice::to_attribute_bytes`]，接到末尾、不带长度）。
    #[must_use]
    pub fn to_value_bytes(&self) -> Vec<u8> {
        let device = self.judged_on.to_attribute_bytes();
        let mut bytes = Vec::with_capacity(
            8 + self.verdict_words.len() * crate::crash_judge_gpu::VERDICT_WORDS * 4 + device.len(),
        );
        bytes.extend_from_slice(
            &u64::try_from(self.verdict_words.len())
                .expect("一张判表的态数装得进 u64")
                .to_le_bytes(),
        );
        for state_words in &self.verdict_words {
            for word in state_words {
                bytes.extend_from_slice(&word.to_le_bytes());
            }
        }
        bytes.extend_from_slice(&device);
        bytes
    }

    /// [`Self::to_value_bytes`] 的逆；态数那 8 字节读不满、结论字不够态数那么多、设备属性认不出（含末尾多字节）交回 `None`。
    #[must_use]
    pub fn from_value_bytes(bytes: &[u8]) -> Option<Self> {
        let mut cursor = ByteCursor { bytes, position: 0 };
        let state_count = usize::try_from(u64::from_le_bytes(
            <[u8; 8]>::try_from(cursor.take(8)?).ok()?,
        ))
        .ok()?;
        let words_bytes =
            cursor.take(state_count.checked_mul(crate::crash_judge_gpu::VERDICT_WORDS * 4)?)?;
        let (bytes_of_each_state, no_bytes_left_over) =
            words_bytes.as_chunks::<{ crate::crash_judge_gpu::VERDICT_WORDS * 4 }>();
        assert!(
            no_bytes_left_over.is_empty(),
            "取的是态数乘一个态的字节数那么多，切成整态不剩"
        );
        let verdict_words = bytes_of_each_state
            .iter()
            .map(|state_bytes| {
                let mut state_words = [0u32; crate::crash_judge_gpu::VERDICT_WORDS];
                let (bytes_of_each_word, no_bytes_left_over_in_the_state) =
                    state_bytes.as_chunks::<4>();
                assert!(
                    no_bytes_left_over_in_the_state.is_empty(),
                    "一个态的字节数是字数乘 4，切成整字不剩"
                );
                for (word, word_bytes) in state_words.iter_mut().zip(bytes_of_each_word) {
                    *word = u32::from_le_bytes(*word_bytes);
                }
                state_words
            })
            .collect();
        let judged_on = JudgingDevice::from_attribute_bytes(&bytes[cursor.position..])?;
        Some(Self {
            verdict_words,
            judged_on,
        })
    }
}

/// 一张判表的结论字在一帧里要写的行（[`VerdictStore::packed_tables_verdict_words_rows_for_one_frame`]）：GPU 批处理服务把它放进自己的写结论字那一帧，
/// 这一帧原子地落：表 `21` 那一行与删表 `20` 那一行同一帧。两个守卫都在写的那一刻核，此刻库里那一行与读到的不同就整帧拒：
/// 判表结论字守卫防盖掉别人刚写的那一行，判表待判行守卫防删掉判段在服务读完之后补写或抬成强制的那一行。
#[cfg(feature = "gpu")]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackedTablesVerdictWordsRowsForOneFrame {
    /// 要写的行（键、值，键是库里的整把键字节）：恰好一行，表 `21` 这一把键那一行。
    pub rows_to_put: Vec<(Vec<u8>, Vec<u8>)>,
    /// 判表结论字守卫：表 `21` 这一把键，与编之前读到的值（不在是 `None`）。
    pub verdict_words_row_key: Vec<u8>,
    pub verdict_words_row_as_read: Option<Vec<u8>>,
    /// 要删的判表待判行的键，与服务取这一行时读到的值。
    pub pending_row_key_to_delete: Vec<u8>,
    pub pending_row_as_read: Vec<u8>,
}

/// 编不出一张判表的结论字那一帧（[`VerdictStore::packed_tables_verdict_words_rows_for_one_frame`]）。
#[cfg(feature = "gpu")]
#[derive(Debug)]
pub enum PackedTablesVerdictWordsFrameRefused {
    /// 结论字的态数与待判行记的态数不等：一行都不编，服务拒这一行。
    StateCountDiffersFromThePendingRow {
        states_judged: u64,
        state_count_of_the_pending_row: u64,
    },
    /// 编之前读表 `21` 那一行（守卫）读不成。
    VerdictWordsRowUnreadable(VerdictStoreError),
}

impl VerdictStore {
    /// 写一行判表待判行，不在才写；在而不强制、这一份强制现判就整份换成这一份（抬成强制）。认领、读回、等别的客户端放手的写法同
    /// [`Self::put_the_gpu_pending_row_unless_present_or_raise_it_to_forced`]。
    ///
    /// # Errors
    /// 读写、认领不成；等过 [`PRUNE_SERVER_NUMBER_CLAIM_WAIT_LIMIT`] 还被别人认领着；库里那一行不合布局。
    pub fn put_the_packed_tables_pending_row_unless_present_or_raise_it_to_forced(
        &self,
        key: &PackedTablesKey,
        row: &PackedTablesPendingRow,
    ) -> Result<PackedTablesPendingRowPut, VerdictStoreError> {
        let key_bytes = VerdictStoreKey::PackedTablesPendingRow(key).to_key_bytes();
        let waiting_started = std::time::Instant::now();
        loop {
            match self.service.claim(&key_bytes.0, "认领判表待判行")? {
                ClaimOutcome::Claimed => {
                    self.put_one_row_through_the_batch(&key_bytes.0, &row.to_value_bytes())?;
                    return Ok(PackedTablesPendingRowPut::Written);
                }
                ClaimOutcome::AlreadyConcluded => {
                    return match self.read_packed_tables_pending_row(key)? {
                        Some(present) => match (present.forced, row.forced) {
                            (ForcedRejudging::NotForced, ForcedRejudging::Forced) => {
                                self.put_one_row_through_the_batch(
                                    &key_bytes.0,
                                    &row.to_value_bytes(),
                                )?;
                                Ok(PackedTablesPendingRowPut::RaisedToForced { replaced: present })
                            }
                            (ForcedRejudging::NotForced, ForcedRejudging::NotForced)
                            | (ForcedRejudging::Forced, ForcedRejudging::NotForced)
                            | (ForcedRejudging::Forced, ForcedRejudging::Forced) => {
                                Ok(PackedTablesPendingRowPut::AlreadyPresent(present))
                            }
                        },
                        None => {
                            self.put_one_row_through_the_batch(
                                &key_bytes.0,
                                &row.to_value_bytes(),
                            )?;
                            Ok(PackedTablesPendingRowPut::Written)
                        }
                    };
                }
                ClaimOutcome::HeldByOther { .. } => {
                    if waiting_started.elapsed() >= PRUNE_SERVER_NUMBER_CLAIM_WAIT_LIMIT {
                        return Err(self.service.failed(
                            PRUNE_SERVER_UNAVAILABLE_EXIT_CODE,
                            format!(
                                "判表待判行（判表 {}）被别的客户端认领着，等过 {} 秒还没放",
                                crate::hexadecimal::hexadecimal_text(
                                    &key.packed_judge_tables_digest[..8]
                                ),
                                PRUNE_SERVER_NUMBER_CLAIM_WAIT_LIMIT.as_secs()
                            ),
                        ));
                    }
                    std::thread::sleep(PRUNE_SERVER_NUMBER_CLAIM_RETRY_INTERVAL);
                }
            }
        }
    }

    /// 照键读一行判表待判行（服务认领之后照键重读也经它）；不在（没写过、服务判完删了）是 `None`。
    ///
    /// # Errors
    /// 服务读不成；库里那一行不合布局（[`PackedTablesPendingRow::from_value_bytes`] 认不出）。
    pub fn read_packed_tables_pending_row(
        &self,
        key: &PackedTablesKey,
    ) -> Result<Option<PackedTablesPendingRow>, VerdictStoreError> {
        let key = VerdictStoreKey::PackedTablesPendingRow(key);
        match self.read_one_stored_value(&key)? {
            None => Ok(None),
            Some(value_bytes) => PackedTablesPendingRow::from_value_bytes(&value_bytes)
                .map(Some)
                .ok_or_else(|| {
                    stored_bytes_mismatch(
                        VerdictStoreTable::PackedTablesPendingRows,
                        &key.to_key_bytes().0,
                        "判表待判行的值不合布局",
                    )
                }),
        }
    }

    /// 库里全部判表待判行（键、行），按键的次序：GPU 批处理服务从这里取要判的判表。
    ///
    /// # Errors
    /// 服务扫不成；库里的一行键或值不合布局。
    pub fn every_packed_tables_pending_row(
        &self,
    ) -> Result<Vec<(PackedTablesKey, PackedTablesPendingRow)>, VerdictStoreError> {
        let mut rows = Vec::new();
        self.visit_entries_with_key_prefix_kept_in_or_out(
            VerdictStoreKey::WholeTablePrefix(VerdictStoreTable::PackedTablesPendingRows)
                .to_key_bytes(),
            &mut |(key_bytes, value_bytes)| {
                let not_in_the_layout = || {
                    stored_bytes_mismatch(
                        VerdictStoreTable::PackedTablesPendingRows,
                        &key_bytes,
                        "判表待判行的键或值不合布局",
                    )
                };
                let key = key_bytes
                    .get(KEY_TAG_BYTES..)
                    .and_then(PackedTablesKey::from_bytes_after_the_tag)
                    .ok_or_else(not_in_the_layout)?;
                let row = PackedTablesPendingRow::from_value_bytes(&value_bytes)
                    .ok_or_else(not_in_the_layout)?;
                rows.push((key, row));
                Ok(())
            },
        )?;
        Ok(rows)
    }

    /// 删一行判表待判行（写删除记号）。产品里由 GPU 批处理服务在写判表结论字的那一帧里删；判段不删。
    ///
    /// # Errors
    /// 服务写不成。
    pub fn delete_the_packed_tables_pending_row(
        &self,
        key: &PackedTablesKey,
    ) -> Result<(), VerdictStoreError> {
        let mut batch = LibraryWriteBatch::default();
        batch.delete(
            VerdictStoreKey::PackedTablesPendingRow(key)
                .to_key_bytes()
                .0,
        );
        self.write_batch_keeping_the_kept_out_rows_in_this_process(batch)
    }

    /// 照键读一行判表结论字；不在是 `None`。
    ///
    /// # Errors
    /// 服务读不成；库里那一行不合布局（`PackedTablesVerdictWordsRow::from_value_bytes` 认不出）。
    #[cfg(feature = "gpu")]
    pub fn read_packed_tables_verdict_words(
        &self,
        key: &PackedTablesKey,
    ) -> Result<Option<PackedTablesVerdictWordsRow>, VerdictStoreError> {
        let key = VerdictStoreKey::PackedTablesVerdictWords(key);
        match self.read_one_stored_value(&key)? {
            None => Ok(None),
            Some(value_bytes) => PackedTablesVerdictWordsRow::from_value_bytes(&value_bytes)
                .map(Some)
                .ok_or_else(|| {
                    stored_bytes_mismatch(
                        VerdictStoreTable::PackedTablesVerdictWords,
                        &key.to_key_bytes().0,
                        "判表结论字的值不合布局",
                    )
                }),
        }
    }

    /// 服务判完一张判表之后要在一帧里写的行（[`PackedTablesVerdictWordsRowsForOneFrame`]）：照键、服务读到的那一份待判行、全部状态的结论字与写它的设备编
    /// 表 `21` 那一行与删表 `20` 那一行；编之前先读表 `21` 这一把键那一行当守卫。不写库：GPU 批处理服务拿它在自己的写结论字那一帧里写。
    ///
    /// # Errors
    /// 结论字的态数与待判行记的态数不等（一行都不编）；服务读不成。
    #[cfg(feature = "gpu")]
    pub fn packed_tables_verdict_words_rows_for_one_frame(
        &self,
        key: &PackedTablesKey,
        pending_row_as_read: &PackedTablesPendingRow,
        verdict_words: &[[u32; crate::crash_judge_gpu::VERDICT_WORDS]],
        judged_on: &JudgingDevice,
    ) -> Result<PackedTablesVerdictWordsRowsForOneFrame, PackedTablesVerdictWordsFrameRefused> {
        let states_judged = u64::try_from(verdict_words.len()).expect("一张判表的态数装得进 u64");
        if states_judged != pending_row_as_read.state_count {
            return Err(
                PackedTablesVerdictWordsFrameRefused::StateCountDiffersFromThePendingRow {
                    states_judged,
                    state_count_of_the_pending_row: pending_row_as_read.state_count,
                },
            );
        }
        let verdict_words_row_as_read = self
            .read_values_kept_in_or_out(&[VerdictStoreKey::PackedTablesVerdictWords(key)
                .to_key_bytes()
                .0
                .as_slice()])
            .map_err(PackedTablesVerdictWordsFrameRefused::VerdictWordsRowUnreadable)?
            .pop()
            .expect("点读一把键交回一项");
        let verdict_words_row_key =
            key_bytes_of_the_packed_tables_verdict_words_in(&self.service.key_space, key);
        Ok(PackedTablesVerdictWordsRowsForOneFrame {
            rows_to_put: vec![(
                verdict_words_row_key.clone(),
                PackedTablesVerdictWordsRow {
                    verdict_words: verdict_words.to_vec(),
                    judged_on: judged_on.clone(),
                }
                .to_value_bytes(),
            )],
            verdict_words_row_key,
            verdict_words_row_as_read,
            pending_row_key_to_delete: key_bytes_of_the_packed_tables_pending_row_in(
                &self.service.key_space,
                key,
            ),
            pending_row_as_read: pending_row_as_read.to_value_bytes(),
        })
    }

    /// 单测替 GPU 批处理服务把一帧的判表结论字写进这个库（写表 `21` 那一行、删表 `20` 那一行同一批），不核守卫：服务那一侧的写帧核守卫。
    /// 帧里的键是接上键空间之后的整把键，经这一份写时先去掉键空间那一段（这一份写的时候再接上）。
    ///
    /// # Errors
    /// 服务写不成。
    #[cfg(all(feature = "gpu", test))]
    pub(crate) fn write_the_packed_tables_verdict_words_rows_of_one_frame(
        &self,
        frame: &PackedTablesVerdictWordsRowsForOneFrame,
    ) -> Result<(), VerdictStoreError> {
        let mut batch = LibraryWriteBatch::default();
        for (key, value) in &frame.rows_to_put {
            batch.put(self.service.key_without_the_key_space(key), value);
        }
        batch.delete(
            self.service
                .key_without_the_key_space(&frame.pending_row_key_to_delete),
        );
        self.write_batch_keeping_the_kept_out_rows_in_this_process(batch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 一把写死的判表待判行的键：两段各一个写死的字节。
    fn a_packed_tables_key() -> PackedTablesKey {
        PackedTablesKey {
            packed_judge_tables_digest: [0x41; SHA256_DIGEST_BYTES],
            container_capacity_knobs_digest: [0x42; SHA256_DIGEST_BYTES],
        }
    }

    /// 一行写死的判表待判行：旋钮、态数与强制现判由调用方给，入池序号写死。
    fn a_packed_tables_pending_row(
        container_capacity_knobs: ContainerCapacityKnobs,
        state_count: u64,
        forced: ForcedRejudging,
    ) -> PackedTablesPendingRow {
        PackedTablesPendingRow {
            container_capacity_knobs,
            state_count,
            forced,
            pool_entry_number: 1_700_000_000_987_654_321,
        }
    }

    /// 墓碑树压到 2、谱系树压到 1359（它的产品容量）、退役树没装旋钮的那一组。
    fn container_capacity_knobs_of_two_trees() -> ContainerCapacityKnobs {
        ContainerCapacityKnobs {
            tombstone: Some(TombstoneContainerCapacityKnob::new(2).expect("1..=815")),
            lineage: Some(LineageContainerCapacityKnob::new(1359).expect("1..=1359")),
            retirement: None,
        }
    }

    /// 判表待判行与判表结论字两张表的键：表标签 `20`、`21` 之后是判表摘要、旋钮摘要两段，次序就是这样；
    /// 表标签之后那一截读回原样，多一个字节、少一个字节都读不出。
    /// 判别力（`crates/mutations.tsv`）：键里旋钮摘要排到判表摘要前面，这条红。
    #[test]
    fn packed_tables_keys_take_the_two_segments_after_their_tags() {
        let key = a_packed_tables_key();
        let mut segments = Vec::new();
        segments.extend_from_slice(&[0x41; SHA256_DIGEST_BYTES]);
        segments.extend_from_slice(&[0x42; SHA256_DIGEST_BYTES]);
        let mut pending_row_key = vec![0x20];
        pending_row_key.extend_from_slice(&segments);
        let mut verdict_words_key = vec![0x21];
        verdict_words_key.extend_from_slice(&segments);
        assert_eq!(
            (
                key_bytes_of_the_packed_tables_pending_row_in_the_one_library(&key),
                key_bytes_of_the_packed_tables_verdict_words_in_the_one_library(&key)
            ),
            (pending_row_key, verdict_words_key),
            "两张表的键：标签、判表摘要、旋钮摘要"
        );
        assert_eq!(
            segments.len(),
            PACKED_TABLES_KEY_BYTES_AFTER_THE_TAG,
            "表标签之后定长两段"
        );
        assert_eq!(
            PackedTablesKey::from_bytes_after_the_tag(&segments),
            Some(key),
            "读回原样"
        );
        let mut one_more_byte = segments.clone();
        one_more_byte.push(0);
        assert_eq!(
            (
                PackedTablesKey::from_bytes_after_the_tag(&one_more_byte),
                PackedTablesKey::from_bytes_after_the_tag(&segments[..segments.len() - 1])
            ),
            (None, None),
            "多一个字节、少一个字节都读不出"
        );
    }

    /// 判表结论字与判表待判行的键里都没有版本那一截（判器的代码不进键，键只是判表摘要与旋钮摘要）：照判不入库从不挡这两张表的行，
    /// 一把与键里某一段逐字节相同的版本登记成照判不入库，那一行照样入库。
    /// 判别力（`crates/mutations.tsv`）：判表结论字的键又登版本那一截（从旋钮摘要起），这条红。
    #[test]
    fn the_packed_tables_rows_carry_no_version_field() {
        let key = a_packed_tables_key();
        let no_version_field: &[usize] = &[];
        assert_eq!(
            (
                version_field_starts_in_the_key(
                    &key_bytes_of_the_packed_tables_verdict_words_in_the_one_library(&key)
                ),
                version_field_starts_in_the_key(
                    &key_bytes_of_the_packed_tables_pending_row_in_the_one_library(&key)
                )
            ),
            (no_version_field, no_version_field),
            "判表结论字与判表待判行都没有版本那一截"
        );
        let kept_out = RowsKeptOutOfTheLibrary {
            versions: BTreeSet::from([key.container_capacity_knobs_digest]),
            rows: BTreeMap::new(),
        };
        assert!(
            !kept_out
                .keeps_out(&key_bytes_of_the_packed_tables_verdict_words_in_the_one_library(&key)),
            "与旋钮摘要逐字节相同的那一把登记成照判不入库：判表结论字那一行照样入库"
        );
    }

    /// 判表待判行的值照写下的读回原样（压了旋钮的一组、产品那一组；强制与不强制）；定长；有无那一个字节不是 `00`、`01` 的，
    /// 没装旋钮的那一栏值不是 0 的，装了的那一栏越出它那一棵树的取值范围的，强制现判那个字节认不出的，多一个字节、少一个字节的，都读不出。
    /// 判别力（`crates/mutations.tsv`）：没装旋钮的那一栏值不是 0 照样读成没装，这条红。
    #[test]
    fn packed_tables_pending_rows_read_back_as_written() {
        for row in [
            a_packed_tables_pending_row(
                container_capacity_knobs_of_two_trees(),
                65_536,
                ForcedRejudging::Forced,
            ),
            a_packed_tables_pending_row(
                ContainerCapacityKnobs::PRODUCT,
                1,
                ForcedRejudging::NotForced,
            ),
        ] {
            let bytes = row.to_value_bytes();
            assert_eq!(
                bytes.len(),
                PACKED_TABLES_PENDING_ROW_VALUE_BYTES,
                "判表待判行的值定长"
            );
            assert_eq!(
                PackedTablesPendingRow::from_value_bytes(&bytes),
                Some(row),
                "读回原样"
            );
        }
        let bytes = a_packed_tables_pending_row(
            container_capacity_knobs_of_two_trees(),
            3,
            ForcedRejudging::NotForced,
        )
        .to_value_bytes();
        let retirement_column = 2 * CONTAINER_CAPACITY_KNOB_COLUMN_BYTES;
        let with_the_byte = |position: usize, byte: u8| {
            let mut changed = bytes.clone();
            changed[position] = byte;
            changed
        };
        let forced_position = CONTAINER_CAPACITY_KNOB_COLUMNS_BYTES + 8;
        let mut one_more_byte = bytes.clone();
        one_more_byte.push(0);
        for (what, changed) in [
            (
                "有无那一个字节是 02",
                with_the_byte(retirement_column, 0x02),
            ),
            (
                "没装旋钮的那一栏值不是 0",
                with_the_byte(retirement_column + 8, 1),
            ),
            ("墓碑树的旋钮压到 0", with_the_byte(8, 0)),
            (
                "强制现判那个字节是 02",
                with_the_byte(forced_position, 0x02),
            ),
            ("多一个字节", one_more_byte),
            ("少一个字节", bytes[..bytes.len() - 1].to_vec()),
        ] {
            assert_eq!(
                PackedTablesPendingRow::from_value_bytes(&changed),
                None,
                "{what}读不出"
            );
        }
        assert_eq!(
            container_capacity_knobs_of_the_columns(&container_capacity_knob_columns_bytes(
                container_capacity_knobs_of_two_trees()
            )),
            Some(container_capacity_knobs_of_two_trees()),
            "旋钮栏单独编解也读回原样"
        );
    }

    /// 判表待判行不在才写：头一次写下；同一把键再写一份不强制的交回库里那一份；再写一份强制的抬成强制、照它整份写下；强制的再来一份交回库里那一份；
    /// 库里全部判表待判行照键列得出；删掉之后读成不在，再写照写。
    /// 判别力（`crates/mutations.tsv`）：在而不强制、这一份强制时不抬成强制（交回库里那一份），这条红。
    #[test]
    fn a_packed_tables_pending_row_is_put_unless_present_or_raised_to_forced() {
        let library = TestLibrary::new("packed-tables-pending-row-unless-present");
        let store = library.open();
        let key = a_packed_tables_key();
        let not_forced = a_packed_tables_pending_row(
            ContainerCapacityKnobs::PRODUCT,
            2,
            ForcedRejudging::NotForced,
        );
        let forced = PackedTablesPendingRow {
            forced: ForcedRejudging::Forced,
            pool_entry_number: not_forced.pool_entry_number + 1,
            ..not_forced
        };
        let put = |row: &PackedTablesPendingRow| {
            store
                .put_the_packed_tables_pending_row_unless_present_or_raise_it_to_forced(&key, row)
                .expect("写得进库")
        };
        assert_eq!(
            put(&not_forced),
            PackedTablesPendingRowPut::Written,
            "头一次写下"
        );
        assert_eq!(
            put(&not_forced),
            PackedTablesPendingRowPut::AlreadyPresent(not_forced),
            "在而这一份不强制：交回库里那一份"
        );
        assert_eq!(
            put(&forced),
            PackedTablesPendingRowPut::RaisedToForced {
                replaced: not_forced
            },
            "在而不强制、这一份强制：抬成强制"
        );
        assert_eq!(
            put(&not_forced),
            PackedTablesPendingRowPut::AlreadyPresent(forced),
            "已是强制：交回库里那一份"
        );
        assert_eq!(
            (
                store
                    .read_packed_tables_pending_row(&key)
                    .expect("读得了库"),
                store.every_packed_tables_pending_row().expect("扫得了库")
            ),
            (Some(forced), vec![(key, forced)]),
            "库里是抬成强制的那一份"
        );
        store
            .delete_the_packed_tables_pending_row(&key)
            .expect("删得了");
        assert_eq!(
            store
                .read_packed_tables_pending_row(&key)
                .expect("读得了库"),
            None,
            "删掉之后读成不在"
        );
        assert_eq!(
            put(&not_forced),
            PackedTablesPendingRowPut::Written,
            "删掉之后再写照写"
        );
    }

    /// 判表结论字的值照写下的读回原样（CPU 与 GPU 两种设备）：态数 8 字节小端、逐态的字各 4 字节小端；少一个字节（设备属性读不全）、
    /// 多一个字节、态数比字多的都读不出。
    /// 判别力（`crates/mutations.tsv`）：结论字写成大端（读回时照小端解），这条红。
    #[cfg(feature = "gpu")]
    #[test]
    fn packed_tables_verdict_words_rows_read_back_as_written() {
        use crate::crash_judge_gpu::VERDICT_WORDS;
        let verdict_words: Vec<[u32; VERDICT_WORDS]> = (0..3u32)
            .map(|state| {
                let mut words = [0u32; VERDICT_WORDS];
                for (word_index, word) in words.iter_mut().enumerate() {
                    *word = 0x0102_0300
                        + state * 0x100
                        + u32::try_from(word_index).expect("字号装得进 u32");
                }
                words
            })
            .collect();
        for judged_on in [
            JudgingDevice::Cpu,
            JudgingDevice::Gpu {
                card_type: "测试造的卡".to_string(),
                driver_version: "测试造的驱动".to_string(),
            },
        ] {
            let row = PackedTablesVerdictWordsRow {
                verdict_words: verdict_words.clone(),
                judged_on: judged_on.clone(),
            };
            let bytes = row.to_value_bytes();
            assert_eq!(bytes[..8], 3u64.to_le_bytes(), "头 8 字节是态数，小端");
            assert_eq!(
                bytes[8..12],
                verdict_words[0][0].to_le_bytes(),
                "态序号 0 的那个状态的字 0，小端"
            );
            assert_eq!(
                PackedTablesVerdictWordsRow::from_value_bytes(&bytes),
                Some(row),
                "{judged_on:?}：读回原样"
            );
            let mut one_more_byte = bytes.clone();
            one_more_byte.push(0);
            let mut more_states_than_words = bytes.clone();
            more_states_than_words[..8].copy_from_slice(&4u64.to_le_bytes());
            for (what, changed) in [
                ("少一个字节", bytes[..bytes.len() - 1].to_vec()),
                ("多一个字节", one_more_byte),
                ("态数比字多", more_states_than_words),
            ] {
                assert_eq!(
                    PackedTablesVerdictWordsRow::from_value_bytes(&changed),
                    None,
                    "{judged_on:?}：{what}读不出"
                );
            }
        }
    }

    /// 服务那一帧：写表 `21` 这一把键那一行（值是结论字连同设备）、删表 `20` 这一把键那一行；判表结论字守卫是编之前读到的表 `21` 那一行
    /// （头一回不在，写进库之后再编一回读到的就是写下的那一份），判表待判行守卫是服务读到的那一份待判行的字节；结论字的态数与待判行记的不等就不编。
    /// 写进库之后判表结论字读得回、判表待判行不在。
    /// 判别力（`crates/mutations.tsv`）：那一帧删的是判表结论字那一把键（判表待判行留着），这条红；编之前不读表 `21` 那一行（守卫恒是不在），这条红。
    #[cfg(feature = "gpu")]
    #[test]
    fn a_packed_tables_verdict_words_frame_puts_the_words_and_deletes_the_pending_row() {
        use crate::crash_judge_gpu::VERDICT_WORDS;
        let library = TestLibrary::new("packed-tables-verdict-words-frame");
        let store = library.open();
        let key = a_packed_tables_key();
        let row = a_packed_tables_pending_row(
            ContainerCapacityKnobs::PRODUCT,
            2,
            ForcedRejudging::NotForced,
        );
        store
            .put_the_packed_tables_pending_row_unless_present_or_raise_it_to_forced(&key, &row)
            .expect("写得进库");
        let verdict_words = vec![[5u32; VERDICT_WORDS], [6u32; VERDICT_WORDS]];
        assert!(
            matches!(
                store.packed_tables_verdict_words_rows_for_one_frame(
                    &key,
                    &row,
                    &[[5u32; VERDICT_WORDS]],
                    &JudgingDevice::Cpu
                ),
                Err(
                    PackedTablesVerdictWordsFrameRefused::StateCountDiffersFromThePendingRow {
                        states_judged: 1,
                        state_count_of_the_pending_row: 2
                    }
                )
            ),
            "结论字的态数与待判行记的不等：不编"
        );
        let frame = store
            .packed_tables_verdict_words_rows_for_one_frame(
                &key,
                &row,
                &verdict_words,
                &JudgingDevice::Cpu,
            )
            .expect("态数相同、读得了库：编得出");
        let verdict_words_row = (
            key_bytes_of_the_packed_tables_verdict_words_in(store.key_space(), &key),
            PackedTablesVerdictWordsRow {
                verdict_words: verdict_words.clone(),
                judged_on: JudgingDevice::Cpu,
            }
            .to_value_bytes(),
        );
        assert_eq!(
            frame,
            PackedTablesVerdictWordsRowsForOneFrame {
                rows_to_put: vec![verdict_words_row.clone()],
                verdict_words_row_key: verdict_words_row.0.clone(),
                verdict_words_row_as_read: None,
                pending_row_key_to_delete: key_bytes_of_the_packed_tables_pending_row_in(
                    store.key_space(),
                    &key
                ),
                pending_row_as_read: row.to_value_bytes(),
            },
            "写表 21 那一行、删表 20 那一行，键都接上这一份一次性库的名字空间；表 21 那一行编之前不在，守卫是读到的那一份待判行"
        );
        store
            .write_the_packed_tables_verdict_words_rows_of_one_frame(&frame)
            .expect("写得进库");
        assert_eq!(
            (
                store
                    .read_packed_tables_pending_row(&key)
                    .expect("读得了库"),
                store
                    .read_packed_tables_verdict_words(&key)
                    .expect("读得了库")
                    .map(|words| words.verdict_words)
            ),
            (None, Some(verdict_words.clone())),
            "那一帧写下了判表结论字、删掉了判表待判行"
        );
        let composed_again = store
            .packed_tables_verdict_words_rows_for_one_frame(
                &key,
                &row,
                &verdict_words,
                &JudgingDevice::Cpu,
            )
            .expect("态数相同、读得了库：编得出");
        assert_eq!(
            composed_again.verdict_words_row_as_read,
            Some(verdict_words_row.1),
            "表 21 那一行写下之后再编：守卫读到的是写下的那一份"
        );
    }
}

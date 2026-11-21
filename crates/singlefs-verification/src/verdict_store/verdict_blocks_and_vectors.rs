//! 判定块、判定向量（按内容、按编号）、违例与判法单元组成清单几张表（`01`–`04`、`09`）：登记判定向量、
//! 编号由服务统一给、开库之后在读里取来的判定向量怎么并进来，存块读块与组成清单。

use super::*;

impl VerdictStore {
    /// 给一种判定向量一个编号：登记过的返回原来的编号、什么都不写；没登记过的在服务里认领下一个编号、两张表同一帧写下去之后才返回
    /// （[`Self::register_verdict_vector_in_the_prune_server`]：别的进程同时登记同一种，只得一个编号）。
    ///
    /// # Errors
    /// 2^32 种都登记过了（`EveryU32VerdictVectorNumberIsAlreadyRegistered`，什么都没写）；服务读写、认领不成。
    pub fn register_verdict_vector(
        &mut self,
        verdict_vector: &EncodedVerdictVector,
    ) -> Result<VerdictVectorNumber, VerdictStoreError> {
        self.take_in_the_verdict_vectors_fetched_after_opening()?;
        if let Some(number) = self.verdict_vector_number_by_content.get(&verdict_vector.0) {
            return Ok(*number);
        }
        let service = Arc::clone(&self.service);
        self.register_verdict_vector_in_the_prune_server(&service, verdict_vector)
    }

    /// 这个编号登记的是哪种判定向量；这个库里没有这个编号时是 `None`。
    #[must_use]
    pub fn verdict_vector_of_number(
        &self,
        number: VerdictVectorNumber,
    ) -> Option<&EncodedVerdictVector> {
        if let Some(verdict_vector) = self
            .verdict_vector_by_number
            .get(number.position_in_the_registration_order())
        {
            return Some(verdict_vector);
        }
        self.lock_the_verdict_vectors_fetched_after_opening()
            .get(&number.as_u32())
            .copied()
    }

    /// 存一块在这一判法单元指纹下的判定与它的违例：块与每个违例状态同一批同步写下去之后才返回；违例状态的判定向量编号取块里那个状态的。
    /// 同一块键在别的判法单元指纹下的行不受影响。这一版本下已经存着不同的一份、两份里有一份红的：照这一份换上
    /// （[`VerdictBlockStoring::ReplacedBecauseOneOfTheTwoIsRed`]，旧的违例行同一批删掉）。
    ///
    /// # Errors
    /// 这个块键在这一版本下已经存着不同的块或不同的违例、两份都不红（`DifferentVerdictsUnderAStoredBlockKeyNotDecidedInTheFirstVersion`，
    /// 什么都没写）；已存的违例读回来不合布局；服务读写不成。
    ///
    /// # Panics
    /// 块里有这个库没登记过的编号（拿了别的库的编号），或违例状态的块内序号不在块里：都是调用方的 bug。
    pub fn store_verdict_block(
        &mut self,
        block_key: &VerdictBlockKey,
        version: &JudgingUnitsFingerprint,
        block: &VerdictBlock,
        violating_states: &[StateOffsetInBlock],
    ) -> Result<VerdictBlockStoring, VerdictStoreError> {
        let registered_count = self.known_verdict_vector_count();
        assert!(
            block
                .verdict_vector_number_of_each_state()
                .iter()
                .all(|number| number.position_in_the_registration_order() < registered_count),
            "块里的判定向量编号必须是这个库登记过的（这个库登记了 {registered_count} 种）"
        );
        let mut batch = LibraryWriteBatch::default();
        let storing = self.put_a_verdict_block_into_the_batch(
            block_key,
            version,
            block,
            violating_states,
            &mut batch,
        )?;
        match storing {
            VerdictBlockStoring::AlreadyStoredIdentically => return Ok(storing),
            VerdictBlockStoring::WrittenAndSynced
            | VerdictBlockStoring::ReplacedBecauseOneOfTheTwoIsRed => {}
        }
        self.write_batch_keeping_the_kept_out_rows_in_this_process(batch)?;
        Ok(storing)
    }

    /// 一块判定连同违例放进 `batch`（[`Self::store_verdict_block`] 与导判定都经它，规矩只在这一处）：这一版本下已经存着逐字节相同的块与相同的违例的，
    /// 什么都不放、交回 [`VerdictBlockStoring::AlreadyStoredIdentically`]；不同而两份里有一份红的，照这一份换上、旧的有而这一份没有的违例行同一批删掉
    /// （[`VerdictBlockStoring::ReplacedBecauseOneOfTheTwoIsRed`]）；没存过的交回 [`VerdictBlockStoring::WrittenAndSynced`]。交回的是这一批写下去之后的结局，
    /// 写不写由调用方定。块里的编号可以是导判定攒在这一批里、还没写下去的（不核登记过没有，存块那一路调用方先核）。
    ///
    /// # Errors
    /// 两份都不红而不同（`DifferentVerdictsUnderAStoredBlockKeyNotDecidedInTheFirstVersion`，`batch` 里没放这一块的行）；已存的违例读回来不合布局；
    /// 服务读不成。
    ///
    /// # Panics
    /// 违例状态的块内序号不在块里：调用方的 bug。
    fn put_a_verdict_block_into_the_batch(
        &self,
        block_key: &VerdictBlockKey,
        version: &JudgingUnitsFingerprint,
        block: &VerdictBlock,
        violating_states: &[StateOffsetInBlock],
        batch: &mut LibraryWriteBatch,
    ) -> Result<VerdictBlockStoring, VerdictStoreError> {
        let requested_violations: Vec<StoredViolation> = violating_states
            .iter()
            .copied()
            .collect::<BTreeSet<StateOffsetInBlock>>()
            .into_iter()
            .map(|offset_in_block| StoredViolation {
                offset_in_block,
                verdict_vector_number: block
                    .verdict_vector_number_at(offset_in_block)
                    .expect("违例状态的块内序号必须在块里（调用方给的序号超出了块长）"),
            })
            .collect();
        let block_key_bytes = VerdictStoreKey::VerdictBlock(block_key, version).to_key_bytes();
        let block_value_bytes = block.to_block_value_bytes();
        let stored_block_value = self.read_value_kept_in_or_out(&block_key_bytes.0)?;
        let mut storing = VerdictBlockStoring::WrittenAndSynced;
        if let Some(stored_block_value_bytes) = stored_block_value {
            let stored_violations = self.violations_in_block(block_key, version)?;
            let is_identical = stored_block_value_bytes == block_value_bytes
                && stored_violations == requested_violations;
            if is_identical {
                return Ok(VerdictBlockStoring::AlreadyStoredIdentically);
            }
            // 记红、撤绿：两份里有一份红的，照要存的这一份换上；旧的那一份有、这一份没有的违例行同一批删掉
            let one_of_the_two_is_red =
                !stored_violations.is_empty() || !requested_violations.is_empty();
            if !one_of_the_two_is_red {
                return Err(
                    VerdictStoreError::DifferentVerdictsUnderAStoredBlockKeyNotDecidedInTheFirstVersion {
                        block_key: block_key.clone(),
                    },
                );
            }
            for stale in stored_violations.iter().filter(|stored| {
                !requested_violations
                    .iter()
                    .any(|requested| requested.offset_in_block == stored.offset_in_block)
            }) {
                batch.delete(
                    VerdictStoreKey::Violation(block_key, version, stale.offset_in_block)
                        .to_key_bytes()
                        .0,
                );
            }
            storing = VerdictBlockStoring::ReplacedBecauseOneOfTheTwoIsRed;
        }
        batch.put(&block_key_bytes.0, &block_value_bytes);
        for violation in &requested_violations {
            batch.put(
                VerdictStoreKey::Violation(block_key, version, violation.offset_in_block)
                    .to_key_bytes()
                    .0,
                violation.verdict_vector_number.to_stored_bytes(),
            );
        }
        Ok(storing)
    }

    pub(super) fn registered_verdict_vector_number(
        &self,
        table: VerdictStoreTable,
        key_bytes: &[u8],
        stored_number: u32,
    ) -> Result<VerdictVectorNumber, VerdictStoreError> {
        self.fetch_the_verdict_vectors_of_unknown_numbers(&[stored_number])?;
        let number = VerdictVectorNumber(stored_number);
        if number.position_in_the_registration_order() < self.known_verdict_vector_count() {
            return Ok(number);
        }
        Err(stored_bytes_mismatch(
            table,
            key_bytes,
            "存着的判定向量编号在这个库里没登记过",
        ))
    }

    /// 存着的一块（判定块或结论块）的值读成每个状态的编号：按 4 字节大端切，每个编号都要在这个库里登记过。空不空不在这里判。
    pub(super) fn registered_verdict_vector_numbers_of_a_stored_block(
        &self,
        table: VerdictStoreTable,
        key_bytes: &[u8],
        value_bytes: &[u8],
    ) -> Result<Vec<VerdictVectorNumber>, VerdictStoreError> {
        let Some(stored_numbers) = stored_verdict_vector_numbers(value_bytes) else {
            return Err(stored_bytes_mismatch(
                table,
                key_bytes,
                "存着的块值的字节数不是 4 字节编号的整数倍",
            ));
        };
        self.fetch_the_verdict_vectors_of_unknown_numbers(&stored_numbers)?;
        let known_count = self.known_verdict_vector_count();
        stored_numbers
            .into_iter()
            .map(|stored_number| {
                let number = VerdictVectorNumber(stored_number);
                if number.position_in_the_registration_order() < known_count {
                    Ok(number)
                } else {
                    Err(stored_bytes_mismatch(
                        table,
                        key_bytes,
                        "存着的判定向量编号在这个库里没登记过",
                    ))
                }
            })
            .collect()
    }

    /// 读回一块在这一判法单元指纹下的判定；这个块键在这一版本下没存过时是 `None`（别的版本存过也是 `None`：复用只认同一版本）。
    ///
    /// # Errors
    /// 存着的块值是空的或点名了没登记的编号；服务读不成。
    pub fn read_verdict_block(
        &self,
        block_key: &VerdictBlockKey,
        version: &JudgingUnitsFingerprint,
    ) -> Result<Option<VerdictBlock>, VerdictStoreError> {
        let block_key_bytes = VerdictStoreKey::VerdictBlock(block_key, version).to_key_bytes();
        let Some(block_value_bytes) = self.read_value_kept_in_or_out(&block_key_bytes.0)? else {
            return Ok(None);
        };
        let verdict_vector_number_of_each_state = self
            .registered_verdict_vector_numbers_of_a_stored_block(
                VerdictStoreTable::VerdictBlocks,
                &block_key_bytes.0,
                &block_value_bytes,
            )?;
        VerdictBlock::new(verdict_vector_number_of_each_state)
            .map(Some)
            .map_err(|EmptyVerdictBlockRejected| {
                stored_bytes_mismatch(
                    VerdictStoreTable::VerdictBlocks,
                    &block_key_bytes.0,
                    "存着的块值是空的",
                )
            })
    }

    /// 这一块在这一判法单元指纹下的违例，按块内序号排；没有违例或没存过这一块时是空的。
    ///
    /// # Errors
    /// 违例表里的键或值长度不对、点名了没登记的编号；服务读不成。
    pub fn violations_in_block(
        &self,
        block_key: &VerdictBlockKey,
        version: &JudgingUnitsFingerprint,
    ) -> Result<Vec<StoredViolation>, VerdictStoreError> {
        let prefix = VerdictStoreKey::ViolationsOfBlockPrefix(block_key, version).to_key_bytes();
        let prefix_length = prefix.0.len();
        let mut violations = Vec::new();
        for entry in self.entries_with_key_prefix_kept_in_or_out(prefix) {
            let (key_bytes, number_bytes) = entry?;
            let offset_bytes =
                <[u8; STATE_OFFSET_IN_BLOCK_BYTES]>::try_from(&key_bytes[prefix_length..]);
            let (Ok(offset_bytes), Some(stored_number)) =
                (offset_bytes, stored_verdict_vector_number(&number_bytes))
            else {
                return Err(stored_bytes_mismatch(
                    VerdictStoreTable::Violations,
                    &key_bytes,
                    "违例表里块键之后不是 8 字节的块内序号，或值不是 4 字节的编号",
                ));
            };
            violations.push(StoredViolation {
                offset_in_block: StateOffsetInBlock(u64::from_be_bytes(offset_bytes)),
                verdict_vector_number: self.registered_verdict_vector_number(
                    VerdictStoreTable::Violations,
                    &key_bytes,
                    stored_number,
                )?,
            });
        }
        Ok(violations)
    }

    /// 录入一块：这一块要核 `state_count` 个状态。同一块录两次、状态数相同不算错；不同是调用方的 bug。
    ///
    /// # Errors
    /// 服务读写不成；同一个块键已录过、状态数不同。
    pub fn record_block(
        &mut self,
        block_key: &VerdictBlockKey,
        state_count: u64,
    ) -> Result<(), VerdictStoreError> {
        let key_bytes = VerdictStoreKey::RecordedBlock(block_key).to_key_bytes();
        if let Some(stored) = self.read_value_kept_in_or_out(&key_bytes.0)? {
            if stored == state_count.to_be_bytes() {
                return Ok(());
            }
            return Err(VerdictStoreError::DifferentVerdictsUnderAStoredBlockKeyNotDecidedInTheFirstVersion { block_key: block_key.clone() });
        }
        let mut batch = LibraryWriteBatch::default();
        batch.put(&key_bytes.0, state_count.to_be_bytes());
        self.write_batch_keeping_the_kept_out_rows_in_this_process(batch)
    }

    /// 录入分片的段的一块（C790）：同 [`Self::record_block`]，只是录过、状态数不同、而这一块在判定块表、结论块表、核对表里一行都没有
    /// （上一趟录了这一块、没判完就停了，两趟之间片大小改了，这一块的长度跟着变）时照这一趟的状态数改写；有判过的行还是拒。
    ///
    /// # Errors
    /// 服务读写不成；同一个块键录过、状态数不同、这一块已经有判过的行。
    pub fn record_block_of_a_slice(
        &mut self,
        block_key: &VerdictBlockKey,
        state_count: u64,
    ) -> Result<(), VerdictStoreError> {
        let key_bytes = VerdictStoreKey::RecordedBlock(block_key).to_key_bytes();
        if let Some(stored) = self.read_value_kept_in_or_out(&key_bytes.0)? {
            if stored == state_count.to_be_bytes() {
                return Ok(());
            }
            if self.block_has_a_judged_row(block_key)? {
                return Err(VerdictStoreError::DifferentVerdictsUnderAStoredBlockKeyNotDecidedInTheFirstVersion { block_key: block_key.clone() });
            }
        }
        self.put_one_synced(
            &VerdictStoreKey::RecordedBlock(block_key),
            &state_count.to_be_bytes(),
        )
    }

    /// 这一块在判定块表、结论块表、核对表里有没有任何一行（哪一判法单元指纹、哪一种结论、哪一份核对代码都算）。结论块看块属性表
    /// （与结论块同一批写、键的后缀相同、值小），各表只问「这个前缀下有没有一行」（[`Self::has_an_entry_with_key_prefix_kept_in_or_out`]），不读整块。
    ///
    /// # Errors
    /// 服务读不成。
    fn block_has_a_judged_row(
        &self,
        block_key: &VerdictBlockKey,
    ) -> Result<bool, VerdictStoreError> {
        for table in [
            VerdictStoreTable::VerdictBlocks,
            VerdictStoreTable::ConclusionBlockAttributes,
            VerdictStoreTable::VerifierVerdicts,
        ] {
            let prefix = VerdictStoreKey::RowsOfABlockPrefix(table, block_key).to_key_bytes();
            if self.has_an_entry_with_key_prefix_kept_in_or_out(prefix)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// 录入表里这一块要核几个状态；没录过交回 None（分片的段从 0 起逐块问库时按它找下一块的起点）。
    ///
    /// # Errors
    /// 服务读不成；值不是 8 字节。
    pub fn recorded_state_count_of_block(
        &self,
        block_key: &VerdictBlockKey,
    ) -> Result<Option<u64>, VerdictStoreError> {
        let key_bytes = VerdictStoreKey::RecordedBlock(block_key).to_key_bytes();
        let Some(stored) = self.read_value_kept_in_or_out(&key_bytes.0)? else {
            return Ok(None);
        };
        <[u8; 8]>::try_from(stored.as_slice())
            .map(|bytes| Some(u64::from_be_bytes(bytes)))
            .map_err(|_| {
                stored_bytes_mismatch(
                    VerdictStoreTable::RecordedBlocks,
                    &key_bytes.0,
                    "状态数不是 8 字节",
                )
            })
    }

    /// 录入表里的全部块与各自的状态数，按键序。对账用：录了而判定块表里没有同一个键的，就是还没核完的。值是 8 字节，整表逐帧扫、只留块键与状态数。
    ///
    /// # Errors
    /// 服务读不成；键或值字节认不回来。
    pub fn recorded_blocks(&self) -> Result<Vec<(VerdictBlockKey, u64)>, VerdictStoreError> {
        let mut found = Vec::new();
        self.visit_rows_of_a_block_table(
            VerdictStoreTable::RecordedBlocks,
            &mut |block_key, suffix, value| {
                if !suffix.is_empty() {
                    return Err(stored_bytes_mismatch(
                        VerdictStoreTable::RecordedBlocks,
                        suffix,
                        "录入表的键在块键之后不该有别的字节",
                    ));
                }
                let count = <[u8; 8]>::try_from(value)
                    .map(u64::from_be_bytes)
                    .map_err(|_| {
                        stored_bytes_mismatch(
                            VerdictStoreTable::RecordedBlocks,
                            value,
                            "状态数不是 8 字节",
                        )
                    })?;
                found.push((block_key, count));
                Ok(())
            },
        )?;
        Ok(found)
    }

    /// 判定块表里的全部（块键，判法单元指纹），按键序：同一块在几个版本下判过就有几行。只留键：服务的前缀扫连值一起回（协议没有只回键的扫），
    /// 逐帧扫、逐行丢掉值，内存只随块数涨；传的字节随整张表涨（这张表只有测试的一次性库在读，产品的对账走结论那几张）。
    ///
    /// # Errors
    /// 服务读不成；键字节认不回来。
    pub fn stored_verdict_block_keys(
        &self,
    ) -> Result<Vec<(VerdictBlockKey, JudgingUnitsFingerprint)>, VerdictStoreError> {
        let mut found = Vec::new();
        self.visit_rows_of_a_block_table(
            VerdictStoreTable::VerdictBlocks,
            &mut |block_key, suffix, _value| {
                let version = <[u8; SHA256_DIGEST_BYTES]>::try_from(suffix)
                    .map(JudgingUnitsFingerprint)
                    .map_err(|_| {
                        stored_bytes_mismatch(
                            VerdictStoreTable::VerdictBlocks,
                            suffix,
                            "判定块的键在块键之后要正好是 32 字节的判法单元指纹",
                        )
                    })?;
                found.push((block_key, version));
                Ok(())
            },
        )?;
        Ok(found)
    }

    /// 判法单元组成清单表登一把键的组成清单（随键落盘：库对不上时照它点名变了的单元）。同一把再登照写。
    /// 组成清单是空的不写：它的文字是空串，组成清单表不是判定向量那两张表，空值在 prune-server 里是删除记号（`an_empty_value_may_be_put_under_the_key`）；
    /// 读回时没登过（[`Self::read_judging_units_composition`] 交 `None`）与空清单点名的变了的单元相同，都是一个都没有。
    ///
    /// # Errors
    /// 服务写不成。
    pub fn store_judging_units_composition(
        &mut self,
        key: &JudgingUnitsKey,
    ) -> Result<(), VerdictStoreError> {
        if key.composition().is_empty() {
            return Ok(());
        }
        self.put_one_synced(
            &VerdictStoreKey::JudgingUnitsComposition(key.fingerprint()),
            key.composition_text().as_bytes(),
        )
    }

    /// 一把判法单元指纹登过的组成清单；没登过交回 None。
    ///
    /// # Errors
    /// 服务读不成；值认不回组成清单。
    pub fn read_judging_units_composition(
        &self,
        fingerprint: &[u8; SHA256_DIGEST_BYTES],
    ) -> Result<Option<Vec<JudgingUnitFingerprint>>, VerdictStoreError> {
        let key_bytes = VerdictStoreKey::JudgingUnitsComposition(fingerprint).to_key_bytes();
        let Some(stored) = self.read_value_kept_in_or_out(&key_bytes.0)? else {
            return Ok(None);
        };
        JudgingUnitsKey::composition_from_text(&String::from_utf8_lossy(&stored))
            .map(Some)
            .ok_or_else(|| {
                stored_bytes_mismatch(
                    VerdictStoreTable::JudgingUnitsCompositions,
                    &key_bytes.0,
                    "组成清单认不回来",
                )
            })
    }

    /// 判法单元组成清单表里登过的每一把指纹，按指纹字节排。
    ///
    /// # Errors
    /// 服务读不成；键不是 32 字节。
    pub fn judging_units_fingerprints(
        &self,
    ) -> Result<Vec<[u8; SHA256_DIGEST_BYTES]>, VerdictStoreError> {
        let prefix = VerdictStoreKey::WholeTablePrefix(VerdictStoreTable::JudgingUnitsCompositions)
            .to_key_bytes();
        let mut found = Vec::new();
        self.visit_entries_with_key_prefix_kept_in_or_out(prefix, &mut |(
            key_bytes,
            _value_bytes,
        )| {
            found.push(
                <[u8; SHA256_DIGEST_BYTES]>::try_from(&key_bytes[KEY_TAG_BYTES..]).map_err(
                    |_| {
                        stored_bytes_mismatch(
                            VerdictStoreTable::JudgingUnitsCompositions,
                            &key_bytes,
                            "组成清单表的键要是 32 字节",
                        )
                    },
                )?,
            );
            Ok(())
        })?;
        Ok(found)
    }
}

impl VerdictStore {
    fn lock_the_verdict_vectors_fetched_after_opening(
        &self,
    ) -> std::sync::MutexGuard<'_, BTreeMap<u32, &'static EncodedVerdictVector>> {
        self.verdict_vectors_fetched_after_opening
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// 这一份认得的判定向量编号有几个：按编号的内存表连同开库之后从 prune-server 取来的（后者从内存表末尾起连续）。编号由这一份自己给的库
    /// 没有取来的，就是内存表的长度。
    pub(super) fn known_verdict_vector_count(&self) -> usize {
        self.verdict_vector_by_number.len()
            + self.lock_the_verdict_vectors_fetched_after_opening().len()
    }

    /// 读到的编号这一份还不认得的，从 prune-server 取来（编号由服务统一给：别的进程在这一份开库之后登记的，这一份没见过）：从认得的末尾起
    /// 连续取到最大的那个，成批点读（[`PruneServerStorage::read_values`]：一帧几千个键，回应越过帧长上限由容错客户端拆两半）。
    /// 服务的按编号表里缺了的那一号起不再取（认得的编号要从内存表末尾起连续）、不报错：缺的是块里引的编号没登记过，错在引它的那一块，
    /// 由调用方照那一块的表与键报不合布局（[`Self::registered_verdict_vector_numbers_of_a_stored_block`]、[`Self::registered_verdict_vector_number`]）。
    ///
    /// # Errors
    /// 读不成。
    fn fetch_the_verdict_vectors_of_unknown_numbers(
        &self,
        stored_numbers: &[u32],
    ) -> Result<(), VerdictStoreError> {
        let Some(largest) = stored_numbers.iter().copied().max() else {
            return Ok(());
        };
        let mut fetched = self.lock_the_verdict_vectors_fetched_after_opening();
        let known_count = self.verdict_vector_by_number.len() + fetched.len();
        if VerdictVectorNumber(largest).position_in_the_registration_order() < known_count {
            return Ok(());
        }
        let first_unknown = u32::try_from(known_count).expect(
            "认得的个数不超过最大的那个编号：编号是 u32，最大的编号没认得，认得的个数装得进 u32",
        );
        let numbers: Vec<VerdictVectorNumber> =
            (first_unknown..=largest).map(VerdictVectorNumber).collect();
        let keys: Vec<VerdictStoreKeyBytes> = numbers
            .iter()
            .map(|number| VerdictStoreKey::VerdictVectorContentByNumber(*number).to_key_bytes())
            .collect();
        let key_slices: Vec<&[u8]> = keys.iter().map(|key| key.0.as_slice()).collect();
        let values = self.service.read_values(&key_slices)?;
        for (number, value) in numbers.iter().zip(values) {
            let Some(vector_bytes) = value else {
                break;
            };
            let leaked: &'static EncodedVerdictVector =
                Box::leak(Box::new(EncodedVerdictVector(vector_bytes)));
            fetched.insert(number.as_u32(), leaked);
        }
        Ok(())
    }

    /// 开库之后从 prune-server 取来的判定向量并进内存表（编号依次接在末尾）。
    ///
    /// # Errors
    /// 取来的编号接不上、或那一种判定向量已经有别的编号（服务的判定向量表不是一种一个编号）。
    fn take_in_the_verdict_vectors_fetched_after_opening(
        &mut self,
    ) -> Result<(), VerdictStoreError> {
        let fetched = std::mem::take(
            self.verdict_vectors_fetched_after_opening
                .get_mut()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        );
        for (stored_number, verdict_vector) in fetched {
            self.append_a_verdict_vector_known_by_its_number(
                VerdictVectorNumber(stored_number),
                verdict_vector.clone(),
            )?;
        }
        Ok(())
    }

    /// 编号由服务统一给时，把服务里这个编号的判定向量接到内存表末尾：编号要正是下一个，那一种判定向量不能已有别的编号。
    ///
    /// # Errors
    /// 编号不是下一个；那一种已有别的编号。
    fn append_a_verdict_vector_known_by_its_number(
        &mut self,
        number: VerdictVectorNumber,
        verdict_vector: EncodedVerdictVector,
    ) -> Result<(), VerdictStoreError> {
        let is_the_next_number =
            VerdictVectorNumber::next_after_registered_count(self.verdict_vector_by_number.len())
                == Some(number);
        let has_another_number = self
            .verdict_vector_number_by_content
            .get(&verdict_vector.0)
            .is_some_and(|known_number| *known_number != number);
        if !is_the_next_number || has_another_number {
            return Err(stored_bytes_mismatch(
                VerdictStoreTable::VerdictVectorContentByNumber,
                &VerdictStoreKey::VerdictVectorContentByNumber(number)
                    .to_key_bytes()
                    .0,
                "prune-server 的判定向量表不是从 0 起连续、一种一个编号",
            ));
        }
        self.verdict_vector_number_by_content
            .insert(verdict_vector.0.clone(), number);
        self.verdict_vector_by_number.push(verdict_vector);
        Ok(())
    }

    /// 在 prune-server 里登记一种这一份还不认得的判定向量：从内存表末尾的下一个编号起，认领那个编号的按编号行——认到了就把两张判定向量表的
    /// 行同一帧写进服务（写帧确认时认领释放）；服务答已有就读回那一种接进内存表，正是要登记的这一种就用这个编号，不是就往下一个编号走；
    /// 别的客户端认领着就隔一会再问（它写完答已有，它死了租期一过放出来）。几个进程、两台同时登记同一种，只有一个认到，别的读回同一个编号。
    ///
    /// # Errors
    /// 读写、认领不成；等过 [`PRUNE_SERVER_NUMBER_CLAIM_WAIT_LIMIT`] 还被别人认领着；2^32 种都登记过了；服务的判定向量表不连续。
    fn register_verdict_vector_in_the_prune_server(
        &mut self,
        authority: &PruneServerStorage,
        verdict_vector: &EncodedVerdictVector,
    ) -> Result<VerdictVectorNumber, VerdictStoreError> {
        let content_key =
            VerdictStoreKey::VerdictVectorNumberByContent(verdict_vector).to_key_bytes();
        let waiting_started = std::time::Instant::now();
        loop {
            let Some(number) = VerdictVectorNumber::next_after_registered_count(
                self.verdict_vector_by_number.len(),
            ) else {
                return Err(VerdictStoreError::EveryU32VerdictVectorNumberIsAlreadyRegistered);
            };
            let number_key = VerdictStoreKey::VerdictVectorContentByNumber(number).to_key_bytes();
            match authority.claim(&number_key.0, "认领判定向量编号")? {
                ClaimOutcome::Claimed => {
                    // 经 write_batch 写：绿的判定向量是空字节，按编号那一行的值为空，只有这张表许写空值，别的表照旧拒
                    let mut registration = LibraryWriteBatch::default();
                    registration.put(&content_key.0, number.to_stored_bytes());
                    registration.put(&number_key.0, &verdict_vector.0);
                    authority.write_batch(registration)?;
                    self.append_a_verdict_vector_known_by_its_number(
                        number,
                        verdict_vector.clone(),
                    )?;
                    return Ok(number);
                }
                ClaimOutcome::AlreadyConcluded => {
                    let Some(stored_vector_bytes) = authority
                        .read_values(&[number_key.0.as_slice()])?
                        .pop()
                        .flatten()
                    else {
                        return Err(stored_bytes_mismatch(
                            VerdictStoreTable::VerdictVectorContentByNumber,
                            &number_key.0,
                            "prune-server 答这个判定向量编号已经登记过，读回来却没有",
                        ));
                    };
                    let is_this_verdict_vector = stored_vector_bytes == verdict_vector.0;
                    self.append_a_verdict_vector_known_by_its_number(
                        number,
                        EncodedVerdictVector(stored_vector_bytes),
                    )?;
                    if is_this_verdict_vector {
                        return Ok(number);
                    }
                }
                ClaimOutcome::HeldByOther { .. } => {
                    if waiting_started.elapsed() >= PRUNE_SERVER_NUMBER_CLAIM_WAIT_LIMIT {
                        return Err(authority.failed(
                            PRUNE_SERVER_UNAVAILABLE_EXIT_CODE,
                            format!(
                                "判定向量编号 {} 被别的客户端认领着，等过 {} 秒还没放",
                                number.as_u32(),
                                PRUNE_SERVER_NUMBER_CLAIM_WAIT_LIMIT.as_secs()
                            ),
                        ));
                    }
                    std::thread::sleep(PRUNE_SERVER_NUMBER_CLAIM_RETRY_INTERVAL);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests;

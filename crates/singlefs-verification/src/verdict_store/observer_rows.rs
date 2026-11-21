//! 观察者那几张表（`14`–`17`、`1A`、`1B`）：观察者基底、看过的恢复镜像、读集见证与读集、观察者录入块、镜像上的池级 checker。

use super::*;

/// 观察者去重一个桶里的一条读集见证行（键是桶键与见证摘要）：见证读的那组段的摘要（读集行的键）、它看出的那张镜像的去重键。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObserverReadSetWitnessRow {
    pub witness_digest: [u8; SHA256_DIGEST_BYTES],
    pub read_set_digest: [u8; SHA256_DIGEST_BYTES],
    pub observed_image_key: [u8; SHA256_DIGEST_BYTES],
}

/// 读集见证行的值：读集摘要 ‖ 镜像的去重键。
const OBSERVER_READ_SET_WITNESS_ROW_VALUE_BYTES: usize = 2 * SHA256_DIGEST_BYTES;

/// 观察者见证那几批的测试开关不拒的时候的值（[`VerdictStore::refuse_the_observer_witness_batches_after`]）。
pub(super) const OBSERVER_WITNESS_BATCHES_ARE_NEVER_REFUSED: u64 = u64::MAX;

impl VerdictStore {
    /// 观察者录入段写下一条流的崩溃镜像共用的基底（键是它的摘要；同一摘要再写就是同一份内容，照写）。
    ///
    /// # Errors
    /// 服务写不成。
    pub fn store_observer_foundation(
        &mut self,
        digest: &[u8; 32],
        bytes: &[u8],
    ) -> Result<(), VerdictStoreError> {
        self.put_one_synced(&VerdictStoreKey::ObserverFoundation(digest), bytes)
    }

    /// 读观察者基底的字节；没写过交回 None。
    ///
    /// # Errors
    /// 服务读不成。
    pub fn read_observer_foundation(
        &self,
        digest: &[u8; 32],
    ) -> Result<Option<Vec<u8>>, VerdictStoreError> {
        self.read_one_stored_value(&VerdictStoreKey::ObserverFoundation(digest))
    }

    /// 观察者录入段写下一批看过的恢复镜像（每条：去重键、它的字节），一批原子地落盘：被打断时这一批要么全在、要么全不在，
    /// 重跑同一条命令只补没写的。
    ///
    /// # Errors
    /// 服务写不成。
    pub fn store_observed_images(
        &mut self,
        records: &[([u8; 32], Vec<u8>)],
    ) -> Result<(), VerdictStoreError> {
        if records.is_empty() {
            return Ok(());
        }
        let mut batch = LibraryWriteBatch::default();
        for (deduplication_key, bytes) in records {
            batch.put(
                VerdictStoreKey::ObservedImage(deduplication_key)
                    .to_key_bytes()
                    .0,
                bytes,
            );
        }
        self.write_batch_keeping_the_kept_out_rows_in_this_process(batch)
    }

    /// 读一张看过的恢复镜像的字节；没看过交回 None。
    ///
    /// # Errors
    /// 服务读不成。
    pub fn read_observed_image(
        &self,
        deduplication_key: &[u8; 32],
    ) -> Result<Option<Vec<u8>>, VerdictStoreError> {
        self.read_one_stored_value(&VerdictStoreKey::ObservedImage(deduplication_key))
    }

    /// 读几张看过的恢复镜像的字节（[`Self::read_observed_image`] 的成批版：一帧成批点读，不逐张一帧），交回的次序与 `deduplication_keys` 相同，
    /// 没看过的那一张是 None。
    ///
    /// # Errors
    /// 服务读不成。
    pub fn read_observed_images_in_one_read(
        &self,
        deduplication_keys: &[[u8; 32]],
    ) -> Result<Vec<Option<Vec<u8>>>, VerdictStoreError> {
        if deduplication_keys.is_empty() {
            return Ok(Vec::new());
        }
        let keys: Vec<VerdictStoreKeyBytes> = deduplication_keys
            .iter()
            .map(|deduplication_key| {
                VerdictStoreKey::ObservedImage(deduplication_key).to_key_bytes()
            })
            .collect();
        let key_slices: Vec<&[u8]> = keys.iter().map(|key| key.0.as_slice()).collect();
        self.read_values_kept_in_or_out(&key_slices)
    }

    /// 一个桶的全部读集见证行，按见证摘要的字节序；这个桶还没有见证交回空的。
    ///
    /// # Errors
    /// 键不是桶键之后接 32 字节见证摘要、值不是 64 字节（[`VerdictStoreError::StoredBytesDoNotMatchTheLayout`]）；服务读不成。
    pub fn read_observer_read_set_witnesses_of_the_bucket(
        &self,
        bucket_key: &[u8; 32],
    ) -> Result<Vec<ObserverReadSetWitnessRow>, VerdictStoreError> {
        let prefix =
            VerdictStoreKey::ObserverReadSetWitnessRowsOfABucketPrefix(bucket_key).to_key_bytes();
        let prefix_length = prefix.0.len();
        let mut witnesses = Vec::new();
        for entry in self.entries_with_key_prefix_kept_in_or_out(prefix) {
            let (key_bytes, value_bytes) = entry?;
            let witness_digest = <[u8; SHA256_DIGEST_BYTES]>::try_from(&key_bytes[prefix_length..]);
            let (Ok(witness_digest), true) = (
                witness_digest,
                value_bytes.len() == OBSERVER_READ_SET_WITNESS_ROW_VALUE_BYTES,
            ) else {
                return Err(stored_bytes_mismatch(
                    VerdictStoreTable::ObserverReadSetWitnessRows,
                    &key_bytes,
                    "读集见证行的键里桶键之后不是 32 字节的见证摘要，或值不是读集摘要与镜像键各 32 字节",
                ));
            };
            let (read_set_digest, observed_image_key) = value_bytes.split_at(SHA256_DIGEST_BYTES);
            witnesses.push(ObserverReadSetWitnessRow {
                witness_digest,
                read_set_digest: read_set_digest
                    .try_into()
                    .expect("值的长度上面核过是两份摘要"),
                observed_image_key: observed_image_key
                    .try_into()
                    .expect("值的长度上面核过是两份摘要"),
            });
        }
        Ok(witnesses)
    }

    /// 读一组读集行的字节；没写过交回 None。
    ///
    /// # Errors
    /// 服务读不成。
    pub fn read_observer_read_set(
        &self,
        read_set_digest: &[u8; 32],
    ) -> Result<Option<Vec<u8>>, VerdictStoreError> {
        self.read_one_stored_value(&VerdictStoreKey::ObserverReadSet(read_set_digest))
    }

    /// 观察者真跑了一张镜像：这一条见证行、这组读集在这个桶里头一次出现时的读集行（`read_set_bytes_new_in_the_bucket`，
    /// 键是 `witness.read_set_digest`）、那张镜像的记录（键是 `witness.observed_image_key`），三样一批原子地落盘——见证在、它引的读集与镜像必在，
    /// 被打断后重跑找到见证就读得到读集与镜像。写下的字节只随这一条见证与它的镜像（读集行只在这组读集头一次出现时写），与桶里已有几条见证无关。
    ///
    /// # Errors
    /// 服务写不成；测试开关拒了这一批（[`VerdictStoreError::ObserverWitnessBatchRefusedByTheTestSwitch`]，一行都没写）。
    pub fn store_observed_image_with_its_read_set_witness(
        &mut self,
        bucket_key: &[u8; 32],
        witness: &ObserverReadSetWitnessRow,
        read_set_bytes_new_in_the_bucket: Option<&[u8]>,
        image_bytes: &[u8],
    ) -> Result<(), VerdictStoreError> {
        let mut batch = LibraryWriteBatch::default();
        let mut witness_row_value = witness.read_set_digest.to_vec();
        witness_row_value.extend_from_slice(&witness.observed_image_key);
        batch.put(
            VerdictStoreKey::ObserverReadSetWitnessRow(bucket_key, &witness.witness_digest)
                .to_key_bytes()
                .0,
            witness_row_value,
        );
        if let Some(read_set_bytes) = read_set_bytes_new_in_the_bucket {
            batch.put(
                VerdictStoreKey::ObserverReadSet(&witness.read_set_digest)
                    .to_key_bytes()
                    .0,
                read_set_bytes,
            );
        }
        batch.put(
            VerdictStoreKey::ObservedImage(&witness.observed_image_key)
                .to_key_bytes()
                .0,
            image_bytes,
        );
        self.write_an_observer_witness_batch(batch)
    }

    /// 观察者见证那几批经这里写：测试开关（[`Self::refuse_the_observer_witness_batches_after`]）放过去的才写。
    fn write_an_observer_witness_batch(
        &self,
        batch: LibraryWriteBatch,
    ) -> Result<(), VerdictStoreError> {
        let still_written = self
            .observer_witness_batches_still_written
            .load(std::sync::atomic::Ordering::SeqCst);
        match still_written {
            0 => return Err(VerdictStoreError::ObserverWitnessBatchRefusedByTheTestSwitch),
            OBSERVER_WITNESS_BATCHES_ARE_NEVER_REFUSED => {}
            counted_down => self
                .observer_witness_batches_still_written
                .store(counted_down - 1, std::sync::atomic::Ordering::SeqCst),
        }
        self.write_batch_keeping_the_kept_out_rows_in_this_process(batch)
    }

    /// 观察者录入段写下一块（每个状态落在哪一份观察结果上、按发布分的态数）。同一键再写就盖掉上一趟的。
    ///
    /// # Errors
    /// 服务写不成。
    pub fn store_observer_block(
        &mut self,
        block_key: &VerdictBlockKey,
        kind: &ConclusionKind,
        observer_version: &ConclusionVersion,
        bytes: &[u8],
    ) -> Result<(), VerdictStoreError> {
        self.put_one_synced(
            &VerdictStoreKey::ObserverBlock(block_key, kind, observer_version),
            bytes,
        )
    }

    /// 读观察者录入段的一块；录入段没录过交回 None。
    ///
    /// # Errors
    /// 服务读不成。
    pub fn read_observer_block(
        &self,
        block_key: &VerdictBlockKey,
        kind: &ConclusionKind,
        observer_version: &ConclusionVersion,
    ) -> Result<Option<Vec<u8>>, VerdictStoreError> {
        self.read_one_stored_value(&VerdictStoreKey::ObserverBlock(
            block_key,
            kind,
            observer_version,
        ))
    }

    /// 判段写下一批观察者交回的镜像上池级 checker 的结论（同一个判器种类与版本），一批原子地落盘。
    ///
    /// # Errors
    /// 服务写不成。
    pub fn store_observed_image_judgements(
        &mut self,
        kind: &ConclusionKind,
        judging_units_fingerprint: &ConclusionVersion,
        records: &[([u8; 32], Vec<u8>)],
    ) -> Result<(), VerdictStoreError> {
        if records.is_empty() {
            return Ok(());
        }
        let mut batch = LibraryWriteBatch::default();
        for (deduplication_key, bytes) in records {
            batch.put(
                VerdictStoreKey::ObservedImageJudgement(
                    deduplication_key,
                    kind,
                    judging_units_fingerprint,
                )
                .to_key_bytes()
                .0,
                bytes,
            );
        }
        self.write_batch_keeping_the_kept_out_rows_in_this_process(batch)
    }

    /// 读一张观察者交回的镜像在这个判器种类与版本下的池级 checker 结论；没判过交回 None。
    ///
    /// # Errors
    /// 服务读不成。
    pub fn read_observed_image_judgement(
        &self,
        deduplication_key: &[u8; 32],
        kind: &ConclusionKind,
        judging_units_fingerprint: &ConclusionVersion,
    ) -> Result<Option<Vec<u8>>, VerdictStoreError> {
        self.read_one_stored_value(&VerdictStoreKey::ObservedImageJudgement(
            deduplication_key,
            kind,
            judging_units_fingerprint,
        ))
    }

    /// 读几张观察者交回的镜像在这个判器种类与版本下的池级 checker 结论（[`Self::read_observed_image_judgement`] 的成批版：一帧成批点读），
    /// 交回的次序与 `deduplication_keys` 相同，没判过的那一张是 None。
    ///
    /// # Errors
    /// 服务读不成。
    pub fn read_observed_image_judgements_in_one_read(
        &self,
        deduplication_keys: &[[u8; 32]],
        kind: &ConclusionKind,
        judging_units_fingerprint: &ConclusionVersion,
    ) -> Result<Vec<Option<Vec<u8>>>, VerdictStoreError> {
        if deduplication_keys.is_empty() {
            return Ok(Vec::new());
        }
        let keys: Vec<VerdictStoreKeyBytes> = deduplication_keys
            .iter()
            .map(|deduplication_key| {
                VerdictStoreKey::ObservedImageJudgement(
                    deduplication_key,
                    kind,
                    judging_units_fingerprint,
                )
                .to_key_bytes()
            })
            .collect();
        let key_slices: Vec<&[u8]> = keys.iter().map(|key| key.0.as_slice()).collect();
        self.read_values_kept_in_or_out(&key_slices)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 观察者见证按行存：一个桶里添三条见证（两条共用一组读集，读集行只随头一条写）、另一个桶添一条，每添一条恰好一批；按桶键读回的
    /// 是这个桶自己的那几行、一条见证一行、按见证摘要排，引的读集行与镜像都读得到；重开库照样读得回。
    #[test]
    fn observer_read_set_witness_rows_read_back_one_row_per_witness_from_their_own_bucket() {
        let library = TestLibrary::new("observer-witness-rows");
        let bucket = [0xB1; SHA256_DIGEST_BYTES];
        let other_bucket = [0xB2; SHA256_DIGEST_BYTES];
        let shared_read_set_bytes = b"read set shared by two witnesses".to_vec();
        let other_read_set_bytes = b"read set of one witness".to_vec();
        let witness_numbered = |number: u8, read_set_digest: u8| ObserverReadSetWitnessRow {
            witness_digest: [number; SHA256_DIGEST_BYTES],
            read_set_digest: [read_set_digest; SHA256_DIGEST_BYTES],
            observed_image_key: [number.wrapping_add(0x80); SHA256_DIGEST_BYTES],
        };
        let witnesses_of_the_bucket = [
            witness_numbered(0x03, 0xD1),
            witness_numbered(0x01, 0xD1),
            witness_numbered(0x02, 0xD2),
        ];
        let witness_of_the_other_bucket = witness_numbered(0x04, 0xD1);
        {
            let mut store = library.open();
            let batches_before = store.synced_write_tally().batches_written;
            store
                .store_observed_image_with_its_read_set_witness(
                    &bucket,
                    &witnesses_of_the_bucket[0],
                    Some(&shared_read_set_bytes),
                    b"image of witness three",
                )
                .expect("写见证");
            store
                .store_observed_image_with_its_read_set_witness(
                    &bucket,
                    &witnesses_of_the_bucket[1],
                    None,
                    b"image of witness one",
                )
                .expect("写见证（读集行已在）");
            store
                .store_observed_image_with_its_read_set_witness(
                    &bucket,
                    &witnesses_of_the_bucket[2],
                    Some(&other_read_set_bytes),
                    b"image of witness two",
                )
                .expect("写见证");
            store
                .store_observed_image_with_its_read_set_witness(
                    &other_bucket,
                    &witness_of_the_other_bucket,
                    Some(&shared_read_set_bytes),
                    b"image of witness four",
                )
                .expect("另一个桶写见证");
            assert_eq!(
                store.synced_write_tally().batches_written - batches_before,
                4,
                "每添一条见证恰好一批（见证行、读集行、镜像同一批）"
            );
        }
        let store = library.open();
        let read_back = store
            .read_observer_read_set_witnesses_of_the_bucket(&bucket)
            .expect("读桶的见证");
        assert_eq!(
            read_back,
            vec![
                witnesses_of_the_bucket[1],
                witnesses_of_the_bucket[2],
                witnesses_of_the_bucket[0]
            ],
            "这个桶的三条见证各一行、按见证摘要排，另一个桶的那一条不在里面"
        );
        assert_eq!(
            store
                .read_observer_read_set_witnesses_of_the_bucket(&other_bucket)
                .expect("读另一个桶"),
            vec![witness_of_the_other_bucket]
        );
        assert_eq!(
            store
                .read_observer_read_set_witnesses_of_the_bucket(&[0xB3; SHA256_DIGEST_BYTES])
                .expect("读没有见证的桶"),
            Vec::new()
        );
        assert_eq!(
            store
                .read_observer_read_set(&[0xD1; SHA256_DIGEST_BYTES])
                .expect("读读集行"),
            Some(shared_read_set_bytes)
        );
        assert_eq!(
            store
                .read_observed_image(&witnesses_of_the_bucket[1].observed_image_key)
                .expect("读镜像"),
            Some(b"image of witness one".to_vec())
        );
    }
}

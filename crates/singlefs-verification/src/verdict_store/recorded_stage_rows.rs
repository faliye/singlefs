//! 两段之间交接的几张表与路径索引：录下的流程、一趟的清单、GPU 判表、给定状态表、事实表，按路径记的身份。

use super::*;

impl VerdictStore {
    /// 路径索引登一条：这条路径上的节点是这个身份。同一路径再登、身份相同不算错，什么都不写（与 [`Self::record_block`] 一样先比再写）；
    /// 身份不同也照写（同名不同数据的分叉在路径这一级并成一段，最后写的那个留在索引里；两边都在块表里，一个不丢）。
    ///
    /// # Errors
    /// 服务读写不成。
    pub fn record_path(&mut self, path: &str, identity: &str) -> Result<(), VerdictStoreError> {
        let key_bytes = VerdictStoreKey::Path(path).to_key_bytes();
        let stored_identity = self.read_value_kept_in_or_out(&key_bytes.0)?;
        if stored_identity.as_deref() == Some(identity.as_bytes()) {
            return Ok(());
        }
        let mut batch = LibraryWriteBatch::default();
        batch.put(&key_bytes.0, identity.as_bytes());
        self.write_batch_keeping_the_kept_out_rows_in_this_process(batch)
    }

    /// 这条路径上登过的身份；没登过交回 None。
    ///
    /// # Errors
    /// 服务读不成；值不是 UTF-8。
    pub fn identity_of_path(&self, path: &str) -> Result<Option<String>, VerdictStoreError> {
        let key_bytes = VerdictStoreKey::Path(path).to_key_bytes();
        let Some(value) = self.read_value_kept_in_or_out(&key_bytes.0)? else {
            return Ok(None);
        };
        String::from_utf8(value.clone()).map(Some).map_err(|_| {
            stored_bytes_mismatch(VerdictStoreTable::PathIndex, &value, "身份不是 UTF-8")
        })
    }

    /// 路径索引里以这段前缀起头的全部（路径，身份），按键序：给空串就是整棵树；给 `<某节点路径>/` 就是它的子树。服务那一路逐帧扫、逐行认
    /// （[`Self::visit_entries_with_key_prefix_kept_in_or_out`]），交回的只有路径与身份的文字。
    ///
    /// # Errors
    /// 服务读不成；键或值不是 UTF-8。
    pub fn paths_with_prefix(
        &self,
        prefix: &str,
    ) -> Result<Vec<(String, String)>, VerdictStoreError> {
        let prefix_bytes = VerdictStoreKey::Path(prefix).to_key_bytes();
        let mut found = Vec::new();
        self.visit_entries_with_key_prefix_kept_in_or_out(prefix_bytes, &mut |entry| {
            let (key_bytes, value_bytes) = entry;
            let path = String::from_utf8(key_bytes[1..].to_vec()).map_err(|_| {
                stored_bytes_mismatch(VerdictStoreTable::PathIndex, &key_bytes, "路径不是 UTF-8")
            })?;
            let identity = String::from_utf8(value_bytes.to_vec()).map_err(|_| {
                stored_bytes_mismatch(VerdictStoreTable::PathIndex, &value_bytes, "身份不是 UTF-8")
            })?;
            found.push((path, identity));
            Ok(())
        })?;
        Ok(found)
    }

    /// 事实表登一条流（第 ① 段）：同一摘要再登就是同一份内容，照写不算错。
    ///
    /// # Errors
    /// 服务写不成。
    pub fn store_flow_facts(
        &mut self,
        facts_digest: &[u8; 32],
        facts_bytes: &[u8],
    ) -> Result<(), VerdictStoreError> {
        let key_bytes = VerdictStoreKey::FlowFacts(facts_digest).to_key_bytes();
        let mut batch = LibraryWriteBatch::default();
        batch.put(&key_bytes.0, facts_bytes);
        self.write_batch_keeping_the_kept_out_rows_in_this_process(batch)
    }

    /// 读一条流的事实；没登过交回 None。
    ///
    /// # Errors
    /// 服务读不成。
    pub fn read_flow_facts(
        &self,
        facts_digest: &[u8; 32],
    ) -> Result<Option<Vec<u8>>, VerdictStoreError> {
        let key_bytes = VerdictStoreKey::FlowFacts(facts_digest).to_key_bytes();
        self.read_value_kept_in_or_out(&key_bytes.0)
    }

    /// 录入段写下一条流程（`crash_amplification::RecordedFlow` 的字节，键是它的摘要）。同一摘要再写就是同一份内容，照写。
    ///
    /// # Errors
    /// 服务写不成。
    pub fn store_recorded_flow(
        &mut self,
        digest: &[u8; 32],
        bytes: &[u8],
    ) -> Result<(), VerdictStoreError> {
        self.put_one_synced(&VerdictStoreKey::RecordedFlow(digest), bytes)
    }

    /// 读录入段写下的一条流程的字节；没写过交回 None。
    ///
    /// # Errors
    /// 服务读不成。
    pub fn read_recorded_flow(
        &self,
        digest: &[u8; 32],
    ) -> Result<Option<Vec<u8>>, VerdictStoreError> {
        self.read_one_stored_value(&VerdictStoreKey::RecordedFlow(digest))
    }

    /// 录入段登一趟的清单（键是录入键 [`RecordingKey`]：同一把键就是同一份录入输入、同一份产品代码与场景，再登照写同一份）。
    ///
    /// # Errors
    /// 服务写不成。
    pub fn store_recorded_run_manifest(
        &mut self,
        recording_key: &RecordingKey,
        bytes: &[u8],
    ) -> Result<(), VerdictStoreError> {
        self.put_one_synced(&VerdictStoreKey::RecordedRunManifest(recording_key), bytes)
    }

    /// 一趟的清单表里有几行（拒跑之后核「一份清单都没登」用）：整表逐帧扫、逐行数，不留行。
    ///
    /// # Errors
    /// 服务读不成。
    pub fn recorded_run_manifest_count(&self) -> Result<u64, VerdictStoreError> {
        let prefix = VerdictStoreKey::WholeTablePrefix(VerdictStoreTable::RecordedRunManifests)
            .to_key_bytes();
        let mut count = 0u64;
        self.visit_entries_with_key_prefix_kept_in_or_out(prefix, &mut |_entry| {
            count += 1;
            Ok(())
        })?;
        Ok(count)
    }

    /// 读录入键下登的清单；没登过交回 None。
    ///
    /// # Errors
    /// 服务读不成。
    pub fn read_recorded_run_manifest(
        &self,
        recording_key: &RecordingKey,
    ) -> Result<Option<Vec<u8>>, VerdictStoreError> {
        self.read_one_stored_value(&VerdictStoreKey::RecordedRunManifest(recording_key))
    }

    /// 录入段写下打包好的 GPU 判表（键是它的摘要）。
    ///
    /// # Errors
    /// 服务写不成。
    pub fn store_packed_judge_tables(
        &mut self,
        digest: &[u8; 32],
        bytes: &[u8],
    ) -> Result<(), VerdictStoreError> {
        self.put_one_synced(&VerdictStoreKey::PackedJudgeTables(digest), bytes)
    }

    /// 读录入段写下的 GPU 判表的字节；没写过交回 None。
    ///
    /// # Errors
    /// 服务读不成。
    pub fn read_packed_judge_tables(
        &self,
        digest: &[u8; 32],
    ) -> Result<Option<Vec<u8>>, VerdictStoreError> {
        self.read_one_stored_value(&VerdictStoreKey::PackedJudgeTables(digest))
    }

    /// 写下一张给定状态表（键是它的摘要）。
    ///
    /// # Errors
    /// 服务写不成。
    pub fn store_given_states_table(
        &mut self,
        digest: &[u8; 32],
        bytes: &[u8],
    ) -> Result<(), VerdictStoreError> {
        self.put_one_synced(&VerdictStoreKey::GivenStatesTable(digest), bytes)
    }

    /// 读一张给定状态表的字节；没写过交回 None。
    ///
    /// # Errors
    /// 服务读不成。
    pub fn read_given_states_table(
        &self,
        digest: &[u8; 32],
    ) -> Result<Option<Vec<u8>>, VerdictStoreError> {
        self.read_one_stored_value(&VerdictStoreKey::GivenStatesTable(digest))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 同一条路径再登同一个身份：什么都不写（库的序号不动）；换一个身份照写。
    #[test]
    fn recording_a_path_again_with_the_same_identity_writes_nothing() {
        let library = TestLibrary::new("path-recorded-twice");
        let mut store = library.open();
        store
            .record_path("a.unit_write.1", "f::a.unit_write.1::00")
            .expect("登路径");
        let batches_after_the_first = store.synced_write_tally().batches_written;
        store
            .record_path("a.unit_write.1", "f::a.unit_write.1::00")
            .expect("再登同一个身份");
        assert_eq!(
            store.synced_write_tally().batches_written,
            batches_after_the_first,
            "同一个身份再登不写库"
        );
        store
            .record_path("a.unit_write.1", "f::a.unit_write.1::11")
            .expect("换一个身份");
        assert!(
            store.synced_write_tally().batches_written > batches_after_the_first,
            "换了身份照写"
        );
        assert_eq!(
            store.identity_of_path("a.unit_write.1").expect("读路径"),
            Some("f::a.unit_write.1::11".to_string()),
            "最后登的身份留在索引里"
        );
    }
}

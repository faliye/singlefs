//! 在 prune-server 里开库：读回按编号的判定向量表，库的名字、全部行与往返计时。

use super::*;

/// prune-server 里那一份的判定向量表：按编号那一张（键是编号的大端 4 字节，按键升序就是按编号升序）要从 0 起连续、一种一个编号；
/// 按内容那一张由它现算（按内容那一张只给登记时查重用，开库时不整张读它）。整张按编号表一次前缀扫读回（被截断就拆前缀分着扫）；
/// 绿那一号的值为空，照一行原样算（文件头「删除记号对判定向量两张表不适用」那一段）。
///
/// # Errors
/// 扫不成；编号不连续、不是 4 字节，或同一种判定向量有两个编号。
fn load_verdict_vectors_of_the_prune_server(
    storage: &PruneServerStorage,
) -> Result<LoadedVerdictVectors, VerdictStoreError> {
    let by_number_table = VerdictStoreTable::VerdictVectorContentByNumber;
    let mut verdict_vector_by_number = Vec::new();
    let mut verdict_vector_number_by_content = HashMap::new();
    let rows = storage.rows_with_prefix(
        &VerdictStoreKey::WholeTablePrefix(by_number_table)
            .to_key_bytes()
            .0,
    )?;
    for (key_bytes, vector_bytes) in rows {
        let expected =
            VerdictVectorNumber::next_after_registered_count(verdict_vector_by_number.len());
        let is_the_next_number = key_bytes
            .get(KEY_TAG_BYTES..)
            .and_then(stored_verdict_vector_number)
            .is_some_and(|stored_number| expected == Some(VerdictVectorNumber(stored_number)));
        let Some(number) = expected.filter(|_number| is_the_next_number) else {
            return Err(stored_bytes_mismatch(
                by_number_table,
                &key_bytes,
                "prune-server 里按编号的判定向量表编号不是从 0 起连续的 4 字节",
            ));
        };
        if verdict_vector_number_by_content
            .insert(vector_bytes.to_vec(), number)
            .is_some()
        {
            return Err(stored_bytes_mismatch(
                by_number_table,
                &key_bytes,
                "prune-server 里同一种判定向量登记了两个编号",
            ));
        }
        verdict_vector_by_number.push(EncodedVerdictVector(vector_bytes.to_vec()));
    }
    Ok(LoadedVerdictVectors {
        verdict_vector_by_number,
        verdict_vector_number_by_content,
    })
}

/// prune-server 那一侧的开库、编号与成批读（模块里「prune-server 那一侧」一节）。
impl VerdictStore {
    /// 开 prune-server 里那一份崩溃判定库（`service` 连的那一处、那一个键空间）：读回按编号的判定向量表，开库一行都不写。
    /// 判定向量编号由服务统一给（[`Self::register_verdict_vector`] 在服务里认领编号）。
    ///
    /// # Errors
    /// 读不成；判定向量表编号不连续或一种两个编号。
    pub fn open_the_prune_server(
        service: Arc<PruneServerStorage>,
    ) -> Result<Self, VerdictStoreError> {
        let loaded = load_verdict_vectors_of_the_prune_server(&service)?;
        Ok(Self::assembled_over_the_service(service, loaded))
    }

    /// 这一份库（这个键空间）里的全部行，按键升序（不连照判不入库的）：单测核「什么都没写」用。
    #[cfg(test)]
    pub(crate) fn every_stored_entry(&self) -> Vec<StoredEntry> {
        self.service
            .rows_with_prefix(&[])
            .expect("单测扫得了这一份库的全部行")
    }

    /// 库的名字（连的是哪一处、哪一个键空间；只进报错与行里报的名字）。
    #[must_use]
    pub fn library_name(&self) -> &str {
        &self.library_name
    }

    /// 从开库起与 prune-server 往返的纳秒数（[`PruneServerStorage::nanoseconds_with_the_client_so_far`]）：判定库锁里有多少花在往返上，
    /// 取锁与放锁时各读一次、相减。
    #[must_use]
    pub fn nanoseconds_in_round_trips_so_far(&self) -> u64 {
        self.service.nanoseconds_with_the_client_so_far()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 头一次开的一次性库（这个键空间里一行都没有）开得了，开完这个键空间里照旧一行都没有、一种判定向量都没登记：开库只读不写。
    /// 判别力（`crates/mutations.tsv`）：开库时先往库里写一行再读判定向量表，这条红。
    #[test]
    fn a_library_opened_the_first_time_writes_no_row_and_registers_no_verdict_vector() {
        let library = TestLibrary::new("opening-writes-nothing");
        let opened = library.open();
        assert_eq!(
            opened.every_stored_entry(),
            Vec::<StoredEntry>::new(),
            "开库之后这个键空间里一行都没有"
        );
        assert!(
            opened.registered_verdict_vectors().is_empty(),
            "头一次开的库一种判定向量都没登记"
        );
    }
}

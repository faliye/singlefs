//! 事实表与 GPU 判表进库、再从库里读回（锁里读与锁外读两种）。

use super::*;

// ============================== 事实表进库与读回 ==============================

/// 按摘要从 `store` 读事实表：读字节，再交 [`FlowFactsReadBack::from_the_library_bytes`] 解码、核摘要。读库这一步放在这里，
/// `crash_facts` 不引判定库。
///
/// # Errors
/// 库读失败；库里没有这个摘要；解不开；摘要对不上。
pub fn read_flow_facts_back(
    store: &VerdictStore,
    digest: &[u8; DIGEST_BYTES],
) -> Result<FlowFactsReadBack, String> {
    let bytes = store
        .read_flow_facts(digest)
        .map_err(|error| format!("读事实表失败：{error:?}"))?
        .ok_or_else(|| "事实表不在库里：先跑录入段".to_string())?;
    FlowFactsReadBack::from_the_library_bytes(&bytes, digest)
}

/// 同 [`read_flow_facts_back`]，库只在读字节那一次往返里取：解码、核摘要在锁外。
///
/// # Errors
/// 同 [`read_flow_facts_back`]。
pub fn read_flow_facts_back_outside_the_lock(
    library: &LibraryTakenAtEachReadOrWrite<'_>,
    digest: &[u8; DIGEST_BYTES],
) -> Result<FlowFactsReadBack, String> {
    let bytes = library
        .take()
        .read_flow_facts(digest)
        .map_err(|error| format!("读事实表失败：{error:?}"))?
        .ok_or_else(|| "事实表不在库里：先跑录入段".to_string())?;
    assert_this_thread_holds_no_library_lock("核事实表的摘要、解事实表");
    FlowFactsReadBack::from_the_library_bytes(&bytes, digest)
}

/// 把事实表按摘要写进 `store`，再从 `store` 读回：核对只拿读回的那一份。
///
/// # Errors
/// 写不进库；读回失败（[`read_flow_facts_back`]）。
pub fn record_the_facts_and_read_them_back(
    store: &mut VerdictStore,
    facts: &FlowFacts,
) -> Result<FlowFactsReadBack, String> {
    let digest = facts.digest();
    store
        .store_flow_facts(&digest, &facts.to_bytes())
        .map_err(|error| format!("事实表写不进库：{error:?}"))?;
    read_flow_facts_back(store, &digest)
}

#[cfg(feature = "gpu")]
/// 按摘要从库里读打包判表（GPU 判器只收读回的那一份，`crate::crash_judge_dispatch::PackedJudgeTablesReadBack`）：读字节、核摘要、解码。
///
/// # Errors
/// 库读失败；库里没有这个摘要；字节的摘要对不上（库里那份被改过）；解不开。
pub fn read_packed_judge_tables_back(
    store: &VerdictStore,
    digest: &[u8; DIGEST_BYTES],
) -> Result<PackedJudgeTablesReadBack, String> {
    let bytes = store
        .read_packed_judge_tables(digest)
        .map_err(|error| format!("读 GPU 判表失败：{error:?}"))?
        .ok_or_else(|| "GPU 判表不在库里：先跑录入段".to_string())?;
    PackedJudgeTablesReadBack::from_the_library_bytes(&bytes, digest)
}

#[cfg(feature = "gpu")]
/// 同 [`read_packed_judge_tables_back`]，库只在读字节那一次往返里取：核摘要、解码在锁外（判表可到几百 MiB，别的线程不等它解）。
///
/// # Errors
/// 同 [`read_packed_judge_tables_back`]。
pub fn read_packed_judge_tables_back_outside_the_lock(
    library: &LibraryTakenAtEachReadOrWrite<'_>,
    digest: &[u8; DIGEST_BYTES],
) -> Result<PackedJudgeTablesReadBack, String> {
    let bytes = library
        .take()
        .read_packed_judge_tables(digest)
        .map_err(|error| format!("读 GPU 判表失败：{error:?}"))?
        .ok_or_else(|| "GPU 判表不在库里：先跑录入段".to_string())?;
    assert_this_thread_holds_no_library_lock("核判表的摘要、解读回的判表");
    PackedJudgeTablesReadBack::from_the_library_bytes(&bytes, digest)
}

#[cfg(feature = "gpu")]
/// 把打包判表按摘要写进 `store`，再从 `store` 读回：GPU 判器只拿读回的那一份。录入段、给定状态表的引擎与对拍用例都经它。
///
/// # Errors
/// 写不进库；读回失败（[`read_packed_judge_tables_back`]）。
pub fn record_the_judge_tables_and_read_them_back(
    store: &mut VerdictStore,
    tables: &PackedJudgeTables,
) -> Result<PackedJudgeTablesReadBack, String> {
    let bytes = tables.to_bytes();
    let digest = sha256_digest(&bytes);
    store
        .store_packed_judge_tables(&digest, &bytes)
        .map_err(|error| format!("GPU 判表写不进库：{error:?}"))?;
    drop(bytes);
    read_packed_judge_tables_back(store, &digest)
}

#[cfg(feature = "gpu")]
/// 同 [`record_the_judge_tables_and_read_them_back`]，锁只包写与读那两次往返：序列化、算摘要、核摘要与解码都在锁外（几条线程同时判给定状态表时互不等）。
///
/// # Errors
/// 同 [`record_the_judge_tables_and_read_them_back`]。
pub(crate) fn record_the_judge_tables_and_read_them_back_outside_the_lock(
    library: &LibraryOpenInThisProcess,
    tables: &PackedJudgeTables,
) -> Result<PackedJudgeTablesReadBack, String> {
    assert_this_thread_holds_no_library_lock("序列化判表");
    let bytes = tables.to_bytes();
    let digest = sha256_digest(&bytes);
    library
        .lock()
        .store_packed_judge_tables(&digest, &bytes)
        .map_err(|error| format!("GPU 判表写不进库：{error:?}"))?;
    drop(bytes);
    let bytes_read_back = library
        .lock()
        .read_packed_judge_tables(&digest)
        .map_err(|error| format!("读 GPU 判表失败：{error:?}"))?
        .ok_or_else(|| "GPU 判表不在库里：先跑录入段".to_string())?;
    assert_this_thread_holds_no_library_lock("核摘要、解判表");
    PackedJudgeTablesReadBack::from_the_library_bytes(&bytes_read_back, &digest)
}

#[cfg(feature = "gpu")]
/// 同 [`record_the_judge_tables_and_read_them_back`]，库每一次读写现取现放（[`LibraryTakenAtEachReadOrWrite`]）：序列化、算摘要、
/// 核摘要与解码都在锁外。判段在观察者交回的盘上判池级 checker 时经它。
///
/// # Errors
/// 同 [`record_the_judge_tables_and_read_them_back`]。
pub(crate) fn record_the_judge_tables_and_read_them_back_at_each_read_or_write(
    library: &LibraryTakenAtEachReadOrWrite<'_>,
    tables: &PackedJudgeTables,
) -> Result<PackedJudgeTablesReadBack, String> {
    assert_this_thread_holds_no_library_lock("序列化判段要入库的判表");
    let bytes = tables.to_bytes();
    let digest = sha256_digest(&bytes);
    library
        .take()
        .store_packed_judge_tables(&digest, &bytes)
        .map_err(|error| format!("GPU 判表写不进库：{error:?}"))?;
    drop(bytes);
    read_packed_judge_tables_back_outside_the_lock(library, &digest)
}

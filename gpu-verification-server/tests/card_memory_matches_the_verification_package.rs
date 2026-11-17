//! 调度算的判器显存（`card_memory_ledger::mebibytes_of_a_judge`）与验证包借卡时算的（`crash_judge_dispatch::mebibytes_a_judge_needs`）
//! 在一条真流的打包判表上逐个相同：一次派活要 1 个态、几个态、正好整条流、多于整条流（压回整条流）。不上卡。

mod common;

use common::new_pool_file_creation_judge_tables;
use gpu_verification_server::card_memory_ledger::mebibytes_of_a_judge;
use gpu_verification_server::stream_identity::{StreamFacts, StreamKey};
use singlefs_verification::crash_judge_dispatch::{mebibytes_a_judge_needs, PackedJudgeTables};
use singlefs_verification::crash_judge_gpu::JudgeKernelSources;

/// 新池新建文件那条流的展开上限：段长不超过它的段逐个真子集展开（几百个态以内）。
const EXPAND_UP_TO: usize = 3;

#[test]
fn judge_memory_of_the_scheduler_equals_the_memory_the_verification_package_leases() {
    let tables = PackedJudgeTables::of(&new_pool_file_creation_judge_tables(EXPAND_UP_TO));
    let facts = StreamFacts::of_the_tables(&tables);
    assert!(facts.state_count > 1, "这条流至少两个态");
    for requested in [
        1,
        2,
        7,
        facts.state_count / 2,
        facts.state_count,
        facts.state_count + 5,
        facts.state_count * 4,
    ] {
        assert_eq!(
            mebibytes_of_a_judge(&facts, requested),
            mebibytes_a_judge_needs(&tables, usize::try_from(requested).expect("装得进 usize")),
            "一次派活要 {requested} 个态（这条流 {} 个态）",
            facts.state_count
        );
    }
}

#[test]
fn the_stream_key_changes_with_the_tables_and_the_kernel_sources_and_nothing_else() {
    let wider = PackedJudgeTables::of(&new_pool_file_creation_judge_tables(EXPAND_UP_TO));
    let narrower = PackedJudgeTables::of(&new_pool_file_creation_judge_tables(0));
    let sources = JudgeKernelSources::of_the_judge();
    let key = StreamKey::of_the_tables_under_the_kernel_sources(&wider, &sources);
    assert_eq!(
        key,
        StreamKey::of_the_tables_under_the_kernel_sources(
            &wider,
            &JudgeKernelSources::of_the_judge()
        ),
        "同一份表、同一套源码是同一条流"
    );
    assert_ne!(
        key.tables_digest,
        StreamKey::of_the_tables_under_the_kernel_sources(&narrower, &sources).tables_digest,
        "展开上限不同、表不同，是另一条流"
    );
    let mut other_sources = JudgeKernelSources::of_the_judge();
    other_sources.checker_scan.push('\n');
    assert_ne!(
        key,
        StreamKey::of_the_tables_under_the_kernel_sources(&wider, &other_sources),
        "源码差一个字节就是另一条流"
    );
}

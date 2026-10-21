//! checker 档模块：无（只经截断流与整理流的搭建模块取两个观察者另读的输入，本文件不直接导入 checker 档的模块）
//! 观察者另读的输入的摘要进观察者的版本与去重键，只由流里逐版的内容组成，不带标签与手写的版本号
//! （D13（验证路线） 已定项 17「一律不进键」：手写的版本常量）：可写挂载续做截断那个观察者的
//! `TruncationMountContinuationInputs::digest`（`tests/common_truncation_crash_streams/mod.rs`）与整理流那几个观察者的
//! `text_of_the_versions_of_the_compaction_flow`（`crates/singlefs-harness/tests/common_compaction_prefix/mod.rs`）。
//! 钉法：一版都没有时，文字就是空串、摘要就是空串的 SHA-256；文字里只要多一行标签或版本号，这两条就红。
#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;
#[path = "../../singlefs-harness/tests/common_compaction_prefix/mod.rs"]
mod common_compaction_prefix;
mod common_crash_point_observers;
mod common_crash_points;
#[path = "../../singlefs-harness/tests/common_snapshot_prefix/mod.rs"]
mod common_snapshot_prefix;
#[path = "../../singlefs-harness/tests/common_truncation/mod.rs"]
mod common_truncation;
mod common_truncation_crash_streams;

use common_compaction_prefix::text_of_the_versions_of_the_compaction_flow;
use common_truncation_crash_streams::TruncationMountContinuationInputs;
use singlefs_harness::sha256::sha256_digest;

/// 可写挂载续做截断那个观察者另读的输入：一版都没有时摘要是空串的 SHA-256（文字里只有逐版那几行，没有标签、没有版本号）。
#[test]
fn the_truncation_mount_continuation_inputs_without_any_version_digest_to_the_digest_of_the_empty_text(
) {
    assert_eq!(
        TruncationMountContinuationInputs {
            versions: Vec::new(),
        }
        .digest(),
        sha256_digest(b""),
        "另读的输入的摘要只由逐版那几行组成：一版都没有时就是空串的摘要，文字里不该有标签或手写的版本号"
    );
}

/// 整理流那几个观察者另读的输入：一版都没有时文字是空串（文字里只有逐版那几行，没有标签、没有版本号）。
#[test]
fn the_text_of_the_versions_of_the_compaction_flow_without_any_version_is_empty() {
    assert_eq!(
        text_of_the_versions_of_the_compaction_flow(&[]),
        "",
        "另读的输入的文字只由逐版那几行组成：一版都没有时就是空串，不该有标签或手写的版本号"
    );
}

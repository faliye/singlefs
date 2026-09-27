"""崩溃注入那份挪走文件：合入后验证一 crates-moved.patch 第 1 块（use 行）在挪包之后打不上，照它的意思手改——
改钉「C554 乙之后不可达」之后不再用 check_records_against 与 RecordStreamContinuity。"""
from edit_lib import edit

CI = "crates/singlefs-checker-tier/tests/second_transaction_supplement_three_crash_injection.rs"
edit(CI, "use singlefs_checker_tier::crash::{check_records, check_records_against};", "use singlefs_checker_tier::crash::check_records;")
edit(CI, """use singlefs_harness::memory_pool::{
    root_identity_written_by, writes_and_segments, CrashImage, MemoryPool, RecordCheck,
    RecordStreamContinuity, RetainedWrite,
};""", """use singlefs_harness::memory_pool::{
    root_identity_written_by, writes_and_segments, CrashImage, MemoryPool, RecordCheck,
    RetainedWrite,
};""")
print("16_crash_injection_imports done")

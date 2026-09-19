#!/usr/bin/env python3
"""攻方腿（Opus）m2-supp3-item1-code-r1 · Z4：给仓副本打「快档几何从环境变量取」的补丁（主工作区不打）。
- 测试文件：FAST_TIER_FIRST_SEED / FAST_TIER_SEEDS / FAST_TIER_OPERATIONS_PER_HISTORY 三个常量改成读 OPUS_FAST_FIRST / OPUS_FAST_SEEDS / OPUS_FAST_OPS（缺省照原值 0 / 96 / 30），
  快档的判定（新发现为空 + assert_every_path_was_exercised）一个字不动。
- history.rs：operation_weights 的「开着、有文件」那张表可由 OPUS_WEIGHTS_WITH_FILE 覆盖（七个数，次序照原表：覆盖写、抬 F、可写挂载、回退、冷启动、发第一个文件、空发布），
  「关着」那张由 OPUS_WEIGHTS_CLOSED 覆盖（三个数：可写挂载、回退、冷启动）；不给就是原表。
用法：python3 geometry-env-patch.py <副本根>
"""
import sys, os
root = sys.argv[1]
def replace_once(path, old, new):
    full = os.path.join(root, path)
    text = open(full, encoding="utf-8").read()
    assert text.count(old) == 1, (path, old[:60], text.count(old))
    open(full, "w", encoding="utf-8").write(text.replace(old, new))
test = "crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs"
replace_once(test, "const FAST_TIER_FIRST_SEED: u64 = 0;\nconst FAST_TIER_SEEDS: u64 = 96;\nconst FAST_TIER_OPERATIONS_PER_HISTORY: usize = 30;\n",
    "fn opus_env(name: &str, default: u64) -> u64 { std::env::var(name).map_or(default, |text| text.parse().expect(\"整数\")) }\n")
replace_once(test, "    let report = run_history_campaign(\n        FAST_TIER_FIRST_SEED,\n        FAST_TIER_SEEDS,\n        FAST_TIER_OPERATIONS_PER_HISTORY,\n        worker_threads_by_default(),\n        FindingShrinking::ReportSeedsOnly,\n    );",
    "    let report = run_history_campaign(\n        opus_env(\"OPUS_FAST_FIRST\", 0),\n        opus_env(\"OPUS_FAST_SEEDS\", 96),\n        usize::try_from(opus_env(\"OPUS_FAST_OPS\", 30)).expect(\"步数\"),\n        worker_threads_by_default(),\n        FindingShrinking::ReportSeedsOnly,\n    );")
replace_once(test, "        u64::try_from(FAST_TIER_OPERATIONS_PER_HISTORY).expect(\"步数\"),", "        30,")
history = "crates/singlefs-harness/src/history.rs"
replace_once(history, "fn operation_weights(expected: ExpectedSession) -> &'static [(HistoryOperationKind, u64)] {\n    match expected {\n",
    "fn operation_weights(expected: ExpectedSession) -> &'static [(HistoryOperationKind, u64)] {\n"
    "    let override_of = |name: &str, kinds: &[HistoryOperationKind]| -> Option<&'static [(HistoryOperationKind, u64)]> {\n"
    "        let text = std::env::var(name).ok()?;\n"
    "        let numbers: Vec<u64> = text.split(',').map(|part| part.trim().parse().expect(\"比重是整数\")).collect();\n"
    "        assert_eq!(numbers.len(), kinds.len(), \"{name} 要 {} 个数\", kinds.len());\n"
    "        Some(Box::leak(kinds.iter().copied().zip(numbers).collect::<Vec<_>>().into_boxed_slice()))\n"
    "    };\n"
    "    if expected == ExpectedSession::OpenWithFile {\n"
    "        if let Some(table) = override_of(\"OPUS_WEIGHTS_WITH_FILE\", &[HistoryOperationKind::PublishOverwrite, HistoryOperationKind::RaiseRollbackFloor, HistoryOperationKind::CloseAndMountWritable, HistoryOperationKind::CloseAndMountRollback, HistoryOperationKind::ColdStartRecover, HistoryOperationKind::PublishFirstFile, HistoryOperationKind::PublishWithoutUnits]) { return table; }\n"
    "    }\n"
    "    if expected == ExpectedSession::Closed {\n"
    "        if let Some(table) = override_of(\"OPUS_WEIGHTS_CLOSED\", &[HistoryOperationKind::CloseAndMountWritable, HistoryOperationKind::CloseAndMountRollback, HistoryOperationKind::ColdStartRecover]) { return table; }\n"
    "    }\n"
    "    match expected {\n")
print("patched", root)

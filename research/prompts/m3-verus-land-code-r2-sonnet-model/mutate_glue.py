#!/usr/bin/env python3
"""H3/H4：对改后胶水（tree_g/crates/singlefs-core/src/recovery.rs 的 replay_journal 里「核失败 → RecoveryFailure」「核报告 → 报告」「核的新根来源 → 根」那几处）做单点改动，
只跑点名的 harness 目标，看哪些红。用法：mutate_glue.py <草稿根>"""
import subprocess, sys, pathlib, re, os
ROOT = pathlib.Path(sys.argv[1])
SRC = ROOT / "tree_g/crates/singlefs-core/src/recovery.rs"
pristine = SRC.read_text()
TARGETS = ["recovery_anchor_failure_carries_the_counters_the_prefix_kernel_saw", "core_review_tree_table_duplicates_and_slot_one_search",
           "journal_last_record_flag_of_a_multi_record_publish", "journal_replay_applies_no_more_records_than_the_in_flight_limit",
           "writable_mount_takes_a_second_instance"]
M = [
 ("counters_reversed", "counters: counters.iter().map(|counter| counter.0).collect(),", "counters: counters.iter().rev().map(|counter| counter.0).collect(),"),
 ("counters_drop_first", "counters: counters.iter().map(|counter| counter.0).collect(),", "counters: counters.iter().skip(1).map(|counter| counter.0).collect(),"),
 ("error_txg_wrong", "                    checkpoint_txg: root.checkpoint_txg,\n                    counters:", "                    checkpoint_txg: CheckpointTxg(root.checkpoint_txg.0 + 1),\n                    counters:"),
 ("error_instance_wrong", "                    instance: root.instance,\n                    checkpoint_txg: root.checkpoint_txg,\n                    counters:", "                    instance: InstanceGeneration(root.instance.0 + 1),\n                    checkpoint_txg: root.checkpoint_txg,\n                    counters:"),
 ("no_successor_maps_to_no_valid_root", "            return Err(RecoveryFailure::UnitMalformed {\n                what: CHOSEN_ROOT_CHECKPOINT_TXG_HAS_NO_SUCCESSOR,\n            });", "            return Err(RecoveryFailure::NoValidRoot);"),
 ("no_successor_wrong_what", "                what: CHOSEN_ROOT_CHECKPOINT_TXG_HAS_NO_SUCCESSOR,\n            });\n        }\n    };", "                what: TREE_TABLE_CARRIES_ONE_KIND_OF_TREE_TWICE,\n            });\n        }\n    };"),
 ("above_water_is_table_size", "report.above_water = outcome.report.above_water;", "report.above_water = records.len();"),
 ("applied_is_zero", "report.prefix_applied = outcome.report.prefix_applied;", "report.prefix_applied = 0;"),
 ("max_tx_zero", "report.maximum_applied_transaction = outcome.report.maximum_applied_transaction.0;", "report.maximum_applied_transaction = 0;"),
 ("passed_swapped_with_failed", "report.verification_passed = outcome.report.verification_passed;\n    report.verification_failed = outcome.report.verification_failed;", "report.verification_passed = outcome.report.verification_failed;\n    report.verification_failed = outcome.report.verification_passed;"),
 ("back_chain_inverted", "BackChainJudgement::Broken => true,", "BackChainJudgement::Broken => false,"),
 ("named_all_to_any", "records_in_key_order[index].named.iter().all(|named| {", "records_in_key_order[index].named.iter().any(|named| {"),
 ("counter_assert_le", "record.counter < u64::MAX,", "record.counter <= u64::MAX,"),
 ("key_assert_removed", "            *key,\n            (record.instance, record.counter),", "            (record.instance, record.counter),\n            (record.instance, record.counter),"),
 ("skip_mode_inverted", "let named_unit_verification = if verify_named_units {", "let named_unit_verification = if !verify_named_units {"),
 ("rebuilt_wrong_record", "let record = records_in_key_order[index];\n            // 每条记录都带整次发布的新根段", "let record = records_in_key_order[0];\n            // 每条记录都带整次发布的新根段"),
]
env = {**os.environ}
try:
    for name, old, new in M:
        assert pristine.count(old) == 1, (name, pristine.count(old))
        SRC.write_text(pristine.replace(old, new))
        log = ROOT / f"glue_{name}.log"
        cmd = ["bash", "/home/fy5090/code/singlefs/research/scripts/run-with-memory-cap.sh", "8G", "bash", "/home/fy5090/code/singlefs/research/scripts/capped.sh", "6", "cargo", "test", "--release", "--offline", "-p", "singlefs-harness"]
        for t in TARGETS: cmd += ["--test", t]
        cmd += ["--no-fail-fast"]
        e = {**env, "RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON": "派发提示给定内存上限八G，胶水变异只跑点名的 harness 目标", "RUN_WITH_MEMORY_CAP_PARALLEL_REASON": "同会话派出的几个腿各跑各的，这一条是胶水变异"}
        r = subprocess.run(["nice", "-n", "19"] + cmd, cwd=ROOT / "tree_g", env=e, capture_output=True, text=True)
        out = r.stdout + r.stderr
        log.write_text(out)
        failed = sorted(set(re.findall(r"^test (\S+) \.\.\. FAILED", out, re.M)))
        results = re.findall(r"^test result: (\w+)\. (\d+) passed; (\d+) failed", out, re.M)
        compiled = "error[" not in out and "could not compile" not in out
        print(f"GLUE {name} | 编过={compiled} | 退出码={r.returncode} | 各目标 passed/failed={[(p, f) for _, p, f in results]} | 红的测试={failed}", flush=True)
finally:
    SRC.write_text(pristine)

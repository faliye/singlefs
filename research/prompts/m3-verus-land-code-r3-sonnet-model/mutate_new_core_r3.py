#!/usr/bin/env python3
"""对改后核（冻结副本拷贝里的 crates/singlefs-replay-prefix/src/lib.rs）的可执行代码做单点改动，每个改动用差分（对 fcc4d340 那一版）量红不红。
用法：mutate_new_core.py <草稿根>（里面有 tree_m/ 与 run.sh）；每个改动先断言旧串恰好命中一次，跑完换回原文。"""
import subprocess, sys, pathlib, re, os
ROOT = pathlib.Path(sys.argv[1])
LIB = ROOT / "tree_m/crates/singlefs-replay-prefix/src/lib.rs"
pristine = LIB.read_text()
M = [
 ("limit_gt", "if records_taken_toward_the_in_flight_limit >= in_flight_limit {", "if records_taken_toward_the_in_flight_limit > in_flight_limit {"),
 ("backchain_off", "if back_chain_broken(index) {", "if false {"),
 ("first_ordinal_unchecked_open", "if records_of_the_open_publish.is_empty() && record.ordinal_within_publish.0 != 1 {", "if false {"),
 ("txg_change_unchecked", "if record.checkpoint_txg.0 != previous_record_of_the_open_publish.checkpoint_txg.0 {", "if false {"),
 ("tx_change_unchecked", "&& record.transaction.0 != previous_record_of_the_open_publish.transaction.0 {", "&& false {"),
 ("last_without_commit_unchecked", "if !record.is_commit && record_ends_its_publish(record) {", "if false {"),
 ("followers_unchecked", "if record_ends_its_publish(record) && readable_record_of_the_same_publish_follows(", "if false && record_ends_its_publish(record) && readable_record_of_the_same_publish_follows("),
 ("named_skipped", "NamedUnitVerification::Verify => named_units_verified(index),", "NamedUnitVerification::Verify => true,"),
 ("passed_plus_two", "report.verification_passed += 1;", "report.verification_passed += 2;"),
 ("above_water_plus_two", "report.above_water += 1;", "report.above_water += 2;"),
 ("root_not_updated", "root_source_after_replay = record.new_root_source;", "let _unused = 0;"),
 ("applied_count_one", "report.prefix_applied += records_of_the_open_publish.len();", "report.prefix_applied += 1;"),
 ("expected_next_plus_two", "expected_next = Some((record.instance, JournalCounter(record.counter.0 + 1)));", "expected_next = Some((record.instance, JournalCounter(record.counter.0 + 2)));"),
 ("anchor_error_needs_three", "if counters_carrying_the_last_record_flag.len() > 1 {", "if counters_carrying_the_last_record_flag.len() > 2 {"),
 ("no_successor_off", "if root_checkpoint_txg.0 == u64::MAX {", "if false {"),
 ("above_ge", "record.instance.0 == root_instance.0 && record.checkpoint_txg.0 > root_checkpoint_txg.0", "record.instance.0 == root_instance.0 && record.checkpoint_txg.0 >= root_checkpoint_txg.0"),
 ("ordinal_step_two", ") + 1 {", ") + 2 {"),
 ("head_ordinal_unchecked", "|| record.ordinal_within_publish.0 != 1 {", "|| false {"),
 ("head_txg_unchecked", "if record.checkpoint_txg.0 != chain_start_txg_without_anchor.0", "if false"),
 ("verification_failed_not_counted", "report.verification_failed += 1;", "report.verification_failed += 0;"),
 ("follower_same_txg_blind", "if later.checkpoint_txg.0 == record.checkpoint_txg.0 {", "if false {"),
 ("maximum_takes_minimum", "if transaction.0 > maximum.0 {", "if transaction.0 < maximum.0 {"),
]
env = {**os.environ, "DIFF_THREADS": "6", "DIFF_N": "150000"}
try:
    for name, old, new in M:
        assert pristine.count(old) == 1, (name, pristine.count(old))
        LIB.write_text(pristine.replace(old, new))
        log = ROOT / f"mut_{name}.log"
        subprocess.run(["bash", str(ROOT / "run.sh"), str(ROOT / "tree_m"), str(log), "--", "--test-threads=1", "--nocapture", "random_family", "exhaustive_structural_quick"], env=env)
        text = log.read_text()
        diverged = [int(x) for x in re.findall(r"RESULT\[[^\]]*\] Tally \{ total: \d+, diverged: (\d+)", text)]
        built = "could not compile" not in text and "error[" not in text
        print(f"MUT {name} | 编过={built} | 各族 diverged={diverged} | 合计={sum(diverged)} | {'红(抓到)' if sum(diverged) > 0 else '绿(没抓到)'}", flush=True)
finally:
    LIB.write_text(pristine)

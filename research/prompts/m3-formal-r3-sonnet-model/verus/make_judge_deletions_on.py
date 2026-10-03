#!/usr/bin/env python3
"""核里每一道 break 判一份「删判」变体：条件前加 `false &&`。写到 verus/variants/<名>/src/lib.rs。"""
import pathlib, shutil
import sys
src = pathlib.Path(sys.argv[1]).read_text()
out = pathlib.Path(sys.argv[2])
J = [
 ("d01_continuity_I83", "                if record.instance != expected_key.0 || record.counter != expected_key.1 {",
                        "                if false && (record.instance != expected_key.0 || record.counter != expected_key.1) {"),
 ("d02_chain_head_without_anchor", "                if record.checkpoint_txg != chain_start_txg_without_anchor\n                    || record.ordinal_within_publish != 1 {",
                                    "                if false && (record.checkpoint_txg != chain_start_txg_without_anchor\n                    || record.ordinal_within_publish != 1) {"),
 ("d03_back_chain", "        if back_chain_broken(index) {", "        if false && back_chain_broken(index) {"),
 ("d04_first_ordinal_is_1", "        if open.is_empty() && record.ordinal_within_publish != 1 {", "        if false && open.is_empty() && record.ordinal_within_publish != 1 {"),
 ("d05_txg_change_in_open_publish", "            if record.checkpoint_txg != previous_record_of_the_open_publish.checkpoint_txg {",
                                     "            if false && record.checkpoint_txg != previous_record_of_the_open_publish.checkpoint_txg {"),
 ("d06_ordinal_skip", "            if record.ordinal_within_publish as u64\n                != previous_record_of_the_open_publish.ordinal_within_publish as u64 + 1 {",
                      "            if false && (record.ordinal_within_publish as u64\n                != previous_record_of_the_open_publish.ordinal_within_publish as u64 + 1) {"),
 ("d07_transaction_boundary", "            if !previous_record_of_the_open_publish.is_commit && record.transaction\n                != previous_record_of_the_open_publish.transaction {",
                             "            if false && (!previous_record_of_the_open_publish.is_commit && record.transaction\n                != previous_record_of_the_open_publish.transaction) {"),
 ("d08_commit_mark_of_last_record", "        if !record.is_commit && record_ends_its_publish(record) {", "        if false && !record.is_commit && record_ends_its_publish(record) {"),
 ("d09_reader_rule_follows", "        if record_ends_its_publish(record) && readable_record_of_the_same_publish_follows(", "        if false && record_ends_its_publish(record) && readable_record_of_the_same_publish_follows("),
 ("d10_named_units_not_verified", "        if !all_verified {", "        if false && !all_verified {"),
 ("d11_in_flight_limit", "        if taken >= in_flight_limit {", "        if false && taken >= in_flight_limit {"),
 ("d12_top_txg_check", "    if root_checkpoint_txg == u64::MAX {", "    if false && root_checkpoint_txg == u64::MAX {"),
 ("d13_above_the_root_filter", "        if !is_above_the_chosen_root(record, root_instance, root_checkpoint_txg) {", "        if false && !is_above_the_chosen_root(record, root_instance, root_checkpoint_txg) {"),
]
if out.exists(): shutil.rmtree(out)
for name, old, new in J:
    assert src.count(old) == 1, (name, src.count(old))
    d = out / name / "src"; d.mkdir(parents=True)
    (d / "lib.rs").write_text(src.replace(old, new))
(out / "base" / "src").mkdir(parents=True)
(out / "base" / "src" / "lib.rs").write_text(src)
print(len(J), "variants + base")

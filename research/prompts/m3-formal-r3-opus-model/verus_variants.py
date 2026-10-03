#!/usr/bin/env python3
"""m3-formal-r3 攻方腿：把冻结副本的核 lib.rs 照下表各改一处，各写一份到 <输出目录>/<名>/src/lib.rs（原文要恰好命中一次）。
用法：verus_variants.py <冻结副本的 lib.rs> <输出目录>"""
import sys
from pathlib import Path

source = Path(sys.argv[1]).read_text()
out = Path(sys.argv[2])
VARIANTS = [
    ("v00-original", None, None),
    ("v01-skip-I-8.3-jsn-check", "                if record.instance != expected_key.0 || record.counter != expected_key.1 {\n",
                                 "                if false {\n"),
    ("v02-tsv36-no-anchor-ignores-txg", "                if record.checkpoint_txg != chain_start_txg_without_anchor\n",
                                        "                if false && record.checkpoint_txg != chain_start_txg_without_anchor\n"),
    ("v03-Z1-applies-nothing", "        let record = &records[index];\n        // [原文 3078–3080] above 的筛选条件，同上面那一遍\n",
                               "        if true {\n            break ;\n        }\n        let record = &records[index];\n        // [原文 3078–3080] above 的筛选条件，同上面那一遍\n"),
    ("v04-Z2-ignores-in-flight-limit", "        if taken >= in_flight_limit {\n", "        if false {\n"),
    ("v05-tsv516-last-without-commit", "        if !record.is_commit && record_ends_its_publish(record) {\n", "        if false {\n"),
    ("v06-tsv515-txn-change-without-commit", "            if !previous_record_of_the_open_publish.is_commit && record.transaction\n",
                                             "            if false && record.transaction\n"),
    ("v07-tsv614-fourth-cell-off", "        if record_ends_its_publish(record) && readable_record_of_the_same_publish_follows(\n",
                                   "        if false && readable_record_of_the_same_publish_follows(\n"),
    ("v08-back-chain-off", "        if back_chain_broken(index) {\n", "        if false {\n"),
    ("v09-named-units-off", "        let all_verified = !verify_named_units || named_units_verified(index);\n",
                            "        let all_verified = true;\n"),
    ("v10-tsv613-first-ordinal-not-1", "        if open.is_empty() && record.ordinal_within_publish != 1 {\n", "        if false {\n"),
    ("v11-tsv406-every-record-last", "        if !record_ends_its_publish(record) {\n", "        if false {\n"),
    ("v12-tsv513-two-anchors-no-error", "            if found.is_some() {\n                return Err(CoreFailure::RootPublishCarriesMoreThanOneLastRecordFlag);\n            }\n",
                                         "            if false {\n                return Err(CoreFailure::RootPublishCarriesMoreThanOneLastRecordFlag);\n            }\n"),
]
for name, old, new in VARIANTS:
    text = source
    if old is not None:
        count = text.count(old)
        if count != 1:
            print(f"✗ {name}: 原文命中 {count} 次", file=sys.stderr)
            sys.exit(3)
        text = text.replace(old, new)
    target = out / name / "src" / "lib.rs"
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(text)
    print(name)

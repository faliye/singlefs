#!/usr/bin/env python3
"""攻方腿（m3-batch62-code-r1 opus）的变异：在仓副本上按名字做一处原样替换（旧串在文件里恰好命中一次，否则退出 2）。
名字可以用 + 连几种（依次打）。用法：mutants.py <副本根> <名字>；mutants.py --list。
前三种是第 62 批追加进 crates/mutations.tsv 的 Q3 三行（与整行替换那一行同文），照原文抄；后面的是这一条腿自己加的邻居变异。"""
import sys
from pathlib import Path

STORE = "crates/singlefs-checker-tier/src/verdict_store.rs"
LOOP = "        for verdict_vector in pending.verdict_vectors_registered_in_the_batch {"

MUTANTS = {
    # 表里 Q3 第 1 行与整行替换那一行：批长判据反过来
    "flip": [(STORE, "        if pending.batch.size_in_bytes() < batch_size_in_bytes {",
              "        if pending.batch.size_in_bytes() >= batch_size_in_bytes {")],
    # 表里 Q3 第 2 行：写下一批之后登记的判定向量一种都不进内存表（跨批编号重起）
    "take0": [(STORE, LOOP, "        for verdict_vector in pending.verdict_vectors_registered_in_the_batch.into_iter().take(0) {")],
    # 表里 Q3 第 3 行：核对行点名事实挪到「本库已有且相同就跳过」之后
    "facts-after": [(STORE,
        "            facts_digests_named.insert(facts_digest);\n            let stored_value = self\n                .database\n                .get(&key_bytes.0)\n                .map_err(VerdictStoreError::RocksDatabase)?;\n            if stored_value.as_deref() == Some(value_bytes.as_slice()) {\n                continue;\n            }\n",
        "            let stored_value = self\n                .database\n                .get(&key_bytes.0)\n                .map_err(VerdictStoreError::RocksDatabase)?;\n            if stored_value.as_deref() == Some(value_bytes.as_slice()) {\n                continue;\n            }\n            facts_digests_named.insert(facts_digest);\n")],
    # 邻居：一批写下去之后，这一批新登记的判定向量只有第一种进内存表（一批只登一种时与原样相同）
    "take1": [(STORE, LOOP, "        for verdict_vector in pending.verdict_vectors_registered_in_the_batch.into_iter().take(1) {")],
    # 邻居：一批写下去之后，这一批新登记的判定向量倒着进内存表（编号照进表的次序取，一批只登一种时与原样相同）
    "rev": [(STORE, LOOP, "        for verdict_vector in pending.verdict_vectors_registered_in_the_batch.into_iter().rev() {")],
}


def main():
    if sys.argv[1:] == ["--list"]:
        print("\n".join(sorted(MUTANTS)))
        return 0
    root, names = Path(sys.argv[1]), sys.argv[2].split("+")
    for name in names:
        for relative, old, new in MUTANTS[name]:
            path = root / relative
            text = path.read_text()
            if text.count(old) != 1:
                print(f"变异 {name}：{relative} 里旧串命中 {text.count(old)} 次", file=sys.stderr)
                return 2
            path.write_text(text.replace(old, new))
            print(f"变异 {name}：{relative} 已替换")
    return 0


sys.exit(main())

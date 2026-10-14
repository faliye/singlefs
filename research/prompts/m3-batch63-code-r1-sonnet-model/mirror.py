#!/usr/bin/env python3
"""正推腿（m3-batch63-code-r1 sonnet）：在仓副本里把 verdict_store.rs 的 `#[cfg(test)] mod tests` 改成公开的 mirror_tests
（测试函数改成 pub fn、去掉 #[test] 与 should_panic/ignore），好让副本里另一个 crate（probe-crate）的测试二进制逐条调它们；
不改任何产品代码，不跑 singlefs-checker-tier 包自己的测试二进制。再按名字施加变异（旧串必须恰好命中一次，否则退 2）。
用法：mirror.py <副本根> [变异名…]；list 列变异名。另生成 <副本根>/crates/sonnet-probe-b63/tests/mirror_all.rs（每条镜像用例一个 #[test]）。"""
import re
import sys
from pathlib import Path

STORE = "crates/singlefs-checker-tier/src/verdict_store.rs"

# 变异：名字 -> [(旧串, 新串)]，每个旧串在文件里恰好命中一次
MUTANTS = {
    # 本批 S1 三行（原文与替换文照 crates/mutations.tsv 追加的三行）
    "take1": [(
        "        for verdict_vector in pending.verdict_vectors_registered_in_the_batch {\n",
        "        for verdict_vector in pending.verdict_vectors_registered_in_the_batch.into_iter().take(1) {\n")],
    "rev": [(
        "        for verdict_vector in pending.verdict_vectors_registered_in_the_batch {\n",
        "        for verdict_vector in pending.verdict_vectors_registered_in_the_batch.into_iter().rev() {\n")],
    "no-insert": [(
        "            self.verdict_vector_number_by_content\n                .insert(verdict_vector.0.clone(), number);\n            self.verdict_vector_by_number.push(verdict_vector);\n",
        "            let _ = number;\n            self.verdict_vector_by_number.push(verdict_vector);\n")],
    # S2：新替换文；旧的第 62 批 Q3 那一行（替换文 >=）
    "s2-new": [(
        "        if pending.batch.size_in_bytes() < batch_size_in_bytes {\n",
        "        if batch_size_in_bytes < PLANNED_IMPORT_BATCH_BYTES || pending.batch.size_in_bytes() < batch_size_in_bytes {\n")],
    "s2-old-flip": [(
        "        if pending.batch.size_in_bytes() < batch_size_in_bytes {\n",
        "        if pending.batch.size_in_bytes() >= batch_size_in_bytes {\n")],
    # 本腿加的几种错法（T1）
    "push-first": [(
        "            let number = VerdictVectorNumber(\n                u8::try_from(self.verdict_vector_by_number.len())\n                    .expect(\"放进这一批时核过编号装得进一个字节\"),\n            );\n            self.verdict_vector_number_by_content\n                .insert(verdict_vector.0.clone(), number);\n            self.verdict_vector_by_number.push(verdict_vector);\n",
        "            self.verdict_vector_by_number.push(verdict_vector.clone());\n            let number = VerdictVectorNumber(\n                u8::try_from(self.verdict_vector_by_number.len())\n                    .expect(\"放进这一批时核过编号装得进一个字节\"),\n            );\n            self.verdict_vector_number_by_content\n                .insert(verdict_vector.0.clone(), number);\n")],
    "no-push": [(
        "            self.verdict_vector_by_number.push(verdict_vector);\n        }\n        Ok(rows)\n",
        "            let _ = verdict_vector;\n        }\n        Ok(rows)\n")],
    "content-plus1": [(
        "                .insert(verdict_vector.0.clone(), number);\n            self.verdict_vector_by_number.push(verdict_vector);\n",
        "                .insert(verdict_vector.0.clone(), VerdictVectorNumber(number.0 + 1));\n            self.verdict_vector_by_number.push(verdict_vector);\n")],
    "number-once": [(
        "        for verdict_vector in pending.verdict_vectors_registered_in_the_batch {\n            let number = VerdictVectorNumber(\n",
        "        let number_once = VerdictVectorNumber(\n            u8::try_from(self.verdict_vector_by_number.len()).expect(\"x\"),\n        );\n        for verdict_vector in pending.verdict_vectors_registered_in_the_batch {\n            let number = number_once; let _unused = VerdictVectorNumber(\n")],
    "last-batch-only": [(
        "        let full_batch = std::mem::replace(pending, PlannedImportBatch::empty());\n        self.write_the_planned_import_batch(full_batch).map(Some)\n",
        "        let mut full_batch = std::mem::replace(pending, PlannedImportBatch::empty());\n        full_batch.verdict_vectors_registered_in_the_batch.clear();\n        self.write_the_planned_import_batch(full_batch).map(Some)\n")],
    "final-flush-skips-memory": [(
        "        if !pending.batch.is_empty() {\n            tally.count_a_written_batch(self.write_the_planned_import_batch(pending)?);\n",
        "        if !pending.batch.is_empty() {\n            pending.verdict_vectors_registered_in_the_batch.clear();\n            tally.count_a_written_batch(self.write_the_planned_import_batch(pending)?);\n")],
    # T3：第 62 批 Q3 事实行那一行（crates/mutations.tsv 第 3989 行）：核对行点名的事实挪到「本库已有且相同就跳过」之后
    "facts-after": [(
        "            facts_digests_named.insert(facts_digest);\n            let stored_value = self\n                .database\n                .get(&key_bytes.0)\n                .map_err(VerdictStoreError::RocksDatabase)?;\n            if stored_value.as_deref() == Some(value_bytes.as_slice()) {\n                continue;\n            }\n",
        "            let stored_value = self\n                .database\n                .get(&key_bytes.0)\n                .map_err(VerdictStoreError::RocksDatabase)?;\n            if stored_value.as_deref() == Some(value_bytes.as_slice()) {\n                continue;\n            }\n            facts_digests_named.insert(facts_digest);\n")],
    "identity-map": [(
        "                    let this_number =\n                        self.verdict_vector_number_for_the_planned_import(vector, pending)?;\n",
        "                    let this_number = {\n                        let _real = self.verdict_vector_number_for_the_planned_import(vector, pending)?;\n                        *other_number\n                    };\n")],
}


EXTRA_PROBES = r"""
    /// 正推腿加的探针（T4、T6）：产品那一段（crash_amplification.rs 的 `for directory in list.split(\x27:\x27)`）的缩影——同一个库实例按同一份计划
    /// （含路径与核对行）依次从 B、C 两份对方库导入，一趟完了重开库再来第二趟；每份每趟打 written / identical / missing / paths / rows / synced，
    /// 并算三种「imported_blocks」口径：written+identical（产品今天的）、只数 written、数点名到的（含对方库里没有的）。
    pub fn sonnet_t4_t6_two_runs_two_libraries_like_the_product_loop() {
        let b_directory = TemporaryLibraryDirectory::new("sonnet-t4-b");
        let b = build_the_other_library_b_with_a_second_new_verdict_vector(&b_directory.path);
        let c_directory = TemporaryLibraryDirectory::new("sonnet-t4-c");
        let c = build_the_second_other_library_c_with_the_same_new_verdict_vectors(&c_directory.path);
        let local_directory = TemporaryLibraryDirectory::new("sonnet-t4-local");
        let mut local = build_the_local_library_a(&local_directory.path);
        let planned = planned_keys_of_flow_f_beyond_this_host_s_share();
        for run in 1..=3 {
            let (mut sum_product, mut sum_written, mut sum_named) = (0u64, 0u64, 0u64);
            for (name, other) in [("B", &b), ("C", &c)] {
                let tally = local.import_the_planned_rows_from(other, &planned).expect("导入");
                println!(
                    "SONNET_T4 run={run} from={name} written={} identical={} missing={} paths={} verifier={} facts={} rows={} synced={}",
                    tally.conclusion_blocks_written,
                    tally.conclusion_blocks_already_identical,
                    tally.conclusion_blocks_missing_in_the_other_library,
                    tally.paths_written,
                    tally.verifier_blocks_written,
                    tally.flow_facts_written,
                    tally.rows_written,
                    tally.synced_writes
                );
                sum_product += tally.conclusion_blocks_written + tally.conclusion_blocks_already_identical;
                sum_written += tally.conclusion_blocks_written;
                sum_named += tally.conclusion_blocks_written
                    + tally.conclusion_blocks_already_identical
                    + tally.conclusion_blocks_missing_in_the_other_library;
            }
            println!("SONNET_T4 run={run} imported_blocks(written+identical)={sum_product} written_only={sum_written} named_incl_missing={sum_named}");
            drop(local);
            local = VerdictStore::open_existing(&local_directory.path).expect("重开").0;
        }
    }
"""


def apply(text, name):
    for old, new in MUTANTS[name]:
        if text.count(old) != 1:
            print(f"mirror：变异 {name} 的旧串命中 {text.count(old)} 次", file=sys.stderr)
            sys.exit(2)
        text = text.replace(old, new)
    return text


def mirror(text):
    head, sep, tail = text.partition("#[cfg(test)]\nmod tests {")
    assert sep, "认不出 mod tests"
    head = head.replace("    #[cfg(test)]\n    fn every_stored_entry", "    fn every_stored_entry")
    names = re.findall(r"    #\[test\]\n    fn (\w+)\(", tail)
    tail = tail.replace("    #[test]\n    fn ", "    pub fn ")
    tail = re.sub(r"    #\[(should_panic|ignore)[^\n]*\n", "", tail)
    return head + "pub mod mirror_tests {" + tail, names


def same_order_variant(text):
    """加一条 same_order 变体：C 先登记 F2 那一种（rjo）、再登记 F3 那一种（rb3），与本库、B 的编号次序相同。"""
    m = re.search(r"    fn build_the_second_other_library_c_with_the_same_new_verdict_vectors\(.*?\n    \}\n", text, re.S)
    builder = m.group(0)
    swapped = builder.replace("build_the_second_other_library_c_with_the_same_new_verdict_vectors", "build_c_same_order_as_b")
    a = swapped.index("        let red_of_block_three")
    b = swapped.index("        let red_judged_on_the_other_host")
    c = swapped.index("        store_conclusion_in_the_test_library(")
    first, second = swapped[a:b], swapped[b:c]
    swapped = swapped[:a] + second + first + swapped[c:]
    # 块 F3 CPU 的状态顺序也换成 [rjo, rb3, clean]，让两种向量在 C 里的编号 2、3 与块里出现的次序同 B
    swapped = swapped.replace("block_of(&[red_of_block_three, red_judged_on_the_other_host, clean])",
                              "block_of(&[red_judged_on_the_other_host, red_of_block_three, clean])")
    t = re.search(r"    pub fn planned_import_in_one_byte_batches_numbers_new_verdict_vectors_on_across_batches_and_writes_what_one_batch_writes\(\n    \) \{.*?\n    \}\n", text, re.S)
    case = t.group(0)
    case2 = case.replace("planned_import_in_one_byte_batches_numbers_new_verdict_vectors_on_across_batches_and_writes_what_one_batch_writes",
                         "same_order_c_variant_of_the_s1_case")
    case2 = case2.replace("build_the_second_other_library_c_with_the_same_new_verdict_vectors", "build_c_same_order_as_b")
    return text.replace(builder, builder + "\n" + swapped, 1).replace(t.group(0), case + "\n" + case2, 1)


def main():
    if sys.argv[1:] == ["list"]:
        print(" ".join(MUTANTS))
        return 0
    root = Path(sys.argv[1])
    path = root / STORE
    text, names = mirror(path.read_text())
    text = same_order_variant(text)
    names.append("same_order_c_variant_of_the_s1_case")
    text = text.rstrip()
    assert text.endswith("}")
    text = text[:-1] + EXTRA_PROBES + "}\n"
    names.append("sonnet_t4_t6_two_runs_two_libraries_like_the_product_loop")
    for mutant in sys.argv[2:]:
        text = apply(text, mutant)
    path.write_text(text)
    body = "// 生成：mirror.py\nuse singlefs_checker_tier::verdict_store::mirror_tests as m;\n"
    for name in names:
        body += f"#[test]\nfn {name}() {{\n    m::{name}();\n}}\n"
    (root / "crates/sonnet-probe-b63/tests").mkdir(parents=True, exist_ok=True)
    (root / "crates/sonnet-probe-b63/tests/mirror_all.rs").write_text(body)
    print(f"mirror：{len(names)} 条镜像用例，变异 {sys.argv[2:]}")
    return 0


sys.exit(main())

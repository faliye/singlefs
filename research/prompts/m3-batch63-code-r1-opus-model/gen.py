#!/usr/bin/env python3
"""攻方腿（m3-batch63-code-r1 opus）：从仓副本的 crates/singlefs-checker-tier/src/verdict_store.rs 逐字生成原型 crate 的
src/verdict_store_under_test.rs：先按名字打变异（旧串在文件里恰好命中一次，否则退出 2），再在单测模块末尾（文件最后一个「}」之前）接上
(1) 被判 Q3 用例的一份逐字拷贝，只把第二份对方库 C 的构造函数换成「与本库同序」那一个（用例名加前缀 opus_same_order_c_），
(2) attack_tests.rs 里的攻方用例。
用法：gen.py <副本根> [变异名]；gen.py --list。变异名 base 或不给就是原样。"""
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
SOURCE = "crates/singlefs-checker-tier/src/verdict_store.rs"
TARGET = "crates/opus-probe-b63/src/verdict_store_under_test.rs"
LOOP = "        for verdict_vector in pending.verdict_vectors_registered_in_the_batch {"
LOOP_BODY = (
    "        for verdict_vector in pending.verdict_vectors_registered_in_the_batch {\n"
    "            let number = VerdictVectorNumber(\n"
    "                u8::try_from(self.verdict_vector_by_number.len())\n"
    "                    .expect(\"放进这一批时核过编号装得进一个字节\"),\n"
    "            );\n"
    "            self.verdict_vector_number_by_content\n"
    "                .insert(verdict_vector.0.clone(), number);\n"
    "            self.verdict_vector_by_number.push(verdict_vector);\n"
    "        }\n"
)
WRITE_THEN_LOOP = (
    "        self.database\n"
    "            .write_opt(pending.batch, &synced_write_options())\n"
    "            .map_err(VerdictStoreError::RocksDatabase)?;\n" + LOOP_BODY
)
JUDGE = "        if pending.batch.size_in_bytes() < batch_size_in_bytes {"

MUTANTS = {
    # crates/mutations.tsv 里第 63 批 S1 三行（原文、替换文照 research/prompts/_m3-batch63-code-r1-diff.md 抄）
    "take1": [(LOOP, "        for verdict_vector in pending.verdict_vectors_registered_in_the_batch.into_iter().take(1) {")],
    "rev": [(LOOP, "        for verdict_vector in pending.verdict_vectors_registered_in_the_batch.into_iter().rev() {")],
    "noinsert": [("            self.verdict_vector_number_by_content\n                .insert(verdict_vector.0.clone(), number);\n            self.verdict_vector_by_number.push(verdict_vector);",
                  "            let _ = number;\n            self.verdict_vector_by_number.push(verdict_vector);")],
    # S2 那一行（第 62 批 Q3 换批判据，第 63 批整行替换后的替换文）与原来的「批长判据反过来」（替换文 >=）
    "s2": [(JUDGE, "        if batch_size_in_bytes < PLANNED_IMPORT_BATCH_BYTES || pending.batch.size_in_bytes() < batch_size_in_bytes {")],
    "flip": [(JUDGE, "        if pending.batch.size_in_bytes() >= batch_size_in_bytes {")],
    # 表里第 62 批 Q3 另两行：一种都不进内存表；核对行点名事实挪到「已有且相同就跳过」之后
    "take0": [(LOOP, "        for verdict_vector in pending.verdict_vectors_registered_in_the_batch.into_iter().take(0) {")],
    "factsafter": [(
        "            facts_digests_named.insert(facts_digest);\n            let stored_value = self\n                .database\n                .get(&key_bytes.0)\n                .map_err(VerdictStoreError::RocksDatabase)?;\n            if stored_value.as_deref() == Some(value_bytes.as_slice()) {\n                continue;\n            }\n",
        "            let stored_value = self\n                .database\n                .get(&key_bytes.0)\n                .map_err(VerdictStoreError::RocksDatabase)?;\n            if stored_value.as_deref() == Some(value_bytes.as_slice()) {\n                continue;\n            }\n            facts_digests_named.insert(facts_digest);\n")],
    # 攻方邻居（T1：第 2069–2076 行的别的错法）
    "off1": [("                u8::try_from(self.verdict_vector_by_number.len())\n                    .expect(\"放进这一批时核过编号装得进一个字节\"),\n            );\n            self.verdict_vector_number_by_content",
              "                u8::try_from(self.verdict_vector_by_number.len() + 1)\n                    .expect(\"放进这一批时核过编号装得进一个字节\"),\n            );\n            self.verdict_vector_number_by_content")],
    "pushfirst": [(LOOP_BODY,
        "        for verdict_vector in pending.verdict_vectors_registered_in_the_batch {\n"
        "            self.verdict_vector_by_number.push(verdict_vector.clone());\n"
        "            let number = VerdictVectorNumber(\n"
        "                u8::try_from(self.verdict_vector_by_number.len())\n"
        "                    .expect(\"放进这一批时核过编号装得进一个字节\"),\n"
        "            );\n"
        "            self.verdict_vector_number_by_content\n"
        "                .insert(verdict_vector.0.clone(), number);\n"
        "        }\n")],
    "hoist": [(LOOP_BODY,
        "        let number = VerdictVectorNumber(\n"
        "            u8::try_from(self.verdict_vector_by_number.len())\n"
        "                .expect(\"放进这一批时核过编号装得进一个字节\"),\n"
        "        );\n"
        "        for verdict_vector in pending.verdict_vectors_registered_in_the_batch {\n"
        "            self.verdict_vector_number_by_content\n"
        "                .insert(verdict_vector.0.clone(), number);\n"
        "            self.verdict_vector_by_number.push(verdict_vector);\n"
        "        }\n")],
    "index": [(LOOP_BODY,
        "        for (index, verdict_vector) in pending.verdict_vectors_registered_in_the_batch.into_iter().enumerate() {\n"
        "            let number = VerdictVectorNumber(\n"
        "                u8::try_from(index)\n"
        "                    .expect(\"放进这一批时核过编号装得进一个字节\"),\n"
        "            );\n"
        "            self.verdict_vector_number_by_content\n"
        "                .insert(verdict_vector.0.clone(), number);\n"
        "            self.verdict_vector_by_number.push(verdict_vector);\n"
        "        }\n")],
    "onlyinsert": [("            self.verdict_vector_number_by_content\n                .insert(verdict_vector.0.clone(), number);\n            self.verdict_vector_by_number.push(verdict_vector);",
                    "            self.verdict_vector_number_by_content\n                .insert(verdict_vector.0.clone(), number);\n            let _ = verdict_vector;")],
    "take2": [(LOOP, "        for verdict_vector in pending.verdict_vectors_registered_in_the_batch.into_iter().take(2) {")],
    # 内存表先于写盘改（写成功时与原样相同；写失败时内存表多了盘上没有的编号）
    "eager": [(WRITE_THEN_LOOP, LOOP_BODY +
        "        self.database\n"
        "            .write_opt(pending.batch, &synced_write_options())\n"
        "            .map_err(VerdictStoreError::RocksDatabase)?;\n")],
    # 第 2069–2076 行之外（T1 问 C 的次序分辨什么）：换编号时照抄对方库的编号（新向量照样登记）
    "raw": [("            this_numbers.push(this_number);\n",
             "            let _ = this_number;\n            this_numbers.push(*other_number);\n")],
    # T1「只在最后一批更新」：攒满写下的那几批不更新内存表，把这一批登记的向量挪进下一批，到收尾那一批写下时一起进内存表
    # （编号照样对：下一批按批里的位置接着编）；收尾那一批是空的（最后一组恰好攒满写下）时一种都不进
    "carry": [("        let full_batch = std::mem::replace(pending, PlannedImportBatch::empty());\n        self.write_the_planned_import_batch(full_batch).map(Some)\n",
               "        let mut full_batch = std::mem::replace(pending, PlannedImportBatch::empty());\n        pending.verdict_vectors_registered_in_the_batch =\n            std::mem::take(&mut full_batch.verdict_vectors_registered_in_the_batch);\n        self.write_the_planned_import_batch(full_batch).map(Some)\n")],
    # T3：按计划导入一行事实都不导
    "nofacts": [("        for facts_digest in &facts_digests_named {", "        for facts_digest in facts_digests_named.iter().take(0) {")],
}

Q3_START = "    #[test]\n    fn planned_import_in_one_byte_batches_numbers_new_verdict_vectors_on_across_batches_and_writes_what_one_batch_writes(\n"
Q3_END = "\n    /// 事实行补写（正推 B2）"
C_BUILDER_CALL = "build_the_second_other_library_c_with_the_same_new_verdict_vectors(\n"


def main():
    if sys.argv[1:] == ["--list"]:
        print("\n".join(["base", *MUTANTS]))
        return 0
    root = Path(sys.argv[1])
    name = sys.argv[2] if len(sys.argv) > 2 else "base"
    text = (root / SOURCE).read_text()
    # 先从原样里截出被判 Q3 用例，再打变异（同序 C 那一份照原样的用例文本，变异打在被测代码上，两份用例一起受它）
    start = text.index(Q3_START)
    end = text.index(Q3_END, start)
    q3 = text[start:end]
    if q3.count(C_BUILDER_CALL) != 1:
        print(f"gen：Q3 用例里 C 的构造调用命中 {q3.count(C_BUILDER_CALL)} 次", file=sys.stderr)
        return 2
    same_order = q3.replace(
        "    fn planned_import_in_one_byte_batches_numbers_new_verdict_vectors_on_across_batches_and_writes_what_one_batch_writes(",
        "    fn opus_same_order_c_planned_import_in_one_byte_batches_numbers_new_verdict_vectors_on_across_batches_and_writes_what_one_batch_writes(",
    ).replace(C_BUILDER_CALL, "opus_build_c_in_the_local_order(\n")
    if name != "base":
        for old, new in MUTANTS[name]:
            if text.count(old) != 1:
                print(f"gen：变异 {name} 的旧串命中 {text.count(old)} 次", file=sys.stderr)
                return 2
            text = text.replace(old, new)
    if not text.endswith("}\n"):
        print("gen：文件不以单测模块的「}」收尾", file=sys.stderr)
        return 2
    attack = (HERE / "attack_tests.rs").read_text()
    text = text[: -len("}\n")] + "\n" + same_order + "\n" + attack + "}\n"
    target = root / TARGET
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(text)
    print(f"gen：{TARGET} 写好（变异 {name}）")
    return 0


sys.exit(main())

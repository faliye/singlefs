#!/usr/bin/env python3
"""gen_alt4.py：第一个文件之外的文件照抄单元时每个单元现建一份上一版 units 索引（记进没点名的那一格，不走索引遍历），整份用例文件都跑。写 alt4-mutations.tsv。"""
old = """                | FileOtherThanTheFirstOrigin::CutByADeletionBatch => carried_unit(
                    units_of_the_previous_version
                        .as_ref()
                        .expect("照抄的文件只出现在接着上一版的发布里"),
                    identity,
                ),"""
new = """                | FileOtherThanTheFirstOrigin::CutByADeletionBatch => carried_unit(
                    &previous
                        .expect("照抄的文件只出现在接着上一版的发布里")
                        .units_by_role(),
                    identity,
                ),"""
args = "-p singlefs-harness --test publish_lookup_by_identity"
with open("alt4-mutations.tsv", "w", encoding="utf-8") as out:
    out.write("\t".join(["ALT-None 照抄第一个文件之外的文件的单元：每个单元现建一份上一版units索引（记进没点名的那一格），整份用例文件都跑", "crates/singlefs-core/src/transaction.rs", old.replace("\n", "\\n"), new.replace("\n", "\\n"), args, "publish_lookup_by_identity（全部）"]) + "\n")

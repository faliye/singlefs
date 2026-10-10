#!/usr/bin/env python3
"""对副本里的 walk.rs 临时施加 / 还原一个命名的探针补丁（只在自己草稿目录的副本里）。
用法 patch_walk.py <副本根> <冻结副本根> apply <名字> | restore
名字：q1a（由记录施加出来的那一版也照 core 跟全零树表指针）、q2（跟过去时另判 I-2.5）、q4（I-5.5 认出来而没有数据单元写另一句理由）、
      rvoff（I-5.4 不收两类由记录造出来的版本，同 mutations.tsv 的 take(0) 行）、instrument（RV 的版本数打到 stderr，设 PROBE_RV=1 才打）"""
import hashlib, shutil, sys
root, frozen, action = sys.argv[1], sys.argv[2], sys.argv[3]
rel = "crates/singlefs-checker/src/walk.rs"
path, original_path = f"{root}/{rel}", f"{frozen}/{rel}"
if action == "restore":
    shutil.copyfile(original_path, path)
    same = hashlib.sha256(open(path, "rb").read()).hexdigest() == hashlib.sha256(open(original_path, "rb").read()).hexdigest()
    print("restore", "ok" if same else "MISMATCH"); sys.exit(0 if same else 1)
name = sys.argv[4]
s = open(path, encoding="utf-8").read()
def once(old, new):
    global s
    assert s.count(old) == 1, (name, s.count(old))
    s = s.replace(old, new)
if name == "q1a":
    once("            &version_name,\n            AllZeroTreeTablePointerReading::NoTreeTable,\n        );",
         "            &version_name,\n            AllZeroTreeTablePointerReading::FollowedLikeTheRootRecordReaderOfTheCore,\n        );")
elif name == "q2":
    once("        self.walk_failures.push(match unit {\n            None => format!(\"{what} 两份都读不到对得上的\"),",
         "        crate::image::judge_location_entries_order(&mut self.judgements, &tree_table_pointer.locations, &what);\n        self.walk_failures.push(match unit {\n            None => format!(\"{what} 两份都读不到对得上的\"),")
elif name == "q4":
    once("        judgements.not_applicable(invariant, NO_VERSION_BUILDS_TREES_FROM_THE_MKFS_WATERMARK);\n    }",
         "        judgements.not_applicable(invariant, NO_VERSION_BUILDS_TREES_FROM_THE_MKFS_WATERMARK);\n    } else {\n        judgements.not_applicable(invariant, \"Q4 候选：认出了从 mkfs 水位第一次建树的那一版，但它们没有数据单元：一版都不判\");\n    }")
elif name == "rvoff":
    a = "            .chain(versions_applied_only_by_records.iter().map(|version| {\n                VersionToJudgeTheAllocationRecordsOf {"
    b = "            .chain(version_the_next_mount_applies_first.iter().map(|version| {"
    once(a, a.replace(".iter().map", ".iter().take(0).map")); once(b, b.replace(".iter().map", ".iter().take(0).map"))
elif name == "instrument":
    once("            .collect();\n    judge_allocation_records_disjoint(\n        reader,\n        &versions_to_judge_the_allocation_records_of,",
         "            .collect();\n    if std::env::var(\"PROBE_RV\").is_ok() {\n        eprintln!(\"RVVERSIONS candidates={} applied_only_by_records={} next_mount_first={}\", candidate_indexes.len(), versions_applied_only_by_records.len(), version_the_next_mount_applies_first.is_some());\n    }\n    judge_allocation_records_disjoint(\n        reader,\n        &versions_to_judge_the_allocation_records_of,")
else:
    sys.exit("名字不认识")
open(path, "w", encoding="utf-8").write(s); print("applied", name)

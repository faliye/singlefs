#!/usr/bin/env python3
"""m3-batch23-code-r1 攻方改法 A（零轮，只在仓副本里）：I-9.29 那一判「滤没滤全」改按主走读沿最新根的实例表链走到了末片
（链指针记录读成「没有下一片」）来判，不再另用只读不判那个读法。先跑 apply_capture_hook.py 再跑它。
用法：apply_fix_a.py <副本里的 crates/singlefs-checker/src/walk.rs>；每处锚点要恰好命中一次，不然退 1。"""
import sys

path = sys.argv[1]
text = open(path, encoding="utf-8").read()

def once(anchor, replacement):
    global text
    count = text.count(anchor)
    if count != 1:
        sys.exit(f"锚点命中 {count} 次：{anchor[:80]!r}")
    text = text.replace(anchor, replacement)

once(
    "    instance_table_rows: Vec<InstanceTableRow>,\n",
    "    instance_table_rows: Vec<InstanceTableRow>,\n"
    "    /// 改法 A：主走读沿最新根的实例表链走到了末片（链指针记录读成没有下一片），`instance_table_rows` 是整张表的行。\n"
    "    newest_instance_table_read_to_its_last_page: bool,\n",
)
once(
    "            instance_table_rows: Vec::new(),\n",
    "            instance_table_rows: Vec::new(),\n            newest_instance_table_read_to_its_last_page: false,\n",
)
once(
    "                ChainRecordView::LastPage => break,\n",
    "                ChainRecordView::LastPage => {\n"
    "                    if mount_root_instance_when_newest.is_some() {\n"
    "                        self.newest_instance_table_read_to_its_last_page = true;\n"
    "                    }\n"
    "                    break;\n"
    "                }\n",
)
once(
    "    let abandoned_roots_filtering = match instance_table_rows_of_root_without_judging(\n"
    "        reader,\n"
    "        &roots[newest_index].2.record_bytes,\n"
    "    ) {\n"
    "        Some(_) => AbandonedRootsFilteringOfTheCandidateSet::FilteredByTheNewestRootsInstanceTable,\n"
    "        None => AbandonedRootsFilteringOfTheCandidateSet::UnfilteredAsTheNewestRootsInstanceTableDoesNotReadBack,\n"
    "    };\n",
    "    let abandoned_roots_filtering = if walk.newest_instance_table_read_to_its_last_page {\n"
    "        AbandonedRootsFilteringOfTheCandidateSet::FilteredByTheNewestRootsInstanceTable\n"
    "    } else {\n"
    "        AbandonedRootsFilteringOfTheCandidateSet::UnfilteredAsTheNewestRootsInstanceTableDoesNotReadBack\n"
    "    };\n",
)
open(path, "w", encoding="utf-8").write(text)
print("改法 A 已加")

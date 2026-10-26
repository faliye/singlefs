#!/usr/bin/env python3
"""把冻结副本拷贝（已打改法甲）里的两份产品文件按名字改成一种错法，写进变异树。用法：mutate.py <基准树> <变异树> <名字>"""
import sys, shutil, pathlib

base, mutant, name = map(pathlib.Path, sys.argv[1:3] + [pathlib.Path(sys.argv[3])]) if False else (pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2]), sys.argv[3])
ext = "crates/singlefs-core/src/extent_tree.rs"
txn = "crates/singlefs-core/src/transaction.rs"

CHANGES = {
    # 行键 5510 那一条：size 0 取 None
    "size-zero-none": (ext,
        "            Some(_) => Some(ExtentUpperLeafTarget::NoDataUnit),\n",
        "            Some(0) => None,\n            Some(_) => Some(ExtentUpperLeafTarget::NoDataUnit),\n"),
    # 只在 size 0 时写，size > 0 不写
    "only-size-zero": (ext,
        "            Some(_) => Some(ExtentUpperLeafTarget::NoDataUnit),\n",
        "            Some(0) => Some(ExtentUpperLeafTarget::NoDataUnit),\n            Some(_) => None,\n"),
    # 只有算有条目号集合的那个调用方丢 size 0
    "set-caller-drops-size-zero": (txn,
        "                self.inode_record.map(|record| record.size),\n",
        "                self.inode_record.map(|record| record.size).filter(|size| *size > 0),\n"),
    # 只有装上段叶的那个调用方丢 size 0
    "build-caller-drops-size-zero": (txn,
        "                        size_of_the_first_file_after_this_publish,\n                    ),\n                }\n                .map(|target| target.with_the_tail(tail_of_the_first_file));",
        "                        size_of_the_first_file_after_this_publish.filter(|size| *size > 0),\n                    ),\n                }\n                .map(|target| target.with_the_tail(tail_of_the_first_file));"),
    # 记录没了也写（unlink 最后一批之后留孤条目）
    "write-without-record": (ext,
        "            None => None,\n        }\n    }\n\n    /// 这个标签声明的尾部",
        "            None => Some(ExtentUpperLeafTarget::NoDataUnit),\n        }\n    }\n\n    /// 这个标签声明的尾部"),
    # 行键 5144 那一条
    "row5144-arm-none": (txn,
        "                    (None, None) => ExtentUpperLeafTarget::of_a_file_without_data_units(\n                        size_of_the_first_file_after_this_publish,\n                    ),",
        "                    (None, None) => None,"),
    # 行键 5145 那一条
    "row5145-set-filter-zero": (txn,
        "                self.inode_record.map(|record| record.size),\n",
        "                self.inode_record.map(|record| record.size).filter(|size| *size == 0),\n"),
    # 算集合的调用方不看记录，一律算第一个文件
    "set-caller-always-first": (txn,
        "                self.inode_record.map(|record| record.size),\n",
        "                Some(0),\n"),
    # 装叶的调用方不看记录，一律写
    "build-caller-always-write": (txn,
        "                        size_of_the_first_file_after_this_publish,\n                    ),\n                }\n                .map(|target| target.with_the_tail(tail_of_the_first_file));",
        "                        Some(0),\n                    ),\n                }\n                .map(|target| target.with_the_tail(tail_of_the_first_file));"),
}

if name not in CHANGES:
    sys.exit(f"没有这个名字：{name}；有 {sorted(CHANGES)}")
relative, old, new = CHANGES[name]
for path in (ext, txn):
    shutil.copyfile(base / path, mutant / path)
text = (mutant / relative).read_text()
if text.count(old) != 1:
    sys.exit(f"旧串在 {relative} 里命中 {text.count(old)} 次，不是恰好 1 次")
(mutant / relative).write_text(text.replace(old, new))
print(f"mutant={name} file={relative}")

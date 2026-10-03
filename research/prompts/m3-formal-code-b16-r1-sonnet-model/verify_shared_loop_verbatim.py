#!/usr/bin/env python3
"""核 16-2 抽出的共用函数里的循环与抬 F 原来那一段逐字相同。

做法：把背景里 model.rs 的 diff 反向套到冻结副本的 model.rs 上得到合入前的文件，取合入前
`answer_raise_rollback_floor` 里从 `let mut roots: Vec<ModelRoot> = Vec::new();` 起到循环体收尾那一行的原文，
与冻结副本里 `empty_publishes_carrying_the_floor_to_every_device` 同一段比，只许差新加的那一行注释。
用法：verify_shared_loop_verbatim.py <冻结副本 model.rs> <model.rs 的 diff 文件> <草稿目录>
"""
import subprocess
import sys

new_path, diff_path, scratch = sys.argv[1:4]
old_path = scratch + "/model.rs.old"
open(old_path, "w", encoding="utf-8").write(open(new_path, encoding="utf-8").read())
subprocess.run(["patch", "-R", "-s", old_path, diff_path], check=True)


def section(path, function_marker):
    lines = open(path, encoding="utf-8").read().split("\n")
    start_function = next(i for i, line in enumerate(lines) if function_marker in line)
    start = next(i for i in range(start_function, len(lines)) if "let mut roots: Vec<ModelRoot> = Vec::new();" in lines[i])
    end = next(i for i in range(start, len(lines)) if lines[i] == "        }" and "roots.push(self.next_root(" in "\n".join(lines[start:i]))
    return lines[start : end + 1]


old = section(old_path, "pub fn answer_raise_rollback_floor(")
new = section(new_path, "fn empty_publishes_carrying_the_floor_to_every_device(")
added = [line for line in new if line not in old]
removed = [line for line in old if line not in new]
print("合入前循环段行数", len(old), "；合入后", len(new))
print("合入后多出的行：", added)
print("合入前被去掉的行：", removed)
without_comment = [line for line in new if "迭代上界是根环区域数 R" not in line]
print("去掉新加注释后逐行相同：", without_comment == old)

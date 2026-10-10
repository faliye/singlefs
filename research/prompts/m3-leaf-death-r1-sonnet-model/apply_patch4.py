#!/usr/bin/env python3
"""第四层补丁（接在 apply_patch3.py 之后）：候选 retire_shrink = retire（容器身份消失写退役记录）加 shrink_range（文件变短写区间墓碑）。
用法：apply_patch4.py <第三层输出.rs> <输出.rs>"""
import sys
source = open(sys.argv[1], encoding="utf-8").read()

def replace_once(text, old, new):
    count = text.count(old)
    if count != 1:
        sys.exit(f"旧串出现 {count} 次，不是 1 次：{old[:80]!r}")
    return text.replace(old, new)

source = replace_once(source, "    ShrinkRange,\n}\n", "    ShrinkRange,\n    RetireAndShrinkRange,\n}\n")
source = replace_once(source, '        Ok("shrink_range") => Candidate::ShrinkRange,\n', '        Ok("shrink_range") => Candidate::ShrinkRange,\n        Ok("retire_shrink") => Candidate::RetireAndShrinkRange,\n')
source = replace_once(source, "    if candidate() == Candidate::RetireRecord {\n        RETIRED_IDENTITIES.with(|retired| {", "    if matches!(candidate(), Candidate::RetireRecord | Candidate::RetireAndShrinkRange) {\n        RETIRED_IDENTITIES.with(|retired| {")
source = replace_once(source, "            Candidate::RetireRecord => plan.retired_containers.push(", "            Candidate::RetireRecord | Candidate::RetireAndShrinkRange => plan.retired_containers.push(")
source = replace_once(source, "    if candidate() == Candidate::ShrinkRange && length_as_u64(previous.file_units.len()) > data_units {", "    if matches!(candidate(), Candidate::ShrinkRange | Candidate::RetireAndShrinkRange) && length_as_u64(previous.file_units.len()) > data_units {")
source = replace_once(source, "            if candidate() == Candidate::ShrinkRange && arm == Arm::TodaysTombstones {", "            if matches!(candidate(), Candidate::ShrinkRange | Candidate::RetireAndShrinkRange) && arm == Arm::TodaysTombstones {")
open(sys.argv[2], "w", encoding="utf-8").write(source)
print("ok")

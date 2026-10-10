#!/usr/bin/env python3
"""第五层补丁（接在 apply_patch4.py 之后）：环境变量 E177_TIGHT=literal 时 G-紧 的盘按 G-宽 下「这一版的活槽」峰值定（登记 5.1 字面读法），
不再按「活槽 + 这次换下的」峰值定（执行员修订 2 的 C17）。用法：apply_patch5.py <第四层输出.rs> <输出.rs>"""
import sys
source = open(sys.argv[1], encoding="utf-8").read()
old = "            let slots = tight_unit_area_slots_from(wide_summary.slots_held_during_a_publish_peak);\n"
new = ("            let slots = tight_unit_area_slots_from(if std::env::var(\"E177_TIGHT\").as_deref() == Ok(\"literal\") {\n"
       "                wide_summary.live_slot_peak\n            } else {\n                wide_summary.slots_held_during_a_publish_peak\n            });\n")
if source.count(old) != 1:
    sys.exit(f"旧串出现 {source.count(old)} 次，不是 1 次")
open(sys.argv[2], "w", encoding="utf-8").write(source.replace(old, new))
print("ok")

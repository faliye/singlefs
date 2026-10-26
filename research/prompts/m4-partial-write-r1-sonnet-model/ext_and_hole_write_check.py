#!/usr/bin/env python3
"""补充运行：只留今天的入口加「变长」与「洞里的部分写」（不含对已有单元的部分写 pw），今天的三条读法 Z 规则认不认得回活单元。
用法：python3 -I ext_and_hole_write_check.py [深度]（与 scan_rule_model.py 放同一目录）"""
import os
import sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import scan_rule_model as model

depth = int(sys.argv[1]) if len(sys.argv) > 1 else 10
full_ops = model.ops_list
model.ops_list = lambda: [op for op in full_ops() if op[0] != "pw"]
cfg = dict(whole_field="clear", pw="only")
rule = dict(order=True, field=True, tomb=False)
states, bad = model.explore(cfg, rule, depth)
print(f"name=today_rule_with_ext_and_hole_writes_only depth={depth} states={states} mismatches={len(bad)}")
cfg_without_hole = dict(whole_field="clear", pw="only")
model.ops_list = lambda: [op for op in full_ops() if op[0] not in ("pw", "phole")]
states, bad = model.explore(cfg_without_hole, rule, depth)
print(f"name=today_rule_without_any_partial_write depth={depth} states={states} mismatches={len(bad)}")

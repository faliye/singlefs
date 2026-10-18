"""列出第二轮全量跑的件：每行「候选 前缀 起点 终点 M2 读故障」。用法：python3 pieces-r2.py <每件状态数>"""
import sys
per_piece = int(sys.argv[1])
candidates = ["k6", "pointer_slots_fixed", "later_root_or_record"]
jobs = [("clean", 20511, "y2"), ("tree_table_and_record", 20516, "y2"),
        ("tree_table_and_record", 20516, "tree_and_mapping"), ("root_units_and_record", 20516, "none")]
for candidate in candidates:
    for prefix, count, faults in jobs:
        for start in range(0, count, per_piece):
            print(candidate, prefix, start, min(start + per_piece, count), faults)

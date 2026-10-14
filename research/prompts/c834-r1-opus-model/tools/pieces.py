"""列出全量跑的件：每行「候选 前缀 起点 终点」。用法：python3 pieces.py <每件状态数>"""
import sys
per_piece = int(sys.argv[1])
states = {"clean": 20511, "tree_table_and_record": 20516}
for candidate in ["today", "largest", "smallest", "later_root_or_record", "pointer_slots_overwritten", "always_refuse"]:
    for prefix, count in states.items():
        for start in range(0, count, per_piece):
            print(candidate, prefix, start, min(start + per_piece, count))

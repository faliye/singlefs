"""mapping_root_only 只跑头尾两段（序号 0–20 与 16000 起）：每行「候选 前缀 起点 终点 M2 读故障」。"""
for prefix, count, faults in [("clean", 20511, "y2"), ("tree_table_and_record", 20516, "y2"), ("tree_table_and_record", 20516, "tree_and_mapping"), ("root_units_and_record", 20516, "none")]:
    print("mapping_root_only", prefix, 0, 20, faults)
    print("mapping_root_only", prefix, 16000, count, faults)

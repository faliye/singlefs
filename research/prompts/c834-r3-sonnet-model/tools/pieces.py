"""列这一轮复跑的件：python3 pieces.py opus|sonnet <每件状态数>。
opus 每行「历史 起点 终点 读故障」，sonnet 每行「候选 前缀 起点 终点 读故障」。件只含正文点名的格：
丙、丁、丁加 y2、丁只藏记录、三次挂载（opus）；映射根变体与指针槽读法各在四个前缀上全量（sonnet）。"""
import sys
which, per_piece = sys.argv[1], int(sys.argv[2])
if which == "opus":
    cells = [("tree_table_and_record", 20516, "tree_and_mapping"), ("root_units_and_record", 20516, "none"),
             ("root_units_and_record", 20516, "y2"), ("root_units_and_record", 20516, "record_only"),
             ("three_mount_tail", 8227, "y2"), ("three_mount_tail", 8227, "y2_m0_record")]
    for history, count, faults in cells:
        for start in range(0, count, per_piece):
            print(history, start, min(start + per_piece, count), faults)
else:
    # 映射根变体与指针槽读法各在四个前缀上全量（序号 0 到 count）：映射根变体补第二轮只跑头尾两段的缺（乙那一类的中间约 16000 个状态），
    # 指针槽读法对第二轮的全量。第一遍只起过映射根变体的头尾两段加乙中间（90 件）核对过流程，结果和这一份全量一致，不另入库。
    prefixes = [("tree_table_and_record", 20516, "tree_and_mapping"), ("root_units_and_record", 20516, "none"),
                ("clean", 20511, "y2"), ("tree_table_and_record", 20516, "y2")]
    for candidate in ("mapping_root_only", "pointer_slots_fixed"):
        for prefix, count, faults in prefixes:
            for start in range(0, count, per_piece):
                print(candidate, prefix, start, min(start + per_piece, count), faults)

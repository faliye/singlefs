#!/usr/bin/env python3
"""E166：六条臂 × 三个 Q1 候选 × Q-1..Q-9 的判决行汇成一张表。Q-1..Q-7 = 资源重复数（登记：U 用户数据落点、G 提交内生块落点、
I inode 号、T 树 ID 水位、X 事务号、J/JS journal 写指针、TXG/RS 根环槽按行序）；Q-8 = 冲突重算；Q-9 = 同步轮次。只读产物。"""
from collections import defaultdict
from a_common import E166, rows

ARMS = ("A", "A字面", "P区", "P设", "O整", "O读")
STAGES = ("Q1甲", "Q1乙", "Q1丙")


def main():
    verdicts = [(n, f) for n, f in rows(E166, "name=verdict")]
    table = defaultdict(dict)
    for _, f in verdicts:
        if f.get("question", "").startswith("Q-"):
            table[(f["arm"], f["stage"])][f["question"]] = f
    print("重复数合计（duplicate_total，Q-1..Q-7）按 臂 × Q1 候选；非零用 ! 标")
    print("  臂      候选   " + "  ".join(f"Q-{i}" for i in range(1, 8)))
    for arm in ARMS:
        for stage in STAGES:
            cells = table[(arm, stage)]
            out = []
            for i in range(1, 8):
                row = cells.get(f"Q-{i}")
                total = row.get("duplicate_total", "-") if row else "-"
                out.append(("!" if total not in ("0", "-") else " ") + total)
            print(f"  {arm:6s} {stage}  " + "  ".join(out))
    print("\n冲突重算（Q-8 retries_total、有重算的格数、judgement）：")
    for arm in ARMS:
        for stage in STAGES:
            row = table[(arm, stage)].get("Q-8")
            if row:
                print(f"  {arm:6s} {stage} retries_total={row['retries_total']} cells_with_retries={row['cells_with_retries']} "
                      f"device_truncated={row.get('device_truncated')} judgement={row['judgement']}")
    print("\n同步轮次（Q-9：每格每轮提交的事务数；cells_with_fewer_rounds_than_serial 条格里比串行少）：")
    for arm in ARMS:
        for stage in STAGES:
            row = table[(arm, stage)].get("Q-9")
            if row:
                print(f"  {arm:6s} {stage} fewer={row.get('cells_with_fewer_rounds_than_serial')} "
                      f"equal_every_cell={row.get('rounds_equal_serial_in_every_cell')} judgement={row['judgement']}")
    print("\n失败条款行：")
    for number, f in verdicts:
        if "failure" in f:
            print(f"  第 {number} 行 failure={f['failure']} clause={f['clause']} triggered={f['triggered']}")
    anchors = [f for _, f in rows(E166, "name=anchor")]
    print(f"\n核：anchor 行 {len(anchors)} 条，ok=true {sum(1 for a in anchors if a.get('ok') == 'true')} 条；"
          f"cell 行 {len(rows(E166, 'name=cell'))}、control_cell 行 {len(rows(E166, 'name=control_cell'))}")


if __name__ == "__main__":
    main()
